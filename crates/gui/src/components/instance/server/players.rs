use std::time::Duration;

use crate::{
	ops::{
		launch::{FetchInstanceRunState, InstanceRunState},
		server::{FetchServerPing, send_console_command},
		server_ping::{PlayerSample, ServerPingState},
	},
	prelude::*,
};

fn is_running(run_state: Option<InstanceRunState>) -> bool {
	run_state == Some(InstanceRunState::Running)
}

#[derive(PartialEq)]
pub struct ServerPlayers {
	pub id: String,
}

impl Component for ServerPlayers {
	fn render(&self) -> impl IntoElement {
		let theme = use_theme();
		let back_state = use_consume::<BackState>();
		let front_state = use_front_state();

		let run_state = use_query(Query::new(
			self.id.clone(),
			FetchInstanceRunState::new(back_state.clone()),
		));
		let ping_query = use_query(Query::new(
			self.id.clone(),
			FetchServerPing::new(back_state.clone()),
		));

		// Refresh periodically while the component is visible
		let ping_query2 = ping_query.clone();
		use_future(move || {
			let ping_query2 = ping_query2;
			async move {
				loop {
					tokio::time::sleep(Duration::from_secs(3)).await;
					ping_query2.invalidate();
				}
			}
		});

		let state = run_state.read().state().ok().cloned();
		let ping = ping_query.read().state().ok().cloned().flatten();

		let content = if is_running(state) {
			match ping {
				Some(ServerPingState::Online {
					players_online,
					players_max,
					player_sample,
					latency_ms,
					..
				}) => {
					let mut rows = rect().width(Size::fill()).spacing(theme.gap2);

					rows = rows.child(
						rect()
							.width(Size::fill())
							.background(theme.item)
							.corner_radius(theme.round)
							.border(theme.border(theme.item_border))
							.padding(Gaps::new(theme.gap2, theme.gap3, theme.gap2, theme.gap3))
							.horizontal()
							.cross_align(Alignment::Center)
							.spacing(theme.gap2)
							.child(
								rect()
									.horizontal()
									.cross_align(Alignment::Center)
									.spacing(theme.gap)
									.child(icon("multiple_users", 20.0).color(theme.primary))
									.child(
										label()
											.text("Игроки на сервере")
											.font_size(theme.font2)
											.font_weight(FontWeight::BOLD)
											.color(theme.fg),
									),
							)
							.child(rect().width(Size::flex(1.0)))
							.child(
								rect()
									.horizontal()
									.cross_align(Alignment::Center)
									.spacing(theme.gap)
									.padding(Gaps::new(4.0, 10.0, 4.0, 10.0))
									.background(theme.item_hover)
									.corner_radius(theme.round)
									.child(
										label()
											.text(format!(
												"{players_online} из {players_max} онлайн • {latency_ms} мс"
											))
											.font_size(theme.font0)
											.font_weight(FontWeight::BOLD)
											.color(theme.fg),
									),
							),
					);

					if player_sample.is_empty() {
						rows = rows.child(
							rect()
								.width(Size::fill())
								.height(Size::px(160.0))
								.background(theme.item)
								.corner_radius(theme.round)
								.border(theme.border(theme.item_border))
								.center()
								.spacing(theme.gap)
								.child(icon("multiple_users", 36.0).color(theme.fg3))
								.child(
									label()
										.text("Сейчас на сервере никого нет")
										.font_size(theme.font)
										.font_weight(FontWeight::BOLD)
										.color(theme.fg),
								)
								.child(
									label()
										.text(
											"Игроки появятся в этом списке сразу после подключения",
										)
										.font_size(theme.font0)
										.color(theme.fg3),
								),
						);
					} else {
						for PlayerSample { name, .. } in player_sample.clone() {
							let id_cmd = self.id.clone();
							let bs_cmd = back_state.clone();
							let fs_cmd = front_state.clone();
							let player_name = name.clone();

							let p1 = player_name.clone();
							let p2 = player_name.clone();
							let p3 = player_name.clone();
							let p4 = player_name.clone();
							let p5 = player_name.clone();

							let id1 = id_cmd.clone();
							let id2 = id_cmd.clone();
							let id3 = id_cmd.clone();
							let id4 = id_cmd.clone();
							let id5 = id_cmd.clone();

							let bs1 = bs_cmd.clone();
							let bs2 = bs_cmd.clone();
							let bs3 = bs_cmd.clone();
							let bs4 = bs_cmd.clone();
							let bs5 = bs_cmd.clone();

							let fs1 = fs_cmd.clone();
							let fs2 = fs_cmd.clone();
							let fs3 = fs_cmd.clone();
							let fs4 = fs_cmd.clone();
							let fs5 = fs_cmd.clone();

							rows = rows.child(
								rect()
									.width(Size::fill())
									.height(Size::px(60.0))
									.background(theme.item)
									.corner_radius(theme.round)
									.border(theme.border(theme.item_border))
									.padding(Gaps::new(
										theme.gap, theme.gap3, theme.gap, theme.gap3,
									))
									.horizontal()
									.cross_align(Alignment::Center)
									.spacing(theme.gap2)
									.child(
										rect()
											.width(Size::px(38.0))
											.height(Size::px(38.0))
											.center()
											.background(theme.item_hover)
											.corner_radius(theme.round)
											.child(icon("user", 20.0).color(theme.primary)),
									)
									.child(
										rect().width(Size::flex(1.0)).child(
											label()
												.text(name)
												.font_size(theme.font)
												.font_weight(FontWeight::BOLD)
												.color(theme.fg),
										),
									)
									.child(
										elem_text_button(icon("star", 12.0), "OP", &theme)
											.on_press(move |_| {
												let id = id1.clone();
												let bs = bs1.clone();
												let fs = fs1.clone();
												let p = p1.clone();
												spawn(async move {
													let _ = send_console_command(
														&bs,
														&id,
														&format!("op {p}"),
													)
													.await;
													fs.write().toast(Toast::success(&format!(
														"Выданы права оператора: {p}"
													)));
												});
											}),
									)
									.child(
										elem_text_button(icon("dice", 12.0), "Креатив", &theme)
											.on_press(move |_| {
												let id = id2.clone();
												let bs = bs2.clone();
												let fs = fs2.clone();
												let p = p2.clone();
												spawn(async move {
													let _ = send_console_command(
														&bs,
														&id,
														&format!("gamemode creative {p}"),
													)
													.await;
													fs.write().toast(Toast::success(&format!(
														"Режим Creative: {p}"
													)));
												});
											}),
									)
									.child(
										elem_text_button(icon("sword", 12.0), "Выживание", &theme)
											.on_press(move |_| {
												let id = id3.clone();
												let bs = bs3.clone();
												let fs = fs3.clone();
												let p = p3.clone();
												spawn(async move {
													let _ = send_console_command(
														&bs,
														&id,
														&format!("gamemode survival {p}"),
													)
													.await;
													fs.write().toast(Toast::success(&format!(
														"Режим Survival: {p}"
													)));
												});
											}),
									)
									.child(
										elem_text_button(icon("stop", 12.0), "Кик", &theme)
											.color(theme.warning)
											.on_press(move |_| {
												let id = id4.clone();
												let bs = bs4.clone();
												let fs = fs4.clone();
												let p = p4.clone();
												spawn(async move {
													let _ = send_console_command(
														&bs,
														&id,
														&format!("kick {p}"),
													)
													.await;
													fs.write().toast(Toast::success(&format!(
														"Игрок {p} исключен"
													)));
												});
											}),
									)
									.child(
										elem_text_button(icon("trash", 12.0), "Бан", &theme)
											.color(theme.error)
											.on_press(move |_| {
												let id = id5.clone();
												let bs = bs5.clone();
												let fs = fs5.clone();
												let p = p5.clone();
												spawn(async move {
													let _ = send_console_command(
														&bs,
														&id,
														&format!("ban {p}"),
													)
													.await;
													fs.write().toast(Toast::success(&format!(
														"Игрок {p} забанен"
													)));
												});
											}),
									),
							);
						}
					}

					rect().width(Size::fill()).child(rows).into_element()
				}
				Some(ServerPingState::Loading) | None => rect()
					.width(Size::fill())
					.height(Size::px(220.0))
					.background(theme.item)
					.corner_radius(theme.round)
					.border(theme.border(theme.item_border))
					.center()
					.child(
						label()
							.text("Опрос сервера...")
							.font_size(theme.font)
							.color(theme.fg3),
					)
					.into_element(),
				Some(ServerPingState::Offline { .. }) => rect()
					.width(Size::fill())
					.height(Size::px(220.0))
					.background(theme.item)
					.corner_radius(theme.round)
					.border(theme.border(theme.item_border))
					.center()
					.spacing(theme.gap)
					.child(icon("error", 32.0).color(theme.warning))
					.child(
						label()
							.text("Сервер не отвечает на пинг")
							.font_size(theme.font)
							.font_weight(FontWeight::BOLD)
							.color(theme.fg),
					)
					.into_element(),
			}
		} else {
			rect()
				.width(Size::fill())
				.height(Size::px(240.0))
				.background(theme.item)
				.corner_radius(theme.round)
				.border(theme.border(theme.item_border))
				.center()
				.spacing(theme.gap2)
				.child(icon("server", 44.0).color(theme.fg3))
				.child(
					label()
						.text("Сервер остановлен")
						.font_size(theme.font2)
						.font_weight(FontWeight::BOLD)
						.color(theme.fg),
				)
				.child(
					label()
						.text(
							"Запустите сервер, чтобы просматривать активных игроков и управлять ими",
						)
						.font_size(theme.font0)
						.color(theme.fg3),
				)
				.into_element()
		};

		let body = rect()
			.width(Size::fill())
			.padding(theme.gap3)
			.child(content);

		ScrollView::new()
			.width(Size::fill())
			.height(Size::fill())
			.child(body)
			.into_element()
	}
}
