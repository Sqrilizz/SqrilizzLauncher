use crate::io::paths::Paths;
use basic::BasicPackageRepository;
use sqrilizz_net::modrinth;
use sqrilizz_pkg::PackageContentType;
use sqrilizz_pkg::repo::{PackageFlag, RepoMetadata, RepoPkgEntry};
use sqrilizz_pkg_gen::modrinth::convert_category;
use sqrilizz_pkg_gen::relation_substitution::RelationSubNone;

use anyhow::Context;
use sqrilizz_shared::output::{MessageContents, SqrilizzOutput};
use sqrilizz_shared::pkg::ArcPkgReq;
use sqrilizz_shared::translate;
use reqwest::Client;

use std::borrow::Cow;
use std::collections::{HashMap, HashSet};

use super::PkgLocation;
use super::core::{
	get_all_core_packages, get_core_package_content_type, get_core_package_count, is_core_package,
};

/// Basic index-based repositories
pub mod basic;

/// The ID of the built-in Modrinth repository
pub const MODRINTH_REPO_ID: &str = "modrinth";

/// The Modrinth logo as an SVG icon
const MODRINTH_ICON: &str = include_str!("modrinth_icon.svg");

/// The Modrinth categories that the repository supports
const MODRINTH_CATEGORIES: &[&str] = &[
	"adventure",
	"atmosphere",
	"audio",
	"blocks",
	"cartoon",
	"challenging",
	"combat",
	"decoration",
	"economy",
	"entities",
	"equipment",
	"fantasy",
	"fonts",
	"food",
	"game-mechanics",
	"gui",
	"items",
	"kitchen-sink",
	"library",
	"lightweight",
	"locale",
	"magic",
	"minigame",
	"mobs",
	"multiplayer",
	"optimization",
	"realistic",
	"simplistic",
	"social",
	"storage",
	"technology",
	"transportation",
	"tweaks",
	"utility",
	"vanilla-like",
	"worldgen",
];

/// A remote source for Sqrilizz Launcher packages
pub enum PackageRepository {
	/// A basic indexed repository
	Basic(BasicPackageRepository),
	/// The internal core repository
	Core,
	/// The built-in Modrinth repository
	Modrinth,
}

/// Gets the static metadata for the Modrinth repository
pub fn modrinth_metadata() -> RepoMetadata {
	RepoMetadata {
		name: Some("Modrinth".into()),
		description: Some("Mods, plugins, resource packs, and more from Modrinth".into()),
		color: Some("#1bd96a".into()),
		text_color: Some("black".into()),
		icon: Some(MODRINTH_ICON.to_string()),
		package_types: vec![
			sqrilizz_shared::pkg::PackageKind::Mod,
			sqrilizz_shared::pkg::PackageKind::Plugin,
			sqrilizz_shared::pkg::PackageKind::ResourcePack,
			sqrilizz_shared::pkg::PackageKind::Datapack,
			sqrilizz_shared::pkg::PackageKind::Shader,
			sqrilizz_shared::pkg::PackageKind::Modpack,
		],
		package_categories: MODRINTH_CATEGORIES
			.iter()
			.flat_map(|x| convert_category(x))
			.collect(),
		..Default::default()
	}
}

/// Gets the static metadata for the core repository
pub fn core_metadata() -> RepoMetadata {
	RepoMetadata {
		name: Some("Core".into()),
		description: Some("Internal packages for Sqrilizz Launcher".into()),
		color: Some("#8a2be2".into()),
		..Default::default()
	}
}

impl PackageRepository {
	/// Gets the ID of the repository
	pub fn get_id(&self) -> &str {
		match self {
			Self::Basic(repo) => &repo.id,
			Self::Core => "core",
			Self::Modrinth => MODRINTH_REPO_ID,
		}
	}

	/// Gets the displayed location of the repository
	pub fn get_displayed_location(&self) -> String {
		match self {
			PackageRepository::Basic(repo) => repo.get_location().to_string(),
			PackageRepository::Core => "Internal".into(),
			PackageRepository::Modrinth => modrinth::current_api_base(),
		}
	}

	/// Gets the static metadata for repos that don't need to fetch it
	pub fn static_metadata(&self) -> Option<RepoMetadata> {
		match self {
			Self::Core => Some(core_metadata()),
			Self::Modrinth => Some(modrinth_metadata()),
			Self::Basic(..) => None,
		}
	}

	/// Create the core repository
	pub fn core() -> Self {
		Self::Core
	}

	/// Create the Modrinth repository
	pub fn modrinth() -> Self {
		Self::Modrinth
	}

	/// Get the default set of repositories
	pub fn default_repos() -> Vec<Self> {
		vec![Self::core(), Self::modrinth()]
	}

	/// Update cached packages
	pub async fn sync(
		&self,
		paths: &Paths,
		client: &Client,
		_o: &mut impl SqrilizzOutput,
	) -> anyhow::Result<()> {
		match self {
			Self::Basic(repo) => repo.sync(paths, client).await,
			Self::Core | Self::Modrinth => Ok(()),
		}
	}

	/// Ask if the index has a package and return the url and version for that package if it exists
	pub async fn query(
		&self,
		id: &str,
		paths: &Paths,
		client: &Client,
		o: &mut impl SqrilizzOutput,
	) -> anyhow::Result<Option<RepoQueryResult>> {
		match self {
			Self::Basic(repo) => repo.query(id, paths, client, o).await,
			Self::Core => {
				if is_core_package(id) {
					Ok(Some(RepoQueryResult {
						location: PkgLocation::Core,
						content_type: get_core_package_content_type(id)
							.expect("Core package exists and should have a content type"),
						flags: HashSet::new(),
					}))
				} else {
					Ok(None)
				}
			}
			Self::Modrinth => {
				let pkg = sqrilizz_pkg_gen::modrinth::generate_from_id_optional(
					id,
					RelationSubNone,
					&[],
					false,
					false,
					Some(MODRINTH_REPO_ID),
				)
				.await
				.with_context(|| format!("Failed to generate Modrinth package '{id}'"))?;
				let Some(pkg) = pkg else {
					return Ok(None);
				};

				let json = serde_json::to_string(&pkg)
					.context("Failed to serialize generated Modrinth package")?;

				Ok(Some(RepoQueryResult {
					location: PkgLocation::Inline(std::sync::Arc::from(json)),
					content_type: PackageContentType::Declarative,
					flags: HashSet::new(),
				}))
			}
		}
	}

	/// Preloads multiple packages from this repo
	pub async fn preload(
		&self,
		packages: Vec<ArcPkgReq>,
		paths: &Paths,
		o: &mut impl SqrilizzOutput,
	) -> anyhow::Result<()> {
		let _ = (packages, paths, o);
		Ok(())
	}

	/// Get all packages from this repo. Returns an empty array for custom repos.
	pub async fn get_all_packages(
		&self,
		paths: &Paths,
		client: &Client,
		o: &mut impl SqrilizzOutput,
	) -> anyhow::Result<Vec<(ArcPkgReq, RepoPkgEntry)>> {
		match self {
			Self::Basic(repo) => repo.get_all_packages(paths, client, o).await,
			Self::Core => Ok(get_all_core_packages()),
			// Modrinth is too large to enumerate
			Self::Modrinth => Ok(Vec::new()),
		}
	}

	/// Get the number of packages in the repo
	pub async fn get_package_count(
		&self,
		paths: &Paths,
		client: &Client,
		o: &mut impl SqrilizzOutput,
	) -> anyhow::Result<Option<usize>> {
		match self {
			Self::Basic(repo) => repo.get_package_count(paths, client, o).await.map(Some),
			Self::Core => Ok(Some(get_core_package_count())),
			Self::Modrinth => Ok(None),
		}
	}

	/// Get the repo's metadata
	pub async fn get_metadata(
		&'_ self,
		paths: &Paths,
		client: &Client,
		o: &mut impl SqrilizzOutput,
	) -> anyhow::Result<Cow<'_, RepoMetadata>> {
		match self {
			Self::Basic(repo) => repo.get_metadata(paths, client, o).await.map(Cow::Borrowed),
			Self::Core => Ok(Cow::Owned(RepoMetadata {
				name: Some(translate!(o, CoreRepoName)),
				description: Some(translate!(o, CoreRepoDescription)),
				..Default::default()
			})),
			Self::Modrinth => Ok(Cow::Owned(modrinth_metadata())),
		}
	}
}

/// Query a list of repos
pub async fn query_all(
	repos: &[PackageRepository],
	pkg: &ArcPkgReq,
	paths: &Paths,
	client: &Client,
	o: &mut impl SqrilizzOutput,
) -> anyhow::Result<Option<RepoQueryResult>> {
	for repo in repos {

		if let Some(requested_repo) = &pkg.repository
			&& repo.get_id() != requested_repo
		{
			continue;
		}

		let query = match repo.query(&pkg.id, paths, client, o).await {
			Ok(val) => val,
			Err(e) => {
				o.display(MessageContents::Error(format!(
					"Failed to get package from repository {}: {e:?}",
					repo.get_id()
				)));
				continue;
			}
		};
		if query.is_some() {
			return Ok(query);
		}
	}
	Ok(None)
}

/// Get all packages from a list of repositories with the normal priority order
pub async fn get_all_packages(
	repos: &[PackageRepository],
	paths: &Paths,
	client: &Client,
	o: &mut impl SqrilizzOutput,
) -> anyhow::Result<HashMap<ArcPkgReq, RepoPkgEntry>> {
	let mut out = HashMap::new();
	// Iterate in reverse to make sure that repos at the beginning take precendence
	for repo in repos.iter().rev() {
		let packages = repo
			.get_all_packages(paths, client, o)
			.await
			.with_context(|| {
				format!(
					"Failed to get all packages from repository '{}'",
					repo.get_id()
				)
			})?;
		out.extend(packages);
	}

	Ok(out)
}

/// Result from repository querying. This represents an entry
/// for a package that can be accessed
pub struct RepoQueryResult {
	/// The location to copy the package from
	pub location: PkgLocation,
	/// The content type of the package
	pub content_type: PackageContentType,
	/// The flags for the package
	pub flags: HashSet<PackageFlag>,
}

/// Get the content type of a package from the repository
pub async fn get_content_type(entry: &RepoPkgEntry) -> PackageContentType {
	if let Some(content_type) = &entry.content_type {
		*content_type
	} else {
		PackageContentType::Script
	}
}
