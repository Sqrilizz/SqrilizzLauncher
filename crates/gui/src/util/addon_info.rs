use std::{
	collections::{HashMap, HashSet},
	fs::File,
	io::{Read, copy},
	path::{Path, PathBuf},
	sync::{LazyLock, RwLock},
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256, Sha512};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AddonMetadata {
	pub title: Option<String>,
	pub icon_path: Option<String>,
	#[serde(default)]
	pub mod_id: Option<String>,
	#[serde(default)]
	pub slug: Option<String>,
	#[serde(default)]
	pub project_id: Option<String>,
}

static ADDON_CACHE: LazyLock<RwLock<HashMap<PathBuf, AddonMetadata>>> =
	LazyLock::new(|| RwLock::new(HashMap::new()));

pub fn get_addon_cache_dir() -> PathBuf {
	if let Some(proj) = directories::ProjectDirs::from("", "sqrilizz", "sqrilizz") {
		proj.cache_dir().to_path_buf()
	} else {
		std::env::temp_dir().join("sqrilizz_addon_cache")
	}
}

pub fn get_addon_metadata_cached(path: &Path) -> Option<AddonMetadata> {
	get_addon_metadata(path, &get_addon_cache_dir())
}

fn insert_addon_cache(path: &Path, meta: AddonMetadata) {
	if let Ok(mut cache) = ADDON_CACHE.write() {
		cache.insert(path.to_path_buf(), meta.clone());
		let path_str = path.to_string_lossy();
		if let Some(clean) = path_str.strip_suffix(".disabled") {
			cache.insert(PathBuf::from(clean), meta);
		} else {
			cache.insert(PathBuf::from(format!("{path_str}.disabled")), meta);
		}
	}
}

pub fn get_file_cache_key(path: &Path) -> Option<String> {
	let metadata = std::fs::metadata(path).ok()?;
	let len = metadata.len();
	let modified = metadata
		.modified()
		.ok()
		.and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
		.map(|d| d.as_secs())
		.unwrap_or(0);
	let path_str = path.to_string_lossy();
	let clean_path = path_str.strip_suffix(".disabled").unwrap_or(&path_str);
	let input = format!("{}:{}:{}", clean_path, len, modified);
	let mut hasher = Sha256::new();
	hasher.update(input.as_bytes());
	Some(format!("{:x}", hasher.finalize()))
}

pub fn get_addon_metadata(path: &Path, cache_dir: &Path) -> Option<AddonMetadata> {
	if let Ok(cache) = ADDON_CACHE.read() {
		if let Some(meta) = cache.get(path) {
			return Some(meta.clone());
		}
		let path_str = path.to_string_lossy();
		if let Some(clean) = path_str.strip_suffix(".disabled") {
			if let Some(meta) = cache.get(Path::new(clean)) {
				return Some(meta.clone());
			}
		}
	}

	let key = get_file_cache_key(path)?;
	let cache_folder = cache_dir.join("addon_cache");
	let json_path = cache_folder.join(format!("{key}.json"));

	if json_path.exists() {
		if let Ok(content) = std::fs::read_to_string(&json_path) {
			if let Ok(meta) = serde_json::from_str::<AddonMetadata>(&content) {
				insert_addon_cache(path, meta.clone());
				return Some(meta);
			}
		}
	}

	None
}

pub fn extract_local_addon_metadata(path: &Path, cache_dir: &Path) -> Option<AddonMetadata> {
	if !path.exists() {
		return None;
	}

	let key = get_file_cache_key(path)?;
	let cache_folder = cache_dir.join("addon_cache");
	let _ = std::fs::create_dir_all(&cache_folder);
	let json_path = cache_folder.join(format!("{key}.json"));
	let img_path = cache_folder.join(format!("{key}.img"));

	// Check if already cached on disk
	if json_path.exists() {
		if let Ok(content) = std::fs::read_to_string(&json_path) {
			if let Ok(meta) = serde_json::from_str::<AddonMetadata>(&content) {
				insert_addon_cache(path, meta.clone());
				return Some(meta);
			}
		}
	}

	let mut title: Option<String> = None;
	let mut mod_id: Option<String> = None;
	let mut icon_bytes: Option<Vec<u8>> = None;

	if path.is_dir() {
		// Directory format (e.g. unpacked resourcepack or datapack)
		for icon_name in &["pack.png", "icon.png", "logo.png"] {
			let cand = path.join(icon_name);
			if cand.is_file() {
				if let Ok(bytes) = std::fs::read(&cand) {
					icon_bytes = Some(bytes);
					break;
				}
			}
		}

		let mcmeta = path.join("pack.mcmeta");
		if mcmeta.is_file() {
			if let Ok(content) = std::fs::read_to_string(&mcmeta) {
				if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
					if let Some(desc) = val
						.get("pack")
						.and_then(|x| x.get("description"))
						.and_then(|x| x.as_str())
					{
						let clean = strip_mc_formatting(desc);
						if clean.len() >= 3 && clean.len() <= 40 && !clean.starts_with('{') {
							title = Some(clean);
						}
					}
				}
			}
		}
	} else if path.is_file() {
		let Ok(file) = File::open(path) else {
			return None;
		};
		let Ok(mut zip) = zip::ZipArchive::new(file) else {
			return None;
		};

		let mut icon_entry_name: Option<String> = None;

		// 1. Fabric mod
		if let Ok(mut f) = zip.by_name("fabric.mod.json") {
			let mut buf = String::new();
			if f.read_to_string(&mut buf).is_ok() {
				if let Ok(val) = serde_json::from_str::<serde_json::Value>(&buf) {
					if mod_id.is_none() {
						mod_id = val
							.get("id")
							.and_then(|x| x.as_str())
							.map(|x| x.to_string());
					}
					title = val
						.get("name")
						.and_then(|x| x.as_str())
						.map(|x| x.to_string());
					if let Some(ico) = val.get("icon") {
						if let Some(s) = ico.as_str() {
							icon_entry_name = Some(s.trim_start_matches('/').to_string());
						} else if let Some(obj) = ico.as_object() {
							if let Some(last) = obj.values().last().and_then(|x| x.as_str()) {
								icon_entry_name = Some(last.trim_start_matches('/').to_string());
							}
						}
					}
				}
			}
		}

		// 2. Quilt mod
		if title.is_none() || mod_id.is_none() {
			if let Ok(mut f) = zip.by_name("quilt.mod.json") {
				let mut buf = String::new();
				if f.read_to_string(&mut buf).is_ok() {
					if let Ok(val) = serde_json::from_str::<serde_json::Value>(&buf) {
						if mod_id.is_none() {
							mod_id = val
								.get("quilt_loader")
								.and_then(|x| x.get("id"))
								.and_then(|x| x.as_str())
								.map(|x| x.to_string());
						}
						let meta = val.get("quilt_loader").and_then(|x| x.get("metadata"));
						if let Some(meta) = meta {
							if mod_id.is_none() {
								mod_id = meta
									.get("id")
									.and_then(|x| x.as_str())
									.map(|x| x.to_string());
							}
							title = meta
								.get("name")
								.and_then(|x| x.as_str())
								.map(|x| x.to_string());
							if let Some(ico) = meta.get("icon").and_then(|x| x.as_str()) {
								icon_entry_name = Some(ico.trim_start_matches('/').to_string());
							}
						}
					}
				}
			}
		}

		// 3. Forge / NeoForge mod
		if title.is_none() || mod_id.is_none() {
			for toml_name in &["META-INF/neoforge.mods.toml", "META-INF/mods.toml"] {
				if let Ok(mut f) = zip.by_name(toml_name) {
					let mut buf = String::new();
					if f.read_to_string(&mut buf).is_ok() {
						for line in buf.lines() {
							let trimmed = line.trim();
							if mod_id.is_none()
								&& (trimmed.starts_with("modId =") || trimmed.starts_with("modId="))
							{
								if let Some((_, val)) = trimmed.split_once('=') {
									let clean = val.trim().trim_matches(|c| c == '"' || c == '\'');
									if !clean.is_empty() {
										mod_id = Some(clean.to_string());
									}
								}
							}
							if title.is_none()
								&& (trimmed.starts_with("displayName =")
									|| trimmed.starts_with("displayName="))
							{
								if let Some((_, val)) = trimmed.split_once('=') {
									let clean = val.trim().trim_matches(|c| c == '"' || c == '\'');
									if !clean.is_empty() {
										title = Some(clean.to_string());
									}
								}
							} else if icon_entry_name.is_none()
								&& (trimmed.starts_with("logoFile =")
									|| trimmed.starts_with("logoFile="))
							{
								if let Some((_, val)) = trimmed.split_once('=') {
									let clean = val.trim().trim_matches(|c| c == '"' || c == '\'');
									if !clean.is_empty() {
										icon_entry_name =
											Some(clean.trim_start_matches('/').to_string());
									}
								}
							}
						}
						if title.is_some() && mod_id.is_some() {
							break;
						}
					}
				}
			}
		}

		// 4. ResourcePack pack.mcmeta
		if let Ok(mut f) = zip.by_name("pack.mcmeta") {
			let mut buf = String::new();
			if f.read_to_string(&mut buf).is_ok() {
				if let Ok(val) = serde_json::from_str::<serde_json::Value>(&buf) {
					if let Some(desc) = val
						.get("pack")
						.and_then(|x| x.get("description"))
						.and_then(|x| x.as_str())
					{
						let clean = strip_mc_formatting(desc);
						if clean.len() >= 3 && clean.len() <= 40 && !clean.starts_with('{') {
							if title.is_none() {
								title = Some(clean);
							}
						}
					}
				}
			}
			if icon_entry_name.is_none() {
				icon_entry_name = Some("pack.png".to_string());
			}
		}

		// Attempt to read specified icon entry
		if let Some(entry_name) = icon_entry_name {
			if let Ok(mut f) = zip.by_name(&entry_name) {
				let mut bytes = Vec::new();
				if f.read_to_end(&mut bytes).is_ok() {
					icon_bytes = Some(bytes);
				}
			}
		}

		// Fallback for common icon file names
		if icon_bytes.is_none() {
			for icon_name in &["pack.png", "icon.png", "logo.png", "assets/icon.png"] {
				if let Ok(mut f) = zip.by_name(icon_name) {
					let mut bytes = Vec::new();
					if f.read_to_end(&mut bytes).is_ok() {
						icon_bytes = Some(bytes);
						break;
					}
				}
			}
		}
	}

	let icon_path = if let Some(bytes) = &icon_bytes {
		if std::fs::write(&img_path, bytes).is_ok() {
			Some(img_path.to_string_lossy().to_string())
		} else {
			None
		}
	} else {
		None
	};

	let meta = AddonMetadata {
		title,
		icon_path,
		mod_id,
		slug: None,
		project_id: None,
	};

	let _ = std::fs::write(&json_path, serde_json::to_string(&meta).unwrap_or_default());
	insert_addon_cache(path, meta.clone());

	Some(meta)
}

fn strip_mc_formatting(text: &str) -> String {
	let mut out = String::with_capacity(text.len());
	let mut chars = text.chars().peekable();
	while let Some(c) = chars.next() {
		if c == '§' {
			chars.next();
		} else {
			out.push(c);
		}
	}
	out.trim().to_string()
}

pub async fn resolve_remote_addons_batch(
	unresolved_paths: Vec<PathBuf>,
	cache_dir: PathBuf,
	client: reqwest::Client,
) {
	if unresolved_paths.is_empty() {
		return;
	}

	let hashes: Vec<(PathBuf, String)> = tokio::task::spawn_blocking(move || {
		let mut list = Vec::new();
		for p in unresolved_paths {
			if let Ok(mut f) = File::open(&p) {
				let mut hasher = Sha512::new();
				if copy(&mut f, &mut hasher).is_ok() {
					let hash = format!("{:x}", hasher.finalize());
					list.push((p, hash));
				}
			}
		}
		list
	})
	.await
	.unwrap_or_default();

	if hashes.is_empty() {
		return;
	}

	let hash_strings: Vec<String> = hashes.iter().map(|(_, h)| h.clone()).collect();
	let body = serde_json::json!({
		"hashes": hash_strings,
		"algorithm": "sha512"
	});

	let Ok(resp) = client
		.post("https://api.modrinth.com/v2/version_files")
		.header("User-Agent", "SqrilizzLauncher/1.0")
		.json(&body)
		.send()
		.await
	else {
		return;
	};

	let Ok(versions) = resp.json::<HashMap<String, serde_json::Value>>().await else {
		return;
	};

	let mut project_ids = HashSet::new();
	let mut hash_to_project = HashMap::new();
	for (hash, ver) in &versions {
		if let Some(pid) = ver.get("project_id").and_then(|x| x.as_str()) {
			project_ids.insert(pid.to_string());
			hash_to_project.insert(hash.clone(), pid.to_string());
		}
	}

	if project_ids.is_empty() {
		return;
	}

	let pids: Vec<String> = project_ids.into_iter().collect();
	let Ok(pids_json) = serde_json::to_string(&pids) else {
		return;
	};

	let Ok(proj_url) = reqwest::Url::parse_with_params(
		"https://api.modrinth.com/v2/projects",
		&[("ids", pids_json.as_str())],
	) else {
		return;
	};

	let Ok(proj_resp) = client
		.get(proj_url)
		.header("User-Agent", "SqrilizzLauncher/1.0")
		.send()
		.await
	else {
		return;
	};

	let Ok(projects) = proj_resp.json::<Vec<serde_json::Value>>().await else {
		return;
	};

	let mut project_map = HashMap::new();
	for p in projects {
		if let Some(id) = p.get("id").and_then(|x| x.as_str()) {
			let slug = p
				.get("slug")
				.and_then(|x| x.as_str())
				.map(|x| x.to_string());
			let title = p
				.get("title")
				.and_then(|x| x.as_str())
				.map(|x| x.to_string());
			let icon_url = p
				.get("icon_url")
				.and_then(|x| x.as_str())
				.map(|x| x.to_string());
			project_map.insert(id.to_string(), (title, icon_url, slug));
		}
	}

	let cache_folder = cache_dir.join("addon_cache");
	let _ = tokio::fs::create_dir_all(&cache_folder).await;

	for (path, hash) in hashes {
		if let Some(pid) = hash_to_project.get(&hash) {
			if let Some((title, icon_url, slug)) = project_map.get(pid) {
				let Some(key) = get_file_cache_key(&path) else {
					continue;
				};
				let img_path = cache_folder.join(format!("{key}.img"));
				let json_path = cache_folder.join(format!("{key}.json"));

				let mut saved_icon = None;
				if let Some(url) = icon_url {
					if let Ok(img_resp) = client.get(url).send().await {
						if let Ok(bytes) = img_resp.bytes().await {
							if tokio::fs::write(&img_path, &bytes).await.is_ok() {
								saved_icon = Some(img_path.to_string_lossy().to_string());
							}
						}
					}
				}

				let existing_meta = get_addon_metadata_cached(&path);
				let mod_id = existing_meta.and_then(|m| m.mod_id);

				let meta = AddonMetadata {
					title: title.clone(),
					icon_path: saved_icon,
					mod_id,
					slug: slug.clone(),
					project_id: Some(pid.clone()),
				};

				let _ =
					tokio::fs::write(&json_path, serde_json::to_string(&meta).unwrap_or_default())
						.await;

				insert_addon_cache(&path, meta);
			}
		}
	}
}
