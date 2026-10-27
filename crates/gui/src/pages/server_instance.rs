use std::rc::Rc;

use sqrilizz::{
	config_crate::{ConfigKind, instance::InstanceConfig, template::TemplateConfig},
	core::QuickPlayType,
	shared::{
		loaders::Loader,
		versions::{MinecraftLatestVersion, MinecraftVersionDeser},
	},
};

use crate::ops::server::write_console_command;

use crate::{
	components::{
		input::{select::Selected, tabs::SideTabs},
		instance::server::{
			files::ServerFiles, overview::ServerOverview, players::ServerPlayers,
			properties::ServerProperties, worlds::ServerWorlds,
		},
	},
	ops::{
		instance::{
			DuplicateInstance, FetchInstanceConfig, FetchParentConfigs, SaveConfig, UpdateInstance,
		},
		launch::{
			FetchInstanceRunState, InstanceRunState, KillInstance, LaunchInstance,
			LaunchInstanceParams, RestartInstance,
		},
		misc::{ShowDirectory, ShowDirectoryOption},
	},
	pages::config::{ConfigState, ConfiguredItem, content::ContentConfig},
	prelude::*,
	state::ModalType,
	util::{PtrEq, gif::AnimatedIconViewer},
};

#[derive(PartialEq)]
pub struct ServerInstancePage {
	pub id: String,
}

impl Component for ServerInstancePage {
	fn render(&self) -> impl IntoElement {
		let theme = use_theme();
		let front_state = use_front_state();
		let back_state = use_consume::<BackState>();

		let config_query = use_query(FetchInstanceConfig::new(
			self.id.clone(),
			back_state.clone(),
		));
		let duplicate_instance =
			use_mutation(Mutation::new(DuplicateInstance::new(back_state.clone())));
		let save_config = use_mutation(Mutation::new(SaveConfig::new(back_state.clone()).toast(
			&back_state,
			Some("Сохранено"),
			"Не удалось сохранить конфигурацию",
		)));
		let update = use_mutation(Mutation::new(UpdateInstance::new(back_state.clone())));
		let _ = &update;
		let launch = use_mutation(LaunchInstance::new(back_state.clone()));
		let kill = use_mutation(KillInstance::new(back_state.clone()));
		let restart = use_mutation(Mutation::new(RestartInstance::new(back_state.clone())));
		let run_state = use_query(Query::new(
			self.id.clone(),
			FetchInstanceRunState::new(back_state.clone()),
		));
		let is_running = run_state.read().state().ok().cloned() == Some(InstanceRunState::Running);
		let mut is_starting = use_state(|| false);
		if is_running && *is_starting.read() {
			is_starting.set(false);
		}
		let show_directory = use_mutation(Mutation::new(ShowDirectory::new(back_state.clone())));

		let tab = use_state(|| ServerTab::Console);
		let config = use_state(InstanceConfig::default);

		let id = self.id.clone();
		let config_state = ConfigState::new(ConfigKind::Instance, false);
		let mut config_state2 = config_state.clone();
		let mut config2 = config;
		let _tab2 = tab;
		use_side_effect(move || {
			let config = config_query
				.read()
				.state()
				.ok()
				.cloned()
				.flatten()
				.unwrap_or_default();

			let template_config = TemplateConfig {
				instance: config.no_templates,
				..Default::default()
			};

			config_state2.update(Some(id.clone()), template_config);
			config2.set(config.main);
		});

		let parent_configs = use_query(FetchParentConfigs::new(
			config_state.from.read().cloned(),
			back_state.clone(),
		));
		let parent_configs = parent_configs
			.read()
			.state()
			.ok()
			.cloned()
			.unwrap_or_default();
		config_state
			.parent_configs
			.clone()
			.set_if_modified(PtrEq(parent_configs.clone()));

		let name = config
			.read()
			.name
			.clone()
			.unwrap_or_else(|| self.id.clone());

		let head = rect()
			.width(Size::fill())
			.height(Size::px(60.0))
			.border(border_bottom(theme.border, theme.panel_border))
			.background(theme.panel)
			.horizontal()
			.cross_align(Alignment::Center);

		let path = config.read().icon.clone();
		let icon = if path.is_some() {
			AnimatedIconViewer::new(path).into_element()
		} else {
			icon("server", 24.0).color(theme.primary).into_element()
		};

		let icon = rect()
			.center()
			.width(Size::px(60.0))
			.height(Size::px(60.0))
			.border(border_right(theme.border, theme.panel_border))
			.child(icon);

		let head = head.child(icon);

		let info = rect()
			.height(Size::fill())
			.vertical()
			.main_align(Alignment::Center)
			.padding(Gaps::new(0.0, theme.gap2, 0.0, theme.gap2));

		let name = label()
			.font_size(theme.font2)
			.font_weight(FontWeight::BOLD)
			.color(theme.fg)
			.text(name);

		let loader_str = config.read().loader.clone().unwrap_or_default();
		let loader = Loader::parse_from_str(&loader_str);
		let is_plugins = matches!(
			loader,
			Loader::Paper
				| Loader::CraftBukkit
				| Loader::Spigot
				| Loader::Purpur
				| Loader::Pufferfish
				| Loader::Folia
		);
		let content_tab_name = if is_plugins {
			"Плагины"
		} else {
			"Моды"
		};

		let mut desc = String::new();
		if !loader_str.is_empty() {
			desc += &loader_str;
		}
		if let Some(ref ver) = config.read().version {
			if !desc.is_empty() {
				desc += " • ";
			}
			match ver {
				MinecraftVersionDeser::Latest(MinecraftLatestVersion::Release) => desc += "Релиз",
				MinecraftVersionDeser::Latest(MinecraftLatestVersion::Snapshot) => {
					desc += "Снапшот"
				}
				MinecraftVersionDeser::Version(v) => desc += v,
			}
		}

		let is_start = *is_starting.read() && !is_running;

		let status_badge = if is_running {
			rect()
				.horizontal()
				.cross_align(Alignment::Center)
				.spacing(6.0)
				.padding(Gaps::new(2.0, 8.0, 2.0, 8.0))
				.background(Color::from_argb(35, 16, 185, 129))
				.corner_radius(4.0)
				.child(
					rect()
						.width(Size::px(8.0))
						.height(Size::px(8.0))
						.corner_radius(4.0)
						.background(Color::from_rgb(16, 185, 129)),
				)
				.child(
					label()
						.text("В сети")
						.font_size(11.0)
						.font_weight(FontWeight::BOLD)
						.color(Color::from_rgb(16, 185, 129)),
				)
		} else if is_start {
			rect()
				.horizontal()
				.cross_align(Alignment::Center)
				.spacing(6.0)
				.padding(Gaps::new(2.0, 8.0, 2.0, 8.0))
				.background(Color::from_argb(35, 245, 158, 11))
				.corner_radius(4.0)
				.child(
					rect()
						.width(Size::px(8.0))
						.height(Size::px(8.0))
						.corner_radius(4.0)
						.background(Color::from_rgb(245, 158, 11)),
				)
				.child(
					label()
						.text("Запуск...")
						.font_size(11.0)
						.font_weight(FontWeight::BOLD)
						.color(Color::from_rgb(245, 158, 11)),
				)
		} else {
			rect()
				.horizontal()
				.cross_align(Alignment::Center)
				.spacing(6.0)
				.padding(Gaps::new(2.0, 8.0, 2.0, 8.0))
				.background(theme.item)
				.border(theme.border(theme.item_border))
				.corner_radius(4.0)
				.child(
					rect()
						.width(Size::px(8.0))
						.height(Size::px(8.0))
						.corner_radius(4.0)
						.background(theme.fg3),
				)
				.child(
					label()
						.text("Остановлен")
						.font_size(11.0)
						.color(theme.fg2),
				)
		};

		let title_row = rect()
			.horizontal()
			.cross_align(Alignment::Center)
			.spacing(theme.gap)
			.child(name)
			.child(status_badge);

		let desc = label().font_size(theme.font0).color(theme.fg2).text(desc);

		let info = info.child(title_row).child(desc);
		let head = head.child(info);

		let is_editable = !is_running;

		let id = self.id.clone();
		let id_launch = id.clone();
		let mut starting_setter = is_starting;
		let launch_button = if is_start {
			icon_text_button("clock", "Запуск сервера...", &theme)
				.border_fill(Color::from_rgb(245, 158, 11))
				.background(Color::from_argb(35, 245, 158, 11))
				.color(Color::from_rgb(245, 158, 11))
		} else {
			icon_text_button("play", "Запустить сервер", &theme)
				.active(&theme)
				.border_fill(Color::from_rgb(16, 185, 129))
				.background(Color::from_argb(35, 16, 185, 129))
				.color(Color::from_rgb(16, 185, 129))
				.hover_background(Color::from_argb(60, 16, 185, 129))
				.on_press(move |_| {
					starting_setter.set(true);
					launch.mutate(LaunchInstanceParams {
						id: id_launch.clone(),
						offline: false,
						quick_play: QuickPlayType::default(),
					});
				})
		};

		let id_restart = id.clone();
		let restart_button = icon_text_button("cycle", "Перезапуск", &theme)
			.border_fill(Color::from_rgb(245, 158, 11))
			.color(Color::from_rgb(245, 158, 11))
			.hover_background(theme.item_hover)
			.on_press(move |_| {
				restart.mutate(id_restart.clone());
			});

		let id_stop = id.clone();
		let bs_stop = back_state.clone();
		let fs_stop = front_state.clone();
		let stop_button = icon_text_button("stop", "Стоп", &theme)
			.border_fill(Color::from_rgb(239, 68, 68))
			.color(Color::from_rgb(239, 68, 68))
			.hover_background(theme.error_bg)
			.on_press(move |_| {
				let id = id_stop.clone();
				let bs = bs_stop.clone();
				let fs = fs_stop.clone();
				spawn(async move {
					fs.write().toast(Toast::info(
						"Остановка сервера... (отправлена команда stop)",
						None,
					));
					let _ = write_console_command(&bs, &id, "stop").await;
				});
			});

		let id_kill = id.clone();
		let kill_button = icon_text_button("cross", "Kill", &theme)
			.border_fill(Color::from_rgb(185, 28, 28))
			.color(Color::from_rgb(239, 68, 68))
			.hover_background(theme.error_bg)
			.on_press(move |_| {
				kill.mutate((id_kill.clone(), None));
			});

		let id_folder = id.clone();
		let folder_button = icon_text_button("folder", "Папка", &theme)
			.border_fill(theme.panel_border)
			.on_press(move |_| {
				show_directory.mutate(ShowDirectoryOption::Instance(id_folder.clone()));
			});

		let id_settings = id.clone();
		let front_state2 = front_state.clone();
		let settings_button = icon_text_button("gear", "Настройки", &theme)
			.border_fill(theme.panel_border)
			.on_press(move |_| {
				front_state2
					.write()
					.set_modal(Some(ModalType::Configuration(ConfiguredItem {
						ty: ConfigKind::Instance,
						id: Some(id_settings.clone()),
						is_new: false,
					})));
			});

		let id_more = id.clone();
		let front_state_more = front_state.clone();
		let more_dropdown = Dropdown::new(
			Selected::Single(MoreOption::More),
			Rc::new(move |selected| match selected.single() {
				MoreOption::More => {}
				MoreOption::Duplicate => {
					duplicate_instance.mutate(id_more.clone());
					front_state_more
						.write()
						.toast(Toast::success("Копия инстанса создается..."));
				}
				MoreOption::Delete => {
					front_state_more
						.write()
						.set_modal(Some(ModalType::DeleteInstance(id_more.clone())));
				}
			}),
		)
		.options_width(160.0)
		.align_options_right()
		.custom_header(SelectOption::new(MoreOption::More, "Ещё", Some("ellipsis")))
		.header_width(Size::auto())
		.hide_arrow()
		.panel_colorway()
		.child(SelectOption::new(
			MoreOption::Duplicate,
			"Дублировать",
			Some("copy"),
		))
		.child(SelectOption::new(
			MoreOption::Delete,
			"Удалить",
			Some("trash"),
		));

		let mut controls = rect()
			.height(Size::fill())
			.cont()
			.main_align(Alignment::End)
			.cross_align(Alignment::Center)
			.horizontal()
			.spacing(theme.gap)
			.padding(Gaps::new(0.0, theme.gap3, 0.0, 0.0));

		if is_running {
			controls = controls
				.child(restart_button)
				.child(stop_button)
				.child(kill_button);
		} else {
			controls = controls.child(launch_button);
		}

		controls = controls
			.child(folder_button)
			.child(if is_editable {
				settings_button.into_element()
			} else {
				rect().into_element()
			})
			.child(more_dropdown);

		let head = head.child(controls);

		// Pterodactyl-Style Subnavigation Tabs
		let tabs = SideTabs::from_state(tab)
			.child(SelectOption::new(
				ServerTab::Console,
				"Консоль",
				Some("text"),
			))
			.child(SelectOption::new(ServerTab::Files, "Файлы", Some("folder")))
			.child(SelectOption::new(
				ServerTab::Properties,
				"Параметры",
				Some("properties"),
			))
			.child(SelectOption::new(
				ServerTab::Plugins,
				content_tab_name,
				Some("honeycomb"),
			))
			.child(SelectOption::new(
				ServerTab::Players,
				"Игроки",
				Some("multiple_users"),
			))
			.child(SelectOption::new(
				ServerTab::Worlds,
				"Миры и бэкапы",
				Some("globe"),
			));
		let tabs = rect()
			.width(Size::flex(1.0))
			.height(Size::fill())
			.border(border_right(theme.border, theme.panel_border))
			.child(tabs);

		let config_state2 = config_state.clone();
		let save_fn = config_state.save_fn(front_state.clone(), save_config);
		let contents = match &*tab.read() {
			ServerTab::Console => ServerOverview {
				id: self.id.clone(),
			}
			.into_element(),
			ServerTab::Files => ServerFiles {
				id: self.id.clone(),
			}
			.into_element(),
			ServerTab::Properties => ServerProperties {
				id: self.id.clone(),
			}
			.into_element(),
			ServerTab::Plugins => ContentConfig {
				config_state: config_state.clone(),
				parent_configs: PtrEq(parent_configs.clone()),
				on_edit: Some(
					(move |_| {
						if config_state2.has_changed() {
							save_fn();
						}
					})
					.into(),
				),
			}
			.into_element(),
			ServerTab::Players => ServerPlayers {
				id: self.id.clone(),
			}
			.into_element(),
			ServerTab::Worlds => ServerWorlds {
				id: self.id.clone(),
			}
			.into_element(),
		};
		let contents = rect()
			.width(Size::flex(5.0))
			.height(Size::fill())
			.child(contents);

		let body = rect()
			.width(Size::fill())
			.height(Size::flex(1.0))
			.flex()
			.horizontal()
			.child(tabs)
			.child(contents);

		rect().expanded().flex().child(head).child(body)
	}
}

#[derive(PartialEq, Clone)]
enum ServerTab {
	Console,
	Files,
	Properties,
	Plugins,
	Players,
	Worlds,
}

#[derive(PartialEq, Clone)]
enum MoreOption {
	More,
	Duplicate,
	Delete,
}
