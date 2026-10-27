use std::{rc::Rc, sync::LazyLock};

use crate::theme::{ThemeInfo as Theme, ThemeType};

use crate::{
	components::input::{slider, switch::Switch},
	pages::settings::SettingsState,
	prelude::*,
	theme::ThemeDeser,
};

static BUILTIN_THEMES: LazyLock<[Theme; 2]> = LazyLock::new(|| {
	[
		Theme {
			id: "dark".into(),
			name: "Тёмная".into(),
			description: Some("Стандартная тёмная тема".into()),
			r#type: ThemeType::Base,
			settings: serde_json::to_string(&ThemeDeser::dark()).unwrap_or_default(),
			color: "#1e1e1e".into(),
		},
		Theme {
			id: "light".into(),
			name: "Светлая".into(),
			description: Some("Стандартная светлая тема".into()),
			r#type: ThemeType::Base,
			settings: serde_json::to_string(&ThemeDeser::light()).unwrap_or_default(),
			color: "#ffffff".into(),
		},
	]
});

#[derive(PartialEq)]
pub struct GeneralSettings {
	pub state: SettingsState,
}

impl Component for GeneralSettings {
	fn render(&self) -> impl IntoElement {
		let theme = use_theme();
		let front_state = use_front_state();
		let back_state = use_consume::<BackState>();

		let base_themes = BUILTIN_THEMES
			.iter()
			.chain(
				back_state
					.themes()
					.iter()
					.filter(|x| matches!(x.r#type, ThemeType::Base) && !x.settings.is_empty()),
			)
			.map(|x| SelectableTheme {
				theme: NotEq(x.clone()),
				is_selected: *self.state.base_theme.read() == x.id,
				on_select: self.state.base_theme.setter(),
			});
		let base_themes = grid(3, base_themes).gap(theme.gap2);
		let base_themes = field("Основная тема", "palette", &theme, base_themes)
			.tip(&front_state, "Основная тема оформления лаунчера");

		let overlay_themes = back_state
			.themes()
			.iter()
			.filter(|x| matches!(x.r#type, ThemeType::Overlay) && !x.settings.is_empty())
			.map(|x| SelectableTheme {
				theme: NotEq(x.clone()),
				is_selected: self.state.overlay_themes.read().contains(&x.id),
				on_select: self.state.overlay_themes.select_setter(),
			});
		let overlay_themes = grid(3, overlay_themes).gap(theme.gap2);
		let overlay_themes = field("Дополнительные эффекты", "palette", &theme, overlay_themes)
			.tip(&front_state, "Визуальные оверлеи поверх основной темы.");

		let zoom = slider(
			*self.state.zoom.read(),
			0.5,
			1.75,
			0.1,
			self.state.zoom.setter(),
			&theme,
			&front_state,
		);
		let zoom = field("Масштаб интерфейса", "search", &theme, zoom)
			.tip(&front_state, "Изменяет масштаб всего интерфейса лаунчера");
		let discord_rpc = field(
			"Discord Rich Presence",
			"controller",
			&theme,
			Switch {
				enabled: *self.state.discord_rpc.read(),
				on_toggle: EventHandler::from({
					let mut state = self.state.discord_rpc;
					move |_| state.set(!*state.read())
				}),
			},
		)
		.tip(
			&front_state,
			"Показывает Sqrilizz и запущенную игру в Discord.",
		);

		let cur_ver = env!("CARGO_PKG_VERSION");
		let check_btn = button(&theme)
			.background(theme.item_hover)
			.padding(Gaps::new(6.0, 14.0, 6.0, 14.0))
			.corner_radius(theme.round)
			.on_press(move |_| {
				tokio::spawn(async move {
					let _ = crate::ops::updater::check_for_updates().await;
				});
			})
			.child(
				rect()
					.horizontal()
					.cross_align(Alignment::Center)
					.spacing(theme.gap)
					.child(crate::icons::icon("refresh", 14.0))
					.child("Проверить обновления"),
			);

		let update_field = field(
			&format!("Версия Sqrilizz v{cur_ver}"),
			"rocket",
			&theme,
			check_btn,
		)
		.tip(&front_state, "Проверить наличие свежих релизов на GitHub.");

		let out = rect()
			.padding(theme.gap3)
			.child(base_themes)
			.child(overlay_themes)
			.child(zoom)
			.child(discord_rpc)
			.child(update_field);

		ScrollView::new().expanded().child(out)
	}
}

#[derive(PartialEq)]
struct SelectableTheme {
	theme: NotEq<Theme>,
	is_selected: bool,
	on_select: EventHandler<String>,
}

impl Component for SelectableTheme {
	fn render(&self) -> impl IntoElement {
		let theme = use_theme();
		let is_hovered = use_state(|| false);

		let calculated: Rc<crate::prelude::Theme> = use_hook(|| {
			Rc::new(
				serde_json::from_str::<ThemeDeser>(&self.theme.0.settings)
					.unwrap_or_else(|_| ThemeDeser::dark())
					.into(),
			)
		});

		let fg = if calculated.fg == Color::TRANSPARENT {
			theme.fg
		} else {
			calculated.fg
		};

		let preview = rect()
			.width(Size::fill())
			.height(Size::px(128.0))
			.background(calculated.bg)
			.color(fg)
			.corner_radius(theme.round)
			.main_align(Alignment::SpaceEvenly)
			.cross_align(Alignment::Center)
			.padding(theme.gap3)
			.spacing(theme.gap3)
			.child(
				rect()
					.width(Size::fill())
					.horizontal()
					.main_align(Alignment::SpaceEvenly)
					.cross_align(Alignment::Center)
					.child(swatch(calculated.item))
					.child(swatch(calculated.primary))
					.child(swatch(calculated.secondary)),
			)
			.maybe_child(self.theme.0.description.clone());

		let id = self.theme.0.id.clone();
		let on_select = self.on_select.clone();
		rect()
			.panel_colorway(&theme, *is_hovered.read(), self.is_selected)
			.corner_radius(theme.round)
			.padding(theme.gap3)
			.spacing(theme.gap3)
			.hover(is_hovered)
			.clickable()
			.on_press(move |_| {
				on_select.call(id.clone());
			})
			.child(
				label()
					.text(self.theme.0.name.clone())
					.font_weight(FontWeight::BOLD),
			)
			.child(preview)
	}
}

fn swatch(color: Color) -> Rect {
	rect()
		.width(Size::px(12.0))
		.height(Size::px(12.0))
		.background(color)
		.corner_radius(6.0)
}
