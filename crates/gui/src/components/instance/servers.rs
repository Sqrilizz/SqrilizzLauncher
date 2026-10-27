use std::time::Duration;

use crate::{
	components::{
		dialog::{
			modal::{Modal, ModalButton},
			toast::Toast,
		},
		input::text::TextInput,
		misc::number_indicator,
	},
	ops::{
		instance::{
			AddInstanceServer, AddServerParams, FetchInstanceFiles, RemoveInstanceServer,
			RemoveServerParams,
		},
		launch::{LaunchInstance, LaunchInstanceParams},
		server_ping::{ServerPingState, parse_minecraft_formatting, ping_server},
	},
	prelude::*,
};
use sqrilizz::core::QuickPlayType;

fn render_motd(motd: &str, theme: &Theme) -> Element {
	let clean_motd = motd.replace("\r\n", " ").replace('\n', " ");
	let spans = parse_minecraft_formatting(&clean_motd, theme.fg2);
	if spans.is_empty() || spans.iter().all(|s| s.text.trim().is_empty()) {
		return label()
			.text("Сервер онлайн")
			.font_size(theme.font0)
			.color(theme.fg2)
			.into_element();
	}

	let mut p = paragraph()
		.font_size(theme.font0)
		.max_lines(1)
		.text_overflow(TextOverflow::Ellipsis);
	for span in spans {
		let mut s = Span::from(span.text)
			.font_size(theme.font0)
			.color(span.color.unwrap_or(theme.fg2));
		if span.bold {
			s = s.font_weight(FontWeight::BOLD);
		}
		if span.italic {
			s = s.font_slant(FontSlant::Italic);
		}
		if span.underline {
			s = s.text_decoration(TextDecoration::Underline);
		}
		if span.strikethrough {
			s = s.text_decoration(TextDecoration::LineThrough);
		}
		p = p.span(s);
	}
	p.into_element()
}

#[derive(PartialEq)]
pub struct ServerCard {
	pub icon: Option<ImageSource>,
	pub name: String,
	pub address: String,
	pub instance_id: String,
	pub refresh_id: usize,
}

impl Component for ServerCard {
	fn render(&self) -> impl IntoElement {
		let theme = use_theme();
		let front_state = use_front_state();
		let back_state = use_consume::<BackState>();

		let is_hovered = use_state(|| false);
		let ping_state = use_state(|| ServerPingState::Loading);

		let manual_refresh = use_state(|| 0usize);
		let last_refresh_id = use_state(|| self.refresh_id);
		let mut last_refresh_setter = last_refresh_id;
		let mut manual_setter = manual_refresh;

		let current_refresh_id = self.refresh_id;
		use_side_effect(move || {
			if *last_refresh_setter.peek() != current_refresh_id {
				last_refresh_setter.set(current_refresh_id);
				manual_setter.set(*manual_setter.peek() + 1);
			}
		});

		let addr = self.address.clone();
		let mut ping_setter = ping_state;
		use_future(move || {
			let addr = addr.clone();
			let _ = *manual_refresh.read();
			async move {
				ping_setter.set(ServerPingState::Loading);
				let res = ping_server(&addr).await;
				ping_setter.set(res);
			}
		});

		let current_ping = ping_state.read().clone();

		let ico = if let Some(icon) = &self.icon {
			ImageViewer::new(icon.clone())
				.width(Size::px(44.0))
				.height(Size::px(44.0))
				.corner_radius(theme.round)
				.into_element()
		} else if let ServerPingState::Online {
			favicon_bytes: Some(bytes),
			..
		} = &current_ping
		{
			ImageViewer::new(ImageSource::from(Bytes::from(bytes.clone())))
				.width(Size::px(44.0))
				.height(Size::px(44.0))
				.corner_radius(theme.round)
				.into_element()
		} else {
			rect()
				.width(Size::px(44.0))
				.height(Size::px(44.0))
				.corner_radius(theme.round)
				.background(theme.item_hover)
				.center()
				.child(icon("server", 22.0))
				.into_element()
		};

		let launch_mutation = use_mutation(LaunchInstance::new(back_state.clone()));
		let remove_server_mutation = use_mutation(RemoveInstanceServer::new(back_state));

		let is_confirm_delete_open = use_state(|| false);
		let mut confirm_delete_setter = is_confirm_delete_open;

		let inst_id_del = self.instance_id.clone();
		let server_addr_del = self.address.clone();
		let server_name_del = self.name.clone();
		let front_state_del = front_state.clone();

		let delete_button = Button::new().child(icon("trash", 14.0)).on_press(move |_| {
			confirm_delete_setter.set(true);
		});
		let delete_button_tipped = rect()
			.tip(&front_state, "Удалить сервер")
			.child(delete_button);

		let confirm_delete_modal = if *is_confirm_delete_open.read() {
			let mut close_setter = is_confirm_delete_open;
			let inst_id = inst_id_del.clone();
			let addr = server_addr_del.clone();
			let fs = front_state_del.clone();
			let rm_mut = remove_server_mutation;
			let name = server_name_del.clone();

			Some(
				Modal::simple_confirm(
					"Удалить сервер",
					"trash",
					rect().expanded().center().padding(theme.gap2).child(
						label()
							.text(format!(
								"Вы уверены, что хотите удалить «{name}» ({addr}) из списка серверов?"
							))
							.font_size(theme.font0)
							.color(theme.fg),
					),
					true,
					move |_| close_setter.set(false),
					move |_| {
						close_setter.set(false);
						rm_mut.mutate(RemoveServerParams {
							instance_id: inst_id.clone(),
							address: addr.clone(),
						});
						fs.write().toast(Toast::success("Сервер удалён"));
					},
				)
				.into_element(),
			)
		} else {
			None
		};

		let instance_id = self.instance_id.clone();
		let server_addr = self.address.clone();
		let launch_button = Button::new().child("Играть").on_press(move |_| {
			launch_mutation.mutate(LaunchInstanceParams {
				id: instance_id.clone(),
				offline: false,
				quick_play: QuickPlayType::Server {
					server: server_addr.clone(),
					port: None,
				},
			});
		});

		let is_ip_hovered = use_state(|| false);
		let addr_to_copy = self.address.clone();
		let front_state_ip = front_state.clone();

		let ip_badge = rect()
			.horizontal()
			.cross_align(Alignment::Center)
			.spacing(4.0)
			.background(if *is_ip_hovered.read() {
				theme.item_select
			} else {
				theme.item_hover
			})
			.corner_radius(theme.round)
			.padding(Gaps::new(1.0, 6.0, 1.0, 6.0))
			.clickable()
			.hover(is_ip_hovered)
			.tip(&front_state, "Нажмите, чтобы скопировать IP")
			.child(
				label()
					.text(self.address.clone())
					.font_size(theme.font0)
					.color(if *is_ip_hovered.read() {
						theme.fg
					} else {
						theme.fg3
					}),
			)
			.child(icon("copy", 11.0))
			.on_press(move |_| {
				let _ = Clipboard::set(addr_to_copy.clone());
				front_state_ip
					.write()
					.toast(Toast::success("IP скопирован"));
			});

		let is_badge_hovered = use_state(|| false);
		let mut re_ping_setter = manual_refresh;

		let status_badge = match &current_ping {
			ServerPingState::Loading => rect()
				.horizontal()
				.cross_align(Alignment::Center)
				.spacing(4.0)
				.padding(Gaps::new(2.0, 6.0, 2.0, 6.0))
				.background(theme.item_hover)
				.corner_radius(theme.round)
				.child(label().text("●").font_size(theme.font0).color(theme.fg3))
				.child(
					label()
						.text("Пинг...")
						.font_size(theme.font0)
						.color(theme.fg3),
				),
			ServerPingState::Online { latency_ms, .. } => {
				let dot_color = if *latency_ms < 100 {
					Color::from_rgb(46, 204, 113)
				} else if *latency_ms < 250 {
					Color::from_rgb(241, 196, 15)
				} else {
					Color::from_rgb(231, 76, 60)
				};
				rect()
					.horizontal()
					.cross_align(Alignment::Center)
					.spacing(4.0)
					.padding(Gaps::new(2.0, 7.0, 2.0, 7.0))
					.background(if *is_badge_hovered.read() {
						theme.item_select
					} else {
						theme.item_hover
					})
					.corner_radius(theme.round)
					.clickable()
					.hover(is_badge_hovered)
					.tip(&front_state, "Нажмите, чтобы обновить пинг")
					.child(label().text("●").font_size(theme.font0).color(dot_color))
					.child(
						label()
							.text(format!("{latency_ms} ms"))
							.font_size(theme.font0)
							.font_weight(FontWeight::BOLD)
							.color(dot_color),
					)
					.on_press(move |_| {
						re_ping_setter.set(*re_ping_setter.peek() + 1);
					})
			}
			ServerPingState::Offline { .. } => rect()
				.horizontal()
				.cross_align(Alignment::Center)
				.spacing(4.0)
				.padding(Gaps::new(2.0, 6.0, 2.0, 6.0))
				.background(if *is_badge_hovered.read() {
					theme.item_select
				} else {
					theme.item_hover
				})
				.corner_radius(theme.round)
				.clickable()
				.hover(is_badge_hovered)
				.tip(&front_state, "Нажмите для повторной попытки")
				.child(
					label()
						.text("●")
						.font_size(theme.font0)
						.color(Color::from_rgb(231, 76, 60)),
				)
				.child(
					label()
						.text("Офлайн")
						.font_size(theme.font0)
						.color(Color::from_rgb(231, 76, 60)),
				)
				.on_press(move |_| {
					re_ping_setter.set(*re_ping_setter.peek() + 1);
				}),
		};

		let sub_line = match &current_ping {
			ServerPingState::Online {
				motd,
				players_online,
				players_max,
				player_sample,
				version,
				..
			} => {
				let motd_child = if let Some(m) = motd {
					render_motd(m, &theme)
				} else {
					label()
						.text("Сервер онлайн")
						.font_size(theme.font0)
						.color(theme.fg2)
						.into_element()
				};

				let ver_chip = version
					.as_ref()
					.filter(|v| !v.is_empty() && *v != "Unknown")
					.map(|v| {
						rect()
							.background(theme.item_hover)
							.corner_radius(theme.round)
							.padding(Gaps::new(1.0, 5.0, 1.0, 5.0))
							.child(
								label()
									.text(v.clone())
									.font_size(theme.font0)
									.color(theme.fg3),
							)
							.into_element()
					});

				let players_chip = if *players_max > 0 {
					let count_str = format!("{players_online}/{players_max}");
					let mut counter_rect = rect()
						.horizontal()
						.cross_align(Alignment::Center)
						.spacing(3.0)
						.background(theme.item_hover)
						.corner_radius(theme.round)
						.padding(Gaps::new(1.0, 5.0, 1.0, 5.0))
						.child(icon("multiple_users", 12.0))
						.child(
							label()
								.text(count_str)
								.font_size(theme.font0)
								.color(theme.fg2),
						);

					if !player_sample.is_empty() {
						let mut tip_lines =
							vec![format!("Игроки онлайн ({players_online}/{players_max}):")];
						for p in player_sample.iter().take(15) {
							tip_lines.push(format!("• {}", p.name));
						}
						if player_sample.len() < (*players_online as usize) {
							let diff = (*players_online as usize) - player_sample.len();
							tip_lines.push(format!("...и ещё {diff}"));
						}
						counter_rect = counter_rect.tip(&front_state, &tip_lines.join("\n"));
					}
					Some(counter_rect.into_element())
				} else {
					None
				};

				rect()
					.width(Size::fill())
					.horizontal()
					.flex()
					.cross_align(Alignment::Center)
					.spacing(theme.gap)
					.child(
						rect()
							.width(Size::flex(1.0))
							.overflow(Overflow::Clip)
							.child(motd_child),
					)
					.maybe(ver_chip.is_some(), |this| this.child(ver_chip.unwrap()))
					.maybe(players_chip.is_some(), |this| {
						this.child(players_chip.unwrap())
					})
					.into_element()
			}
			ServerPingState::Offline { reason: Some(r) } => {
				let err = format!("Недоступен: {r}");
				clip_text(&err)
					.font_size(theme.font0)
					.color(theme.fg3)
					.into_element()
			}
			ServerPingState::Offline { .. } => clip_text("Не удалось подключиться")
				.font_size(theme.font0)
				.color(theme.fg3)
				.into_element(),
			ServerPingState::Loading => clip_text("Проверка статуса...")
				.font_size(theme.font0)
				.color(theme.fg3)
				.into_element(),
		};

		rect()
			.width(Size::fill())
			.height(Size::px(74.0))
			.border(theme.border(theme.panel_border))
			.corner_radius(theme.round)
			.hover(is_hovered)
			.panel_colorway(&theme, *is_hovered.read(), false)
			.padding(theme.gap2)
			.spacing(theme.gap2)
			.cont()
			.cross_align(Alignment::Center)
			.overflow(Overflow::Clip)
			.child(ico)
			.child(
				rect()
					.width(Size::flex(1.0))
					.height(Size::fill())
					.main_align(Alignment::Center)
					.spacing(4.0)
					.overflow(Overflow::Clip)
					.child(
						rect()
							.width(Size::fill())
							.horizontal()
							.cross_align(Alignment::Center)
							.spacing(theme.gap)
							.child(
								label()
									.text(self.name.clone())
									.font_size(theme.font)
									.font_weight(FontWeight::BOLD)
									.color(theme.fg),
							)
							.child(ip_badge)
							.child(status_badge),
					)
					.child(sub_line),
			)
			.child(delete_button_tipped)
			.child(launch_button)
			.maybe(confirm_delete_modal.is_some(), |this| {
				this.child(confirm_delete_modal.unwrap())
			})
	}
}

#[derive(PartialEq)]
pub struct InstanceServersView {
	pub id: String,
}

impl Component for InstanceServersView {
	fn render(&self) -> impl IntoElement {
		let theme = use_theme();
		let front_state = use_front_state();
		let back_state = use_consume::<BackState>();
		let files_query = use_query(Query::new(
			self.id.clone(),
			FetchInstanceFiles::new(back_state.clone()),
		));
		let files = files_query.read().state().ok().cloned().unwrap_or_default();

		let refresh_trigger = use_state(|| 0usize);
		let current_refresh = *refresh_trigger.read();
		let mut refresh_setter = refresh_trigger;

		// Feature 5: Auto-ping in background every 45s while Servers tab is open
		let mut auto_refresh_setter = refresh_trigger;
		use_future(move || async move {
			loop {
				tokio::time::sleep(Duration::from_secs(45)).await;
				auto_refresh_setter.set(*auto_refresh_setter.peek() + 1);
			}
		});

		let is_add_modal_open = use_state(|| false);
		let mut open_add_modal_setter = is_add_modal_open;
		let add_server_name = use_state(String::new);
		let add_server_addr = use_state(String::new);
		let add_server_mutation = use_mutation(AddInstanceServer::new(back_state));

		let add_modal = if *is_add_modal_open.read() {
			let mut close_setter = is_add_modal_open;
			let mut name_state = add_server_name;
			let mut addr_state = add_server_addr;
			let add_mut = add_server_mutation;
			let inst_id = self.id.clone();
			let fs = front_state.clone();
			let q = files_query;

			let body = rect()
				.width(Size::fill())
				.padding(theme.gap3)
				.spacing(theme.gap2)
				.child(
					label()
						.text("Название сервера")
						.font_size(theme.font0)
						.color(theme.fg2),
				)
				.child(TextInput::new(add_server_name).placeholder("Minecraft Server"))
				.child(
					label()
						.text("Адрес сервера (IP:порт или домен)")
						.font_size(theme.font0)
						.color(theme.fg2),
				)
				.child(TextInput::new(add_server_addr).placeholder("mc.example.com"));

			Some(
				Modal::new("Добавить сервер".into(), "server".into())
					.size(520.0, 320.0)
					.on_close(move |_| close_setter.set(false))
					.cancel_button()
					.button(ModalButton {
						title: "Добавить".into(),
						icon: "plus".into(),
						active: true,
						danger: false,
						on_click: (move |_| {
							let addr = addr_state.read().trim().to_string();
							if addr.is_empty() {
								fs.write()
									.toast(Toast::warning("Введите адрес сервера", None));
								return;
							}
							let raw_name = name_state.read().trim().to_string();
							let name = if raw_name.is_empty() {
								addr.clone()
							} else {
								raw_name
							};

							add_mut.mutate(AddServerParams {
								instance_id: inst_id.clone(),
								name,
								address: addr,
							});

							close_setter.set(false);
							name_state.set(String::new());
							addr_state.set(String::new());
							q.invalidate();
							fs.write().toast(Toast::success("Сервер добавлен"));
						})
						.into(),
					})
					.child(body)
					.into_element(),
			)
		} else {
			None
		};

		let server_count = files.servers.len();
		let servers = files.servers.into_iter().map(|x| {
			ServerCard {
				icon: x.icon_png.map(|p| ImageSource::from(Bytes::from(p))),
				name: x.name.unwrap_or_else(|| x.address.clone()),
				address: x.address,
				instance_id: self.id.clone(),
				refresh_id: current_refresh,
			}
			.into_element()
		});

		let scroll = ScrollView::new()
			.width(Size::fill())
			.height(Size::fill())
			.spacing(theme.gap2)
			.children(servers);

		let header = rect()
			.width(Size::fill())
			.horizontal()
			.cross_align(Alignment::Center)
			.child(
				rect()
					.width(Size::flex(1.0))
					.horizontal()
					.cross_align(Alignment::Center)
					.spacing(theme.gap)
					.child(icon("server", 20.0))
					.child(
						label()
							.text("Серверы")
							.font_size(theme.font2)
							.font_weight(FontWeight::BOLD)
							.color(theme.fg),
					)
					.child(number_indicator(server_count, &theme)),
			)
			.child(
				rect()
					.horizontal()
					.cross_align(Alignment::Center)
					.spacing(theme.gap)
					.child(
						Button::new()
							.child(icon("plus", 14.0))
							.child("+ Добавить сервер")
							.on_press(move |_| {
								open_add_modal_setter.set(true);
							}),
					)
					.child(
						Button::new()
							.child(icon("refresh", 14.0))
							.child("Обновить")
							.on_press(move |_| {
								refresh_setter.set(*refresh_setter.peek() + 1);
							}),
					),
			);

		let content = if server_count == 0 {
			rect()
				.width(Size::fill())
				.height(Size::fill())
				.center()
				.spacing(theme.gap2)
				.child(icon("server", 48.0))
				.child(
					label()
						.text("Список серверов пуст")
						.font_size(theme.font)
						.font_weight(FontWeight::BOLD)
						.color(theme.fg),
				)
				.child(
					label()
						.text("Нажмите «+ Добавить сервер» или добавьте серверы в самой игре")
						.font_size(theme.font0)
						.color(theme.fg3),
				)
				.into_element()
		} else {
			scroll.into_element()
		};

		rect()
			.width(Size::fill())
			.height(Size::fill())
			.padding(theme.gap3)
			.spacing(theme.gap2)
			.child(header)
			.child(content)
			.maybe(add_modal.is_some(), |this| this.child(add_modal.unwrap()))
	}
}
