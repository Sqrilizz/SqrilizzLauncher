use std::{rc::Rc, sync::LazyLock};

use crate::theme::{ThemeInfo as Theme, ThemeType};

use crate::{
	components::input::switch::Switch, pages::settings::SettingsState, prelude::*,
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
		let on_select = self.on_select.clone();
		let id = self.theme.0.id.clone();

		let round = theme.round2;
		let border_color = if self.is_selected {
			theme.primary
		} else {
			theme.panel_border
		};

		let mut preview_colors: Vec<Color> = Vec::new();
		if let Ok(theme_deser) = serde_json::from_str::<ThemeDeser>(&self.theme.0.settings) {
			if let Some(bg) = theme_deser.bg {
				preview_colors.push(bg.into());
			}
			if let Some(primary) = theme_deser.primary {
				preview_colors.push(primary.into());
			}
			if let Some(fg) = theme_deser.fg {
				preview_colors.push(fg.into());
			}
		}

		let preview_dots = preview_colors.into_iter().map(|color| {
			rect()
				.width(Size::px(10.0))
				.height(Size::px(10.0))
				.corner_radius(5.0)
				.background(color)
				.margin(Gaps::new(0.0, 4.0, 0.0, 4.0))
		});

		let title = self.theme.0.name.clone();
		let desc = self.theme.0.description.clone();

		let is_dark_bg = self.theme.0.id == "dark";
		let mini_preview_bg = if is_dark_bg {
			Color::from_rgb(18, 18, 20)
		} else if self.theme.0.id == "light" {
			Color::from_rgb(245, 245, 247)
		} else {
			theme.bg
		};

		let mini_preview_fg = if self.theme.0.id == "light" {
			Color::from_rgb(30, 30, 30)
		} else {
			Color::from_rgb(240, 240, 240)
		};

		rect()
			.width(Size::fill())
			.height(Size::px(160.0))
			.border(Border {
				width: if self.is_selected {
					2.0.into()
				} else {
					theme.border.into()
				},
				fill: border_color.into(),
				alignment: BorderAlignment::Inner,
			})
			.corner_radius(round)
			.padding(theme.gap2)
			.background(theme.item)
			.hover(is_hovered)
			.clickable()
			.on_press(move |_| on_select.call(id.clone()))
			.child(
				label()
					.text(title)
					.font_size(14.0)
					.font_weight(FontWeight::BOLD)
					.color(if self.is_selected {
						theme.primary
					} else {
						theme.fg
					}),
			)
			.child(
				rect()
					.width(Size::fill())
					.height(Size::px(80.0))
					.margin(Gaps::new(theme.gap, 0.0, theme.gap, 0.0))
					.corner_radius(theme.round)
					.background(mini_preview_bg)
					.center()
					.child(rect().horizontal().children(preview_dots))
					.maybe(desc.is_some(), |this| {
						this.child(
							rect().margin(Gaps::new(theme.gap, 0.0, 0.0, 0.0)).child(
								label()
									.text(desc.unwrap())
									.font_size(11.0)
									.color(mini_preview_fg),
							),
						)
					}),
			)
			.into_element()
	}
}
