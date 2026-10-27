use crate::{
	components::{
		dialog::modal::{MODAL_LARGE_HEIGHT, MODAL_LARGE_WIDTH, Modal, ModalButton},
		input::tabs::TopTabs,
	},
	pages::settings::accounts::AccountSettings,
	prelude::*,
	state::use_launcher_data,
	util::assets::SPLASH,
};

#[derive(PartialEq, Clone, Copy)]
pub enum HelpTab {
	Overview,
	Guide,
	Troubleshooting,
	Accounts,
	About,
}

#[derive(PartialEq)]
pub struct OnboardingModal;

impl Component for OnboardingModal {
	fn render(&self) -> impl IntoElement {
		let theme = use_theme();
		let front_state = use_front_state();
		let data = use_launcher_data();
		let is_first_run = !data.data.read().launcher_opened_before;

		let tab = use_state(|| HelpTab::Overview);

		let tabs = TopTabs::from_state(tab)
			.child(SelectOption::new(
				HelpTab::Overview,
				"Обзор лаунчера",
				Some("box_dot"),
			))
			.child(SelectOption::new(
				HelpTab::Guide,
				"Руководство",
				Some("text"),
			))
			.child(SelectOption::new(
				HelpTab::Troubleshooting,
				"Решение проблем",
				Some("warning"),
			))
			.child(SelectOption::new(
				HelpTab::Accounts,
				"Аккаунты",
				Some("multiple_users"),
			))
			.child(SelectOption::new(
				HelpTab::About,
				"О лаунчере",
				Some("info"),
			));

		let front_state3 = front_state.clone();

		let contents = match *tab.read() {
			HelpTab::Overview => overview_view(&theme).into_element(),
			HelpTab::Guide => guide_view(&theme).into_element(),
			HelpTab::Troubleshooting => troubleshooting_view(&theme).into_element(),
			HelpTab::Accounts => accounts_view(&theme).into_element(),
			HelpTab::About => about_view(&theme).into_element(),
		};

		let top_nav = rect()
			.width(Size::fill())
			.background(theme.panel)
			.border(border_bottom(theme.border, theme.panel_border))
			.padding(Gaps::new(2.0, theme.gap2, 2.0, theme.gap2))
			.cont()
			.horizontal()
			.cross_align(Alignment::Center)
			.child(rect().width(Size::flex(1.0)).child(tabs))
			.child(rect().padding(Gaps::new(0.0, 0.0, 0.0, theme.gap)).child(
				icon_text_button("globe", "sqrilizz.tech", &theme).on_press(move |_| {
					let _ = open::that("https://sqrilizz.tech");
				}),
			));

		let body = rect()
			.width(Size::fill())
			.height(Size::fill())
			.vertical()
			.child(top_nav)
			.child(
				rect()
					.width(Size::fill())
					.height(Size::flex(1.0))
					.child(contents),
			);

		let title = if is_first_run {
			"Добро пожаловать в Sqrilizz Launcher"
		} else {
			"Помощь и руководство"
		};

		let action_text = if is_first_run {
			"Начать играть!"
		} else {
			"Закрыть"
		};

		let mut data_close = data.clone();
		let mut data_click = data.clone();
		Modal::new(title.into(), "info".into())
			.size(MODAL_LARGE_WIDTH, MODAL_LARGE_HEIGHT)
			.on_close(move |_| {
				data_close.data.write().launcher_opened_before = true;
				data_close.save();
				front_state3.write().set_modal(None);
			})
			.button(ModalButton {
				title: action_text.into(),
				icon: "check".into(),
				on_click: EventHandler::new(move |_| {
					data_click.data.write().launcher_opened_before = true;
					data_click.save();
					front_state.write().set_modal(None);
				}),
				active: true,
				danger: false,
			})
			.maybe_child(true, || body)
	}
}

fn overview_view(theme: &Theme) -> impl IntoElement {
	let banner = rect()
		.width(Size::fill())
		.height(Size::px(160.0))
		.corner_radius(theme.round2)
		.shiny_border(theme)
		.background(theme.panel)
		.padding(theme.gap2)
		.cont()
		.horizontal()
		.child(
			ImageViewer::new(("splash", SPLASH))
				.width(Size::px(280.0))
				.height(Size::fill())
				.corner_radius(theme.round)
				.image_cover(ImageCover::Center),
		)
		.child(
			rect()
				.width(Size::flex(1.0))
				.padding(Gaps::new(theme.gap, theme.gap2, theme.gap, theme.gap3))
				.spacing(theme.gap)
				.vertical()
				.child(
					label()
						.text("Sqrilizz Launcher")
						.font_size(26.0)
						.font_weight(FontWeight::BOLD)
						.color(theme.fg),
				)
				.child(
					label()
						.text(
							"Современный, производительный и модульный лаунчер Minecraft с интерфейсом на Rust и Skia.",
						)
						.font_size(14.0)
						.color(theme.fg2),
				)
				.child(
					rect()
						.horizontal()
						.spacing(theme.gap)
						.child(badge(
							"Modrinth API",
							theme.primary,
							theme.primary_bg,
							theme,
						))
						.child(badge(
							"Fabric / Forge / NeoForge",
							theme.warning,
							theme.item_hover,
							theme,
						))
						.child(badge(
							"Java 8-21 Auto",
							theme.fg2,
							theme.panel_border,
							theme,
						)),
				),
		);

	let col_left = rect()
		.width(Size::flex(1.0))
		.spacing(theme.gap2)
		.vertical()
		.child(feature_card(
			"rocket",
			"Быстрый запуск и офлайн-режим",
			"Автоматическая загрузка Java (8, 17, 21), библиотек и игровых ассетов. Запуск Minecraft в офлайн-режиме без интернета («Запуск» -> «Офлайн»).",
			theme,
		))
		.child(feature_card(
			"box",
			"Каталог модов, сборок и шейдеров",
			"Прямой поиск в каталоге Modrinth с автофильтром под версию Minecraft и загрузчик. Установка модов в один клик.",
			theme,
		))
		.child(feature_card(
			"gear",
			"Гибкая настройка инстансов",
			"Выделение оперативной памяти (RAM), аргументы JVM, переключение загрузчиков и система шаблонов сборок.",
			theme,
		));

	let col_right = rect()
		.width(Size::flex(1.0))
		.spacing(theme.gap2)
		.vertical()
		.child(feature_card(
			"cycle",
			"Автопроверка обновлений и тумблеры",
			"Сверка версий модов через Modrinth API в один клик. Временное выключение модов (.disabled) без удаления файлов с диска.",
			theme,
		))
		.child(feature_card(
			"download",
			"Импорт и экспорт модпаков",
			"Экспорт ваших сборок в стандартный формат Modrinth .mrpack, а также импорт готовых архивов прямо с компьютера.",
			theme,
		))
		.child(feature_card(
			"folder",
			"Быстрый доступ к файлам и логам",
			"Удобные кнопки открытия папки игры, mods/, скриншотов, а также встроенный просмотрщик логов и краш-дампов.",
			theme,
		));

	let cards_grid = rect()
		.width(Size::fill())
		.spacing(theme.gap2)
		.horizontal()
		.child(col_left)
		.child(col_right);

	let content = rect()
		.width(Size::fill())
		.spacing(theme.gap2)
		.padding(theme.gap3)
		.vertical()
		.child(banner)
		.child(cards_grid);

	ScrollView::new()
		.expanded()
		.direction(Direction::Vertical)
		.child(content)
}

fn guide_view(theme: &Theme) -> impl IntoElement {
	let col_left = rect()
		.width(Size::flex(1.0))
		.spacing(theme.gap2)
		.vertical()
		.child(guide_step(
			"1",
			"Создание или выбор сборки",
			"Нажмите «+ Создать» в левом верхнем углу. Выберите «Новый инстанс» для чистой версии, «Новая сборка» для каталога Modrinth, либо «Импорт файла (.mrpack / .zip)» для локального архива.",
			theme,
		))
		.child(guide_step(
			"2",
			"Установка модов и шейдеров",
			"Откройте сборку -> «Пакеты» -> «+ Добавить моды». Каталог откроется с фильтром по вашей версии игры и загрузчику. Нажмите «Установить».",
			theme,
		))
		.child(guide_step(
			"3",
			"Управление и отключение модов",
			"Чтобы временно отключить мод, щелкните тумблер справа от названия в списке. Файл переименуется в .disabled, и игра запустится без него.",
			theme,
		));

	let col_right = rect()
		.width(Size::flex(1.0))
		.spacing(theme.gap2)
		.vertical()
		.child(guide_step(
			"4",
			"Проверка обновлений модов",
			"Кнопка «Проверить обновления» в панели модов сверяет версии через Modrinth API. Доступные апдейты подсвечиваются зеленым бейджем.",
			theme,
		))
		.child(guide_step(
			"5",
			"Экспорт сборки в .mrpack",
			"В карточке сборки нажмите «Ещё» -> «Экспорт в .mrpack». Лаунчер упакует все моды, конфиги и настройки в стандартный .mrpack для друзей.",
			theme,
		))
		.child(guide_step(
			"6",
			"Выделение оперативной памяти (RAM)",
			"Если игра тормозит или крашится, откройте «Настройки» сборки -> «Основные» и увеличьте объем RAM (рекомендуется 4096-8192 МБ для сборок).",
			theme,
		));

	let steps_grid = rect()
		.width(Size::fill())
		.spacing(theme.gap2)
		.horizontal()
		.child(col_left)
		.child(col_right);

	let content = rect()
		.width(Size::fill())
		.spacing(theme.gap2)
		.padding(theme.gap3)
		.vertical()
		.child(
			label()
				.text("Пошаговое руководство")
				.font_size(20.0)
				.font_weight(FontWeight::BOLD)
				.color(theme.fg),
		)
		.child(steps_grid);

	ScrollView::new()
		.expanded()
		.direction(Direction::Vertical)
		.child(content)
}

fn troubleshooting_view(theme: &Theme) -> impl IntoElement {
	let col_left = rect()
		.width(Size::flex(1.0))
		.spacing(theme.gap2)
		.vertical()
		.child(faq_item(
			"Игра вылетает при запуске / краш",
			"Проверьте вкладку «Логи» или нажмите «Папка» -> crash-reports/. Чаще всего краш вызывают несовместимые моды — временно отключите недавно добавленные моды тумблером в списке. Также убедитесь в совместимости модов с вашим загрузчиком.",
			"warning",
			theme,
		))
		.child(faq_item(
			"Какая версия Java требуется?",
			"Minecraft 1.16.5 и старее использует Java 8. Версии 1.17–1.20.4 требуют Java 17. Версии 1.20.5+ требуют Java 21. Лаунчер автоматически загружает и настраивает нужную версию Java при первом запуске.",
			"info",
			theme,
		));

	let col_right = rect()
		.width(Size::flex(1.0))
		.spacing(theme.gap2)
		.vertical()
		.child(faq_item(
			"Ошибка OutOfMemoryError / зависание",
			"Выделено слишком мало оперативной памяти. Откройте «Настройки» сборки (шестерёнка) и увеличьте значение RAM до 4–8 ГБ в зависимости от объёма памяти на вашем компьютере.",
			"warning",
			theme,
		))
		.child(faq_item(
			"Как запустить Minecraft без интернета?",
			"В карточке сборки нажмите на стрелочку справа от кнопки «Запуск» и выберите пункт «Офлайн». Лаунчер запустит игру без проверки серверов авторизации.",
			"rocket",
			theme,
		))
		.child(faq_item(
			"Как открыть папку игры или mods/?",
			"В карточке инстанса нажмите кнопку «Папка», чтобы открыть корень игры. На вкладке модов нажмите «Папка модов», чтобы перейти к файлам .jar.",
			"folder",
			theme,
		));

	let faq_grid = rect()
		.width(Size::fill())
		.spacing(theme.gap2)
		.horizontal()
		.child(col_left)
		.child(col_right);

	let content = rect()
		.width(Size::fill())
		.spacing(theme.gap2)
		.padding(theme.gap3)
		.vertical()
		.child(
			label()
				.text("Частые вопросы и решение проблем")
				.font_size(20.0)
				.font_weight(FontWeight::BOLD)
				.color(theme.fg),
		)
		.child(faq_grid);

	ScrollView::new()
		.expanded()
		.direction(Direction::Vertical)
		.child(content)
}

fn accounts_view(_theme: &Theme) -> impl IntoElement {
	rect().expanded().padding(12.0).child(AccountSettings)
}

fn about_view(theme: &Theme) -> impl IntoElement {
	let banner = rect()
		.width(Size::fill())
		.padding(theme.gap3)
		.background(theme.panel)
		.border(theme.border(theme.panel_border))
		.corner_radius(theme.round2)
		.spacing(theme.gap)
		.vertical()
		.child(
			label()
				.text("Sqrilizz Launcher")
				.font_size(26.0)
				.font_weight(FontWeight::BOLD)
				.color(theme.fg),
		)
		.child(
			label()
				.text("Версия 0.32.2 (Release)")
				.font_size(14.0)
				.color(theme.primary),
		)
		.child(
			label()
				.text(
					"Быстрый, нативный и модульный лаунчер Minecraft с интерфейсом на Rust и Skia.",
				)
				.font_size(13.0)
				.color(theme.fg2),
		);

	let links_card = rect()
		.width(Size::flex(1.0))
		.padding(theme.gap3)
		.background(theme.panel)
		.border(theme.border(theme.panel_border))
		.corner_radius(theme.round2)
		.spacing(theme.gap2)
		.vertical()
		.child(
			label()
				.text("Официальные ресурсы")
				.font_size(16.0)
				.font_weight(FontWeight::BOLD)
				.color(theme.fg),
		)
		.child(
			icon_text_button("globe", "Официальный сайт sqrilizz.tech", theme).on_press(
				move |_| {
					let _ = open::that("https://sqrilizz.tech");
				},
			),
		);

	let folders_card = rect()
		.width(Size::flex(1.0))
		.padding(theme.gap3)
		.background(theme.panel)
		.border(theme.border(theme.panel_border))
		.corner_radius(theme.round2)
		.spacing(theme.gap2)
		.vertical()
		.child(
			label()
				.text("Данные лаунчера")
				.font_size(16.0)
				.font_weight(FontWeight::BOLD)
				.color(theme.fg),
		)
		.child(
			icon_text_button("folder", "Открыть папку данных лаунчера", theme).on_press(
				move |_| {
					if let Some(proj) = directories::ProjectDirs::from("", "sqrilizz", "sqrilizz") {
						let _ = open::that(proj.data_dir());
					}
				},
			),
		);

	let row_cards = rect()
		.width(Size::fill())
		.spacing(theme.gap2)
		.horizontal()
		.child(links_card)
		.child(folders_card);

	let content = rect()
		.width(Size::fill())
		.spacing(theme.gap3)
		.padding(theme.gap3)
		.vertical()
		.child(banner)
		.child(row_cards);

	ScrollView::new()
		.expanded()
		.direction(Direction::Vertical)
		.child(content)
}

fn badge(text: &str, fg: Color, bg: Color, theme: &Theme) -> Rect {
	rect()
		.padding(Gaps::new(4.0, 10.0, 4.0, 10.0))
		.background(bg)
		.border(theme.border(fg))
		.corner_radius(theme.round)
		.child(
			label()
				.text(text.to_string())
				.font_size(11.0)
				.font_weight(FontWeight::BOLD)
				.color(fg),
		)
}

fn feature_card(icon_name: &str, title: &str, desc: &str, theme: &Theme) -> Rect {
	rect()
		.width(Size::fill())
		.padding(theme.gap2)
		.spacing(theme.gap)
		.background(theme.panel)
		.border(theme.border(theme.panel_border))
		.corner_radius(theme.round)
		.child(
			rect()
				.horizontal()
				.spacing(theme.gap)
				.cross_align(Alignment::Center)
				.child(icon(icon_name, 18.0).color(theme.primary))
				.child(
					label()
						.text(title.to_string())
						.font_size(15.0)
						.font_weight(FontWeight::BOLD)
						.color(theme.fg),
				),
		)
		.child(
			label()
				.text(desc.to_string())
				.font_size(13.0)
				.color(theme.fg2),
		)
}

fn guide_step(num: &str, title: &str, desc: &str, theme: &Theme) -> Rect {
	rect()
		.width(Size::fill())
		.padding(theme.gap2)
		.spacing(theme.gap)
		.background(theme.panel)
		.border(theme.border(theme.panel_border))
		.corner_radius(theme.round)
		.child(
			rect()
				.horizontal()
				.spacing(theme.gap)
				.cross_align(Alignment::Center)
				.child(
					rect()
						.width(Size::px(24.0))
						.height(Size::px(24.0))
						.center()
						.corner_radius(theme.round)
						.background(theme.primary_bg)
						.border(theme.border(theme.primary))
						.child(
							label()
								.text(num.to_string())
								.font_size(12.0)
								.font_weight(FontWeight::BOLD)
								.color(theme.primary),
						),
				)
				.child(
					label()
						.text(title.to_string())
						.font_size(15.0)
						.font_weight(FontWeight::BOLD)
						.color(theme.fg),
				),
		)
		.child(
			label()
				.text(desc.to_string())
				.font_size(13.0)
				.color(theme.fg2),
		)
}

fn faq_item(question: &str, answer: &str, icon_name: &str, theme: &Theme) -> Rect {
	rect()
		.width(Size::fill())
		.padding(theme.gap2)
		.spacing(theme.gap)
		.background(theme.panel)
		.border(theme.border(theme.panel_border))
		.corner_radius(theme.round)
		.child(
			rect()
				.horizontal()
				.spacing(theme.gap)
				.cross_align(Alignment::Center)
				.child(icon(icon_name, 16.0).color(theme.warning))
				.child(
					label()
						.text(question.to_string())
						.font_size(15.0)
						.font_weight(FontWeight::BOLD)
						.color(theme.fg),
				),
		)
		.child(
			label()
				.text(answer.to_string())
				.font_size(13.0)
				.color(theme.fg2),
		)
}
