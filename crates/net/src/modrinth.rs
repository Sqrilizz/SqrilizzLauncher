use crate::download::user_agent;
use anyhow::{Context, anyhow};
use sqrilizz_shared::{
	loaders::Loader,
	pkg::{PackageCategory, PackageKind, PackageSearchParameters},
};
use reqwest::{Client, StatusCode};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{LazyLock, RwLock};

/// The official Modrinth API base URL
pub const DEFAULT_API_BASE: &str = "https://api.modrinth.com";

static API_BASES: LazyLock<RwLock<Vec<String>>> = LazyLock::new(|| {
	RwLock::new(vec![DEFAULT_API_BASE.to_string()])
});
static CURRENT_BASE: AtomicUsize = AtomicUsize::new(0);

/// Set the ordered list of Modrinth API base URLs (main API + mirrors).
/// The first entry is the preferred base. Falls back to the official API if empty.
pub fn set_api_bases(bases: Vec<String>) {
	let mut bases: Vec<String> = bases
		.into_iter()
		.map(|x| x.trim_end_matches('/').to_string())
		.filter(|x| !x.is_empty())
		.collect();
	bases.dedup();
	if bases.is_empty() {
		bases.push(DEFAULT_API_BASE.to_string());
	}
	*API_BASES.write().unwrap() = bases;
	CURRENT_BASE.store(0, Ordering::Relaxed);
}

/// Get the ordered list of Modrinth API base URLs
pub fn api_bases() -> Vec<String> {
	API_BASES.read().unwrap().clone()
}

/// Get the currently preferred Modrinth API base URL
pub fn current_api_base() -> String {
	let bases = api_bases();
	let idx = CURRENT_BASE.load(Ordering::Relaxed) % bases.len();
	bases[idx].clone()
}

/// Fetch a path on the Modrinth API (relative to /v2/), trying each configured
/// base URL (the current preferred base first, then mirrors) until one succeeds.
pub async fn fetch_api(
	path: &str,
	client: &Client,
) -> anyhow::Result<(String, reqwest::Response)> {
	let bases = api_bases();
	let start = CURRENT_BASE.load(Ordering::Relaxed) % bases.len();
	let mut last_err = None;
	let mut not_found = None;

	for i in 0..bases.len() {
		let idx = (start + i) % bases.len();
		let url = format!("{}/v2/{}", bases[idx], path);

		let resp = client
			.get(&url)
			.header("User-Agent", user_agent())
			.send()
			.await;

		match resp {
			Ok(resp) if resp.status().is_success() => {
				CURRENT_BASE.store(idx, Ordering::Relaxed);
				return Ok((url, resp));
			}
			// Remember 404s in case no mirror has the resource, but try the others first
			// in case a mirror has incomplete data
			Ok(resp) if resp.status() == StatusCode::NOT_FOUND => {
				not_found = Some((url, resp));
			}
			Ok(resp) => {
				last_err = Some(anyhow!(
					"Server reported an error (HTTP {}) for {url}",
					resp.status()
				));
			}
			Err(e) => {
				last_err = Some(anyhow!("Failed to send request to {url}: {e}"));
			}
		}
	}

	if let Some(not_found) = not_found {
		return Ok(not_found);
	}

	Err(last_err.unwrap_or_else(|| anyhow!("No Modrinth API bases configured")))
		.context("Failed to fetch from the Modrinth API")
}

/// Fetch and deserialize JSON from a path on the Modrinth API, trying each configured base
pub async fn fetch_api_json<T: DeserializeOwned>(
	path: &str,
	client: &Client,
) -> anyhow::Result<T> {
	let (url, resp) = fetch_api(path, client).await?;
	let resp = resp
		.error_for_status()
		.with_context(|| format!("Server reported an error for {url}"))?;
	resp.json()
		.await
		.context("Failed to deserialize JSON")
}

/// Format a path on the Modrinth API with the current base URL
pub fn api_url(path: &str) -> String {
	let bases = api_bases();
	let idx = CURRENT_BASE.load(Ordering::Relaxed) % bases.len();
	format!("{}/v2/{}", bases[idx], path)
}

/// A Modrinth project (mod, resource pack, etc.)
#[derive(Deserialize, Serialize, Clone, Default)]
#[serde(default)]
pub struct Project {
	/// The ID of the project
	#[serde(alias = "project_id")]
	pub id: String,
	/// The slug of the project
	pub slug: String,
	/// The type of this project and its files
	pub project_type: ProjectType,
	/// The ID's of the available project versions
	pub versions: Vec<String>,
	/// The Minecraft versions this project is available for
	pub game_versions: Vec<String>,
	/// The loaders this project is available for
	pub loaders: Vec<ModrinthLoader>,
	/// The project's support on the client side
	pub client_side: SideSupport,
	/// The project's support on the server side
	pub server_side: SideSupport,
	/// The project's team ID
	pub team: String,
	/// The display name of the project
	pub title: String,
	/// The short description of the project
	pub description: String,
	/// The long description of the project
	pub body: Option<String>,
	/// URL to the icon
	pub icon_url: Option<String>,
	/// URL to the issue tracker
	pub issues_url: Option<String>,
	/// URL to the source
	pub source_url: Option<String>,
	/// URL to the wiki
	pub wiki_url: Option<String>,
	/// URL to the Discord
	pub discord_url: Option<String>,
	/// Donation URLs
	pub donation_urls: Vec<DonationLink>,
	/// The license of the project
	pub license: License,
	/// The gallery items of the project
	pub gallery: Option<Vec<GalleryEntry>>,
	/// Categories for the project
	pub categories: Vec<String>,
	/// Number of downloads for the project
	pub downloads: u32,
}

/// The type of a Modrinth project
#[derive(Deserialize, Serialize, Copy, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ProjectType {
	/// A mod project
	#[default]
	Mod,
	/// A modpack project
	Modpack,
	/// A resource pack project
	ResourcePack,
	/// A shader project
	Shader,
	/// A datapack project
	Datapack,
	/// A plugin project
	Plugin,
}

/// Get a project from the API
pub async fn get_project(project_id: &str, client: &Client) -> anyhow::Result<Project> {
	fetch_api_json(&format!("project/{project_id}"), client)
		.await
		.context("Failed to download Modrinth project")
}

/// Get a project from the API, returning none if it does not exist
pub async fn get_project_optional(
	project_id: &str,
	client: &Client,
) -> anyhow::Result<Option<Project>> {
	let (url, resp) = fetch_api(&format!("project/{project_id}"), client).await?;
	if resp.status() == StatusCode::NOT_FOUND {
		return Ok(None);
	}

	let resp = resp
		.error_for_status()
		.with_context(|| format!("Server reported an error for {url}"))?;

	resp.json()
		.await
		.map(Some)
		.context("Failed to deserialize JSON")
}

/// Get the raw response of a project from the API
pub async fn get_project_raw(project_id: &str, client: &Client) -> anyhow::Result<String> {
	let (_, resp) = fetch_api(&format!("project/{project_id}"), client).await?;
	let resp = resp
		.error_for_status()
		.context("Server reported an error")?;
	resp.text()
		.await
		.context("Failed to download Modrinth project")
}

/// Get multiple Modrinth projects
pub async fn get_multiple_projects(
	projects: &[String],
	client: &Client,
) -> anyhow::Result<Vec<Project>> {
	if projects.is_empty() {
		return Ok(Vec::new());
	}
	// Use the multiple-projects API endpoint as it's faster
	let param = serde_json::to_string(projects)
		.context("Failed to convert project list to API parameter")?;
	fetch_api_json(&format!("projects?ids={param}"), client).await
}

/// Release channel for a Modrinth project version
#[derive(Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum ReleaseChannel {
	/// A finished release version
	Release,
	/// An unfinished beta version
	Beta,
	/// An unfinished alpha version
	Alpha,
}

/// A Modrinth project version
#[derive(Deserialize, Serialize, Clone)]
pub struct Version {
	/// The ID of this version
	pub id: String,
	/// The ID of the project this version is from
	pub project_id: String,
	/// The name of this version
	pub name: String,
	/// The version number of this version
	pub version_number: String,
	/// The type / release channel of this version
	pub version_type: ReleaseChannel,
	/// The loaders that this version supports
	pub loaders: Vec<ModrinthLoader>,
	/// The list of downloads for this version
	pub files: Vec<Download>,
	/// The game versions this version supports
	pub game_versions: Vec<String>,
	/// The dependencies that this version has
	pub dependencies: Vec<Dependency>,
	/// Whether this version is featured
	pub featured: bool,
	/// The date this version was published in ISO-8601
	pub date_published: String,
}

/// Loader for a Modrinth project version
#[derive(Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(untagged)]
pub enum ModrinthLoader {
	/// A loader that is known
	Known(KnownLoader),
	/// A loader that we do not know about
	Unknown(String),
}

/// A known plugin / mod loader that Modrinth supports
#[derive(Deserialize, Serialize, Copy, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum KnownLoader {
	/// The Vanilla game
	Minecraft,
	/// MinecraftForge
	Forge,
	/// Fabric loader
	Fabric,
	/// Quilt loader
	Quilt,
	/// NeoForged loader
	#[serde(rename = "neoforge")]
	NeoForged,
	/// Rift loader
	Rift,
	/// Liteloader
	Liteloader,
	/// Risugami's Modloader
	#[serde(rename = "modloader")]
	Risugamis,
	/// Bukkit loaders
	Bukkit,
	/// Spigot server
	Spigot,
	/// Paper server
	Paper,
	/// Sponge server
	Sponge,
	/// Purpur server
	Purpur,
	/// Folia server
	Folia,
	/// Iris shader loader
	Iris,
	/// Optifine shader loader
	Optifine,
	/// Datapack loader
	Datapack,
	/// Velocity loader
	Velocity,
	/// BungeeCord loader
	#[serde(rename = "bungeecord")]
	BungeeCord,
	/// Waterfall loader
	Waterfall,
}

impl Version {
	/// Returns the primary file download for this version
	pub fn get_primary_download(&self) -> anyhow::Result<&Download> {
		let primary = self.files.iter().find(|x| x.primary);
		if let Some(primary) = primary {
			Ok(primary)
		} else {
			self.files
				.first()
				.ok_or(anyhow!("Version has no downloads"))
		}
	}
}

/// Gets all the versions for a Modrinth project
pub async fn get_project_versions(
	project_id: &str,
	client: &Client,
) -> anyhow::Result<Vec<Version>> {
	fetch_api_json(&format!("project/{project_id}/version"), client).await
}

/// Get a Modrinth project version
pub async fn get_version(version_id: &str, client: &Client) -> anyhow::Result<Version> {
	fetch_api_json(&format!("version/{version_id}"), client)
		.await
		.context("Failed to download Modrinth version")
}

/// Get the raw response of a version from the API
pub async fn get_version_raw(version_id: &str, client: &Client) -> anyhow::Result<String> {
	let (_, resp) = fetch_api(&format!("version/{version_id}"), client).await?;
	let resp = resp
		.error_for_status()
		.context("Server reported an error")?;
	resp.text()
		.await
		.context("Failed to download Modrinth version")
}

/// Get multiple Modrinth project versions
pub async fn get_multiple_versions(
	versions: &[String],
	client: &Client,
) -> anyhow::Result<Vec<Version>> {
	if versions.is_empty() {
		return Ok(Vec::new());
	}

	// Use the multiple-versions API endpoint as it's faster
	let param = serde_json::to_string(versions)
		.context("Failed to convert version list to API parameter")?;
	fetch_api_json(&format!("versions?ids={param}"), client).await
}

/// A file download from the Modrinth API
#[derive(Deserialize, Serialize, Clone)]
pub struct Download {
	/// The URL to the file download
	pub url: String,
	/// The name of the file
	pub filename: String,
	/// Whether or not this is the primary file for this version
	pub primary: bool,
	/// Hashes for this file
	pub hashes: Hashes,
}

/// Hashes for a Modrinth file
#[derive(Deserialize, Serialize, Clone)]
pub struct Hashes {
	/// SHA-512 hash
	pub sha512: String,
}

/// A version dependency
#[derive(Deserialize, Serialize, Clone)]
pub struct Dependency {
	/// The ID of the project
	pub project_id: Option<String>,
	/// The ID of the version
	pub version_id: Option<String>,
	/// The type of the dependency
	pub dependency_type: DependencyType,
}

/// The type of a dependency
#[derive(Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum DependencyType {
	/// A required dependency
	Required,
	/// An optional / recommended dependency
	Optional,
	/// An incompatible dependency
	Incompatible,
	/// An embedded dependency
	Embedded,
}

/// Information about a project license
#[derive(Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum License {
	Short(String),
	Long(LongLicense),
}

impl Default for License {
	fn default() -> Self {
		Self::Short("ARR".into())
	}
}

/// Long information about a project license
#[derive(Deserialize, Serialize, Clone)]
pub struct LongLicense {
	/// The short ID of the license
	pub id: String,
	/// The URL to a custom license
	pub url: Option<String>,
}

/// Information about a donation link
#[derive(Deserialize, Serialize, Clone)]
pub struct DonationLink {
	/// The URL of the link
	pub url: String,
}

/// An entry in a project's gallery
#[derive(Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum GalleryEntry {
	Simple(String),
	Full(FullGalleryEntry),
}

impl GalleryEntry {
	/// Get the URL to this entry
	pub fn get_url(&self) -> &str {
		match self {
			Self::Simple(url) => url,
			Self::Full(entry) => &entry.raw_url,
		}
	}
}

/// An entry in a project's gallery
#[derive(Deserialize, Serialize, Clone)]
pub struct FullGalleryEntry {
	/// The URL to the low-quality version of the gallery image
	pub url: String,
	/// The URL to the high-quality version of the gallery image
	pub raw_url: String,
	/// Whether the gallery image is a featured banner on the project page
	pub featured: bool,
}

/// Support status for a project on a specific side
#[derive(Deserialize, Serialize, Clone, Copy, Default)]
#[serde(rename_all = "snake_case")]
pub enum SideSupport {
	/// Required to be on this side
	Required,
	/// Can optionally be on this side
	Optional,
	/// Unsupported on this side
	Unsupported,
	/// Support unknown
	#[default]
	Unknown,
}

/// Get the team members of a project
pub async fn get_project_team(project_id: &str, client: &Client) -> anyhow::Result<Vec<Member>> {
	fetch_api_json(&format!("project/{project_id}/members"), client).await
}

/// Get multiple Modrinth teams
pub async fn get_multiple_teams(
	teams: &[String],
	client: &Client,
) -> anyhow::Result<Vec<Vec<Member>>> {
	if teams.is_empty() {
		return Ok(Vec::new());
	}
	// Use the multiple-teams API endpoint as it's faster
	let param =
		serde_json::to_string(teams).context("Failed to convert team list to API parameter")?;
	fetch_api_json(&format!("teams?ids={param}"), client).await
}

/// A member of a project team
#[derive(Deserialize, Serialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Member {
	/// The ordering of the team member
	pub ordering: i32,
	/// The user that represents this member
	pub user: User,
	/// The ID of the team this member is a part of
	pub team_id: String,
}

/// A user on the platform
#[derive(Deserialize, Serialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct User {
	/// The user's username
	pub username: String,
}

/// Search projects from the Modrinth API. Note that the projects returned by this have many default fields and should NOT be used as the final projects.
pub async fn search_projects(
	params: PackageSearchParameters,
	client: &Client,
) -> anyhow::Result<SearchResults> {
	let limit = if params.count > 100 {
		100
	} else {
		params.count
	};
	let search = if let Some(search) = params.search {
		format!("&query={}", search.replace(' ', "%20"))
	} else {
		String::new()
	};

	let mut facets = Vec::new();

	let types = params
		.types
		.into_iter()
		.map(|x| match x {
			PackageKind::Mod => "mod",
			PackageKind::ResourcePack => "resourcepack",
			PackageKind::Datapack => "datapack",
			PackageKind::Plugin => "plugin",
			PackageKind::Shader => "shader",
			PackageKind::Bundle | PackageKind::Modpack => "modpack",
		})
		.map(|x| format!("\"project_types={x}\""))
		.collect::<Vec<_>>()
		.join(",");
	if !types.is_empty() {
		facets.push(format!("[{types}]"));
	}

	if !params.minecraft_versions.is_empty() {
		let versions = params
			.minecraft_versions
			.into_iter()
			.map(|x| format!("\"versions={x}\""))
			.collect::<Vec<_>>()
			.join(",");
		facets.push(format!("[{versions}]"));
	};

	if !params.loaders.is_empty() {
		let loaders = params
			.loaders
			.into_iter()
			.filter_map(|x| match x {
				Loader::Fabric => Some(KnownLoader::Fabric),
				Loader::Forge => Some(KnownLoader::Forge),
				Loader::Folia => Some(KnownLoader::Folia),
				Loader::Quilt => Some(KnownLoader::Quilt),
				Loader::Rift => Some(KnownLoader::Rift),
				Loader::Risugamis => Some(KnownLoader::Risugamis),
				Loader::LiteLoader => Some(KnownLoader::Liteloader),
				Loader::Paper => Some(KnownLoader::Paper),
				Loader::Purpur => Some(KnownLoader::Purpur),
				Loader::CraftBukkit => Some(KnownLoader::Bukkit),
				Loader::Spigot => Some(KnownLoader::Spigot),
				Loader::Sponge => Some(KnownLoader::Sponge),
				Loader::NeoForged => Some(KnownLoader::NeoForged),
				_ => None,
			})
			.map(|x| {
				format!(
					"\"categories={}\"",
					serde_json::to_string(&x).unwrap().replace("\"", "")
				)
			})
			.collect::<Vec<_>>();
		if !loaders.is_empty() {
			facets.push(format!("[{}]", loaders.join(",")));
		}
	};

	if !params.categories.is_empty() {
		let categories = params
			.categories
			.into_iter()
			.flat_map(convert_category)
			.map(|x| {
				format!(
					"\"categories={}\"",
					serde_json::to_string(&x).unwrap().replace("\"", "")
				)
			})
			.collect::<Vec<_>>();
		if categories.is_empty() {
			return Ok(SearchResults::default());
		}

		let categories = categories.join(",");
		facets.push(format!("[{categories}]"));
	};

	let facets_inside = facets.join(",");
	let facets = if facets_inside.is_empty() {
		String::new()
	} else {
		format!("&facets=[{facets_inside}]")
	};
	let path = format!(
		"search?limit={limit}{search}{facets}&offset={}",
		params.skip
	);

	fetch_api_json(&path, client).await
}

#[derive(Deserialize, Serialize, Clone, Default)]
pub struct SearchResults {
	/// The results
	pub hits: Vec<SearchedProject>,
	/// The total number of results outside limits
	pub total_hits: usize,
}

/// A project result in the search
#[derive(Deserialize, Serialize, Clone, Default)]
#[serde(default)]
pub struct SearchedProject {
	/// The ID of the project
	#[serde(alias = "project_id")]
	pub id: String,
	/// The slug of the project
	pub slug: String,
	/// The type of this project and its files
	pub project_type: ProjectType,
	/// The display name of the project
	pub title: String,
	/// A short description of the project
	pub description: String,
	/// Displayed categories of the project on it's grid tile
	pub display_categories: Vec<String>,
	/// Minecraft versions this project supports
	pub versions: Vec<String>,
	/// Icon for this project
	pub icon_url: Option<String>,
	/// Gallery for this project
	pub gallery: Option<Vec<String>>,
	/// Featured gallery image for this project
	pub featured_gallery: Option<String>,
	/// Number of downloads for this project
	pub downloads: u32,
}

fn convert_category(category: PackageCategory) -> &'static [&'static str] {
	match category {
		PackageCategory::Blocks => &["blocks"],
		PackageCategory::Building => &["blocks", "decoration"],
		PackageCategory::Decoration => &["decoration"],
		PackageCategory::Exploration | PackageCategory::Worldgen => &["worldgen", "adventure"],
		PackageCategory::Adventure => &["adventure"],
		PackageCategory::Atmosphere => &["atmosphere"],
		PackageCategory::Audio => &["audio"],
		PackageCategory::Cartoon => &["cartoon"],
		PackageCategory::Challenge => &["challenging"],
		PackageCategory::Combat => &["combat"],
		PackageCategory::Economy => &["economy"],
		PackageCategory::Entities => &["entities"],
		PackageCategory::Equipment => &["equipment"],
		PackageCategory::Fantasy => &["fantasy"],
		PackageCategory::Fonts => &["fonts"],
		PackageCategory::Food => &["food"],
		PackageCategory::GameMechanics => &["game-mechanics"],
		PackageCategory::Gui => &["gui"],
		PackageCategory::Items => &["items"],
		PackageCategory::Extensive => &["kitchen-sink"],
		PackageCategory::Library => &["library"],
		PackageCategory::Lightweight => &["lightweight"],
		PackageCategory::Language => &["locale"],
		PackageCategory::Magic => &["magic"],
		PackageCategory::Minigame => &["minigame"],
		PackageCategory::Mobs => &["mobs"],
		PackageCategory::Multiplayer => &["multiplayer"],
		PackageCategory::Optimization => &["optimization"],
		PackageCategory::Realistic => &["realistic"],
		PackageCategory::Simplistic => &["simplistic"],
		PackageCategory::Social => &["social"],
		PackageCategory::Storage => &["storage"],
		PackageCategory::Technology => &["technology"],
		PackageCategory::Transportation => &["transportation"],
		PackageCategory::Tweaks => &["tweaks"],
		PackageCategory::Utility => &["utility"],
		PackageCategory::VanillaPlus => &["vanilla-like"],
		_ => &[],
	}
}
