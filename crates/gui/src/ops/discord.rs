use std::{
	sync::{
		LazyLock,
		mpsc::{Receiver, Sender, channel},
	},
	thread,
	time::Duration,
};
use std::sync::atomic::{AtomicBool, Ordering};

use discord_rich_presence::{
	DiscordIpc, DiscordIpcClient,
	activity::{Activity, ActivityType, Assets, Button, StatusDisplayType, Timestamps},
};
use sqrilizz::shared::util::utc_timestamp;

pub static DISCORD_CLIENT_ID: &str = "1454150579629588684";
pub static DISCORD_LARGE_IMAGE: &str = "sqrilizz";
static DISCORD_ENABLED: AtomicBool = AtomicBool::new(true);

#[derive(Clone, Debug)]
pub enum DiscordAction {
	SetInLauncher,
	SetInGame {
		instance_name: String,
		details: Option<String>,
		start_time: i64,
	},
	#[allow(dead_code)]
	Clear,
}

static DISCORD_TX: LazyLock<Sender<DiscordAction>> = LazyLock::new(|| {
	let (tx, rx) = channel::<DiscordAction>();
	let _ = thread::Builder::new()
		.name("discord-rpc".into())
		.spawn(move || rpc_worker(rx));
	tx
});

fn rpc_worker(rx: Receiver<DiscordAction>) {
	let mut client = DiscordIpcClient::new(DISCORD_CLIENT_ID);
	let mut is_connected = false;
	let mut current_action: Option<DiscordAction> = Some(DiscordAction::SetInLauncher);

	loop {
		if !is_connected && client.connect().is_ok() {
			is_connected = true;
			if let Some(ref action) = current_action {
				let _ = apply_action(&mut client, action);
			}
		}

		match rx.recv_timeout(Duration::from_secs(3)) {
			Ok(action) => {
				current_action = Some(action.clone());
				if !is_connected && client.connect().is_ok() {
					is_connected = true;
				}
				if is_connected && apply_action(&mut client, &action).is_err() {
					is_connected = false;
					let _ = client.close();
				}
			}
			Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
				if !is_connected && client.connect().is_ok() {
					is_connected = true;
					if let Some(ref action) = current_action {
						let _ = apply_action(&mut client, action);
					}
				}
			}
			Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
				let _ = client.close();
				break;
			}
		}
	}
}

fn apply_action(client: &mut DiscordIpcClient, action: &DiscordAction) -> Result<(), ()> {
	match action {
		DiscordAction::SetInLauncher => {
			let now = utc_timestamp().unwrap_or_default() as i64;
			let payload = Activity::new()
				.status_display_type(StatusDisplayType::Name)
				.state("В лаунчере")
				.details("В главном меню")
				.activity_type(ActivityType::Playing)
				.timestamps(Timestamps::new().start(now))
				.assets(
					Assets::new()
						.large_image(DISCORD_LARGE_IMAGE)
						.large_text("Sqrilizz Launcher"),
				)
				.buttons(vec![Button::new(
					"Дискорд сообщество",
					"https://discord.gg/25fhkjeTvW",
				)]);
			client.set_activity(payload).map_err(|_| ())?;
		}
		DiscordAction::SetInGame {
			instance_name,
			details,
			start_time,
		} => {
			let state_text = format!("Играет в {}", instance_name);
			let mut payload = Activity::new()
				.status_display_type(StatusDisplayType::Name)
				.state(&state_text)
				.activity_type(ActivityType::Playing)
				.timestamps(Timestamps::new().start(*start_time))
				.assets(
					Assets::new()
						.large_image(DISCORD_LARGE_IMAGE)
						.large_text("Sqrilizz Launcher"),
				)
				.buttons(vec![Button::new(
					"Дискорд сообщество",
					"https://discord.gg/25fhkjeTvW",
				)]);
			if let Some(d) = details {
				payload = payload.details(d);
			}
			client.set_activity(payload).map_err(|_| ())?;
		}
		DiscordAction::Clear => {
			let _ = client.clear_activity();
		}
	}
	Ok(())
}

pub fn set_enabled(enabled: bool) {
	DISCORD_ENABLED.store(enabled, Ordering::Relaxed);
	if enabled {
		let _ = DISCORD_TX.send(DiscordAction::SetInLauncher);
	} else {
		let _ = DISCORD_TX.send(DiscordAction::Clear);
	}
}

pub fn init_discord_rpc() {
	if DISCORD_ENABLED.load(Ordering::Relaxed) {
		let _ = DISCORD_TX.send(DiscordAction::SetInLauncher);
	}
}

pub fn set_instance_presence(instance_name: &str, details: Option<&str>) {
	if !DISCORD_ENABLED.load(Ordering::Relaxed) {
		return;
	}
	let start_time = utc_timestamp().unwrap_or_default() as i64;
	let _ = DISCORD_TX.send(DiscordAction::SetInGame {
		instance_name: instance_name.to_string(),
		details: details.map(|s| s.to_string()),
		start_time,
	});
}

pub fn clear_presence() {
	if DISCORD_ENABLED.load(Ordering::Relaxed) {
		let _ = DISCORD_TX.send(DiscordAction::SetInLauncher);
	}
}
