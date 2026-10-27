use std::time::Duration;

use freya::query::{Captured, Query};
use sqrilizz::{
	instance::update::loader_setup::get_loader_versions,
	shared::{UpdateDepth, loaders::Loader},
};

use crate::{ops::query_spawn, prelude::*, simple_query};

/// Loaders that can be selected natively
pub fn supported_loaders() -> Vec<Loader> {
	vec![
		Loader::Vanilla,
		Loader::Fabric,
		Loader::Quilt,
		Loader::Forge,
		Loader::NeoForged,
		Loader::Paper,
		Loader::Folia,
		Loader::Purpur,
		Loader::Sponge,
		Loader::SpongeForge,
		Loader::CraftBukkit,
		Loader::Spigot,
	]
}

simple_query!(
	name = FetchSupportedLoaders,
	ok = Vec<Loader>,
	err = anyhow::Error,
	keys = (),
	fn run(&self, _: &Self::Keys) -> impl Future<Output = Result<Self::Ok, Self::Err>> {
		let back_state = self.back_state.clone();

		query_spawn(back_state.0.clone(), async move { Ok(supported_loaders()) })
	}
);

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct FetchLoaderVersionsKey {
	loader: Option<Loader>,
	minecraft_version: Option<String>,
}

simple_query!(
	name = FetchLoaderVersions,
	ok = Vec<String>,
	err = anyhow::Error,
	keys = FetchLoaderVersionsKey,
	fn run(&self, keys: &Self::Keys) -> impl Future<Output = Result<Self::Ok, Self::Err>> {
		let back_state = self.back_state.clone();
		let keys = keys.clone();

		query_spawn(back_state.0.clone(), async move {
			let (Some(loader), Some(version)) = (keys.loader, keys.minecraft_version) else {
				return Ok(Vec::new());
			};

			get_loader_versions(
				&loader,
				&version,
				UpdateDepth::Shallow,
				&back_state.paths.data,
			)
			.await
		})
	}
);#[derive(Clone, PartialEq)]
pub struct JavaType {
	pub id: String,
	pub name: String,
}

simple_query!(
	name = FetchJavaTypes,
	ok = Vec<JavaType>,
	err = anyhow::Error,
	keys = (),
	fn run(&self, _: &Self::Keys) -> impl Future<Output = Result<Self::Ok, Self::Err>> {
		let back_state = self.back_state.clone();

		query_spawn(back_state.0.clone(), async move { Ok(Vec::new()) })
	}
);

#[derive(Clone, PartialEq)]
pub struct AccountType {
	pub id: String,
	pub name: String,
}

simple_query!(
	name = FetchAccountTypes,
	ok = Vec<AccountType>,
	err = anyhow::Error,
	keys = (),
	fn run(&self, _: &Self::Keys) -> impl Future<Output = Result<Self::Ok, Self::Err>> {
		let back_state = self.back_state.clone();

		query_spawn(back_state.0.clone(), async move { Ok(Vec::new()) })
	}
);

impl FetchLoaderVersions {
	pub fn with_loader(
		back_state: BackState,
		loader: Option<Loader>,
		minecraft_version: Option<String>,
	) -> Query<Self> {
		Query::new(
			FetchLoaderVersionsKey {
				loader,
				minecraft_version,
			},
			Self {
				back_state: Captured(back_state),
			},
		)
		.stale_time(Duration::from_secs(300))
	}
}
