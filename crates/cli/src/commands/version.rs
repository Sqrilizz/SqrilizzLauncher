use clap::Subcommand;
use color_print::{cprint, cprintln};
use sqrilizz::shared::{UpdateDepth, minecraft::VersionType};
use reqwest::Client;

use crate::{
	commands::CmdData,
	output::HYPHEN_POINT,
};

#[derive(Debug, Subcommand)]
pub enum VersionSubcommand {
	#[command(about = "List available Minecraft versions")]
	List {
		#[arg(short, long)]
		all: bool,
		#[arg(short, long)]
		release: bool,
		#[arg(short, long)]
		snapshot: bool,
	},
	#[command(about = "Check for launcher self-updates")]
	CheckUpdate,
}

pub async fn run(command: VersionSubcommand, data: &mut CmdData<'_>) -> anyhow::Result<()> {
	match command {
		VersionSubcommand::List {
			all,
			release,
			snapshot,
		} => list(data, all, release, snapshot).await,
		VersionSubcommand::CheckUpdate => check_self_update(data).await,

	}
}

async fn list(
	data: &mut CmdData<'_>,
	all: bool,
	release: bool,
	snapshot: bool,
) -> anyhow::Result<()> {
	data.ensure_config(true).await?;
	let config = data.config.get();

	let client = Client::new();

	let core = config.get_core(None, &client, &data.paths, data.output).await?;

	let versions = core
		.get_version_manifest(None, UpdateDepth::Full, data.output)
		.await?;

	let all_false = !release && !snapshot;
	let release = release || all_false;
	let snapshot = snapshot || all_false;

	let limit = if all { None } else { Some(30) };

	let versions: Vec<_> = versions
		.manifest
		.versions
		.iter()
		.filter(|x| match &x.ty {
			VersionType::Release if !release => false,
			VersionType::Snapshot if !snapshot => false,
			_ => true,
		})
		.take(limit.unwrap_or(versions.manifest.versions.len()))
		.collect();

	for version in versions.into_iter().rev() {
		cprint!("{HYPHEN_POINT}");
		match &version.ty {
			VersionType::Release => cprint!("[<s,g>Release] "),
			VersionType::Snapshot => cprint!("[<s,y>Snapshot] "),
			VersionType::OldAlpha => cprint!("[<s,b>Old Alpha] "),
			VersionType::OldBeta => cprint!("[<s,b>Old Beta] "),
			VersionType::Other(ty) => cprint!("[<s>{ty}] "),
		}

		cprintln!("{}", version.id);
	}

	Ok(())
}

async fn check_self_update(_data: &mut CmdData<'_>) -> anyhow::Result<()> {
	let current = env!("CARGO_PKG_VERSION");
	cprintln!("<g>Current Sqrilizz Launcher version: <b>{current}</>");
	cprintln!("Checking GitHub releases for updates...");

	let client = reqwest::Client::builder()
		.user_agent("SqrilizzLauncher")
		.build()?;

	let url = "https://api.github.com/repos/Sqrilizz/SqrilizzLauncher/releases?per_page=5";
	match client.get(url).send().await {
		Ok(resp) if resp.status().is_success() => {
			if let Ok(json_arr) = resp.json::<Vec<serde_json::Value>>().await
				&& let Some(json) = json_arr.into_iter().next()
			{
				let tag = json.get("tag_name").and_then(|x| x.as_str()).unwrap_or("unknown");
				let tag_clean = tag.trim_start_matches('v');
				if tag_clean != current && tag != "unknown" {
					cprintln!("<y>A new version is available: <b>{tag}</> (current: {current})");
					if let Some(html_url) = json.get("html_url").and_then(|x| x.as_str()) {
						cprintln!("Download update at: <cyan><u>{html_url}</>");
					}
				} else {
					cprintln!("<g>Sqrilizz Launcher is up to date!</>");
				}
			}
		}
		Ok(resp) => {
			cprintln!("<y>Could not check for updates: HTTP status {}</>", resp.status());
		}
		Err(e) => {
			cprintln!("<y>Could not reach update server: {e}</>");
		}
	}
	Ok(())
}
