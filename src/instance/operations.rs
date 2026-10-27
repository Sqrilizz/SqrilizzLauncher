use std::path::PathBuf;

use anyhow::{Context, bail};
use sqrilizz_config::{instance::InstanceConfig, template::TemplateConfig};
use sqrilizz_instance::files::{
	InstanceSave, ServerInfo, get_instance_saves, get_instance_screenshots, get_instance_servers, add_instance_server, remove_instance_server,
};
use sqrilizz_shared::{
	Side,
	id::{InstanceID, TemplateID},
	io::dir_size,
	output::SqrilizzOutput,
	util::DeserListOrSingle,
};

use crate::{
	config::{
		Config,
		modifications::{ConfigModification, apply_modifications_and_write},
	},
	instance::Instance,
	io::paths::Paths,
};

impl Instance {
	/// Consolidates the parent configs of this instance into it's config, and saves the result
	pub async fn consolidate(
		&self,
		paths: &Paths,
		o: &mut impl SqrilizzOutput,
	) -> anyhow::Result<()> {
		let mut config = self.config.clone();
		config.from = DeserListOrSingle::default();

		let modifications = vec![ConfigModification::UpdateInstance(self.id.clone(), config)];
		let mut config = Config::open(&Config::get_path(paths))?;

		apply_modifications_and_write(&mut config, modifications, paths, o).await
	}

	/// Duplicates this instance to create a new one
	pub async fn duplicate(
		&self,
		new_id: &InstanceID,
		paths: &Paths,
		o: &mut impl SqrilizzOutput,
	) -> anyhow::Result<()> {
		let mut config = self.original_config.clone();
		if let Some(ref name) = self.config.name {
			config.name = Some(format!("{name} (копия)"));
		}

		let src_path = if self.config.dir.is_none() && self.side() == Side::Client {
			self.dir.as_ref().and_then(|d| d.parent()).map(std::path::Path::to_path_buf)
		} else {
			self.dir.clone()
		};

		if let Some(src) = src_path {
			if src.exists() {
				let dest = paths.data.join("instances").join(&**new_id);
				let _ = sqrilizz_core::io::files::copy_dir_contents(&src, &dest);
			}
		}

		let modifications = vec![ConfigModification::AddInstance(new_id.clone(), config)];
		let mut config = Config::open(&Config::get_path(paths))?;

		apply_modifications_and_write(&mut config, modifications, paths, o).await
	}

	/// Extracts a template from this instance
	pub async fn extract(
		&self,
		new_id: &TemplateID,
		paths: &Paths,
		o: &mut impl SqrilizzOutput,
	) -> anyhow::Result<()> {
		let mut config = self.original_config.clone();
		config.side = None;
		let config = TemplateConfig {
			instance: config,
			..Default::default()
		};

		let inst_config = InstanceConfig {
			side: Some(self.side()),
			..Default::default()
		};

		let modifications = vec![
			ConfigModification::AddTemplate(new_id.clone(), config),
			ConfigModification::UpdateInstance(self.id.clone(), inst_config),
		];
		let mut config = Config::open(&Config::get_path(paths))?;

		apply_modifications_and_write(&mut config, modifications, paths, o).await
	}

	/// Deletes this instance and all of its files. Use with caution!
	pub async fn delete(
		&self,
		paths: &Paths,
		o: &mut impl SqrilizzOutput,
	) -> anyhow::Result<()> {
		self.delete_files()
			.await
			.context("Failed to delete files")?;

		let mut config = Config::open(&Config::get_path(paths))?;
		let modifications = vec![ConfigModification::RemoveInstance(self.id.clone())];
		apply_modifications_and_write(&mut config, modifications, paths, o)
			.await
			.context("Failed to modify and write config")?;

		Ok(())
	}

	/// Removes all game files for an instance, including saves. Does not remove the instance from config. Use with caution!
	pub async fn delete_files(&self) -> anyhow::Result<()> {
		if let Some(dir) = &self.dir {
			// Remove the parent directory above .minecraft for clients
			let path = if self.config.dir.is_none() && self.side() == Side::Client {
				if let Some(parent) = dir.parent() {
					if parent
						.file_name()
						.is_some_and(|x| x.to_string_lossy() == "instances")
					{
						bail!("Attempted to remove instances directory");
					}
					parent
				} else {
					dir
				}
			} else {
				dir
			};

			if path.exists() {
				tokio::fs::remove_dir_all(path).await?;
			}
		}

		Ok(())
	}

	/// Gets the size of this instance on the disk
	pub async fn get_size(&self) -> anyhow::Result<usize> {
		let Some(dir) = &self.dir else {
			return Ok(0);
		};

		if !dir.exists() {
			return Ok(0);
		}

		dir_size(dir)
	}

	/// Gets the saves for this instance
	pub fn get_saves(&self) -> anyhow::Result<Vec<InstanceSave>> {
		let Some(dir) = &self.dir else {
			return Ok(Vec::new());
		};

		get_instance_saves(dir, self.side())
	}

	/// Gets the screenshots for this instance
	pub fn get_screenshots(&self) -> anyhow::Result<Vec<PathBuf>> {
		let Some(dir) = &self.dir else {
			return Ok(Vec::new());
		};

		get_instance_screenshots(dir, self.side())
	}

	/// Gets the servers for this instance
	pub fn get_servers(&self) -> anyhow::Result<Vec<ServerInfo>> {
		let Some(dir) = &self.dir else {
			return Ok(Vec::new());
		};

		get_instance_servers(dir, self.side())
	}

	/// Adds a server to this instance
	pub fn add_server(&self, name: &str, address: &str) -> anyhow::Result<()> {
		let Some(dir) = &self.dir else {
			return Ok(());
		};

		add_instance_server(dir, self.side(), name, address)
	}

	/// Removes a server from this instance
	pub fn remove_server(&self, address: &str) -> anyhow::Result<()> {
		let Some(dir) = &self.dir else {
			return Ok(());
		};

		remove_instance_server(dir, self.side(), address)
	}
}
