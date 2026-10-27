use std::time::Duration;

use crate::{
	components::instance::console::InstanceConsole,
	ops::{
		launch::{FetchInstanceRunState, InstanceRunState},
		server::{
			FetchServerAddress, FetchServerPing, FetchServerStats, PollServerTps,
			write_console_command,
		},
		server_ping::ServerPingState,
	},
	prelude::*,
};

fn format_uptime(secs: u64) -> String {
	let hours = secs / 3600;
	let mins = (secs % 3600) / 60;
	let s = secs % 60;
	if hours > 0 {
		format!("{hours:02}:{mins:02}:{s:02}")
	} else {
		format!("{mins:02}:{s:02}")
	}
}

fn format_filesize(bytes: u64) -> String {
	if bytes >= 1024 * 1024 * 1024 {
		format!("{:.2} ГБ", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
	} else if bytes >= 1024 * 1024 {
		format!("{:.1} МБ", bytes as f64 / (1024.0 * 1024.0))
	} else if bytes >= 1024 {
		format!("{} КБ", bytes / 1024)
	} else {
		format!("{bytes} Б")
	}
}

#[derive(PartialEq)]
pub struct ServerOverview {
	pub id: String,
}

impl Component for ServerOverview {
	fn render(&self) -> impl IntoElement {
		let theme = use_theme();
		let back_state = use_consume::<BackState>();
		let front_state = use_front_state();

		// Live polling for stats
		let stats_query = use_query(Query::new(
			self.id.clone(),
			FetchServerStats::new(back_state.clone()),
		));
		let ping_query = use_query(Query::new(
			self.id.clone(),
			FetchServerPing::new(back_state.clone()),
		));
		let addr_query = use_query(Query::new(
			self.id.clone(),
			FetchServerAddress::new(back_state.clone()),
		));
		let tps_mutation = use_mutation(Mutation::new(PollServerTps::new(back_state.clone())));

		// Run state
		let run_state = use_query(Query::new(
			self.id.clone(),
			FetchInstanceRunState::new(back_state.clone()),
		));
		let is_running = run_state.read().state().ok().cloned() == Some(InstanceRunState::Running);

		// Periodic background refresher for telemetry
		let stats_q = stats_query;
		let ping_q = ping_query;
		let addr_q = addr_query;
		let tps_mut = tps_mutation;
		let id_poll = self.id.clone();
		use_future(move || {
			let id = id_poll.clone();
			async move {
				let mut tick_counter: u32 = 0;
				loop {
					tokio::time::sleep(Duration::from_millis(2000)).await;
					stats_q.invalidate();
					ping_q.invalidate();
					addr_q.invalidate();
					tick_counter += 1;
					if is_running && tick_counter % 15 == 1 {
						tps_mut.mutate(id.clone());
					}
				}
			}
		});

		let stats = stats_query.read().state().ok().cloned().unwrap_or_default();
		let ping = ping_query.read().state().ok().cloned().flatten();
		let server_addr = addr_query
			.read()
			.state()
			.ok()
			.cloned()
			.flatten()
			.unwrap_or_else(|| "localhost:25565".to_string());

		let (players_online, players_max, latency_ms) = match &ping {
			Some(ServerPingState::Online {
				players_online,
				players_max,
				latency_ms,
				..
			}) => (*players_online, *players_max, *latency_ms),
			_ => (0, 0, 0),
		};

		let tps = tps_mutation.read().state().ok().cloned().flatten();

		// --- 1. Pterodactyl-Style Telemetry Bar (5 Equal-width Flex Cards) ---
		// CPU Card
		let cpu_val = stats.cpu_percent;
		let cpu_color = if cpu_val < 60.0 {
			Color::from_rgb(16, 185, 129)
		} else if cpu_val < 85.0 {
			Color::from_rgb(245, 158, 11)
		} else {
			Color::from_rgb(239, 68, 68)
		};

		let cpu_card = rect()
			.width(Size::flex(1.0))
			.height(Size::fill())
			.background(theme.item)
			.border(theme.border(theme.panel_border))
			.corner_radius(theme.round)
			.padding(Gaps::new(8.0, 12.0, 8.0, 12.0))
			.spacing(3.0)
			.child(
				rect()
					.horizontal()
					.cross_align(Alignment::Center)
					.spacing(6.0)
					.child(icon("speed", 14.0).color(theme.fg3))
					.child(
						label()
							.text("НАГРУЗКА ЦП")
							.font_size(10.0)
							.font_weight(FontWeight::BOLD)
							.color(theme.fg3),
					),
			)
			.child(
				label()
					.text(if stats.running {
						format!("{cpu_val:.1}%")
					} else {
						"0.0%".to_string()
					})
					.font_size(theme.font)
					.font_weight(FontWeight::BOLD)
					.color(theme.fg),
			)
			.child(
				rect()
					.width(Size::fill())
					.height(Size::px(3.0))
					.background(theme.item_border)
					.corner_radius(1.5)
					.child(
						rect()
							.width(Size::percent(if stats.running {
								cpu_val.clamp(2.0, 100.0)
							} else {
								0.0
							}))
							.height(Size::fill())
							.background(cpu_color)
							.corner_radius(1.5),
					),
			);

		// RAM Card
		let ram_card = rect()
			.width(Size::flex(1.0))
			.height(Size::fill())
			.background(theme.item)
			.border(theme.border(theme.panel_border))
			.corner_radius(theme.round)
			.padding(Gaps::new(8.0, 12.0, 8.0, 12.0))
			.spacing(3.0)
			.child(
				rect()
					.horizontal()
					.cross_align(Alignment::Center)
					.spacing(6.0)
					.child(icon("box", 14.0).color(theme.fg3))
					.child(
						label()
							.text("ПАМЯТЬ (ОЗУ)")
							.font_size(10.0)
							.font_weight(FontWeight::BOLD)
							.color(theme.fg3),
					),
			)
			.child(
				label()
					.text(if stats.running {
						format_filesize(stats.mem_bytes)
					} else {
						"0 МБ".to_string()
					})
					.font_size(theme.font)
					.font_weight(FontWeight::BOLD)
					.color(theme.fg),
			)
			.child(
				rect()
					.width(Size::fill())
					.height(Size::px(3.0))
					.background(theme.item_border)
					.corner_radius(1.5)
					.child(
						rect()
							.width(Size::percent(if stats.running {
								((stats.mem_bytes as f32 / (4.0 * 1024.0 * 1024.0 * 1024.0))
									* 100.0)
									.clamp(3.0, 100.0)
							} else {
								0.0
							}))
							.height(Size::fill())
							.background(theme.primary)
							.corner_radius(1.5),
					),
			);

		// TPS Card
		let tps_chips = if let Some((t1, t5, t15)) = tps {
			rect()
				.horizontal()
				.spacing(4.0)
				.child(tps_chip(t1, &theme))
				.child(tps_chip(t5, &theme))
				.child(tps_chip(t15, &theme))
		} else {
			rect().child(
				label()
					.text(if stats.running { "20.0" } else { "20.0" })
					.font_size(theme.font)
					.font_weight(FontWeight::BOLD)
					.color(if stats.running {
						Color::from_rgb(16, 185, 129)
					} else {
						theme.fg3
					}),
			)
		};

		let tps_card = rect()
			.width(Size::flex(1.0))
			.height(Size::fill())
			.background(theme.item)
			.border(theme.border(theme.panel_border))
			.corner_radius(theme.round)
			.padding(Gaps::new(8.0, 12.0, 8.0, 12.0))
			.spacing(3.0)
			.child(
				rect()
					.horizontal()
					.cross_align(Alignment::Center)
					.spacing(6.0)
					.child(icon("lightning", 14.0).color(theme.fg3))
					.child(
						label()
							.text("TPS СЕРВЕРА")
							.font_size(10.0)
							.font_weight(FontWeight::BOLD)
							.color(theme.fg3),
					),
			)
			.child(tps_chips)
			.child(
				label()
					.text("1м / 5м / 15м")
					.font_size(9.0)
					.color(theme.fg3),
			);

		// Players Card
		let players_card = rect()
			.width(Size::flex(1.0))
			.height(Size::fill())
			.background(theme.item)
			.border(theme.border(theme.panel_border))
			.corner_radius(theme.round)
			.padding(Gaps::new(8.0, 12.0, 8.0, 12.0))
			.spacing(3.0)
			.child(
				rect()
					.horizontal()
					.cross_align(Alignment::Center)
					.spacing(6.0)
					.child(icon("multiple_users", 14.0).color(theme.fg3))
					.child(
						label()
							.text("ИГРОКИ")
							.font_size(10.0)
							.font_weight(FontWeight::BOLD)
							.color(theme.fg3),
					),
			)
			.child(
				label()
					.text(if stats.running {
						format!("{players_online} / {players_max}")
					} else {
						"0 / 0".to_string()
					})
					.font_size(theme.font)
					.font_weight(FontWeight::BOLD)
					.color(theme.fg),
			)
			.child(
				label()
					.text(if stats.running {
						format!("{latency_ms} мс пинг")
					} else {
						"Оффлайн".to_string()
					})
					.font_size(9.0)
					.color(theme.fg3),
			);

		// Uptime & Address Card
		let address_to_copy = server_addr.clone();
		let fs_addr = front_state.clone();
		let uptime_card = rect()
			.width(Size::flex(1.0))
			.height(Size::fill())
			.background(theme.item)
			.border(theme.border(theme.panel_border))
			.corner_radius(theme.round)
			.padding(Gaps::new(8.0, 12.0, 8.0, 12.0))
			.spacing(3.0)
			.child(
				rect()
					.horizontal()
					.cross_align(Alignment::Center)
					.spacing(6.0)
					.child(icon("clock", 14.0).color(theme.fg3))
					.child(
						label()
							.text("АПТАЙМ")
							.font_size(10.0)
							.font_weight(FontWeight::BOLD)
							.color(theme.fg3),
					),
			)
			.child(
				label()
					.text(if stats.running {
						format_uptime(stats.uptime_secs)
					} else {
						"Остановлен".to_string()
					})
					.font_size(theme.font)
					.font_weight(FontWeight::BOLD)
					.color(if stats.running {
						Color::from_rgb(16, 185, 129)
					} else {
						theme.fg3
					}),
			)
			.child(
				rect()
					.horizontal()
					.cross_align(Alignment::Center)
					.spacing(4.0)
					.clickable()
					.on_press(move |_| {
						let _ = Clipboard::set(address_to_copy.clone());
						fs_addr
							.write()
							.toast(Toast::success("Адрес скопирован в буфер обмена"));
					})
					.child(icon("copy", 10.0).color(theme.primary))
					.child(
						label()
							.text(server_addr)
							.font_size(9.0)
							.color(theme.primary),
					),
			);

		let telemetry_bar = rect()
			.width(Size::fill())
			.height(Size::px(70.0))
			.horizontal()
			.flex()
			.spacing(theme.gap)
			.child(cpu_card)
			.child(ram_card)
			.child(tps_card)
			.child(players_card)
			.child(uptime_card);

		// --- 2. Live Streaming Console Terminal ---
		let console_view =
			rect()
				.width(Size::fill())
				.height(Size::flex(1.0))
				.child(InstanceConsole {
					id: self.id.clone(),
					is_server: true,
				});

		// --- 3. Pterodactyl Quick Actions Chips Bar ---
		let id_chip = self.id.clone();
		let bs_chip = back_state.clone();
		let fs_chip = front_state.clone();
		let theme_chip = theme.clone();

		let make_chip =
			move |cmd: &'static str, label_text: &'static str, icon_name: &'static str| {
				let id = id_chip.clone();
				let bs = bs_chip.clone();
				let fs = fs_chip.clone();
				let th = theme_chip.clone();
				let is_hovered = use_state(|| false);

				rect()
					.height(Size::px(26.0))
					.padding(Gaps::new(0.0, 10.0, 0.0, 10.0))
					.background(if *is_hovered.read() {
						th.item_hover
					} else {
						th.item
					})
					.border(th.border(th.item_border))
					.corner_radius(th.round)
					.horizontal()
					.cross_align(Alignment::Center)
					.spacing(6.0)
					.hover(is_hovered)
					.clickable()
					.on_press(move |_| {
						let id = id.clone();
						let bs = bs.clone();
						let fs = fs.clone();
						spawn(async move {
							match write_console_command(&bs, &id, cmd).await {
								Ok(_) => {
									fs.write()
										.toast(Toast::success(&format!("Отправлено: /{cmd}")));
								}
								Err(e) => {
									fs.write()
										.toast(Toast::error(&format!("Ошибка: {e}"), None));
								}
							}
						});
					})
					.child(icon(icon_name, 12.0).color(th.primary))
					.child(
						label()
							.text(format!("/{label_text}"))
							.font_size(th.font0)
							.color(th.fg),
					)
			};

		let quick_bar = rect()
			.width(Size::fill())
			.height(Size::px(30.0))
			.horizontal()
			.cross_align(Alignment::Center)
			.spacing(theme.gap)
			.padding(Gaps::new(0.0, 2.0, 0.0, 2.0))
			.child(
				label()
					.text("Шаблоны:")
					.font_size(theme.font0)
					.font_weight(FontWeight::BOLD)
					.color(theme.fg3),
			)
			.child(make_chip("list", "list", "text"))
			.child(make_chip("tps", "tps", "lightning"))
			.child(make_chip("save-all", "save-all", "box"))
			.child(make_chip("time set day", "time set day", "sun"))
			.child(make_chip("weather clear", "weather clear", "feather"))
			.child(make_chip("whitelist on", "whitelist on", "key"))
			.child(make_chip("reload confirm", "reload confirm", "cycle"));

		// Overall Pterodactyl Dashboard Root Container
		rect()
			.width(Size::fill())
			.height(Size::fill())
			.flex()
			.padding(theme.gap2)
			.spacing(theme.gap)
			.child(telemetry_bar)
			.child(console_view)
			.child(quick_bar)
	}
}

fn tps_chip(val: f32, theme: &Theme) -> Rect {
	let color = if val >= 19.0 {
		Color::from_rgb(16, 185, 129)
	} else if val >= 15.0 {
		Color::from_rgb(245, 158, 11)
	} else {
		Color::from_rgb(239, 68, 68)
	};

	rect()
		.padding(Gaps::new(1.0, 4.0, 1.0, 4.0))
		.background(Color::from_argb(35, color.r(), color.g(), color.b()))
		.border(theme.border(color))
		.corner_radius(3.0)
		.child(
			label()
				.text(format!("{val:.1}"))
				.font_size(10.0)
				.font_weight(FontWeight::BOLD)
				.color(color),
		)
}
