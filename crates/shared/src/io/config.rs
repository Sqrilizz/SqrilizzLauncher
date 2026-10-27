use std::{collections::HashMap, fs::File, sync::LazyLock};

use crate::io::home_dir;

/// Global IO configuration
pub static IO_CONFIG: LazyLock<IoConfig> = LazyLock::new(|| {
	IoConfig::new().unwrap_or_else(|e| {
		eprintln!("Failed to load IO config from file: {e:?}");
		IoConfig::default()
	})
});

/// Global IO configuration
#[derive(Default)]
pub struct IoConfig {
	/// Configuration in key-value form from the file
	file_values: HashMap<String, serde_json::Value>,
}

/// Gets an environment variable for the given config key, preferring the
/// `SQRILIZZ_` prefix.
fn env_value(key: &str) -> Option<String> {
	let key = key.to_ascii_uppercase();
	std::env::var(format!("SQRILIZZ_{}", key))
		
		.ok()
}

impl IoConfig {
	fn new() -> anyhow::Result<Self> {
		let home = home_dir()?;
		let path = home.join(".sqrilizz.json");
		let path = if path.exists() {
			path
		} else {
			return Ok(Self::default());
		};

		let file = File::open(path)?;
		let data: HashMap<String, serde_json::Value> = serde_json::from_reader(file)?;

		Ok(Self { file_values: data })
	}

	/// Gets the value of a string property from either the file or environment. The key should be lower_case.
	pub fn get_string(&self, key: &str) -> Option<String> {
		if let Some(value) = env_value(key) {
			Some(value)
		} else {
			self.file_values
				.get(key)
				.and_then(|x| x.as_str())
				.map(|x| x.to_string())
		}
	}

	/// Gets the value of a boolean property from either the file or environment. The key should be lower_case.
	pub fn get_bool(&self, key: &str) -> Option<bool> {
		if let Some(value) = env_value(key) {
			Some(match value.as_str() {
				"1" => true,
				_ => false,
			})
		} else {
			self.file_values.get(key).and_then(|x| x.as_bool())
		}
	}

	/// Gets the value of an integer property from either the file or environment. The key should be lower_case.
	pub fn get_int(&self, key: &str) -> Option<i64> {
		if let Some(value) = env_value(key) {
			value.parse().ok()
		} else {
			self.file_values.get(key).and_then(|x| x.as_i64())
		}
	}
}
