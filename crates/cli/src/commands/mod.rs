mod account;
mod config;
pub mod dashboard;
mod files;
mod instance;
mod log;
mod modpack;
mod package;
mod template;
mod r#try;
mod version;


use anyhow::{Context, bail};
use clap::{Parser, Subcommand};
use color_print::cformat;

use sqrilizz::config::{Config, is_first_run};
use sqrilizz::config_crate::ConfigDeser;
use sqrilizz::core::QuickPlayType;
use sqrilizz::io::paths::Paths;
use sqrilizz::shared::later::Later;
use sqrilizz::shared::sqrilizz_executable::{SqrilizzClientId, SqrilizzExecutableRegistry};
use sqrilizz::shared::output::{MessageContents, MessageLevel, SqrilizzOutput};

use self::account::AccountSubcommand;
use self::config::ConfigSubcommand;
use self::dashboard::run_dashboard;
use self::files::FilesSubcommand;
use self::instance::InstanceSubcommand;
use self::log::LogSubcommand;
use self::modpack::ModpackSubcommand;
use self::package::PackageSubcommand;
use self::template::TemplateSubcommand;
use self::r#try::TrySubcommand;
use self::version::VersionSubcommand;

use super::output::TerminalOutput;

#[derive(Debug, Subcommand)]
pub enum Command {
	#[command(about = "Открыть интерактивный терминальный дашборд (TUI)")]
	#[clap(alias = "tui", alias = "dash")]
	Dashboard,
	#[command(about = "Запуск сборки Minecraft (онлайн или офлайн)")]
	#[clap(alias = "run", alias = "start", alias = "play")]
	Launch {
		/// Аккаунт для запуска (если не указан, используется основной)
		#[arg(short, long)]
		account: Option<String>,
		/// Офлайн-режим (без проверки серверов авторизации)
		#[arg(short, long)]
		offline: bool,
		/// Быстрый вход в мир или сервер (world:<мир>, server:<ip> или realm:<realm>)
		#[arg(short, long)]
		quick_play: Option<QuickPlayType>,
		/// Имя или ID сборки для запуска
		instance: Option<String>,
	},
	#[command(about = "Управление сборками и инстансами")]
	#[clap(alias = "inst", alias = "i")]
	Instance {
		#[command(subcommand)]
		command: InstanceSubcommand,
	},
	#[command(about = "Управление пакетами, модами, ресурспаками и шейдерами")]
	#[clap(alias = "pkg", alias = "mod", alias = "p")]
	Package {
		#[command(subcommand)]
		command: PackageSubcommand,
	},
	#[command(about = "Импорт и экспорт модпаков (.mrpack / .zip)")]
	#[clap(alias = "pack", alias = "mp")]
	Modpack {
		#[command(subcommand)]
		command: ModpackSubcommand,
	},
	#[command(about = "Управление аккаунтами и авторизацией")]
	#[clap(alias = "acc", alias = "a")]
	Account {
		#[command(subcommand)]
		command: AccountSubcommand,
	},
	#[command(about = "Шаблоны сборок и наследование настроек")]
	#[clap(alias = "temp")]
	Template {
		#[command(subcommand)]
		command: TemplateSubcommand,
	},
	#[command(about = "Быстрый запуск во временном изолированном инстансе")]
	Try {
		#[command(subcommand)]
		command: TrySubcommand,
	},
	#[command(about = "Управление конфигурацией лаунчера")]
	#[clap(alias = "cfg", alias = "conf")]
	Config {
		#[command(subcommand)]
		command: ConfigSubcommand,
	},
	#[command(about = "Просмотр и управление лог-файлами лаунчера")]
	#[clap(alias = "logs")]
	Log {
		#[command(subcommand)]
		command: LogSubcommand,
	},
	#[command(about = "Быстрый доступ к папкам и файлам лаунчера")]
	Files {
		#[command(subcommand)]
		command: FilesSubcommand,
	},
	#[command(about = "Информация о доступных версиях Minecraft")]
	#[command(long_about = "Используйте --version для проверки версии самого лаунчера")]
	#[clap(alias = "ver", alias = "v")]
	Version {
		#[command(subcommand)]
		command: VersionSubcommand,
	},
}

#[derive(Debug, Parser)]
#[command(
	name = "sqrilizz",
	author = "Sqrilizz Team",
	version = "1.0.4",
	about = "Sqrilizz Launcher CLI — консольный интерфейс для Minecraft",
	long_about = "Sqrilizz Launcher CLI — быстрый и удобный консольный лаунчер Minecraft на Rust.\nОфициальный сайт: https://sqrilizz.tech\nРепозиторий: https://github.com/Sqrilizz"
)]
pub struct Cli {
	#[command(subcommand)]
	command: Option<Command>,
	#[arg(short, long, help = "Включить подробный отладочный вывод (debug)")]
	debug: bool,
	#[arg(short = 'D', long, help = "Включить трассировочный вывод (trace)")]
	trace: bool,
	/// Автоматически отвечать "да" на все запросы подтверждения
	#[arg(short, long)]
	yes: bool,
	/// Пропустить начальное приветствие
	#[arg(long)]
	skip_onboarding: bool,
}

pub async fn run_cli() -> anyhow::Result<()> {
	// Parse the CLI
	let cli = Cli::try_parse();
	if let Err(e) = &cli {
		if let clap::error::ErrorKind::DisplayHelp
		| clap::error::ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand
		| clap::error::ErrorKind::DisplayVersion = e.kind()
		{
			crate::output::print_banner();
			println!("{e}");
			return Ok(());
		} else {
			eprintln!("{}", cformat!("<r>{e}"));
			bail!("");
		}
	}
	let cli = cli?;

	let log_level = get_log_level(&cli);

	// Prepare the command data
	let paths = Paths::new()
		.await
		.context("Failed to set up system paths")?;

	let command = match cli.command {
		Some(cmd) => cmd,
		None => {
			if let Some(target_instance) = run_dashboard(&paths).await? {
				Command::Launch {
					account: None,
					offline: false,
					quick_play: None,
					instance: Some(target_instance),
				}
			} else {
				return Ok(());
			}
		}
	};

	let mut output = TerminalOutput::new(&paths).context("Failed to set up output")?;

	if let Ok(mut exec_registry) = SqrilizzExecutableRegistry::open(&paths.internal) {
		let _ = exec_registry.add_this(SqrilizzClientId::Cli);
	}

	let _ = write_extra_commands_file(&paths);

	if is_first_run(&paths) && !cli.skip_onboarding {
		crate::output::print_banner();
		output.display(MessageContents::Header("Добро пожаловать в Sqrilizz Launcher CLI!".into()));
	}

	let res = {
		let mut data = CmdData::new(paths, &mut output)?;
		data.output.set_log_level(log_level);
		data.output.set_auto_yes(cli.yes);

		match command {
			Command::Dashboard => {
				drop(data);
				let p = Paths::new().await?;
				if let Some(target_instance) = run_dashboard(&p).await? {
					let mut new_output = TerminalOutput::new(&p).context("Failed to set up output")?;
					let new_data = CmdData::new(p, &mut new_output)?;
					instance::launch(Some(target_instance), None, false, None, new_data).await
				} else {
					Ok(())
				}
			}
			Command::Account { command } => account::run(command, &mut data).await,
			Command::Launch {
				account,
				offline,
				quick_play,
				instance,
			} => instance::launch(instance, account, offline, quick_play, data).await,
			Command::Files { command } => files::run(command, &mut data).await,
			Command::Package { command } => package::run(command, data).await,
			Command::Instance { command } => instance::run(command, data).await,
			Command::Config { command } => config::run(command, &mut data).await,
			Command::Template { command } => template::run(command, &mut data).await,
			Command::Modpack { command } => modpack::run(command, &mut data).await,
			Command::Log { command } => log::run(command, &mut data).await,
			Command::Try { command } => r#try::run(command, &mut data).await,
			Command::Version { command } => version::run(command, &mut data).await,
		}
	};

	if let Err(e) = &res {
		// Don't use the existing process or section
		output.end_process();
		output.end_section();
		output.display(MessageContents::Error(format!("{e:?}")));
	}

	output.shutdown().await;

	res
}

/// Get the log level based on the debug options
fn get_log_level(cli: &Cli) -> MessageLevel {
	if cli.trace {
		MessageLevel::Trace
	} else if cli.debug {
		MessageLevel::Debug
	} else {
		MessageLevel::Important
	}
}

/// Data passed to commands
pub struct CmdData<'a> {
	pub paths: Paths,
	pub config: Later<Config>,
	pub output: &'a mut TerminalOutput,
}

impl<'a> CmdData<'a> {
	pub fn new(paths: Paths, output: &'a mut TerminalOutput) -> anyhow::Result<Self> {
		Ok(Self {
			paths,
			config: Later::new(),
			output,
		})
	}

	/// Ensure that the config is loaded
	pub async fn ensure_config(&mut self, show_warnings: bool) -> anyhow::Result<()> {
		if self.config.is_empty() {
			self.config.fill(
				Config::load(
					&Config::get_path(&self.paths),
					show_warnings,
					&self.paths,
					crate::secrets::get_ms_client_id(),
					self.output,
				)
				.await
				.context("Failed to load config")?,
			);
		}

		Ok(())
	}

	/// Get the raw deserialized config
	pub fn get_raw_config(&self) -> anyhow::Result<ConfigDeser> {
		let config =
			Config::open(&Config::get_path(&self.paths)).context("Failed to open config")?;

		Ok(config)
	}
}

/// Writes a file that can be sourced to provide extra shell commands
fn write_extra_commands_file(paths: &Paths) -> anyhow::Result<()> {
	let commands = r#"
sqrilizz-instance-cd () {
	cd $(sqrilizz inst dir $1)
}
sqrilizz-inst-cd () {
	cd $(sqrilizz inst dir $1)
}
	"#;

	let file = paths.internal.join("commands.sh");
	std::fs::write(file, commands).context("Failed to write")
}
