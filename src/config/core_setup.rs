use anyhow::Context;
use sqrilizz_core::{
	SqrilizzCore,
	auth_crate::mc::ClientId,
	config::BrandingProperties,
};
use sqrilizz_shared::output::SqrilizzOutput;
use reqwest::Client;

use crate::io::paths::Paths;

/// Sets up and configures a SqrilizzCore according to Sqrilizz Launcher's features
pub async fn setup_core(
	client_id: Option<&ClientId>,
	client: &Client,
	_paths: &Paths,
	_o: &mut impl SqrilizzOutput,
) -> anyhow::Result<SqrilizzCore> {
	let mut core_config = sqrilizz_core::ConfigBuilder::new().branding(BrandingProperties::new(
		"Sqrilizz Launcher".into(),
		crate::VERSION.into(),
	));
	if let Some(client_id) = client_id {
		core_config = core_config.ms_client_id(client_id.clone());
	}
	let core_config = core_config.build();
	let mut core = SqrilizzCore::with_config(core_config).context("Failed to initialize core")?;

	core.set_client(client.clone());

	Ok(core)
}
