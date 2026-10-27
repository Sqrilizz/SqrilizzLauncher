use anyhow::{Context, bail};
use sqrilizz_auth::RsaPrivateKey;
use sqrilizz_shared::minecraft::MinecraftUserProfile;
use sqrilizz_shared::output::{MessageContents, SqrilizzOutput};
use sqrilizz_shared::translate;
use sqrilizz_shared::util::utc_timestamp;

use crate::Paths;
use sqrilizz_auth::db::{AuthDatabase, DatabaseAccount, SensitiveAccountInfo};
use sqrilizz_auth::mc::Keypair;
use sqrilizz_auth::mc::{
	self as auth, AccessToken, ClientId, RefreshToken, authenticate_microsoft_account,
	authenticate_microsoft_account_from_token,
};

use super::{Account, AccountKind};

impl Account {
	pub(crate) async fn authenticate(
		&mut self,
		params: AuthParameters<'_>,
		o: &mut impl SqrilizzOutput,
	) -> anyhow::Result<()> {
		match &mut self.kind {
			AccountKind::Microsoft { xbox_uid } => {
				if params.offline {
					let db = AuthDatabase::open(&params.paths.auth)
						.context("Failed to open authentication database")?;
					let Some((account, sensitive)) = get_full_account(&db, &self.id, o)
						.await
						.context("Failed to get account from database")?
					else {
						bail!(
							"Account not present in database. Make sure to authenticate at least once before logging in in offline mode"
						);
					};

					self.name = Some(account.username.clone());
					self.uuid = Some(account.uuid.clone());
					self.keypair = sensitive.keypair.clone();
					*xbox_uid = sensitive.xbox_uid.clone();
				} else {
					let account_data = update_microsoft_account_auth(&self.id, params, o)
						.await
						.context("Failed to update account authentication")?;

					self.access_token = Some(account_data.access_token);
					self.name = Some(account_data.profile.name);
					self.uuid = Some(account_data.profile.uuid);
					self.keypair = account_data.keypair;
					*xbox_uid = account_data.xbox_uid;
				}
			}
			AccountKind::Demo => {}
			AccountKind::Unknown(other) => {
				if other == "offline" {
					self.name = Some(self.id.to_string());
					let mut bytes = [0u8; 16];
					for (i, b) in format!("OfflinePlayer:{}", self.id).bytes().enumerate() {
						bytes[i % 16] = bytes[i % 16].wrapping_add(b);
					}
					self.uuid = Some(bytes.iter().map(|b| format!("{b:02x}")).collect());
					return Ok(());
				}
				o.debug(MessageContents::Simple(
					"Authentication for custom account type not handled".into(),
				));
			}
		}

		Ok(())
	}

	/// Checks if the authentication of this account is still valid
	pub fn is_auth_valid(&self, paths: &Paths) -> bool {
		match &self.kind {
			AccountKind::Microsoft { .. } => {
				let Ok(db) = AuthDatabase::open(&paths.auth) else {
					return false;
				};

				db.get_valid_account(&self.id)
					.is_some_and(|x| x.is_logged_in())
			}
			AccountKind::Demo => true,
			AccountKind::Unknown(..) => true,
		}
	}

	/// Checks if this account is authenticated
	pub fn is_authenticated(&self) -> bool {
		match &self.kind {
			AccountKind::Microsoft { .. } => self.access_token.is_some() && self.uuid.is_some(),
			AccountKind::Demo => true,
			AccountKind::Unknown(..) => true,
		}
	}

	/// Updates the passkey of this account
	pub async fn update_passkey(
		&self,
		paths: &Paths,
		o: &mut impl SqrilizzOutput,
	) -> anyhow::Result<()> {
		let mut db =
			AuthDatabase::open(&paths.auth).context("Failed to open authentication database")?;
		let account = db.get_account_mut(&self.id).context(
			"Account does not exist in database. Try authenticating first before setting a passkey",
		)?;
		let old_passkey = if account.has_passkey() {
			Some(
				o.prompt_password(MessageContents::Simple(format!(
					"Enter the old passkey for account '{}'",
					self.id
				)))
				.await
				.context("Failed to get old passkey")?,
			)
		} else {
			None
		};
		let new_passkey = o
			.prompt_new_password(MessageContents::Simple(format!(
				"Enter the new passkey for account '{}'",
				self.id
			)))
			.await
			.context("Failed to get new passkey")?;
		account
			.update_passkey(old_passkey.as_deref(), &new_passkey)
			.context("Failed to update passkey for account")?;

		db.write()
			.context("Failed to write to authentication database")?;

		Ok(())
	}

	/// Logs this account out
	pub fn logout(&mut self, paths: &Paths) -> anyhow::Result<()> {
		let mut db =
			AuthDatabase::open(&paths.auth).context("Failed to open authentication database")?;
		db.logout_account(&self.id)
			.context("Failed to logout account in database")?;

		db.write()
			.context("Failed to write authentication database")?;

		Ok(())
	}
}

pub(crate) struct MicrosoftAccountData {
	pub access_token: AccessToken,
	pub profile: MinecraftUserProfile,
	pub xbox_uid: Option<String>,
	pub keypair: Option<Keypair>,
}

pub(crate) async fn update_microsoft_account_auth(
	account_id: &str,
	params: AuthParameters<'_>,
	o: &mut impl SqrilizzOutput,
) -> anyhow::Result<MicrosoftAccountData> {
	let mut db =
		AuthDatabase::open(&params.paths.auth).context("Failed to open authentication database")?;

	if params.force {
		return reauth_microsoft_account(
			account_id,
			&mut db,
			params.client_id,
			params.req_client,
			o,
		)
		.await;
	}

	let account_data = if let Some((db_account, sensitive)) = get_full_account(&db, account_id, o)
		.await
		.context("Failed to get full account from database")?
	{
		let db_account = db_account.clone();

		let access_token = if let (Some(access_token), Some(expiration)) =
			(&sensitive.access_token, &sensitive.access_token_expires)
		{
			if utc_timestamp().unwrap_or(u64::MAX) < *expiration {
				AccessToken(access_token.clone())
			} else {
				update_using_refresh_token(account_id, &sensitive, &params, &mut db, o)
					.await
					.context("Failed to refresh authentication")?
			}
		} else {
			update_using_refresh_token(account_id, &sensitive, &params, &mut db, o)
				.await
				.context("Failed to refresh authentication")?
		};

		MicrosoftAccountData {
			access_token,
			profile: MinecraftUserProfile {
				name: db_account.username.clone(),
				uuid: db_account.uuid.clone(),
				skins: Vec::new(),
				capes: Vec::new(),
			},
			xbox_uid: sensitive.xbox_uid.clone(),
			keypair: sensitive.keypair.clone(),
		}
	} else {
		reauth_microsoft_account(account_id, &mut db, params.client_id, params.req_client, o)
			.await?
	};

	Ok(account_data)
}

async fn update_using_refresh_token(
	account_id: &str,
	sensitive: &SensitiveAccountInfo,
	params: &AuthParameters<'_>,
	db: &mut AuthDatabase,
	o: &mut impl SqrilizzOutput,
) -> anyhow::Result<AccessToken> {
	let refresh_token = RefreshToken::new(
		sensitive
			.refresh_token
			.clone()
			.expect("Refresh token should be present in a full valid account"),
	);
	let oauth_client =
		auth::create_client(params.client_id.clone()).context("Failed to create OAuth client")?;
	let token = auth::refresh_microsoft_token(&oauth_client, &refresh_token)
		.await
		.context("Failed to get refreshed token")?;

	let auth_result = authenticate_microsoft_account_from_token(token, params.req_client, o)
		.await
		.context("Failed to authenticate with refreshed token")?;

	let mut db_account = db
		.get_account(account_id)
		.context("Failed to get account from database")?
		.clone();

	let mut sensitive = get_sensitive_info(&db_account, o)
		.await
		.context("Failed to get sensitive info")?;
	sensitive.access_token = Some(auth_result.access_token.0.clone());
	sensitive.access_token_expires = utc_timestamp().map(|x| x + 24 * 3600).ok();
	db_account
		.set_sensitive_info(sensitive)
		.context("Failed to set sensitive info for account")?;

	db.update_account(db_account, account_id)
		.context("Failed to update account in database")?;

	Ok(AccessToken(auth_result.access_token.0.clone()))
}

async fn reauth_microsoft_account(
	account_id: &str,
	db: &mut AuthDatabase,
	client_id: ClientId,
	client: &reqwest::Client,
	o: &mut impl SqrilizzOutput,
) -> anyhow::Result<MicrosoftAccountData> {
	let auth_result = authenticate_microsoft_account(client_id, client, o)
		.await
		.context("Failed to authenticate account")?;

	let ownership_task = {
		let client = client.clone();
		let token = auth_result.access_token.0.clone();
		async move {
			let owns_game = auth::account_owns_game(&token, &client)
				.await
				.context("Failed to check for game ownership")?;

			Ok::<bool, anyhow::Error>(owns_game)
		}
	};

	let profile_task = {
		let client = client.clone();
		let token = auth_result.access_token.0.clone();
		async move {
			let profile = crate::net::minecraft::get_user_profile(&token, &client)
				.await
				.context("Failed to get Microsoft account profile")?;

			Ok::<MinecraftUserProfile, anyhow::Error>(profile)
		}
	};

	let certificate_task = {
		let client = client.clone();
		let token = auth_result.access_token.0.clone();
		async move {
			let certificate = crate::net::minecraft::get_user_certificate(&token, &client)
				.await
				.context("Failed to get user certificate")?;

			Ok(certificate)
		}
	};

	let (owns_game, profile, certificate) =
		tokio::try_join!(ownership_task, profile_task, certificate_task)?;

	if !owns_game {
		bail!("Specified account does not own Minecraft");
	}

	let expiration_time = sqrilizz_auth::db::calculate_expiration_date();

	let sensitive = SensitiveAccountInfo {
		refresh_token: auth_result.refresh_token.map(|x| x.secret().clone()),
		xbox_uid: Some(auth_result.xbox_uid.clone()),
		keypair: Some(certificate.key_pair.clone()),
		access_token: Some(auth_result.access_token.0.clone()),
		access_token_expires: utc_timestamp().map(|x| x + 24 * 3600).ok(),
	};
	let db_account = DatabaseAccount::new(
		account_id.to_string(),
		profile.name.clone(),
		profile.uuid.clone(),
		expiration_time,
		sensitive,
	)
	.context("Failed to create new account in database")?;

	db.update_account(db_account, account_id)
		.context("Failed to update account in database")?;

	Ok(MicrosoftAccountData {
		access_token: auth_result.access_token,
		xbox_uid: Some(auth_result.xbox_uid),
		profile,
		keypair: Some(certificate.key_pair),
	})
}

async fn get_full_account<'db>(
	db: &'db AuthDatabase,
	account_id: &str,
	o: &mut impl SqrilizzOutput,
) -> anyhow::Result<Option<(&'db DatabaseAccount, SensitiveAccountInfo)>> {
	let Some(account) = db.get_valid_account(account_id) else {
		return Ok(None);
	};
	if !account.is_logged_in() {
		return Ok(None);
	}

	let sensitive = get_sensitive_info(account, o)
		.await
		.context("Failed to get sensitive information")?;
	if sensitive.refresh_token.is_none() {
		return Ok(None);
	}

	Ok(Some((account, sensitive)))
}

async fn get_sensitive_info(
	db_account: &DatabaseAccount,
	o: &mut impl SqrilizzOutput,
) -> anyhow::Result<SensitiveAccountInfo> {
	let out = if db_account.has_passkey() {
		let private_key = get_private_key(
			db_account,
			MessageContents::Simple(format!(
				"Please enter the passkey for the account '{}'",
				db_account.id
			)),
			o,
		)
		.await
		.context("Failed to get key")?;

		let out = db_account
			.get_sensitive_info_with_key(&private_key)
			.context("Failed to get sensitive account info using key")?;
		o.display(MessageContents::Success(translate!(o, PasskeyAccepted)));

		out
	} else {
		db_account
			.get_sensitive_info_no_passkey()
			.context("Failed to get sensitive account info without key")?
	};

	Ok(out)
}

async fn get_private_key(
	account: &DatabaseAccount,
	message: MessageContents,
	o: &mut impl SqrilizzOutput,
) -> anyhow::Result<RsaPrivateKey> {
	const MAX_ATTEMPTS: u8 = 3;

	for _ in 0..MAX_ATTEMPTS {
		let result = o
			.prompt_special_account_passkey(message.clone(), &account.id)
			.await;
		if let Ok(passkey) = result {
			let result = account.get_private_key(&passkey);
			match result {
				Ok(private_key) => {
					return Ok(private_key.expect("Account should have passkey"));
				}
				Err(e) => {
					o.display(MessageContents::Error(format!("{e:?}")));
				}
			}
		}
	}

	bail!("Passkey authentication failed; max attempts exceeded")
}

pub(crate) struct AuthParameters<'a> {
	pub force: bool,
	pub offline: bool,
	pub client_id: ClientId,
	pub paths: &'a Paths,
	pub req_client: &'a reqwest::Client,
}

/// Checks if the current Microsoft account owns Minecraft
pub fn check_game_ownership(paths: &Paths) -> anyhow::Result<bool> {
	let db = AuthDatabase::open(&paths.auth).context("Failed to open auth database")?;

	Ok(db.has_logged_in_account())
}
