use std::sync::Arc;

use sqrilizz::config::preferences::ConfigPreferences;

use crate::{
	components::{
		console::{Console, ConsoleImpl},
		dialog::modal::{Modal, ModalButton},
		input::tabs::SideTabs,
	},
	data::LauncherData,
	ops::{
		QueryOption,
		misc::{FetchGlobalLog, FetchGlobalLogs, ShowDirectory, ShowDirectoryOption},
		settings::{FetchPreferences, SavePreferences},
	},
	pages::settings::{accounts::AccountSettings, general::GeneralSettings},
	prelude::*,
	state::ModalType,
	util::PtrEq,
};

pub mod accounts;
mod general;

#[derive(PartialEq)]
pub struct SettingsPage;

impl Component for SettingsPage {
	fn render(&self) -> impl IntoElement {
		let front_state = use_front_state();
		front_state.read().subscribe(FrontChannel::Modal);
		let tab = if let Some(ModalType::Settings(tab)) = front_state.read().modal() {
			Some(tab.clone())
		} else {
			None
		};

		let on_submit = use_state::<PtrEq<dyn Fn() -> bool>>(|| PtrEq(Arc::new(|| true)));
		let is_dirty = use_state(|| false);

		let front_state2 = front_state.clone();
		Modal::new("Настройки".into(), "gear".into())
			.maybe_child(tab.is_some(), || SettingsModal {
				tab: tab.unwrap_or(Tab::General),
				on_submit,
				is_dirty,
			})
			.size_large()
			.on_close(move |_| front_state.write().set_modal(None))
			.cancel_button()
			.button(ModalButton {
				title: "Сохранить".into(),
				icon: "check".into(),
				on_click: EventHandler::from(move |_| {
					let successful = (on_submit.read().0)();
					if successful {
						front_state2.write().set_modal(None);
					}
				}),
				active: *is_dirty.read(),
				danger: false,
			})
	}
}

#[derive(PartialEq)]
struct SettingsModal {
	tab: Tab,
	on_submit: State<PtrEq<dyn Fn() -> bool>>,
	is_dirty: State<bool>,
}

impl Component for SettingsModal {
	fn render(&self) -> impl IntoElement {
		let theme = use_theme();
		let front_state = use_front_state();
		let back_state = use_consume::<BackState>();
		let prefs_query = use_query(Query::new((), FetchPreferences::new(back_state.clone())));
		let data = use_hook(|| Arc::new(back_state.data()));
		let save_prefs = use_mutation(Mutation::new(
			SavePreferences::new(back_state.clone()).toast(
				&back_state,
				Some("Saved"),
				"Failed to save config",
			),
		));
		let show_directory = use_mutation(Mutation::new(ShowDirectory::new(back_state.clone())));

		let settings_state = SettingsState::new(self.is_dirty);

		let front_state2 = front_state.clone();
		let back_state2 = back_state.clone();
		let settings_state2 = settings_state.clone();
		let mut on_submit_state = self.on_submit;
		use_side_effect(move || {
			let prefs = prefs_query.read().state().ok().cloned().unwrap_or_default();

			let _original_prefs = prefs.clone();
			let original_data = data.clone();
			settings_state2.update(prefs.clone(), (*data).clone());

			let settings_state3 = settings_state2.clone();
			let back_state2 = back_state2.clone();
			let front_state = front_state2.clone();
			let on_submit = move || {
				let mut data = (*original_data).clone();
				if settings_state3.apply(&mut data).is_err() {
					return false;
				};

				if let Err(e) = data.write(&back_state2.paths) {
					front_state
						.write()
						.toast(Toast::from_error("Failed to write launcher data", e));
					return false;
				}
				save_prefs.mutate(NotEq(prefs.clone()));

				front_state.write().invalidate(FrontChannel::Data);

				true
			};
			on_submit_state.set(PtrEq(Arc::new(on_submit)));
		});

		let tab = use_state(|| self.tab.clone());
		let show_directory2 = show_directory;
		let show_directory3 = show_directory;

		let tabs = SideTabs::from_state(tab)
			.child(SelectOption::new(Tab::General, "Основные", Some("gear")))
			.child(SelectOption::new(
				Tab::Accounts,
				"Аккаунты",
				Some("multiple_users"),
			))
			.child(SelectOption::new(Tab::Logs, "Логи", Some("text")));

		let front_state2 = front_state.clone();
		let left_panel = rect()
			.width(Size::flex(1.0))
			.border(border_right(theme.border, theme.panel_border))
			.cont()
			.vertical()
			.child(
				rect()
					.width(Size::fill())
					.height(Size::flex(1.0))
					.child(tabs),
			)
			.child(
				rect()
					.width(Size::fill())
					.cont()
					.vertical()
					.padding(theme.gap2)
					.child(crate::components::input::tabs::Tab {
						option: SelectOption::new("", "Папка данных", Some("folder")),
						is_selected: false,
						on_select: EventHandler::from(move |_: &str| {
							show_directory2.mutate(ShowDirectoryOption::Data);
						}),
						horizontal: false,
					})
					.child(crate::components::input::tabs::Tab {
						option: SelectOption::new("", "Папка конфигурации", Some("gear")),
						is_selected: false,
						on_select: EventHandler::from(move |_: &str| {
							show_directory3.mutate(ShowDirectoryOption::Config);
						}),
						horizontal: false,
					})
					.child(crate::components::input::tabs::Tab {
						option: SelectOption::new("", "Приветствие", Some("refresh")),
						is_selected: false,
						on_select: EventHandler::from(move |_: &str| {
							front_state2.write().set_modal(Some(ModalType::Onboarding));
						}),
						horizontal: false,
					}),
			);

		let tab_contents = match &*tab.read() {
			Tab::General => GeneralSettings {
				state: settings_state.clone(),
			}
			.into_element(),
			Tab::Accounts => AccountSettings.into_element(),
			Tab::Logs => SettingsConsole.into_element(),
		};

		let right_panel = rect()
			.width(Size::flex(1.0))
			.height(Size::fill())
			.padding(theme.gap3)
			.child(tab_contents);

		rect()
			.width(Size::fill())
			.height(Size::fill())
			.child(left_panel)
			.child(right_panel)
	}
}

#[derive(PartialEq, Clone)]
pub enum Tab {
	General,
	Accounts,
	Logs,
}

#[derive(PartialEq, Clone)]
pub struct SettingsState {
	pub base_theme: State<String>,
	pub overlay_themes: State<Vec<String>>,
	pub zoom: State<f64>,
	pub discord_rpc: State<bool>,
	orig: State<PtrEq<OrigSettings>>,
}

#[derive(PartialEq, Default, Clone)]
struct OrigSettings {
	base_theme: String,
	overlay_themes: Vec<String>,
	zoom: f64,
	discord_rpc: bool,
}

impl SettingsState {
	pub fn new(mut on_dirty: State<bool>) -> Self {
		let base_theme = use_state(String::new);
		let overlay_themes = use_state(Vec::new);
		let zoom = use_state(|| 1.0);
		let discord_rpc = use_state(|| true);
		let orig = use_state(|| PtrEq(Arc::new(OrigSettings::default())));

		let st = Self {
			base_theme,
			overlay_themes,
			zoom,
			discord_rpc,
			orig,
		};

		let st2 = st.clone();
		use_side_effect(move || {
			let orig = (*st2.orig.read().0).clone();
			let current = OrigSettings {
				base_theme: (*st2.base_theme.read()).clone(),
				overlay_themes: st2.overlay_themes.read().clone(),
				zoom: *st2.zoom.read(),
				discord_rpc: *st2.discord_rpc.read(),
			};
			if current != orig {
				on_dirty.set(true);
			}
		});

		st
	}

	pub fn update(&self, _prefs: ConfigPreferences, data: LauncherData) {
		let mut base_theme = self.base_theme.clone();
		base_theme.set_if_modified(data.base_theme.clone().unwrap_or_default());

		let mut overlay_themes = self.overlay_themes.clone();
		overlay_themes.set_if_modified(data.overlay_themes.clone());

		let mut zoom = self.zoom.clone();
		zoom.set_if_modified(data.zoom);
		let mut discord_rpc = self.discord_rpc.clone();
		discord_rpc.set_if_modified(data.discord_rpc);

		let mut orig = self.orig.clone();
		orig.set_if_modified(PtrEq(Arc::new(OrigSettings {
			base_theme: data.base_theme.clone().unwrap_or_default(),
			overlay_themes: data.overlay_themes.clone(),
			zoom: data.zoom,
			discord_rpc: data.discord_rpc,
		})));
	}

	pub fn apply(&self, data: &mut LauncherData) -> anyhow::Result<()> {
		data.base_theme = Some((*self.base_theme.read()).clone()).filter(|x| !x.is_empty());
		data.overlay_themes = self.overlay_themes.read().clone();
		data.zoom = *self.zoom.read();
		data.discord_rpc = *self.discord_rpc.read();
		crate::ops::discord::set_enabled(data.discord_rpc);

		Ok(())
	}
}

#[derive(PartialEq)]
struct SettingsConsole;

impl Component for SettingsConsole {
	fn render(&self) -> impl IntoElement {
		let back_state = use_consume::<BackState>();
		let selected_log = use_state::<Option<String>>(|| None);
		let contents_query = use_query(Query::new(
			QueryOption(selected_log.read().clone()),
			FetchGlobalLog::new(back_state.clone()),
		));
		let logs = use_query(Query::new((), FetchGlobalLogs::new(back_state.clone())));

		let contents = contents_query
			.read()
			.state()
			.ok()
			.cloned()
			.map(PtrEq);
		let logs = logs.read().state().ok().cloned().unwrap_or_default();

		let console = Impl {
			contents,
			log_files: PtrEq(logs),
			selected_log,
		};

		Console { console }
	}
}

#[derive(PartialEq, Clone)]
struct Impl {
	contents: Option<PtrEq<str>>,
	log_files: PtrEq<[String]>,
	selected_log: State<Option<String>>,
}

impl ConsoleImpl for Impl {
	fn contents(&self) -> Option<Arc<str>> {
		self.contents.as_ref().map(|x| x.0.clone())
	}

	fn is_loading(&self) -> bool {
		false
	}

	fn get_log_files(&self) -> impl Iterator<Item = &String> {
		self.log_files.0.iter()
	}

	fn get_log_file(&self) -> Option<String> {
		self.selected_log.read().clone()
	}

	fn set_log_file(&self, file: Option<String>) {
		self.selected_log.clone().set(file);
	}

	fn input_fn(&self) -> Option<impl Fn(String) -> bool + 'static> {
		None::<Box<dyn Fn(String) -> bool>>
	}
}
