use std::{collections::BTreeSet, io, path::PathBuf};

use anyhow::{Context, Result};
use crossterm::event::{KeyCode, KeyEvent};
use image::DynamicImage;
use itertools::Itertools;
use ratatui::{
	DefaultTerminal, Frame,
	layout::{Constraint, HorizontalAlignment, Layout, Margin, Rect},
	style::{Color, Style},
	symbols,
	text::{Line, Span},
	widgets::{
		Block, Borders, Clear, List, ListItem, ListState, Paragraph, StatefulWidget, Widget, Wrap,
	},
};
use ratatui_image::{Image, Resize, picker::Picker};
use ratatui_textarea::TextArea;
use sqrilizz::{
	config::Config,
	config_crate::{ConfigDeser, package::PackageConfigDeser},
	instance::tracking::RunningInstanceRegistry,
	io::paths::Paths,
	shared::{
		Side,
		versions::{MinecraftLatestVersion, MinecraftVersionDeser},
	},
};

use crate::{output::fit_message_width, prompt::get_key};

const COLOR_PURPLE: Color = Color::Rgb(168, 85, 247);
const COLOR_CYAN: Color = Color::Rgb(56, 189, 248);
const COLOR_GREEN: Color = Color::Rgb(34, 197, 94);
const COLOR_GRAY: Color = Color::Rgb(156, 163, 175);

pub async fn run_dashboard(paths: &Paths) -> Result<Option<String>> {
	let state = State::load(paths)?;
	let res = ratatui::run(move |terminal| renderer(terminal, state))
		.context("Failed to run dashboard")?;
	Ok(res)
}

#[derive(Clone, Debug)]
struct InstanceData {
	id: String,
	name: String,
	version: String,
	loader: String,
	side: Side,
	packages: Vec<String>,
	resource_packs: Vec<String>,
	shaders: Vec<String>,
	memory: String,
	java: String,
	quick_play: String,
	is_running: bool,
	icon_path: Option<PathBuf>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum FocusState {
	None,
	Search,
	Preview,
	Popup(Popup),
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Popup {
	Side,
	Loader,
	Version,
	Running,
}

impl Popup {
	fn title(&self) -> &'static str {
		match self {
			Self::Side => "Сторона",
			Self::Loader => "Загрузчик",
			Self::Version => "Версия",
			Self::Running => "Статус",
		}
	}

	fn get_items(&self, state: &State) -> Vec<String> {
		match self {
			Self::Side => vec![
				"Все".to_string(),
				"Клиент".to_string(),
				"Сервер".to_string(),
			],
			Self::Loader => {
				let mut items = vec!["Все".to_string()];
				items.extend(state.available_loaders.iter().cloned());
				items
			}
			Self::Version => {
				let mut items = vec!["Все".to_string()];
				items.extend(state.available_versions.iter().cloned());
				items
			}
			Self::Running => vec![
				"Все".to_string(),
				"В сети".to_string(),
				"Остановлен".to_string(),
			],
		}
	}

	fn get_selected(&self, state: &State) -> Vec<usize> {
		match self {
			Self::Side => match state.filter_side {
				None => vec![0],
				Some(Side::Client) => vec![1],
				Some(Side::Server) => vec![2],
			},
			Self::Loader => match &state.filter_loader {
				None => vec![0],
				Some(l) => state
					.available_loaders
					.iter()
					.position(|x| x == l)
					.map(|x| x + 1)
					.into_iter()
					.collect(),
			},
			Self::Version => match &state.filter_version {
				None => vec![0],
				Some(v) => state
					.available_versions
					.iter()
					.position(|x| x == v)
					.map(|x| x + 1)
					.into_iter()
					.collect(),
			},
			Self::Running => match state.filter_running {
				None => vec![0],
				Some(true) => vec![1],
				Some(false) => vec![2],
			},
		}
	}

	fn select(&self, state: &mut State, idx: usize) {
		match self {
			Self::Side => {
				state.filter_side = match idx {
					1 => Some(Side::Client),
					2 => Some(Side::Server),
					_ => None,
				};
			}
			Self::Loader => {
				if idx == 0 {
					state.filter_loader = None;
				} else if let Some(loader) = state.available_loaders.get(idx - 1) {
					state.filter_loader = Some(loader.clone());
				}
			}
			Self::Version => {
				if idx == 0 {
					state.filter_version = None;
				} else if let Some(ver) = state.available_versions.get(idx - 1) {
					state.filter_version = Some(ver.clone());
				}
			}
			Self::Running => {
				state.filter_running = match idx {
					1 => Some(true),
					2 => Some(false),
					_ => None,
				};
			}
		}
		state.update_filtered();
		state.focus = FocusState::None;
	}

	fn render(&self, state: &mut State, frame: &mut Frame, area: Rect) {
		let filter_layout = Layout::horizontal(Constraint::from_fills([1, 1, 1, 1, 1]));
		let filter_layout = area.layout::<5>(&filter_layout);

		let popup_pane = match self {
			Self::Side => filter_layout[0],
			Self::Loader => filter_layout[1],
			Self::Version => filter_layout[2],
			Self::Running => filter_layout[3],
		};

		let mut popup_area = popup_pane;
		popup_area.height = 10.min(frame.area().height.saturating_sub(popup_area.y));

		let block = rounded_block()
			.border_style(Style::new().fg(COLOR_PURPLE))
			.title(self.title());

		let selected = self.get_selected(state);
		let items = self.get_items(state);
		let items = items.into_iter().enumerate().map(|(i, x)| {
			if selected.contains(&i) {
				format!("{x} (✓)")
			} else {
				x
			}
		});

		let list = List::default()
			.block(block)
			.items(items)
			.highlight_symbol("▸ ")
			.highlight_style(Style::new().fg(COLOR_CYAN).bold());

		frame.render_widget(Clear, popup_area);
		frame.render_stateful_widget(list, popup_area, &mut state.popup_list_state);
	}

	fn input(&self, state: &mut State, key: KeyEvent) {
		match key.code {
			KeyCode::Char('t') if *self == Popup::Side => state.focus_none(),
			KeyCode::Char('l') if *self == Popup::Loader => state.focus_none(),
			KeyCode::Char('v') if *self == Popup::Version => state.focus_none(),
			KeyCode::Char('r') if *self == Popup::Running => state.focus_none(),
			KeyCode::Up | KeyCode::Char('k') => state.popup_list_state.select_previous(),
			KeyCode::Down | KeyCode::Char('j') => state.popup_list_state.select_next(),
			KeyCode::Enter => {
				if let Some(idx) = state.popup_list_state.selected() {
					self.select(state, idx);
				}
			}
			_ => {}
		}
	}
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PreviewTab {
	Overview,
	Packages,
	ResourcePacks,
	Launch,
}

impl PreviewTab {
	fn title(&self) -> &'static str {
		match self {
			Self::Overview => "Обзор [o]",
			Self::Packages => "Пакеты [m]",
			Self::ResourcePacks => "Ресурспаки [r]",
			Self::Launch => "Параметры [c]",
		}
	}
}

struct State<'a> {
	instances: Vec<InstanceData>,
	filtered: Vec<InstanceData>,
	list_state: ListState,
	focus: FocusState,
	search: TextArea<'a>,
	popup_list_state: ListState,
	preview_scroll: u16,
	preview_scroll_height: u16,
	preview_tab: PreviewTab,
	filter_side: Option<Side>,
	filter_loader: Option<String>,
	filter_version: Option<String>,
	filter_running: Option<bool>,
	available_loaders: Vec<String>,
	available_versions: Vec<String>,
	selected_launch: Option<String>,
}

impl<'a> State<'a> {
	fn load(paths: &Paths) -> Result<Self> {
		let raw_config =
			Config::open(&Config::get_path(paths)).unwrap_or_else(|_| ConfigDeser::default());
		let mut running_registry = RunningInstanceRegistry::open(paths).ok();

		let mut instances = Vec::new();
		let mut loaders_set = BTreeSet::new();
		let mut versions_set = BTreeSet::new();

		for (id, inst) in raw_config.instances.iter().sorted_by_key(|x| x.0) {
			let ver_str = match &inst.version {
				Some(MinecraftVersionDeser::Version(v)) => v.to_string(),
				Some(MinecraftVersionDeser::Latest(MinecraftLatestVersion::Release)) => {
					"latest-release".to_string()
				}
				Some(MinecraftVersionDeser::Latest(MinecraftLatestVersion::Snapshot)) => {
					"latest-snapshot".to_string()
				}
				None => "default".to_string(),
			};
			let loader_str = inst.loader.as_deref().unwrap_or("vanilla").to_string();
			let side = inst.side.unwrap_or(Side::Client);
			let name = inst.name.clone().unwrap_or_else(|| id.to_string());
			let packages = inst
				.packages
				.iter()
				.map(|p| match p {
					PackageConfigDeser::Basic(s) => s.clone(),
					PackageConfigDeser::Full(f) => f.id.clone(),
				})
				.collect();
			let memory = match &inst.launch.memory {
				sqrilizz::config_crate::instance::LaunchMemory::None => "По умолчанию".to_string(),
				sqrilizz::config_crate::instance::LaunchMemory::Single(s) => s.clone(),
				sqrilizz::config_crate::instance::LaunchMemory::Both { min, max } => {
					format!("{min} - {max}")
				}
			};
			let java = inst
				.launch
				.java
				.clone()
				.unwrap_or_else(|| "Автоопределение".to_string());
			let quick_play = match &inst.launch.quick_play {
				sqrilizz::config_crate::instance::QuickPlay::World { world } => {
					format!("Мир: {world}")
				}
				sqrilizz::config_crate::instance::QuickPlay::Server { server, port } => {
					if let Some(p) = port {
						format!("Сервер: {server}:{p}")
					} else {
						format!("Сервер: {server}")
					}
				}
				sqrilizz::config_crate::instance::QuickPlay::Realm { realm } => {
					format!("Realm: {realm}")
				}
				sqrilizz::config_crate::instance::QuickPlay::None => "Отключено".to_string(),
			};

			let is_running = running_registry
				.as_mut()
				.map(|r| r.get_instance(id, None).is_some())
				.unwrap_or(false);

			let instance_dir = paths.data.join("instances").join(id.as_ref());
			let mut icon_path = None;
			for ext in ["png", "jpg", "jpeg", "webp"] {
				let candidate = instance_dir.join(format!("icon.{ext}"));
				if candidate.exists() {
					icon_path = Some(candidate);
					break;
				}
			}

			loaders_set.insert(loader_str.clone());
			versions_set.insert(ver_str.clone());

			let resource_packs =
				sqrilizz::instance_crate::files::get_instance_resource_packs(&instance_dir)
					.unwrap_or_default()
					.into_iter()
					.map(|rp| {
						if let Some(d) = rp.description {
							format!("{} ({})", rp.name, d)
						} else {
							rp.name
						}
					})
					.collect();
			let shaders = sqrilizz::instance_crate::files::get_instance_shader_packs(&instance_dir)
				.unwrap_or_default()
				.into_iter()
				.map(|s| s.name)
				.collect();

			instances.push(InstanceData {
				id: id.to_string(),
				name,
				version: ver_str,
				loader: loader_str,
				side,
				packages,
				resource_packs,
				shaders,
				memory,
				java,
				quick_play,
				is_running,
				icon_path,
			});
		}

		let mut search = TextArea::default();
		search.set_cursor_line_style(Style::default());
		search.set_placeholder_text("Введите строку поиска...");
		let search_block = rounded_block().title(" Поиск ");
		search.set_block(search_block);

		let mut state = Self {
			filtered: instances.clone(),
			instances,
			list_state: ListState::default(),
			focus: FocusState::None,
			search,
			popup_list_state: ListState::default(),
			preview_scroll: 0,
			preview_scroll_height: 0,
			preview_tab: PreviewTab::Overview,
			filter_side: None,
			filter_loader: None,
			filter_version: None,
			filter_running: None,
			available_loaders: loaders_set.into_iter().collect(),
			available_versions: versions_set.into_iter().collect(),
			selected_launch: None,
		};

		if !state.filtered.is_empty() {
			state.list_state.select(Some(0));
		}

		Ok(state)
	}

	fn update_filtered(&mut self) {
		let q = self
			.search
			.lines()
			.first()
			.cloned()
			.unwrap_or_default()
			.to_lowercase();
		let side_filter = self.filter_side;
		let loader_filter = self.filter_loader.as_deref();
		let version_filter = self.filter_version.as_deref();
		let running_filter = self.filter_running;

		self.filtered = self
			.instances
			.iter()
			.filter(|inst| {
				if let Some(side) = side_filter {
					if inst.side != side {
						return false;
					}
				}
				if let Some(loader) = loader_filter {
					if !inst.loader.eq_ignore_ascii_case(loader) {
						return false;
					}
				}
				if let Some(version) = version_filter {
					if !inst.version.eq_ignore_ascii_case(version) {
						return false;
					}
				}
				if let Some(running) = running_filter {
					if inst.is_running != running {
						return false;
					}
				}
				if q.is_empty() {
					true
				} else {
					inst.id.to_lowercase().contains(&q)
						|| inst.name.to_lowercase().contains(&q)
						|| inst.version.to_lowercase().contains(&q)
						|| inst.loader.to_lowercase().contains(&q)
				}
			})
			.cloned()
			.collect();

		if self.filtered.is_empty() {
			self.list_state.select(None);
		} else {
			let cur = self.list_state.selected().unwrap_or(0);
			if cur >= self.filtered.len() {
				self.list_state.select(Some(self.filtered.len() - 1));
			} else {
				self.list_state.select(Some(cur));
			}
		}
		self.preview_scroll = 0;
	}

	fn current_instance(&self) -> Option<&InstanceData> {
		self.list_state
			.selected()
			.and_then(|i| self.filtered.get(i))
	}

	fn focus_none(&mut self) {
		self.focus = FocusState::None;
	}

	fn focus_preview(&mut self) {
		self.focus = FocusState::Preview;
	}

	fn focus_popup(&mut self, popup: Popup) {
		let selected = popup.get_selected(self);
		self.popup_list_state.select(selected.first().cloned());
		self.focus = FocusState::Popup(popup);
	}

	fn reset_filters(&mut self) {
		self.filter_side = None;
		self.filter_loader = None;
		self.filter_version = None;
		self.filter_running = None;
		let mut search = TextArea::default();
		search.set_cursor_line_style(Style::default());
		search.set_placeholder_text("Введите строку поиска...");
		let search_block = rounded_block().title(" Поиск ");
		search.set_block(search_block);
		self.search = search;
		self.update_filtered();
		if !self.filtered.is_empty() {
			self.list_state.select(Some(0));
		}
	}
}

fn rounded_block<'a>() -> Block<'a> {
	Block::bordered().border_set(symbols::border::ROUNDED)
}

fn renderer(terminal: &mut DefaultTerminal, mut state: State<'_>) -> io::Result<Option<String>> {
	terminal.draw(|frame| render(frame, &mut state))?;

	loop {
		let mut should_render = false;

		let key = get_key().map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
		if let Some(key) = key {
			should_render = true;

			match state.focus {
				_ if key.code == KeyCode::Esc && state.focus != FocusState::None => {
					state.focus_none();
				}
				FocusState::None => match key.code {
					KeyCode::Char('q') | KeyCode::Esc => break,
					KeyCode::Char('s') | KeyCode::Char('/') => {
						state.focus = FocusState::Search;
					}
					KeyCode::Char('x') => state.reset_filters(),
					KeyCode::Char('t') => state.focus_popup(Popup::Side),
					KeyCode::Char('l') => state.focus_popup(Popup::Loader),
					KeyCode::Char('v') => state.focus_popup(Popup::Version),
					KeyCode::Char('r') => state.focus_popup(Popup::Running),
					KeyCode::Up | KeyCode::Char('k') => {
						state.list_state.select_previous();
						state.preview_scroll = 0;
					}
					KeyCode::Down | KeyCode::Char('j') => {
						state.list_state.select_next();
						state.preview_scroll = 0;
					}
					KeyCode::Enter => {
						if let Some(inst) = state.current_instance() {
							state.selected_launch = Some(inst.id.clone());
							break;
						}
					}
					KeyCode::Tab | KeyCode::Right | KeyCode::Char('p') => {
						state.focus_preview();
					}
					_ => {}
				},
				FocusState::Search => match key.code {
					KeyCode::Enter => {
						state.update_filtered();
						state.focus_none();
					}
					KeyCode::Esc => {
						state.focus_none();
					}
					_ => {
						state.search.input(key);
						state.update_filtered();
					}
				},
				FocusState::Popup(popup) => popup.input(&mut state, key),
				FocusState::Preview => match key.code {
					KeyCode::Char('q') => break,
					KeyCode::Esc | KeyCode::Tab | KeyCode::Char('p') => {
						state.focus_none();
					}
					KeyCode::Char('o') => {
						state.preview_tab = PreviewTab::Overview;
						state.preview_scroll = 0;
					}
					KeyCode::Char('m') => {
						state.preview_tab = PreviewTab::Packages;
						state.preview_scroll = 0;
					}
					KeyCode::Char('r') => {
						state.preview_tab = PreviewTab::ResourcePacks;
						state.preview_scroll = 0;
					}
					KeyCode::Char('c') => {
						state.preview_tab = PreviewTab::Launch;
						state.preview_scroll = 0;
					}
					KeyCode::Up | KeyCode::Char('k') if state.preview_scroll > 0 => {
						state.preview_scroll = state.preview_scroll.saturating_sub(1);
					}
					KeyCode::Down | KeyCode::Char('j')
						if state.preview_scroll < state.preview_scroll_height =>
					{
						state.preview_scroll = state.preview_scroll.saturating_add(1);
					}
					KeyCode::Enter => {
						if let Some(inst) = state.current_instance() {
							state.selected_launch = Some(inst.id.clone());
							break;
						}
					}
					_ => {}
				},
			}
		}

		if should_render {
			terminal.draw(|frame| render(frame, &mut state))?;
		}
	}

	Ok(state.selected_launch)
}

fn render(frame: &mut Frame, state: &mut State) {
	let layout = Layout::vertical([
		Constraint::Length(1), // Top Brand Header
		Constraint::Length(1), // Filters bar
		Constraint::Fill(1),   // Main content
		Constraint::Length(3), // Search box
		Constraint::Length(1), // Status & keybinds bar
	]);
	let layout = frame.area().layout::<5>(&layout);
	let brand_pane = layout[0];
	let filters_pane = layout[1];
	let preview_pane = layout[2];
	let search_pane = layout[3];
	let status_pane = layout[4];

	// Top Brand Header
	let brand_layout = Layout::horizontal([Constraint::Fill(2), Constraint::Fill(1)]);
	let brand_layout = brand_pane.layout::<2>(&brand_layout);
	let title_spans = Line::from(vec![
		Span::styled("◆ ", Style::new().fg(COLOR_PURPLE).bold()),
		Span::styled("Sqrilizz Launcher ", Style::new().fg(COLOR_PURPLE).bold()),
		Span::styled("│ ", Style::new().fg(COLOR_GRAY)),
		Span::styled("TUI Панель управления", Style::new().fg(COLOR_CYAN).bold()),
	]);
	let brand_left = Paragraph::new(title_spans);
	frame.render_widget(brand_left, brand_layout[0]);

	let brand_right = Paragraph::new("sqrilizz.tech")
		.alignment(HorizontalAlignment::Right)
		.style(Style::new().fg(COLOR_GRAY));
	frame.render_widget(brand_right, brand_layout[1]);

	// Keybinds & Status bar
	let status_layout = Layout::horizontal([Constraint::Fill(3), Constraint::Fill(1)]);
	let status_layout = status_pane.layout::<2>(&status_layout);
	let keybinds_pane = status_layout[0];
	let state_pane = status_layout[1];

	let keybinds_text = match state.focus {
		FocusState::None => "q: выход │ s: поиск │ k/j: выбор │ Enter: запуск │ Tab/p: детали",
		FocusState::Search => "Esc: закрыть поиск │ Enter: применить",
		FocusState::Popup(..) => "Esc: закрыть окно │ k/j: выбор │ Enter: применить",
		FocusState::Preview => "Esc: назад │ k/j: прокрутка │ Enter: запуск │ o/m/r/c: вкладки",
	};

	let keybinds = Paragraph::new(keybinds_text).style(Style::new().fg(COLOR_GRAY));
	frame.render_widget(keybinds, keybinds_pane);

	let status_text = format!(
		"Экземпляры: {}/{}",
		state.filtered.len(),
		state.instances.len()
	);
	let status_widget = Paragraph::new(status_text)
		.alignment(HorizontalAlignment::Right)
		.style(Style::new().fg(COLOR_PURPLE).bold());
	frame.render_widget(status_widget, state_pane);

	// Search Box
	let search_block = state.search.block().unwrap();
	let search_block_style = if state.focus == FocusState::Search {
		Style::new().fg(COLOR_CYAN).bold()
	} else {
		Style::new().fg(COLOR_GRAY)
	};
	let search_block = search_block.clone().border_style(search_block_style);
	state.search.set_block(search_block);
	frame.render_widget(&state.search, search_pane);

	// Main content pane: Instance list + Preview
	let preview_layout = Layout::horizontal([Constraint::Fill(1), Constraint::Fill(3)]);
	let preview_layout = preview_pane.layout::<2>(&preview_layout);
	let list_pane = preview_layout[0];
	let preview_pane = preview_layout[1];

	let package_items = state.filtered.iter().map(|inst| {
		let prefix = if inst.is_running { "● " } else { "○ " };
		let name_display = if inst.name != inst.id {
			format!("{prefix}{} ({})", inst.id, inst.name)
		} else {
			format!("{prefix}{}", inst.id)
		};
		if inst.is_running {
			ListItem::new(Span::styled(
				name_display,
				Style::new().fg(COLOR_GREEN).bold(),
			))
		} else {
			ListItem::new(Span::raw(name_display))
		}
	});
	let package_list = List::default()
		.highlight_style(Style::new().fg(COLOR_CYAN).bold())
		.highlight_symbol("▸ ")
		.items(package_items);

	let mut block = rounded_block().title(" Экземпляры ");
	if state.focus == FocusState::None {
		block = block.border_style(Style::new().fg(COLOR_PURPLE));
	} else {
		block = block.border_style(Style::new().fg(COLOR_GRAY));
	}
	let inner = block.inner(list_pane);
	frame.render_widget(block, list_pane);
	frame.render_stateful_widget(package_list, inner, &mut state.list_state);

	let mut block = rounded_block().title(" Информация ");
	if state.focus == FocusState::Preview {
		block = block.border_style(Style::new().fg(COLOR_PURPLE));
	} else {
		block = block.border_style(Style::new().fg(COLOR_GRAY));
	}
	let inner_area = block.inner(preview_pane);
	frame.render_widget(block, preview_pane);

	if let Some(inst) = state.current_instance().cloned() {
		let mut scroll_height = 0;
		let widget = InstancePreviewWidget {
			inst: &inst,
			state,
			scroll_height: &mut scroll_height,
		};
		frame.render_widget(widget, inner_area);
		state.preview_scroll_height = scroll_height;
	} else {
		frame.render_widget(Clear, inner_area);
	}

	// Filters bar
	let filter_layout = Layout::horizontal(Constraint::from_fills([1, 1, 1, 1, 1]));
	let filter_layout = filters_pane.layout::<5>(&filter_layout);
	let side_pane = filter_layout[0];
	let loader_pane = filter_layout[1];
	let version_pane = filter_layout[2];
	let running_pane = filter_layout[3];
	let reset_pane = filter_layout[4];

	let side_str = match state.filter_side {
		None => "Все",
		Some(Side::Client) => "Клиент",
		Some(Side::Server) => "Сервер",
	};
	let side_widget = Paragraph::new(format!("[t] Сторона: {side_str}"))
		.style(Style::new().bold().fg(COLOR_CYAN));
	frame.render_widget(side_widget, side_pane);

	let loader_str = state.filter_loader.as_deref().unwrap_or("Все");
	let loader_widget = Paragraph::new(format!("[l] Загрузчик: {loader_str}"))
		.style(Style::new().bold().fg(COLOR_CYAN));
	frame.render_widget(loader_widget, loader_pane);

	let version_str = state.filter_version.as_deref().unwrap_or("Все");
	let version_widget = Paragraph::new(format!("[v] Версия: {version_str}"))
		.style(Style::new().bold().fg(COLOR_CYAN));
	frame.render_widget(version_widget, version_pane);

	let running_str = match state.filter_running {
		None => "Все",
		Some(true) => "В сети",
		Some(false) => "Остановлен",
	};
	let running_widget = Paragraph::new(format!("[r] Статус: {running_str}"))
		.style(Style::new().bold().fg(COLOR_CYAN));
	frame.render_widget(running_widget, running_pane);

	let reset_widget = Paragraph::new("[x] Сбросить")
		.alignment(HorizontalAlignment::Right)
		.style(Style::new().fg(COLOR_GRAY));
	frame.render_widget(reset_widget, reset_pane);

	if let FocusState::Popup(popup) = state.focus {
		let mut base_area = frame.area().inner(Margin::new(1, 2));
		base_area.height = 10;
		popup.render(state, frame, base_area);
	}
}

struct InstancePreviewWidget<'a> {
	inst: &'a InstanceData,
	state: &'a State<'a>,
	scroll_height: &'a mut u16,
}

impl<'a> Widget for InstancePreviewWidget<'a> {
	fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer) {
		let layout = Layout::vertical([Constraint::Length(4), Constraint::Fill(1)]);
		let layout = area.layout::<2>(&layout);
		let top_pane = layout[0];
		let bottom_pane = layout[1];

		let block = Block::new()
			.borders(Borders::BOTTOM)
			.border_style(Style::new().fg(COLOR_GRAY));
		let inner_area = block.inner(top_pane);
		block.render(top_pane, buf);

		let has_icon = self.inst.icon_path.is_some();
		let (icon_pane, details_pane) = if has_icon {
			let layout = Layout::horizontal([Constraint::Length(6), Constraint::Fill(1)]);
			let layout = inner_area.layout::<2>(&layout);
			(Some(layout[0]), layout[1].inner(Margin::new(1, 1)))
		} else {
			(None, inner_area.inner(Margin::new(1, 1)))
		};

		if let Some(icon_pane) = icon_pane {
			if let Some(path) = &self.inst.icon_path {
				if let Ok(img) = image::open(path) {
					let picker = Picker::from_query_stdio().unwrap_or(Picker::halfblocks());
					if let Ok(protocol) = picker.new_protocol(
						DynamicImage::ImageRgb8(img.to_rgb8()),
						icon_pane,
						Resize::Scale(None),
					) {
						let img_widget = Image::new(&protocol);
						img_widget.render(icon_pane, buf);
					}
				}
			}
		}

		let layout = Layout::horizontal(Constraint::from_fills([1, 1, 1]));
		let [title_pane, req_pane, not_loaded_pane] = details_pane.layout::<3>(&layout);

		let (title, title_style) = if self.inst.is_running {
			(
				format!("● {} [В сети]", self.inst.name),
				Style::new().fg(COLOR_GREEN).bold(),
			)
		} else {
			(
				format!("○ {}", self.inst.name),
				Style::new().fg(COLOR_PURPLE).bold(),
			)
		};
		let title_widget = Paragraph::new(title).style(title_style);
		title_widget.render(title_pane, buf);

		let req_text = format!("ID: {} │ Сторона: {:?}", self.inst.id, self.inst.side);
		let req_widget = Paragraph::new(req_text).style(Style::new().fg(COLOR_GRAY));
		req_widget.render(req_pane, buf);

		let launch_indicator = Paragraph::new("[Enter] Запустить игру")
			.alignment(HorizontalAlignment::Right)
			.style(Style::new().fg(COLOR_PURPLE).bold());
		launch_indicator.render(not_loaded_pane, buf);

		let mut subtitle_area = details_pane;
		subtitle_area.y += 1;
		let subtitle_text = format!(
			"Minecraft {} │ Загрузчик: {} │ Установлено пакетов: {}",
			self.inst.version,
			self.inst.loader,
			self.inst.packages.len()
		);
		let subtitle_widget = Paragraph::new(subtitle_text).style(Style::new().fg(COLOR_GRAY));
		subtitle_widget.render(subtitle_area, buf);

		let layout = Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]);
		let layout = bottom_pane.layout::<2>(&layout);
		let tabs_pane = layout[0];
		let body_pane = layout[1].inner(Margin::new(1, 1));

		let layout = Layout::horizontal(Constraint::from_fills([1, 1, 1, 1]));
		let layout = tabs_pane.layout::<4>(&layout);

		for (i, id) in [
			PreviewTab::Overview,
			PreviewTab::Packages,
			PreviewTab::ResourcePacks,
			PreviewTab::Launch,
		]
		.into_iter()
		.enumerate()
		{
			let mut tab = Paragraph::new(id.title()).alignment(HorizontalAlignment::Center);
			if self.state.preview_tab == id {
				tab = tab.style(Style::new().fg(COLOR_PURPLE).bold().reversed());
			} else {
				tab = tab.style(Style::new().fg(COLOR_GRAY));
			}
			tab.render(layout[i], buf);
		}

		match self.state.preview_tab {
			PreviewTab::Overview => {
				let status_desc = if self.inst.is_running {
					"В сети (Активен)"
				} else {
					"Остановлен"
				};
				let markdown_text = format!(
					"# Экземпляр: {}\n\n\
					- **Идентификатор**: `{}`\n\
					- **Состояние**: {}\n\
					- **Версия Minecraft**: {}\n\
					- **Загрузчик модов**: {}\n\
					- **Сторона среды**: `{:?}`\n\
					- **Выделенная память JVM**: {}\n\
					- **Исполняемый Java**: {}\n\
					- **Быстрый запуск**: {}\n\
					- **Всего пакетов/модов**: {}\n\n\
					### Горячие клавиши\n\
					Нажмите **[Enter]** для запуска данного экземпляра.\n\
					Нажмите **[m]** для просмотра установленных пакетов.\n\
					Нажмите **[r]** для ресурспаков и шейдеров.\n\
					Нажмите **[c]** для параметров запуска и консольных команд.",
					self.inst.name,
					self.inst.id,
					status_desc,
					self.inst.version,
					self.inst.loader,
					self.inst.side,
					self.inst.memory,
					self.inst.java,
					self.inst.quick_play,
					self.inst.packages.len()
				);

				let text = tui_markdown::from_str(&markdown_text);
				let visible_lines = body_pane.height;
				let scroll_height = (text.lines.len() as u16).saturating_sub(visible_lines);
				*self.scroll_height = scroll_height;

				let markdown = Paragraph::new(text)
					.scroll((self.state.preview_scroll, 0))
					.wrap(Wrap { trim: false });
				markdown.render(body_pane, buf);
			}
			PreviewTab::Packages => {
				Clear.render(body_pane, buf);
				*self.scroll_height =
					render_packages_list(&self.inst.packages, self.state, body_pane, buf);
			}
			PreviewTab::ResourcePacks => {
				Clear.render(body_pane, buf);
				*self.scroll_height = render_resource_packs_list(
					&self.inst.resource_packs,
					&self.inst.shaders,
					self.state,
					body_pane,
					buf,
				);
			}
			PreviewTab::Launch => {
				Clear.render(body_pane, buf);
				*self.scroll_height = render_launch_info(self.inst, self.state, body_pane, buf);
			}
		}
	}
}

fn render_packages_list(
	packages: &[String],
	state: &State,
	area: Rect,
	buf: &mut ratatui::prelude::Buffer,
) -> u16 {
	if packages.is_empty() {
		let empty_msg = Paragraph::new("В данном экземпляре нет установленных пакетов.\nИспользуйте 'sqrilizz pkg browse' для установки модов и пакетов.")
			.style(Style::new().fg(COLOR_GRAY));
		empty_msg.render(area, buf);
		return 0;
	}

	let list = List::default()
		.highlight_symbol("▸ ")
		.highlight_style(Style::new().fg(COLOR_CYAN).bold());

	let items = packages.iter().map(|p| {
		let layout = Layout::horizontal([
			Constraint::Length(9),
			Constraint::Fill(2),
			Constraint::Fill(1),
		]);
		let [_tag_pane, name_pane, _] = area.layout(&layout);

		let tag = Span::styled("[мод]    ", Style::new().fg(COLOR_PURPLE).bold());
		let pkg_name = fit_message_width(p, name_pane.width as usize);
		let pkg_name = Span::styled(pkg_name, Style::new().bold());

		let line = Line::from(vec![tag, pkg_name]);
		ListItem::new(line)
	});
	let list = list.items(items);
	let len = packages.len() as u16;

	let mut list_state = ListState::default();
	list_state.select(Some(state.preview_scroll as usize));

	StatefulWidget::render(list, area, buf, &mut list_state);

	len.saturating_sub(area.height)
}

fn render_launch_info(
	inst: &InstanceData,
	state: &State,
	area: Rect,
	buf: &mut ratatui::prelude::Buffer,
) -> u16 {
	let markdown_text = format!(
		"### Параметры JVM и запуска\n\n\
		- **Память (RAM)**: `{}`\n\
		- **Исполняемый Java**: `{}`\n\
		- **Быстрый запуск**: `{}`\n\n\
		### Быстрые команды CLI\n\
		```bash\n\
		sqrilizz launch {}\n\
		sqrilizz package browse --instance {}\n\
		sqrilizz log {}\n\
		sqrilizz instance dir {}\n\
		```",
		inst.memory, inst.java, inst.quick_play, inst.id, inst.id, inst.id, inst.id
	);

	let text = tui_markdown::from_str(&markdown_text);
	let visible_lines = area.height;
	let scroll_height = (text.lines.len() as u16).saturating_sub(visible_lines);

	let markdown = Paragraph::new(text)
		.scroll((state.preview_scroll, 0))
		.wrap(Wrap { trim: false });
	markdown.render(area, buf);

	scroll_height
}

fn render_resource_packs_list(
	rps: &[String],
	shaders: &[String],
	state: &State,
	area: Rect,
	buf: &mut ratatui::prelude::Buffer,
) -> u16 {
	if rps.is_empty() && shaders.is_empty() {
		let empty_msg = Paragraph::new(
			"Ресурспаки и шейдеры не найдены.\nПоместите архивы .zip в resourcepacks или shaderpacks.",
		)
		.style(Style::new().fg(COLOR_GRAY));
		empty_msg.render(area, buf);
		return 0;
	}

	let mut all_items = Vec::new();
	for s in shaders {
		all_items.push((
			Span::styled("[шейдер] ", Style::new().fg(COLOR_PURPLE).bold()),
			s,
		));
	}
	for rp in rps {
		all_items.push((
			Span::styled("[пак]    ", Style::new().fg(COLOR_CYAN).bold()),
			rp,
		));
	}

	let items = all_items.into_iter().map(|(tag, name)| {
		let layout = Layout::horizontal([
			Constraint::Length(9),
			Constraint::Fill(2),
			Constraint::Fill(1),
		]);
		let [_tag_pane, name_pane, _] = area.layout(&layout);
		let pack_name = fit_message_width(name, name_pane.width as usize);
		let pack_name = Span::styled(pack_name, Style::new().bold());
		let line = Line::from(vec![tag, pack_name]);
		ListItem::new(line)
	});

	let list = List::default()
		.highlight_symbol("▸ ")
		.highlight_style(Style::new().fg(COLOR_CYAN).bold())
		.items(items);

	let len = (rps.len() + shaders.len()) as u16;
	let mut list_state = ListState::default();
	list_state.select(Some(state.preview_scroll as usize));

	StatefulWidget::render(list, area, buf, &mut list_state);

	len.saturating_sub(area.height)
}
