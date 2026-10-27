use std::{
	path::PathBuf,
	sync::{Arc, LazyLock},
};

use anyhow::Context;
use reqwest::Client;
use serde::Deserialize;
use tokio::sync::{RwLock, broadcast};

#[derive(Clone, Debug, PartialEq)]
pub struct UpdateInfo {
	pub version: String,
	pub tag: String,
	pub release_name: String,
	pub body: String,
	pub download_url: String,
	pub asset_name: String,
	pub size_bytes: Option<u64>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum UpdateStatus {
	Idle,
	Checking,
	Available(UpdateInfo),
	Downloading {
		version: String,
		progress: f32, // 0.0 .. 1.0
		downloaded: u64,
		total: u64,
	},
	ReadyToRestart {
		version: String,
	},
	Error(String),
	Dismissed,
}

static CURRENT_STATUS: LazyLock<Arc<RwLock<UpdateStatus>>> =
	LazyLock::new(|| Arc::new(RwLock::new(UpdateStatus::Idle)));

static STATUS_NOTIFY: LazyLock<broadcast::Sender<UpdateStatus>> = LazyLock::new(|| {
	let (tx, _) = broadcast::channel(32);
	tx
});

pub async fn get_status() -> UpdateStatus {
	CURRENT_STATUS.read().await.clone()
}

pub fn subscribe_status() -> broadcast::Receiver<UpdateStatus> {
	STATUS_NOTIFY.subscribe()
}

pub async fn set_status(status: UpdateStatus) {
	*CURRENT_STATUS.write().await = status.clone();
	let _ = STATUS_NOTIFY.send(status);
}

#[derive(Deserialize)]
struct GhRelease {
	tag_name: String,
	name: Option<String>,
	body: Option<String>,
	assets: Vec<GhAsset>,
}

#[derive(Deserialize)]
struct GhAsset {
	name: String,
	browser_download_url: String,
	size: Option<u64>,
}

/// Checks GitHub releases to determine if a newer version is available.
pub async fn check_for_updates() -> anyhow::Result<Option<UpdateInfo>> {
	set_status(UpdateStatus::Checking).await;

	let current_version = env!("CARGO_PKG_VERSION");
	let client = Client::builder()
		.user_agent("SqrilizzLauncher")
		.build()
		.context("Failed to build HTTP client")?;

	let url = "https://api.github.com/repos/Sqrilizz/SqrilizzLauncher/releases?per_page=5";
	let resp = match client.get(url).send().await {
		Ok(r) if r.status().is_success() => r,
		Ok(r) => {
			let err = format!("Сервер обновлений вернул статус {}", r.status());
			set_status(UpdateStatus::Error(err.clone())).await;
			anyhow::bail!("{err}");
		}
		Err(e) => {
			let err = format!("Ошибка подключения к серверу: {e}");
			set_status(UpdateStatus::Error(err.clone())).await;
			anyhow::bail!("{err}");
		}
	};

	let releases: Vec<GhRelease> = resp.json().await.context("Failed to parse release JSON")?;
	let release = releases.into_iter().next().context("No releases found")?;
	let tag = release.tag_name.clone();
	let latest_ver = tag.trim_start_matches('v').to_string();

	let is_newer = match version_compare::compare(&latest_ver, current_version) {
		Ok(cmp) => cmp == version_compare::Cmp::Gt,
		Err(_) => latest_ver != current_version,
	};

	if !is_newer {
		set_status(UpdateStatus::Idle).await;
		return Ok(None);
	}

	// Pick appropriate asset for current OS
	let mut chosen_asset = None;

	for asset in &release.assets {
		let name_lower = asset.name.to_lowercase();
		#[cfg(target_os = "linux")]
		{
			if name_lower.ends_with(".appimage") {
				chosen_asset = Some(asset);
				break;
			}
		}
		#[cfg(target_os = "windows")]
		{
			if name_lower.ends_with(".exe") || name_lower.ends_with(".msi") {
				chosen_asset = Some(asset);
				break;
			}
		}
		#[cfg(target_os = "macos")]
		{
			if name_lower.ends_with(".dmg") {
				chosen_asset = Some(asset);
				break;
			}
		}
	}

	// If no strict extension match, fallback to any matching asset
	if chosen_asset.is_none() {
		for asset in &release.assets {
			let name_lower = asset.name.to_lowercase();
			#[cfg(target_os = "linux")]
			if name_lower.contains("linux") || name_lower.contains("x86_64") {
				chosen_asset = Some(asset);
				break;
			}
			#[cfg(target_os = "windows")]
			if name_lower.contains("windows") || name_lower.contains("x64") {
				chosen_asset = Some(asset);
				break;
			}
			#[cfg(target_os = "macos")]
			if name_lower.contains("macos") || name_lower.contains("darwin") {
				chosen_asset = Some(asset);
				break;
			}
		}
	}

	let Some(asset) = chosen_asset else {
		let err = format!("Для версии {tag} нет файла под вашу систему");
		set_status(UpdateStatus::Error(err.clone())).await;
		anyhow::bail!("{err}");
	};

	let info = UpdateInfo {
		version: latest_ver.clone(),
		tag,
		release_name: release
			.name
			.unwrap_or_else(|| format!("Sqrilizz v{latest_ver}")),
		body: release.body.unwrap_or_default(),
		download_url: asset.browser_download_url.clone(),
		asset_name: asset.name.clone(),
		size_bytes: asset.size,
	};

	set_status(UpdateStatus::Available(info.clone())).await;
	Ok(Some(info))
}

/// Downloads and installs the update in-place
pub async fn start_update(info: UpdateInfo) -> anyhow::Result<()> {
	let client = Client::builder()
		.user_agent("SqrilizzLauncher")
		.build()
		.context("Failed to build HTTP client")?;

	let mut resp = client
		.get(&info.download_url)
		.send()
		.await
		.context("Failed to initiate download")?
		.error_for_status()
		.context("Server download error")?;

	let total_bytes = resp.content_length().or(info.size_bytes).unwrap_or(0);

	// Target file to replace
	let target_path = get_target_executable_path()?;
	let temp_download_path = target_path.with_extension("sqrilizz_update_tmp");

	let mut file = tokio::fs::File::create(&temp_download_path)
		.await
		.with_context(|| {
			format!(
				"Failed to create temp file at {}",
				temp_download_path.display()
			)
		})?;

	use tokio::io::AsyncWriteExt;
	let mut downloaded_bytes: u64 = 0;

	while let Some(chunk) = resp.chunk().await.context("Error downloading chunk")? {
		file.write_all(&chunk)
			.await
			.context("Error writing to temp file")?;
		downloaded_bytes += chunk.len() as u64;

		let progress = if total_bytes > 0 {
			(downloaded_bytes as f32 / total_bytes as f32).clamp(0.0, 1.0)
		} else {
			0.0
		};

		set_status(UpdateStatus::Downloading {
			version: info.version.clone(),
			progress,
			downloaded: downloaded_bytes,
			total: total_bytes,
		})
		.await;
	}

	file.flush()
		.await
		.context("Failed to flush downloaded file")?;
	drop(file);

	// Set executable permissions on Unix
	#[cfg(unix)]
	{
		use std::os::unix::fs::PermissionsExt;
		let _ =
			std::fs::set_permissions(&temp_download_path, std::fs::Permissions::from_mode(0o755));
	}

	// In-place replacement
	replace_executable(&temp_download_path, &target_path)?;

	set_status(UpdateStatus::ReadyToRestart {
		version: info.version.clone(),
	})
	.await;

	Ok(())
}

fn get_target_executable_path() -> anyhow::Result<PathBuf> {
	#[cfg(target_os = "linux")]
	{
		if let Ok(appimage) = std::env::var("APPIMAGE") {
			return Ok(PathBuf::from(appimage));
		}
	}
	let current_exe = std::env::current_exe().context("Failed to get current executable path")?;
	Ok(current_exe)
}

fn replace_executable(temp_path: &PathBuf, target_path: &PathBuf) -> anyhow::Result<()> {
	#[cfg(unix)]
	{
		std::fs::rename(temp_path, target_path).with_context(|| {
			format!(
				"Failed to rename {} to {}",
				temp_path.display(),
				target_path.display()
			)
		})?;
	}

	#[cfg(windows)]
	{
		let old_path = target_path.with_extension("exe.old");
		let _ = std::fs::remove_file(&old_path);
		std::fs::rename(target_path, &old_path).with_context(|| {
			format!(
				"Failed to rename running executable to {}",
				old_path.display()
			)
		})?;
		std::fs::rename(temp_path, target_path).with_context(|| {
			format!(
				"Failed to install new executable to {}",
				target_path.display()
			)
		})?;
	}

	Ok(())
}

/// Restarts the application into the updated executable
pub fn restart_launcher() -> anyhow::Result<()> {
	let target = get_target_executable_path()?;
	std::process::Command::new(target)
		.spawn()
		.context("Failed to restart launcher")?;
	std::process::exit(0);
}
