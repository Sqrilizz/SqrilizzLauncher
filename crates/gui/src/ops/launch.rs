use std::io::Write;

use anyhow::Context;
use sqrilizz::{
	core::{QuickPlayType, io::open_named_pipe},
	instance::{
		launch::LaunchSettings, tracking::RunningInstanceEntry, update::InstanceUpdateContext,
	},
	io::lock::Lockfile,
	shared::id::InstanceID,
};

use crate::{
	data::LauncherData,
	dependency::BackDependency,
	ops::{MakeSend, discord, task::Task},
	prelude::*,
	secrets::get_ms_client_id,
};

struct PresenceGuard {
	back_state: BackState,
}

impl Drop for PresenceGuard {
	fn drop(&mut self) {
		let bs = self.back_state.clone();
		tokio::spawn(async move {
			let running = bs.running_instances.get_running_instances().await;
			if running.is_empty() {
				discord::clear_presence();
			} else if let Some(first) = running.first() {
				discord::set_instance_presence(&first.instance_id, None);
			}
		});
	}
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct LaunchInstance {
	pub back_state: Captured<BackState>,
}

#[derive(Clone, PartialEq, Hash)]
pub struct LaunchInstanceParams {
	pub id: String,
	pub offline: bool,
	pub quick_play: QuickPlayType,
}

impl LaunchInstance {
	pub fn new(back_state: BackState) -> Mutation<Self> {
		Mutation::new(Self {
			back_state: Captured(back_state),
		})
	}
}

impl MutationCapability for LaunchInstance {
	type Ok = ();
	type Err = anyhow::Error;
	type Keys = LaunchInstanceParams;

	fn run(&self, keys: &Self::Keys) -> impl Future<Output = Result<Self::Ok, Self::Err>> {
		let id = keys.id.clone();
		let offline = keys.offline;
		let quick_play = keys.quick_play.clone();
		let back_state = self.back_state.clone();

		let task = async move {
			let mut config = back_state.config().await?;
			let mut output = back_state.output();
			output.set_task(Task::LaunchInstance(id.clone()));

			let data = LauncherData::open(&back_state.paths)?;
			if let Some(account) = data.current_account {
				let _ = config.accounts.choose_account(&account);
			}

			let core = config
				.get_core(
					Some(&get_ms_client_id()),
					&back_state.client,
					&back_state.paths,
					&mut output,
				)
				.await?;

			let instance = config
				.instances
				.get_mut(&InstanceID::from(id.clone()))
				.context("Instance does not exist")?;

			let settings = LaunchSettings {
				offline_auth: offline,
				pipe_stdin: false,
				quick_play: Some(quick_play),
			};

			let mut lock = Lockfile::open(&back_state.paths)?;
			let mut ctx = InstanceUpdateContext {
				packages: &config.packages,
				accounts: &mut config.accounts,
				prefs: &config.prefs,
				paths: &back_state.paths,
				lock: &mut lock,
				client: &back_state.client,
				output: &mut output,
				core: &core,
			};

			let mut handle = instance
				.launch(settings, &mut ctx)
				.await
				.context("Failed to launch instance")?;

			handle.silence_output(true);

			let now_ts = std::time::SystemTime::now()
				.duration_since(std::time::UNIX_EPOCH)
				.unwrap_or_default()
				.as_secs();
			if let Ok(mut data) = LauncherData::open(&back_state.paths) {
				*data.play_launches.entry(id.clone()).or_insert(0) += 1;
				data.last_played.insert(id.clone(), now_ts);
				let _ = data.write(&back_state.paths);
				back_state.invalidate(BackDependency::Items);
			}

			let display_name = instance.config().name.as_deref().unwrap_or(&id);
			let mut details_parts = Vec::new();
			if let Some(ref ver) = instance.config().version {
				match ver {
					sqrilizz::shared::versions::MinecraftVersionDeser::Version(v) => {
						details_parts.push(format!("MC {v}"));
					}
					_ => {
						details_parts.push("Minecraft".to_string());
					}
				}
			}
			if let Some(ref loader) = instance.config().loader {
				details_parts.push(loader.to_string());
			}
			let details_str = if details_parts.is_empty() {
				None
			} else {
				Some(details_parts.join(" • "))
			};
			discord::set_instance_presence(display_name, details_str.as_deref());
			let _presence_guard = PresenceGuard {
				back_state: (*back_state).clone(),
			};

			let stdio_path = handle.stdout().to_path_buf();
			let mut wait_fut = Box::pin(handle.wait(&mut output));
			let mut interval = tokio::time::interval(std::time::Duration::from_secs(5));
			let mut last_tick = std::time::Instant::now();
			let wait_res;
			loop {
				tokio::select! {
					res = &mut wait_fut => {
						let elapsed = last_tick.elapsed().as_secs();
						if elapsed > 0 {
							if let Ok(mut data) = LauncherData::open(&back_state.paths) {
								*data.playtimes.entry(id.clone()).or_insert(0) += elapsed;
								let _ = data.write(&back_state.paths);
								back_state.invalidate(BackDependency::Items);
							}
						}
						wait_res = res;
						break;
					}
					_ = interval.tick() => {
						let elapsed = last_tick.elapsed().as_secs();
						if elapsed >= 5 {
							last_tick = std::time::Instant::now();
							if let Ok(mut data) = LauncherData::open(&back_state.paths) {
								*data.playtimes.entry(id.clone()).or_insert(0) += elapsed;
								let _ = data.write(&back_state.paths);
								back_state.invalidate(BackDependency::Items);
							}
						}
					}
				}
			}

			drop(wait_fut);
			output.finish_task();
			let exit_status = wait_res?;

			if !exit_status.success() {
				let mut log_text = if let Some(inst_dir) = instance.dir() {
					let latest_log = inst_dir.join("logs").join("latest.log");
					std::fs::read_to_string(&latest_log).unwrap_or_default()
				} else {
					String::new()
				};

				if let Ok(process_output) = std::fs::read_to_string(&stdio_path) {
					if !process_output.is_empty() {
						log_text.push_str("\n--- JVM Process Output ---\n");
						log_text.push_str(&process_output);
					}
				}

				let (title, advice) = diagnose_crash(&log_text, exit_status.code());
				let _ = back_state.event_tx.send(crate::state::BackEvent::ErrorToast(title, Some(advice)));
			}

			Ok(())
		};

		let task = unsafe { MakeSend::new(task) };
		self.back_state
			.register_task(Task::LaunchInstance(keys.id.clone()), tokio::spawn(task));

		async { Ok(()) }
	}
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct FetchRunningInstances {
	back_state: Captured<BackState>,
}

impl FetchRunningInstances {
	pub fn new(back_state: BackState) -> Query<Self> {
		Query::new(
			(),
			Self {
				back_state: Captured(back_state),
			},
		)
	}
}

impl QueryCapability for FetchRunningInstances {
	type Ok = Vec<RunningInstanceEntry>;
	type Err = anyhow::Error;
	type Keys = ();

	fn run(&self, _: &Self::Keys) -> impl Future<Output = Result<Self::Ok, Self::Err>> {
		let back_state = self.back_state.clone();
		async move { Ok(back_state.running_instances.get_running_instances().await) }
	}
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct KillInstance {
	back_state: Captured<BackState>,
}

impl KillInstance {
	pub fn new(back_state: BackState) -> Mutation<Self> {
		Mutation::new(Self {
			back_state: Captured(back_state),
		})
	}
}

impl MutationCapability for KillInstance {
	type Ok = ();
	type Err = anyhow::Error;
	type Keys = (String, Option<String>);

	fn run(&self, keys: &Self::Keys) -> impl Future<Output = Result<Self::Ok, Self::Err>> {
		let (id, account) = keys.clone();
		let back_state = self.back_state.clone();
		async move {
			back_state
				.kill_task(&Task::LaunchInstance(id.clone()))
				.await;
			back_state
				.running_instances
				.kill(&id, account.as_deref())
				.await;
			Ok(())
		}
	}
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct RestartInstance {
	back_state: Captured<BackState>,
}

impl RestartInstance {
	pub fn new(back_state: BackState) -> Self {
		Self {
			back_state: Captured(back_state),
		}
	}
}

impl MutationCapability for RestartInstance {
	type Ok = ();
	type Err = anyhow::Error;
	type Keys = String;

	fn run(&self, keys: &Self::Keys) -> impl Future<Output = Result<Self::Ok, Self::Err>> {
		let id = keys.clone();
		let back_state = self.back_state.clone();

		async move {
			back_state.running_instances.kill(&id, None).await;

			// Wait for the instance process to fully stop before relaunching
			let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
			loop {
				let still_running = back_state
					.running_instances
					.get_running_instances()
					.await
					.iter()
					.any(|x| x.instance_id == id);
				if !still_running || std::time::Instant::now() > deadline {
					break;
				}
				tokio::time::sleep(std::time::Duration::from_millis(250)).await;
			}

			LaunchInstance {
				back_state: self.back_state.clone(),
			}
			.run(&LaunchInstanceParams {
				id: id.clone(),
				offline: false,
				quick_play: QuickPlayType::default(),
			})
			.await
		}
	}
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct FetchInstanceRunState {
	back_state: Captured<BackState>,
}

impl FetchInstanceRunState {
	pub fn new(back_state: BackState) -> Self {
		Self {
			back_state: Captured(back_state),
		}
	}
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum InstanceRunState {
	#[default]
	Stopped,
	Running,
}

impl QueryCapability for FetchInstanceRunState {
	type Ok = InstanceRunState;
	type Err = anyhow::Error;
	type Keys = String;

	fn run(&self, id: &Self::Keys) -> impl Future<Output = Result<Self::Ok, Self::Err>> {
		let id = id.clone();
		let back_state = self.back_state.clone();
		async move {
			let is_running = back_state
				.running_instances
				.get_running_instances()
				.await
				.iter()
				.any(|x| x.instance_id == id);
			if is_running {
				Ok(InstanceRunState::Running)
			} else {
				Ok(InstanceRunState::Stopped)
			}
		}
	}
}

#[derive(Clone, PartialEq, Eq, Hash)]
#[allow(dead_code)]
pub struct FetchIsInstanceRunning {
	back_state: Captured<BackState>,
}

impl FetchIsInstanceRunning {
	#[allow(dead_code)]
	pub fn new(back_state: BackState) -> Self {
		Self {
			back_state: Captured(back_state),
		}
	}
}

impl QueryCapability for FetchIsInstanceRunning {
	type Ok = bool;
	type Err = anyhow::Error;
	type Keys = String;

	fn run(&self, id: &Self::Keys) -> impl Future<Output = Result<Self::Ok, Self::Err>> {
		let id = id.clone();
		let back_state = self.back_state.clone();
		async move {
			let is_running = back_state
				.running_instances
				.get_running_instances()
				.await
				.iter()
				.any(|x| x.instance_id == id);
			Ok(is_running)
		}
	}
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct WriteInstanceInput {
	back_state: Captured<BackState>,
}

impl WriteInstanceInput {
	pub fn new(back_state: BackState) -> Self {
		Self {
			back_state: Captured(back_state),
		}
	}
}

impl MutationCapability for WriteInstanceInput {
	type Ok = ();
	type Err = anyhow::Error;
	type Keys = (String, String);

	fn run(&self, keys: &Self::Keys) -> impl Future<Output = Result<Self::Ok, Self::Err>> {
		let (id, line) = keys.clone();
		let back_state = self.back_state.clone();
		async move {
			let running = back_state.running_instances.get_running_instances().await;
			let entry = running
				.iter()
				.find(|x| x.instance_id == id)
				.context("Instance is not running")?;
			let stdin_file = entry
				.stdin_file
				.as_ref()
				.context("Stdin is not piped for this instance")?;
			let stdin_path = back_state.paths.core.stdio.join(stdin_file);
			let mut pipe = open_named_pipe(&stdin_path).context("Failed to open named pipe")?;
			pipe.write_all(line.as_bytes())
				.context("Failed to write to stdin pipe")?;
			pipe.write_all(
				b"
",
			)
			.context("Failed to write newline to stdin pipe")?;
			Ok(())
		}
	}
}

pub fn diagnose_crash(log: &str, exit_code: Option<i32>) -> (String, String) {
	let code_str = exit_code.map(|c| format!(" (код {c})")).unwrap_or_default();
	let title = format!("Игра завершилась с ошибкой{code_str}");

	if log.contains("java.lang.OutOfMemoryError") || log.contains("OutOfMemory") {
		return (
			title,
			"Недостаточно оперативной памяти (OutOfMemoryError). Увеличьте объем ОЗУ в настройках инстанса.".into(),
		);
	}
	if log.contains("UnsupportedClassVersionError") || log.contains("has been compiled by a more recent version of the Java Runtime") {
		return (
			title,
			"Несовместимая версия Java: один из модов требует более новую версию Java (например, Java 21). Измените версию Java в настройках запуска.".into(),
		);
	}
	if log.contains("DuplicateModsFoundException") || log.contains("duplicate mods found") {
		return (
			title,
			"Обнаружены дублирующиеся моды. Проверьте список установленных модов на наличие двух версий одного мода.".into(),
		);
	}
	if log.contains("ModResolutionException") || log.contains("IncompatibleModsException") || log.contains("Could not find required mod") {
		return (
			title,
			"Конфликт модов или отсутствуют обязательные зависимости (например, Fabric API). Проверьте консоль для деталей.".into(),
		);
	}
	if log.contains("MixinApplyError") || log.contains("org.spongepowered.asm.mixin.transformer.throwables") {
		return (
			title,
			"Ошибка применения Mixin: несовместимость между модами. Попробуйте отключить недавно добавленные моды.".into(),
		);
	}

	(
		title,
		"Игра аварийно закрылась. Откройте вкладку «Консоль» для просмотра полного лога ошибки.".into(),
	)
}
