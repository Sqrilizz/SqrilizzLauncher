use sqrilizz_shared::output::{MessageContents, SqrilizzOutput};
use sqrilizz_shared::util::{ARCH_STRING, OS_STRING};

use crate::account::AccountKind;
use crate::instance::{InstanceKind, WindowResolution};
use crate::io::files::paths::Paths;
use crate::launch::{LaunchParameters, QuickPlayType};
use crate::net::game_files::assets::get_virtual_dir_path;
use crate::net::game_files::client_meta::args::ArgumentItem;

#[allow(dead_code)]
pub(crate) fn fill_logging_path_arg(arg: String, version: &str, paths: &Paths) -> Option<String> {
	let path = paths
		.internal
		.join("versions")
		.join(version)
		.join("logging.xml");
	let path_str = path.to_str()?;
	Some(arg.replace("${path}", path_str))
}

pub(crate) fn process_arg(arg: &ArgumentItem, params: &LaunchParameters) -> Vec<String> {
	let mut out = Vec::new();
	let InstanceKind::Client { window } = &params.side else {
		panic!("Instance is not a client")
	};
	match arg {
		ArgumentItem::Simple(arg) => {
			let arg = process_simple_arg(arg, params);
			if let Some(arg) = arg {
				out.push(arg);
			}
		}
		ArgumentItem::Conditional(arg) => {
			for rule in &arg.rules {
				let allowed = rule.action.is_allowed();

				if let Some(os_name) = &rule.os.name
					&& allowed != (OS_STRING == os_name.to_string())
				{
					return vec![];
				}
				if let Some(os_arch) = &rule.os.arch
					&& allowed != (ARCH_STRING == os_arch.to_string())
				{
					return vec![];
				}

				if let Some(has_custom_resolution) = &rule.features.has_custom_resolution
					&& *has_custom_resolution
					&& window.resolution.is_none()
				{
					return vec![];
				}
				if let Some(is_demo_user) = &rule.features.is_demo_user
					&& *is_demo_user
				{
					let use_demo = match params.accounts.get_chosen_account() {
						Some(account) => matches!(account.kind, AccountKind::Demo),
						None => false,
					};
					if !use_demo {
						return vec![];
					}
				}
				if let Some(quick_play_support) = &rule.features.has_quick_play_support
					&& *quick_play_support
					&& matches!(params.launch_config.quick_play, QuickPlayType::None)
				{
					return vec![];
				}
				if let Some(quick_play_singleplayer) = &rule.features.is_quick_play_singleplayer
					&& *quick_play_singleplayer
					&& !matches!(params.launch_config.quick_play, QuickPlayType::World { .. })
				{
					return vec![];
				}
				if let Some(quick_play_multiplayer) = &rule.features.is_quick_play_multiplayer
					&& *quick_play_multiplayer
					&& !matches!(
						params.launch_config.quick_play,
						QuickPlayType::Server { .. }
					) {
					return vec![];
				}
				if let Some(quick_play_realms) = &rule.features.is_quick_play_realms
					&& *quick_play_realms
					&& !matches!(params.launch_config.quick_play, QuickPlayType::Realm { .. })
				{
					return vec![];
				}
			}

			for arg in arg.value.iter() {
				out.extend(process_simple_arg(arg, params));
			}
		}
	};

	out
}

pub(crate) fn process_simple_arg(arg: &str, params: &LaunchParameters) -> Option<String> {
	replace_arg_placeholders(arg, params)
}

macro_rules! placeholder {
	($name:expr) => {
		concat!("${", $name, "}")
	};
}

pub(crate) fn replace_arg_placeholders(arg: &str, params: &LaunchParameters) -> Option<String> {
	let mut out = arg.replace(
		placeholder!("launcher_name"),
		&params.branding.launcher_name,
	);
	out = out.replace(
		placeholder!("launcher_version"),
		&params.branding.launcher_version,
	);

	out = out.replace(placeholder!("classpath"), &params.classpath.get_str());
	out = out.replace(
		placeholder!("natives_directory"),
		params
			.paths
			.internal
			.join("versions")
			.join(params.version.to_string())
			.join("natives")
			.to_string_lossy()
			.as_ref(),
	);
	out = out.replace(placeholder!("version_name"), params.version);
	out = out.replace(placeholder!("version_type"), "Sqrilizz Launcher");
	out = out.replace(
		placeholder!("game_directory"),
		params.launch_dir.to_string_lossy().as_ref(),
	);
	out = out.replace(
		placeholder!("assets_root"),
		params.paths.assets.to_string_lossy().as_ref(),
	);
	out = out.replace(placeholder!("assets_index_name"), params.version);
	out = out.replace(
		placeholder!("game_assets"),
		get_virtual_dir_path(params.paths)
			.to_string_lossy()
			.as_ref(),
	);

	out = out.replace(placeholder!("clientid"), "Sqrilizz Launcher");
	out = out.replace(placeholder!("user_properties"), "\"\"");

	let InstanceKind::Client { window } = &params.side else {
		panic!("Instance is not a client")
	};
	if let Some(WindowResolution { width, height }) = window.resolution {
		out = out.replace(placeholder!("resolution_width"), &width.to_string());
		out = out.replace(placeholder!("resolution_height"), &height.to_string());
	}

	out = out.replace(placeholder!("quickPlayPath"), "quickPlay/log.json");
	out = out.replace(
		placeholder!("quickPlaySingleplayer"),
		if let QuickPlayType::World { world } = &params.launch_config.quick_play {
			world
		} else {
			""
		},
	);
	out = out.replace(
		placeholder!("quickPlayMultiplayer"),
		&if let QuickPlayType::Server { server, port } = &params.launch_config.quick_play {
			if let Some(port) = port {
				format!("{server}:{port}")
			} else {
				server.clone()
			}
		} else {
			String::new()
		},
	);
	out = out.replace(
		placeholder!("quickPlayRealms"),
		if let QuickPlayType::Realm { realm } = &params.launch_config.quick_play {
			realm
		} else {
			""
		},
	);

	match params.accounts.get_chosen_account() {
		Some(account) => {
			let user_type = match account.get_kind() {
				AccountKind::Microsoft { .. } => "msa",
				_ => "mojang",
			};
			out = out.replace(placeholder!("user_type"), user_type);

			if let Some(username) = account.get_name().or(Some(account.get_id())) {
				out = out.replace(placeholder!("auth_player_name"), username);
			}
			if let Some(uuid) = account.get_uuid() {
				out = out.replace(placeholder!("auth_uuid"), uuid);
			} else {
				let mut bytes = [0u8; 16];
				for (i, b) in format!("OfflinePlayer:{}", account.get_id())
					.bytes()
					.enumerate()
				{
					bytes[i % 16] = bytes[i % 16].wrapping_add(b);
				}
				let uuid_str: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
				out = out.replace(placeholder!("auth_uuid"), &uuid_str);
			}
			if let Some(access_token) = account.get_access_token() {
				out = out.replace(placeholder!("auth_access_token"), &access_token.0);
			} else {
				out = out.replace(placeholder!("auth_access_token"), "0");
			}
			if let AccountKind::Microsoft {
				xbox_uid: Some(xbox_uid),
			} = &account.kind
			{
				out = out.replace(placeholder!("auth_xuid"), xbox_uid);
			} else {
				out = out.replace(placeholder!("auth_xuid"), "0");
			}

			if out.contains(placeholder!("auth_player_name"))
				|| out.contains(placeholder!("auth_access_token"))
				|| out.contains(placeholder!("auth_uuid"))
				|| out.contains(placeholder!("auth_xuid"))
			{
				return Some(String::new());
			}
		}
		None => {
			if out.contains(placeholder!("auth_player_name")) {
				return Some("UnknownUser".into());
			}
			if out.contains(placeholder!("auth_access_token"))
				|| out.contains(placeholder!("auth_uuid"))
			{
				return Some(String::new());
			}
		}
	}

	Some(out)
}

pub fn create_quick_play_args(
	quick_play: &QuickPlayType,
	version: &str,
	_version_list: &[String],
	o: &mut impl SqrilizzOutput,
) -> Vec<String> {
	let mut out = Vec::new();

	let is_ge_1_20 = matches!(
		version_compare::compare(version, "1.20"),
		Ok(version_compare::Cmp::Ge | version_compare::Cmp::Gt | version_compare::Cmp::Eq)
	);

	match quick_play {
		QuickPlayType::None => {}
		QuickPlayType::World { world } => {
			if is_ge_1_20 {
				out.extend(["--quickPlayPath".into(), "quickPlay/log.json".into()]);
				out.extend(["--quickPlaySingleplayer".into(), world.clone()]);
			} else {
				o.display(MessageContents::Warning(
					"Minecraft version does not support quick play".into(),
				));
			}
		}
		QuickPlayType::Server { server, port } => {
			if is_ge_1_20 {
				out.extend(["--quickPlayPath".into(), "quickPlay/log.json".into()]);
				let server = if let Some(port) = port {
					format!("{server}:{port}")
				} else {
					server.clone()
				};
				out.extend(["--quickPlayMultiplayer".into(), server]);
			} else {
				o.display(MessageContents::Warning(
					"Minecraft version does not support quick play".into(),
				));
			}
		}
		QuickPlayType::Realm { realm } => {
			if is_ge_1_20 {
				out.extend(["--quickPlayPath".into(), "quickPlay/log.json".into()]);
				out.extend(["--quickPlayRealms".into(), realm.clone()]);
			} else {
				o.display(MessageContents::Warning(
					"Minecraft version does not support quick play".into(),
				));
			}
		}
	}

	out
}
