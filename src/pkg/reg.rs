use anyhow::{Context, anyhow};
use dashmap::DashMap;
use itertools::Itertools;
use sqrilizz_core::net::get_transfer_limit;
use sqrilizz_net::modrinth::{self, ProjectType};
use sqrilizz_pkg::PackageContentType;
use sqrilizz_pkg::PackageMetaAndProps;
use sqrilizz_pkg::PackageSearchResults;
use sqrilizz_pkg::PkgRequest;
use sqrilizz_pkg::metadata::PackageMetadata;
use sqrilizz_pkg::properties::PackageProperties;
use sqrilizz_pkg_gen::modrinth::convert_category;
use sqrilizz_shared::output::MessageContents;
use sqrilizz_shared::output::SqrilizzOutput;
use sqrilizz_shared::pkg::ArcPkgReq;
use sqrilizz_shared::pkg::PackageKind;
use sqrilizz_shared::pkg::PackageSearchParameters;
use sqrilizz_shared::versions::VersionPattern;
use reqwest::Client;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

use super::Package;
use super::repo::{MODRINTH_REPO_ID, query_all, PackageRepository};
use crate::io::paths::Paths;

use std::collections::HashMap;
use std::sync::Arc;

/// An object used to store and cache all of the packages that we are working with.
/// It queries repositories automatically when asking for a package that isn't in the
/// registry, and prevents having a bunch of copies of packages everywhere.
pub struct PkgRegistry {
	/// The package repositories that the user has configured
	pub repos: Vec<PackageRepository>,
	packages: DashMap<ArcPkgReq, Arc<Package>>,
}

impl PkgRegistry {
	/// Create a new PkgRegistry with repositories
	pub fn new(repos: Vec<PackageRepository>) -> Self {
		Self {
			repos,
			packages: DashMap::new(),
		}
	}

	/// Clear the registry
	pub fn clear(&self) {
		self.packages.clear();
	}

	/// Insert a package into the registry and return a mutable reference to the
	/// newly inserted package
	fn insert(&self, req: ArcPkgReq, pkg: Arc<Package>) -> Arc<Package> {
		self.packages.insert(req.clone(), pkg.clone());
		self.packages
			.get(&req)
			.expect("Package was not inserted into map")
			.clone()
	}

	/// Checks if a package is in the registry already
	pub fn has_now(&self, req: &PkgRequest) -> bool {
		self.packages.contains_key(req)
	}

	/// Query repositories to insert a package
	async fn query_insert(
		&self,
		req: &ArcPkgReq,
		paths: &Paths,
		client: &Client,
		o: &mut impl SqrilizzOutput,
	) -> anyhow::Result<Arc<Package>> {
		// First check the remote repositories
		let query = query_all(
			&self.repos,
			req,
			paths,
			client,
			o,
		)
		.await
		.context("Failed to query remote repositories")?;
		if let Some(result) = query {
			Ok(self.insert(
				req.clone(),
				Arc::new(Package::new(
					req.clone(),
					result.location,
					result.content_type,
					result.flags,
				)),
			))
		} else {
			Err(anyhow!("Package '{req}' does not exist"))
		}
	}

	/// Get a package from the registry, fetching it if it isn't present
	pub async fn get(
		&self,
		req: &ArcPkgReq,
		paths: &Paths,
		client: &Client,
		o: &mut impl SqrilizzOutput,
	) -> anyhow::Result<Arc<Package>> {
		if let Some(pkg) = self.packages.get(req) {
			Ok(pkg.clone())
		} else {
			self.query_insert(req, paths, client, o).await
		}
	}

	/// Ensure that a package is in the registry
	pub async fn ensure_package(
		&self,
		req: &ArcPkgReq,
		paths: &Paths,
		client: &Client,
		o: &mut impl SqrilizzOutput,
	) -> anyhow::Result<()> {
		self.get(req, paths, client, o)
			.await
			.with_context(|| format!("Failed to get package {req}"))?;

		Ok(())
	}

	/// Replaces the slug and version of a request if possible, to make it better for user display
	pub async fn make_req_displayable(
		&self,
		req: &ArcPkgReq,
		paths: &Paths,
		client: &Client,
		o: &mut impl SqrilizzOutput,
	) -> ArcPkgReq {
		let Ok(pkg) = self.get(req, paths, client, o).await else {
			return req.clone();
		};

		if pkg.content_type != PackageContentType::Declarative {
			return req.clone();
		}

		let pkg = match pkg.get_declarative_contents(paths, client).await {
			Ok(pkg) => pkg.expect("Should be a declarative package"),
			Err(e) => {
				o.display(MessageContents::Warning(format!("{e:?}")));
				return req.clone();
			}
		};

		let slug = pkg.meta.slug.clone();
		let version = match &req.content_version {
			VersionPattern::Any => VersionPattern::Any,
			VersionPattern::Single(x) => {
				VersionPattern::Single(pkg.get_content_version_name(x).clone())
			}
			VersionPattern::Before(x) => {
				VersionPattern::Before(pkg.get_content_version_name(x).clone())
			}
			VersionPattern::After(x) => {
				VersionPattern::After(pkg.get_content_version_name(x).clone())
			}
			VersionPattern::Range(x, y) => VersionPattern::Range(
				pkg.get_content_version_name(x).clone(),
				pkg.get_content_version_name(y).clone(),
			),
			VersionPattern::Prefer(x) => {
				VersionPattern::Prefer(pkg.get_content_version_name(x).clone())
			}
			VersionPattern::Latest(x) => VersionPattern::Latest(x.clone()),
		};

		Arc::new(PkgRequest {
			source: req.source.clone(),
			id: req.id.clone(),
			repository: req.repository.clone(),
			content_version: version,
			slug,
		})
	}

	/// Remove a cached package
	pub async fn remove_cached(
		&self,
		req: &ArcPkgReq,
		paths: &Paths,
		client: &Client,
		o: &mut impl SqrilizzOutput,
	) -> anyhow::Result<()> {
		let pkg = self
			.get(req, paths, client, o)
			.await
			.with_context(|| format!("Failed to get package {req}"))?;
		pkg.remove_cached(paths)?;
		Ok(())
	}

	/// Iterator over all package requests in the registry
	pub fn iter_requests(&self) -> impl Iterator<Item = ArcPkgReq> {
		let vec: Vec<_> = self.packages.iter().map(|x| x.key().clone()).collect();
		vec.into_iter()
	}

	/// Get all of the package requests in the registry in an owned manner
	pub fn get_all_packages(&self) -> Vec<ArcPkgReq> {
		self.packages.iter().map(|x| x.key().clone()).collect()
	}

	/// Get all of the available package requests from the repos
	pub async fn get_all_available_packages(
		&self,
		paths: &Paths,
		client: &Client,
		o: &mut impl SqrilizzOutput,
	) -> anyhow::Result<Vec<ArcPkgReq>> {
		let out = super::repo::get_all_packages(&self.repos, paths, client, o)
			.await
			.context("Failed to retrieve all packages from repos")?
			.into_iter()
			.map(|x| x.0)
			.collect();

		Ok(out)
	}

	/// Remove cached packages
	async fn remove_cached_packages(
		&self,
		packages: impl Iterator<Item = &ArcPkgReq>,
		paths: &Paths,
		client: &Client,
		o: &mut impl SqrilizzOutput,
	) -> anyhow::Result<()> {
		for package in packages {
			self.remove_cached(package, paths, client, o)
				.await
				.with_context(|| format!("Failed to remove cached package '{package}'"))?;
		}

		Ok(())
	}

	/// Update cached package scripts based on the caching strategy
	pub async fn update_cached_packages(
		&self,
		paths: &Paths,
		client: &Client,
		o: &mut impl SqrilizzOutput,
	) -> anyhow::Result<()> {
		let packages = self
			.get_all_available_packages(paths, client, o)
			.await
			.context("Failed to get list of available packages")?;

		self.remove_cached_packages(packages.iter(), paths, client, o)
			.await
			.context("Failed to remove all cached packages")?;

		// Redownload all the packages
		let mut tasks = JoinSet::new();
		let semaphore = Arc::new(Semaphore::new(get_transfer_limit()));
		for package in packages {
			let pkg = self
				.get(&package, paths, client, o)
				.await
				.with_context(|| format!("Failed to get package {package}"))?;

			if let Some(task) = pkg.get_download_task(paths, true, client) {
				let semaphore = semaphore.clone();
				let task = async move {
					let _ = semaphore.acquire_owned().await;
					task.await
				};
				tasks.spawn(task);
			}
		}

		while let Some(res) = tasks.join_next().await {
			res??;
		}

		Ok(())
	}

	/// Searches the registry and repositories for packages
	pub async fn search(
		&self,
		params: PackageSearchParameters,
		repo: &str,
		paths: &Paths,
		client: &Client,
		o: &mut impl SqrilizzOutput,
	) -> anyhow::Result<PackageSearchResults> {
		let original_count = params.count;
		if original_count == 0 {
			return Ok(PackageSearchResults::default());
		}

		if repo == MODRINTH_REPO_ID {
			return search_modrinth(params, client).await;
		}

		let mut total_results = 0;

		let mut out = Vec::with_capacity(params.count as usize);
		let mut previews = HashMap::with_capacity(params.count as usize);

		let searched_repo = self
			.repos
			.iter()
			.find(|x| x.get_id() == repo)
			.context("Repository does not exist")?;

		let all_packages = searched_repo
			.get_all_packages(paths, client, o)
			.await
			.with_context(|| format!("Failed to get packages from repository '{repo}'"))?;

		let mut num_skipped = 0;

		for (req, _) in all_packages.into_iter().sorted_by_key(|x| x.0.clone()) {
			if !params.categories.is_empty() || params.search.is_some() {
				let pkg = self.query_insert(&req, paths, client, o).await?;
				let meta = pkg.get_metadata(paths, client).await?;

				// Check all of the parameters
				if !params.categories.is_empty() {
					let default = Vec::new();
					if !params
						.categories
						.iter()
						.any(|x| meta.categories.as_ref().unwrap_or(&default).contains(x))
					{
						continue;
					}
				}

				if let Some(search) = &params.search {
					let default = String::new();
					if !req.id.to_lowercase().contains(search)
						&& !meta
							.name
							.as_ref()
							.unwrap_or(&default)
							.to_lowercase()
							.contains(search) && !meta
						.description
						.as_ref()
						.unwrap_or(&default)
						.to_lowercase()
						.contains(search)
					{
						continue;
					}
				}
			}

			total_results += 1;
			if num_skipped < params.skip {
				num_skipped += 1;
				continue;
			}

			if out.len() < original_count as usize {
				out.push(req.to_string());
				if let Ok(pkg) = self.get(&req, paths, client, o).await {
					let meta = pkg.get_metadata(paths, client).await.unwrap_or_default();
					let props = pkg.get_properties(paths, client).await.unwrap_or_default();
					previews.insert(req.to_string(), PackageMetaAndProps { meta, props });
				}
			}
		}

		Ok(PackageSearchResults {
			results: out,
			total_results,
			previews,
		})
	}

	/// Preloads packages, if it can, from the repos in this registry
	pub async fn preload_packages(
		&self,
		packages: impl Iterator<Item = &ArcPkgReq>,
		paths: &Paths,
		client: &Client,
		o: &mut impl SqrilizzOutput,
	) -> anyhow::Result<()> {
		// Remove any packages that are already stored
		let mut packages: Vec<_> = packages.filter(|x| !self.has_now(x)).collect();

		// Query insert and remove from the list any packages that are matched by basic repositories
		let mut to_remove = Vec::new();
		for (i, package) in packages.iter().enumerate() {
			if self
				.query_insert(package, paths, client, o)
				.await
				.is_ok()
			{
				to_remove.push(i);
			}
		}

		let mut i = 0;
		packages.retain(|_| {
			let out = !to_remove.contains(&i);
			i += 1;
			out
		});

		let _ = packages;

		Ok(())
	}

	/// Gets the repositories stored in this registry in their correct order
	pub fn get_repos(&self) -> &[PackageRepository] {
		&self.repos
	}
}

/// Searches the Modrinth repository through its API
async fn search_modrinth(
	params: PackageSearchParameters,
	client: &Client,
) -> anyhow::Result<PackageSearchResults> {
	let results = modrinth::search_projects(params, client)
		.await
		.context("Failed to search Modrinth")?;

	let mut out = Vec::with_capacity(results.hits.len());
	let mut previews = HashMap::with_capacity(results.hits.len());

	for hit in results.hits {
		let req = format!("{}:{}", MODRINTH_REPO_ID, hit.slug);

		let kind = match hit.project_type {
			ProjectType::Mod => PackageKind::Mod,
			ProjectType::Modpack => PackageKind::Modpack,
			ProjectType::ResourcePack => PackageKind::ResourcePack,
			ProjectType::Shader => PackageKind::Shader,
			ProjectType::Datapack => PackageKind::Datapack,
			ProjectType::Plugin => PackageKind::Plugin,
		};

		let meta = PackageMetadata {
			slug: Some(hit.slug.clone()),
			name: Some(hit.title.clone()),
			description: Some(hit.description.clone()),
			downloads: Some(hit.downloads),
			icon: hit.icon_url.clone(),
			categories: Some(
				hit.display_categories
					.iter()
					.flat_map(|x| convert_category(x))
					.collect(),
			),
			..Default::default()
		};

		let props = PackageProperties {
			kinds: vec![kind],
			modrinth_id: Some(hit.id.clone()),
			supported_versions: Some(
				hit.versions
					.iter()
					.map(|x| VersionPattern::Single(x.clone()))
					.collect(),
			),
			..Default::default()
		};

		previews.insert(
			req.clone(),
			PackageMetaAndProps {
				meta: meta.into(),
				props: props.into(),
			},
		);
		out.push(req);
	}

	Ok(PackageSearchResults {
		results: out,
		total_results: results.total_hits,
		previews,
	})
}
