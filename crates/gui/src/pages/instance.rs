use std::rc::Rc;

use sqrilizz::{
	config_crate::{ConfigKind, instance::InstanceConfig, template::TemplateConfig},
	core::QuickPlayType,
	shared::{Side, id::InstanceID},
};

use crate::{
	components::{
		input::{select::Selected, tabs::SideTabs},
		instance::{
			console::InstanceConsole, files::InstanceFilesView, servers::InstanceServersView,
		},
	},
	ops::{
		instance::{
			DuplicateInstance, FetchInstanceConfig, FetchParentConfigs, SaveConfig, UpdateInstance,
			UpdateInstanceKeys, UpdateInstanceMode,
		},
		launch::{
			FetchInstanceRunState, InstanceRunState, KillInstance, LaunchInstance,
			LaunchInstanceParams,
		},
		misc::{ShowDirectory, ShowDirectoryOption},
	},
	pages::config::{ConfigState, ConfiguredItem, content::ContentConfig},
	prelude::*,
	state::ModalType,
	util::{
		PtrEq,
		assets::pick_image_file,
		gif::AnimatedIconViewer,
	},
};

#[derive(PartialEq)]
pub struct InstancePage {
	pub id: String,
}

impl Component for InstancePage {
	fn render(&self) -> impl IntoElement {
		let theme = use_theme();
		let front_state = use_front_state();
		let back_state = use_consume::<BackState>();
		let config_query = use_query(FetchInstanceConfig::new(
			self.id.clone(),
			back_state.clone(),
		));
		let duplicate_instance = use_mutation(Mutation::new(DuplicateInstance::new(back_state.clone())));
		let save_config = use_mutation(Mutation::new(SaveConfig::new(back_state.clone()).toast(
			&back_state,
			Some("Сохранено"),
			"Не удалось сохранить конфигурацию",
		)));
		let update = use_mutation(Mutation::new(UpdateInstance::new(back_state.clone())));
		let launch = use_mutation(LaunchInstance::new(back_state.clone()));
		let kill = use_mutation(KillInstance::new(back_state.clone()));
		let run_state = use_query(Query::new(
			self.id.clone(),
			FetchInstanceRunState::new(back_state.clone()),
		));
		let run_state = run_state.read().state().ok().cloned().unwrap_or_default();
		let show_directory = use_mutation(Mutation::new(ShowDirectory::new(back_state.clone())));

		let tab = use_state(|| Tab::Info);
		let config = use_state(InstanceConfig::default);

		let id = self.id.clone();
		let config_state = ConfigState::new(ConfigKind::Instance, false);
		let mut config_state2 = config_state.clone();
		let mut config2 = config;
		let mut tab2 = tab;
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
			if config.main.side == Some(Side::Server) {
				tab2.set(Tab::Servers);
			}
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

		let is_editable = true;
		let is_deletable = true;
		let is_server = config.read().side == Some(Side::Server);

		let is_ico_hovered = use_state(|| false);
		let id_ico = self.id.clone();
		let save_config_ico = save_config;
		let config_ico_write = config.clone();
		let cfg_read = config.clone();
		let back_state_ico = back_state.clone();
		let front_state_ico = front_state.clone();

		let ico_element = if config.read().icon.is_some() {
			AnimatedIconViewer::new(config.read().icon.clone())
				.width(Size::px(52.0))
				.height(Size::px(52.0))
				.image_cover(ImageCover::Center)
				.corner_radius(theme.round2)
				.into_element()
		} else {
			icon("box_dot", 36.0).into_element()
		};

		let ico = rect()
			.width(Size::px(80.0))
			.height(Size::px(80.0))
			.center()
			.clickable()
			.cursor(CursorIcon::Pointer)
			.hover(is_ico_hovered)
			.tip(&front_state, "Нажмите, чтобы изменить иконку")
			.on_press(move |_| {
				let id = id_ico.clone();
				let save_cfg = save_config_ico;
				let cfg_fallback = cfg_read.read().clone();
				let mut config_ico = config_ico_write.clone();
				let bs = back_state_ico.clone();
				let fs = front_state_ico.clone();
				spawn(async move {
					if let Some(path) = pick_image_file().await {
						let path_str = path.to_string_lossy().to_string();
						let mut inst_config = if let Ok(raw_cfg) = bs.raw_config().await {
							raw_cfg
								.instances
								.get(&InstanceID::from(id.clone()))
								.cloned()
								.unwrap_or(cfg_fallback)
						} else {
							cfg_fallback
						};
						inst_config.icon = Some(path_str.clone());
						let template_cfg = TemplateConfig {
							instance: inst_config,
							..Default::default()
						};
						let item = ConfiguredItem {
							id: Some(id),
							ty: ConfigKind::Instance,
							is_new: false,
						};
						save_cfg.mutate(crate::ops::instance::SaveConfigParams {
							item,
							config: NotEq(template_cfg),
						});
						config_ico.write().icon = Some(path_str);
						if let Ok(mut data) = crate::data::LauncherData::open(&bs.paths) {
							let entry = crate::data::InstanceIcon::File(path);
							if !data.saved_instance_icons.contains(&entry) {
								data.saved_instance_icons.push(entry);
								let _ = data.write(&bs.paths);
							}
						}
						fs.write()
							.toast(Toast::info("Иконка сборки обновлена", None));
					}
				});
			})
			.child(ico_element);
		let name = config
			.read()
			.name
			.clone()
			.unwrap_or_else(|| self.id.clone());

		let selected = match run_state {
			InstanceRunState::Stopped => LaunchOption::Launch,
			InstanceRunState::Running => LaunchOption::Kill,
		};
		let id = self.id.clone();
		let launch_dropdown = Dropdown::new(
			Selected::Single(selected),
			Rc::new(move |selected| match selected.single() {
				LaunchOption::Launch => {
					launch.mutate(LaunchInstanceParams {
						id: id.clone(),
						offline: false,
						quick_play: QuickPlayType::default(),
					});
				}
				LaunchOption::LaunchOffline => {
					launch.mutate(LaunchInstanceParams {
						id: id.clone(),
						offline: true,
						quick_play: QuickPlayType::default(),
					});
				}
				LaunchOption::Kill => {
					kill.mutate((id.clone(), None));
				}
			}),
		)
		.options_width(160.0)
		.header_width(Size::auto())
		.panel_colorway()
		.child(SelectOption::new(
			LaunchOption::Launch,
			"Запуск",
			Some("rocket"),
		))
		.maybe_child(!is_server, || {
			SelectOption::new(LaunchOption::LaunchOffline, "Офлайн", Some("rocket"))
		})
		.maybe_child(run_state == InstanceRunState::Running, || {
			SelectOption::new(LaunchOption::Kill, "Остановить процесс", Some("stop"))
		});

		let id = self.id.clone();
		let is_updating = update.read().state().is_loading();
		let update_button = icon_text_button("cycle", "Обновить", &theme)
			.border_fill(theme.panel_border)
			.on_press(move |_| {
				if is_updating {
					return;
				}
				update.mutate(UpdateInstanceKeys {
					id: id.clone(),
					mode: UpdateInstanceMode::Full,
					force: false,
				});
			});

		let id = self.id.clone();
		let front_state2 = front_state.clone();
		let settings_button = icon_text_button("gear", "Настройки", &theme)
			.border_fill(theme.panel_border)
			.on_press(move |_| {
				front_state2
					.write()
					.set_modal(Some(ModalType::Configuration(ConfiguredItem {
						ty: ConfigKind::Instance,
						id: Some(id.clone()),
						is_new: false,
					})));
			});

		let id_folder = self.id.clone();
		let open_folder_button = icon_text_button("folder", "Папка", &theme)
			.border_fill(theme.panel_border)
			.on_press(move |_| {
				show_directory.mutate(ShowDirectoryOption::Instance(id_folder.clone()));
			});

		let id = self.id.clone();
		let front_state2 = front_state.clone();
		let more_dropdown = Dropdown::new(
			Selected::Single(MoreOption::More),
			Rc::new(move |selected| match selected.single() {
				MoreOption::More => {}
				MoreOption::OpenFolder => {
					show_directory.mutate(ShowDirectoryOption::Instance(id.clone()));
				}
				MoreOption::Duplicate => {
					duplicate_instance.mutate(id.clone());
					front_state2.write().toast(Toast::success("Копия инстанса создается..."));
				}
				MoreOption::ExportMrpack => {
					let id_export = id.clone();
					let back_state_export = back_state.clone();
					let front_state_export = front_state2.clone();
					spawn(async move {
						let file = rfd::AsyncFileDialog::new()
							.set_file_name(format!("{id_export}.mrpack"))
							.add_filter("Modrinth Modpack", &["mrpack"])
							.save_file()
							.await;
						let Some(handle) = file else {
							return;
						};
						let path = handle.path().to_path_buf();
						front_state_export.write().toast(Toast::info("Экспорт сборки в .mrpack...", None));
						match crate::ops::modpack_io::export_instance_to_mrpack(&id_export, &path, &back_state_export).await {
							Ok(()) => {
								front_state_export.write().toast(Toast::success("Сборка успешно экспортирована в .mrpack!"));
							}
							Err(e) => {
								front_state_export.write().toast(Toast::error(&format!("Ошибка экспорта: {e:#}"), None));
							}
						}
					});
				}
				MoreOption::Delete => {
					front_state2
						.write()
						.set_modal(Some(ModalType::DeleteInstance(id.clone())));
				}
			}),
		)
		.options_width(160.0)
		.align_options_right()
		.custom_header(SelectOption::new(
			MoreOption::More,
			"Ещё",
			Some("ellipsis"),
		))
		.header_width(Size::auto())
		.hide_arrow()
		.panel_colorway()
		.child(SelectOption::new(
			MoreOption::OpenFolder,
			"Открыть папку",
			Some("folder"),
		))
		.child(SelectOption::new(
			MoreOption::Duplicate,
			"Дублировать",
			Some("copy"),
		))
		.child(SelectOption::new(
			MoreOption::ExportMrpack,
			"Экспорт в .mrpack",
			Some("download"),
		))
		.maybe_child(is_deletable, || {
			SelectOption::new(MoreOption::Delete, "Удалить", Some("trash"))
		});

		let controls = rect()
			.height(Size::fill())
			.cont()
			.main_align(Alignment::End)
			.cross_align(Alignment::Center)
			.padding(Gaps::new(0.0, theme.gap3, 0.0, 0.0))
			.child(launch_dropdown)
			.child(update_button)
			.maybe(is_editable, |this| this.child(settings_button))
			.child(open_folder_button)
			.child(more_dropdown);

		let data_hook = crate::state::use_launcher_data();
		let playtime_secs = data_hook.data.read().get_playtime(&self.id);
		let launches = data_hook.data.read().get_launches(&self.id);
		let last_played = data_hook.data.read().get_last_played(&self.id).unwrap_or(0);
		let pt_str = crate::data::format_playtime(playtime_secs);
		let last_played_str = crate::data::format_last_played(last_played);

		let front_state_stats = front_state.clone();
		let activity_tag = rect()
			.horizontal()
			.cross_align(Alignment::Center)
			.spacing(theme.gap)
			.padding(Gaps::new(2.0, theme.gap2, 2.0, theme.gap))
			.background(theme.item_hover)
			.corner_radius(theme.round)
			.cursor(CursorIcon::Pointer)
			.on_press(move |_| {
				front_state_stats
					.write()
					.set_modal(Some(ModalType::PlaytimeStats));
			})
			.child(icon("clock", 14.0))
			.child(label().text(if is_server {
				format!("{launches} запусков • Последний: {last_played_str}")
			} else {
				format!("{pt_str} в игре • {launches} зап. • Последний: {last_played_str}")
			}).font_size(theme.font0).color(theme.fg3));

		let head = rect()
			.width(Size::fill())
			.height(Size::px(80.0))
			.cont()
			.border(border_bottom(theme.border, theme.panel_border))
			.child(ico)
			.child(
				rect()
					.width(Size::flex(1.0))
					.height(Size::fill())
					.main_align(Alignment::Center)
					.spacing(4.0)
					.child(
						label()
							.text(name.to_string())
							.font_size(theme.font2)
							.font_weight(FontWeight::BOLD)
							.color(theme.fg),
					)
					.child(activity_tag),
			)
			.child(controls);

		let tabs = SideTabs::from_state(tab)
			.child(SelectOption::new(
				Tab::Info,
				if is_server { "Файлы" } else { "Инфо" },
				Some("info"),
			))
			.child(SelectOption::new(Tab::Servers, "Серверы", Some("server")))
			.child(SelectOption::new(
				Tab::Content,
				if is_server { "Плагины" } else { "Моды и дополнения" },
				Some("honeycomb"),
			))
			.child(SelectOption::new(Tab::Console, "Консоль", Some("text")));
		let tabs = rect()
			.width(Size::flex(1.0))
			.height(Size::fill())
			.border(border_right(theme.border, theme.panel_border))
			.child(tabs);

		let config_state2 = config_state.clone();
		let save_fn = config_state.save_fn(front_state.clone(), save_config);
		let contents = match &*tab.read() {
			Tab::Info => InstanceFilesView {
				id: self.id.clone(),
			}
			.into_element(),
			Tab::Servers => InstanceServersView {
				id: self.id.clone(),
			}
			.into_element(),
			Tab::Content => ContentConfig {
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
			Tab::Console => InstanceConsole {
				id: self.id.clone(),
				is_server,
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
enum Tab {
	Info,
	Servers,
	Content,
	Console,
}

#[derive(PartialEq, Clone)]
enum LaunchOption {
	Launch,
	LaunchOffline,
	Kill,
}

#[derive(PartialEq, Clone)]
enum MoreOption {
	More,
	OpenFolder,
	Duplicate,
	ExportMrpack,
	Delete,
}
