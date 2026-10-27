use super::CmdData;
use crate::output::{CHECK, HYPHEN_POINT};
use crate::prompt::pick_account;
use anyhow::{Context, bail};
use itertools::Itertools;
use sqrilizz::config::modifications::{ConfigModification, apply_modifications_and_write};
use sqrilizz::config_crate::account::{AccountConfig, AccountVariant};
use sqrilizz::core::account::AccountKind;

use clap::Subcommand;
use color_print::{cformat, cprint, cprintln};
use reqwest::Client;
use sqrilizz::net_crate::load_from_uri;
use sqrilizz::shared::minecraft::{CosmeticState, SkinVariant};
use sqrilizz::shared::output::{MessageContents, SqrilizzOutput};

#[derive(Debug, Subcommand)]
pub enum AccountSubcommand {
	#[command(about = "List all accounts")]
	#[clap(alias = "ls")]
	List {
		/// Whether to remove formatting and warnings from the output
		#[arg(short, long)]
		raw: bool,
	},
	#[command(about = "Switch to another default account")]
	Switch { account: Option<String> },
	#[command(about = "Get current authentication status")]
	Status,
	#[command(about = "Update the passkey for an account")]
	Passkey {
		/// The account to update the passkey for. If not specified, uses the default account
		account: Option<String>,
	},
	#[command(about = "Log in an account")]
	Login {
		/// The account to authenticate. If not specified, uses the default account
		account: Option<String>,
	},
	#[command(about = "Log out an account")]
	Logout {
		/// The account to log out. If not specified, uses the default account
		account: Option<String>,
	},
	#[command(about = "Add new accounts to your config")]
	Add {},
	#[command(about = "Get or set skins and capes")]
	Cosmetic {
		#[command(subcommand)]
		subcommand: CosmeticSubcommand,
	},
}

#[derive(Debug, Subcommand)]
pub enum CosmeticSubcommand {
	#[command(about = "List cosmetics")]
	List {
		/// The account to use. If not specified, uses the default account
		account: Option<String>,
	},
	#[command(about = "Upload a new skin to an account")]
	Upload {
		/// The account to use. If not specified, uses the default account
		account: String,
		/// The path or URL to the skin file. It must be in PNG format.
		skin: String,
		/// Whether this is a slim (Alex-like) skin
		#[arg(long)]
		slim: bool,
	},
}

pub async fn run(subcommand: AccountSubcommand, data: &mut CmdData<'_>) -> anyhow::Result<()> {
	match subcommand {
		AccountSubcommand::List { raw } => list(data, raw).await,
		AccountSubcommand::Switch { account } => switch(data, account).await,
		AccountSubcommand::Status => status(data).await,
		AccountSubcommand::Passkey { account } => passkey(data, account).await,
		AccountSubcommand::Login { account } => login(data, account).await,
		AccountSubcommand::Logout { account } => logout(data, account).await,
		AccountSubcommand::Add {} => add(data).await,
		AccountSubcommand::Cosmetic { subcommand } => match subcommand {
			CosmeticSubcommand::List { account } => cosmetic_list(data, account).await,
			CosmeticSubcommand::Upload {
				account,
				skin,
				slim,
			} => cosmetic_upload(data, account, skin, slim).await,
		},
	}
}

async fn list(data: &mut CmdData<'_>, raw: bool) -> anyhow::Result<()> {
	data.ensure_config(!raw).await?;
	let config = data.config.get();

	if raw {
		for (id, _) in config.accounts.iter_accounts().sorted_by_key(|x| x.0) {
			println!("{id}");
		}
		return Ok(());
	}

	cprintln!("
<m,s>╭─ ◆ Sqrilizz Launcher — Список аккаунтов ──────────────────────────────────────╮</>");
	cprintln!("<m,s>│</>  <s,k!>{:<5}</> <s,w!>{:<26}</> <s,c!>{:<20}</> <s,g!>{:<22}</> <m,s>│</>",
		"АКТ.", "НИКНЕЙМ / ID", "ТИП АККАУНТА", "СТАТУС"
	);
	cprintln!("<m,s>├───────────────────────────────────────────────────────────────────────────────┤</>");

	let chosen_id = config
		.accounts
		.get_chosen_account()
		.map(|x| x.get_id().to_string());
	let mut count = 0;

	for (id, account) in config.accounts.iter_accounts().sorted_by_key(|x| x.0) {
		count += 1;
		let is_default = chosen_id.as_deref() == Some(id);
		let star_str = if is_default { " ★" } else { "  " };

		let kind_str = match account.get_kind() {
			AccountKind::Microsoft { .. } => "Microsoft",
			AccountKind::Demo => "Демо-режим",
			AccountKind::Unknown(other) => other,
		};

		let status_str = if is_default {
			"[По умолчанию]"
		} else {
			match account.get_kind() {
				AccountKind::Microsoft { .. } => "Авторизован",
				AccountKind::Demo => "Ограничен",
				AccountKind::Unknown(..) => "Офлайн",
			}
		};

		if is_default {
			cprintln!(
				"<m,s>│</>  <y,s>{:<5}</> <w,s>{:<26}</> <c>{:<20}</> <g,s>{:<22}</> <m,s>│</>",
				star_str,
				id,
				kind_str,
				status_str
			);
		} else {
			cprintln!(
				"<m,s>│</>  <k!>{:<5}</> <w>{:<26}</> <c>{:<20}</> <k!>{:<22}</> <m,s>│</>",
				star_str,
				id,
				kind_str,
				status_str
			);
		}
	}

	cprintln!("<m,s>╰───────────────────────────────────────────────────────────────────────────────╯</>");
	cprintln!("<k!>  Всего аккаунтов: <w,s>{}</></>
", count);

	Ok(())
}

async fn switch(data: &mut CmdData<'_>, account: Option<String>) -> anyhow::Result<()> {
	data.ensure_config(true).await?;
	let mut raw_config = data.get_raw_config()?;

	let account = pick_account(account, data.config.get())?;
	raw_config.default_account = Some(account.to_string());

	apply_modifications_and_write(&mut raw_config, Vec::new(), &data.paths, data.output).await?;

	Ok(())
}

async fn status(data: &mut CmdData<'_>) -> anyhow::Result<()> {
	data.ensure_config(true).await?;
	let config = data.config.get();

	match config.accounts.get_chosen_account() {
		Some(account) => {
			let account_valid = account.is_auth_valid(&data.paths.core);
			if account_valid {
				cprint!("<g>Logged in as ");
			} else {
				cprint!("<g>Account chosen as ");
			}
			match account.get_kind() {
				AccountKind::Microsoft { .. } => cprint!("<s,g!>{}", account.get_id()),
				AccountKind::Demo => cprint!("<s,c!>{}", account.get_id()),
				AccountKind::Unknown(other) => cprint!("<s,k!>({other}) {}", account.get_id()),
			}

			if !account_valid {
				cprint!(" - <r>Currently logged out");
			}
			cprintln!();
		}
		None => cprintln!("<r>No account chosen"),
	}

	Ok(())
}

async fn passkey(data: &mut CmdData<'_>, account: Option<String>) -> anyhow::Result<()> {
	data.ensure_config(true).await?;
	let config = data.config.get();
	let account = if let Some(account) = account {
		config.accounts.get_account(&account)
	} else {
		config.accounts.get_chosen_account()
	};
	let Some(account) = account else {
		bail!("Specified account does not exist");
	};

	account
		.update_passkey(&data.paths.core, data.output)
		.await
		.context("Failed to update passkey")?;

	Ok(())
}

async fn login(data: &mut CmdData<'_>, account: Option<String>) -> anyhow::Result<()> {
	data.ensure_config(true).await?;
	let config = data.config.get_mut();
	if let Some(account) = account {
		config.accounts.choose_account(&account)?;
	}

	let client = Client::new();
	config
		.accounts
		.authenticate(false, &data.paths.core, &client, data.output)
		.await
		.context("Failed to authenticate")?;

	Ok(())
}

async fn logout(data: &mut CmdData<'_>, account: Option<String>) -> anyhow::Result<()> {
	data.ensure_config(true).await?;
	let config = data.config.get_mut();
	let account = if let Some(account) = account {
		config.accounts.get_account_mut(&account)
	} else {
		config.accounts.get_chosen_account_mut()
	};
	let Some(account) = account else {
		bail!("Specified account does not exist");
	};

	account
		.logout(&data.paths.core)
		.context("Failed to logout account")?;

	Ok(())
}

async fn add(data: &mut CmdData<'_>) -> anyhow::Result<()> {
	data.ensure_config(true).await?;
	let mut config = data.get_raw_config()?;

	// Build the account
	let id = inquire::Text::new("What is the ID for the account?").prompt()?;

	let options = vec![AccountVariant::Microsoft {}, AccountVariant::Demo {}];
	let kind = inquire::Select::new("What kind of account is this?", options).prompt()?;

	let account = AccountConfig::Simple(kind);

	apply_modifications_and_write(
		&mut config,
		vec![ConfigModification::AddAccount(id, account)],
		&data.paths,
		data.output,
	)
	.await
	.context("Failed to write modified config")?;

	data.output
		.display(MessageContents::Success("Account added".into()));

	Ok(())
}

async fn cosmetic_list(data: &mut CmdData<'_>, account: Option<String>) -> anyhow::Result<()> {
	data.ensure_config(true).await?;
	let config = data.config.get_mut();
	if let Some(account) = account {
		config.accounts.choose_account(&account)?;
	}

	let client = Client::new();
	let (skins, capes) = config
		.accounts
		.get_cosmetics(&data.paths.core, &client, data.output)
		.await
		.context("Failed to get cosmetics")?;

	if !skins.is_empty() {
		cprintln!("<s,m>Skins:");
		for skin in skins {
			let line = cformat!("<m>{}", skin.cosmetic.id);
			let line = match skin.cosmetic.state {
				CosmeticState::Active => cformat!("{line} <s,g>{CHECK} Selected"),
				CosmeticState::Inactive => line,
			};
			println!("{HYPHEN_POINT}{line}");
		}
	}
	if !capes.is_empty() {
		cprintln!("<s,y>Capes:");
		for cape in capes {
			let line = cformat!("<y>{} - {}", cape.alias, cape.cosmetic.id);
			let line = match cape.cosmetic.state {
				CosmeticState::Active => cformat!("{line} <s,g>{CHECK} Selected"),
				CosmeticState::Inactive => line,
			};
			println!("{HYPHEN_POINT}{line}");
		}
	}

	Ok(())
}

async fn cosmetic_upload(
	data: &mut CmdData<'_>,
	account: String,
	skin: String,
	slim: bool,
) -> anyhow::Result<()> {
	data.ensure_config(true).await?;
	let config = data.config.get_mut();

	let client = Client::new();

	let skin = load_from_uri(&skin, &client)
		.await
		.context("Failed to load skin")?;

	let variant = if slim {
		SkinVariant::Slim
	} else {
		SkinVariant::Classic
	};

	config
		.accounts
		.upload_skin(
			&account,
			variant,
			&skin,
			&data.paths.core,
			&client,
			data.output,
		)
		.await
		.context("Failed to upload skin")?;

	data.output
		.display(MessageContents::Success("Skin uploaded".into()));

	Ok(())
}
