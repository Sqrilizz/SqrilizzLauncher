use super::CmdData;

use anyhow::Context;
use clap::Subcommand;
use sqrilizz::core::io::files::create_leading_dirs;
use sqrilizz::shared::output::{MessageContents, SqrilizzOutput};
use sqrilizz::{config::Config, io::paths::Paths};

use std::{path::PathBuf, process::Command};

#[derive(Debug, Subcommand)]
pub enum ConfigSubcommand {
	#[command(about = "Edit config using your default text editor")]
	Edit,
	#[command(about = "Backup configuration files to identical copies")]
	Backup,
}

pub async fn run(subcommand: ConfigSubcommand, data: &mut CmdData<'_>) -> anyhow::Result<()> {
	match subcommand {
		ConfigSubcommand::Edit => edit(data).await,
		ConfigSubcommand::Backup => backup(data).await,
	}
}

async fn edit(data: &mut CmdData<'_>) -> anyhow::Result<()> {
	let path = Config::get_path(&data.paths);

	Config::create_default(&path).context("Failed to create default config")?;

	edit_text(path).context("Failed to edit config")?;

	data.output
		.display(MessageContents::Success("Changes saved".into()));

	Ok(())
}

async fn backup(data: &mut CmdData<'_>) -> anyhow::Result<()> {
	let config_path = Config::get_path(&data.paths);

	Config::create_default(&config_path).context("Failed to create default config")?;

	let mut backup_config_path = config_path.clone();
	backup_config_path.set_extension("json.bak");
	std::fs::copy(config_path, backup_config_path).context("Failed to backup config file")?;

	data.output
		.display(MessageContents::Success("Backup created".into()));

	Ok(())
}

/// Creates a temporary file with the given text and opens it in the user's editor,
/// returning the resulting edited text
pub fn edit_temp_file(text: &str, title: &str, paths: &Paths) -> anyhow::Result<String> {
	let path = paths.config.join("temp").join(title);
	create_leading_dirs(&path)?;

	std::fs::write(&path, text).context("Failed to write to temp file")?;

	edit_text(path.clone()).context("Failed to edit")?;

	std::fs::read_to_string(&path).context("Failed to read resulting edit file")
}

/// Run the text editor on the user's system
fn edit_text(path: PathBuf) -> anyhow::Result<()> {
	#[cfg(target_os = "linux")]
	let mut command = {
		let editor = std::env::var("EDITOR").unwrap_or_else(|_| {
			// Pick the best Vim-style editor based on which one is available
			if which::which("nvim").is_ok_and(|path| path.exists()) {
				"nvim".into()
			} else {
				"vim".into()
			}
		});
		Command::new(editor)
	};
	#[cfg(target_os = "windows")]
	let mut command = Command::new("notepad");
	#[cfg(target_os = "macos")]
	let mut command = Command::new("open");
	#[cfg(target_os = "macos")]
	command.arg("-t");

	command.arg(path);

	command.spawn()?.wait()?;

	Ok(())
}
