#![warn(missing_docs)]

//! This is the library for Sqrilizz Launcher and pretty much all of the features that the
//! CLI uses.
//!
//! Note: The asynchronous functions in this library expect the use of the Tokio runtime and may panic
//! if it is not used
//!
//! # Features
//!
//! - `builder`: Enable or disable the config builder system, which isn't needed if you are just deserializing the standard config.
//! - `disable_instance_update_packages`: A workaround for `https://github.com/rust-lang/rust/issues/102211`. If you are
//!   getting higher-ranked lifetime errors when running the update_templates function, try enabling this. When enabled, the
//!   update_templates function will no longer update packages at all.
//! - `schema`: Enable generation of JSON schemas using the `schemars` crate

pub use sqrilizz_config as config_crate;
pub use sqrilizz_core as core;
pub use sqrilizz_instance as instance_crate;
pub use sqrilizz_net as net_crate;
pub use sqrilizz_parse as parse;
pub use sqrilizz_pkg as pkg_crate;
pub use sqrilizz_shared as shared;

/// Installable addons
pub mod addon;
/// Sqrilizz Launcher configuration
pub mod config;
/// Launchable instances
pub mod instance;
/// File and data format input / output
pub mod io;
/// Dealing with packages
pub mod pkg;
/// Configuration templates for instances
pub mod template;
/// Common utilities that can't live anywhere else
pub mod util;

/// The version of Sqrilizz Launcher
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
