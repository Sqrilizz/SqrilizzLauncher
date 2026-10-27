use std::collections::HashMap;
use std::fs;

use sqrilizz_shared::skip_none;
use sqrilizz_shared::versions::VersionPattern;

use crate::launch::LaunchParameters;
use crate::launch::process::LaunchProcessProperties;
use crate::net::game_files::client_meta::args::Arguments;

pub use self::args::create_quick_play_args;

mod args;

/// Get the launch properties for launching a client instance
pub async fn get_launch_props(
	params: &LaunchParameters<'_>,
) -> anyhow::Result<LaunchProcessProperties> {
	let mut jvm_args = Vec::new();
	let mut game_args = Vec::new();

	match &params.client_meta.arguments {
		Arguments::New(args) => {
			for arg in &args.jvm {
				jvm_args.extend(args::process_arg(arg, params));
			}
			for arg in &args.game {
				game_args.extend(args::process_arg(arg, params));
			}
		}
		Arguments::Old(args) => {
			jvm_args.push(format!(
				"-Djava.library.path={}",
				params
					.paths
					.internal
					.join("versions")
					.join(params.version.to_string())
					.join("natives")
					.to_string_lossy()
			));
			jvm_args.push("-cp".into());
			jvm_args.push(params.classpath.get_str());

			for arg in args.split(' ') {
				game_args.push(skip_none!(args::replace_arg_placeholders(arg, params)));
			}
		}
	}

	let env_vars =
		get_additional_environment_variables(params.version, &params.version_manifest.list);

	let props = LaunchProcessProperties {
		jvm_args,
		game_args,
		additional_env_vars: env_vars,
	};
	Ok(props)
}

/// Detects discrete GPU availability on Linux and returns appropriate offload variables
#[cfg(target_os = "linux")]
fn detect_discrete_gpu_env() -> HashMap<String, String> {
	let mut envs = HashMap::new();
	let mut has_nvidia = false;
	let mut has_amd = false;
	let mut gpu_count = 0;

	if let Ok(entries) = fs::read_dir("/sys/bus/pci/devices") {
		for entry in entries.flatten() {
			let path = entry.path();
			let class_path = path.join("class");
			if let Ok(class) = fs::read_to_string(&class_path) {
				let class = class.trim();
				// 0x030000 = VGA controller, 0x030200 = 3D controller
				if class.starts_with("0x0300") || class.starts_with("0x0302") {
					gpu_count += 1;
					let vendor_path = path.join("vendor");
					if let Ok(vendor) = fs::read_to_string(&vendor_path) {
						let vendor = vendor.trim().to_lowercase();
						if vendor == "0x10de" {
							has_nvidia = true;
						} else if vendor == "0x1002" {
							has_amd = true;
						}
					}
				}
			}
		}
	}

	// If there is an NVIDIA discrete GPU (e.g. GeForce RTX Mobile or desktop hybrid)
	if has_nvidia {
		envs.insert("__NV_PRIME_RENDER_OFFLOAD".to_string(), "1".to_string());
		envs.insert(
			"__VK_LAYER_NV_optimus".to_string(),
			"NVIDIA_only".to_string(),
		);
		envs.insert(
			"__GLX_VENDOR_LIBRARY_NAME".to_string(),
			"nvidia".to_string(),
		);
		envs.insert(
			"VK_LOADER_DRIVERS_SELECT".to_string(),
			"*nvidia*".to_string(),
		);
	} else if has_amd && gpu_count > 1 {
		// Dual GPU setup with AMD discrete GPU
		envs.insert("DRI_PRIME".to_string(), "1".to_string());
	}

	envs
}

/// Get additional environment variables for the client
fn get_additional_environment_variables(
	version: &str,
	version_list: &[String],
) -> HashMap<String, String> {
	#[cfg(not(target_os = "linux"))]
	{
		let _ = version;
		let _ = version_list;
	}

	#[cfg(target_os = "linux")]
	let mut env_vars = detect_discrete_gpu_env();
	#[cfg(not(target_os = "linux"))]
	let env_vars = HashMap::new();

	// Compatibility env var for old versions on Linux to prevent graphical issues
	#[cfg(target_os = "linux")]
	{
		if VersionPattern::Before("1.8.9".into()).matches_single(version, version_list) {
			env_vars.insert("__GL_THREADED_OPTIMIZATIONS".to_string(), "0".to_string());
		}
	}

	env_vars
}
