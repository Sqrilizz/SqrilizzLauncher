use std::path::{Path, PathBuf};

use anyhow::{Context, bail};
use crate::io::paths::Paths;
use sqrilizz_core::io::{files::create_leading_dirs, java::maven::MavenLibraryParts, json_from_file, json_to_file};
use sqrilizz_mods::{fabric_quilt, forge, paper, sponge};
use sqrilizz_net::{download::Client, neoforge};
use sqrilizz_shared::io::update_link;
use sqrilizz_shared::loaders::Loader;
use sqrilizz_shared::output::{MessageContents, SqrilizzOutput, OutputProcess};
use sqrilizz_shared::versions::{VersionInfo, VersionPattern};
use std::ops::DerefMut;
use sqrilizz_shared::{Side, UpdateDepth};

/// Argument given to the native loader installer
#[derive(Debug)]
pub struct LoaderSetupArg {
	/// The side of the instance
	pub side: Option<Side>,
	/// Path to the instance's directory
	pub inst_dir: Option<String>,
	/// Version info for the instance
	pub version_info: VersionInfo,
	/// The current version of the instance before being updated
	pub old_version: Option<String>,
	/// The loader of the instance
	pub loader: Loader,
	/// The current version of the loader as stored in the lockfile
	pub current_loader_version: Option<String>,
	/// The desired version of the loader
	pub desired_loader_version: VersionPattern,
	/// Path to the Sqrilizz Launcher internal dir
	pub internal_dir: String,
	/// The depth to update at
	pub update_depth: UpdateDepth,
	/// Path to the JVM
	pub jvm_path: String,
	/// Path to the vanilla game JAR
	pub game_jar_path: String,
}

/// The result from native loader installation
#[derive(Debug, Default)]
pub struct LoaderSetupResult {
	/// Optional override for the main class
	pub main_class_override: Option<String>,
	/// Optional override for the path to the game JAR file
	pub jar_path_override: Option<String>,
	/// Optional extension to the classpath, as a list of paths
	pub classpath_extension: Vec<String>,
	/// Optional new version for the loader
	pub loader_version: Option<String>,
	/// Optional additional JVM args
	pub jvm_args: Vec<String>,
	/// Optional additional game args
	pub game_args: Vec<String>,
	/// Whether to skip adding the game JAR to the final classpath
	pub exclude_game_jar: bool,
	/// Whether to exclude all of the generated / default arguments
	pub exclude_default_args: bool,
}

/// Install the necessary loader files if needed, replicating the plugin behavior natively
pub async fn install_loader(
	arg: &LoaderSetupArg,
	paths: &Paths,
	o: &mut impl SqrilizzOutput,
) -> anyhow::Result<Option<LoaderSetupResult>> {
	let Some(side) = arg.side else {
		bail!("Instance side is empty");
	};

	let Some(inst_dir) = &arg.inst_dir else {
		return Ok(None);
	};

	let internal_dir = PathBuf::from(&arg.internal_dir);
	let client = Client::new();

	match &arg.loader {
		Loader::Fabric | Loader::Quilt => {
			let mode = if arg.loader == Loader::Fabric {
				fabric_quilt::Mode::Fabric
			} else {
				fabric_quilt::Mode::Quilt
			};

			let desired_fq_version = if let VersionPattern::Single(pat) = &arg.desired_loader_version
			{
				Some(pat.clone())
			} else {
				None
			};

			let meta = fabric_quilt::get_meta(
				&arg.version_info.version,
				desired_fq_version.as_deref(),
				&mode,
				&internal_dir,
				arg.update_depth,
				&client,
			)
			.await
			.context("Failed to get metadata")?;

			let libraries_dir = internal_dir.join("libraries");

			fabric_quilt::download_files(
				&meta,
				&libraries_dir,
				mode,
				arg.update_depth,
				&client,
				&mut *o,
			)
			.await
			.context("Failed to download common files")?;

			fabric_quilt::download_side_specific_files(
				&meta,
				&libraries_dir,
				side,
				arg.update_depth,
				&client,
			)
			.await
			.context("Failed to download side-specific files")?;

			let classpath =
				fabric_quilt::get_classpath(&meta, &libraries_dir, side)
					.context("Failed to get classpath")?;

			let main_class = meta
				.launcher_meta
				.main_class
				.get_main_class_string(side)
				.to_string();

			let new_version = meta.loader.get_maven().map(|x| x.version);

			// Cleanup files when the version changes
			if arg.old_version != new_version {
				cleanup_fabric_files(Path::new(inst_dir));
			}

			Ok(Some(LoaderSetupResult {
				main_class_override: Some(main_class),
				classpath_extension: classpath.get_entries().to_vec(),
				jvm_args: vec!["-Dsodium.checks.issue2561=false".to_string()],
				loader_version: new_version,
				..Default::default()
			}))
		}
		Loader::NeoForged => {
			let mode = forge::Mode::NeoForge;
			let loader_version;

			let _ = create_leading_dirs(&internal_dir.join("forge"));

			let versions_path = internal_dir.join("forge/neoforge_versions.json");
			let versions = if arg.update_depth < UpdateDepth::Full && versions_path.exists() {
				json_from_file(&versions_path)?
			} else {
				let versions = {
					let mut process = OutputProcess::new(o);
					process.display(MessageContents::StartProcess(format!(
						"Fetching {mode} versions"
					)));

				let versions = neoforge::get_versions(&client).await?;
				json_to_file(versions_path, &versions)?;

					process.display(MessageContents::Success(format!(
						"{mode} versions fetched"
					)));

					versions
				};

				versions
			};

			let version = neoforge::get_latest_neoforge_version(
				&versions,
				&arg.version_info.version,
			)
			.context("Could not find NeoForge version for this Minecraft version")?;

			loader_version = Some(version.clone());

			let result = forge::install(
				&client,
				&internal_dir,
				arg.update_depth,
				&arg.version_info,
				side,
				mode,
				version,
				&PathBuf::from(&arg.jvm_path),
				o,
			)
			.await
			.context("Failed to install NeoForge")?;

			Ok(Some(LoaderSetupResult {
				classpath_extension: result.classpath.get_entries().to_vec(),
				main_class_override: Some(result.main_class),
				jvm_args: result.jvm_args,
				game_args: result.game_args,
				loader_version,
				exclude_game_jar: result.exclude_game_jar,
				exclude_default_args: result.exclude_default_args,
				..Default::default()
			}))
		}
		Loader::Paper | Loader::Folia => install_paper(arg, side, paths, o).await,
		Loader::Purpur => install_purpur(arg, o).await,
		Loader::Sponge => install_sponge(arg, side, paths).await,
		_ => Ok(None),
	}
}

/// Get available loader versions for a loader and Minecraft version
pub async fn get_loader_versions(
	loader: &Loader,
	minecraft_version: &str,
	update_depth: UpdateDepth,
	data_dir: &Path,
) -> anyhow::Result<Vec<String>> {
	let internal_dir = data_dir.join("internal");
	let client = Client::new();

	match loader {
		Loader::Fabric | Loader::Quilt => {
			let mode = if *loader == Loader::Fabric {
				fabric_quilt::Mode::Fabric
			} else {
				fabric_quilt::Mode::Quilt
			};

			let meta = fabric_quilt::get_all_meta(
				minecraft_version,
				&mode,
				&internal_dir,
				update_depth,
				&client,
			)
			.await
			.context("Failed to get metadata")?;

			let out = meta.into_iter().map(|version| {
				MavenLibraryParts::parse_from_str(&version.loader.maven)
					.map(|x| x.version)
					.unwrap_or(version.loader.maven)
			});

			Ok(out.collect())
		}
		Loader::Sponge => {
			let mode = sponge::Mode::Vanilla;

			let artifacts = sponge::get_artifacts(mode, minecraft_version, &client)
				.await
				.context("Failed to get list of Sponge versions")?;

			Ok(artifacts)
		}
		_ => Ok(Vec::new()),
	}
}

/// Install Paper or Folia for a server instance
async fn install_paper(
	arg: &LoaderSetupArg,
	side: Side,
	paths: &Paths,
	o: &mut impl SqrilizzOutput,
) -> anyhow::Result<Option<LoaderSetupResult>> {
	let inst_dir = arg.inst_dir.as_ref().expect("Instance dir should exist");

	if side != Side::Server || (arg.loader != Loader::Paper && arg.loader != Loader::Folia) {
		return Ok(None);
	}

	let mode = if arg.loader == Loader::Paper {
		paper::Mode::Paper
	} else {
		paper::Mode::Folia
	};

	let mut process = OutputProcess::new(o);
	process.display(MessageContents::StartProcess(format!(
		"Checking for {mode} updates"
	)));

	let client = sqrilizz_net::download::Client::new();

	// Check if this Minecraft version is available
	let stored_versions_path = get_stored_versions_path(paths, mode);
	let versions = if stored_versions_path.exists() && arg.update_depth == UpdateDepth::Shallow {
		json_from_file(&stored_versions_path).context("Failed to read versions from file")?
	} else {
		process.display(MessageContents::StartProcess(
			"Downloading version list".to_string(),
		));
		paper::get_all_versions(mode, &client)
			.await
			.context("Failed to get list of versions")?
	};
	let _ = create_leading_dirs(&stored_versions_path);
	json_to_file(stored_versions_path, &versions).context("Failed to write versions to file")?;

	let Some(version) = versions
		.iter()
		.find(|v| &v.version.id == &arg.version_info.version)
	else {
		bail!("Could not find a Paper version for the given Minecraft version");
	};

	let build_nums_strings: Vec<_> = version.builds.iter().map(|x| x.to_string()).collect();
	let desired_version = arg
		.desired_loader_version
		.get_match(&build_nums_strings)
		.with_context(|| format!("Failed to find the given {mode} version"))?;
	let desired_build_num: u16 = desired_version
		.parse()
		.context("The desired version must be a an unsigned integer")?;

	let current_build_num: Option<u16> =
		arg.current_loader_version.as_ref().and_then(|x| x.parse().ok());

	// If the new and current build nums mismatch, then get info for the current build num and
	// use it to teardown
	if let Some(current_build_num) = current_build_num
		&& desired_build_num != current_build_num
	{
		process.display(MessageContents::StartProcess(
			"Removing old build".to_string(),
		));
		let build_info = get_paper_build_info(
			paths,
			mode,
			&arg.version_info.version,
			current_build_num,
			arg.update_depth,
			&client,
			process.deref_mut(),
		)
		.await
		.context("Failed to get old version build info")?;

		remove_paper(Path::new(inst_dir), build_info.downloads.application.name)
			.with_context(|| format!("Failed to remove {mode} from the instance"))?;
	}

	// Get the name of the remote JAR file we need to download
	let build_info = get_paper_build_info(
		paths,
		mode,
		&arg.version_info.version,
		desired_build_num,
		arg.update_depth,
		&client,
		process.deref_mut(),
	)
	.await
	.context("Failed to get build info")?;

	// Download the JAR
	let jar_path = paper::get_local_jar_path(mode, &arg.version_info.version, &paths.core);
	if !jar_path.exists() || arg.update_depth == UpdateDepth::Force {
		process.display(MessageContents::StartProcess(
			"Downloading JAR file".to_string(),
		));
		paper::download_server_jar(
			mode,
			&arg.version_info.version,
			desired_build_num,
			&build_info.downloads.application.url,
			&paths.core,
			&client,
		)
		.await
		.with_context(|| format!("Failed to download JAR file for {mode}"))?;
	}

	// Link the Mojang jar to skip redownload
	let mojang_jar_path = get_cached_mojang_jar(Path::new(inst_dir), &arg.version_info.version);
	let _ = create_leading_dirs(&mojang_jar_path);
	let _ = update_link(Path::new(&arg.game_jar_path), &mojang_jar_path);

	process.display(MessageContents::Success(format!("{mode} updated")));

	let main_class = paper::PAPER_SERVER_MAIN_CLASS;

	Ok(Some(LoaderSetupResult {
		main_class_override: Some(main_class.into()),
		jar_path_override: Some(jar_path.to_string_lossy().to_string()),
		loader_version: Some(desired_build_num.to_string()),
		..Default::default()
	}))
}

/// Install Purpur for a server instance
async fn install_purpur(
	arg: &LoaderSetupArg,
	o: &mut impl SqrilizzOutput,
) -> anyhow::Result<Option<LoaderSetupResult>> {
	let inst_dir = arg.inst_dir.as_ref().expect("Instance dir should exist");
	let Some(side) = arg.side else {
		return Ok(None);
	};

	if side != Side::Server || arg.loader != Loader::Purpur {
		return Ok(None);
	}

	let mut process = OutputProcess::new(o);
	process.display(MessageContents::StartProcess("Installing Purpur".into()));

	let internal_dir = PathBuf::from(&arg.internal_dir);
	let builds_path = internal_dir
		.join("purpur")
		.join(format!("{}_builds.json", arg.version_info.version));

	let client = reqwest::Client::new();
	let builds = if builds_path.exists() && arg.update_depth == UpdateDepth::Shallow {
		json_from_file(&builds_path).context("Failed to read Purpur builds from file")?
	} else {
		let builds = get_purpur_builds(&arg.version_info.version, &client)
			.await
			.context("Failed to get Purpur builds")?;

		let _ = create_leading_dirs(&builds_path);
		json_to_file(builds_path, &builds).context("Failed to store Purpur builds")?;

		builds
	};

	let jar_path = internal_dir
		.join("jars")
		.join(format!("{}_server_purpur.jar", arg.version_info.version));

	let build = arg
		.desired_loader_version
		.get_match(&builds)
		.context("No matching Purpur build versions found")?;
	if !jar_path.exists() || arg.update_depth > UpdateDepth::Shallow {
		download_purpur_jar(
			&arg.version_info.version,
			&build,
			&jar_path,
			&client,
		)
		.await
		.context("Failed to download Purpur jar")?;
	}

	// Link the Mojang jar to skip redownload
	let mojang_jar_path = get_cached_mojang_jar(Path::new(inst_dir), &arg.version_info.version);
	let _ = create_leading_dirs(&mojang_jar_path);
	let _ = update_link(Path::new(&arg.game_jar_path), &mojang_jar_path);

	process.display(MessageContents::Success("Purpur Installed".into()));

	Ok(Some(LoaderSetupResult {
		main_class_override: Some("org.bukkit.craftbukkit.Main".into()),
		jar_path_override: Some(jar_path.to_string_lossy().to_string()),
		loader_version: Some(build),
		..Default::default()
	}))
}

/// Install Sponge for a server instance
async fn install_sponge(
	arg: &LoaderSetupArg,
	side: Side,
	paths: &Paths,
) -> anyhow::Result<Option<LoaderSetupResult>> {
	if side != Side::Server || arg.loader != Loader::Sponge {
		return Ok(None);
	}

	let mode = sponge::Mode::Vanilla;

	let client = Client::new();

	let artifacts = sponge::get_artifacts(mode, &arg.version_info.version, &client)
		.await
		.context("Failed to get list of Sponge versions")?;

	let artifact = arg
		.desired_loader_version
		.get_match(&artifacts)
		.context("Sponge version does not exist")?;

	let artifact_info = sponge::get_artifact_info(mode, &artifact, &client)
		.await
		.context("Failed to get artifact info")?;

	sponge::download_server_jar(
		sponge::Mode::Vanilla,
		&arg.version_info.version,
		&artifact_info,
		&paths.core,
		&client,
	)
	.await
	.context("Failed to download Sponge server JAR")?;

	let jar_path = sponge::get_local_jar_path(sponge::Mode::Vanilla, &arg.version_info.version, &paths.core);
	let main_class = sponge::SPONGE_SERVER_MAIN_CLASS;

	Ok(Some(LoaderSetupResult {
		main_class_override: Some(main_class.into()),
		jar_path_override: Some(jar_path.to_string_lossy().to_string()),
		..Default::default()
	}))
}

/// Get Purpur builds for the given Minecraft version
async fn get_purpur_builds(
	minecraft_version: &str,
	client: &reqwest::Client,
) -> anyhow::Result<Vec<String>> {
	#[derive(serde::Deserialize)]
	struct PurpurBuildsResponse {
		builds: PurpurBuilds,
	}

	#[derive(serde::Deserialize)]
	struct PurpurBuilds {
		// Ordered from oldest to newest
		all: Vec<String>,
	}

	let response = client
		.get(format!(
			"https://api.purpurmc.org/v2/purpur/{minecraft_version}"
		))
		.send()
		.await
		.context("Failed to fetch Purpur builds")?;

	if response.status() == reqwest::StatusCode::NOT_FOUND {
		bail!("No Purpur builds found for version {minecraft_version}");
	} else if !response.status().is_success() {
		bail!("Failed to fetch Purpur builds: HTTP {}", response.status());
	}

	let body = response
		.json::<PurpurBuildsResponse>()
		.await
		.context("Failed to parse Purpur builds JSON")?;

	Ok(body.builds.all)
}

async fn download_purpur_jar(
	minecraft_version: &str,
	build: &str,
	path: &Path,
	client: &reqwest::Client,
) -> anyhow::Result<()> {
	let url = format!(
		"https://api.purpurmc.org/v2/purpur/{minecraft_version}/{build}/download"
	);
	let response = client
		.get(url)
		.send()
		.await
		.context("Failed to download Purpur jar")?
		.error_for_status()
		.context("Invalid response from Purpur servers")?;
	let bytes = response
		.bytes()
		.await
		.context("Failed to download Purpur jar")?;
	if let Some(parent) = path.parent() {
		let _ = create_leading_dirs(parent);
	}
	std::fs::write(path, bytes.as_ref()).context("Failed to write Purpur jar")
}

// Remove Fabric files when the version changes to prevent loading errors
fn cleanup_fabric_files(game_dir: &Path) {
	let cache_path = game_dir.join(".fabric");
	if cache_path.exists() {
		let _ = std::fs::remove_dir_all(&cache_path);
	}
}

fn get_stored_versions_path(paths: &Paths, mode: paper::Mode) -> PathBuf {
	paths
		.internal
		.join(format!("paper/{}/versions_new.json", mode.to_str()))
}

fn get_stored_build_info_path(paths: &Paths, mode: paper::Mode, version: &str, build: u16) -> PathBuf {
	paths
		.internal
		.join(format!("paper/{}/{version}_{build}.json", mode.to_str()))
}

async fn get_paper_build_info(
	paths: &Paths,
	mode: paper::Mode,
	version: &str,
	build: u16,
	update_depth: UpdateDepth,
	client: &sqrilizz_net::download::Client,
	o: &mut impl SqrilizzOutput,
) -> anyhow::Result<paper::BuildInfoResponse> {
	let build_info_path = get_stored_build_info_path(paths, mode, version, build);
	let build_info = if build_info_path.exists() && update_depth <= UpdateDepth::Full {
		json_from_file(&build_info_path).context("Failed to read build info from file")?
	} else {
		o.display(MessageContents::StartProcess(
			"Downloading build info".to_string(),
		));
		paper::get_build_info(mode, version, build, client)
			.await
			.with_context(|| format!("Failed to get build info for new {mode} version"))?
	};
	let _ = create_leading_dirs(&build_info_path);
	json_to_file(build_info_path, &build_info).context("Failed to write build info to file")?;

	Ok(build_info)
}

fn remove_paper(game_dir: &Path, paper_file_name: String) -> anyhow::Result<()> {
	let paper_path = game_dir.join(paper_file_name);
	if paper_path.exists() {
		std::fs::remove_file(paper_path).context("Failed to remove Paper jar")?;
	}

	Ok(())
}

fn get_cached_mojang_jar(inst_dir: &Path, version: &str) -> PathBuf {
	inst_dir.join("cache").join(format!("mojang_{version}.jar"))
}
