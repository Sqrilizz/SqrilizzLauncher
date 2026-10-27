use sqrilizz_config::account::{AccountConfig, AccountVariant};
use sqrilizz_core::account::{Account, AccountKind};

/// Creates an account from an account config
pub fn read_account_config(config: &AccountConfig, id: &str) -> Account {
	match config {
		AccountConfig::Simple(variant) | AccountConfig::Advanced { variant } => {
			let kind = match variant {
				AccountVariant::Microsoft => AccountKind::Microsoft { xbox_uid: None },
				AccountVariant::Demo => AccountKind::Demo,
				AccountVariant::Unknown(id) => AccountKind::Unknown(id.clone()),
			};
			Account::new(kind, id.into())
		}
	}
}
