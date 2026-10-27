use std::collections::HashSet;
use std::path::PathBuf;

use crate::{
	io::paths::Paths,
	pkg::repo::{
		PackageRepository,
		basic::{BasicPackageRepository, RepoLocation},
	},
};
use sqrilizz_config::preferences::{PrefDeser, RepoDeser};
use sqrilizz_core::net::download::validate_url;
use sqrilizz_net::modrinth;

use anyhow::{Context, bail};
use sqrilizz_shared::{
	lang::Language,
	output::{MessageContents, SqrilizzOutput},
};

/// Configured user preferences
#[derive(Debug, Default, Clone)]
pub struct ConfigPreferences {
	/// The global language
	pub language: Language,
}

impl ConfigPreferences {
	/// Convert deserialized preferences to the stored format and returns
	/// a list of repositories to add.
	pub async fn read(
		prefs: &PrefDeser,
		_paths: &Paths,
		o: &mut impl SqrilizzOutput,
	) -> (Self, Vec<PackageRepository>) {
		let mut repositories = Vec::new();

		for repo in prefs.repositories.preferred.iter() {
			if !repo.disable
				&& let Err(e) = add_repo(&mut repositories, repo)
			{
				o.display(MessageContents::Error(format!(
					"Failed to add repository {}: {e:?}",
					repo.id
				)));
			}
		}
		repositories.extend(PackageRepository::default_repos());
		for repo in prefs.repositories.backup.iter() {
			if !repo.disable
				&& let Err(e) = add_repo(&mut repositories, repo)
			{
				o.display(MessageContents::Error(format!(
					"Failed to add repository {}: {e:?}",
					repo.id
				)));
			}
		}

		// Check for duplicate IDs
		let mut existing = HashSet::new();
		for repo in &repositories {
			if existing.contains(&repo.get_id()) {
				o.display(MessageContents::Error(format!(
					"Duplicate repository ID '{}'",
					repo.get_id()
				)));
			}
			existing.insert(repo.get_id());
		}

		// Configure the Modrinth API bases: the user's mirrors first (in order),
		// then the official API as a fallback
		let mut api_bases = prefs.modrinth_mirrors.clone();
		api_bases.push(modrinth::DEFAULT_API_BASE.to_string());
		modrinth::set_api_bases(api_bases);

		(
			Self {
				language: prefs.language,
			},
			repositories,
		)
	}
}

/// Add a repo to the list
fn add_repo(repos: &mut Vec<PackageRepository>, repo: &RepoDeser) -> anyhow::Result<()> {
	let location = if let Some(url) = &repo.url {
		validate_url(url).with_context(|| {
			format!("Invalid url '{}' in package repository '{}'", url, repo.id)
		})?;
		RepoLocation::Remote(url.clone())
	} else if let Some(path) = &repo.path {
		RepoLocation::Local(PathBuf::from(path))
	} else {
		bail!("Niether path nor URL was set for repository {}", repo.id);
	};

	repos.push(PackageRepository::Basic(BasicPackageRepository::new(
		&repo.id, location,
	)));

	Ok(())
}
