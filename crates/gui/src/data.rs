use std::collections::HashSet;
use std::{collections::HashMap, path::PathBuf};

use sqrilizz::config_crate::ConfigKind;
use sqrilizz::core::io::{json_from_file, json_to_file};
use sqrilizz::io::paths::Paths;
use serde::{Deserialize, Serialize};

use crate::output::SerializableResolutionError;

#[derive(Serialize, Deserialize, Clone, PartialEq)]
#[serde(default)]
pub struct LauncherData {
	pub launcher_opened_before: bool,
	pub saved_instance_icons: Vec<InstanceIcon>,
	#[serde(alias = "pinned")]
	pub pinned_instances: HashSet<String>,
	#[serde(alias = "current_user")]
	pub current_account: Option<String>,
	pub last_repository: Option<String>,
	pub last_resolution_errors: HashMap<String, SerializableResolutionError>,
	pub last_added_package: Option<(String, ConfigKind)>,
	pub last_opened_instance: Option<(String, ConfigKind)>,
	#[serde(alias = "theme")]
	pub base_theme: Option<String>,
	pub overlay_themes: Vec<String>,
	pub zoom: f64,
	#[serde(default = "default_true")]
	pub discord_rpc: bool,
	#[serde(default)]
	pub playtimes: HashMap<String, u64>,
	#[serde(default)]
	pub play_launches: HashMap<String, u64>,
	#[serde(default)]
	pub last_played: HashMap<String, u64>,
}

impl Default for LauncherData {
	fn default() -> Self {
		Self {
			launcher_opened_before: false,
			saved_instance_icons: Vec::new(),
			pinned_instances: HashSet::new(),
			current_account: None,
			last_repository: None,
			last_resolution_errors: HashMap::new(),
			last_added_package: None,
			last_opened_instance: None,
			base_theme: None,
			overlay_themes: Vec::new(),
			zoom: 1.0,
			discord_rpc: true,
			playtimes: HashMap::new(),
			play_launches: HashMap::new(),
			last_played: HashMap::new(),
		}
	}

}

fn default_true() -> bool {
	true
}

impl LauncherData {
	pub fn open(paths: &Paths) -> anyhow::Result<Self> {
		let path = Self::path(paths);
		if path.exists() {
			let mut data: Self = json_from_file(path)?;
			data.zoom = 1.0;
			Ok(data)
		} else {
			Ok(Self::default())
		}
	}

	pub fn write(&self, paths: &Paths) -> anyhow::Result<()> {
		json_to_file(Self::path(paths), &self)
	}

	pub fn path(paths: &Paths) -> PathBuf {
		paths.internal.join("launcher_data.json")
	}

	pub fn get_playtime(&self, id: &str) -> u64 {
		self.playtimes.get(id).copied().unwrap_or(0)
	}

	pub fn get_launches(&self, id: &str) -> u64 {
		self.play_launches.get(id).copied().unwrap_or(0)
	}

	pub fn get_last_played(&self, id: &str) -> Option<u64> {
		self.last_played.get(id).copied()
	}

	pub fn total_playtime(&self) -> u64 {
		self.playtimes.values().sum()
	}

	pub fn total_launches(&self) -> u64 {
		self.play_launches.values().sum()
	}
}

pub fn format_playtime(seconds: u64) -> String {
	if seconds == 0 {
		return "0 мин".to_string();
	}
	let mins = seconds / 60;
	if mins < 1 {
		return "< 1 мин".to_string();
	}
	let hours = mins / 60;
	let rem_mins = mins % 60;
	if hours == 0 {
		format!("{mins} мин")
	} else if rem_mins == 0 {
		format!("{hours} ч")
	} else {
		format!("{hours} ч {rem_mins} мин")
	}
}

pub fn format_last_played(unix_secs: u64) -> String {
	if unix_secs == 0 {
		return "Никогда".to_string();
	}
	let now = std::time::SystemTime::now()
		.duration_since(std::time::UNIX_EPOCH)
		.unwrap_or_default()
		.as_secs();
	if now >= unix_secs {
		let diff = now - unix_secs;
		if diff < 60 {
			return "Только что".to_string();
		}
		let mins = diff / 60;
		if mins < 60 {
			return format!("{mins} мин назад");
		}
		let hours = mins / 60;
		if hours < 24 {
			return format!("{hours} ч назад");
		}
		let days = hours / 24;
		if days == 1 {
			return "Вчера".to_string();
		}
		if days < 30 {
			return format!("{days} дн. назад");
		}
	}
	"Давно".to_string()
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
#[serde(untagged)]
pub enum InstanceIcon {
	File(PathBuf),
}
