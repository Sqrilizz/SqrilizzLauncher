use std::time::Duration;

use crate::{
	ops::{
		misc::{ShowDirectory, ShowDirectoryOption},
		server::{
			BackupWorld, BackupWorldParams, DeleteBackupParams, DeleteServerBackup,
			FetchServerBackups, FetchServerWorlds,
		},
	},
	prelude::*,
};

/// Formats bytes into a readable size
fn format_filesize(bytes: u64) -> String {
	if bytes >= 1024 * 1024 * 1024 {
		format!("{:.2} ГБ", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
	} else if bytes >= 1024 * 1024 {
		format!("{:.1} МБ", bytes as f64 / (1024.0 * 1024.0))
	} else if bytes >= 1024 {
		format!("{} КБ", bytes / 1024)
	} else {
		format!("{bytes} Б")
	}
}

#[derive(PartialEq)]
pub struct ServerWorlds {
	pub id: String,
}

impl Component for ServerWorlds {
	fn render(&self) -> impl IntoElement {
		let theme = use_theme();
		let back_state = use_consume::<BackState>();
		let front_state = use_front_state();

		let worlds_query = use_query(Query::new(
			self.id.clone(),
			FetchServerWorlds::new(back_state.clone()),
		));
		let backups_query = use_query(Query::new(
			self.id.clone(),
			FetchServerBackups::new(back_state.clone()),
		));
		let backup_mutation =
			use_mutation(Mutation::new(BackupWorld::new(back_state.clone()).toast(
				&back_state,
				Some("Бэкап создаётся, это может занять время"),
				"Не удалось создать бэкап",
			)));
		let delete_backup_mutation = use_mutation(Mutation::new(
			DeleteServerBackup::new(back_state.clone()).toast(
				&back_state,
				Some("Бэкап удалён"),
				"Не удалось удалить бэкап",
			),
		));
		let show_directory = use_mutation(Mutation::new(ShowDirectory::new(back_state.clone())));

		// Refresh periodically
		let worlds_query2 = worlds_query.clone();
		let backups_query2 = backups_query.clone();
		use_future(move || {
			let worlds_query2 = worlds_query2;
			let backups_query2 = backups_query2;
			async move {
				loop {
					tokio::time::sleep(Duration::from_secs(8)).await;
					worlds_query2.invalidate();
					backups_query2.invalidate();
				}
			}
		});

		let worlds = worlds_query
			.read()
			.state()
			.ok()
			.cloned()
			.unwrap_or_default();
		let backups = backups_query
			.read()
			.state()
			.ok()
			.cloned()
			.unwrap_or_default();

		let mut list = rect().width(Size::fill()).spacing(theme.gap3);

		// Section: Worlds
		let mut worlds_section = rect().width(Size::fill()).spacing(theme.gap2);
		worlds_section = worlds_section.child(
			rect()
				.width(Size::fill())
				.horizontal()
				.cross_align(Alignment::Center)
				.spacing(theme.gap)
				.child(icon("globe", 20.0).color(theme.primary))
				.child(
					label()
						.text("Миры сервера")
						.font_size(theme.font2)
						.font_weight(FontWeight::BOLD)
						.color(theme.fg),
				)
				.child(
					rect()
						.padding(Gaps::new(2.0, 8.0, 2.0, 8.0))
						.background(theme.item)
						.corner_radius(theme.round)
						.child(
							label()
								.text(format!("{} шт.", worlds.len()))
								.font_size(theme.font0)
								.color(theme.fg3),
						),
				),
		);

		if worlds.is_empty() {
			worlds_section = worlds_section.child(
				rect()
					.width(Size::fill())
					.height(Size::px(120.0))
					.background(theme.item)
					.corner_radius(theme.round)
					.border(theme.border(theme.item_border))
					.center()
					.spacing(theme.gap)
					.child(icon("globe", 36.0).color(theme.fg3))
					.child(
						label()
							.text("Миры не найдены")
							.font_size(theme.font)
							.font_weight(FontWeight::BOLD)
							.color(theme.fg),
					)
					.child(
						label()
							.text("Запустите сервер, чтобы папки миров создались автоматически")
							.font_size(theme.font0)
							.color(theme.fg3),
					),
			);
		} else {
			for world in worlds {
				let id = self.id.clone();
				let world_name = world.name.clone();
				let id_folder = self.id.clone();
				let row = rect()
					.width(Size::fill())
					.height(Size::px(64.0))
					.background(theme.item)
					.corner_radius(theme.round)
					.border(theme.border(theme.item_border))
					.padding(Gaps::new(theme.gap, theme.gap3, theme.gap, theme.gap3))
					.horizontal()
					.cross_align(Alignment::Center)
					.spacing(theme.gap2)
					.child(
						rect()
							.width(Size::px(40.0))
							.height(Size::px(40.0))
							.center()
							.background(theme.item_hover)
							.corner_radius(theme.round)
							.child(icon("globe", 20.0).color(theme.primary)),
					)
					.child(
						rect()
							.width(Size::flex(1.0))
							.spacing(2.0)
							.child(
								label()
									.text(world.name.clone())
									.max_lines(1)
									.text_overflow(TextOverflow::Ellipsis)
									.font_weight(FontWeight::BOLD)
									.font_size(theme.font)
									.color(theme.fg),
							)
							.child(
								label()
									.text(format!(
										"Размер на диске: {}",
										format_filesize(world.size_bytes)
									))
									.font_size(theme.font0)
									.color(theme.fg3),
							),
					)
					.child(
						elem_text_button(icon("copy", 14.0), "Создать бэкап", &theme)
							.active(&theme)
							.on_press(move |_| {
								backup_mutation.mutate(BackupWorldParams {
									instance_id: id.clone(),
									world_name: world_name.clone(),
								});
							}),
					)
					.child(
						elem_text_button(icon("folder", 14.0), "Папка сервера", &theme).on_press(
							move |_| {
								show_directory
									.mutate(ShowDirectoryOption::Instance(id_folder.clone()));
							},
						),
					);

				worlds_section = worlds_section.child(row);
			}
		}
		list = list.child(worlds_section);

		// Section: Backups
		let id_folder2 = self.id.clone();
		let mut backups_section = rect().width(Size::fill()).spacing(theme.gap2);
		backups_section = backups_section.child(
			rect()
				.width(Size::fill())
				.horizontal()
				.cross_align(Alignment::Center)
				.spacing(theme.gap)
				.child(icon("clock", 20.0).color(theme.primary))
				.child(
					label()
						.text("Резервные копии")
						.font_size(theme.font2)
						.font_weight(FontWeight::BOLD)
						.color(theme.fg),
				)
				.child(
					rect()
						.padding(Gaps::new(2.0, 8.0, 2.0, 8.0))
						.background(theme.item)
						.corner_radius(theme.round)
						.child(
							label()
								.text(format!("{} шт.", backups.len()))
								.font_size(theme.font0)
								.color(theme.fg3),
						),
				)
				.child(rect().width(Size::flex(1.0)))
				.child(
					elem_text_button(icon("folder", 14.0), "Папка бэкапов", &theme).on_press(
						move |_| {
							show_directory
								.mutate(ShowDirectoryOption::Instance(id_folder2.clone()));
						},
					),
				),
		);

		if backups.is_empty() {
			backups_section = backups_section.child(
				rect()
					.width(Size::fill())
					.height(Size::px(120.0))
					.background(theme.item)
					.corner_radius(theme.round)
					.border(theme.border(theme.item_border))
					.center()
					.spacing(theme.gap)
					.child(icon("clock", 36.0).color(theme.fg3))
					.child(
						label()
							.text("Бэкапов пока нет")
							.font_size(theme.font)
							.font_weight(FontWeight::BOLD)
							.color(theme.fg),
					)
					.child(
						label()
							.text("Нажмите кнопку «Создать бэкап» рядом с нужным миром выше")
							.font_size(theme.font0)
							.color(theme.fg3),
					),
			);
		} else {
			for backup in backups {
				let id = self.id.clone();
				let backup_name = backup.name.clone();
				let row = rect()
					.width(Size::fill())
					.height(Size::px(54.0))
					.background(theme.item)
					.corner_radius(theme.round)
					.border(theme.border(theme.item_border))
					.padding(Gaps::new(theme.gap, theme.gap3, theme.gap, theme.gap3))
					.horizontal()
					.cross_align(Alignment::Center)
					.spacing(theme.gap2)
					.child(
						rect()
							.width(Size::px(36.0))
							.height(Size::px(36.0))
							.center()
							.background(theme.item_hover)
							.corner_radius(theme.round)
							.child(icon("clock", 16.0).color(theme.primary)),
					)
					.child(
						rect()
							.width(Size::flex(1.0))
							.spacing(2.0)
							.child(
								label()
									.text(backup.name.clone())
									.font_size(theme.font)
									.font_weight(FontWeight::BOLD)
									.color(theme.fg),
							)
							.child(
								label()
									.text(format!(
										"Размер архива: {}",
										format_filesize(backup.size_bytes)
									))
									.font_size(theme.font0)
									.color(theme.fg3),
							),
					)
					.child(
						rect().tip(&front_state, "Удалить бэкап").child(
							icon_button("trash", &theme)
								.color(theme.error)
								.on_press(move |_| {
									delete_backup_mutation.mutate(DeleteBackupParams {
										instance_id: id.clone(),
										backup_name: backup_name.clone(),
									});
								}),
						),
					);

				backups_section = backups_section.child(row);
			}
		}
		list = list.child(backups_section);

		let body = rect().width(Size::fill()).padding(theme.gap3).child(list);

		ScrollView::new()
			.width(Size::fill())
			.height(Size::fill())
			.child(body)
			.into_element()
	}
}
