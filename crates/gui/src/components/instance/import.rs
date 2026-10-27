use std::path::{Path, PathBuf};

use freya::components::ScrollView;
use sqrilizz::{config_crate::instance::make_valid_instance_id, shared::io::home_dir};

use crate::{
	components::{
		button,
		dialog::modal::{MODAL_MEDIUM_HEIGHT, MODAL_MEDIUM_WIDTH, Modal},
		icon_text_button,
		input::text::{TextInput, search_bar},
	},
	ops::instance::{FetchItems, ImportExternalInstance, ImportExternalParams},
	prelude::*,
	state::use_front_state,
	util::assets::get_instance_icon,
};

#[derive(Clone, PartialEq, Debug)]
pub struct DiscoveredInstance {
	pub id: String,
	pub name: String,
	pub launcher_name: String,
	pub mc_version: String,
	pub loader: Option<String>,
	pub loader_display: String,
	pub game_dir: PathBuf,
	pub icon_path: Option<String>,
	pub is_already_imported: bool,
}

#[derive(PartialEq)]
pub struct ImportLauncherModal;

impl Component for ImportLauncherModal {
	fn render(&self) -> impl IntoElement {
		let theme = use_theme();
		let front_state = use_front_state();
		let back_state = use_consume::<BackState>();
		let import_mutation = use_mutation(Mutation::new(ImportExternalInstance::new(
			back_state.clone(),
		)));

		let items_query = use_query(FetchItems::new(back_state.clone()));
		let existing_instances = match &*items_query.read().state() {
			QueryStateData::Settled { res: Ok(res), .. } => res.instances.clone(),
			_ => Vec::new(),
		};

		let search = use_state(String::new);
		let manual_instances = use_state::<Vec<DiscoveredInstance>>(Vec::new);
		let instances_state = use_state(Vec::<DiscoveredInstance>::new);
		let is_scanning = use_state(|| true);
		let refresh_counter = use_state(|| 0);

		let instances_hook = instances_state;
		let is_scanning_hook = is_scanning;
		let back_state_clone = back_state.clone();
		let existing_instances_clone = existing_instances.clone();
		let refresh_val = *refresh_counter.read();

		use_future(move || {
			let instances_hook = instances_hook;
			let is_scanning_hook = is_scanning_hook;
			let back_state = back_state_clone.clone();
			let existing = existing_instances_clone.clone();
			let _ = refresh_val;
			async move {
				let mut is_scanning_hook = is_scanning_hook;
				let mut instances_hook = instances_hook;
				is_scanning_hook.set(true);
				let existing_ids: Vec<String> = existing.iter().map(|x| x.id.clone()).collect();
				let existing_dirs: Vec<String> =
					existing.iter().filter_map(|x| x.dir.clone()).collect();
				let data_path = back_state.paths.data.clone();

				let scanned = tokio::task::spawn_blocking(move || {
					scan_known_launchers(&existing_ids, &existing_dirs, &data_path)
				})
				.await
				.unwrap_or_default();

				instances_hook.set(scanned);
				is_scanning_hook.set(false);
			}
		});

		let mut combined_instances = instances_state.read().clone();
		for manual in manual_instances.read().iter() {
			if !combined_instances
				.iter()
				.any(|x| x.game_dir == manual.game_dir)
			{
				combined_instances.insert(0, manual.clone());
			}
		}

		let search_text = search.read().to_lowercase();
		let filtered_instances: Vec<DiscoveredInstance> = combined_instances
			.into_iter()
			.filter(|inst| {
				if search_text.is_empty() {
					return true;
				}
				inst.name.to_lowercase().contains(&search_text)
					|| inst.launcher_name.to_lowercase().contains(&search_text)
					|| inst.mc_version.to_lowercase().contains(&search_text)
					|| inst.loader_display.to_lowercase().contains(&search_text)
			})
			.collect();

		let total_found = filtered_instances.len();

		let front_state_close = front_state.clone();
		let close_fn = move |_| {
			front_state_close.write().set_modal(None);
		};

		let manual_instances_hook = manual_instances;
		let front_state_manual = front_state.clone();
		let pick_folder_button = icon_text_button("folder", "Выбрать папку...", &theme)
			.border_fill(theme.panel_border)
			.on_press(move |_| {
				let mut manual_instances_clone = manual_instances_hook;
				let front_state_clone = front_state_manual.clone();
				spawn(async move {
					let picked = tokio::task::spawn_blocking(|| {
						rfd::FileDialog::new()
							.set_title("Выберите папку со сборкой Minecraft")
							.pick_folder()
					})
					.await
					.ok()
					.flatten();

					if let Some(folder) = picked {
						if let Some(discovered) = inspect_custom_folder(&folder) {
							manual_instances_clone.write().insert(0, discovered);
							front_state_clone
								.write()
								.toast(Toast::success("Сборка обнаружена и добавлена в список!"));
						} else {
							front_state_clone.write().toast(Toast::warning(
								"В этой папке не найдены файлы Minecraft/модов",
								None,
							));
						}
					}
				});
			});

		let mut refresh_counter_btn = refresh_counter;
		let refresh_btn = icon_button("cycle", &theme).on_press(move |_| {
			refresh_counter_btn.set(*refresh_counter_btn.read() + 1);
		});
		let refresh_btn = rect()
			.tip(&front_state, "Обновить список лаунчеров")
			.child(refresh_btn);

		let search_input = search_bar(
			TextInput::new(search).placeholder("Поиск по названию или лаунчеру..."),
			&theme,
		);

		let top_bar = rect()
			.width(Size::fill())
			.cont()
			.cross_align(Alignment::Center)
			.child(rect().width(Size::flex(1.0)).child(search_input))
			.child(pick_folder_button)
			.child(refresh_btn);

		let count_badge = rect()
			.horizontal()
			.cross_align(Alignment::Center)
			.spacing(6.0)
			.padding(Gaps::new(4.0, 8.0, 4.0, 8.0))
			.background(theme.item)
			.corner_radius(theme.round)
			.child(icon("honeycomb", 12.0).color(theme.fg3))
			.child(
				label()
					.text(if *is_scanning.read() {
						"Поиск сборок в системе...".to_string()
					} else {
						format!("Найдено сборок: {total_found}")
					})
					.font_size(12.0)
					.color(theme.fg3),
			);

		let list_elem = if *is_scanning.read() && filtered_instances.is_empty() {
			rect()
				.expanded()
				.center()
				.spacing(theme.gap2)
				.child(CircularLoader::new().size(32.0))
				.child(
					label()
						.text("Сканирование установленных лаунчеров...")
						.font_size(theme.font0)
						.color(theme.fg2),
				)
				.into_element()
		} else if filtered_instances.is_empty() {
			rect()
				.expanded()
				.center()
				.spacing(theme.gap2)
				.child(icon("download", 36.0).color(theme.fg3))
				.child(
					label()
						.text(
							"Сборки не найдены. Вы можете нажать «Выбрать папку...», чтобы указать путь вручную.",
						)
						.font_size(theme.font0)
						.color(theme.fg3),
				)
				.into_element()
		} else {
			let front_state_sync = front_state.clone();
			let import_mut_sync = import_mutation.clone();
			let on_sync = EventHandler::new(move |inst: DiscoveredInstance| {
				let toast_msg = format!("Сборка «{}» успешно синхронизирована!", inst.name);
				import_mut_sync.mutate(ImportExternalParams {
					name: inst.name,
					id: inst.id,
					version: inst.mc_version,
					loader: inst.loader,
					game_dir: inst.game_dir,
					is_sync: true,
					icon: inst.icon_path,
				});
				front_state_sync.write().toast(Toast::success(&toast_msg));
				front_state_sync.write().set_modal(None);
			});

			let front_state_copy = front_state.clone();
			let import_mut_copy = import_mutation;
			let on_copy = EventHandler::new(move |inst: DiscoveredInstance| {
				let toast_msg = format!("Сборка «{}» скопирована в автономный инстанс!", inst.name);
				import_mut_copy.mutate(ImportExternalParams {
					name: inst.name,
					id: inst.id,
					version: inst.mc_version,
					loader: inst.loader,
					game_dir: inst.game_dir,
					is_sync: false,
					icon: inst.icon_path,
				});
				front_state_copy.write().toast(Toast::success(&toast_msg));
				front_state_copy.write().set_modal(None);
			});

			let cards: Vec<Element> = filtered_instances
				.into_iter()
				.map(|inst| {
					InstanceCard {
						inst,
						on_sync: on_sync.clone(),
						on_copy: on_copy.clone(),
					}
					.into_element()
				})
				.collect();

			ScrollView::new()
				.width(Size::fill())
				.height(Size::flex(1.0))
				.child(
					rect()
						.width(Size::fill())
						.spacing(theme.gap)
						.children(cards),
				)
				.into_element()
		};

		let content = rect()
			.expanded()
			.padding(theme.gap2)
			.spacing(theme.gap2)
			.child(top_bar)
			.child(count_badge)
			.child(list_elem);

		Modal::new("Импорт и синхронизация сборок".into(), "download".into())
			.size(MODAL_MEDIUM_WIDTH, MODAL_MEDIUM_HEIGHT)
			.on_close(close_fn)
			.child(content)
	}
}

#[derive(PartialEq)]
struct InstanceCard {
	inst: DiscoveredInstance,
	on_sync: EventHandler<DiscoveredInstance>,
	on_copy: EventHandler<DiscoveredInstance>,
}

impl Component for InstanceCard {
	fn render(&self) -> impl IntoElement {
		let theme = use_theme();
		let front_state = use_front_state();
		let is_hovered = use_state(|| false);
		let inst = &self.inst;
		let name = inst.name.clone();
		let launcher_name = inst.launcher_name.clone();
		let version_details = format!("{} • {}", inst.mc_version, inst.loader_display);
		let path_str = inst.game_dir.to_string_lossy().to_string();

		let mut action_buttons = Vec::new();

		if inst.is_already_imported {
			let already_synced_badge = rect()
				.horizontal()
				.cross_align(Alignment::Center)
				.spacing(6.0)
				.padding(Gaps::new(5.0, 10.0, 5.0, 10.0))
				.background(theme.primary_bg)
				.border(theme.border(theme.primary))
				.corner_radius(theme.round)
				.child(icon("check", 14.0).color(theme.primary))
				.child(
					label()
						.text("Синхронизировано")
						.font_size(12.0)
						.font_weight(FontWeight::BOLD)
						.color(theme.primary),
				);
			action_buttons.push(already_synced_badge.into_element());
		} else {
			let inst_sync = inst.clone();
			let on_sync_btn = self.on_sync.clone();
			let sync_button = button(&theme)
				.background(theme.primary)
				.color(theme.fg)
				.hover_background(theme.primary)
				.padding(Gaps::new(6.0, 12.0, 6.0, 12.0))
				.corner_radius(theme.round)
				.child(
					rect()
						.horizontal()
						.cross_align(Alignment::Center)
						.spacing(6.0)
						.child(icon("link", 14.0).color(theme.fg))
						.child(
							label()
								.text("Синхронизировать")
								.font_size(12.0)
								.font_weight(FontWeight::BOLD)
								.color(theme.fg),
						),
				)
				.on_press(move |_| {
					on_sync_btn.call(inst_sync.clone());
				});

			let sync_btn = rect()
				.tip(
					&front_state,
					"Мгновенная синхронизация без дублирования файлов: изменения сразу видны в обоих лаунчерах",
				)
				.child(sync_button);

			action_buttons.push(sync_btn.into_element());

			let inst_copy = inst.clone();
			let on_copy_btn = self.on_copy.clone();
			let copy_button = button(&theme)
				.background(theme.item)
				.border_fill(theme.panel_border)
				.hover_background(theme.item_hover)
				.padding(Gaps::new(6.0, 12.0, 6.0, 12.0))
				.corner_radius(theme.round)
				.child(
					rect()
						.horizontal()
						.cross_align(Alignment::Center)
						.spacing(6.0)
						.child(icon("copy", 14.0).color(theme.fg))
						.child(label().text("Копировать").font_size(12.0).color(theme.fg)),
				)
				.on_press(move |_| {
					on_copy_btn.call(inst_copy.clone());
				});

			let copy_btn = rect()
				.tip(
					&front_state,
					"Скопировать файлы в отдельную папку Sqrilizz Launcher, создав независимую сборку",
				)
				.child(copy_button);

			action_buttons.push(copy_btn.into_element());
		}

		let border_color = if *is_hovered.read() {
			theme.item_select_border
		} else {
			theme.panel_border
		};

		let custom_icon = rect()
			.width(Size::px(36.0))
			.height(Size::px(36.0))
			.corner_radius(theme.round)
			.center()
			.child(
				ImageViewer::new(get_instance_icon(inst.icon_path.as_deref()))
					.width(Size::px(36.0))
					.height(Size::px(36.0))
					.corner_radius(theme.round),
			);

		rect()
			.width(Size::fill())
			.padding(Gaps::new(theme.gap, theme.gap2, theme.gap, theme.gap2))
			.background(theme.item)
			.border(theme.border(border_color))
			.corner_radius(theme.round)
			.hover(is_hovered)
			.cont()
			.cross_align(Alignment::Center)
			.child(custom_icon)
			.child(
				rect()
					.width(Size::flex(1.0))
					.vertical()
					.spacing(3.0)
					.child(
						rect()
							.horizontal()
							.cross_align(Alignment::Center)
							.spacing(theme.gap)
							.child(
								label()
									.text(name)
									.font_size(14.0)
									.font_weight(FontWeight::BOLD)
									.color(theme.fg),
							)
							.child(
								rect()
									.padding(Gaps::new(2.0, 6.0, 2.0, 6.0))
									.background(theme.panel)
									.border(theme.border(theme.panel_border))
									.corner_radius(theme.round)
									.child(
										label().text(launcher_name).font_size(10.0).color(theme.fg2),
									),
							),
					)
					.child(
						label()
							.text(version_details)
							.font_size(12.0)
							.color(theme.fg2),
					)
					.child(
						label()
							.text(path_str)
							.font_size(10.0)
							.color(theme.fg3)
							.max_lines(1),
					),
			)
			.child(
				rect()
					.horizontal()
					.cross_align(Alignment::Center)
					.spacing(theme.gap)
					.children(action_buttons),
			)
			.into_element()
	}
}

pub fn scan_known_launchers(
	existing_ids: &[String],
	existing_dirs: &[String],
	data_path: &Path,
) -> Vec<DiscoveredInstance> {
	let mut out = Vec::new();
	let home = home_dir().unwrap_or_else(|_| PathBuf::from("/"));

	// 1. Prism Launcher
	let prism_candidates = [
		home.join(".local/share/PrismLauncher/instances"),
		home.join(".var/app/org.prismlauncher.PrismLauncher/data/PrismLauncher/instances"),
		data_path.join("../PrismLauncher/instances"),
	];

	for base in prism_candidates {
		if base.exists() && base.is_dir() {
			scan_mmc_instances(
				&base,
				"Prism Launcher",
				existing_ids,
				existing_dirs,
				&mut out,
			);
		}
	}

	// 2. MultiMC
	let mmc_candidates = [
		home.join(".local/share/multimc/instances"),
		home.join("MultiMC/instances"),
	];

	for base in mmc_candidates {
		if base.exists() && base.is_dir() {
			scan_mmc_instances(&base, "MultiMC", existing_ids, existing_dirs, &mut out);
		}
	}

	// 3. Modrinth App
	let modrinth_candidates = [
		home.join(".local/share/modrinthapp/profiles"),
		home.join(".var/app/com.modrinth.theseus/data/modrinthapp/profiles"),
	];

	for base in modrinth_candidates {
		if base.exists() && base.is_dir() {
			scan_modrinth_profiles(&base, existing_ids, existing_dirs, &mut out);
		}
	}

	// 4. .minecraft
	let dot_minecraft = home.join(".minecraft");
	if dot_minecraft.exists() && dot_minecraft.join("versions").exists() {
		let path_str = dot_minecraft.to_string_lossy().to_string();
		let is_imported = existing_ids.contains(&"minecraft-default".to_string())
			|| existing_dirs
				.iter()
				.any(|d| path_str == *d || path_str.starts_with(d));
		out.push(DiscoveredInstance {
			id: "minecraft-default".into(),
			name: "Стандартный .minecraft".into(),
			launcher_name: "Официальный лаунчер".into(),
			mc_version: "1.21.1".into(),
			loader: None,
			loader_display: "Vanilla".into(),
			game_dir: dot_minecraft.clone(),
			icon_path: None,
			is_already_imported: is_imported,
		});
	}

	out
}

fn scan_mmc_instances(
	instances_dir: &Path,
	launcher_name: &str,
	existing_ids: &[String],
	existing_dirs: &[String],
	out: &mut Vec<DiscoveredInstance>,
) {
	let Ok(entries) = std::fs::read_dir(instances_dir) else {
		return;
	};

	let icons_dir = instances_dir.parent().map(|p| p.join("icons"));

	for entry in entries.flatten() {
		let path = entry.path();
		if !path.is_dir() {
			continue;
		}

		let folder_name = entry.file_name().to_string_lossy().to_string();
		let cfg_path = path.join("instance.cfg");
		let pack_path = path.join("mmc-pack.json");

		if !cfg_path.exists() && !pack_path.exists() {
			continue;
		}

		let mut name = folder_name.clone();
		let mut icon_key: Option<String> = None;

		if cfg_path.exists() {
			if let Ok(content) = std::fs::read_to_string(&cfg_path) {
				for line in content.lines() {
					if let Some(val) = line.strip_prefix("name=") {
						let trimmed = val.trim();
						if !trimmed.is_empty() {
							name = trimmed.to_string();
						}
					} else if let Some(val) = line.strip_prefix("iconKey=") {
						let trimmed = val.trim();
						if !trimmed.is_empty() {
							icon_key = Some(trimmed.to_string());
						}
					}
				}
			}
		}

		let mut mc_version = "Неизвестно".to_string();
		let mut loader = None;
		let mut loader_display = "Vanilla".to_string();

		if pack_path.exists() {
			if let Ok(content) = std::fs::read_to_string(&pack_path) {
				if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
					if let Some(components) = val.get("components").and_then(|x| x.as_array()) {
						for comp in components {
							let uid = comp.get("uid").and_then(|x| x.as_str()).unwrap_or_default();
							let ver = comp
								.get("version")
								.and_then(|x| x.as_str())
								.unwrap_or_default();

							if uid == "net.minecraft" {
								mc_version = ver.to_string();
							} else if uid.contains("fabric") && uid.contains("loader") {
								loader = Some(if ver.is_empty() {
									"fabric".to_string()
								} else {
									format!("fabric:{ver}")
								});
								loader_display = format!("Fabric {ver}");
							} else if uid.contains("neoforge") {
								loader = Some(if ver.is_empty() {
									"neoforge".to_string()
								} else {
									format!("neoforge:{ver}")
								});
								loader_display = format!("NeoForge {ver}");
							} else if uid.contains("forge") {
								loader = Some(if ver.is_empty() {
									"forge".to_string()
								} else {
									format!("forge:{ver}")
								});
								loader_display = format!("Forge {ver}");
							} else if uid.contains("quilt") {
								loader = Some(if ver.is_empty() {
									"quilt".to_string()
								} else {
									format!("quilt:{ver}")
								});
								loader_display = format!("Quilt {ver}");
							}
						}
					}
				}
			}
		}

		let game_dir = if path.join("minecraft").exists() {
			path.join("minecraft")
		} else if path.join(".minecraft").exists() {
			path.join(".minecraft")
		} else {
			path.clone()
		};

		let mut icon_path = None;
		if let (Some(key), Some(ref idir)) = (icon_key, icons_dir.as_ref()) {
			let cand_png = idir.join(format!("{key}.png"));
			let cand_webp = idir.join(format!("{key}.webp"));
			let cand_jpg = idir.join(format!("{key}.jpg"));
			if cand_png.exists() {
				icon_path = Some(cand_png.to_string_lossy().to_string());
			} else if cand_webp.exists() {
				icon_path = Some(cand_webp.to_string_lossy().to_string());
			} else if cand_jpg.exists() {
				icon_path = Some(cand_jpg.to_string_lossy().to_string());
			}
		}

		let id = make_valid_instance_id(&name);
		let path_str = game_dir.to_string_lossy().to_string();
		let is_already_imported = existing_ids.contains(&id)
			|| existing_dirs
				.iter()
				.any(|d| path_str == *d || path_str.starts_with(d));

		out.push(DiscoveredInstance {
			id,
			name,
			launcher_name: launcher_name.to_string(),
			mc_version,
			loader,
			loader_display,
			game_dir,
			icon_path,
			is_already_imported,
		});
	}
}

fn scan_modrinth_profiles(
	profiles_dir: &Path,
	existing_ids: &[String],
	existing_dirs: &[String],
	out: &mut Vec<DiscoveredInstance>,
) {
	let Ok(entries) = std::fs::read_dir(profiles_dir) else {
		return;
	};

	for entry in entries.flatten() {
		let path = entry.path();
		if !path.is_dir() {
			continue;
		}

		let profile_json = path.join("profile.json");
		if !profile_json.exists() {
			continue;
		}

		let Ok(content) = std::fs::read_to_string(&profile_json) else {
			continue;
		};
		let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) else {
			continue;
		};

		let name = val
			.get("name")
			.and_then(|x| x.as_str())
			.map(|s| s.to_string())
			.unwrap_or_else(|| entry.file_name().to_string_lossy().to_string());

		let mc_version = val
			.get("game_version")
			.and_then(|x| x.as_str())
			.unwrap_or("Неизвестно")
			.to_string();

		let loader_str = val
			.get("loader")
			.and_then(|x| x.as_str())
			.unwrap_or("vanilla");
		let (loader, loader_display) = match loader_str.to_lowercase().as_str() {
			"fabric" => (Some("fabric".to_string()), "Fabric".to_string()),
			"neoforge" => (Some("neoforge".to_string()), "NeoForge".to_string()),
			"forge" => (Some("forge".to_string()), "Forge".to_string()),
			"quilt" => (Some("quilt".to_string()), "Quilt".to_string()),
			_ => (None, "Vanilla".to_string()),
		};

		let icon_path = val
			.get("icon")
			.and_then(|x| x.as_str())
			.map(|x| x.to_string());

		let id = make_valid_instance_id(&name);
		let path_str = path.to_string_lossy().to_string();
		let is_already_imported = existing_ids.contains(&id)
			|| existing_dirs
				.iter()
				.any(|d| path_str == *d || path_str.starts_with(d));

		out.push(DiscoveredInstance {
			id,
			name,
			launcher_name: "Modrinth App".into(),
			mc_version,
			loader,
			loader_display,
			game_dir: path,
			icon_path,
			is_already_imported,
		});
	}
}

pub fn inspect_custom_folder(folder: &Path) -> Option<DiscoveredInstance> {
	if !folder.is_dir() {
		return None;
	}

	let cfg_path = folder.join("instance.cfg");
	let pack_path = folder.join("mmc-pack.json");
	if cfg_path.exists() || pack_path.exists() {
		let mut list = Vec::new();
		scan_mmc_instances(folder.parent()?, "Внешняя сборка", &[], &[], &mut list);
		if let Some(inst) = list
			.into_iter()
			.find(|x| inst_matches_folder(&x.game_dir, folder))
		{
			return Some(inst);
		}
	}

	let name = folder
		.file_name()
		.map(|x| x.to_string_lossy().to_string())
		.unwrap_or_else(|| "Сборка".into());

	let mc_version = "1.21.1".to_string();
	let mut loader = None;
	let mut loader_display = "Vanilla".to_string();

	if folder.join("mods").exists() {
		loader = Some("fabric".to_string());
		loader_display = "Fabric/Forge (моды)".to_string();
	}

	let game_dir = if folder.join("minecraft").exists() {
		folder.join("minecraft")
	} else if folder.join(".minecraft").exists() {
		folder.join(".minecraft")
	} else {
		folder.to_path_buf()
	};

	Some(DiscoveredInstance {
		id: make_valid_instance_id(&name),
		name,
		launcher_name: "Пользовательская папка".into(),
		mc_version,
		loader,
		loader_display,
		game_dir,
		icon_path: None,
		is_already_imported: false,
	})
}

fn inst_matches_folder(game_dir: &Path, folder: &Path) -> bool {
	game_dir == folder || game_dir.starts_with(folder) || folder.starts_with(game_dir)
}
