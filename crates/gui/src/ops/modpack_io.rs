use std::{collections::HashMap, io::Write, path::Path};

use anyhow::Context;
use reqwest::Client;
use serde_json::json;
use sha2::{Digest, Sha512};
use sqrilizz::{
	config::modifications::{ConfigModification, apply_modifications_and_write},
	config_crate::instance::{InstanceConfig, make_valid_instance_id},
	instance_crate::addon::modpack::mrpack::{
		ModrinthFileEnv, ModrinthHashes, ModrinthIndex, ModrinthPackDependencies,
		ModrinthPackFile, SideSupport,
	},
	shared::{
		Side,
		id::InstanceID,
		output::NoOp,
		pkg::ArcPkgReq,
		versions::MinecraftVersionDeser,
	},
};
use zip::{ZipArchive, ZipWriter, write::SimpleFileOptions};

use crate::{
	dependency::BackDependency,
	ops::packages::PkgInfo,
	pages::config::content::{ContentItem, ContentItemType},
	state::BackState,
};

/// Result of checking updates for installed mods
#[derive(Clone, Debug, PartialEq)]
pub struct ModUpdateInfo {
	pub current_version: Option<String>,
	pub new_version: String,
}

/// Checks Modrinth for available updates for configured packages and local addons
pub async fn check_mod_updates(
	items: &[ContentItem],
	packages_info: &HashMap<ArcPkgReq, anyhow::Result<PkgInfo>>,
	mc_version: Option<&str>,
	loader: Option<&str>,
) -> HashMap<String, String> {
	let client = Client::builder()
		.user_agent("SqrilizzLauncher/1.0")
		.build()
		.unwrap_or_default();

	let mut updates = HashMap::new();
	let mut hash_to_item_id = HashMap::new();
	let mut hashes_to_check = Vec::new();

	// 1. Gather hashes from local addons
	for item in items {
		if let Some(path) = &item.addon_path {
			if let Ok(bytes) = std::fs::read(path) {
				let mut hasher = Sha512::new();
				hasher.update(&bytes);
				let hash = format!("{:x}", hasher.finalize());
				hash_to_item_id.insert(hash.clone(), item.id.to_string());
				hashes_to_check.push(hash);
			}
		}
	}

	// 2. Check hashes with Modrinth /v2/version_files/update
	if !hashes_to_check.is_empty() {
		let mut body = json!({
			"hashes": hashes_to_check,
			"algorithm": "sha512"
		});

		if let Some(loader_name) = loader {
			body["loaders"] = json!([loader_name.to_lowercase()]);
		}
		if let Some(mc) = mc_version {
			body["game_versions"] = json!([mc]);
		}

		if let Ok(resp) = client
			.post("https://api.modrinth.com/v2/version_files/update")
			.json(&body)
			.send()
			.await
		{
			if let Ok(results) = resp.json::<HashMap<String, serde_json::Value>>().await {
				for (hash, ver) in results {
					if let Some(item_id) = hash_to_item_id.get(&hash) {
						if let Some(ver_num) = ver.get("version_number").and_then(|v| v.as_str()) {
							updates.insert(item_id.clone(), ver_num.to_string());
						}
					}
				}
			}
		}
	}

	// 3. Check configured packages against Modrinth API
	let mut join_set = tokio::task::JoinSet::new();

	for item in items {
		if updates.contains_key(&item.id.to_string()) {
			continue;
		}

		let req = match &item.ty {
			ContentItemType::Package { req } => Some(req.clone()),
			_ => None,
		};

		if let Some(req) = req {
			let client = client.clone();
			let item_id = item.id.to_string();
			let locked_ver = item.locked_version.clone();
			let mc = mc_version.map(|s| s.to_string());
			let ld = loader.map(|s| s.to_lowercase());

			// Resolve Modrinth project ID or slug
			let proj_id = if let Some(info) = packages_info.get(&req).and_then(|x| x.as_ref().ok())
			 {
				info.meta.name.clone().unwrap_or_else(|| req.id.to_string())
			} else {
				req.id
					.strip_prefix("modrinth:")
					.unwrap_or(&req.id)
					.to_string()
			};

			join_set.spawn(async move {
				let url = format!("https://api.modrinth.com/v2/project/{}/version", proj_id);
				let mut query = Vec::new();

				if let Some(ld) = &ld {
					if let Ok(json_ld) = serde_json::to_string(&vec![ld]) {
						query.push(("loaders", json_ld));
					}
				}
				if let Some(mc) = &mc {
					if let Ok(json_mc) = serde_json::to_string(&vec![mc]) {
						query.push(("game_versions", json_mc));
					}
				}

				let Ok(resp) = client.get(&url).query(&query).send().await else {
					return None;
				};

				let Ok(versions) = resp.json::<Vec<serde_json::Value>>().await else {
					return None;
				};

				let latest = versions.first()?;
				let latest_ver = latest.get("version_number").and_then(|v| v.as_str())?;

				if let Some(cur) = locked_ver {
					if cur != latest_ver {
						return Some((item_id, latest_ver.to_string()));
					}
				} else {
					// No lock version, consider latest available as update
					return Some((item_id, latest_ver.to_string()));
				}

				None
			});
		}
	}

	while let Some(res) = join_set.join_next().await {
		if let Ok(Some((id, ver))) = res {
			updates.insert(id, ver);
		}
	}

	updates
}

/// Exports an instance to a standard Modrinth modpack (.mrpack)
pub async fn export_instance_to_mrpack(
	instance_id: &str,
	target_file: &Path,
	back_state: &BackState,
) -> anyhow::Result<()> {
	let config = back_state.config().await?;
	let instance = config
		.instances
		.get(&InstanceID::from(instance_id))
		.context("Инстанс не найден")?;

	let inst_dir = instance.dir().context("Папка инстанса не найдена")?;
	let inst_cfg = instance.config();
	let inst_name = inst_cfg.name.as_deref().unwrap_or(instance_id).to_string();
	let mc_ver = inst_cfg
		.version
		.as_ref()
		.map(|v| match v {
			MinecraftVersionDeser::Version(name) => name.to_string(),
			_ => "1.21.1".to_string(),
		})
		.unwrap_or_else(|| "1.21.1".to_string());
	let loader_str = inst_cfg
		.loader
		.as_deref()
		.unwrap_or("fabric")
		.to_lowercase();

	let client = Client::builder()
		.user_agent("SqrilizzLauncher/1.0")
		.build()
		.unwrap_or_default();

	let file =
		 std::fs::File::create(target_file)
		.context("Не удалось создать выходной файл архива")?;
	let mut zip = ZipWriter::new(file);
	let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

	// 1. Scan mods folder
	let mods_dir = inst_dir.join("mods");
	let mut pack_files = Vec::new();

	if mods_dir.exists() {
		let entries = std::fs::read_dir(&mods_dir)?;
		let mut hashes = Vec::new();
		let mut hash_to_file = HashMap::new();

		for entry in entries.flatten() {
			let path = entry.path();
			if !path.is_file() {
				continue;
			}
			let name = path
				.file_name()
				.unwrap_or_default()
				.to_string_lossy()
				.to_string();
			if name.ends_with(".jar") {
				if let Ok(bytes) = std::fs::read(&path) {
					let mut hasher = Sha512::new();
					hasher.update(&bytes);
					let sha512_str = format!("{:x}", hasher.finalize());
					hash_to_file.insert(sha512_str.clone(), (name, bytes));
					hashes.push(sha512_str);
				}
			}
		}

		// Query Modrinth for downloadable hashes
		let mut known_hashes = HashMap::new();
		if !hashes.is_empty() {
			let body = json!({
				"hashes": hashes,
				"algorithm": "sha512"
			});
			if let Ok(resp) = client
				.post("https://api.modrinth.com/v2/version_files")
				.json(&body)
				.send()
				.await
			{
				if let Ok(res_map) = resp.json::<HashMap<String, serde_json::Value>>().await {
					for (h, val) in res_map {
						if let Some(files) = val.get("files").and_then(|f| f.as_array()) {
							if let Some(first_file) = files.first() {
								if let Some(url) = first_file.get("url").and_then(|u| u.as_str()) {
									known_hashes.insert(h, url.to_string());
								}
							}
						}
					}
				}
			}
		}

		// Add known mods to pack files, and unindexed mods to overrides/mods/
		for (hash, (name, bytes)) in hash_to_file {
			if let Some(url) = known_hashes.get(&hash) {
				pack_files.push(ModrinthPackFile {
					path: format!("mods/{name}"),
					hashes: ModrinthHashes { sha512: hash },
					env: Some(ModrinthFileEnv {
						client: SideSupport::Required,
						server: SideSupport::Required,
					}),
					downloads: vec![url.clone()],
				});
			} else {
				// Put in overrides/mods/
				zip.start_file(format!("overrides/mods/{name}"), options)?;
				zip.write_all(&bytes)?;
			}
		}
	}

	// 2. Add config overrides recursively
	let override_dirs = [
		"config",
		"defaultconfigs",
		"resourcepacks",
		"shaderpacks",
		"scripts",
		"kubejs",
	];

	for dir_name in override_dirs {
		let dir_path = inst_dir.join(dir_name);
		if dir_path.exists() && dir_path.is_dir() {
			add_dir_to_zip(&mut zip, &dir_path, &dir_path, &format!("overrides/{dir_name}"), options)?;
		}
	}

	// Also include options.txt if present
	let options_txt = inst_dir.join("options.txt");
	if options_txt.exists() && options_txt.is_file() {
		if let Ok(bytes) = std::fs::read(&options_txt) {
			zip.start_file("overrides/options.txt", options)?;
			zip.write_all(&bytes)?;
		}
	}

	// 3. Write modrinth.index.json
	let mut deps = ModrinthPackDependencies {
		minecraft: mc_ver,
		forge: None,
		neoforge: None,
		fabric_loader: None,
		quilt_loader: None,
	};

	match loader_str.as_str() {
		"fabric" => deps.fabric_loader = Some("*".into()),
		"forge" => deps.forge = Some("*".into()),
		"neoforge" => deps.neoforge = Some("*".into()),
		"quilt" => deps.quilt_loader = Some("*".into()),
		_ => deps.fabric_loader = Some("*".into()),
	}

	let index = ModrinthIndex {
		name: inst_name,
		version_id: "1.0.0".into(),
		summary: Some("Экспортировано из Sqrilizz Launcher".into()),
		files: pack_files,
		dependencies: deps,
	};

	let index_bytes = serde_json::to_vec_pretty(&index)?;
	zip.start_file("modrinth.index.json", options)?;
	zip.write_all(&index_bytes)?;

	zip.finish()?;
	Ok(())
}

fn add_dir_to_zip(
	zip: &mut ZipWriter<std::fs::File>,
	base_dir: &Path,
	current_dir: &Path,
	zip_prefix: &str,
	options: SimpleFileOptions,
) -> anyhow::Result<()> {
	if !current_dir.exists() {
		return Ok(());
	}
	for entry in std::fs::read_dir(current_dir)?.flatten() {
		let path = entry.path();
		if path.is_dir() {
			add_dir_to_zip(zip, base_dir, &path, zip_prefix, options)?;
		} else if path.is_file() {
			if let Ok(rel) = path.strip_prefix(base_dir) {
				let rel_str = rel.to_string_lossy().replace('\\', "/");
				let zip_path = format!("{zip_prefix}/{rel_str}");
				if let Ok(bytes) = std::fs::read(&path) {
					zip.start_file(zip_path, options)?;
					zip.write_all(&bytes)?;
				}
			}
		}
	}
	Ok(())
}

/// Imports a modpack file (.mrpack, CurseForge .zip, or generic instance zip)
pub async fn import_modpack_file(
	pack_path: &Path,
	back_state: &BackState,
) -> anyhow::Result<String> {
	let file = std::fs::File::open(pack_path)
		.context("Не удалось открыть файл модпака")?;
	let mut archive = ZipArchive::new(file)
		.context("Не удалось прочитать ZIP-архив модпака")?;

	// Check if Modrinth pack
	let is_mrpack = archive.by_name("modrinth.index.json").is_ok();
	let is_cfpack = archive.by_name("manifest.json").is_ok();

	let (name, mc_ver, loader, files_to_download, is_mr) = if is_mrpack {
		let index_file = archive
			.by_name("modrinth.index.json")
			.context("Не удалось прочитать modrinth.index.json")?;
		let index: ModrinthIndex =
			 serde_json::from_reader(index_file)
			.context("Ошибка парсинга modrinth.index.json")?;

		let mc = index.dependencies.minecraft.clone();
		let ld = if index.dependencies.fabric_loader.is_some() {
			Some("fabric".to_string())
		} else if index.dependencies.quilt_loader.is_some() {
			Some("quilt".to_string())
		} else if index.dependencies.forge.is_some() {
			Some("forge".to_string())
		} else if index.dependencies.neoforge.is_some() {
			Some("neoforge".to_string())
		} else {
			None
		};

		let mut downloads = Vec::new();
		for f in &index.files {
			if let Some(env) = &f.env {
				if env.client == SideSupport::Unsupported {
					continue;
				}
			}
			if let Some(url) = f.downloads.first() {
				downloads.push((f.path.clone(), url.clone()));
			}
		}

		(index.name, Some(mc), ld, downloads, true)
	} else if is_cfpack {
		let manifest_file = archive
			.by_name("manifest.json")
			.context("Не удалось прочитать manifest.json")?;
		let manifest: serde_json::Value =
			 serde_json::from_reader(manifest_file)
			.context("Ошибка парсинга manifest.json")?;

		let name = manifest
			.get("name")
			.and_then(|n| n.as_str())
			.unwrap_or("CurseForge Modpack")
			.to_string();

		let mc = manifest
			.get("minecraft")
			.and_then(|m| m.get("version"))
			.and_then(|v| v.as_str())
			.map(|s| s.to_string());

		let ld = manifest
			.get("minecraft")
			.and_then(|m| m.get("modLoaders"))
			.and_then(|l| l.as_array())
			.and_then(|a| a.first())
			.and_then(|first| first.get("id"))
			.and_then(|id| id.as_str())
			.and_then(|id| {
				let id_lower = id.to_lowercase();
				if id_lower.contains("fabric") {
					Some("fabric".to_string())
				} else if id_lower.contains("forge") {
					Some("forge".to_string())
				} else if id_lower.contains("quilt") {
					Some("quilt".to_string())
				} else if id_lower.contains("neoforge") {
					Some("neoforge".to_string())
				} else {
					None
				}
			});

		(name, mc, ld, Vec::new(), false)
	} else {
		let stem = pack_path
			.file_stem()
			.unwrap_or_default()
			.to_string_lossy()
			.to_string();
		(stem, None, None, Vec::new(), false)
	};

	// Determine unique instance id
	let mut raw_config = back_state.raw_config().await?;
	let target_id = make_valid_instance_id(&name);
	let mut candidate = target_id.clone();
	let mut counter = 2;
	while raw_config
		.instances
		.contains_key(&InstanceID::from(candidate.clone()))
	 {
		candidate = format!("{}_{}", target_id, counter);
		counter += 1;
	}
	let new_id = InstanceID::from(candidate.clone());

	let inst_dir = back_state.paths.data.join("instances").join(&*new_id);
	tokio::fs::create_dir_all(&inst_dir).await?;

	// Extract overrides from zip
	for i in 0..archive.len() {
		let mut file = archive.by_index(i)?;
		if file.is_dir() {
			continue;
		}

		let Some(name) = file.enclosed_name() else {
			continue;
		};

		let target_rel = if is_mr {
			if let Ok(p) = name.strip_prefix("overrides/") {
				Some(p.to_path_buf())
			} else if let Ok(p) = name.strip_prefix("client-overrides/") {
				Some(p.to_path_buf())
			} else {
				None
			}
		} else if is_cfpack {
			if let Ok(p) = name.strip_prefix("overrides/") {
				Some(p.to_path_buf())
			} else {
				None
			}
		} else {
			// Generic zip
			if let Ok(p) = name.strip_prefix(".minecraft/") {
				Some(p.to_path_buf())
			} else {
				Some(name.to_path_buf())
			}
		};

		if let Some(rel) = target_rel {
			let dest_path = inst_dir.join(rel);
			if let Some(parent) = dest_path.parent() {
				let _ = tokio::fs::create_dir_all(parent).await;
			}
			let mut outfile = std::fs::File::create(&dest_path)?;
			std::io::copy(&mut file, &mut outfile)?;
		}
	}

	// Download online mod files for mrpack
	if !files_to_download.is_empty() {
		let client = Client::builder()
			.user_agent("SqrilizzLauncher/1.0")
			.build()
			.unwrap_or_default();

		let mut join_set = tokio::task::JoinSet::new();
		for (rel_path, url) in files_to_download {
			let dest_path = inst_dir.join(rel_path);
			let client = client.clone();
			join_set.spawn(async move {
				if let Some(parent) = dest_path.parent() {
					let _ = tokio::fs::create_dir_all(parent).await;
				}
				if let Ok(resp) = client.get(&url).send().await {
					if let Ok(bytes) = resp.bytes().await {
						let _ = tokio::fs::write(&dest_path, bytes).await;
					}
				}
			});
		}

		while let Some(res) = join_set.join_next().await {
			let _ = res;
		}
	}

	// Create and register InstanceConfig
	let inst_config = InstanceConfig {
		name: Some(name),
		side: Some(Side::Client),
		version: mc_ver.map(|v| MinecraftVersionDeser::Version(v.into())),
		loader,
		icon: None,
		..Default::default()
	};

	apply_modifications_and_write(
		&mut raw_config,
		vec![ConfigModification::AddInstance(new_id, inst_config)],
		&back_state.paths,
		&mut NoOp,
	)
	.await?;

	back_state.invalidate(BackDependency::Items);

	Ok(candidate)
}
