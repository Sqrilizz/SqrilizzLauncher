

use std::path::{Path, PathBuf};

use sqrilizz_shared::Side;

pub mod addon;
pub mod files;
pub mod lock;
pub mod logs;

pub fn get_instance_dir(base_dir: &Path, side: Side) -> PathBuf {
	match side {
		Side::Client => base_dir.join(".minecraft"),
		Side::Server => base_dir.to_owned(),
	}
}
