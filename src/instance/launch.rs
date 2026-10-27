use sqrilizz_core::{QuickPlayType, SqrilizzCore};
use std::collections::HashMap;
use std::io::Read;
use std::ops::DerefMut;
use std::path::Path;
use std::time::Duration;

use anyhow::Context;
use sqrilizz_config::instance::{QuickPlay, WrapperCommand};
use sqrilizz_core::account::AccountManager;
use sqrilizz_core::io::java::install::JavaInstallationKind;
use sqrilizz_shared::id::InstanceID;
use sqrilizz_shared::java_args::MemoryNum;
use sqrilizz_shared::output::{MessageContents, SqrilizzOutput};
use sqrilizz_shared::{Side, UpdateDepth, translate};
use tokio::io::{AsyncWriteExt, Stdout};

use super::tracking::RunningInstanceRegistry;
use super::update::manager::UpdateManager;
use crate::instance::tracking::RunningInstanceEntry;
use crate::instance::update::manager::UpdateSettings;
use crate::instance::update::{InstanceUpdateContext, UpdateFacets};
use crate::instance::world_files::WorldFilesWatcher;
use crate::io::paths::Paths;

use super::Instance;

impl Instance {
	/// Launch the instance process
	pub async fn launch<'a, O: SqrilizzOutput>(
		&mut self,
		settings: LaunchSettings,
		ctx: &mut InstanceUpdateContext<'a, O>,
	) -> anyhow::Result<InstanceHandle> {
		let _manager = UpdateManager::from_settings(UpdateSettings {
			depth: UpdateDepth::Shallow,
			offline_auth: settings.offline_auth,
		});

		self.update(UpdateDepth::Shallow, UpdateFacets::all(), ctx)
			.await
			.context("Failed to update instance")?;

		// Make sure that any fluff from the update gets ended
		ctx.output.end_process();

		ctx.output.display(MessageContents::Simple(translate!(
			ctx.output,
			PreparingLaunch
		)));
		let mut section = ctx.output.get_section();

		self.launch_standard(
			ctx.core,
			ctx.paths,
			settings,
			ctx.accounts,
			section.deref_mut(),
		)
		.await
	}

	/// Standard Java launch
	async fn launch_standard(
		&mut self,
		core: &SqrilizzCore,
		paths: &Paths,
		settings: LaunchSettings,
		accounts: &mut AccountManager,
		o: &mut impl SqrilizzOutput,
	) -> anyhow::Result<InstanceHandle> {
		let selected_account = accounts.get_chosen_account().map(|x| x.get_id().clone());
		let selected_account = selected_account.map(|x| x.to_string());

		let core_version = core
			.get_version(&self.version, UpdateDepth::Shallow, o)
			.await?;

		let mut instance = self
			.create_core_instance(&core_version, paths, o)
			.await
			.context("Failed to create core instance")?;

		instance.pipe_stdin(settings.pipe_stdin);

		// Launch the instance using core
		let handle = instance
			.launch_with_handle(accounts, settings.offline_auth, settings.quick_play, o)
			.await
			.context("Failed to launch core instance")?;

		let world_files = if self.side() == Side::Client {
			Some(
				WorldFilesWatcher::new(self.dir.as_ref().unwrap())
					.context("Failed to setup world files watcher")?,
			)
		} else {
			None
		};

		let handle = InstanceHandle {
			instance_id: self.id.clone(),
			stdout: tokio::io::stdout(),
			is_silent: false,
			output_fn: None,
			account: selected_account.clone(),
			inner: InstanceHandleInner::Standard {
				inner: handle,
				world_files,
			},
		};

		let _ = handle.create_tracking_entry(paths);

		Ok(handle)
	}
}

/// Settings for launch provided to the instance launch function
pub struct LaunchSettings {
	/// Whether to do offline auth
	pub offline_auth: bool,
	/// Whether to pipe the stdin of this process into the instance process
	pub pipe_stdin: bool,
	/// Quick play for the launch
	pub quick_play: Option<QuickPlayType>,
}

/// Options for launching after conversion from the deserialized version
#[derive(Debug)]
pub struct LaunchOptions {
	/// Java kind
	pub java: JavaInstallationKind,
	/// JVM arguments
	pub jvm_args: Vec<String>,
	/// Game arguments
	pub game_args: Vec<String>,
	/// Minimum JVM memory
	pub min_mem: Option<MemoryNum>,
	/// Maximum JVM memory
	pub max_mem: Option<MemoryNum>,
	/// Environment variables
	pub env: HashMap<String, String>,
	/// Wrapper command
	pub wrapper: Option<WrapperCommand>,
	/// Quick Play options
	pub quick_play: QuickPlay,
	/// Whether or not to use the Log4J configuration
	pub use_log4j_config: bool,
}

/// A handle for an instance
pub struct InstanceHandle {
	/// The ID of the instance
	instance_id: InstanceID,
	/// Global stdout
	stdout: Stdout,
	/// Whether to redirect stdin and stdout to the process stdin and stdout
	is_silent: bool,
	/// Custom function when outputting to stdout
	output_fn: Option<Box<dyn FnMut(&[u8]) + 'static>>,
	/// The account that launched this instance
	account: Option<String>,
	/// Inner implementation
	inner: InstanceHandleInner,
}

/// Inner implementation for an InstanceHandle
enum InstanceHandleInner {
	Standard {
		/// Core InstanceHandle with the process
		inner: sqrilizz_core::InstanceHandle,
		/// Shared world file watcher
		world_files: Option<WorldFilesWatcher>,
	},
}

impl InstanceHandle {
	fn create_tracking_entry(&self, paths: &Paths) -> anyhow::Result<()> {
		let mut registry = RunningInstanceRegistry::open(paths)
			.context("Failed to open registry of running instances")?;
		let entry = RunningInstanceEntry {
			instance_id: self.instance_id.to_string(),
			pid: self.get_pid(),
			parent_pid: std::process::id(),
			is_java: matches!(&self.inner, InstanceHandleInner::Standard { .. }),
			stdin_file: self
				.stdin()
				.map(|x| x.file_name().unwrap().to_string_lossy().to_string()),
			stdout_file: Some(
				self.stdout()
					.file_name()
					.unwrap()
					.to_string_lossy()
					.to_string(),
			),
			account: self.account.clone(),
		};
		registry.add_instance(entry);
		registry.write()
	}

	/// Waits for the process to complete
	pub async fn wait(
		mut self,
		o: &mut impl SqrilizzOutput,
	) -> anyhow::Result<std::process::ExitStatus> {
		// Wait for the process to complete while polling stdio
		let mut stdio_buf = [0u8; 512];
		let status = loop {
			// Instance stdio
			if !self.is_silent {
				let inst_stdout = match &mut self.inner {
					InstanceHandleInner::Standard { inner, .. } => inner.stdout(),
				};

				// This is non-blocking as the stdout file will have an EoF
				if let Ok(bytes_read) = inst_stdout.read(&mut stdio_buf) {
					if let Some(output_fn) = &mut self.output_fn {
						output_fn(&stdio_buf[0..bytes_read]);
					} else {
						let _ = self.stdout.write_all(&stdio_buf[0..bytes_read]).await;
					}
				}
			}

			// Update world files
			if let InstanceHandleInner::Standard {
				world_files: Some(world_files),
				..
			} = &mut self.inner
			{
				let _ = world_files.watch(o).await;
			}

			// Check if the instance has exited
			match &mut self.inner {
				InstanceHandleInner::Standard { inner, .. } => {
					if let Ok(Some(status)) = inner.try_wait() {
						break status;
					}
				}
			}

			tokio::time::sleep(Duration::from_millis(50)).await;
		};

		let pid = self.get_pid();
		Self::on_stop(&self.instance_id, pid, self.account.as_deref(), o).await?;

		Ok(status)
	}

	/// Kills the process early
	pub async fn kill(self, o: &mut impl SqrilizzOutput) -> anyhow::Result<()> {
		let pid = self.get_pid();

		match self.inner {
			InstanceHandleInner::Standard { mut inner, .. } => {
				let _ = inner.kill();
			}
		}

		Self::on_stop(&self.instance_id, pid, self.account.as_deref(), o).await?;

		Ok(())
	}

	/// Gets the internal child process for the game, consuming the
	/// InstanceHandle
	pub fn get_process(self) -> Option<std::process::Child> {
		match self.inner {
			InstanceHandleInner::Standard { inner, .. } => Some(inner.get_process()),
		}
	}

	/// Gets the PID of the instance process
	pub fn get_pid(&self) -> u32 {
		match &self.inner {
			InstanceHandleInner::Standard { inner, .. } => inner.get_pid(),
		}
	}

	/// Set whether the stdio of the instance should be redirected to this process
	pub fn silence_output(&mut self, is_silent: bool) {
		self.is_silent = is_silent;
	}

	/// Sets the output function for this handle to handle stdout in a custom way
	pub fn set_output_fn(&mut self, f: Box<dyn FnMut(&[u8]) + 'static>) {
		self.output_fn = Some(f);
	}

	/// Get the stdout file path for this instance
	pub fn stdout(&self) -> &Path {
		match &self.inner {
			InstanceHandleInner::Standard { inner, .. } => inner.stdout_path(),
		}
	}

	/// Get the stdin file path for this instance
	pub fn stdin(&self) -> Option<&Path> {
		match &self.inner {
			InstanceHandleInner::Standard { inner, .. } => inner.stdin_path(),
		}
	}

	/// Writes to stdin
	pub fn write_stdin(&mut self, data: &[u8]) -> anyhow::Result<()> {
		match &mut self.inner {
			InstanceHandleInner::Standard { inner, .. } => inner
				.write_stdin(data)
				.context("Failed to write to inner stdin"),
		}
	}

	/// Function that should be run whenever the instance stops
	async fn on_stop(
		instance_id: &str,
		pid: u32,
		account: Option<&str>,
		o: &mut impl SqrilizzOutput,
	) -> anyhow::Result<()> {
		let mut process = o.get_process();
		let msg = translate!(process, Stop);
		process.display(MessageContents::StartProcess(msg));

		let paths = Paths::new_no_create()?;

		// Remove the instance from the registry
		let registry = RunningInstanceRegistry::open(&paths);
		if let Ok(mut registry) = registry {
			registry.remove_instance(pid, instance_id, account);
			let _ = registry.write();
		}

		let msg = translate!(process, Stopped);
		process.display(MessageContents::Success(msg));

		Ok(())
	}
}
