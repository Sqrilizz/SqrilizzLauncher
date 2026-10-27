use std::{rc::Rc, sync::Arc};

use sqrilizz::shared::util::open_link;

use crate::{
	components::input::{select::Selected, text::{TextInput, search_bar, transparent_text_input}},
	prelude::*,
	util::PtrEq,
};

#[derive(PartialEq)]
pub struct Console<C: ConsoleImpl> {
	pub console: C,
}

impl<C: ConsoleImpl> Component for Console<C> {
	fn render(&self) -> impl IntoElement {
		let theme = use_theme();
		let front_state = use_front_state();

				let contents = use_reactive(&self.console.contents());
		let mut line_indexes = use_state::<Vec<usize>>(Vec::new);
		let mut line_count = use_state::<usize>(|| 0);

		let ty = use_state::<Option<MessageType>>(|| None);
		let input = use_state(String::new);
		let search = use_state(String::new);

		use_side_effect({
			move || {
				let ty = ty.read().clone();

				if let Some(contents) = &*contents.read() {
					let mut all_indexes = Vec::new();
					all_indexes.push(0);

					for (i, ch) in contents.char_indices() {
						if ch == '\n' {
							all_indexes.push(i + 1);
						}
					}

					let search_term = search.read().to_lowercase();
					let mut indexes = Vec::new();
					let mut uppercase_buf = String::new();
					for (i, start) in all_indexes.iter().copied().enumerate() {
						let end = all_indexes.get(i + 1).copied().unwrap_or(contents.len());
						let line = contents.get(start..end).unwrap_or_default().trim_end();

						let matches_search = search_term.is_empty() || line.to_lowercase().contains(&search_term);
						if line_matches_ty(line, ty.as_ref(), &mut uppercase_buf) && matches_search {
							indexes.push(start);
						}
					}

					let count = indexes.len();
					line_indexes.set(indexes);
					line_count.set(count);
				} else {
					line_indexes.set(Vec::new());
					line_count.set(0);
				}
			}
		});

		let theme2 = theme.clone();
		let contents = match self.console.contents() {
			Some(contents) => {
				let indexes = line_indexes;
				VirtualScrollView::new_with_data(PtrEq(contents.clone()), move |item, contents| {
					let indexes_read = indexes.read();
					let line = if item.index < indexes_read.len() {
						let start = indexes_read[item.index];
						let end = if item.index + 1 < indexes_read.len() {
							indexes_read[item.index + 1]
						} else {
							contents.0.len()
						};

						contents.0.get(start..end).unwrap_or_default().trim_end()
					} else {
						""
					};

					format_line(line, &theme2)
				})
				.width(Size::fill())
				.height(Size::flex(1.0))
				.item_size(24.0)
				.length(*line_count.read())
				.into_element()
			}
			None => {
				let pholder = if self.console.is_loading() {
					"Загрузка..."
				} else {
					"Вывод отсутствует"
				};

				rect()
					.width(Size::fill())
					.height(Size::flex(1.0))
					.child(placeholder(pholder, &theme))
					.into_element()
			}
		};

		let input_fn = self.console.input_fn();
		let mut input2 = input;
		let contents = rect()
			.width(Size::fill())
			.height(Size::flex(1.0))
			.background(theme.bg)
			.border(theme.border(theme.panel_border))
			.padding(theme.gap2)
			.corner_radius(theme.round2)
			.flex()
			.child(contents)
			.maybe(
				input_fn.is_some() && self.console.get_log_file().is_none(),
				|this| {
					this.child(
						rect()
							.width(Size::fill())
							.height(Size::px(theme.input_height))
							.border(border_top(theme.border, theme.panel_border))
							.corner_radius(CornerRadius::new(0.0, 0.0, theme.round2, theme.round2))
							.padding(Gaps::new(0.0, theme.gap2, 0.0, theme.gap2))
							.horizontal()
							.cross_align(Alignment::Center)
							.spacing(theme.gap)
							.child(icon("angle_right", 16.0).color(theme.primary))
							.child(
								transparent_text_input(input, &theme)
									.placeholder("Введите команду сервера (например: help, op, list, tps)...")
									.on_submit(move |s| {
										let result = input_fn.as_ref().unwrap()(s);
								if result {
									input2.set(String::new());
								}
							}),
						),
					)
				},
			);

		let ty_selector = Dropdown::new(
			Selected::Single(ty.read().clone()),
			Rc::new(move |new| {
				ty.clone().set(new.single());
			}),
		)
		.header_width(Size::auto())
		.child(SelectOption::new(
			None,
			"Все сообщения",
			Some("speech_bubble"),
		))
		.child(SelectOption::new(
			Some(MessageType::Error),
			"Ошибки",
			Some("error"),
		))
		.child(SelectOption::new(
			Some(MessageType::Warning),
			"Предупреждения",
			Some("warning"),
		))
		.child(SelectOption::new(
			Some(MessageType::Info),
			"Инфо",
			Some("info"),
		));

		let console2 = self.console.clone();
		let file_selector = Dropdown::new(
			Selected::Single(self.console.get_log_file()),
			Rc::new(move |new| {
				console2.set_log_file(new.single());
			}),
		)
		.header_width(Size::auto())
		.options_width(180.0)
		.child(SelectOption::new(None, "Текущий вывод", Some("text")))
		.children(
			self.console
				.get_log_files()
				.map(|x| SelectOption::new(Some(x.clone()), x, Some("text"))),
		);

		let console_contents = self.console.contents().map(|x| x.to_string()).unwrap_or_default();
		let front_state2 = front_state.clone();
		let front_state3 = front_state.clone();
		let console_contents2 = console_contents.clone();

		let upload_button = rect()
			.tip(&front_state, "Загрузить на mclo.gs")
			.child(
				icon_button("upload", &theme).on_press(move |_| {
					if console_contents.trim().is_empty() {
						return;
					}
					let content = console_contents.clone();
					let front_state2 = front_state2.clone();
					spawn_forever(async move {
						let client = reqwest::Client::new();
						match client
							.post("https://api.mclo.gs/1/log")
							.form(&[("content", &content)])
							.send()
							.await
						{
							Ok(resp) => {
								if let Ok(json) = resp.json::<serde_json::Value>().await {
									if let Some(url) = json.get("url").and_then(|u| u.as_str()) {
										let _ = Clipboard::set(url.to_string());
										let _ = open_link(url);
										front_state2
											.write()
											.toast(Toast::success("Лог загружен на mclo.gs (ссылка скопирована)"));
									}
								}
							}
							Err(_) => {
								front_state2
									.write()
									.toast(Toast::error("Ошибка загрузки лога на mclo.gs", None));
							}
						}
					});
				}),
			);

		let copy_button = rect()
			.tip(&front_state, "Скопировать весь лог")
			.child(
				icon_button("copy", &theme).on_press(move |_| {
					let _ = Clipboard::set(console_contents2.clone());
					front_state3
						.write()
						.toast(Toast::info("Лог скопирован в буфер", None));
				}),
			);

		let actions = rect()
			.horizontal()
			.spacing(theme.gap)
			.cross_align(Alignment::Center)
			.child(upload_button)
			.child(copy_button);

		let search_input = search_bar(
			TextInput::new(search).placeholder("Поиск по логу..."),
			&theme,
		);

		let header = rect()
			.width(Size::fill())
			.height(Size::px(theme.input_height))
			.cont()
			.child(segment(ty_selector, 1.0))
			.child(segment(search_input, 2.0))
			.child(
				segment(actions, 0.8)
					.horizontal()
					.center(),
			)
			.child(
				segment(file_selector, 1.2)
					.horizontal()
					.main_align(Alignment::End),
			);

		rect()
			.expanded()
			.flex()
			.padding(theme.gap2)
			.spacing(theme.gap)
			.child(header)
			.child(contents)
	}
}

fn format_line(line: &str, theme: &Theme) -> Element {
	if line.contains('\x1b') {
		return ansi_line(line, theme);
	}

	let (left, ty, right, ty_color) = if let Some((left, right)) = line.split_once("ERROR") {
		(left, "ERROR", right, theme.error)
	} else if let Some((left, right)) = line.split_once("WARN") {
		(left, "WARN", right, theme.warning)
	} else if let Some((left, right)) = line.split_once("INFO") {
		(left, "INFO", right, theme.fg)
	} else {
		return clip_text(line).height(Size::px(24.0)).into_element();
	};

	rect()
		.width(Size::fill())
		.height(Size::px(24.0))
		.horizontal()
		.child(label().text(left.to_string()).max_lines(1))
		.child(label().text(ty).color(ty_color))
		.child(
			label()
				.text(right.to_string())
				.max_lines(1)
				.color(theme.fg3),
		)
		.into_element()
}

/// Renders a line that contains ANSI escape sequences with the embedded colors
fn ansi_line(line: &str, theme: &Theme) -> Element {
	let spans = parse_ansi_spans(line, theme.fg);
	rect()
		.width(Size::fill())
		.height(Size::px(24.0))
		.horizontal()
		.children(spans.into_iter().filter(|(t, _)| !t.is_empty()).map(
			|(text, color)| label().text(text).color(color).max_lines(1).into_element(),
		))
		.into_element()
}

/// Splits a line into (text, color) spans based on ANSI SGR escape sequences
fn parse_ansi_spans(line: &str, default: Color) -> Vec<(String, Color)> {
	let mut out: Vec<(String, Color)> = Vec::new();
	let mut cur = default;
	let mut buf = String::new();
	let mut chars = line.chars().peekable();

	while let Some(c) = chars.next() {
		if c != '\x1b' {
			buf.push(c);
			continue;
		}

		let mut seq = String::new();
		while let Some(&next) = chars.peek() {
			chars.next();
			seq.push(next);
			if next.is_ascii_alphabetic() || next == '~' {
				break;
			}
		}

		if !buf.is_empty() {
			out.push((std::mem::take(&mut buf), cur));
		}
		apply_sgr(&seq, default, &mut cur);
	}

	if !buf.is_empty() {
		out.push((buf, cur));
	}
	out
}

/// Applies an ANSI SGR sequence like `[0m` / `[38;2;255;170;0m` to the current color
fn apply_sgr(seq: &str, default: Color, cur: &mut Color) {
	let Some(body) = seq.strip_prefix('[') else {
		return;
	};
	let body: String = body
		.chars()
		.take_while(|c| c.is_ascii_digit() || *c == ';')
		.collect();
	let vals: Vec<i64> = body
		.split(';')
		.filter_map(|x| x.trim().parse().ok())
		.collect();

	let mut i = 0;
	while i < vals.len() {
		match vals[i] {
			0 | 39 => *cur = default,
			38 if i + 1 < vals.len() => match vals[i + 1] {
				2 if i + 4 < vals.len() => {
					*cur = Color::from_rgb(
						vals[i + 2].clamp(0, 255) as u8,
						vals[i + 3].clamp(0, 255) as u8,
						vals[i + 4].clamp(0, 255) as u8,
					);
					i += 5;
					continue;
				}
				5 if i + 2 < vals.len() => {
					*cur = ansi_256_color(vals[i + 2].clamp(0, 255) as u8);
					i += 3;
					continue;
				}
				_ => {}
			},
			_ => {}
		}
		i += 1;
	}
}

/// Converts an xterm-256 color index to a freya color
fn ansi_256_color(n: u8) -> Color {
	if n < 16 {
		const BASIC: [(u8, u8, u8); 16] = [
			(0, 0, 0),
			(128, 0, 0),
			(0, 128, 0),
			(128, 128, 0),
			(0, 0, 128),
			(128, 0, 128),
			(0, 128, 128),
			(192, 192, 192),
			(128, 128, 128),
			(255, 0, 0),
			(0, 255, 0),
			(255, 255, 0),
			(0, 0, 255),
			(255, 0, 255),
			(0, 255, 255),
			(255, 255, 255),
		];
		let (r, g, b) = BASIC[n as usize];
		Color::from_rgb(r, g, b)
	} else if n < 232 {
		let c = n - 16;
		let step = |v: u8| if v == 0 { 0 } else { 55 + 40 * v };
		Color::from_rgb(step(c / 36), step((c % 36) / 6), step(c % 6))
	} else {
		let v = 8 + 10 * (n - 232);
		Color::from_rgb(v, v, v)
	}
}

pub trait ConsoleImpl: PartialEq + Clone + 'static {
	fn contents(&self) -> Option<Arc<str>>;

	fn is_loading(&self) -> bool {
		false
	}

	fn get_log_files(&self) -> impl Iterator<Item = &String>;

	fn get_log_file(&self) -> Option<String>;

	fn set_log_file(&self, file: Option<String>);

	fn input_fn(&self) -> Option<impl Fn(String) -> bool + 'static> {
		None::<Box<dyn Fn(String) -> bool>>
	}
}

#[derive(PartialEq, Clone)]
enum MessageType {
	Info,
	Warning,
	Error,
}

fn line_matches_ty(line: &str, ty: Option<&MessageType>, uppercase_buf: &mut String) -> bool {
	match ty {
		None => true,
		Some(ty) => line_type(line, uppercase_buf).as_ref() == Some(ty),
	}
}

fn line_type(line: &str, uppercase_buf: &mut String) -> Option<MessageType> {
	uppercase_buf.clear();
	if line.len() > 32 {
		uppercase_buf.push_str(&line[..32]);
	} else {
		uppercase_buf.push_str(line);
	}
	uppercase_buf.make_ascii_uppercase();

	if uppercase_buf.contains("ERROR") || uppercase_buf.contains("[ERR]") {
		Some(MessageType::Error)
	} else if uppercase_buf.contains("WARN") {
		Some(MessageType::Warning)
	} else if uppercase_buf.contains("INFO") {
		Some(MessageType::Info)
	} else {
		None
	}
}
