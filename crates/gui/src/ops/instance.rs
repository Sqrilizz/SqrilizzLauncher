use std::{
	path::{Component, Path, PathBuf},
	sync::Arc,
	time::Duration,
};

use anyhow::{Context, bail, ensure};
use itertools::Itertools;
use sqrilizz::{
	config::modifications::{ConfigModification, apply_modifications_and_write},
	config_crate::{ConfigKind, instance::InstanceConfig, template::TemplateConfig},
	core::util::versions::MinecraftVersion,
	instance::update::{InstanceUpdateContext, UpdateFacets},
	instance_crate::files::{InstanceSave, ServerInfo},
	io::lock::Lockfile,
	shared::{
		Side, UpdateDepth,
		id::{InstanceID, TemplateID},
		loaders::Loader,
		versions::MinecraftVersionDeser,
		output::NoOp,
	},
};

use crate::{
	dependency::BackDependency,
	ops::{MakeSend, QueryOption, task::Task},
	pages::config::ConfiguredItem,
	prelude::*,
	secrets::get_ms_client_id,
	simple_mutation, simple_query,
	state::BackEvent,
};

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct FetchItems {
	back_state: Captured<BackState>,
}

impl FetchItems {
	pub fn new(back_state: BackState) -> Query<Self> {
		Query::new(
			(),
			Self {
				back_state: Captured(back_state),
			},
		)
		.stale_time(Duration::from_secs(30))
	}
}

impl QueryCapability for FetchItems {
	type Ok = InstancesAndTemplates;
	type Err = anyhow::Error;
	type Keys = ();

	fn run(&self, _: &Self::Keys) -> impl Future<Output = Result<Self::Ok, Self::Err>> {
		let back_state = self.back_state.clone();

		query_spawn(back_state.0.clone(), async move {
			let config = back_state.config_with_warnings().await?;

			let instances = config
				.instances
				.values()
				.sorted_by_cached_key(|x| x.id())
				.map(|x| InstanceItemInfo {
					id: x.id().to_string(),
					ty: ConfigKind::Instance,
					name: x.config().name.clone(),
					icon: x.config().icon.clone(),
					side: Some(x.side()),
					version: Some(x.version().clone()),
					loader: Some(x.loader().clone()),
					dir: x.config().dir.clone(),
					is_editable: true,
					is_deletable: true,
				});

			let templates = config
				.consolidated_templates
				.iter()
				.sorted_by_cached_key(|x| x.0.clone())
				.map(|(id, x)| InstanceItemInfo {
					id: id.to_string(),
					ty: ConfigKind::Template,
					name: x.instance.name.clone(),
					icon: x.instance.icon.clone(),
					side: x.instance.side,
					version: x
						.instance
						.version
						.as_ref()
						.map(MinecraftVersion::from_deser),
					loader: x.client_loader().or(x.server_loader()).map(|x| x.0),
					dir: None,
					is_editable: true,
					is_deletable: true,
				});

			let base_template = InstanceItemInfo {
				id: "base".into(),
				ty: ConfigKind::BaseTemplate,
				name: Some("Base Template".into()),
				icon: None,
				side: None,
				version: None,
				loader: None,
				dir: None,
				is_editable: true,
				is_deletable: false,
			};

			Ok(InstancesAndTemplates {
				instances: instances.collect(),
				templates: std::iter::once(base_template).chain(templates).collect(),
			})
		})
	}
}

/// Simple info about an instance or template
#[derive(Clone, PartialEq)]
pub struct InstanceItemInfo {
	pub id: String,
	pub ty: ConfigKind,
	pub name: Option<String>,
	pub icon: Option<String>,
	pub side: Option<Side>,
	pub version: Option<MinecraftVersion>,
	pub loader: Option<Loader>,
	pub dir: Option<String>,
	pub is_editable: bool,
	pub is_deletable: bool,
}

impl InstanceItemInfo {
	pub fn get_config_item(&self) -> ConfiguredItem {
		let id = if self.ty == ConfigKind::BaseTemplate {
			None
		} else {
			Some(self.id.clone())
		};

		ConfiguredItem {
			id,
			ty: self.ty,
			is_new: false,
		}
	}

	pub fn name(&self) -> &str {
		self.name.as_deref().unwrap_or(self.id.as_str())
	}
}

#[derive(Clone, Default)]
pub struct InstancesAndTemplates {
	pub instances: Vec<InstanceItemInfo>,
	pub templates: Vec<InstanceItemInfo>,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct FetchInstanceConfig {
	back_state: Captured<BackState>,
}

impl FetchInstanceConfig {
	pub fn capability(back_state: BackState) -> Self {
		Self {
			back_state: Captured(back_state),
		}
	}

	pub fn new(id: String, back_state: BackState) -> Query<Self> {
		Query::new(
			id,
			Self {
				back_state: Captured(back_state),
			},
		)
	}
}

impl QueryCapability for FetchInstanceConfig {
	type Ok = Option<InstanceConfigs>;
	type Err = anyhow::Error;
	type Keys = String;

	fn run(&self, id: &Self::Keys) -> impl Future<Output = Result<Self::Ok, Self::Err>> {
		let back_state = self.back_state.clone();
		let id = id.clone();

		query_spawn(back_state.0.clone(), async move {
			let config = back_state.config().await?;

			let Some(instance) = config.instances.get(&InstanceID::from(id)) else {
				return Ok(None);
			};

			Ok(Some(InstanceConfigs {
				main: instance.config().clone(),
				no_templates: instance.original_config().clone(),
			}))
		})
	}
}

#[derive(Clone, Default)]
pub struct InstanceConfigs {
	pub main: InstanceConfig,
	pub no_templates: InstanceConfig,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct FetchInstanceOrTemplateConfig {
	back_state: Captured<BackState>,
}

impl FetchInstanceOrTemplateConfig {
	pub fn new(item: ConfiguredItem, back_state: BackState) -> Query<Self> {
		Query::new(
			item,
			Self {
				back_state: Captured(back_state),
			},
		)
	}
}

impl QueryCapability for FetchInstanceOrTemplateConfig {
	type Ok = Option<InstanceOrTemplateConfigs>;
	type Err = anyhow::Error;
	type Keys = ConfiguredItem;

	fn run(&self, item: &Self::Keys) -> impl Future<Output = Result<Self::Ok, Self::Err>> {
		let back_state = self.back_state.clone();
		let item = item.clone();

		query_spawn(back_state.0.clone(), async move {
			let config = back_state.config().await?;

			match item.ty {
				ConfigKind::Instance => {
					let Some(id) = item.id else {
						return Ok(None);
					};
					let Some(instance) = config.instances.get(&InstanceID::from(id)) else {
						return Ok(None);
					};

					Ok(Some(InstanceOrTemplateConfigs {
						main: TemplateConfig::from_instance(instance.config().clone()),
						no_templates: TemplateConfig::from_instance(
							instance.original_config().clone(),
						),
					}))
				}
				ConfigKind::Template => {
					let Some(id) = item.id else {
						return Ok(None);
					};
					let id = TemplateID::from(id);
					let Some(template) = config.templates.get(&id) else {
						return Ok(None);
					};
					let Some(consolidated_template) = config.consolidated_templates.get(&id) else {
						return Ok(None);
					};

					Ok(Some(InstanceOrTemplateConfigs {
						main: consolidated_template.clone(),
						no_templates: template.clone(),
					}))
				}
				ConfigKind::BaseTemplate => Ok(Some(InstanceOrTemplateConfigs {
					main: config.base_template.clone(),
					no_templates: config.base_template.clone(),
				})),
			}
		})
	}
}

#[allow(dead_code)]
#[derive(Default, Clone)]
pub struct InstanceOrTemplateConfigs {
	pub main: TemplateConfig,
	pub no_templates: TemplateConfig,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct FetchParentConfigs {
	back_state: Captured<BackState>,
}

impl FetchParentConfigs {
	pub fn new(from: Vec<String>, back_state: BackState) -> Query<Self> {
		Query::new(
			from,
			Self {
				back_state: Captured(back_state),
			},
		)
	}
}

impl QueryCapability for FetchParentConfigs {
	type Ok = Arc<[TemplateConfig]>;
	type Err = anyhow::Error;
	type Keys = Vec<String>;

	fn run(&self, keys: &Self::Keys) -> impl Future<Output = Result<Self::Ok, Self::Err>> {
		let back_state = self.back_state.clone();
		let from = keys.clone();

		query_spawn(back_state.0.clone(), async move {
			let config = back_state.config().await?;

			let out = from
				.iter()
				.filter_map(|x| {
					config
						.consolidated_templates
						.get(&TemplateID::from(x.clone()))
						.cloned()
				})
				.collect();

			Ok(out)
		})
	}
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct SaveConfig {
	back_state: Captured<BackState>,
}

#[derive(Clone, PartialEq, Hash)]
pub struct SaveConfigParams {
	pub item: ConfiguredItem,
	pub config: NotEq<TemplateConfig>,
}

impl SaveConfig {
	pub fn new(back_state: BackState) -> Self {
		Self {
			back_state: Captured(back_state),
		}
	}
}

impl MutationCapability for SaveConfig {
	type Ok = ();
	type Err = anyhow::Error;
	type Keys = SaveConfigParams;

	fn run(&self, keys: &Self::Keys) -> impl Future<Output = Result<Self::Ok, Self::Err>> {
		let keys = keys.clone();
		let back_state = self.back_state.clone();

		query_spawn(back_state.0.clone(), async move {
			let mut raw_config = back_state.raw_config().await?;
			let modification = match keys.item.ty {
				ConfigKind::Instance if keys.item.is_new => ConfigModification::AddInstance(
					keys.item.id.unwrap().into(),
					keys.config.0.instance,
				),
				ConfigKind::Instance => ConfigModification::UpdateInstance(
					keys.item.id.unwrap().into(),
					keys.config.0.instance,
				),
				ConfigKind::Template if keys.item.is_new => {
					ConfigModification::AddTemplate(keys.item.id.unwrap().into(), keys.config.0)
				}
				ConfigKind::Template => {
					ConfigModification::UpdateTemplate(keys.item.id.unwrap().into(), keys.config.0)
				}
				ConfigKind::BaseTemplate => {
					raw_config.base_template = Some(keys.config.0);
					apply_modifications_and_write(
						&mut raw_config,
						Vec::new(),
						&back_state.paths,
								&mut NoOp,
					)
					.await?;
					return Ok(());
				}
			};

			apply_modifications_and_write(
				&mut raw_config,
				vec![modification],
				&back_state.paths,
				&mut NoOp,
			)
			.await?;

			Ok(())
		})
	}

	fn on_settled(
		&self,
		keys: &Self::Keys,
		_result: &Result<Self::Ok, Self::Err>,
	) -> impl Future<Output = ()> {
		async {
			invalidate_all::<FetchItems>().await;
			if let Some(id) = &keys.item.id {
				invalidate_matching::<FetchInstanceConfig>(id.clone()).await;
			}
			invalidate_matching::<FetchInstanceOrTemplateConfig>(keys.item.clone()).await;
		}
	}
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct FetchInstanceOutput {
	back_state: Captured<BackState>,
}

impl FetchInstanceOutput {
	pub fn new(id: String, log_file: Option<String>, back_state: BackState) -> Query<Self> {
		Query::new(
			(id, log_file),
			Self {
				back_state: Captured(back_state),
			},
		)
	}
}

impl QueryCapability for FetchInstanceOutput {
	type Ok = Option<Arc<str>>;
	type Err = anyhow::Error;
	type Keys = (String, Option<String>);

	fn run(&self, keys: &Self::Keys) -> impl Future<Output = Result<Self::Ok, Self::Err>> {
		let back_state = self.back_state.clone();
		let (id, log_file) = keys.clone();

		query_spawn(back_state.0.clone(), async move {
			if let Some(log_file) = log_file {
				let config = back_state.config().await?;
				let mut output = back_state.output();
				let instance = config
					.instances
					.get(&InstanceID::from(id.clone()))
					.context("Instance not found")?;

				return Ok(Some(
					instance
						.get_log(
							&log_file,
										&back_state.paths,
							&mut output,
						)
						.await?
						.into(),
				));
			}

			let mut contents = None;

			if let Some(entry) = back_state.running_instances.get_entry(&id, None).await {
				if let Some(path) = &entry.stdout_file {
					let full_path = back_state.paths.internal.join("stdio").join(path);
					if let Ok(text) = tokio::fs::read_to_string(&full_path).await {
						if !text.is_empty() {
							contents = Some(text);
						}
					}
				}
			}

			let latest_log_path = back_state.paths.data.join("instances").join(&id).join("logs").join("latest.log");
			if let Ok(log_text) = tokio::fs::read_to_string(&latest_log_path).await {
				if !log_text.is_empty() {
					if let Some(ref stdio_text) = contents {
						if log_text.len() > stdio_text.len() {
							contents = Some(log_text);
						}
					} else {
						contents = Some(log_text);
					}
				}
			}

			Ok(contents.map(Into::into))
		})
	}
}

simple_query!(
	name = FetchInstanceLogs,
	ok = Arc<[String]>,
	err = anyhow::Error,
	keys = String,
	fn run(&self, keys: &Self::Keys) -> impl Future<Output = Result<Self::Ok, Self::Err>> {
		let back_state = self.back_state.clone();
		let id = keys.clone();

		query_spawn(back_state.0.clone(), async move {
			let config = back_state.config().await?;
			let mut o = back_state.output();
			let instance = config
				.instances
				.get(&InstanceID::from(id.clone()))
				.context("Instance not found")?;

			let logs = instance.get_logs(&back_state.paths, &mut o).await?;

			Ok(logs.into_iter().collect())
		})
	}
);

simple_mutation!(
	name = UpdateInstance,
	ok = (),
	err = anyhow::Error,
	keys = UpdateInstanceKeys,
	fn run(&self, keys: &Self::Keys) -> impl Future<Output = Result<Self::Ok, Self::Err>> {
		let task_id = match keys.mode {
			UpdateInstanceMode::Full => Task::UpdateInstance(keys.id.clone()),
			UpdateInstanceMode::Packages => Task::UpdateInstancePackages(keys.id.clone()),
			UpdateInstanceMode::Modpack => Task::UpdateInstanceModpack(keys.id.clone()),
		};

		let facets = match keys.mode {
			UpdateInstanceMode::Full => UpdateFacets::all(),
			UpdateInstanceMode::Packages => UpdateFacets::packages(),
			UpdateInstanceMode::Modpack => UpdateFacets::modpack(),
		};
		let depth = if keys.force {
			UpdateDepth::Force
		} else {
			UpdateDepth::Full
		};

		update_instance_impl(
			self.back_state.0.clone(),
			keys.id.clone(),
			depth,
			facets,
			task_id,
		)
	}
);

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct UpdateInstanceKeys {
	pub id: String,
	pub mode: UpdateInstanceMode,
	pub force: bool,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum UpdateInstanceMode {
	Full,
	Packages,
	Modpack,
}

pub async fn update_instance_impl(
	back_state: BackState,
	instance_id: String,
	depth: UpdateDepth,
	facets: UpdateFacets,
	task_id: Task,
) -> anyhow::Result<()> {
	if back_state.is_instance_updating(&instance_id).await {
		return Ok(());
	}

	let mut o = back_state.output();
	o.set_task(task_id.clone());

	let back_state2 = back_state.clone();
	let task = {
		let mut config = back_state2.config().await?;
		let mut lock = Lockfile::open(&back_state2.paths).context("Failed to open lockfile")?;

		let core = config
			.get_core(Some(&get_ms_client_id()),
				&back_state2.client,
				&back_state2.paths,
				&mut o,
			)
			.await?;

		let instance_id = instance_id.clone();
		let paths = back_state2.paths.clone();
		async move {
			let instance_id2 = instance_id.clone();
			let Some(instance) = config.instances.get_mut(&InstanceID::from(instance_id)) else {
				bail!("Instance does not exist");
			};

			let mut ctx = InstanceUpdateContext {
				packages: &mut config.packages,
				accounts: &mut config.accounts,
				prefs: &config.prefs,
				paths: &paths,
				lock: &mut lock,
				client: &back_state2.client,
				output: &mut o,
				core: &core,
			};

			let updates_packages = facets.packages;

			instance
				.update(depth, facets, &mut ctx)
				.await
				.context("Failed to update instance")?;

			// Clear any resolution errors for this instance if we updated successfully
			if updates_packages {
				let mut data = back_state2.data();
				data.last_resolution_errors.remove(&instance_id2);
				let _ = data.write(&paths);
				let _ = back_state2.event_tx.send(BackEvent::InvalidateData);
			}

			back_state2.invalidate(BackDependency::InstanceContent(instance_id2));

			Ok(())
		}
	};

	let task = tokio::spawn(unsafe { MakeSend::new(task) });
	back_state.register_task(task_id, task);

	Ok(())
}

#[rustfmt::skip]
simple_mutation!(
	name = DeleteInstance,
	ok = (),
	err = anyhow::Error,
	keys = String,
	fn run(&self, keys: &Self::Keys) -> impl Future<Output = Result<Self::Ok, Self::Err>> {
		let back_state = self.back_state.clone();
		let id = keys.clone();

		query_spawn(back_state.0.clone(), async move {
			let config = back_state.config().await?;
			let mut o = back_state.output();
			o.set_task(Task::DeleteInstance);
			let instance = config
				.instances
				.get(&InstanceID::from(id.clone()))
				.context("Instance not found")?;
			instance
				.delete(&back_state.paths, &mut o)
				.await
				.context("Failed to delete instance")
		})
	}
	fn on_settled(
		&self,
		_keys: &Self::Keys,
		_result: &Result<Self::Ok, Self::Err>,
	) -> impl Future<Output = ()> {
		invalidate_all::<FetchItems>()
	}
);

#[rustfmt::skip]
simple_mutation!(
	name = DuplicateInstance,
	ok = String,
	err = anyhow::Error,
	keys = String,
	fn run(&self, keys: &Self::Keys) -> impl Future<Output = Result<Self::Ok, Self::Err>> {
		let back_state = self.back_state.clone();
		let id = keys.clone();

		query_spawn(back_state.0.clone(), async move {
			let config = back_state.config().await?;
			let mut o = back_state.output();
			let instance = config
				.instances
				.get(&InstanceID::from(id.clone()))
				.context("Instance not found")?;

			let mut copy_id = format!("{}_copy", id);
			let mut counter = 2;
			while config.instances.contains_key(&InstanceID::from(copy_id.clone())) {
				copy_id = format!("{}_copy_{}", id, counter);
				counter += 1;
			}
			let new_id = InstanceID::from(copy_id.clone());

			instance
				.duplicate(&new_id, &back_state.paths, &mut o)
				.await
				.context("Failed to duplicate instance")?;

			Ok(copy_id)
		})
	}
	fn on_settled(
		&self,
		_keys: &Self::Keys,
		_result: &Result<Self::Ok, Self::Err>,
	) -> impl Future<Output = ()> {
		invalidate_all::<FetchItems>()
	}
);

#[rustfmt::skip]
simple_mutation!(
	name = DeleteTemplate,
	ok = (),
	err = anyhow::Error,
	keys = String,
	fn run(&self, keys: &Self::Keys) -> impl Future<Output = Result<Self::Ok, Self::Err>> {
		let back_state = self.back_state.clone();
		let id = keys.clone();

		query_spawn(back_state.0.clone(), async move {
			let mut raw_config = back_state.raw_config().await?;
			let mut o = back_state.output();

			apply_modifications_and_write(
				&mut raw_config,
				vec![ConfigModification::RemoveTemplate(id.into())],
				&back_state.paths,
				&mut o,
			).await
		})
	}
	fn on_settled(
		&self,
		_keys: &Self::Keys,
		_result: &Result<Self::Ok, Self::Err>,
	) -> impl Future<Output = ()> {
		invalidate_all::<FetchItems>()
	}
);

simple_query!(
	name = FetchInstanceFiles,
	ok = InstanceFiles,
	err = anyhow::Error,
	keys = String,
	fn run(&self, keys: &Self::Keys) -> impl Future<Output = Result<Self::Ok, Self::Err>> {
		let back_state = self.back_state.clone();
		let id = keys.clone();

		query_spawn(back_state.0.clone(), async move {
			let config = back_state.config().await?;
			let Some(instance) = config.instances.get(&InstanceID::from(id.clone())) else {
				return Ok(InstanceFiles::default());
			};

			let saves = instance.get_saves()?;
			let screenshots = instance.get_screenshots()?;
			let servers = instance.get_servers()?;

			Ok(InstanceFiles {
				saves,
				screenshots,
				servers,
			})
		})
	}
);

#[derive(Default, Clone)]
pub struct InstanceFiles {
	pub saves: Vec<InstanceSave>,
	pub screenshots: Vec<PathBuf>,
	pub servers: Vec<ServerInfo>,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct AddServerParams {
	pub instance_id: String,
	pub name: String,
	pub address: String,
}

#[rustfmt::skip]
simple_mutation!(
	name = AddInstanceServer,
	ok = (),
	err = anyhow::Error,
	keys = AddServerParams,
	fn run(&self, keys: &Self::Keys) -> impl Future<Output = Result<Self::Ok, Self::Err>> {
		let back_state = self.back_state.clone();
		let params = keys.clone();

		query_spawn(back_state.0.clone(), async move {
			let config = back_state.config().await?;
			let Some(instance) = config.instances.get(&InstanceID::from(params.instance_id.clone())) else {
				anyhow::bail!("Instance not found");
			};

			instance.add_server(&params.name, &params.address)?;
			back_state.invalidate(BackDependency::InstanceContent(params.instance_id));

			Ok(())
		})
	}
);

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct RemoveServerParams {
	pub instance_id: String,
	pub address: String,
}

#[rustfmt::skip]
simple_mutation!(
	name = RemoveInstanceServer,
	ok = (),
	err = anyhow::Error,
	keys = RemoveServerParams,
	fn run(&self, keys: &Self::Keys) -> impl Future<Output = Result<Self::Ok, Self::Err>> {
		let back_state = self.back_state.clone();
		let params = keys.clone();

		query_spawn(back_state.0.clone(), async move {
			let config = back_state.config().await?;
			let Some(instance) = config.instances.get(&InstanceID::from(params.instance_id.clone())) else {
				anyhow::bail!("Instance not found");
			};

			instance.remove_server(&params.address)?;
			back_state.invalidate(BackDependency::InstanceContent(params.instance_id));

			Ok(())
		})
	}
);

/// Key for looking up files inside an instance directory
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct InstanceFileKey {
	pub instance_id: String,
	/// Path relative to the instance directory, empty string for the root
	pub path: String,
}

impl InstanceFileKey {
	pub fn new(instance_id: impl Into<String>, path: impl Into<String>) -> Self {
		Self {
			instance_id: instance_id.into(),
			path: path.into(),
		}
	}

	/// Returns a key for a direct child of this directory
	pub fn child(&self, name: &str) -> Self {
		let path = if self.path.is_empty() {
			name.to_string()
		} else {
			format!("{}/{}", self.path, name)
		};
		Self {
			instance_id: self.instance_id.clone(),
			path,
		}
	}

}

/// A single entry inside an instance directory
#[derive(Clone, PartialEq)]
pub struct InstanceDirEntry {
	pub name: String,
	pub is_dir: bool,
	pub size: u64,
	pub modified_secs: u64,
}

/// Resolves a relative path inside an instance directory, rejecting traversal
fn resolve_instance_path(dir: Option<&Path>, rel: &str) -> anyhow::Result<PathBuf> {
	let dir = dir.context("У инстанса нет директории")?;
	let rel_path = Path::new(rel);
	ensure!(
		rel.is_empty()
			|| (!rel_path.is_absolute()
				&& !rel_path
					.components()
					.any(|c| matches!(c, Component::ParentDir | Component::RootDir | Component::Prefix(_)))),
		"Недопустимый путь: {rel}"
	);
	Ok(if rel.is_empty() {
		dir.to_path_buf()
	} else {
		dir.join(rel_path)
	})
}

simple_query!(
	name = FetchInstanceDirFiles,
	ok = Arc<[InstanceDirEntry]>,
	err = anyhow::Error,
	keys = InstanceFileKey,
	fn run(&self, keys: &Self::Keys) -> impl Future<Output = Result<Self::Ok, Self::Err>> {
		let back_state = self.back_state.clone();
		let keys = keys.clone();

		query_spawn(back_state.0.clone(), async move {
			let config = back_state.config().await?;
			let Some(instance) = config.instances.get(&InstanceID::from(keys.instance_id.clone())) else {
				anyhow::bail!("Instance not found");
			};

			let path = resolve_instance_path(instance.dir(), &keys.path)?;
			let mut dir = tokio::fs::read_dir(&path)
				.await
				.with_context(|| format!("Не удалось открыть папку {}", keys.path))?;

			let mut out = Vec::new();
			while let Some(entry) = dir.next_entry().await? {
				let meta = entry.metadata().await?;
				let modified_secs = meta
					.modified()
					.ok()
					.and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
					.map(|d| d.as_secs())
					.unwrap_or(0);

				out.push(InstanceDirEntry {
					name: entry.file_name().to_string_lossy().to_string(),
					is_dir: meta.is_dir(),
					size: if meta.is_file() { meta.len() } else { 0 },
					modified_secs,
				});
			}

			Ok(out.into())
		})
	}
);

simple_query!(
	name = ReadInstanceFile,
	ok = Option<Arc<str>>,
	err = anyhow::Error,
	keys = QueryOption<InstanceFileKey>,
	fn run(&self, keys: &Self::Keys) -> impl Future<Output = Result<Self::Ok, Self::Err>> {
		let back_state = self.back_state.clone();
		let keys = keys.clone();

		query_spawn(back_state.0.clone(), async move {
			let Some(keys) = keys.0 else {
				return Ok(None);
			};

			let config = back_state.config().await?;
			let Some(instance) = config.instances.get(&InstanceID::from(keys.instance_id.clone())) else {
				anyhow::bail!("Instance not found");
			};

			let path = resolve_instance_path(instance.dir(), &keys.path)?;
			ensure!(path.is_file(), "Это не файл");

			let meta = tokio::fs::metadata(&path).await?;
			ensure!(
				meta.len() <= 2 * 1024 * 1024,
				"Файл слишком большой для редактирования (макс. 2 МБ)"
			);

			let bytes = tokio::fs::read(&path).await?;
			ensure!(!bytes.contains(&0), "Бинарный файл нельзя редактировать");

			Ok(Some(String::from_utf8_lossy(&bytes).into()))
		})
	}
);

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct WriteInstanceFileParams {
	pub instance_id: String,
	pub path: String,
	pub contents: String,
}

simple_mutation!(
	name = WriteInstanceFile,
	ok = (),
	err = anyhow::Error,
	keys = WriteInstanceFileParams,
	fn run(&self, keys: &Self::Keys) -> impl Future<Output = Result<Self::Ok, Self::Err>> {
		let back_state = self.back_state.clone();
		let params = keys.clone();

		query_spawn(back_state.0.clone(), async move {
			ensure!(!params.path.is_empty(), "Путь не может быть пустым");

			let config = back_state.config().await?;
			let Some(instance) = config.instances.get(&InstanceID::from(params.instance_id.clone())) else {
				anyhow::bail!("Instance not found");
			};

			let path = resolve_instance_path(instance.dir(), &params.path)?;
			if let Some(parent) = path.parent() {
				tokio::fs::create_dir_all(parent).await?;
			}
			tokio::fs::write(&path, params.contents).await?;

			Ok(())
		})
	}
);

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct CreateInstanceEntryParams {
	pub instance_id: String,
	pub path: String,
	pub is_dir: bool,
}

simple_mutation!(
	name = CreateInstanceEntry,
	ok = (),
	err = anyhow::Error,
	keys = CreateInstanceEntryParams,
	fn run(&self, keys: &Self::Keys) -> impl Future<Output = Result<Self::Ok, Self::Err>> {
		let back_state = self.back_state.clone();
		let params = keys.clone();

		query_spawn(back_state.0.clone(), async move {
			ensure!(!params.path.is_empty(), "Путь не может быть пустым");

			let config = back_state.config().await?;
			let Some(instance) = config.instances.get(&InstanceID::from(params.instance_id.clone())) else {
				anyhow::bail!("Instance not found");
			};

			let path = resolve_instance_path(instance.dir(), &params.path)?;
			ensure!(!path.exists(), "Такой файл или папка уже существуют");

			if params.is_dir {
				tokio::fs::create_dir_all(&path).await?;
			} else {
				if let Some(parent) = path.parent() {
					tokio::fs::create_dir_all(parent).await?;
				}
				tokio::fs::write(&path, b"").await?;
			}

			Ok(())
		})
	}
);

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct DeleteInstanceEntryParams {
	pub instance_id: String,
	pub path: String,
}

simple_mutation!(
	name = DeleteInstanceEntry,
	ok = (),
	err = anyhow::Error,
	keys = DeleteInstanceEntryParams,
	fn run(&self, keys: &Self::Keys) -> impl Future<Output = Result<Self::Ok, Self::Err>> {
		let back_state = self.back_state.clone();
		let params = keys.clone();

		query_spawn(back_state.0.clone(), async move {
			ensure!(!params.path.is_empty(), "Нельзя удалить корень инстанса");

			let config = back_state.config().await?;
			let Some(instance) = config.instances.get(&InstanceID::from(params.instance_id.clone())) else {
				anyhow::bail!("Instance not found");
			};

			let path = resolve_instance_path(instance.dir(), &params.path)?;
			ensure!(path.exists(), "Файл не найден");

			if path.is_dir() {
				tokio::fs::remove_dir_all(&path).await?;
			} else {
				tokio::fs::remove_file(&path).await?;
			}

			Ok(())
		})
	}
);

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct ImportExternalParams {
	pub name: String,
	pub id: String,
	pub version: String,
	pub loader: Option<String>,
	pub game_dir: PathBuf,
	pub is_sync: bool,
	pub icon: Option<String>,
}

#[rustfmt::skip]
simple_mutation!(
	name = ImportExternalInstance,
	ok = String,
	err = anyhow::Error,
	keys = ImportExternalParams,
	fn run(&self, keys: &Self::Keys) -> impl Future<Output = Result<Self::Ok, Self::Err>> {
		let back_state = self.back_state.clone();
		let params = keys.clone();

		query_spawn(back_state.0.clone(), async move {
			let mut raw_config = back_state.raw_config().await?;

			let mut target_id = params.id.clone();
			if target_id.is_empty() {
				target_id = sqrilizz::config_crate::instance::make_valid_instance_id(&params.name);
			}
			let mut candidate = target_id.clone();
			let mut counter = 2;
			while raw_config.instances.contains_key(&InstanceID::from(candidate.clone())) {
				candidate = format!("{}_{}", target_id, counter);
				counter += 1;
			}
			let new_id = InstanceID::from(candidate.clone());

			let dir_setting = if params.is_sync {
				Some(params.game_dir.to_string_lossy().to_string())
			} else {
				let dest = back_state.paths.data.join("instances").join(&*new_id);
				let _ = tokio::fs::create_dir_all(&dest).await;
				if params.game_dir.exists() {
					let _ = sqrilizz::core::io::files::copy_dir_contents(&params.game_dir, &dest);
				}
				None
			};

			let inst_config = InstanceConfig {
				name: Some(params.name),
				side: Some(Side::Client),
				version: Some(MinecraftVersionDeser::Version(params.version.into())),
				loader: params.loader,
				dir: dir_setting,
				imported: params.is_sync,
				icon: params.icon,
				..Default::default()
			};

			apply_modifications_and_write(
				&mut raw_config,
				vec![ConfigModification::AddInstance(new_id, inst_config)],
				&back_state.paths,
				&mut NoOp,
			)
			.await?;

			Ok(candidate)
		})
	}
	fn on_settled(
		&self,
		_keys: &Self::Keys,
		_result: &Result<Self::Ok, Self::Err>,
	) -> impl Future<Output = ()> {
		invalidate_all::<FetchItems>()
	}
);
