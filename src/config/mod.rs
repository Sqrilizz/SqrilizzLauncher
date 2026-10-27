/// Reading account configuration
mod account;
/// Validation of config and other checks
mod checks;
/// Setting up the SqrilizzCore
mod core_setup;
/// Configuring instance modifications
pub mod modifications;
/// Configuring packages
pub mod package;
/// Configuring global preferences
pub mod preferences;

use crate::config::account::read_account_config;
use crate::config::checks::{check_configured_packages, check_launcher_version};
use crate::config::core_setup::setup_core;
use anyhow::Context;
use sqrilizz_config::ConfigDeser;
use sqrilizz_config::template::TemplateConfig;
use sqrilizz_config::template::consolidate_template_configs;
use sqrilizz_core::SqrilizzCore;
use sqrilizz_core::account::AccountManager;
use sqrilizz_core::auth_crate::mc::ClientId;
use sqrilizz_core::io::{json_from_file, json_to_file_pretty};
use sqrilizz_shared::id::{InstanceID, TemplateID};
use sqrilizz_shared::output::{MessageContents, SqrilizzOutput};
use sqrilizz_shared::util::is_valid_identifier;
use sqrilizz_shared::translate;
use preferences::ConfigPreferences;
use reqwest::Client;

use super::instance::Instance;
use crate::io::paths::Paths;
use crate::pkg::reg::PkgRegistry;

use serde_json::json;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub use checks::is_first_run;

/// The data resulting from reading configuration.
/// Represents all of the configured data that Sqrilizz Launcher will use
pub struct Config {
	/// The account manager
	pub accounts: AccountManager,
	/// Instances
	pub instances: HashMap<InstanceID, Instance>,
	/// Unconsolidated templates
	pub templates: HashMap<TemplateID, TemplateConfig>,
	/// Consolidated templates
	pub consolidated_templates: HashMap<TemplateID, TemplateConfig>,
	/// The globally applied template
	pub base_template: TemplateConfig,
	/// Named groups of instances
	pub instance_groups: HashMap<Arc<str>, Vec<InstanceID>>,
	/// The registry of packages. Will include packages that are configured when created this way
	pub packages: Arc<PkgRegistry>,
	/// Global user preferences
	pub prefs: ConfigPreferences,
}

impl Config {
	/// Get the config path
	pub fn get_path(paths: &Paths) -> PathBuf {
		paths.config.join("sqrilizz.json")
	}

	/// Open the config from a file
	pub fn open(path: &Path) -> anyhow::Result<ConfigDeser> {
		if path.exists() {
			Ok(json_from_file(path).context("Failed to open config")?)
		} else {
			let config = default_config();
			json_to_file_pretty(path, &config).context("Failed to write default configuration")?;
			Ok(serde_json::from_value(config).context("Failed to parse default configuration")?)
		}
	}

	/// Create the default config at the specified path if it does not exist
	pub fn create_default(path: &Path) -> anyhow::Result<()> {
		if !path.exists() {
			let doc = default_config();
			json_to_file_pretty(path, &doc).context("Failed to write default configuration")?;
		}
		Ok(())
	}

	/// Create the Config struct from deserialized config
	async fn load_from_deser(
		config: ConfigDeser,
		show_warnings: bool,
		paths: &Paths,
		client_id: ClientId,
		o: &mut impl SqrilizzOutput,
	) -> Self {
		if show_warnings {
			let _ = check_launcher_version(paths, o);
		}

		let mut accounts = AccountManager::new(client_id);
		let mut instances = HashMap::with_capacity(config.instances.len());
		// Preferences
		let (prefs, repositories) = ConfigPreferences::read(&config.preferences, paths, o).await;

		let packages = PkgRegistry::new(repositories);

		if show_warnings {
			check_configured_packages(&config.instances, &config.templates, o);
		}

		// Accounts
		for (account_id, account_config) in config.accounts.iter() {
			if !is_valid_identifier(account_id) {
				o.display(MessageContents::Error(format!(
					"Invalid account ID '{account_id}'"
				)));
				continue;
			}
			let account = read_account_config(account_config, account_id);

			accounts.add_account(account);
		}

		if let Some(default_account_id) = &config.default_account {
			if accounts.account_exists(default_account_id) {
				accounts
					.choose_account(default_account_id)
					.expect("Default account should exist");
			} else {
				o.display(MessageContents::Error(format!(
					"Provided default account '{default_account_id}' does not exist"
				)));
			}
		} else if config.accounts.is_empty() && show_warnings {
			o.display(MessageContents::Warning(translate!(o, NoDefaultAccount)));
		} else if show_warnings {
			o.display(MessageContents::Warning(translate!(o, NoAccounts)));
		}

		// Consolidate templates
		let consolidated_templates = consolidate_template_configs(
			config.templates.clone(),
			config.base_template.as_ref(),
			o,
		);

		// Instances
		for (instance_id, instance_config) in config.instances {
			let result = Instance::from_config(
				instance_id.clone(),
				instance_config,
				&consolidated_templates,
				paths,
			);

			let instance = match result {
				Ok(instance) => instance,
				Err(e) => {
					o.display(MessageContents::Error(translate!(
						o,
						InvalidInstanceConfig,
						"instance" = &instance_id,
						"error" = &format!("{e:?}")
					)));
					continue;
				}
			};

			instances.insert(instance_id, instance);
		}

		for group in config.instance_groups.keys() {
			if !is_valid_identifier(group) {
				o.display(MessageContents::Error(format!(
					"Invalid ID for instance group '{group}'"
				)));
			}
		}

		Self {
			accounts,
			instances,
			templates: config.templates,
			consolidated_templates,
			base_template: config.base_template.unwrap_or_default(),
			instance_groups: config.instance_groups,
			packages: Arc::new(packages),
			prefs,
		}
	}

	/// Load the configuration from the config file
	pub async fn load(
		path: &Path,
		show_warnings: bool,
		paths: &Paths,
		client_id: ClientId,
		o: &mut impl SqrilizzOutput,
	) -> anyhow::Result<Self> {
		let obj = Self::open(path)?;
		Ok(Self::load_from_deser(obj, show_warnings, paths, client_id, o).await)
	}

	/// Gets the core from the config
	pub async fn get_core(
		&self,
		client_id: Option<&ClientId>,
		client: &Client,
		paths: &Paths,
		o: &mut impl SqrilizzOutput,
	) -> anyhow::Result<SqrilizzCore> {
		setup_core(client_id, client, paths, o).await
	}
}

/// Default program configuration
fn default_config() -> serde_json::Value {
	json!(
		{
			"accounts": {
				"example": "microsoft"
			},
			"default_account": "example",
			"templates": {
				"1.21": {
					"version": "1.21.11",
					"loader": "vanilla"
				}
			},
			"instances": {
				"example-client": {
					"from": "1.21",
					"type": "client"
				},
				"example-server": {
					"from": "1.21",
					"type": "server"
				}
			}
		}
	)
}

#[cfg(test)]
mod tests {
	use super::*;

	use sqrilizz_shared::output;

	#[tokio::test]
	async fn test_default_config() {
		let deser = serde_json::from_value(default_config()).unwrap();
		let paths = Paths::new_no_create().unwrap();
		Config::load_from_deser(
			deser,
			true,
			&paths,
			ClientId::new(String::new()),
			&mut output::Simple(output::MessageLevel::Debug),
		)
		.await;
	}
}
