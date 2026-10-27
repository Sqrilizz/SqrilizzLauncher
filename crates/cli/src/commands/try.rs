use std::collections::HashMap;

use anyhow::Context;
use clap::Subcommand;
use sqrilizz::{
	config::modifications::{ConfigModification, apply_modifications_and_write},
	config_crate::instance::InstanceConfig,
	instance::{
		Instance,
		launch::LaunchSettings,
		update::InstanceUpdateContext,
	},
	io::lock::Lockfile,
	pkg_crate::{PkgRequest, PkgRequestSource},
	shared::{
		Side,
		id::InstanceID,
		output::{MessageContents, SqrilizzOutput},
		util::DeserListOrSingle,
		versions::MinecraftVersionDeser,
	},
};
use reqwest::Client;

use crate::{
	commands::CmdData,
	secrets::get_ms_client_id,
};

#[derive(Debug, Subcommand)]
pub enum TrySubcommand {
	#[command(about = "Try a new Minecraft version")]
	Version {
		/// The Minecraft version
		version: String,
		/// Optional loader to use for the instance
		#[arg(long)]
		loader: Option<String>,
		/// Optional template to derive from for the instance
		#[arg(long)]
		template: Option<String>,
	},
	#[command(about = "Try a modpack")]
	Modpack {
		/// The package of the modpack (i.e. modrinth:fabulously-optimized)
		modpack: String,
	},
}

pub async fn run(command: TrySubcommand, data: &mut CmdData<'_>) -> anyhow::Result<()> {
	data.ensure_config(true).await?;
	let mut raw_config = data.get_raw_config()?;
	let config = data.config.get_mut();

	let mut instance = match command {
		TrySubcommand::Version {
			version,
			loader,
			template,
		} => {
			let id = format!("try-{version}");

			let instance_config = InstanceConfig {
				side: Some(Side::Client),
				version: Some(MinecraftVersionDeser::Version(version.into())),
				loader,
				from: DeserListOrSingle::from_iter(template),
				..Default::default()
			};

			Instance::from_config(id.into(), instance_config, &HashMap::new(), &data.paths)?
		}
		TrySubcommand::Modpack { modpack } => {
			let req = PkgRequest::parse(&modpack, PkgRequestSource::UserRequire).arc();

			let instance_id = InstanceID::from(format!("try-{}", req.id));

			let instance_config = super::modpack::install_into_config(
				&req,
				instance_id.clone(),
				Some(Side::Client),
				config,
				&data.paths,
				data.output,
			)
			.await?;

			Instance::from_config(
				instance_id.clone(),
				instance_config,
				&HashMap::new(),
				&data.paths,
			)?
		}
	};

	let mut lock = Lockfile::open(&data.paths)?;
	let client = Client::new();

	let core = config
		.get_core(Some(&get_ms_client_id()), &client, &data.paths, data.output)
		.await?;

	let mut ctx = InstanceUpdateContext {
		packages: &config.packages,
		accounts: &mut config.accounts,
		prefs: &config.prefs,
		paths: &data.paths,
		lock: &mut lock,
		client: &client,
		output: data.output,
		core: &core,
	};

	let settings = LaunchSettings {
		offline_auth: false,
		pipe_stdin: true,
		quick_play: None,
	};

	let handle = instance
		.launch(settings, &mut ctx)
		.await
		.context("Failed to launch instance")?;

	handle
		.wait(data.output)
		.await?;

	let keep = data
		.output
		.prompt_yes_no(
			false,
			MessageContents::Simple("Do you want to KEEP this instance?".into()),
		)
		.await?;

	if keep {
		apply_modifications_and_write(
			&mut raw_config,
			vec![ConfigModification::AddInstance(
				instance.id().clone(),
				instance.original_config().clone(),
			)],
			&data.paths,
			data.output,
		)
		.await
		.context("Failed to write modified config")?;
	} else {
		let mut process = data.output.get_process();
		process.display(MessageContents::StartProcess(
			"Removing instance files".into(),
		));

		instance.delete_files().await?;

		process.display(MessageContents::Success(
			"Temporary instance removed".into(),
		));
	}

	Ok(())
}
