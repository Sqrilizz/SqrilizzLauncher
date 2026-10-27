use std::collections::HashMap;
use std::path::{Path, PathBuf};

use sqrilizz_shared::Side;
use sqrilizz_shared::java_args::MemoryNum;
use sqrilizz_shared::loaders::Loader;
use sqrilizz_shared::pkg::{PackageOverrides, PackageStability};
use sqrilizz_shared::util::{DefaultExt, DeserListOrSingle, merge_options};
use sqrilizz_shared::versions::{MinecraftVersionDeser, VersionPattern, parse_versioned_string};
#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::package::PackageConfigDeser;

/// Configuration for an instance
#[derive(Deserialize, Serialize, Clone, Debug, Default, PartialEq)]
#[serde(default)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct InstanceConfig {
	// General config
	/// One or more templates to use
	#[serde(skip_serializing_if = "DeserListOrSingle::is_empty")]
	pub from: DeserListOrSingle<String>,
	/// The type or side of this instance
	#[serde(rename = "type")]
	pub side: Option<Side>,
	/// The display name of this instance
	#[serde(skip_serializing_if = "Option::is_none")]
	pub name: Option<String>,
	/// A path to an icon file for this instance
	#[serde(skip_serializing_if = "Option::is_none")]
	pub icon: Option<String>,
	/// The Minecraft version
	#[serde(skip_serializing_if = "Option::is_none")]
	pub version: Option<MinecraftVersionDeser>,
	/// Configured loader
	#[serde(skip_serializing_if = "DefaultExt::is_default")]
	pub loader: Option<String>,

	// In-depth config
	/// Launch configuration
	#[serde(skip_serializing_if = "DefaultExt::is_default")]
	pub launch: LaunchConfig,
	/// Window configuration
	#[serde(skip_serializing_if = "DefaultExt::is_default")]
	pub window: ClientWindowConfig,

	// Package config
	/// Modpack package for this instance
	#[serde(skip_serializing_if = "Option::is_none")]
	pub modpack: Option<String>,
	/// Packages for this instance
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub packages: Vec<PackageConfigDeser>,
	/// The folder for global datapacks to be installed to
	#[serde(skip_serializing_if = "Option::is_none")]
	pub datapack_folder: Option<String>,
	/// Default stability setting of packages on this instance
	#[serde(skip_serializing_if = "DefaultExt::is_default")]
	pub package_stability: Option<PackageStability>,
	/// Overrides for packages on this instance
	#[serde(skip_serializing_if = "DefaultExt::is_default")]
	pub overrides: PackageOverrides,

	// Extended config
	/// Override for the game file directory for this instance
	#[serde(skip_serializing_if = "Option::is_none")]
	#[serde(alias = "game_dir")]
	pub dir: Option<String>,
	/// Whether this instance was imported
	#[serde(skip_serializing_if = "DefaultExt::is_default")]
	pub imported: bool,
}

impl InstanceConfig {
	/// Merge this config with another one, with right side taking precendence
	pub fn merge(&mut self, other: Self) {
		self.from.merge(other.from);
		if other.name.is_some() {
			self.name = other.name;
		}
		self.version = other.version.or(self.version.clone());
		self.loader = other.loader.or(self.loader.clone());
		self.package_stability = other.package_stability.or(self.package_stability);
		self.launch.merge(other.launch);
		self.datapack_folder = other.datapack_folder.or(self.datapack_folder.clone());
		self.packages.extend(other.packages);
		self.overrides.merge(other.overrides);
		self.icon = other.icon.or(self.icon.clone());
		self.side = other.side.or(self.side);
		self.window.merge(other.window);

		// These properties are not derived and instead just overrided
		self.dir = other.dir;
		self.imported = other.imported;
	}

	/// Gets the directory for this instance
	pub fn get_dir(&self, id: &str, instances_dir: &Path) -> Option<PathBuf> {
		if let Some(dir) = &self.dir {
			if dir == "none" {
				None
			} else {
				Some(PathBuf::from(dir))
			}
		} else {
			let base = instances_dir.join(id);
			match self.side {
				None => None,
				Some(Side::Client) => Some(base.join(".minecraft")),
				Some(Side::Server) => Some(base),
			}
		}
	}
}

/// Different representations for JVM / game arguments
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(untagged)]
pub enum Args {
	/// A list of separate arguments
	List(Vec<String>),
	/// A single string of arguments
	String(String),
}

impl Args {
	/// Parse the arguments into a vector
	pub fn parse(&self) -> Vec<String> {
		match self {
			Self::List(vec) => vec.clone(),
			Self::String(string) => string.split(' ').map(|string| string.to_string()).collect(),
		}
	}

	/// Merge Args
	pub fn merge(&mut self, other: Self) {
		let mut out = self.parse();
		out.extend(other.parse());
		*self = Self::List(out);
	}
}

impl Default for Args {
	fn default() -> Self {
		Self::List(Vec::new())
	}
}

/// Arguments for the process when launching
#[derive(Deserialize, Serialize, Debug, Default, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct LaunchArgs {
	/// Arguments for the JVM
	#[serde(default)]
	#[serde(skip_serializing_if = "DefaultExt::is_default")]
	pub jvm: Args,
	/// Arguments for the game
	#[serde(default)]
	#[serde(skip_serializing_if = "DefaultExt::is_default")]
	pub game: Args,
}

/// Different representations of both memory arguments for the JVM
#[derive(Deserialize, Serialize, Debug, Default, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(untagged)]
pub enum LaunchMemory {
	/// No memory arguments
	#[default]
	None,
	/// A single memory argument shared for both
	Single(String),
	/// Different memory arguments for both
	Both {
		/// The minimum memory
		min: String,
		/// The maximum memory
		max: String,
	},
}

impl LaunchMemory {
	/// Parse this memory as a minimum and maximum memory
	pub fn to_min_max(self) -> (Option<MemoryNum>, Option<MemoryNum>) {
		let min_mem = match &self {
			LaunchMemory::None => None,
			LaunchMemory::Single(string) => MemoryNum::parse(string),
			LaunchMemory::Both { min, .. } => MemoryNum::parse(min),
		};
		let max_mem = match &self {
			LaunchMemory::None => None,
			LaunchMemory::Single(string) => MemoryNum::parse(string),
			LaunchMemory::Both { max, .. } => MemoryNum::parse(max),
		};

		(min_mem, max_mem)
	}
}

/// Options for the Minecraft QuickPlay feature
#[derive(Deserialize, Serialize, Debug, PartialEq, Default, Clone)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum QuickPlay {
	/// QuickPlay a world
	World {
		/// The world to play
		world: String,
	},
	/// QuickPlay a server
	Server {
		/// The server address to join
		server: String,
		/// The port for the server to connect to
		port: Option<u16>,
	},
	/// QuickPlay a realm
	Realm {
		/// The realm name to join
		realm: String,
	},
	/// Don't do any QuickPlay
	#[default]
	None,
}

/// Configuration for the launching of the game
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct LaunchConfig {
	/// The arguments for the process
	#[serde(default)]
	#[serde(skip_serializing_if = "DefaultExt::is_default")]
	pub args: LaunchArgs,
	/// JVM memory options
	#[serde(default)]
	#[serde(skip_serializing_if = "DefaultExt::is_default")]
	pub memory: LaunchMemory,
	/// The java installation to use
	#[serde(default)]
	pub java: Option<String>,
	/// Environment variables
	#[serde(default)]
	#[serde(skip_serializing_if = "HashMap::is_empty")]
	pub env: HashMap<String, String>,
	/// A wrapper command
	#[serde(default)]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub wrapper: Option<WrapperCommand>,
	/// QuickPlay options
	#[serde(default)]
	#[serde(skip_serializing_if = "DefaultExt::is_default")]
	pub quick_play: QuickPlay,
	/// Whether or not to use the Log4J configuration
	#[serde(default)]
	#[serde(skip_serializing_if = "DefaultExt::is_default")]
	pub use_log4j_config: bool,
}

impl LaunchConfig {
	/// Merge multiple LaunchConfigs
	pub fn merge(&mut self, other: Self) -> &mut Self {
		self.args.jvm.merge(other.args.jvm);
		self.args.game.merge(other.args.game);
		if !matches!(other.memory, LaunchMemory::None) {
			self.memory = other.memory;
		}
		if other.java.is_some() {
			self.java = other.java;
		}
		self.env.extend(other.env);
		if other.wrapper.is_some() {
			self.wrapper = other.wrapper;
		}
		if !matches!(other.quick_play, QuickPlay::None) {
			self.quick_play = other.quick_play;
		}

		self
	}
}

impl Default for LaunchConfig {
	fn default() -> Self {
		Self {
			args: LaunchArgs {
				jvm: Args::default(),
				game: Args::default(),
			},
			memory: LaunchMemory::default(),
			java: None,
			env: HashMap::new(),
			wrapper: None,
			quick_play: QuickPlay::default(),
			use_log4j_config: false,
		}
	}
}

/// A wrapper command
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct WrapperCommand {
	/// The command to run
	pub cmd: String,
	/// The command's arguments
	#[serde(default)]
	pub args: Vec<String>,
}

/// Resolution for a client window
#[derive(Deserialize, Serialize, Clone, Debug, Copy, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct WindowResolution {
	/// The width of the window
	pub width: u32,
	/// The height of the window
	pub height: u32,
}

/// Configuration for the client window
#[derive(Deserialize, Serialize, Default, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(default)]
pub struct ClientWindowConfig {
	/// The resolution of the window
	#[serde(skip_serializing_if = "Option::is_none")]
	pub resolution: Option<WindowResolution>,
}

impl ClientWindowConfig {
	/// Merge two ClientWindowConfigs
	pub fn merge(&mut self, other: Self) -> &mut Self {
		self.resolution = merge_options(self.resolution, other.resolution);
		self
	}
}

/// Checks if an instance ID is valid
pub fn is_valid_instance_id(id: &str) -> bool {
	for c in id.chars() {
		if !c.is_ascii() {
			return false;
		}

		if c.is_ascii_punctuation() {
			match c {
				'_' | '-' | '.' | ':' => {}
				_ => return false,
			}
		}

		if c.is_ascii_whitespace() {
			return false;
		}
	}

	true
}

/// Converts a string into a valid instance ID
/// Special characters will be converted into hyphens
pub fn make_valid_instance_id(string: &str) -> String {
	let string = string.to_lowercase();
	string
		.chars()
		.map(|c| {
			if !c.is_ascii_alphanumeric() && c != '.' && c != ':' {
				'-'
			} else {
				c
			}
		})
		.collect()
}

/// Parses a loader from configuration
pub fn parse_loader_config(loader: &str) -> (Loader, VersionPattern) {
	let (loader, version) = parse_versioned_string(loader);
	(Loader::parse_from_str(loader), version)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_quickplay_deser() {
		#[derive(Deserialize)]
		struct Test {
			quick_play: QuickPlay,
		}

		let test = serde_json::from_str::<Test>(
			r#"{
			"quick_play": {
				"type": "server",
				"server": "localhost",
				"port": 25565,
				"world": "test",
				"realm": "my_realm"
			}	
		}"#,
		)
		.unwrap();
		assert_eq!(
			test.quick_play,
			QuickPlay::Server {
				server: "localhost".into(),
				port: Some(25565)
			}
		);
	}
}
