use std::{
	fs::File,
	io::BufReader,
	path::{Path, PathBuf},
};

use anyhow::Context;
use base64::Engine;
use nbt::{CompoundTag, decode::read_compound_tag, encode::write_compound_tag};
use sqrilizz_shared::Side;

#[derive(Clone)]
pub struct InstanceSave {
	pub name: String,
	pub icon_path: Option<PathBuf>,
}

pub fn get_instance_saves(instance_dir: &Path, side: Side) -> anyhow::Result<Vec<InstanceSave>> {
	if let Side::Server = side {
		let path = instance_dir.join("world");
		if path.exists() && path.is_dir() {
			return Ok(vec![process_save(&path)?]);
		} else {
			return Ok(Vec::new());
		}
	}

	let saves_dir = instance_dir.join("saves");

	if !saves_dir.exists() {
		return Ok(Vec::new());
	}

	let mut saves = Vec::new();
	for entry in std::fs::read_dir(saves_dir)? {
		let Ok(entry) = entry else {
			continue;
		};
		if entry.file_type().is_ok_and(|x| x.is_dir()) {
			let Ok(save) = process_save(&entry.path()) else {
				continue;
			};
			saves.push(save);
		}
	}

	Ok(saves)
}

fn process_save(dir: &Path) -> anyhow::Result<InstanceSave> {
	let name = dir
		.file_name()
		.context("Failed to get save directory name")?
		.to_string_lossy()
		.to_string();
	let icon_path = dir.join("icon.png");
	let icon_path = Some(icon_path).filter(|p| p.exists());
	Ok(InstanceSave { name, icon_path })
}

pub fn get_instance_screenshots(instance_dir: &Path, side: Side) -> anyhow::Result<Vec<PathBuf>> {
	if let Side::Server = side {
		return Ok(Vec::new());
	}

	let screenshots_dir = instance_dir.join("screenshots");

	if !screenshots_dir.exists() {
		return Ok(Vec::new());
	}

	let mut screenshots = Vec::new();
	for entry in std::fs::read_dir(screenshots_dir)? {
		let Ok(entry) = entry else {
			continue;
		};
		if entry.file_type().is_ok_and(|x| x.is_file()) {
			screenshots.push(entry.path());
		}
	}

	Ok(screenshots)
}

pub fn get_instance_servers(instance_dir: &Path, side: Side) -> anyhow::Result<Vec<ServerInfo>> {
	if let Side::Server = side {
		return Ok(Vec::new());
	}

	let servers_file = instance_dir.join("servers.dat");
	if !servers_file.exists() {
		return Ok(Vec::new());
	}

	let mut servers =
		BufReader::new(File::open(servers_file).context("Failed to open servers.dat")?);
	let root = read_compound_tag(&mut servers).context("Failed to parse servers.dat")?;
	let servers = root
		.get_compound_tag_vec("servers")
		.map_err(|e| anyhow::anyhow!("Failed to read servers list: {e}"))?;

	let out = servers
		.iter()
		.filter_map(|s| {
			Some(ServerInfo {
				name: s.get_str("name").ok().map(|x| x.to_string()),
				address: s.get_str("ip").ok()?.to_string(),
				icon_png: s.get_str("icon").ok().and_then(|x| {
					base64::engine::GeneralPurpose::new(
						&base64::alphabet::STANDARD,
						base64::engine::GeneralPurposeConfig::new(),
					)
					.decode(x)
					.ok()
				}),
			})
		})
		.collect();

	Ok(out)
}

#[derive(Clone)]
pub struct ServerInfo {
	pub name: Option<String>,
	pub address: String,
	pub icon_png: Option<Vec<u8>>,
}

pub fn add_instance_server(
	instance_dir: &Path,
	side: Side,
	name: &str,
	address: &str,
) -> anyhow::Result<()> {
	if let Side::Server = side {
		return Ok(());
	}

	let servers_file = instance_dir.join("servers.dat");
	let mut root = if servers_file.exists() {
		let mut servers =
			BufReader::new(File::open(&servers_file).context("Failed to open servers.dat")?);
		read_compound_tag(&mut servers).context("Failed to parse servers.dat")?
	} else {
		CompoundTag::new()
	};

	let mut servers_vec: Vec<CompoundTag> = root
		.get_compound_tag_vec("servers")
		.map(|v| v.into_iter().cloned().collect())
		.unwrap_or_default();

	if let Some(existing) = servers_vec.iter_mut().find(|s| {
		s.get_str("ip").map(|ip| ip.eq_ignore_ascii_case(address)).unwrap_or(false)
	}) {
		existing.insert_str("name", name);
	} else {
		let mut server_tag = CompoundTag::new();
		server_tag.insert_str("name", name);
		server_tag.insert_str("ip", address);
		servers_vec.push(server_tag);
	}

	root.insert_compound_tag_vec("servers", servers_vec);

	if let Some(parent) = servers_file.parent() {
		std::fs::create_dir_all(parent)?;
	}
	let mut out = File::create(&servers_file).context("Failed to create servers.dat")?;
	write_compound_tag(&mut out, &root).context("Failed to write servers.dat")?;

	Ok(())
}

pub fn remove_instance_server(
	instance_dir: &Path,
	side: Side,
	address: &str,
) -> anyhow::Result<()> {
	if let Side::Server = side {
		return Ok(());
	}

	let servers_file = instance_dir.join("servers.dat");
	if !servers_file.exists() {
		return Ok(());
	}

	let mut servers =
		BufReader::new(File::open(&servers_file).context("Failed to open servers.dat")?);
	let mut root = read_compound_tag(&mut servers).context("Failed to parse servers.dat")?;

	let servers_vec: Vec<CompoundTag> = root
		.get_compound_tag_vec("servers")
		.map(|v| v.into_iter().cloned().collect())
		.unwrap_or_default();

	let filtered_servers: Vec<CompoundTag> = servers_vec
		.into_iter()
		.filter(|s| {
			s.get_str("ip")
				.map(|ip| !ip.eq_ignore_ascii_case(address))
				.unwrap_or(true)
		})
		.collect();

	root.insert_compound_tag_vec("servers", filtered_servers);

	let mut out = File::create(&servers_file).context("Failed to create servers.dat")?;
	write_compound_tag(&mut out, &root).context("Failed to write servers.dat")?;

	Ok(())
}


#[derive(Debug, Clone)]
pub struct InstanceResourcePack {
	pub name: String,
	pub file_name: String,
	pub description: Option<String>,
	pub icon_png: Option<Vec<u8>>,
	pub is_zip: bool,
}

pub fn get_instance_resource_packs(instance_dir: &Path) -> anyhow::Result<Vec<InstanceResourcePack>> {
	let rp_dir = instance_dir.join("resourcepacks");
	if !rp_dir.exists() {
		return Ok(Vec::new());
	}
	let mut packs = Vec::new();
	for entry in std::fs::read_dir(rp_dir)? {
		let Ok(entry) = entry else { continue; };
		let path = entry.path();
		let file_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
		if file_name.starts_with(".") { continue; }
		if path.is_file() && path.extension().is_some_and(|ext| ext == "zip") {
			let Ok(file) = File::open(&path) else { continue; };
			if let Ok(mut archive) = zip::ZipArchive::new(file) {
				let mut desc = None;
				if let Ok(mut mcmeta_file) = archive.by_name("pack.mcmeta") {
					let mut s = String::new();
					use std::io::Read;
					if mcmeta_file.read_to_string(&mut s).is_ok() {
						if let Ok(v) = serde_json::from_str::<serde_json::Value>(&s) {
							if let Some(d) = v.get("pack").and_then(|p| p.get("description")) {
								if let Some(text) = d.as_str() {
									desc = Some(text.to_string());
								} else if let Some(arr) = d.get("extra").and_then(|e| e.as_array()) {
									let joined: String = arr.iter().filter_map(|x| x.get("text").and_then(|t| t.as_str())).collect();
									desc = Some(joined);
								}
							}
						}
					}
				}
				let mut icon = None;
				if let Ok(mut icon_file) = archive.by_name("pack.png") {
					let mut buf = Vec::new();
					use std::io::Read;
					if icon_file.read_to_end(&mut buf).is_ok() {
						icon = Some(buf);
					}
				}
				packs.push(InstanceResourcePack {
					name: file_name.trim_end_matches(".zip").to_string(),
					file_name,
					description: desc,
					icon_png: icon,
					is_zip: true,
				});
			}
		} else if path.is_dir() {
			let mcmeta_path = path.join("pack.mcmeta");
			let mut desc = None;
			if mcmeta_path.exists() {
				if let Ok(s) = std::fs::read_to_string(mcmeta_path) {
					if let Ok(v) = serde_json::from_str::<serde_json::Value>(&s) {
						if let Some(d) = v.get("pack").and_then(|p| p.get("description")) {
							if let Some(text) = d.as_str() {
								desc = Some(text.to_string());
							}
						}
					}
				}
			}
			let icon_path = path.join("pack.png");
			let icon = if icon_path.exists() { std::fs::read(icon_path).ok() } else { None };
			packs.push(InstanceResourcePack {
				name: file_name.clone(),
				file_name,
				description: desc,
				icon_png: icon,
				is_zip: false,
			});
		}
	}
	packs.sort_by(|a, b| a.name.cmp(&b.name));
	Ok(packs)
}

#[derive(Debug, Clone)]
pub struct InstanceShaderPack {
	pub name: String,
	pub file_name: String,
}

pub fn get_instance_shader_packs(instance_dir: &Path) -> anyhow::Result<Vec<InstanceShaderPack>> {
	let sp_dir = instance_dir.join("shaderpacks");
	if !sp_dir.exists() {
		return Ok(Vec::new());
	}
	let mut shaders = Vec::new();
	for entry in std::fs::read_dir(sp_dir)? {
		let Ok(entry) = entry else { continue; };
		let path = entry.path();
		let file_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
		if file_name.starts_with(".") { continue; }
		if (path.is_file() && path.extension().is_some_and(|ext| ext == "zip")) || path.is_dir() {
			shaders.push(InstanceShaderPack {
				name: file_name.trim_end_matches(".zip").to_string(),
				file_name,
			});
		}
	}
	shaders.sort_by(|a, b| a.name.cmp(&b.name));
	Ok(shaders)
}


#[derive(Debug, Clone)]
pub struct ModConflictWarning {
	pub mod_name: String,
	pub duplicates: Vec<String>,
}

pub fn check_mod_conflicts(instance_dir: &Path) -> Vec<ModConflictWarning> {
	let mods_dir = instance_dir.join("mods");
	if !mods_dir.exists() {
		return Vec::new();
	}
	let Ok(entries) = std::fs::read_dir(mods_dir) else {
		return Vec::new();
	};

	let mut base_map: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();

	for entry in entries.flatten() {
		let path = entry.path();
		if !path.is_file() || !path.extension().is_some_and(|e| e == "jar") {
			continue;
		}
		let file_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
		let base_name = file_name
			.trim_end_matches(".jar")
			.split(&['-', '_', '+'][..])
			.next()
			.unwrap_or(&file_name)
			.to_lowercase();

		base_map.entry(base_name).or_default().push(file_name);
	}

	let mut warnings = Vec::new();
	for (base, files) in base_map {
		if files.len() > 1 {
			warnings.push(ModConflictWarning {
				mod_name: base,
				duplicates: files,
			});
		}
	}
	warnings
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_servers_dat_crud() {
		let temp_dir = std::env::temp_dir().join(format!("test_servers_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
		std::fs::create_dir_all(&temp_dir).unwrap();

		// 1. Initial list should be empty
		let list = get_instance_servers(&temp_dir, Side::Client).unwrap();
		assert!(list.is_empty());

		// 2. Add first server
		add_instance_server(&temp_dir, Side::Client, "SilverPeek", "mc.silverpeek.ru").unwrap();
		let list = get_instance_servers(&temp_dir, Side::Client).unwrap();
		assert_eq!(list.len(), 1);
		assert_eq!(list[0].name.as_deref(), Some("SilverPeek"));
		assert_eq!(list[0].address, "mc.silverpeek.ru");

		// 3. Add second server
		add_instance_server(&temp_dir, Side::Client, "Hypixel", "mc.hypixel.net").unwrap();
		let list = get_instance_servers(&temp_dir, Side::Client).unwrap();
		assert_eq!(list.len(), 2);

		// 4. Remove first server
		remove_instance_server(&temp_dir, Side::Client, "mc.silverpeek.ru").unwrap();
		let list = get_instance_servers(&temp_dir, Side::Client).unwrap();
		assert_eq!(list.len(), 1);
		assert_eq!(list[0].name.as_deref(), Some("Hypixel"));

		let _ = std::fs::remove_dir_all(&temp_dir);
	}
}
