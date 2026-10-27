use std::fmt::Display;

#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A Minecraft loader (mod loader or server software)
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default, Hash)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum Loader {
	/// The vanilla Minecraft client/server
	#[default]
	Vanilla,
	/// The Forge mod loader
	Forge,
	/// The NeoForge mod loader
	#[serde(alias = "neoforge")]
	NeoForged,
	/// The Fabric mod loader
	Fabric,
	/// The Quilt mod loader
	Quilt,
	/// The LiteLoader mod loader
	LiteLoader,
	/// The Risugamis mod loader
	Risugamis,
	/// The Rift mod loader
	Rift,
	/// The Paper server software
	Paper,
	/// The Sponge server software
	Sponge,
	/// The SpongeForge server software
	SpongeForge,
	/// The CraftBukkit server software
	CraftBukkit,
	/// The Spigot server software
	Spigot,
	/// The Glowstone server software
	Glowstone,
	/// The Pufferfish server software
	Pufferfish,
	/// The Purpur server software
	Purpur,
	/// The Folia server software
	Folia,
	/// Any loader, used for wildcard matches
	Any,
	/// A loader that is not known by the launcher
	#[cfg_attr(not(feature = "schema"), serde(untagged))]
	Unknown(String),
}

impl Loader {
	/// Parses a loader from a string
	pub fn parse_from_str(string: &str) -> Self {
		let name = string.split(':').next().unwrap_or(string).trim();
		match name {
			"vanilla" => Self::Vanilla,
			"forge" => Self::Forge,
			"neoforged" | "neoforge" => Self::NeoForged,
			"fabric" => Self::Fabric,
			"quilt" => Self::Quilt,
			"liteloader" => Self::LiteLoader,
			"risugamis" => Self::Risugamis,
			"rift" => Self::Rift,
			"paper" => Self::Paper,
			"sponge" => Self::Sponge,
			"craftbukkit" => Self::CraftBukkit,
			"spigot" => Self::Spigot,
			"glowstone" => Self::Glowstone,
			"pufferfish" => Self::Pufferfish,
			"purpur" => Self::Purpur,
			"folia" => Self::Folia,
			"any" => Self::Any,
			other => Self::Unknown(other.to_string()),
		}
	}

	/// Whether this loader is used for client instances
	pub fn is_client(&self) -> bool {
		match self {
			Self::Vanilla
			| Self::Forge
			| Self::NeoForged
			| Self::Fabric
			| Self::Quilt
			| Self::LiteLoader
			| Self::Risugamis
			| Self::Rift
			| Self::Any
			| Self::Unknown(..) => true,
			_ => false,
		}
	}

	/// Whether this loader is used for server instances
	pub fn is_server(&self) -> bool {
		match self {
			Self::Vanilla
			| Self::Fabric
			| Self::Quilt
			| Self::Paper
			| Self::Sponge
			| Self::SpongeForge
			| Self::CraftBukkit
			| Self::Spigot
			| Self::Glowstone
			| Self::Pufferfish
			| Self::Purpur
			| Self::Folia
			| Self::Any
			| Self::Unknown(..) => true,
			_ => false,
		}
	}
}

impl Display for Loader {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Vanilla => write!(f, "Vanilla"),
			Self::Forge => write!(f, "Forge"),
			Self::NeoForged => write!(f, "NeoForged"),
			Self::Fabric => write!(f, "Fabric"),
			Self::Quilt => write!(f, "Quilt"),
			Self::LiteLoader => write!(f, "LiteLoader"),
			Self::Risugamis => write!(f, "Risugami's"),
			Self::Rift => write!(f, "Rift"),
			Self::Paper => write!(f, "Paper"),
			Self::Sponge => write!(f, "Sponge"),
			Self::SpongeForge => write!(f, "SpongeForge"),
			Self::CraftBukkit => write!(f, "CraftBukkit"),
			Self::Spigot => write!(f, "Spigot"),
			Self::Glowstone => write!(f, "Glowstone"),
			Self::Pufferfish => write!(f, "Pufferfish"),
			Self::Purpur => write!(f, "Purpur"),
			Self::Folia => write!(f, "Folia"),
			Self::Any => write!(f, "Any"),
			Self::Unknown(other) => write!(f, "{other}"),
		}
	}
}

/// A match for one or more loaders
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Hash)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum LoaderMatch {
	/// Any Fabric-like loader (Fabric, Quilt)
	FabricLike,
	/// Any Forge-like loader (Forge, SpongeForge)
	ForgeLike,
	/// Any Bukkit-like server software
	Bukkit,
	/// A specific loader
	#[serde(untagged)]
	Loader(Loader),
}

impl LoaderMatch {
	/// Parses a loader match from a string
	pub fn parse_from_str(string: &str) -> Self {
		match string {
			"fabriclike" => Self::FabricLike,
			"forgelike" => Self::ForgeLike,
			"bukkit" => Self::Bukkit,
			other => LoaderMatch::Loader(Loader::parse_from_str(other)),
		}
	}

	/// Checks if the given loader matches this loader match
	pub fn matches(&self, other: &Loader) -> bool {
		if other == &Loader::Any {
			true
		} else if let Loader::Unknown(other_str) = other {
			let parsed = Loader::parse_from_str(other_str);
			if &parsed != other {
				self.matches(&parsed)
			} else {
				self.get_matches().contains(other)
			}
		} else {
			self.get_matches().contains(other)
		}
	}

	/// Returns all loaders matched by this loader match
	pub fn get_matches(&self) -> Vec<Loader> {
		match self {
			Self::FabricLike => vec![Loader::Fabric, Loader::Quilt],
			Self::ForgeLike => vec![Loader::Forge, Loader::SpongeForge],
			Self::Bukkit => vec![
				Loader::Paper,
				Loader::CraftBukkit,
				Loader::Spigot,
				Loader::Glowstone,
				Loader::Pufferfish,
				Loader::Purpur,
			],
			Self::Loader(loader) => vec![loader.clone()],
		}
	}
}

impl Display for LoaderMatch {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::FabricLike => write!(f, "Fabric-like"),
			Self::ForgeLike => write!(f, "Forge-like"),
			Self::Bukkit => write!(f, "Bukkit"),
			Self::Loader(loader) => loader.fmt(f),
		}
	}
}

/// A proxy for servers
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum Proxy {
	/// The BungeeCord proxy
	BungeeCord,
	/// The Waterfall proxy
	Waterfall,
	/// The Velocity proxy
	Velocity,
	/// A proxy that is not known by the launcher
	#[cfg_attr(not(feature = "schema"), serde(untagged))]
	Unknown(String),
}

impl Proxy {
	/// Parses a proxy from a string
	pub fn parse_from_str(string: &str) -> Self {
		match string {
			"bungeecord" => Self::BungeeCord,
			"waterfall" => Self::Waterfall,
			"velocity" => Self::Velocity,
			other => Self::Unknown(other.into()),
		}
	}
}

impl Display for Proxy {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::BungeeCord => write!(f, "BungeeCord"),
			Self::Waterfall => write!(f, "Waterfall"),
			Self::Velocity => write!(f, "Velocity"),
			Self::Unknown(other) => write!(f, "{other}"),
		}
	}
}

/// A match for one or more proxies
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum ProxyMatch {
	/// Any BungeeCord-like proxy (BungeeCord, Waterfall)
	BungeeCordLike,
	/// A specific proxy
	#[serde(untagged)]
	Proxy(Proxy),
}

impl ProxyMatch {
	/// Parses a proxy match from a string
	pub fn parse_from_str(string: &str) -> Self {
		match string {
			"bungeecordlike" => Self::BungeeCordLike,
			other => Self::Proxy(Proxy::parse_from_str(other)),
		}
	}

	/// Checks if the given proxy matches this proxy match
	pub fn matches(&self, other: &Proxy) -> bool {
		match self {
			Self::BungeeCordLike => matches!(other, Proxy::BungeeCord | Proxy::Waterfall),
			Self::Proxy(proxy) => proxy == other,
		}
	}
}
