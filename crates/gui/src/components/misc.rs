use crate::prelude::*;

static DISCORD_ICON: &str = "https://external-content.duckduckgo.com/iu/?u=https%3A%2F%2Fi.pinimg.com%2Foriginals%2Fb5%2Fd4%2Fce%2Fb5d4ce10a744861ffd3314d20d116976.jpg&f=1&nofb=1&ipt=6d62e58133af814dc726e3ec8aceb847e0e652df00aa08798e8916e72a539b63";

pub fn socials(theme: &Theme) -> impl IntoElement {
	rect().width(Size::fill()).cont().padding(theme.gap).child(
		segment(
			button(theme).child(
				img(DISCORD_ICON)
					.width(Size::px(32.0))
					.height(Size::px(32.0))
					.corner_radius(theme.round),
			),
			1.0,
		)
		.center(),
	)
}

pub fn progress_bar(theme: &Theme, progress: f32) -> Rect {
	rect()
		.width(Size::fill())
		.height(Size::px(8.0))
		.corner_radius(4.0)
		.panel_colorway(theme, false, false)
		.child(
			rect()
				.width(Size::percent(progress * 100.0))
				.height(Size::fill())
				.corner_radius(4.0)
				.background(theme.primary),
		)
}

pub fn status_panel(text: &str, color: Color, theme: &Theme) -> Rect {
	rect()
		.padding(theme.gap3)
		.color(color)
		.border(theme.border(color))
		.corner_radius(theme.round)
		.child(label().text(text.to_string()).font_size(16.0))
}

pub fn number_indicator(num: usize, theme: &Theme) -> Rect {
	rect()
		.height(Size::px(20.0))
		.min_width(Size::px(20.0))
		.padding(Gaps::new(0.0, theme.gap, 0.0, theme.gap))
		.center()
		.corner_radius(theme.round)
		.background(theme.item)
		.font_size(theme.font0)
		.font_weight(FontWeight::BOLD)
		.color(theme.fg3)
		.child(num.to_string())
}
