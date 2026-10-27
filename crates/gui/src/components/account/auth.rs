use crate::{
	components::{dialog::modal::Modal, icon_text_button},
	ops::task::{KillTask, Task},
	prelude::*,
};
use sqrilizz::shared::util::open_link;

#[derive(PartialEq)]
pub struct MicrosoftAuthPrompt {
	url: String,
	device_code: String,
	on_close: EventHandler<()>,
}

impl MicrosoftAuthPrompt {
	pub fn new(
		url: impl Into<String>,
		device_code: impl Into<String>,
		on_close: impl Into<EventHandler<()>>,
	) -> Self {
		Self {
			url: url.into(),
			device_code: device_code.into(),
			on_close: on_close.into(),
		}
	}
}

impl Component for MicrosoftAuthPrompt {
	fn render(&self) -> impl IntoElement {
		let theme = use_theme();
		let back_state = use_consume::<BackState>();
		let kill_task_mutation = use_mutation(Mutation::new(KillTask::new(back_state.clone())));

		let body = ms_auth_info(self.url.clone(), self.device_code.clone(), &theme);

		let on_close = self.on_close.clone();
		Modal::new("Авторизация Microsoft".into(), "lock".into())
			.size(560.0, 420.0)
			.on_close(move |_| {
				on_close.call(());
				kill_task_mutation.mutate(Task::LoginAccount);
			})
			.maybe_child(true, move || body)
			.cancel_button()
	}
}

pub fn ms_auth_info(url: String, device_code: String, theme: &Theme) -> Rect {
	let device_code_copy = device_code.clone();
	let code_box = rect()
		.width(Size::fill())
		.height(Size::px(52.0))
		.background(theme.bg)
		.border(theme.border(theme.primary))
		.corner_radius(theme.round)
		.horizontal()
		.cross_align(Alignment::Center)
		.padding(Gaps::new(0.0, theme.gap2, 0.0, theme.gap2))
		.child(
			rect()
				.width(Size::flex(1.0))
				.height(Size::fill())
				.horizontal()
				.cross_align(Alignment::Center)
				.main_align(Alignment::Start)
				.padding(Gaps::new(0.0, theme.gap, 0.0, theme.gap))
				.child(
					label()
						.text(device_code)
						.font_size(22.0)
						.font_weight(FontWeight::BOLD)
						.color(theme.primary),
				),
		)
		.child(
			icon_text_button("copy", "Копировать", theme)
				.width(Size::px(140.0))
				.on_press(move |_| {
					let _ = Clipboard::set(device_code_copy.clone());
				}),
		);
	let code_copy = field("Скопируйте этот код", "copy", theme, code_box);

	let url2 = url.clone();
	let button = icon_text_button("globe", "Открыть страницу входа", theme)
		.width(Size::px(220.0))
		.color(theme.primary)
		.border_fill(theme.primary)
		.background(theme.primary_bg)
		.hover_background(theme.primary_bg)
		.on_press(move |_| {
			let _ = open_link(&url2);
		});
	let open_login_page = field(
		"Вставьте скопированный код на странице входа",
		"globe",
		theme,
		button,
	);

	let fallback = field(
		"Если браузер не открылся автоматически, используйте ссылку ниже",
		"link",
		theme,
		label().text(url).color(theme.fg3),
	);

	rect()
		.expanded()
		.vertical()
		.padding(theme.gap2)
		.child(code_copy)
		.child(open_login_page)
		.child(fallback)
}
