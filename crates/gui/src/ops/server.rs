use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::Context;
use sqrilizz::core::io::open_named_pipe;
use sqrilizz::instance::tracking::get_process_usage;
use sqrilizz::shared::id::InstanceID;

use crate::ops::server_ping::{ServerPingState, ping_server};
use crate::prelude::*;
use crate::{simple_mutation, simple_query};
use crate::state::BackState;

/// Live usage statistics of a running server process
#[derive(Default, Clone, PartialEq)]
pub struct ServerStats {
	/// Whether the instance is currently running
	pub running: bool,
	/// The PID of the server process
	pub pid: u32,
	/// The memory usage of the process in bytes
	pub mem_bytes: u64,
	/// The CPU usage of the process as a percentage
	pub cpu_percent: f32,
	/// How long the process has been running, in seconds
	pub uptime_secs: u64,
}

simple_query!(
	name = FetchServerStats,
	ok = ServerStats,
	err = anyhow::Error,
	keys = String,
	fn run(&self, keys: &Self::Keys) -> impl Future<Output = Result<Self::Ok, Self::Err>> {
		let back_state = self.back_state.clone();
		let id = keys.clone();

		query_spawn(back_state.0.clone(), async move {
			let Some(entry) = back_state.running_instances.get_entry(&id, None).await else {
				return Ok(ServerStats::default());
			};

			let usage = get_process_usage(entry.pid);

			let before = usage.map(|x| x.start_time_secs).unwrap_or_default();
			let now = std::time::SystemTime::now()
				.duration_since(std::time::UNIX_EPOCH)
				.unwrap_or_default()
				.as_secs();

			Ok(ServerStats {
				running: true,
				pid: entry.pid,
				mem_bytes: usage.map(|x| x.mem_bytes).unwrap_or_default(),
				cpu_percent: usage.map(|x| x.cpu_percent).unwrap_or_default(),
				uptime_secs: now.saturating_sub(before),
			})
		})
	}
);

/// TPS values from a Paper-style server, for 1m, 5m and 15m windows
pub type TpsValues = (f32, f32, f32);

/// Writes a command to a running instance's console stdin without waiting for a response
pub async fn write_console_command(
	back_state: &BackState,
	id: &str,
	command: &str,
) -> anyhow::Result<()> {
	let entry = back_state
		.running_instances
		.get_entry(id, None)
		.await
		.context("Instance is not running")?;

	let stdin_file = entry
		.stdin_file
		.as_ref()
		.context("Console input is not available for this instance")?;
	let stdin_path = back_state.paths.core.stdio.join(stdin_file);

	let mut pipe = open_named_pipe(&stdin_path).context("Failed to open named pipe")?;
	pipe.write_all(command.as_bytes())
		.context("Failed to write to stdin pipe")?;
	pipe.write_all(b"\n")
		.context("Failed to write newline to stdin pipe")?;

	Ok(())
}

/// Returns the path of the stdout capture file of a running instance, if any
#[allow(dead_code)]
pub async fn get_instance_stdout_path(back_state: &BackState, id: &str) -> Option<PathBuf> {
	let entry = back_state.running_instances.get_entry(id, None).await?;
	let stdout_file = entry.stdout_file.as_ref()?;
	Some(back_state.paths.core.stdio.join(stdout_file))
}

/// Sends a command to a running instance's console and returns the new
/// text that appeared in the console output since
pub async fn send_console_command(
	back_state: &BackState,
	id: &str,
	command: &str,
) -> anyhow::Result<String> {
	let entry = back_state
		.running_instances
		.get_entry(id, None)
		.await
		.context("Instance is not running")?;

	// Capture the length of the console output before sending the command
	let stdout_before = match &entry.stdout_file {
		Some(stdout_file) => tokio::fs::metadata(back_state.paths.core.stdio.join(stdout_file))
			.await
			.ok()
			.map(|x| x.len()),
		None => None,
	};

	write_console_command(back_state, id, command).await?;

	// Give the server time to respond
	tokio::time::sleep(Duration::from_millis(1200)).await;

	let mut tail = String::new();
	if let Some(stdout_file) = &entry.stdout_file {
		let path = back_state.paths.core.stdio.join(stdout_file);
		if let Ok(data) = tokio::fs::read(&path).await {
			let start = stdout_before
				.map(|x| (x as usize).min(data.len()))
				.unwrap_or(0);
			tail = String::from_utf8_lossy(&data[start..]).to_string();
		}
	}

	Ok(tail)
}

/// Removes ANSI escape sequences from console output
pub fn strip_ansi(input: &str) -> String {
	let mut out = String::with_capacity(input.len());
	let mut chars = input.chars();
	while let Some(c) = chars.next() {
		if c == '\x1b' {
			// Skip until a letter terminates the escape sequence
			for skipped in chars.by_ref() {
				if skipped.is_ascii_alphabetic() {
					break;
				}
			}
		} else {
			out.push(c);
		}
	}
	out
}

/// Parses TPS values from the output of the `tps` command on Paper-style servers
pub fn parse_tps_output(text: &str) -> Option<TpsValues> {
	for line in strip_ansi(text).lines().rev() {
		let Some(values) = line
			.split("TPS from last")
			.nth(1)
			.and_then(|x| x.split(':').nth(1))
		else {
			continue;
		};

		let nums: Vec<f32> = values
			.split(',')
			.filter_map(|x| x.trim().parse().ok())
			.collect();
		if nums.len() >= 3 {
			return Some((nums[0], nums[1], nums[2]));
		}
	}
	None
}

simple_mutation!(
	name = PollServerTps,
	ok = Option<TpsValues>,
	err = anyhow::Error,
	keys = String,
	fn run(&self, keys: &Self::Keys) -> impl Future<Output = Result<Self::Ok, Self::Err>> {
		let back_state = self.back_state.clone();
		let id = keys.clone();

		query_spawn(back_state.0.clone(), async move {
			let tail = send_console_command(&back_state, &id, "tps").await?;
			Ok(parse_tps_output(&tail))
		})
	}
);

simple_mutation!(
	name = SendServerCommand,
	ok = (),
	err = anyhow::Error,
	keys = (String, String),
	fn run(&self, keys: &Self::Keys) -> impl Future<Output = Result<Self::Ok, Self::Err>> {
		let back_state = self.back_state.clone();
		let (id, command) = keys.clone();

		query_spawn(back_state.0.clone(), async move {
			send_console_command(&back_state, &id, &command).await?;
			Ok(())
		})
	}
);

/// The ping status of a running server, refreshed periodically
pub type ServerPingStatus = Option<ServerPingState>;

/// Reads the configured server port from server.properties in the instance directory
pub async fn get_server_port(dir: &Path) -> u16 {
	let Ok(content) = tokio::fs::read_to_string(dir.join("server.properties")).await else {
		return 25565;
	};

	content
		.lines()
		.find(|line| line.trim_start().starts_with("server-port="))
		.and_then(|line| line.split('=').nth(1))
		.and_then(|port| port.trim().parse().ok())
		.unwrap_or(25565)
}

simple_query!(
	name = FetchServerAddress,
	ok = Option<String>,
	err = anyhow::Error,
	keys = String,
	fn run(&self, keys: &Self::Keys) -> impl Future<Output = Result<Self::Ok, Self::Err>> {
		let back_state = self.back_state.clone();
		let id = keys.clone();

		query_spawn(back_state.0.clone(), async move {
			let config = back_state.config().await?;
			let Some(instance) = config.instances.get(&InstanceID::from(id)) else {
				return Ok(None);
			};
			let Some(dir) = instance.dir() else {
				return Ok(None);
			};

			let port = get_server_port(dir).await;
			Ok(Some(format!("localhost:{port}")))
		})
	}
);

simple_query!(
	name = FetchServerPing,
	ok = ServerPingStatus,
	err = anyhow::Error,
	keys = String,
	fn run(&self, keys: &Self::Keys) -> impl Future<Output = Result<Self::Ok, Self::Err>> {
		let back_state = self.back_state.clone();
		let id = keys.clone();

		query_spawn(back_state.0.clone(), async move {
			let config = back_state.config().await?;
			let Some(instance) = config.instances.get(&InstanceID::from(id.clone())) else {
				return Ok(None);
			};
			let Some(dir) = instance.dir() else {
				return Ok(None);
			};

			let port = get_server_port(dir).await;
			let state = ping_server(&format!("localhost:{port}")).await;

			Ok(Some(state))
		})
	}
);

/// Info about a world folder on the server
#[derive(Clone, PartialEq)]
pub struct WorldInfo {
	pub name: String,
	pub path: PathBuf,
	pub size_bytes: u64,
}

/// Recursively computes the size of a directory on disk
fn dir_size<'a>(dir: &'a Path) -> std::pin::Pin<Box<dyn Future<Output = u64> + Send + 'a>> {
	Box::pin(async move {
		let mut total = 0;
		let Ok(mut entries) = tokio::fs::read_dir(dir).await else {
			return 0;
		};
		while let Ok(Some(entry)) = entries.next_entry().await {
			let path = entry.path();
			if path.is_dir() {
				total += dir_size(&path).await;
			} else if let Ok(meta) = entry.metadata().await {
				total += meta.len();
			}
		}
		total
	})
}

/// Checks whether a directory looks like a Minecraft world
async fn is_world_dir(path: &Path) -> bool {
	let is_dir = tokio::fs::metadata(path)
		.await
		.map(|x| x.is_dir())
		.unwrap_or(false);
	is_dir
		&& tokio::fs::try_exists(path.join("level.dat")).await.unwrap_or(false)
}

simple_query!(
	name = FetchServerWorlds,
	ok = Vec<WorldInfo>,
	err = anyhow::Error,
	keys = String,
	fn run(&self, keys: &Self::Keys) -> impl Future<Output = Result<Self::Ok, Self::Err>> {
		let back_state = self.back_state.clone();
		let id = keys.clone();

		query_spawn(back_state.0.clone(), async move {
			let config = back_state.config().await?;
			let Some(instance) = config.instances.get(&InstanceID::from(id.clone())) else {
				return Ok(Vec::new());
			};
			let Some(dir) = instance.dir() else {
				return Ok(Vec::new());
			};

			let mut worlds = Vec::new();
			let mut entries = tokio::fs::read_dir(dir)
				.await
				.context("Failed to read instance directory")?;
			while let Some(entry) = entries.next_entry().await? {
				let path = entry.path();
				if is_world_dir(&path).await {
					let name = entry
						.file_name()
						.to_string_lossy()
						.to_string();
					let size = dir_size(&path).await;
					worlds.push(WorldInfo {
						name,
						path,
						size_bytes: size,
					});
				}
			}
			worlds.sort_by(|a, b| a.name.cmp(&b.name));

			Ok(worlds)
		})
	}
);

/// Info about a world backup
#[derive(Clone, PartialEq)]
pub struct BackupInfo {
	pub name: String,
	pub path: PathBuf,
	pub size_bytes: u64,
}

fn get_backups_dir(dir: &Path) -> PathBuf {
	dir.join("backups")
}

simple_query!(
	name = FetchServerBackups,
	ok = Vec<BackupInfo>,
	err = anyhow::Error,
	keys = String,
	fn run(&self, keys: &Self::Keys) -> impl Future<Output = Result<Self::Ok, Self::Err>> {
		let back_state = self.back_state.clone();
		let id = keys.clone();

		query_spawn(back_state.0.clone(), async move {
			let config = back_state.config().await?;
			let Some(instance) = config.instances.get(&InstanceID::from(id.clone())) else {
				return Ok(Vec::new());
			};
			let Some(dir) = instance.dir() else {
				return Ok(Vec::new());
			};

			let backups_dir = get_backups_dir(dir);
			if !backups_dir.exists() {
				return Ok(Vec::new());
			}

			let mut backups = Vec::new();
			let mut entries = tokio::fs::read_dir(&backups_dir).await?;
			while let Some(entry) = entries.next_entry().await? {
				let path = entry.path();
				if path.is_dir() {
					let name = entry.file_name().to_string_lossy().to_string();
					let size = dir_size(&path).await;
					backups.push(BackupInfo {
						name,
						path,
						size_bytes: size,
					});
				}
			}
			backups.sort_by(|a, b| b.name.cmp(&a.name));

			Ok(backups)
		})
	}
);

/// Recursively copies a directory to a new location
fn copy_dir_recursive<'a>(
	from: &'a Path,
	to: &'a Path,
) -> std::pin::Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'a>> {
	Box::pin(async move {
		tokio::fs::create_dir_all(to).await?;
		let mut entries = tokio::fs::read_dir(from).await?;
		while let Some(entry) = entries.next_entry().await? {
			let path = entry.path();
			let dest = to.join(entry.file_name());
			if path.is_dir() {
				copy_dir_recursive(&path, &dest).await?;
			} else {
				tokio::fs::copy(&path, &dest).await?;
			}
		}
		Ok(())
	})
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct BackupWorldParams {
	pub instance_id: String,
	pub world_name: String,
}

#[rustfmt::skip]
simple_mutation!(
	name = BackupWorld,
	ok = (),
	err = anyhow::Error,
	keys = BackupWorldParams,
	fn run(&self, keys: &Self::Keys) -> impl Future<Output = Result<Self::Ok, Self::Err>> {
		let back_state = self.back_state.clone();
		let params = keys.clone();

		query_spawn(back_state.0.clone(), async move {
			let config = back_state.config().await?;
			let instance = config
				.instances
				.get(&InstanceID::from(params.instance_id.clone()))
				.context("Instance not found")?;
			let dir = instance.dir().context("Instance directory is missing")?;

			let world_path = dir.join(&params.world_name);
			anyhow::ensure!(
				is_world_dir(&world_path).await,
				"World directory is invalid"
			);

			let unix_secs = std::time::SystemTime::now()
				.duration_since(std::time::UNIX_EPOCH)
				.unwrap_or_default()
				.as_secs();
			let dest = get_backups_dir(dir).join(format!("{}-{}", params.world_name, unix_secs));
			copy_dir_recursive(&world_path, &dest)
				.await
				.context("Failed to copy world to backup")?;

			Ok(())
		})
	}
);

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct DeleteBackupParams {
	pub instance_id: String,
	pub backup_name: String,
}

#[rustfmt::skip]
simple_mutation!(
	name = DeleteServerBackup,
	ok = (),
	err = anyhow::Error,
	keys = DeleteBackupParams,
	fn run(&self, keys: &Self::Keys) -> impl Future<Output = Result<Self::Ok, Self::Err>> {
		let back_state = self.back_state.clone();
		let params = keys.clone();

		query_spawn(back_state.0.clone(), async move {
			let config = back_state.config().await?;
			let instance = config
				.instances
				.get(&InstanceID::from(params.instance_id.clone()))
				.context("Instance not found")?;
			let dir = instance.dir().context("Instance directory is missing")?;

			// Protect against path traversal in the backup name
			anyhow::ensure!(
				params.backup_name == Path::new(&params.backup_name).file_name().and_then(|x| x.to_str()).unwrap_or_default(),
				"Invalid backup name"
			);

			let path = get_backups_dir(dir).join(&params.backup_name);
			anyhow::ensure!(
				is_within_dir(&path, &get_backups_dir(dir)),
				"Invalid backup path"
			);

			tokio::fs::remove_dir_all(&path)
				.await
				.context("Failed to delete backup")?;

			Ok(())
		})
	}
);

/// Checks that a path is directly contained within the given parent directory
fn is_within_dir(path: &Path, parent: &Path) -> bool {
	path.parent()
		.and_then(|x| x.canonicalize().ok())
		.zip(parent.canonicalize().ok())
		.is_some_and(|(a, b)| a == b)
}
