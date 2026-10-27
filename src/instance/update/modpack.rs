#![allow(missing_docs)]

use std::{
	ops::DerefMut,
	path::{Path, PathBuf},
	sync::Arc,
};

use anyhow::{Context, bail};
use sqrilizz_config::instance::{InstanceConfig, make_valid_instance_id};
use sqrilizz_core::io::files::create_leading_dirs_async;
use sqrilizz_instance::addon::modpack::mrpack::ModrinthPack;
use sqrilizz_instance::lock::{InstanceLockfile, LockfileAddon, LockfileModpack};
use sqrilizz_net::download;
use sqrilizz_shared::{
	Side, UpdateDepth,
	lang::Language,
	loaders::Loader,
	minecraft::AddonKind,
	output::{MessageContents, SqrilizzOutput},
	pkg::{ArcPkgReq, PackageStability, merge_package_lists},
	versions::{MinecraftVersionDeser, VersionInfo},
};
use reqwest::Client;

use crate::{
	addon::AddonExt,
	instance::{Instance, update::InstanceUpdateContext},
	io::paths::Paths,
	pkg::{
		eval::{EvalConstants, EvalInput, EvalParameters, Routine},
		reg::PkgRegistry,
	},
};

impl Instance {
	pub async fn update_modpack<'a, O: SqrilizzOutput>(
		&mut self,
		modpack: &ArcPkgReq,
		depth: UpdateDepth,
		version_info: &VersionInfo,
		ctx: &mut InstanceUpdateContext<'a, O>,
	) -> anyhow::Result<ModpackInstallResult> {
		let mut inst_lock = self.get_lockfile(ctx.paths)?;

		if self.dir.is_none() {
			return Ok(ModpackInstallResult::default());
		}

		let lock_modpack = inst_lock.get_modpack();
		if depth <= UpdateDepth::Shallow
			&& let Some(modpack) = lock_modpack
		{
			return Ok(ModpackInstallResult {
				supplied_packages: modpack.packages.clone(),
			});
		}

		let mut section = ctx.output.get_section();
		section.display(MessageContents::Header("Updating modpack".into()));

		let mut process = section.get_process();
		process.display(MessageContents::StartProcess("Downloading modpack".into()));

		let constants = EvalConstants {
			version: Some(version_info.version.clone()),
			loader: self.loader.clone(),
			version_list: version_info.versions.clone(),
			language: ctx.prefs.language,
			default_stability: self.config.package_stability.unwrap_or_default(),
			suppress: Vec::new(),
		};
		let params = EvalParameters::new(self.side());

		let input = EvalInput {
			constants: Arc::new(constants),
			params,
		};

		let download_result = download_modpack_package(
			modpack,
			input,
			&self.id,
			ctx.packages,
			ctx.client,
			ctx.paths,
			process.deref_mut(),
		)
		.await
		.context("Failed to download modpack")?;

		let Some(download_result) = download_result else {
			return Ok(ModpackInstallResult::default());
		};

		process.display(MessageContents::Success("Modpack downloaded".into()));
		process.finish();

		let modpack_path_str = download_result.modpack_path.to_string_lossy().to_string();

		let old_path = if let Some(lock_modpack) = lock_modpack {
			if lock_modpack.path == modpack_path_str {
				None
			} else {
				if !Path::new(&lock_modpack.path).exists() {
					section.display(MessageContents::Warning(
						"Old modpack not available. Update may not work properly".into(),
					));
				}

				Some(lock_modpack.path.clone())
			}
		} else {
			None
		};

		if download_result.format == "mrpack" {
			let file = std::fs::File::open(&download_result.modpack_path)
				.context("Failed to open downloaded mrpack file")?;
			let mut pack =
				ModrinthPack::from_stream(file).context("Failed to read modpack index")?;

			pack.download(&ctx.paths.addons, ctx.client)
				.await
				.context("Failed to download modpack files")?;

			let mut old_pack = old_path
				.as_ref()
				.and_then(|p| std::fs::File::open(p).ok())
				.and_then(|f| ModrinthPack::from_stream(f).ok());

			let inst_dir = self.dir().unwrap();
			pack.apply(inst_dir, &ctx.paths.addons, self.side(), old_pack.as_mut())
				.context("Failed to apply modpack")?;

			let addons_list = pack.get_addons(inst_dir, &ctx.paths.addons)?;
			let addons: Vec<_> = addons_list
				.into_iter()
				.map(|x| LockfileAddon {
					id: None,
					package: None,
					from_modpack: true,
					file_name: x.file_name,
					files: x
						.target_paths
						.into_iter()
						.map(|p| p.to_string_lossy().to_string())
						.collect(),
					kind: x.kind,
					hashes: x.hashes,
				})
				.collect();

			let packages = merge_package_lists(
				download_result.bundled.into_iter().map(|x| x.to_string()),
				&[],
			);
			let packages = merge_package_lists(
				download_result.included.into_iter().map(|x| x.to_string()),
				&packages,
			);

			let lockfile_modpack = LockfileModpack {
				name: pack.index().name.clone(),
				path: modpack_path_str,
				packages,
			};

			let files_to_remove = inst_lock.update_modpack(lockfile_modpack, &addons);
			for file in files_to_remove {
				let file = PathBuf::from(file);
				if file.exists() {
					let _ = std::fs::remove_file(file);
				}
			}
			inst_lock.write().context("Failed to write lockfile")?;

			return Ok(ModpackInstallResult::default());
		}

		bail!(
			"Modpack format {} is not supported",
			download_result.format
		);
	}

	pub async fn create_from_modpack_package(
		id: &str,
		modpack: &ArcPkgReq,
		side: Side,
		version_list: Vec<String>,
		reg: &PkgRegistry,
		client: &Client,
		paths: &Paths,
		o: &mut impl SqrilizzOutput,
	) -> anyhow::Result<InstanceConfig> {
		let mut process = o.get_process();
		process.display(MessageContents::StartProcess("Downloading modpack".into()));

		let constants = EvalConstants {
			version: None,
			loader: Loader::Any,
			version_list: version_list.clone(),
			language: Language::default(),
			default_stability: PackageStability::Latest,
			suppress: Vec::new(),
		};
		let params = EvalParameters::new(side);

		let input = EvalInput {
			constants: Arc::new(constants),
			params,
		};

		let download_result = download_modpack_package(
			modpack,
			input,
			id,
			reg,
			client,
			paths,
			process.deref_mut(),
		)
		.await?;

		let Some(download_result) = download_result else {
			bail!("Modpack package did not return an addon");
		};

		process.display(MessageContents::Success("Modpack downloaded".into()));
		process.finish();

		if download_result.format == "mrpack" {
			let target_dir = paths.data.join("instances").join(id);
			std::fs::create_dir_all(&target_dir)
				.context("Failed to create directory for new instance")?;

			let file = std::fs::File::open(&download_result.modpack_path)
				.context("Failed to open downloaded mrpack file")?;
			let mut pack =
				ModrinthPack::from_stream(file).context("Failed to read modpack index")?;

			let mut dl_process = o.get_process();
			dl_process.display(MessageContents::StartProcess(
				"Downloading modpack files".into(),
			));
			pack.download(&paths.addons, client)
				.await
				.context("Failed to download modpack files")?;
			dl_process.display(MessageContents::Success("Modpack files downloaded".into()));
			dl_process.finish();

			pack.apply(&target_dir, &paths.addons, side, None)
				.context("Failed to extract modpack files")?;

			let loader = if let Some(ver) = &pack.index().dependencies.fabric_loader {
				Some(format!("fabric:{ver}"))
			} else if let Some(ver) = &pack.index().dependencies.neoforge {
				Some(format!("neoforge:{ver}"))
			} else if let Some(ver) = &pack.index().dependencies.forge {
				Some(format!("forge:{ver}"))
			} else if let Some(ver) = &pack.index().dependencies.quilt_loader {
				Some(format!("quilt:{ver}"))
			} else {
				None
			};

			let mc_version = pack.index().dependencies.minecraft.clone();
			let mut config = InstanceConfig {
				name: Some(pack.index().name.clone()),
				side: Some(side),
				version: Some(MinecraftVersionDeser::Version(mc_version.into())),
				loader,
				imported: true,
				..Default::default()
			};
			config.modpack = Some(modpack.to_string());

			if let Some(icon) = download_result.icon {
				let pkg_identifier =
					make_valid_instance_id(&modpack.to_string_no_version_or_slug());
				let path = paths.internal.join("icons").join(&pkg_identifier);
				if !path.exists() {
					let _ = create_leading_dirs_async(&path).await;
					if let Err(e) = download::file(&icon, &path, client).await {
						o.display(MessageContents::Error(format!(
							"Failed to download modpack icon: {e:?}"
						)));
					}
				}
				config.icon = Some(path.to_string_lossy().to_string());
			}

			let mut inst_lock = InstanceLockfile::open(&InstanceLockfile::get_path(
				Some(&target_dir),
				id,
				&paths.internal,
			))?;

			let addons_list = pack.get_addons(&target_dir, &paths.addons)?;
			let addons: Vec<_> = addons_list
				.into_iter()
				.map(|x| LockfileAddon {
					id: None,
					package: None,
					from_modpack: true,
					file_name: x.file_name,
					files: x
						.target_paths
						.into_iter()
						.map(|p| p.to_string_lossy().to_string())
						.collect(),
					kind: x.kind,
					hashes: x.hashes,
				})
				.collect();

			let modpack_path_str = download_result.modpack_path.to_string_lossy().to_string();
			let packages = merge_package_lists(
				download_result.bundled.into_iter().map(|x| x.to_string()),
				&[],
			);
			let packages = merge_package_lists(
				download_result.included.into_iter().map(|x| x.to_string()),
				&packages,
			);

			let lockfile_modpack = LockfileModpack {
				name: pack.index().name.clone(),
				path: modpack_path_str,
				packages,
			};

			let files_to_remove = inst_lock.update_modpack(lockfile_modpack, &addons);
			for file in files_to_remove {
				let file = PathBuf::from(file);
				if file.exists() {
					let _ = std::fs::remove_file(file);
				}
			}
			inst_lock.write().context("Failed to write lockfile")?;

			return Ok(config);
		}

		bail!(
			"Modpack format {} is not supported",
			download_result.format
		);
	}
}

#[derive(Default, Clone)]
pub struct ModpackInstallResult {
	pub supplied_packages: Vec<String>,
}

pub async fn download_modpack_package(
	modpack: &ArcPkgReq,
	input: EvalInput,
	instance_id: &str,
	reg: &PkgRegistry,
	client: &Client,
	paths: &Paths,
	o: &mut impl SqrilizzOutput,
) -> anyhow::Result<Option<ModpackDownloadResult>> {
	let package = reg
		.get(modpack, paths, client, o)
		.await
		.context("Failed to get modpack package")?;

	let result = package
		.eval(paths, Routine::Install, input, client)
		.await
		.context("Failed to evaluate modpack")?;

	let Some(addon) = result.addon_reqs.first() else {
		o.debug(MessageContents::Simple(
			"No modpack addon was returned".into(),
		));
		o.display(MessageContents::Success("Modpack installed".into()));
		return Ok(None);
	};

	if addon.addon.kind != AddonKind::Modpack {
		bail!("Modpack package did not have a modpack addon");
	}

	let Some(format) = &addon.addon.modpack_format else {
		bail!("Modpack addon did not specify a format");
	};

	let modpack_path = addon.addon.get_path(paths, instance_id);
	if !modpack_path.exists() {
		addon
			.acquire(paths, instance_id, client)
			.await
			.context("Failed to download modpack")?;
	}

	let meta = package
		.get_metadata(paths, client)
		.await
		.context("Failed to get modpack metadata")?;

	Ok(Some(ModpackDownloadResult {
		modpack_path,
		format: format.clone(),
		icon: meta.icon.clone(),
		bundled: result.bundled,
		included: result.inclusions,
	}))
}

#[derive(Default, Clone)]
pub struct ModpackDownloadResult {
	pub modpack_path: PathBuf,
	pub format: String,
	pub icon: Option<String>,
	pub bundled: Vec<Arc<str>>,
	pub included: Vec<Arc<str>>,
}
