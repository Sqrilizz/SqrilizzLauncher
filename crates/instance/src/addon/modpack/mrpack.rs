#![allow(missing_docs)]

use std::{
	io::{Read, Seek},
	path::{Path, PathBuf},
};

use anyhow::Context;
use sqrilizz_shared::{Side, minecraft::AddonKind, pkg::AddonOptionalHashes};
use serde::{Deserialize, Serialize};
use zip::ZipArchive;

use crate::addon::{
	Addon,
	modpack::{DefaultLinkMethod, LinkMethod, apply_zip_override},
	storage,
};

pub struct ModrinthPack<R> {
	index: ModrinthIndex,
	zip: ZipArchive<R>,
	link_method: Box<dyn LinkMethod + Send + 'static>,
}

impl<R: Read + Seek> ModrinthPack<R> {
	pub fn from_stream(r: R) -> anyhow::Result<Self> {
		let mut zip = ZipArchive::new(r).context("Failed to open pack zip file")?;
		let index = zip
			.by_name("modrinth.index.json")
			.context("Failed to open Modrinth index")?;
		let index: ModrinthIndex =
			serde_json::from_reader(index).context("Failed to deserialize index")?;

		Ok(Self {
			index,
			zip,
			link_method: Box::new(DefaultLinkMethod),
		})
	}

	pub fn index(&self) -> &ModrinthIndex {
		&self.index
	}

	#[cfg(feature = "net")]
	pub async fn download(
		&mut self,
		addons_dir: &Path,
		client: &sqrilizz_net::download::Client,
	) -> anyhow::Result<()> {
		let mut tasks = tokio::task::JoinSet::new();
		for file in &self.index.files {
			let path = storage::get_sha512_addon_path(addons_dir, &file.hashes.sha512);
			if path.exists() {
				continue;
			}

			let Some(url) = file.downloads.first() else {
				continue;
			};
			let url = url.clone();
			let client = client.clone();
			tasks.spawn(async move { sqrilizz_net::download::file(url, path, &client).await });
		}

		while let Some(result) = tasks.join_next().await {
			result??;
		}

		Ok(())
	}

	pub fn apply(
		&mut self,
		target: &Path,
		addons_dir: &Path,
		side: Side,
		mut old_pack: Option<&mut Self>,
	) -> anyhow::Result<()> {
		for file in &self.index.files {
			let source_path = storage::get_sha512_addon_path(addons_dir, &file.hashes.sha512);
			let target_path = target.join(&file.path);

			if let Some(parent) = target_path.parent() {
				let _ = std::fs::create_dir_all(parent);
			}

			self.link_method
				.link(&source_path, &target_path)
				.context("Failed to link addon")?;
		}

		for i in 0..self.zip.len() {
			let file = self.zip.by_index(i)?;
			if file.is_dir() {
				continue;
			}
			let Some(name) = file.enclosed_name() else {
				continue;
			};

			let target_rel_path = if let Ok(path) = name.strip_prefix("overrides/") {
				path
			} else if let Ok(path) = name.strip_prefix("client-overrides/") {
				if side != Side::Client {
					continue;
				}
				path
			} else if let Ok(path) = name.strip_prefix("server-overrides/") {
				if side != Side::Server {
					continue;
				}
				path
			} else {
				continue;
			};

			apply_zip_override(
				file,
				target_rel_path,
				target,
				old_pack.as_mut().map(|x| &mut x.zip),
			)?;
		}

		Ok(())
	}

	pub fn get_addons(&mut self, target: &Path, addons_dir: &Path) -> anyhow::Result<Vec<Addon>> {
		let mut out = Vec::new();
		for file in &self.index.files {
			let source_path = storage::get_sha512_addon_path(addons_dir, &file.hashes.sha512);
			let target_path = target.join(&file.path);

			let kind = if file.path.starts_with("mods") {
				AddonKind::Mod
			} else if file.path.starts_with("resourcepacks") {
				AddonKind::ResourcePack
			} else {
				AddonKind::Mod
			};

			let addon = Addon {
				kind,
				file_name: target_path
					.file_name()
					.unwrap()
					.to_string_lossy()
					.to_string(),
				original_path: None,
				target_paths: vec![target_path],
				source: Some(source_path),
				hashes: AddonOptionalHashes {
					sha512: Some(file.hashes.sha512.clone()),
					..Default::default()
				},
			};

			out.push(addon);
		}

		Ok(out)
	}

	pub fn get_overrides(&mut self, side: Side) -> anyhow::Result<Vec<PathBuf>> {
		let mut out = Vec::new();
		for i in 0..self.zip.len() {
			let file = self.zip.by_index(i)?;
			if file.is_dir() {
				continue;
			}
			let Some(name) = file.enclosed_name() else {
				continue;
			};

			if let Ok(path) = name.strip_prefix("overrides/") {
				out.push(path.to_owned());
			} else if let Ok(path) = name.strip_prefix("client-overrides/") {
				if side != Side::Client {
					continue;
				}
				out.push(path.to_owned());
			} else if let Ok(path) = name.strip_prefix("server-overrides/") {
				if side != Side::Server {
					continue;
				}
				out.push(path.to_owned());
			};
		}

		Ok(out)
	}

	pub fn set_link_method(&mut self, method: Box<dyn LinkMethod + Send + 'static>) {
		self.link_method = method;
	}
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModrinthIndex {
	pub name: String,
	pub version_id: String,
	#[serde(default)]
	pub summary: Option<String>,
	pub files: Vec<ModrinthPackFile>,
	pub dependencies: ModrinthPackDependencies,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModrinthPackFile {
	pub path: String,
	pub hashes: ModrinthHashes,
	#[serde(default)]
	pub env: Option<ModrinthFileEnv>,
	pub downloads: Vec<String>,
}

impl ModrinthPackFile {
	pub fn get_modrinth_info(&self) -> (Option<&str>, Option<&str>) {
		for url in &self.downloads {
			if !url.contains("modrinth") {
				continue;
			}

			let Some(data_pos) = url.find("data/") else {
				continue;
			};
			let slice = &url[data_pos + 5..];
			let mut split = slice.split('/');
			let project = split.next();
			let version = split.nth(1);

			return (project, version);
		}

		(None, None)
	}
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModrinthHashes {
	pub sha512: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModrinthFileEnv {
	pub client: SideSupport,
	pub server: SideSupport,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ModrinthPackDependencies {
	pub minecraft: String,
	#[serde(default)]
	pub forge: Option<String>,
	#[serde(default)]
	pub neoforge: Option<String>,
	#[serde(default)]
	pub fabric_loader: Option<String>,
	#[serde(default)]
	pub quilt_loader: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Copy, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SideSupport {
	Required,
	Optional,
	Unsupported,
	#[default]
	Unknown,
}
