use freya::prelude::*;
use freya::text_edit::*;
use freya_core::elements::paragraph::{ParagraphCursorExt, ParagraphHolder, ParagraphHolderInner, Span};
use freya_core::prelude::TextStyleExt;

use crate::components::border_right;

pub fn tokenize_line_to_spans<'a>(line: &'a str, lang: &str) -> Vec<(String, Color)> {
	let mut tokens = Vec::new();

	let (content, nl) = if let Some(stripped) = line.strip_suffix("\r\n") {
		(stripped, "\r\n")
	} else if let Some(stripped) = line.strip_suffix('\n') {
		(stripped, "\n")
	} else {
		(line, "")
	};

	let trimmed = content.trim_start();
	let indent_len = content.len() - trimmed.len();
	if indent_len > 0 {
		tokens.push((content[..indent_len].to_string(), Color::from_rgb(212, 212, 212)));
	}

	if trimmed.is_empty() {
		if !nl.is_empty() {
			tokens.push((nl.to_string(), Color::from_rgb(212, 212, 212)));
		}
		return tokens;
	}

	// Comments
	if trimmed.starts_with('#') || trimmed.starts_with("//") {
		tokens.push((trimmed.to_string(), Color::from_rgb(106, 153, 85)));
		if !nl.is_empty() {
			tokens.push((nl.to_string(), Color::from_rgb(212, 212, 212)));
		}
		return tokens;
	}

	match lang {
		"yaml" | "yml" => {
			let mut rem = trimmed;
			if rem.starts_with("- ") {
				tokens.push(("- ".to_string(), Color::from_rgb(197, 134, 192)));
				rem = &rem[2..];
			}
			if let Some(colon_pos) = rem.find(':') {
				let key = &rem[..colon_pos];
				tokens.push((key.to_string(), Color::from_rgb(156, 220, 254)));
				tokens.push((":".to_string(), Color::from_rgb(255, 215, 0)));

				let val = &rem[colon_pos + 1..];
				let val_trimmed = val.trim_start();
				let val_indent = val.len() - val_trimmed.len();
				if val_indent > 0 {
					tokens.push((val[..val_indent].to_string(), Color::from_rgb(212, 212, 212)));
				}
				if !val_trimmed.is_empty() {
					tokens.push((val_trimmed.to_string(), color_for_val(val_trimmed)));
				}
			} else {
				tokens.push((rem.to_string(), Color::from_rgb(212, 212, 212)));
			}
		}
		"properties" | "conf" | "cfg" | "ini" => {
			if let Some(eq_pos) = trimmed.find('=') {
				let key = &trimmed[..eq_pos];
				tokens.push((key.to_string(), Color::from_rgb(156, 220, 254)));
				tokens.push(("=".to_string(), Color::from_rgb(255, 215, 0)));
				let val = &trimmed[eq_pos + 1..];
				if !val.is_empty() {
					tokens.push((val.to_string(), color_for_val(val)));
				}
			} else {
				tokens.push((trimmed.to_string(), Color::from_rgb(212, 212, 212)));
			}
		}
		"json" => {
			if trimmed.starts_with('"') {
				if let Some(end_q) = trimmed[1..].find('"') {
					let key = &trimmed[..end_q + 2];
					let after = &trimmed[end_q + 2..];
					if let Some(colon_pos) = after.find(':') {
						tokens.push((key.to_string(), Color::from_rgb(156, 220, 254)));
						if colon_pos > 0 {
							tokens.push((after[..colon_pos].to_string(), Color::from_rgb(212, 212, 212)));
						}
						tokens.push((":".to_string(), Color::from_rgb(255, 215, 0)));
						let val = &after[colon_pos + 1..];
						let val_trimmed = val.trim_start();
						let val_indent = val.len() - val_trimmed.len();
						if val_indent > 0 {
							tokens.push((val[..val_indent].to_string(), Color::from_rgb(212, 212, 212)));
						}
						if !val_trimmed.is_empty() {
							tokens.push((val_trimmed.to_string(), color_for_val(val_trimmed)));
						}
					} else {
						tokens.push((trimmed.to_string(), Color::from_rgb(212, 212, 212)));
					}
				} else {
					tokens.push((trimmed.to_string(), Color::from_rgb(212, 212, 212)));
				}
			} else {
				tokens.push((trimmed.to_string(), Color::from_rgb(212, 212, 212)));
			}
		}
		_ => {
			tokens.push((trimmed.to_string(), Color::from_rgb(212, 212, 212)));
		}
	}

	if !nl.is_empty() {
		tokens.push((nl.to_string(), Color::from_rgb(212, 212, 212)));
	}

	tokens
}

fn color_for_val(val: &str) -> Color {
	let v = val.trim();
	if (v.starts_with('"') && v.ends_with('"')) || (v.starts_with('\'') && v.ends_with('\'')) {
		Color::from_rgb(206, 145, 120) // Orange string
	} else if v == "true" || v == "false" || v == "null" || v == "yes" || v == "no" || v == "on" || v == "off" {
		Color::from_rgb(86, 156, 214) // Blue keyword
	} else if v.parse::<f64>().is_ok() {
		Color::from_rgb(181, 206, 168) // Light green number
	} else {
		Color::from_rgb(212, 212, 212)
	}
}

pub fn tokenize_full_text(text: &str, lang: &str) -> Vec<(String, Color)> {
	let mut all_spans = Vec::new();
	let mut start = 0;
	let bytes = text.as_bytes();

	while start < bytes.len() {
		let next_nl = bytes[start..].iter().position(|&b| b == b'\n');
		let line = match next_nl {
			Some(idx) => &text[start..start + idx + 1],
			None => &text[start..],
		};
		let line_spans = tokenize_line_to_spans(line, lang);
		all_spans.extend(line_spans);
		start += line.len();
	}

	all_spans
}

#[derive(PartialEq)]
pub struct CodeEditor {
	pub value: State<String>,
	pub lang: String,
	pub placeholder: String,
}

impl Component for CodeEditor {
	fn render(&self) -> impl IntoElement {
		let a11y_id = use_hook(|| AccessibilityId::new_unique());
		let focus = use_focus(a11y_id);
		let holder = use_state(ParagraphHolder::default);
		let mut area = use_state(Area::default);
		let mut viewport_area = use_state(Area::default);
		let mut scroll_controller = use_scroll_controller(ScrollConfig::default);
		let mut is_dragging = use_state(|| false);
		let mut value = self.value.clone();

		let mut editable = use_editable(
			|| self.value.read().to_string(),
			move || EditableConfig::new(),
		);

		if *value.read() != editable.editor().read().committed_text() {
			let mut editor = editable.editor_mut().write();
			editor.clear_preedit();
			editor.set(&value.read());
			editor.editor_history_mut().clear();
			editor.clear_selection();
		}

		let mut follow_cursor = move || {
			if !a11y_id.is_focused() {
				return;
			}

			let holder_peek = holder.peek();
			let holder_borrow = holder_peek.0.borrow();
			let Some(ParagraphHolderInner {
				paragraph,
				scale_factor,
			}) = holder_borrow.as_ref()
			else {
				return;
			};

			let viewport = viewport_area();
			if viewport.width() == 0.0 {
				return;
			}

			let editor = editable.editor().peek();
			let text = editor.rope().to_string();
			let cursor_rect = paragraph.cursor_rect(&text, editor.cursor_pos(), TextAlign::Left);
			let cursor_top = cursor_rect.top / (*scale_factor as f32);
			let cursor_bottom = cursor_rect.bottom / (*scale_factor as f32);
			let visible_start_y = viewport.min_y() - area.peek().min_y();

			if cursor_top < visible_start_y {
				scroll_controller.scroll_to_y(-cursor_top as i32);
			} else if cursor_bottom + 16.0 > visible_start_y + viewport.height() {
				scroll_controller.scroll_to_y(-(cursor_bottom + 16.0 - viewport.height()) as i32);
			}
		};

		let on_key_down = move |e: Event<KeyboardEventData>| {
			let key = e.key.clone();
			let modifiers = e.modifiers;

			match &key {
				Key::Named(NamedKey::Escape) => {
					a11y_id.request_unfocus();
				}
				Key::Named(NamedKey::Tab) => {
					e.stop_propagation();
					e.prevent_default();
					editable.process_event(EditableEvent::KeyDown {
						key: &Key::Character("  ".to_string()),
						modifiers,
						editor_line: Some(EditorLine::SingleParagraph),
						holder: Some(&holder.read()),
					});
					let text = editable.editor().read().committed_text();
					*value.write() = text;
					follow_cursor();
				}
				_ => {
					e.stop_propagation();
					e.prevent_default();
					let prev_version = editable.editor().peek().editor_history().version;
					editable.process_event(EditableEvent::KeyDown {
						key: &key,
						modifiers,
						editor_line: Some(EditorLine::SingleParagraph),
						holder: Some(&holder.read()),
					});
					let text = editable.editor().read().committed_text();
					*value.write() = text;
					if editable.editor().peek().editor_history().version != prev_version {
						follow_cursor();
					}
				}
			}
		};

		let on_key_up = move |e: Event<KeyboardEventData>| {
			e.stop_propagation();
			editable.process_event(EditableEvent::KeyUp { key: &e.key });
		};

		let on_ime_preedit = move |e: Event<ImePreeditEventData>| {
			let mut editor = editable.editor_mut().write();
			if e.data().text.is_empty() {
				editor.clear_preedit();
			} else {
				editor.set_preedit(&e.data().text);
			}
		};

		let on_focus_press = move |e: Event<FocusPressEventData>| {
			e.stop_propagation();
			e.prevent_default();
			is_dragging.set(true);
			editable.process_event(EditableEvent::Down {
				location: e.element_location(),
				editor_line: EditorLine::SingleParagraph,
				holder: &holder.read(),
			});
			a11y_id.request_focus();
		};

		let on_global_pointer_move = move |e: Event<PointerEventData>| {
			if a11y_id.is_focused() && *is_dragging.read() {
				let min_x = area.read().min_x() as f64 + 12.0;
				let min_y = area.read().min_y() as f64 + 8.0;
				let location = CursorPoint::new(e.global_location().x - min_x, e.global_location().y - min_y);
				editable.process_event(EditableEvent::Move {
					location,
					editor_line: EditorLine::SingleParagraph,
					holder: &holder.read(),
				});
				follow_cursor();
			}
		};

		let on_global_pointer_press = move |_: Event<PointerEventData>| {
			if a11y_id.is_focused() {
				if *is_dragging.read() {
					is_dragging.set(false);
				} else {
					a11y_id.request_unfocus();
				}
			}
		};

		let on_paragraph_sized = move |e: Event<SizedEventData>| {
			let text_size_changed = area.peek().size != e.area.size;
			area.set_if_modified(e.area);
			if text_size_changed {
				follow_cursor();
			}
		};

		let text = editable.editor().read().rope().to_string();
		let is_empty = text.is_empty();
		let line_count = text.lines().count().max(1);

		let (cursor_index, text_selection) = if focus().is_focused() {
			(
				Some(editable.editor().read().cursor_pos()),
				editable.editor().read().get_visible_selection(EditorLine::SingleParagraph),
			)
		} else {
			(None, None)
		};

		let mut gutter = rect()
			.width(Size::px(44.0))
			.padding(Gaps::new(8.0, 8.0, 8.0, 4.0))
			.background(Color::from_rgb(26, 26, 26))
			.border(border_right(1.0, Color::from_rgb(45, 45, 48)));

		for i in 1..=line_count {
			gutter = gutter.child(
				rect()
					.main_align(Alignment::End)
					.cross_align(Alignment::Center)
					.child(
						label()
							.text(i.to_string())
							.font_size(12.0)
							.color(Color::from_rgb(110, 110, 110)),
					),
			);
		}

		let mut p = paragraph()
			.holder(holder.read().clone())
			.on_sized(on_paragraph_sized)
			.min_width(Size::func(move |context| Some(context.parent - 24.0)))
			.on_focus_press(on_focus_press)
			.cursor_index(cursor_index)
			.cursor_color(Color::from_rgb(0, 122, 204))
			.highlights(text_selection.map(|h| vec![h]))
			.margin(Gaps::new(8.0, 12.0, 8.0, 12.0));

		if is_empty {
			p = p.span(Span::new(self.placeholder.clone()).color(Color::from_rgb(110, 110, 110)));
		} else {
			let spans = tokenize_full_text(&text, &self.lang);
			for (span_text, span_color) in spans {
				p = p.span(Span::new(span_text).color(span_color));
			}
		}

		rect()
			.a11y_id(a11y_id)
			.a11y_focusable(true)
			.on_key_up(on_key_up)
			.on_key_down(on_key_down)
			.on_ime_preedit(on_ime_preedit)
			.on_focus_press(on_focus_press)
			.on_global_pointer_press(on_global_pointer_press)
			.on_global_pointer_move(on_global_pointer_move)
			.cursor(CursorIcon::Text)
			.width(Size::fill())
			.height(Size::fill())
			.background(Color::from_rgb(30, 30, 30))
			.child(
				ScrollView::new_controlled(scroll_controller)
					.width(Size::fill())
					.height(Size::fill())
					.show_scrollbar(true)
					.on_sized(move |e: Event<SizedEventData>| viewport_area.set_if_modified(e.area))
					.child(
						rect()
							.width(Size::fill())
							.horizontal()
							.child(gutter)
							.child(p),
					),
			)
	}
}
