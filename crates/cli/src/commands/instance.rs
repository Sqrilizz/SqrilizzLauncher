use std::ops::DerefMut;
use std::sync::Arc;

use anyhow::{Context, bail};
use clap::Subcommand;
use color_print::cprintln;
use inquire::Confirm;
use itertools::Itertools;
use sqrilizz::config::modifications::{ConfigModification, apply_modifications_and_write};
use sqrilizz::config_crate::instance::InstanceConfig;
use sqrilizz::core::QuickPlayType;
use sqrilizz::instance::tracking::RunningInstanceRegistry;
use sqrilizz::instance::update::{InstanceUpdateContext, UpdateFacets};
use sqrilizz::io::lock::Lockfile;
use sqrilizz::shared::id::InstanceID;
use sqrilizz::shared::io::config::IO_CONFIG;
use sqrilizz::shared::java_args::MemoryNum;
use sqrilizz::shared::output::{MessageContents, NoOp};
use sqrilizz::shared::util::to_string_json;

use sqrilizz::instance::launch::LaunchSettings;
use sqrilizz::shared::{Side, UpdateDepth, output::SqrilizzOutput};
use reqwest::Client;

use super::CmdData;
use crate::commands::config::edit_temp_file;
use crate::output::{
	format_instance_stdout_line,
	instance_stdout_line,
};
use crate::prompt::{
	pick_instance, pick_instance_id, pick_instances, pick_loader, pick_minecraft_version, pick_side,
};
use crate::secrets::get_ms_client_id;

/// Buffers partial lines from raw chunked output, so that only complete
/// lines are yielded
pub struct LineDecoder {
	buffer: String,
}

impl LineDecoder {
	pub fn new() -> Self {
		Self {
			buffer: String::new(),
		}
	}

	/// Decode a chunk into complete lines. Returns None if no complete line
	/// was produced
	pub fn decode(&mut self, bytes: &[u8], len: usize) -> Option<Vec<String>> {
		let chunk = String::from_utf8_lossy(&bytes[..len.min(bytes.len())]);
		self.buffer.push_str(&chunk);

		let newline = self.buffer.rfind(['\n', '\r'])?;
		let out = self.buffer[..=newline]
			.lines()
			.map(|x| x.to_string())
			.collect();
		self.buffer = self.buffer[newline + 1..].to_string();

		Some(out)
	}
}

#[derive(Debug, Subcommand)]
pub enum InstanceSubcommand {
	#[command(about = "List all instances")]
	#[clap(alias = "ls")]
	List {
		/// Whether to remove formatting and warnings from the output
		#[arg(short, long)]
		raw: bool,
		/// Filter by instance side
		#[arg(short, long)]
		side: Option<Side>,
	},
	#[command(about = "Print useful information about an instance")]
	Info { instance: Option<String> },
	#[command(about = "Launch instances to play the game")]
	Launch {
		/// An optional account to choose when launching
		#[arg(short, long)]
		account: Option<String>,
		/// Whether to launch in offline mode, skipping authentication. This only works
		/// if you have authenticated at least once
		#[arg(short, long)]
		offline: bool,
		/// Launch into a world or server. Can be either world:<world>, server:<ip> or realm:<realm>
		#[arg(short, long)]
		quick_play: Option<QuickPlayType>,
		/// The instance to launch
		instance: Option<String>,
	},
	#[command(about = "Update versions, files, and packages of an instance")]
	Update {
		/// Whether to force update files that have already been downloaded
		#[arg(short, long)]
		force: bool,
		/// Whether to update all instances
		#[arg(short, long)]
		all: bool,
		/// Whether to only update packages
		#[arg(short, long)]
		packages: bool,
		/// Whether to only update the modpack
		#[arg(short, long)]
		modpack: bool,
		/// Additional instance groups to update
		#[arg(short, long)]
		groups: Vec<String>,
		/// The instances to update
		instances: Vec<String>,
	},
	#[command(about = "Easily create a new instance")]
	Add,
	#[command(about = "Delete an instance and its files forever")]
	Delete {
		/// The instance to delete
		instance: Option<String>,
	},
	#[command(about = "Edit configuration for an instance")]
	Edit {
		/// The instance to edit
		instance: Option<String>,
	},
	#[command(about = "See which instances are running")]
	Status {},
	#[command(about = "Kill a running instance")]
	Kill {
		/// The ID of the instance to kill
		instance: Option<String>,
		/// The ID of the account that launched this instance
		#[arg(short, long)]
		account: Option<String>,
	},
	#[command(about = "View logs for an instance")]
	Logs {
		/// The instance to view the logs of
		instance: Option<String>,
	},
	#[command(about = "Duplicates an instance into a new one")]
	Duplicate {
		/// The instance to duplicate
		instance: Option<String>,
		/// The ID of the new instance
		new_id: Option<String>,
	},
	#[command(
		about = "Unlink an instance from its parent templates and combine into a single config"
	)]
	Consolidate {
		/// The instance to consolidate
		instance: Option<String>,
	},
	#[command(
		about = "Extract a template from an instance to let you share it with other instances"
	)]
	Extract {
		/// The instance to extract the template from
		instance: Option<String>,
		/// The ID of the new template
		new_id: Option<String>,
	},
	#[command(about = "Print the directory of an instance")]
	Dir {
		/// The instance to print the directory of
		instance: Option<String>,
	},
}

pub async fn run(command: InstanceSubcommand, mut data: CmdData<'_>) -> anyhow::Result<()> {
	match command {
		InstanceSubcommand::List { raw, side } => list(&mut data, raw, side).await,
		InstanceSubcommand::Launch {
			account,
			offline,
			quick_play,
			instance,
		} => launch(instance, account, offline, quick_play, data).await,
		InstanceSubcommand::Info { instance } => info(&mut data, instance).await,
		InstanceSubcommand::Update {
			force,
			all,
			packages,
			modpack,
			groups,
			instances,
		} => update(&mut data, instances, groups, all, force, packages, modpack).await,
		InstanceSubcommand::Dir { instance } => dir(&mut data, instance).await,
		InstanceSubcommand::Add => add(&mut data).await,
		InstanceSubcommand::Delete { instance } => delete(&mut data, instance).await,
		InstanceSubcommand::Edit { instance } => edit(&mut data, instance).await,
		InstanceSubcommand::Duplicate { instance, new_id } => {
			duplicate(&mut data, instance, new_id).await
		}
		InstanceSubcommand::Consolidate { instance } => consolidate(&mut data, instance).await,
		InstanceSubcommand::Extract { instance, new_id } => {
			extract(&mut data, instance, new_id).await
		}
		InstanceSubcommand::Logs { instance } => logs(&mut data, instance).await,
		InstanceSubcommand::Status {} => status(&mut data).await,
		InstanceSubcommand::Kill { instance, account } => kill(&mut data, instance, account).await,
	}
}

async fn list(data: &mut CmdData<'_>, raw: bool, side: Option<Side>) -> anyhow::Result<()> {
	data.ensure_config(!raw).await?;
	let config = data.config.get_mut();

	if raw {
		for (id, instance) in config.instances.iter().sorted_by_key(|x| x.0) {
			if let Some(side) = side
				&& instance.side() != side
			{
				continue;
			}
			println!("{id}");
		}
		return Ok(());
	}

	let mut running_registry = RunningInstanceRegistry::open(&data.paths).ok();

	cprintln!("\n<m,s>╭─ ◆ Sqrilizz Launcher — Список сборок ─────────────────────────────────────────╮</>");
	cprintln!("<m,s>│</>  <s,k!>{:<10}</> <s,w!>{:<26}</> <s,c!>{:<10}</> <s,y!>{:<10}</> <s,g!>{:<8}</> <s,m!>{:<7}</> <m,s>│</>",
		"СТАТУС", "СБОРКА", "ВЕРСИЯ", "ЗАГРУЗЧИК", "ТИП", "ПАКЕТЫ"
	);
	cprintln!("<m,s>├───────────────────────────────────────────────────────────────────────────────┤</>");

	let mut total_count = 0;
	let mut running_count = 0;

	for (id, instance) in config.instances.iter().sorted_by_key(|x| x.0) {
		if let Some(s) = side
			&& instance.side() != s
		{
			continue;
		}

		total_count += 1;
		let is_running = running_registry
			.as_mut()
			.map(|r| r.get_instance(id, None).is_some())
			.unwrap_or(false);

		if is_running {
			running_count += 1;
		}

		let status_text = if is_running { "● В сети" } else { "○ Останов" };

		let name = instance.config().name.as_deref().unwrap_or(id.as_ref());
		let display_name = if name != id.as_ref() {
			format!("{id} ({name})")
		} else {
			id.to_string()
		};

		let name_truncated = if display_name.chars().count() > 26 {
			format!("{}...", display_name.chars().take(23).collect::<String>())
		} else {
			display_name
		};

		let version_text = format!("{}", instance.version());
		let loader_text = format!("{}", instance.loader());
		let side_text = match instance.side() {
			Side::Client => "Client",
			Side::Server => "Server",
		};
		let pkgs_text = format!("{} шт.", instance.packages().len());

		if is_running {
			cprintln!(
				"<m,s>│</>  <g,s>{:<10}</> <w,s>{:<26}</> <c>{:<10}</> <y>{:<10}</> <g>{:<8}</> <m>{:<7}</> <m,s>│</>",
				status_text,
				name_truncated,
				version_text,
				loader_text,
				side_text,
				pkgs_text
			);
		} else {
			cprintln!(
				"<m,s>│</>  <k!>{:<10}</> <w>{:<26}</> <c>{:<10}</> <y>{:<10}</> <k!>{:<8}</> <k!>{:<7}</> <m,s>│</>",
				status_text,
				name_truncated,
				version_text,
				loader_text,
				side_text,
				pkgs_text
			);
		}
	}

	cprintln!("<m,s>╰───────────────────────────────────────────────────────────────────────────────╯</>");
	cprintln!("<k!>  Всего сборок: <w,s>{}</> (активных: <g,s>{}</>, остановлено: <k,s>{}</>)</>\n",
		total_count,
		running_count,
		total_count - running_count
	);

	Ok(())
}

async fn info(data: &mut CmdData<'_>, id: Option<String>) -> anyhow::Result<()> {
	data.ensure_config(true).await?;
	let config = data.config.get_mut();

	let id = pick_instance(id, config)?;
	let instance = config
		.instances
		.get(&id)
		.with_context(|| format!("Unknown instance '{id}'"))?;

	let mut running_registry = RunningInstanceRegistry::open(&data.paths).ok();
	let is_running = running_registry
		.as_mut()
		.map(|r| r.get_instance(&id, None).is_some())
		.unwrap_or(false);

	let status_str = if is_running {
		color_print::cformat!("<g,s>● Запущена (Active)</>")
	} else {
		color_print::cformat!("<k!>○ Остановлена (Stopped)</>")
	};

	let size = instance.get_size().await;
	let size_str = match size {
		Ok(s) => format!("{}", MemoryNum::from_bytes(s)),
		Err(_) => "Неизвестно".into(),
	};

	let instance_dir = data.paths.data.join("instances").join(id.as_ref());
	let name_display = instance.config().name.as_deref().unwrap_or(id.as_ref());

	cprintln!("\n<m,s>╭─ ◆ Информация о сборке: <b>{}</b> ─────────────────────────────╮</>", id);
	cprintln!("<m,s>│</>  <k,s>Статус:</>         {}", status_str);
	cprintln!("<m,s>│</>  <k,s>Название:</>       <w,s>{}</>", name_display);
	cprintln!("<m,s>│</>  <k,s>ID инстанса:</>    <k!>{}</>", id);
	cprintln!("<m,s>│</>  <k,s>Версия игры:</>    <c,s>Minecraft {}</>", instance.version());
	cprintln!("<m,s>│</>  <k,s>Загрузчик:</>      <y,s>{}</>", instance.loader());
	cprintln!("<m,s>│</>  <k,s>Окружение:</>      {}", match instance.side() {
		Side::Client => color_print::cformat!("<g>Клиент (Client)</>"),
		Side::Server => color_print::cformat!("<c>Сервер (Server)</>"),
	});
	cprintln!("<m,s>│</>  <k,s>Размер на диске:</><w>{}</>", size_str);
	cprintln!("<m,s>│</>  <k,s>Папка с файлами:</><c,u>{}</>", instance_dir.display());
	cprintln!("<m,s>├───────────────────────────────────────────────────────────────────────────────┤</>");
	cprintln!("<m,s>│</>  <m,s>Установленные пакеты ({})</>:", instance.packages().len());

	if instance.packages().is_empty() {
		cprintln!("<m,s>│</>    <k!>(нет установленных пакетов)</>");
	} else {
		for pkg in instance.packages().iter().take(15) {
			cprintln!("<m,s>│</>    <m>▸</> <w>{}</>", pkg.req);
		}
		if instance.packages().len() > 15 {
			cprintln!("<m,s>│</>    <k!>... и ещё {} пакетов (sqrilizz pkg list -i {})</>",
				instance.packages().len() - 15, id);
		}
	}

	cprintln!("<m,s>╰───────────────────────────────────────────────────────────────────────────────╯</>");
	cprintln!("<k!>  Команда запуска: <m>sqrilizz launch {}</>\n", id);

	Ok(())
}

pub async fn launch(
	instance: Option<String>,
	account: Option<String>,
	offline: bool,
	quick_play: Option<QuickPlayType>,
	mut data: CmdData<'_>,
) -> anyhow::Result<()> {
	data.ensure_config(true).await?;
	let config = data.config.get_mut();

	let instance_id = pick_instance(instance, config).context("Failed to pick instance")?;

	let client = Client::new();
	let core = config
		.get_core(Some(&get_ms_client_id()), &client, &data.paths, data.output)
		.await?;

	let instance = config
		.instances
		.get_mut(&instance_id)
		.context("Instance does not exist")?;

	let inst_dir = data.paths.data.join("instances").join(instance_id.as_ref());
	let conflicts = sqrilizz::instance_crate::files::check_mod_conflicts(&inst_dir);
	if !conflicts.is_empty() {
		data.output.display(sqrilizz::shared::output::MessageContents::Warning(
			format!("Found {} potential mod conflict(s):", conflicts.len())
		));
		for c in conflicts {
			data.output.display(sqrilizz::shared::output::MessageContents::Simple(
				format!(" - Mod '{}' has duplicate versions: {}", c.mod_name, c.duplicates.join(", "))
			));
		}
	}

	if let Some(account) = account {
		config
			.accounts
			.choose_account(&account)
			.context("Failed to choose account")?;
	}

	let launch_settings = LaunchSettings {
		offline_auth: offline,
		pipe_stdin: true,
		quick_play,
	};

	let mut lock = Lockfile::open(&data.paths)?;

	let mut ctx = InstanceUpdateContext {
		packages: &mut config.packages,
		accounts: &mut config.accounts,
		prefs: &config.prefs,
		paths: &data.paths,
		lock: &mut lock,
		client: &client,
		output: data.output,
		core: &core,
	};

	let mut instance_handle = instance
		.launch(launch_settings, &mut ctx)
		.await
		.context("Instance failed to launch")?;

	// Drop items early so that they aren't wasting memory while the instance is running
	std::mem::drop(data.config);
	lock.finish(&data.paths)?;
	std::mem::drop(lock);
	std::mem::drop(client);

	let format_output = IO_CONFIG.get_bool("format_instance_output").unwrap_or(true);

	if format_output {
		let mut decoder = LineDecoder::new();
		let output_fn = move |chunk: &[u8]| {
			if chunk.is_empty() {
				return;
			}

			let Some(lines) = decoder.decode(chunk, chunk.len()) else {
				return;
			};

			for line in lines {
				instance_stdout_line(&line);
			}
		};

		instance_handle.set_output_fn(Box::new(output_fn));
	}

	instance_handle
		.wait(data.output)
		.await
		.context("Failed to wait for instance child process")?;

	Ok(())
}

async fn dir(data: &mut CmdData<'_>, instance: Option<String>) -> anyhow::Result<()> {
	data.ensure_config(true).await?;
	let config = data.config.get();

	let instance = pick_instance(instance, config).context("Failed to pick instance")?;
	let instance = config
		.instances
		.get(&instance)
		.context("Instance does not exist")?;
	instance.ensure_dir()?;

	if let Some(dir) = instance.dir() {
		println!("{}", dir.to_string_lossy());
	} else {
		bail!("Instance has no directory");
	}

	Ok(())
}

async fn update(
	data: &mut CmdData<'_>,
	instances: Vec<String>,
	groups: Vec<String>,
	all: bool,
	force: bool,
	packages: bool,
	modpack: bool,
) -> anyhow::Result<()> {
	data.ensure_config(true).await?;
	let config = data.config.get_mut();

	let mut ids: Vec<InstanceID> = if all {
		config.instances.keys().cloned().collect()
	} else {
		instances.into_iter().map(InstanceID::from).collect()
	};

	for group in groups {
		let group = Arc::from(group);
		let group = config
			.instance_groups
			.get(&group)
			.with_context(|| format!("Instance group '{group}' does not exist"))?;
		ids.extend(group.clone());
	}

	if ids.is_empty() {
		ids = pick_instances(config)?;
	}

	let client = Client::new();
	let mut lock = Lockfile::open(&data.paths).context("Failed to open lockfile")?;
	let core = config
		.get_core(Some(&get_ms_client_id()), &client, &data.paths, &mut NoOp)
		.await?;

	for id in ids {
		let instance = config
			.instances
			.get_mut(&id)
			.with_context(|| format!("Unknown instance '{id}'"))?;

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

		let depth = if force {
			UpdateDepth::Force
		} else {
			UpdateDepth::Full
		};

		let facets = UpdateFacets::from_flags(packages, modpack);

		instance
			.update(depth, facets, &mut ctx)
			.await
			.context("Failed to update instance")?;

		// Clear the package registry to prevent dependency chains in requests being carried over
		config.packages.clear();

		// Mark the instance as having completed its first update
		lock.update_instance_has_done_first_update(instance.id());
		lock.finish(&data.paths)
			.context("Failed to finish using lockfile")?;
	}

	Ok(())
}

async fn add(data: &mut CmdData<'_>) -> anyhow::Result<()> {
	data.ensure_config(true).await?;
	let config = data.config.get();
	let mut raw_config = data.get_raw_config()?;

	// Build the instance
	let id = pick_instance_id()?;

	let side = pick_side(None)?;

	let client = Client::new();
	let core = config
		.get_core(Some(&get_ms_client_id()), &client, &data.paths, &mut NoOp)
		.await?;
	let manifest = core
		.get_version_manifest(None, UpdateDepth::Shallow, &mut NoOp)
		.await?;

	let version = pick_minecraft_version(&manifest.list).await?;

	let loader = pick_loader(None, Some(side)).await?;

	let instance_config = InstanceConfig {
		side: Some(side),
		version: Some(version.to_serialized()),
		loader: Some(to_string_json(&loader)),
		..Default::default()
	};

	apply_modifications_and_write(
		&mut raw_config,
		vec![ConfigModification::AddInstance(id, instance_config)],
		&data.paths,
		data.output,
	)
	.await
	.context("Failed to write modified config")?;

	data.output
		.display(MessageContents::Success("Instance added".into()));

	Ok(())
}

async fn delete(data: &mut CmdData<'_>, id: Option<String>) -> anyhow::Result<()> {
	data.ensure_config(true).await?;
	let config = data.config.get_mut();

	let id = pick_instance(id, config)?;

	let instance = config
		.instances
		.get_mut(&id)
		.with_context(|| format!("Unknown instance '{id}'"))?;

	let prompt = Confirm::new(
		"Are you SURE you want to delete this instance? This will remove world saves as well! (y/n)",
	);
	if !prompt.prompt()? {
		cprintln!("<r>Cancelled.");
		return Ok(());
	}

	let mut process = data.output.get_process();
	process.display(MessageContents::StartProcess("Deleting instance".into()));

	instance
		.delete(&data.paths, process.deref_mut())
		.await
		.context("Failed to delete instance")?;

	process.display(MessageContents::Success("Instance deleted".into()));

	Ok(())
}

async fn edit(data: &mut CmdData<'_>, id: Option<String>) -> anyhow::Result<()> {
	data.ensure_config(true).await?;
	let mut raw_config = data.get_raw_config()?;
	let config = data.config.get_mut();

	let id = pick_instance(id, config)?;

	let instance = config
		.instances
		.get_mut(&id)
		.with_context(|| format!("Unknown instance '{id}'"))?;

	let inst_config = instance.original_config().clone();

	let text = serde_json::to_string_pretty(&inst_config).context("Failed to serialize config")?;
	let edited = edit_temp_file(&text, &format!("Editing instance {id}"), &data.paths)?;
	let new_config: InstanceConfig = serde_json::from_str(&edited)
		.context("Failed to serialize. Make sure your config is valid JSON")?;

	let modifications = vec![ConfigModification::UpdateInstance(id, new_config)];
	apply_modifications_and_write(
		&mut raw_config,
		modifications,
		&data.paths,
		data.output,
	)
	.await
	.context("Failed to modify and write config")?;

	data.output
		.display(MessageContents::Success("Changes saved".into()));

	Ok(())
}

async fn logs(data: &mut CmdData<'_>, id: Option<String>) -> anyhow::Result<()> {
	data.ensure_config(true).await?;
	let config = data.config.get_mut();

	let id = pick_instance(id, config)?;

	let instance = config
		.instances
		.get_mut(&id)
		.with_context(|| format!("Unknown instance '{id}'"))?;

	let logs = instance
		.get_logs(&data.paths, data.output)
		.await
		.context("Failed to get instance logs")?;

	if logs.is_empty() {
		cprintln!("No logs available");
		return Ok(());
	}

	loop {
		let select = inquire::Select::new("Browsing logs. Press Escape to exit.", logs.clone());
		let log = select.prompt_skippable()?;
		if let Some(log) = log {
			if let Ok(log_text) = instance
				.get_log(&log, &data.paths, data.output)
				.await
			{
				let log_text = log_text.lines().map(format_instance_stdout_line).join("\n");
				cprintln!("<s>Log <g>{log}");
				println!("{log_text}");
			} else {
				cprintln!("<s,r>Failed to read log {log}");
			}
			inquire::Confirm::new("Press Escape to return to browse page").prompt_skippable()?;
		} else {
			break;
		}
	}

	Ok(())
}

async fn duplicate(
	data: &mut CmdData<'_>,
	instance: Option<String>,
	new_id: Option<String>,
) -> anyhow::Result<()> {
	data.ensure_config(true).await?;
	let config = data.config.get();

	let instance = pick_instance(instance, config)?;
	let instance = config
		.instances
		.get(&instance)
		.context("Instance does not exist")?;

	let new_id = if let Some(new_id) = new_id {
		new_id.into()
	} else {
		pick_instance_id()?
	};

	instance
		.duplicate(&new_id, &data.paths, data.output)
		.await?;

	data.output
		.display(MessageContents::Success("Changes saved".into()));

	Ok(())
}

async fn consolidate(data: &mut CmdData<'_>, instance: Option<String>) -> anyhow::Result<()> {
	data.ensure_config(true).await?;
	let config = data.config.get();

	let instance = pick_instance(instance, config)?;
	let instance = config
		.instances
		.get(&instance)
		.context("Instance does not exist")?;

	instance
		.consolidate(&data.paths, data.output)
		.await?;

	data.output
		.display(MessageContents::Success("Changes saved".into()));

	Ok(())
}

async fn extract(
	data: &mut CmdData<'_>,
	instance: Option<String>,
	new_id: Option<String>,
) -> anyhow::Result<()> {
	data.ensure_config(true).await?;
	let config = data.config.get();

	let instance = pick_instance(instance, config)?;
	let instance = config
		.instances
		.get(&instance)
		.context("Instance does not exist")?;

	let new_id = if let Some(new_id) = new_id {
		new_id.into()
	} else {
		pick_instance_id()?
	};

	instance
		.extract(&new_id, &data.paths, data.output)
		.await?;

	data.output
		.display(MessageContents::Success("Changes saved".into()));

	Ok(())
}

async fn status(data: &mut CmdData<'_>) -> anyhow::Result<()> {
	let registry = RunningInstanceRegistry::open(&data.paths).context("Failed to open registry")?;
	let count = registry.iter_entries().count();
	if count == 0 {
		cprintln!("\n<k!>○ Нет запущенных сборок Minecraft</>\n");
	} else {
		cprintln!("\n<m,s>╭─ ◆ Запущенные сборки Minecraft ({count}) ──────────────────────╮</>");
		cprintln!("<m,s>│</>  <s,k!>{:<6}</> <s,w!>{:<30}</> <s,c!>{:<20}</> <s,y!>{:<10}</> <m,s>│</>",
			"PID", "СБОРКА", "АККАУНТ", "СТАТУС"
		);
		cprintln!("<m,s>├───────────────────────────────────────────────────────────────────┤</>");

		for entry in registry.iter_entries() {
			let acc_str = entry.account.as_deref().unwrap_or("-");
			cprintln!(
				"<m,s>│</>  <y>{:<6}</> <g,s>{:<30}</> <w>{:<20}</> <g,s>{:<10}</> <m,s>│</>",
				entry.pid,
				entry.instance_id,
				acc_str,
				"В сети"
			);
		}
		cprintln!("<m,s>╰───────────────────────────────────────────────────────────────────╯</>\n");
	}

	Ok(())
}

async fn kill(
	data: &mut CmdData<'_>,
	instance: Option<String>,
	account: Option<String>,
) -> anyhow::Result<()> {
	data.ensure_config(true).await?;
	let config = data.config.get();

	let instance = pick_instance(instance, config)?;

	let mut registry =
		RunningInstanceRegistry::open(&data.paths).context("Failed to open registry")?;

	registry.kill_instance(&instance, account.as_deref());

	registry.write()?;

	data.output
		.display(MessageContents::Success("Instance killed".into()));

	Ok(())
}
