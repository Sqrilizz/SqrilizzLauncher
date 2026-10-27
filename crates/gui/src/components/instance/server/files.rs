use std::time::{SystemTime, UNIX_EPOCH};

use super::code_editor::CodeEditor;

use crate::{
	components::{
		border_bottom, border_top,
		dialog::modal::{Modal, ModalButton},
		input::text::{TextInput, search_bar},
	},
	ops::QueryOption,
	ops::instance::{
		CreateInstanceEntry, CreateInstanceEntryParams, DeleteInstanceEntry,
		DeleteInstanceEntryParams, FetchInstanceDirFiles, InstanceFileKey, ReadInstanceFile,
		WriteInstanceFile, WriteInstanceFileParams,
	},
	prelude::*,
};

fn format_size(bytes: u64) -> String {
	if bytes >= 1024 * 1024 {
		format!("{:.1} МБ", bytes as f64 / (1024.0 * 1024.0))
	} else if bytes >= 1024 {
		format!("{:.1} КБ", bytes as f64 / 1024.0)
	} else {
		format!("{bytes} Б")
	}
}

fn format_modified(secs: u64) -> String {
	if secs == 0 {
		return String::new();
	}
	let now = SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.map(|d| d.as_secs())
		.unwrap_or(secs);
	let delta = now.saturating_sub(secs);
	if delta < 60 {
		"только что".into()
	} else if delta < 3600 {
		format!("{} мин назад", delta / 60)
	} else if delta < 86400 {
		format!("{} ч назад", delta / 3600)
	} else {
		format!("{} дн назад", delta / 86400)
	}
}

fn get_file_lang(name: &str) -> &'static str {
	let lower = name.to_lowercase();
	if lower.ends_with(".yml") || lower.ends_with(".yaml") {
		"YAML"
	} else if lower.ends_with(".json") {
		"JSON"
	} else if lower.ends_with(".properties") {
		"Properties"
	} else if lower.ends_with(".toml") {
		"TOML"
	} else if lower.ends_with(".cfg") || lower.ends_with(".conf") || lower.ends_with(".ini") {
		"Config"
	} else if lower.ends_with(".sh") || lower.ends_with(".bash") {
		"Shell Script"
	} else if lower.ends_with(".bat") || lower.ends_with(".cmd") {
		"Batch"
	} else if lower.ends_with(".log") {
		"Log"
	} else if lower.ends_with(".xml") {
		"XML"
	} else if lower.ends_with(".md") {
		"Markdown"
	} else {
		"Plain Text"
	}
}

fn get_file_icon_color(name: &str) -> Color {
	let lower = name.to_lowercase();
	if lower.ends_with(".yml") || lower.ends_with(".yaml") {
		Color::from_rgb(245, 158, 11)
	} else if lower.ends_with(".json") {
		Color::from_rgb(56, 189, 248)
	} else if lower.ends_with(".properties") || lower.ends_with(".toml") || lower.ends_with(".cfg") {
		Color::from_rgb(192, 132, 252)
	} else if lower.ends_with(".sh") || lower.ends_with(".bat") {
		Color::from_rgb(16, 185, 129)
	} else if lower.ends_with(".log") {
		Color::from_rgb(251, 146, 60)
	} else {
		Color::from_rgb(148, 163, 184)
	}
}

fn is_editable_text_file(name: &str) -> bool {
	let lower = name.to_lowercase();
	!lower.ends_with(".jar")
		&& !lower.ends_with(".zip")
		&& !lower.ends_with(".gz")
		&& !lower.ends_with(".png")
		&& !lower.ends_with(".jpg")
		&& !lower.ends_with(".jpeg")
		&& !lower.ends_with(".exe")
}


#[derive(PartialEq)]
pub struct ServerFiles {
	pub id: String,
}

impl Component for ServerFiles {
	fn render(&self) -> impl IntoElement {
		let theme = use_theme();
		let back_state = use_consume::<BackState>();
		let front_state = use_front_state();

		let current_path = use_state(String::new);
		let search = use_state(String::new);
		let create_modal = use_state::<Option<bool>>(|| None);
		let create_name = use_state(String::new);
		let edit_file = use_state::<Option<String>>(|| None);
		let edit_content = use_state(String::new);
		let delete_target = use_state::<Option<String>>(|| None);

		let dir_key = InstanceFileKey::new(self.id.clone(), current_path.read().clone());
		let dir_query = use_query(Query::new(
			dir_key.clone(),
			FetchInstanceDirFiles::new(back_state.clone()),
		));

		let edit_key = edit_file
			.read()
			.clone()
			.map(|name| dir_key.child(&name));
		let read_query = use_query(Query::new(
			QueryOption(edit_key),
			ReadInstanceFile::new(back_state.clone()),
		));

		let write_mutation = use_mutation(Mutation::new(
			WriteInstanceFile::new(back_state.clone()).toast(
				&back_state,
				Some("Файл сохранён"),
				"Не удалось сохранить файл",
			),
		));
		let create_mutation = use_mutation(Mutation::new(
			CreateInstanceEntry::new(back_state.clone()).toast(
				&back_state,
				Some("Создано"),
				"Не удалось создать файл или папку",
			),
		));
		let delete_mutation = use_mutation(Mutation::new(
			DeleteInstanceEntry::new(back_state.clone()).toast(
				&back_state,
				Some("Удалено"),
				"Не удалось удалить",
			),
		));

		// Reactive sync of editor contents when read_query arrives
		let mut edit_content_setter = edit_content;
		let mut loaded_file_tracker = use_state::<Option<(String, Option<String>)>>(|| None);

		use_side_effect(move || {
			let current_edit_file = edit_file.read().clone();
			let query_read = read_query.read();
			let file_contents = query_read.state().ok().cloned().flatten();

			if let Some(filename) = current_edit_file {
				let tracker = loaded_file_tracker.read().clone();
				let content_str = file_contents.as_ref().map(|x| x.to_string());

				if tracker.as_ref().map(|(f, _)| f) != Some(&filename) {
					if let Some(ref text) = content_str {
						edit_content_setter.set(text.clone());
						loaded_file_tracker.set(Some((filename, Some(text.clone()))));
					} else {
						edit_content_setter.set(String::new());
						loaded_file_tracker.set(Some((filename, None)));
					}
				} else if tracker.as_ref().and_then(|(_, c)| c.as_ref()).is_none() && content_str.is_some() {
					if let Some(ref text) = content_str {
						edit_content_setter.set(text.clone());
						loaded_file_tracker.set(Some((filename, Some(text.clone()))));
					}
				}
			} else {
				if loaded_file_tracker.read().is_some() {
					loaded_file_tracker.set(None);
					edit_content_setter.set(String::new());
				}
			}
		});

		let mut entries: Vec<_> = dir_query
			.read()
			.state()
			.ok()
			.cloned()
			.map(|x| x.to_vec())
			.unwrap_or_default();

		let filter = search.read().to_lowercase();
		if !filter.is_empty() {
			entries.retain(|e| e.name.to_lowercase().contains(&filter));
		}
		entries.sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then(a.name.to_lowercase().cmp(&b.name.to_lowercase())));

		// Breadcrumbs
		let mut crumbs = rect()
			.width(Size::fill())
			.height(Size::px(theme.input_height))
			.horizontal()
			.spacing(theme.gap)
			.cross_align(Alignment::Center);
		{
			let mut root_setter = current_path;
			let root_btn = icon_text_button("home", "Инстанс", &theme)
				.on_press(move |_| root_setter.set(String::new()));
			crumbs = crumbs.child(root_btn);

			let parts: Vec<String> = current_path
				.read()
				.split('/')
				.filter(|p| !p.is_empty())
				.map(|p| p.to_string())
				.collect();
			let mut acc = String::new();
			for part in parts {
				if !acc.is_empty() {
					acc.push('/');
				}
				acc.push_str(&part);
				let acc2 = acc.clone();
				let mut setter = current_path;
				crumbs = crumbs
					.child(icon("angle_right", 12.0))
					.child(
						icon_text_button("folder", &part, &theme)
							.on_press(move |_| setter.set(acc2.clone())),
					);
			}
		}

		// Toolbar
		let toolbar = rect()
			.width(Size::fill())
			.horizontal()
			.spacing(theme.gap)
			.cross_align(Alignment::Center)
			.child(segment(
				search_bar(
					TextInput::new(search).placeholder("Поиск файлов..."),
					&theme,
				),
				1.0,
			))
			.child({
				let mut setter = create_modal;
				icon_text_button("file_cross", "Файл", &theme).on_press(move |_| setter.set(Some(false)))
			})
			.child({
				let mut setter = create_modal;
				icon_text_button("folder", "Папка", &theme).on_press(move |_| setter.set(Some(true)))
			})
			.child({
				let q = dir_query.clone();
				rect()
					.tip(&front_state, "Обновить")
					.child(icon_button("refresh", &theme).on_press(move |_| q.invalidate()))
			});

		// File rows
		let dir_names: Vec<(String, bool)> = entries
			.iter()
			.map(|e| (e.name.clone(), e.is_dir))
			.collect();

		let mut rows = rect()
			.width(Size::fill())
			.spacing(theme.gap);

		if entries.is_empty() {
			rows = rows.child(
				rect()
					.width(Size::fill())
					.height(Size::px(80.0))
					.center()
					.child(
						label()
							.text(if filter.is_empty() {
								"Папка пуста".to_string()
							} else {
								"Ничего не найдено".to_string()
							})
							.color(theme.fg3),
					),
			);
		}

		for entry in entries {
			rows = rows.child(FileRow {
				name: entry.name.clone(),
				is_dir: entry.is_dir,
				size: entry.size,
				modified_secs: entry.modified_secs,
				current_path,
				edit_file,
				delete_target,
			});
		}

		// Create modal
		let create_modal_elem = if let Some(is_dir) = create_modal.read().clone() {
			let mut close_setter = create_modal;
			let mut name_state = create_name;
			let mut dir_setter = current_path;
			let query = dir_query;
			let create_mut = create_mutation;
			let inst_id = self.id.clone();
			let base = current_path.read().clone();
			let fs = front_state.clone();

			Some(
				Modal::new(
					if is_dir {
						"Новая папка".to_string()
					} else {
						"Новый файл".to_string()
					},
					if is_dir { "folder" } else { "file_cross" }.into(),
				)
				.size(480.0, 260.0)
				.on_close(move |_| close_setter.set(None))
				.cancel_button()
				.button(ModalButton {
					title: "Создать".into(),
					icon: "plus".into(),
					active: true,
					danger: false,
					on_click: (move |_| {
						let raw = name_state.read().trim().to_string();
						let name: String = raw
							.chars()
							.filter(|c| *c != '/' && *c != '\\')
							.collect();
						if name.is_empty() {
							fs.write()
								.toast(Toast::warning("Введите название", None));
							return;
						}
						let path = if base.is_empty() {
							name.clone()
						} else {
							format!("{base}/{name}")
						};
						create_mut.mutate(CreateInstanceEntryParams {
							instance_id: inst_id.clone(),
							path,
							is_dir,
						});
						name_state.set(String::new());
						close_setter.set(None);
						if is_dir {
							dir_setter.set(if base.is_empty() {
								name
							} else {
								format!("{base}/{name}")
							});
						}
						query.invalidate();
					})
					.into(),
				})
				.child(
					rect()
						.width(Size::fill())
						.padding(theme.gap3)
						.spacing(theme.gap2)
						.child(
							label()
								.text("Название")
								.font_size(theme.font0)
								.color(theme.fg2),
						)
						.child(TextInput::new(create_name).placeholder("example.txt")),
				),
			)
		} else {
			None
		};

		// VS Code-Like Editor Modal
		let editor_modal = if let Some(name) = edit_file.read().clone() {
			let mut close_setter = edit_file;
			let content_state = edit_content;
			let save_mut = write_mutation;
			let q = read_query.clone();
			let fs = front_state.clone();
			let inst_id = self.id.clone();
			let path = dir_key.child(&name);
			let title = format!("VS Code Editor — {name}");

			let content_str = content_state.read().clone();
			let char_count = content_str.len();
			let line_count = content_str.lines().count().max(1);
			let lang = get_file_lang(&name);
			let icon_color = get_file_icon_color(&name);
			let full_path = if current_path.read().is_empty() {
				format!("/{name}")
			} else {
				format!("/{}/{}", current_path.read(), name)
			};

			let tab_bar = rect()
				.width(Size::fill())
				.height(Size::px(35.0))
				.background(Color::from_rgb(37, 37, 38))
				.border(border_bottom(1.0, Color::from_rgb(45, 45, 48)))
				.horizontal()
				.main_align(Alignment::SpaceBetween)
				.cross_align(Alignment::Center)
				.child(
					rect()
						.height(Size::fill())
						.horizontal()
						.cross_align(Alignment::Center)
						.spacing(12.0)
						.child(
							rect()
								.height(Size::fill())
								.background(Color::from_rgb(30, 30, 30))
								.border(border_top(2.0, Color::from_rgb(0, 122, 204)))
								.padding(Gaps::new(0.0, 14.0, 0.0, 14.0))
								.horizontal()
								.cross_align(Alignment::Center)
								.spacing(8.0)
								.child(icon("text", 13.0).color(icon_color))
								.child(
									label()
										.text(name.clone())
										.font_size(12.0)
										.font_weight(FontWeight::BOLD)
										.max_lines(1)
										.color(Color::from_rgb(240, 240, 240)),
								),
						)
						.child(
							rect()
								.horizontal()
								.cross_align(Alignment::Center)
								.spacing(4.0)
								.child(icon("folder", 12.0).color(Color::from_rgb(130, 130, 130)))
								.child(
									label()
										.text(full_path)
										.font_size(11.0)
										.max_lines(1)
										.color(Color::from_rgb(150, 150, 150)),
								),
						),
				)
				.child(
					rect()
						.padding(Gaps::new(0.0, 14.0, 0.0, 0.0))
						.horizontal()
						.cross_align(Alignment::Center)
						.spacing(6.0)
						.child(
							rect()
								.height(Size::px(22.0))
								.padding(Gaps::new(0.0, 8.0, 0.0, 8.0))
								.background(Color::from_rgb(50, 50, 53))
								.corner_radius(3.0)
								.horizontal()
								.cross_align(Alignment::Center)
								.spacing(4.0)
								.child(icon("text", 11.0).color(Color::from_rgb(156, 220, 254)))
								.child(
									label()
										.text(lang)
										.font_size(11.0)
										.font_weight(FontWeight::BOLD)
										.max_lines(1)
										.color(Color::WHITE),
								),
						),
				);

			let query_read = read_query.read();
			let query_error = query_read.state().err().map(|e| e.to_string());
			let query_data = query_read.state().ok().cloned().flatten();
			let is_loading = query_read.state().is_ok() && query_data.is_none();

			let editor_body = if let Some(err) = query_error {
				rect()
					.width(Size::fill())
					.height(Size::flex(1.0))
					.background(Color::from_rgb(30, 30, 30))
					.center()
					.spacing(12.0)
					.child(icon("warning", 32.0).color(Color::from_rgb(245, 158, 11)))
					.child(
						label()
							.text(format!("Не удалось открыть файл: {err}"))
							.font_size(theme.font)
							.font_weight(FontWeight::BOLD)
							.color(Color::from_rgb(240, 240, 240)),
					)
					.into_element()
			} else {
				CodeEditor {
					value: edit_content,
					lang: lang.to_string(),
					placeholder: if is_loading {
						"Загрузка содержимого файла...".to_string()
					} else {
						"Пустой файл (начните вводить текст)".to_string()
					},
				}
				.into_element()
			};

			let status_bar = rect()
				.width(Size::fill())
				.height(Size::px(24.0))
				.background(Color::from_rgb(0, 122, 204))
				.padding(Gaps::new(0.0, 12.0, 0.0, 12.0))
				.horizontal()
				.main_align(Alignment::SpaceBetween)
				.cross_align(Alignment::Center)
				.child(
					rect()
						.horizontal()
						.cross_align(Alignment::Center)
						.spacing(14.0)
						.child(
							rect()
								.horizontal()
								.cross_align(Alignment::Center)
								.spacing(4.0)
								.child(icon("check", 11.0).color(Color::WHITE))
								.child(
									label()
										.text("Готов")
										.font_size(11.0)
										.max_lines(1)
										.color(Color::WHITE),
								),
						)
						.child(
							label()
								.text(format!("Строк: {line_count}"))
								.font_size(11.0)
								.max_lines(1)
								.color(Color::WHITE),
						)
						.child(
							label()
								.text(format!("Символов: {char_count}"))
								.font_size(11.0)
								.max_lines(1)
								.color(Color::WHITE),
						),
				)
				.child(
					rect()
						.horizontal()
						.cross_align(Alignment::Center)
						.spacing(14.0)
						.child(
							label()
								.text("UTF-8")
								.font_size(11.0)
								.max_lines(1)
								.color(Color::WHITE),
						)
						.child(
							label()
								.text("LF")
								.font_size(11.0)
								.max_lines(1)
								.color(Color::WHITE),
						)
						.child(
							label()
								.text(lang)
								.font_size(11.0)
								.font_weight(FontWeight::BOLD)
								.max_lines(1)
								.color(Color::WHITE),
						),
				);

			let vscode_window = rect()
				.width(Size::fill())
				.height(Size::fill())
				.flex()
				.background(Color::from_rgb(30, 30, 30))
				.border(Border::new().width(1.0).fill(Color::from_rgb(45, 45, 48)))
				.corner_radius(theme.round)
				.child(tab_bar)
				.child(editor_body)
				.child(status_bar);

			Some(
				Modal::new(title, "edit".into())
					.size(980.0, 640.0)
					.on_close(move |_| close_setter.set(None))
					.cancel_button()
					.button(ModalButton {
						title: "Сохранить файл".into(),
						icon: "check".into(),
						active: true,
						danger: false,
						on_click: (move |_| {
							save_mut.mutate(WriteInstanceFileParams {
								instance_id: inst_id.clone(),
								path: path.path.clone(),
								contents: content_state.read().clone(),
							});
							q.invalidate();
							fs.write().toast(Toast::success("Файл успешно сохранён"));
						})
						.into(),
					})
					.child(
						rect()
							.width(Size::fill())
							.height(Size::fill())
							.padding(theme.gap)
							.child(vscode_window),
					),
			)
		} else {
			None
		};

		// Delete confirm modal
		let delete_modal = if let Some(name) = delete_target.read().clone() {
			let mut close_setter = delete_target;
			let del_mut = delete_mutation;
			let query = dir_query;
			let inst_id = self.id.clone();
			let path = dir_key.child(&name);
			let is_dir_target = dir_names
				.iter()
				.find(|(n, _)| *n == name)
				.map(|(_, d)| *d)
				.unwrap_or(false);

			Some(
				Modal::simple_confirm(
					"Удалить",
					"trash",
					rect().expanded().center().padding(theme.gap2).child(
						label()
							.text(format!(
								"Вы уверены, что хотите удалить «{name}»{}?",
								if is_dir_target { " и всё содержимое" } else { "" }
							))
							.font_size(theme.font0)
							.color(theme.fg),
					),
					true,
					move |_| close_setter.set(None),
					move |_| {
						close_setter.set(None);
						del_mut.mutate(DeleteInstanceEntryParams {
							instance_id: inst_id.clone(),
							path: path.path.clone(),
						});
						query.invalidate();
					},
				)
				.into_element(),
			)
		} else {
			None
		};

		let mut root = rect()
			.width(Size::fill())
			.height(Size::fill())
			.padding(theme.gap2)
			.spacing(theme.gap2)
			.child(crumbs)
			.child(toolbar)
			.child(
				ScrollView::new()
					.width(Size::fill())
					.height(Size::fill())
					.child(rows),
			);

		if let Some(modal) = editor_modal {
			root = root.child(modal.into_element());
		}
		if let Some(modal) = create_modal_elem {
			root = root.child(modal.into_element());
		}
		if let Some(modal) = delete_modal {
			root = root.child(modal);
		}

		root
	}
}

#[derive(PartialEq)]
struct FileRow {
	name: String,
	is_dir: bool,
	size: u64,
	modified_secs: u64,
	current_path: State<String>,
	edit_file: State<Option<String>>,
	delete_target: State<Option<String>>,
}

impl Component for FileRow {
	fn render(&self) -> impl IntoElement {
		let theme = use_theme();
		let is_hovered = use_state(|| false);

		let meta = if self.is_dir {
			"Папка".to_string()
		} else {
			let m = format_modified(self.modified_secs);
			if m.is_empty() {
				format_size(self.size)
			} else {
				format!("{} • {}", format_size(self.size), m)
			}
		};

		let mut row_actions = rect()
			.horizontal()
			.spacing(theme.gap)
			.cross_align(Alignment::Center);

		if !self.is_dir && is_editable_text_file(&self.name) {
			let mut edit_setter = self.edit_file;
			let name = self.name.clone();
			row_actions = row_actions.child(
				icon_button("edit", &theme).on_press(move |_| edit_setter.set(Some(name.clone()))),
			);
		}

		let mut delete_setter = self.delete_target;
		let name = self.name.clone();
		row_actions = row_actions.child(
			icon_button("trash", &theme).on_press(move |_| delete_setter.set(Some(name.clone()))),
		);

		let mut nav_setter = self.current_path;
		let mut open_setter = self.edit_file;
		let child_path = if self.current_path.read().is_empty() {
			self.name.clone()
		} else {
			format!("{}/{}", self.current_path.read(), self.name)
		};
		let entry_name = self.name.clone();
		let is_dir = self.is_dir;

		rect()
			.width(Size::fill())
			.height(Size::px(48.0))
			.padding(Gaps::new(theme.gap, theme.gap2, theme.gap, theme.gap2))
			.border(theme.border(theme.item_border))
			.corner_radius(theme.round)
			.background(theme.item)
			.horizontal()
			.spacing(theme.gap2)
			.cross_align(Alignment::Center)
			.hover(is_hovered)
			.item_colorway(&theme, *is_hovered.read(), false)
			.clickable()
			.on_press(move |_| {
				if is_dir {
					nav_setter.set(child_path.clone());
				} else if is_editable_text_file(&entry_name) {
					open_setter.set(Some(entry_name.clone()));
				}
			})
			.child(icon(if is_dir { "folder" } else { "text" }, 16.0))
			.child(
				rect()
					.width(Size::flex(1.0))
					.child(
						label()
							.text(self.name.clone())
							.color(theme.fg)
							.font_size(theme.font0)
							.max_lines(1)
							.text_overflow(TextOverflow::Ellipsis),
					)
					.child(
						label()
							.text(meta)
							.color(theme.fg3)
							.font_size(theme.font0)
							.max_lines(1),
					),
			)
			.child(row_actions)
	}
}
