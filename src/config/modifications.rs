use anyhow::{Context, anyhow, bail};
use sqrilizz_config::ConfigDeser;
use sqrilizz_config::instance::InstanceConfig;
use sqrilizz_config::package::add_or_update_package_config;
use sqrilizz_config::template::TemplateConfig;
use sqrilizz_config::{account::AccountConfig, package::PackageConfigDeser};
use sqrilizz_core::io::json_to_file_pretty;
use sqrilizz_shared::output::SqrilizzOutput;
use sqrilizz_shared::util::DeserListOrSingle;

use crate::io::paths::Paths;
use sqrilizz_shared::id::{InstanceID, TemplateID};

use super::Config;

impl Config {
	/// Consolidates the parents of a template into a template, saving the modified config
	pub async fn consolidate_template(
		&self,
		template_id: &TemplateID,
		paths: &Paths,
		o: &mut impl SqrilizzOutput,
	) -> anyhow::Result<()> {
		let mut template = self
			.consolidated_templates
			.get(template_id)
			.context("Template does not exist")?
			.clone();
		template.instance.from = DeserListOrSingle::default();

		let modifications = vec![ConfigModification::UpdateTemplate(
			template_id.clone(),
			template,
		)];
		let mut config = Self::open(&Self::get_path(paths))?;

		apply_modifications_and_write(&mut config, modifications, paths, o).await
	}

	/// Duplicates a template into a new one
	pub async fn duplicate_template(
		&self,
		template_id: &TemplateID,
		new_template_id: &TemplateID,
		paths: &Paths,
		o: &mut impl SqrilizzOutput,
	) -> anyhow::Result<()> {
		let template = self
			.templates
			.get(template_id)
			.context("Template does not exist")?
			.clone();

		let modifications = vec![ConfigModification::AddTemplate(
			new_template_id.clone(),
			template,
		)];
		let mut config = Self::open(&Self::get_path(paths))?;

		apply_modifications_and_write(&mut config, modifications, paths, o).await
	}
}

/// A modification operation that can be applied to the config
pub enum ConfigModification {
	/// Adds a new account
	AddAccount(String, AccountConfig),
	/// Adds a new instance
	AddInstance(InstanceID, InstanceConfig),
	/// Updates config for an instance
	UpdateInstance(InstanceID, InstanceConfig),
	/// Adds a new template
	AddTemplate(TemplateID, TemplateConfig),
	/// Updates config for a template
	UpdateTemplate(InstanceID, TemplateConfig),
	/// Adds a new package to an instance
	AddPackage(InstanceID, PackageConfigDeser),
	/// Removes an account
	RemoveAccount(String),
	/// Removes an instance
	RemoveInstance(InstanceID),
	/// Removes a template
	RemoveTemplate(InstanceID),
}

/// Applies modifications to the config
pub async fn apply_modifications(
	config: &mut ConfigDeser,
	modifications: Vec<ConfigModification>,
	_paths: &Paths,
	_o: &mut impl SqrilizzOutput,
) -> anyhow::Result<()> {
	for modification in modifications {
		// Existence checks
		if let ConfigModification::AddInstance(id, ..) = &modification {
			if config.instances.contains_key(id) {
				bail!("Instance {id} already exists");
			}
		}
		if let ConfigModification::AddTemplate(id, ..) = &modification {
			if config.templates.contains_key(id) {
				bail!("Template {id} already exists");
			}
		}

		match modification {
			ConfigModification::AddAccount(id, account) => {
				config.accounts.insert(id, account);
			}
			ConfigModification::AddInstance(instance_id, instance)
			| ConfigModification::UpdateInstance(instance_id, instance) => {
				config.instances.insert(instance_id, instance);
			}
			ConfigModification::AddTemplate(template_id, template)
			| ConfigModification::UpdateTemplate(template_id, template) => {
				config.templates.insert(template_id, template);
			}
			ConfigModification::AddPackage(instance_id, package) => {
				let instance = config
					.instances
					.get_mut(&instance_id)
					.ok_or(anyhow!("Unknown instance '{instance_id}'"))?;
				add_or_update_package_config(&mut instance.packages, package);
			}
			ConfigModification::RemoveAccount(account) => {
				config.accounts.remove(&account);
			}
			ConfigModification::RemoveInstance(instance) => {
				config.instances.remove(&instance);
			}
			ConfigModification::RemoveTemplate(template) => {
				config.templates.remove(&template);
			}
		};
	}
	Ok(())
}

/// Applies modifications to the config and writes it to the config file
pub async fn apply_modifications_and_write(
	config: &mut ConfigDeser,
	modifications: Vec<ConfigModification>,
	paths: &Paths,
	o: &mut impl SqrilizzOutput,
) -> anyhow::Result<()> {
	apply_modifications(config, modifications, paths, o).await?;
	let path = Config::get_path(paths);
	// Backup the contents first
	std::fs::copy(&path, paths.config.join("sqrilizz_write_backup.json"))
		.context("Failed to backup config")?;
	json_to_file_pretty(path, config).context("Failed to write modified configuration")?;

	Ok(())
}

#[cfg(test)]
mod tests {
	use sqrilizz_config::account::AccountVariant;
	use sqrilizz_shared::output::NoOp;

	use super::*;

	#[tokio::test]
	async fn test_account_add_modification() {
		let mut config = ConfigDeser::default();

		let account_config = AccountConfig::Simple(AccountVariant::Demo {});

		let modifications = vec![ConfigModification::AddAccount("bob".into(), account_config)];

		let paths = Paths::new_no_create().unwrap();

		apply_modifications(&mut config, modifications, &paths, &mut NoOp)
			.await
			.unwrap();
		assert!(config.accounts.contains_key("bob"));
	}
}
