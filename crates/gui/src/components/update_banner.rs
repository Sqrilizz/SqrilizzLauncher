use crate::{
	components::{button, icon_button, misc::progress_bar},
	icons::icon,
	ops::updater::{self, UpdateStatus},
	prelude::*,
};

#[derive(PartialEq)]
pub struct UpdateBanner;

impl Component for UpdateBanner {
	fn render(&self) -> impl IntoElement {
		let theme = use_theme();
		let status = use_state(|| UpdateStatus::Idle);

		use_future(move || {
			let mut status = status.clone();
			async move {
				let initial = updater::get_status().await;
				status.set(initial);

				// Start background check
				let _ = updater::check_for_updates().await;

				// Listen to update status events
				let mut rx = updater::subscribe_status();
				while let Ok(new_status) = rx.recv().await {
					status.set(new_status);
				}
			}
		});

		let current = status.read().clone();
		match current {
			UpdateStatus::Available(info) => {
				let info_for_btn = info.clone();
				let status_clone = status.clone();

				rect()
					.width(Size::fill())
					.background(theme.navbar)
					.border(border_bottom(2.0, theme.primary))
					.padding(Gaps::new(8.0, 16.0, 8.0, 16.0))
					.horizontal()
					.cross_align(Alignment::Center)
					.main_align(Alignment::SpaceBetween)
					.child(
						rect()
							.horizontal()
							.cross_align(Alignment::Center)
							.spacing(theme.gap2)
							.child(icon("rocket", 18.0).color(theme.primary))
							.child(
								label()
									.text(format!(
										"Доступно обновление Sqrilizz Launcher v{}!",
										info.version
									))
									.font_size(14.0)
									.font_weight(FontWeight::BOLD)
									.color(theme.fg),
							),
					)
					.child(
						rect()
							.horizontal()
							.cross_align(Alignment::Center)
							.spacing(theme.gap)
							.child(
								button(&theme)
									.background(theme.primary)
									.color(Color::WHITE)
									.padding(Gaps::new(6.0, 14.0, 6.0, 14.0))
									.corner_radius(theme.round)
									.on_press(move |_| {
										let info = info_for_btn.clone();
										tokio::spawn(async move {
											let _ = updater::start_update(info).await;
										});
									})
									.child(
										rect()
											.horizontal()
											.cross_align(Alignment::Center)
											.spacing(theme.gap)
											.child(icon("download", 14.0).color(Color::WHITE))
											.child("Обновить в 1 клик"),
									),
							)
							.child(icon_button("cross", &theme).on_press(move |_| {
								let mut s = status_clone.clone();
								tokio::spawn(async move {
									updater::set_status(UpdateStatus::Dismissed).await;
								});
								s.set(UpdateStatus::Dismissed);
							})),
					)
			}
			UpdateStatus::Downloading {
				version,
				progress,
				downloaded,
				total,
			} => {
				let mb_down = downloaded as f64 / 1_048_576.0;
				let mb_total = total as f64 / 1_048_576.0;
				let pct = (progress * 100.0) as u32;

				rect()
					.width(Size::fill())
					.background(theme.navbar)
					.border(border_bottom(2.0, theme.primary))
					.padding(Gaps::new(8.0, 16.0, 10.0, 16.0))
					.vertical()
					.spacing(theme.gap)
					.child(
						rect()
							.horizontal()
							.cross_align(Alignment::Center)
							.main_align(Alignment::SpaceBetween)
							.child(
								rect()
									.horizontal()
									.cross_align(Alignment::Center)
									.spacing(theme.gap2)
									.child(icon("download", 16.0).color(theme.primary))
									.child(
										label()
											.text(format!("Загрузка обновления v{}...", version))
											.font_size(13.0)
											.font_weight(FontWeight::BOLD)
											.color(theme.fg),
									),
							)
							.child(
								label()
									.text(format!("{pct}% ({mb_down:.1} / {mb_total:.1} MB)"))
									.font_size(12.0)
									.color(theme.fg2),
							),
					)
					.child(progress_bar(&theme, progress))
			}
			UpdateStatus::ReadyToRestart { version } => {
				let success_color = Color::from_rgb(34, 197, 94);

				rect()
					.width(Size::fill())
					.background(theme.navbar)
					.border(border_bottom(2.0, success_color))
					.padding(Gaps::new(8.0, 16.0, 8.0, 16.0))
					.horizontal()
					.cross_align(Alignment::Center)
					.main_align(Alignment::SpaceBetween)
					.child(
						rect()
							.horizontal()
							.cross_align(Alignment::Center)
							.spacing(theme.gap2)
							.child(icon("check", 18.0).color(success_color))
							.child(
								label()
									.text(format!("Обновление v{} успешно установлено!", version))
									.font_size(14.0)
									.font_weight(FontWeight::BOLD)
									.color(theme.fg),
							),
					)
					.child(
						button(&theme)
							.background(success_color)
							.color(Color::WHITE)
							.padding(Gaps::new(6.0, 14.0, 6.0, 14.0))
							.corner_radius(theme.round)
							.on_press(move |_| {
								let _ = updater::restart_launcher();
							})
							.child(
								rect()
									.horizontal()
									.cross_align(Alignment::Center)
									.spacing(theme.gap)
									.child(icon("refresh", 14.0).color(Color::WHITE))
									.child("Перезапустить сейчас"),
							),
					)
			}
			UpdateStatus::Error(err) => {
				let status_clone = status.clone();
				let danger_color = Color::from_rgb(239, 68, 68);

				rect()
					.width(Size::fill())
					.background(theme.navbar)
					.border(border_bottom(2.0, danger_color))
					.padding(Gaps::new(6.0, 16.0, 6.0, 16.0))
					.horizontal()
					.cross_align(Alignment::Center)
					.main_align(Alignment::SpaceBetween)
					.child(
						rect()
							.horizontal()
							.cross_align(Alignment::Center)
							.spacing(theme.gap2)
							.child(icon("warning", 16.0).color(danger_color))
							.child(
								label()
									.text(format!("Ошибка обновления: {err}"))
									.font_size(12.0)
									.color(theme.fg),
							),
					)
					.child(icon_button("cross", &theme).on_press(move |_| {
						let mut s = status_clone.clone();
						tokio::spawn(async move {
							updater::set_status(UpdateStatus::Dismissed).await;
						});
						s.set(UpdateStatus::Dismissed);
					}))
			}
			_ => rect(),
		}
	}
}
