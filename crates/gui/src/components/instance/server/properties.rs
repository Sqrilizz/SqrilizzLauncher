use std::collections::HashMap;

use crate::{
	components::{
		border_bottom,
		icon_text_button,
		input::{switch::Switch, text::TextInput},
	},
	ops::QueryOption,
	ops::instance::{
		InstanceFileKey, ReadInstanceFile, WriteInstanceFile, WriteInstanceFileParams,
	},
	prelude::*,
};

#[derive(PartialEq)]
pub struct ServerProperties {
	pub id: String,
}

impl Component for ServerProperties {
	fn render(&self) -> impl IntoElement {
		let theme = use_theme();
		let back_state = use_consume::<BackState>();

		let file_key =
			InstanceFileKey::new(self.id.clone(), String::new()).child("server.properties");
		let read_query = use_query(Query::new(
			QueryOption(Some(file_key.clone())),
			ReadInstanceFile::new(back_state.clone()),
		));
		let write_mutation = use_mutation(Mutation::new(
			WriteInstanceFile::new(back_state.clone()).toast(
				&back_state,
				Some("Параметры сервера сохранены"),
				"Не удалось сохранить параметры",
			),
		));

		// Reactive property states
		let motd = use_state(|| "A Minecraft Server".to_string());
		let port = use_state(|| "25565".to_string());
		let max_players = use_state(|| "20".to_string());
		let online_mode = use_state(|| true);
		let whitelist = use_state(|| false);
		let pvp = use_state(|| true);
		let difficulty = use_state(|| "normal".to_string());
		let gamemode = use_state(|| "survival".to_string());
		let allow_flight = use_state(|| false);
		let command_blocks = use_state(|| true);
		let hardcore = use_state(|| false);
		let view_distance = use_state(|| "10".to_string());
		let simulation_distance = use_state(|| "10".to_string());
		let spawn_protection = use_state(|| "16".to_string());
		let level_name = use_state(|| "world".to_string());
		let level_seed = use_state(String::new);

		let raw_file_content = use_state(String::new);
		let is_loaded = use_state(|| false);

		// Synchronize parsed values when file loads
		let mut motd_set = motd;
		let mut port_set = port;
		let mut max_players_set = max_players;
		let mut online_mode_set = online_mode;
		let mut whitelist_set = whitelist;
		let mut pvp_set = pvp;
		let mut difficulty_set = difficulty;
		let mut gamemode_set = gamemode;
		let mut allow_flight_set = allow_flight;
		let mut command_blocks_set = command_blocks;
		let mut hardcore_set = hardcore;
		let mut view_distance_set = view_distance;
		let mut simulation_distance_set = simulation_distance;
		let mut spawn_protection_set = spawn_protection;
		let mut level_name_set = level_name;
		let mut level_seed_set = level_seed;
		let mut raw_content_set = raw_file_content;
		let mut loaded_set = is_loaded;

		use_side_effect(move || {
			if let Some(content) = read_query.read().state().ok().cloned().flatten() {
				if !*loaded_set.peek() || raw_content_set.peek().as_str() != content.as_ref() {
					raw_content_set.set(content.to_string());
					loaded_set.set(true);

					for line in content.lines() {
						let trimmed = line.trim();
						if trimmed.starts_with('#') || !trimmed.contains('=') {
							continue;
						}
						let Some((k, v)) = trimmed.split_once('=') else {
							continue;
						};
						let k = k.trim();
						let v = v.trim();
						match k {
							"motd" => motd_set.set(v.to_string()),
							"server-port" => port_set.set(v.to_string()),
							"max-players" => max_players_set.set(v.to_string()),
							"online-mode" => online_mode_set.set(v == "true"),
							"white-list" => whitelist_set.set(v == "true"),
							"pvp" => pvp_set.set(v == "true"),
							"difficulty" => difficulty_set.set(v.to_string()),
							"gamemode" => gamemode_set.set(v.to_string()),
							"allow-flight" => allow_flight_set.set(v == "true"),
							"enable-command-block" => command_blocks_set.set(v == "true"),
							"hardcore" => hardcore_set.set(v == "true"),
							"view-distance" => view_distance_set.set(v.to_string()),
							"simulation-distance" => simulation_distance_set.set(v.to_string()),
							"spawn-protection" => spawn_protection_set.set(v.to_string()),
							"level-name" => level_name_set.set(v.to_string()),
							"level-seed" => level_seed_set.set(v.to_string()),
							_ => {}
						}
					}
				}
			}
		});

		// Header Save Bar
		let save_mut = write_mutation;
		let inst_id = self.id.clone();
		let raw_snapshot = raw_file_content.read().clone();

		let save_btn = icon_text_button("check", "Сохранить параметры", &theme)
			.active(&theme)
			.border_fill(theme.primary)
			.background(theme.primary_bg)
			.color(theme.primary)
			.on_press(move |_| {
				let mut lines = Vec::new();
				let mut handled_keys = HashMap::new();

				let new_values: [(&str, String); 16] = [
					("motd", motd.read().clone()),
					("server-port", port.read().clone()),
					("max-players", max_players.read().clone()),
					("online-mode", online_mode.read().to_string()),
					("white-list", whitelist.read().to_string()),
					("pvp", pvp.read().to_string()),
					("difficulty", difficulty.read().clone()),
					("gamemode", gamemode.read().clone()),
					("allow-flight", allow_flight.read().to_string()),
					("enable-command-block", command_blocks.read().to_string()),
					("hardcore", hardcore.read().to_string()),
					("view-distance", view_distance.read().clone()),
					("simulation-distance", simulation_distance.read().clone()),
					("spawn-protection", spawn_protection.read().clone()),
					("level-name", level_name.read().clone()),
					("level-seed", level_seed.read().clone()),
				];

				for (k, v) in &new_values {
					handled_keys.insert(*k, v.clone());
				}

				if !raw_snapshot.is_empty() {
					for line in raw_snapshot.lines() {
						let trimmed = line.trim();
						if trimmed.starts_with('#') || !trimmed.contains('=') {
							lines.push(line.to_string());
							continue;
						}
						if let Some((k, _)) = trimmed.split_once('=') {
							let k = k.trim();
							if let Some(new_val) = handled_keys.remove(k) {
								lines.push(format!("{k}={new_val}"));
							} else {
								lines.push(line.to_string());
							}
						} else {
							lines.push(line.to_string());
						}
					}
				}

				// Append any remaining keys
				for (k, v) in handled_keys {
					lines.push(format!("{k}={v}"));
				}

				let new_content = lines.join("\n") + "\n";
				save_mut.mutate(WriteInstanceFileParams {
					instance_id: inst_id.clone(),
					path: "server.properties".to_string(),
					contents: new_content,
				});
			});

		let toolbar = rect()
			.width(Size::fill())
			.height(Size::px(56.0))
			.background(theme.item)
			.border(border_bottom(theme.border, theme.panel_border))
			.padding(Gaps::new(0.0, theme.gap2, 0.0, theme.gap2))
			.horizontal()
			.cross_align(Alignment::Center)
			.main_align(Alignment::SpaceBetween)
			.child(
				rect()
					.horizontal()
					.cross_align(Alignment::Center)
					.spacing(8.0)
					.child(icon("properties", 18.0).color(theme.primary))
					.child(
						label()
							.text("Параметры сервера (server.properties)")
							.font_size(theme.font)
							.font_weight(FontWeight::BOLD)
							.color(theme.fg),
					),
			)
			.child(save_btn);

		// Helper card maker
		let section_header = |title: &'static str, ico: &'static str| {
			rect()
				.horizontal()
				.cross_align(Alignment::Center)
				.spacing(8.0)
				.padding(Gaps::new(12.0, 0.0, 4.0, 0.0))
				.child(icon(ico, 14.0).color(theme.primary))
				.child(
					label()
						.text(title)
						.font_size(theme.font)
						.font_weight(FontWeight::BOLD)
						.color(theme.fg),
				)
		};

		let field_card = |title: &'static str, desc: &'static str, input_element: Element| {
			rect()
				.width(Size::fill())
				.background(theme.item)
				.border(theme.border(theme.panel_border))
				.corner_radius(theme.round)
				.padding(Gaps::new(10.0, 14.0, 10.0, 14.0))
				.horizontal()
				.cross_align(Alignment::Center)
				.main_align(Alignment::SpaceBetween)
				.child(
					rect()
						.flex()
						.spacing(2.0)
						.child(
							label()
								.text(title)
								.font_size(theme.font0)
								.font_weight(FontWeight::BOLD)
								.color(theme.fg),
						)
						.child(label().text(desc).font_size(11.0).color(theme.fg3)),
				)
				.child(
					rect()
						.cross_align(Alignment::End)
						.child(input_element),
				)
		};

		let text_box = |input: Element| {
			rect()
				.width(Size::px(260.0))
				.child(input)
				.into_element()
		};

		let toggle_card = |title: &'static str,
		                   desc: &'static str,
		                   is_active: bool,
		                   on_tgl: EventHandler<()>| {
			rect()
				.width(Size::fill())
				.background(theme.item)
				.border(theme.border(theme.panel_border))
				.corner_radius(theme.round)
				.padding(Gaps::new(10.0, 14.0, 10.0, 14.0))
				.horizontal()
				.cross_align(Alignment::Center)
				.main_align(Alignment::SpaceBetween)
				.child(
					rect()
						.flex()
						.spacing(2.0)
						.child(
							label()
								.text(title)
								.font_size(theme.font0)
								.font_weight(FontWeight::BOLD)
								.color(theme.fg),
						)
						.child(label().text(desc).font_size(11.0).color(theme.fg3)),
				)
				.child(Switch {
					enabled: is_active,
					on_toggle: on_tgl,
				})
		};

		// Option Pills helper
		let option_pills = |current: String,
		                    options: &[(&'static str, &'static str)],
		                    mut setter: State<String>| {
			let mut container = rect().horizontal().spacing(6.0);
			for (val, label_text) in options {
				let active = current == *val;
				let val_str = val.to_string();
				let pill = rect()
					.height(Size::px(30.0))
					.padding(Gaps::new(0.0, 12.0, 0.0, 12.0))
					.background(if active {
						theme.primary
					} else {
						theme.item_hover
					})
					.border(theme.border(if active {
						theme.primary
					} else {
						theme.panel_border
					}))
					.corner_radius(theme.round)
					.center()
					.clickable()
					.on_press(move |_| setter.set(val_str.clone()))
					.child(
						label()
							.text(*label_text)
							.font_size(12.0)
							.font_weight(if active {
								FontWeight::BOLD
							} else {
								FontWeight::NORMAL
							})
							.color(if active { Color::WHITE } else { theme.fg }),
					);
				container = container.child(pill);
			}
			container.into_element()
		};

		// Toggles
		let mut onl_set = online_mode;
		let online_mode_card = toggle_card(
			"Проверка лицензии (Online Mode)",
			"Выключите, если у игроков пиратский лаунчер или сторонний аккаунт",
			*online_mode.read(),
			(move |_| onl_set.set(!*onl_set.peek())).into(),
		);

		let mut wl_set = whitelist;
		let whitelist_card = toggle_card(
			"Белый список (White-list)",
			"Разрешать вход только игрокам из файла whitelist.json",
			*whitelist.read(),
			(move |_| wl_set.set(!*wl_set.peek())).into(),
		);

		let mut pvp_set_fn = pvp;
		let pvp_card = toggle_card(
			"Сражения (PVP)",
			"Разрешить игрокам наносить урон друг другу",
			*pvp.read(),
			(move |_| pvp_set_fn.set(!*pvp_set_fn.peek())).into(),
		);

		let mut fly_set = allow_flight;
		let allow_flight_card = toggle_card(
			"Разрешить полёт (Allow Flight)",
			"Отключает автоматический кик игроков за полет (нужно для модов/элитр)",
			*allow_flight.read(),
			(move |_| fly_set.set(!*fly_set.peek())).into(),
		);

		let mut cmd_set = command_blocks;
		let command_blocks_card = toggle_card(
			"Командные блоки (Command Blocks)",
			"Включить исполнение команд в командных блоках",
			*command_blocks.read(),
			(move |_| cmd_set.set(!*cmd_set.peek())).into(),
		);

		let mut hc_set = hardcore;
		let hardcore_card = toggle_card(
			"Хардкор (Hardcore)",
			"После смерти игрок навсегда переводится в режим наблюдателя",
			*hardcore.read(),
			(move |_| hc_set.set(!*hc_set.peek())).into(),
		);

		// Pills for Gamemode & Difficulty
		let gamemode_card = field_card(
			"Режим игры (Gamemode)",
			"Стандартный режим игры для подключающихся игроков",
			option_pills(
				gamemode.read().clone(),
				&[
					("survival", "Выживание"),
					("creative", "Творчество"),
					("adventure", "Приключение"),
					("spectator", "Наблюдатель"),
				],
				gamemode,
			),
		);

		let difficulty_card = field_card(
			"Сложность (Difficulty)",
			"Уровень сложности игрового процесса на сервере",
			option_pills(
				difficulty.read().clone(),
				&[
					("peaceful", "Мирный"),
					("easy", "Легкий"),
					("normal", "Норма"),
					("hard", "Сложный"),
				],
				difficulty,
			),
		);

		// Text inputs
		let motd_card = field_card(
			"Описание сервера (MOTD)",
			"Текст, отображаемый под сервером в списке серверов Minecraft",
			text_box(
				TextInput::new(motd)
					.placeholder("A Minecraft Server")
					.into_element(),
			),
		);

		let port_card = field_card(
			"Порт сервера (Port)",
			"Сетевой порт сервера (стандартный для Java: 25565)",
			text_box(
				TextInput::new(port)
					.placeholder("25565")
					.into_element(),
			),
		);

		let max_players_card = field_card(
			"Максимум игроков (Max Players)",
			"Лимит одновременных подключений на сервере",
			text_box(
				TextInput::new(max_players)
					.placeholder("20")
					.into_element(),
			),
		);

		let view_dist_card = field_card(
			"Дальность прорисовки (View Distance)",
			"Радиус отправки чанков клиентам (влияет на нагрузку ЦП и ОЗУ)",
			text_box(
				TextInput::new(view_distance)
					.placeholder("10")
					.into_element(),
			),
		);

		let sim_dist_card = field_card(
			"Дальность симуляции (Simulation Distance)",
			"Радиус тикающих мобов и механизмов вокруг игроков",
			text_box(
				TextInput::new(simulation_distance)
					.placeholder("10")
					.into_element(),
			),
		);

		let spawn_prot_card = field_card(
			"Защита спавна (Spawn Protection)",
			"Радиус неразрушаемой зоны вокруг точки спавна (в блоках)",
			text_box(
				TextInput::new(spawn_protection)
					.placeholder("16")
					.into_element(),
			),
		);

		let level_name_card = field_card(
			"Папка мира (Level Name)",
			"Название директории мира в папке сервера (по умолчанию world)",
			text_box(
				TextInput::new(level_name)
					.placeholder("world")
					.into_element(),
			),
		);

		let level_seed_card = field_card(
			"Сид генерации (Seed)",
			"Ключ генерации нового мира (оставьте пустым для случайного)",
			text_box(
				TextInput::new(level_seed)
					.placeholder("Случайный сид")
					.into_element(),
			),
		);

		// Layout
		rect().expanded().flex().child(toolbar).child(
			ScrollView::new()
				.width(Size::fill())
				.height(Size::fill())
				.child(
					rect()
						.width(Size::fill())
						.padding(theme.gap3)
						.spacing(theme.gap2)
						.child(section_header("ОСНОВНЫЕ ПАРАМЕТРЫ", "properties"))
						.child(motd_card)
						.child(port_card)
						.child(max_players_card)
						.child(online_mode_card)
						.child(whitelist_card)
						.child(section_header("ИГРОВОЙ ПРОЦЕСС", "minecraft"))
						.child(gamemode_card)
						.child(difficulty_card)
						.child(pvp_card)
						.child(hardcore_card)
						.child(allow_flight_card)
						.child(command_blocks_card)
						.child(section_header("МИР И ПРОИЗВОДИТЕЛЬНОСТЬ", "globe"))
						.child(view_dist_card)
						.child(sim_dist_card)
						.child(spawn_prot_card)
						.child(level_name_card)
						.child(level_seed_card),
				),
		)
	}
}
