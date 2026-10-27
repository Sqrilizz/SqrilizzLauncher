use std::{
	collections::{HashMap, HashSet},
	hash::{DefaultHasher, Hash, Hasher},
	path::PathBuf,
	rc::Rc,
	sync::Arc,
};

use itertools::Itertools;
use sqrilizz::{
	config_crate::{
		ConfigKind,
		template::{TemplateConfig, TemplatePackageConfiguration},
	},
	instance_crate::{addon::Addon, lock::InstanceLockfile},
	pkg_crate::{PkgRequest, PkgRequestSource},
	shared::{
		Side,
		pkg::{ArcPkgReq, PackageKind},
		util::{from_string_json, to_string_json},
		versions::VersionPattern,
	},
};

use crate::{
	components::{
		input::{
			final_value_owned,
			select::Selected,
			switch::Switch,
			text::{TextInput, search_bar},
		},
		pkg::{error::ResolutionErrorView, versions::InstalledPackageVersion},
	},
	ops::{
		ConditionalQuery,
		instance::{UpdateInstance, UpdateInstanceKeys, UpdateInstanceMode},
		packages::{
			FetchInstanceAddons, FetchInstanceLockfile, FetchPackages, PkgInfo, PreloadPackages,
		},
	},
	pages::{config::ConfigState, package::browse::BrowseFilters},
	prelude::*,
	routing::Page,
	state::{FrontState, use_launcher_data},
	util::{
		PtrEq, Shared,
		addon_info::{
			extract_local_addon_metadata, get_addon_cache_dir, get_addon_metadata_cached,
			resolve_remote_addons_batch,
		},
		assets::get_package_kind_icon,
	},
};

#[derive(PartialEq)]
pub struct ContentConfig {
	pub config_state: ConfigState,
	pub parent_configs: PtrEq<[TemplateConfig]>,
	pub on_edit: Option<EventHandler<()>>,
}

impl Component for ContentConfig {
	fn render(&self) -> impl IntoElement {
		let theme = use_theme();
		let front_state = use_front_state();
		let back_state = use_consume::<BackState>();
		let lockfile = use_query(ConditionalQuery::new(
			FetchInstanceLockfile::new(back_state.clone()),
			self.config_state.ty == ConfigKind::Instance,
			|| self.config_state.id.read().cloned(),
		));
		let addons = use_query(ConditionalQuery::new(
			FetchInstanceAddons::new(back_state.clone()),
			self.config_state.ty == ConfigKind::Instance,
			|| self.config_state.id.read().cloned(),
		));
		let update = use_mutation(Mutation::new(UpdateInstance::new(back_state.clone())));
		let data = use_launcher_data();

		let filter = use_state(|| Filter::All);
		let pkg_ty = use_state::<Option<PackageKind>>(|| None);
		let search = use_state(String::new);

		let open_states = use_state::<HashSet<String>>(HashSet::new);
		let updates_map = use_state(|| Arc::new(HashMap::<String, String>::new()));
		let is_checking_updates = use_state(|| false);

		let modpack = self.config_state.modpack;
		let packages = self.config_state.packages;
		let parent_configs = self.parent_configs.clone();
		let results = use_memo(move || {
			build_items(
				lockfile.read().state().ok(),
				modpack.read().as_ref(),
				&packages.read(),
				&parent_configs.0,
				addons.read().state().ok().map(|x| x.as_slice()),
			)
		});

		let preload = use_mutation(PreloadPackages::new(back_state.clone()));
		let ty = self.config_state.ty;
		use_side_effect(move || {
			let results = results.read();
			if lockfile.read().state().is_ok() || ty != ConfigKind::Instance {
				preload.mutate(results.1.clone());
			}
		});

		let scan_trigger = use_state(|| 0usize);
		let scan_trigger_effect = scan_trigger;
		let results_scan = results;
		let last_scanned_len = use_state(|| 0usize);
		let last_scanned_len_effect = last_scanned_len;
		use_side_effect(move || {
			let res = results_scan.read();
			let addon_paths: Vec<PathBuf> =
				res.0.iter().filter_map(|x| x.addon_path.clone()).collect();

			if !addon_paths.is_empty() && addon_paths.len() != *last_scanned_len_effect.read() {
				last_scanned_len_effect.clone().set(addon_paths.len());
				let scan_trigger = scan_trigger_effect.clone();
				spawn(async move {
					let cache_dir = get_addon_cache_dir();
					let paths_clone = addon_paths.clone();
					let cache_dir_clone = cache_dir.clone();

					let (extracted_any, unresolved) = tokio::task::spawn_blocking(move || {
						let mut extracted = false;
						let mut unresolved = Vec::new();
						for path in &paths_clone {
							if extract_local_addon_metadata(path, &cache_dir_clone).is_some() {
								extracted = true;
							} else {
								unresolved.push(path.clone());
							}
						}
						(extracted, unresolved)
					})
					.await
					.unwrap_or((false, Vec::new()));

					if extracted_any {
						let current = *scan_trigger.read();
						scan_trigger.clone().set(current + 1);
					}

					if !unresolved.is_empty() {
						let client = reqwest::Client::new();
						resolve_remote_addons_batch(unresolved, cache_dir, client).await;
						let current = *scan_trigger.read();
						scan_trigger.clone().set(current + 1);
					}
				});
			}
		});

		let packages = use_query(ConditionalQuery::new(
			FetchPackages::new(back_state.clone()),
			preload.read().state().is_ok(),
			move || results.read().1.clone(),
		));
		let default_packages = Arc::new(HashMap::new());

		let packages = use_memo(move || {
			let packages = packages.read();
			let packages = packages.state();
			PtrEq(packages.ok().unwrap_or(&default_packages).clone())
		});

		let packages2 = packages;
		let scan_trigger_memo = scan_trigger;
		let processed_items = use_memo(move || {
			let _ = *scan_trigger_memo.read();
			filter_sort_items(
				results.read().0.clone(),
				&packages2.read().0,
				&filter.read(),
				&search.read(),
				pkg_ty.read().as_ref(),
			)
		});

		let items = if processed_items.read().is_empty() {
			rect()
				.expanded()
				.child(placeholder(
					"Здесь ничего нет. Попробуйте изменить фильтры или добавьте пакеты.",
					&theme,
				))
				.into_element()
		} else {
			let config_state = self.config_state.clone();
			let processed_items2 = processed_items;
			let open_states2 = open_states;
			let open_states3 = open_states;
			let theme2 = theme.clone();
			let on_edit = self.on_edit.clone();
			let scan_trigger_vs = scan_trigger;
			VirtualScrollView::new_with_data(
				(
					packages.read().cloned(),
					processed_items.read().cloned(),
					open_states.read().cloned(),
					*scan_trigger_vs.read(),
					updates_map.read().clone(),
				),
				move |item, (packages, processed_items, open_states, scan_version, updates)| {
					let item = processed_items.get(item.index).unwrap();
					let open_states2 = open_states2;
					let id = item.id.to_string();
					let open_toggle = EventHandler::new(move |_: ()| {
						let contains = open_states2.read().contains(&id);
						if contains {
							open_states2.clone().write().remove(&id);
						} else {
							open_states2.clone().write().insert(id.clone());
						}
					});

					ContentItemElem {
						item: item.clone(),
						packages: packages.0.clone(),
						config_state: config_state.clone(),
						is_open: open_states.contains(&item.id.to_string()),
						on_edit: on_edit.clone(),
						open_toggle,
						scan_version: *scan_version,
						updates: updates.clone(),
					}
					.into_element()
				},
			)
			.expanded()
			.item_size(move |i| {
				let height = processed_items2
					.read()
					.get(i)
					.map(|x| {
						ContentItemElem::height(x, open_states3.read().contains(&x.id.to_string()))
					})
					.unwrap_or(ContentItemElem::base_height(false));

				height + theme2.gap
			})
			.length(processed_items.read().len())
			.into_element()
		};

		let filters = Dropdown::from_state(filter)
			.header_width(Size::fill())
			.child(SelectOption::new(Filter::All, "Все", Some("asterisk")))
			.child(SelectOption::new(
				Filter::Enabled,
				"Включенные",
				Some("check"),
			))
			.child(SelectOption::new(
				Filter::Disabled,
				"Отключенные",
				Some("cross"),
			))
			.child(SelectOption::new(
				Filter::Configured,
				"Основные",
				Some("gear"),
			))
			.child(SelectOption::new(
				Filter::Dependencies,
				"Зависимости",
				Some("scale"),
			));

		let pkg_ty2 = pkg_ty;
		let ty_selector = Dropdown::new(
			Selected::Single(*pkg_ty.read()),
			Rc::new(move |selected| {
				pkg_ty2.clone().set(selected.single());
			}),
		)
		.header_width(Size::fill())
		.child(SelectOption::new(None, "Все типы", Some("asterisk")))
		.children(
			[
				PackageKind::Mod,
				PackageKind::ResourcePack,
				PackageKind::Datapack,
				PackageKind::Plugin,
				PackageKind::Shader,
				PackageKind::Modpack,
			]
			.into_iter()
			.map(|x| SelectOption::new(x, x.to_string_pretty(), Some(get_package_kind_icon(x)))),
		);

		let search_input = search_bar(
			TextInput::new(search).placeholder("Поиск по названию или ID..."),
			&theme,
		);

		let front_state2 = front_state.clone();
		let back_state2 = back_state.clone();
		let parent_configs = self.parent_configs.clone();
		let id = self.config_state.id;
		let version = self.config_state.version;
		let loader = match self.config_state.side.read().as_ref() {
			Some(Side::Client) | None => self.config_state.client_loader,
			Some(Side::Server) => self.config_state.server_loader,
		};
		let ty = self.config_state.ty;
		let is_dirty = self.config_state.is_dirty.clone();
		let browse_button = icon_text_button("search", "+ Добавить моды", &theme)
			.border_fill(theme.primary)
			.color(theme.primary)
			.background(theme.primary_bg)
			.hover_background(theme.primary_bg)
			.on_press(move |_| {
				let front_state2 = front_state2.clone();
				let back_state2 = back_state2.clone();
				let parent_configs = parent_configs.clone();
				let id = id;
				let version = version;
				let loader = loader;
				spawn(async move {
					if *is_dirty.read() {
						front_state2
							.write()
							.toast(Toast::warning("У вас есть несохраненные изменения", None));
						return;
					}

					let id = id.read().clone();
					let version =
						final_value_owned(version.read().clone(), &parent_configs.0, |x| {
							x.instance.version.as_ref().map(to_string_json)
						});
					let canonical_version = if let Some(version) = version {
						let id_canon = id.clone();
						let canonical_version = tokio::spawn(async move {
							let version = from_string_json(&version).ok()?;
							back_state2
								.canonicalize_version(Some(&id_canon), ty, &version)
								.await
						})
						.await;

						canonical_version.ok().flatten()
					} else {
						None
					};

					front_state2
						.write()
						.navigate(Page::Packages(Some(BrowseFilters {
							instance_id: Some(id),
							mc_versions: canonical_version.into_iter().collect(),
							loader: loader.read().clone().unwrap_or_default(),
							kind: None,
							repo: None,
						})));
				});
			});

		let id = self.config_state.id;
		let is_dirty = self.config_state.is_dirty.clone();
		let front_state2 = front_state.clone();
		let is_updating = update.read().state().is_loading();
		let update_button = icon_text_button("cycle", "Обновить пакеты", &theme)
			.active(&theme)
			.on_press(move |_| {
				if is_updating {
					return;
				}
				if *is_dirty.read() {
					front_state2
						.write()
						.toast(Toast::warning("У вас есть несохраненные изменения", None));
					return;
				}

				update.mutate(UpdateInstanceKeys {
					id: id.read().clone(),
					mode: UpdateInstanceMode::Packages,
					force: false,
				});
			});

		let id_str = self.config_state.id.read().clone();
		let back_state_folder = back_state.clone();
		let open_mods_button =
			icon_text_button("folder", "Папка модов", &theme).on_press(move |_| {
				let id_clone = id_str.clone();
				let back_clone = back_state_folder.clone();
				spawn_forever(async move {
					if let Ok(config) = back_clone.config().await {
						if let Some(inst) = config
							.instances
							.get(&sqrilizz::shared::id::InstanceID::from(id_clone))
						{
							if let Some(dir) = inst.dir() {
								let mods_dir = dir.join("mods");
								let _ = std::fs::create_dir_all(&mods_dir);
								let _ = open::that(&mods_dir);
							}
						}
					}
				});
			});

		let total_count = processed_items.read().len();
		let count_badge = rect()
			.horizontal()
			.cross_align(Alignment::Center)
			.spacing(theme.gap)
			.padding(Gaps::new(theme.gap, theme.gap2, theme.gap, theme.gap2))
			.background(theme.panel)
			.border(theme.border(theme.panel_border))
			.corner_radius(theme.round)
			.child(icon("honeycomb", 14.0).color(theme.fg3))
			.child(
				label()
					.text(format!("Всего: {total_count}"))
					.font_size(12.0)
					.color(theme.fg2),
			);

		let is_checking = *is_checking_updates.read();
		let updates_map_check = updates_map;
		let is_checking_hook = is_checking_updates;
		let results_check = results.clone();
		let packages_check = packages.clone();
		let version_check = version.read().clone();
		let loader_check = loader.read().clone();
		let front_state_check = front_state.clone();

		let check_updates_btn = if is_checking {
			icon_text_button("loader", "Проверка...", &theme)
		} else {
			icon_text_button("search", "Проверить обновления", &theme).on_press(move |_| {
				let mut is_checking_hook = is_checking_hook;
				let mut updates_map_check = updates_map_check;
				is_checking_hook.set(true);

				let items = results_check.read().0.clone();
				let version_check = version_check.clone();
				let loader_check = loader_check.clone();
				let packages_check = packages_check.clone();
				let pkgs_info = packages_check.read().0.clone();
				let mc_ver = version_check.clone();
				let ld_str = loader_check.clone().map(|l| l.to_string().to_lowercase());
				let fs = front_state_check.clone();

				spawn(async move {
					let found = crate::ops::modpack_io::check_mod_updates(
						&items,
						&pkgs_info,
						mc_ver.as_deref(),
						ld_str.as_deref(),
					)
					.await;

					let count = found.len();
					updates_map_check.set(Arc::new(found));
				is_checking_hook.set(false);

					if count > 0 {
						fs.write().toast(Toast::success(&format!("Найдено обновлений: {count}")));
					} else {
						fs.write().toast(Toast::info("Все моды обновлены до актуальных версий!", None));
					}
				});
			})
		};

		let updates_count = updates_map.read().len();
		let update_id = id;
		let update_mut = update.clone();
		let mut updates_map_clear = updates_map;
		let update_all_btn = if updates_count > 0 {
			Some(
				icon_text_button(
					"arrow_up",
					&format!("Обновить всё ({updates_count})"),
					&theme,
				)
				.border_fill(theme.success)
				.color(theme.success)
				.background(theme.success_bg)
				.hover_background(theme.success_bg)
				.on_press(move |_| {
					update_mut.mutate(UpdateInstanceKeys {
						id: update_id.read().clone(),
						mode: UpdateInstanceMode::Packages,
						force: true,
					});
					updates_map_clear.set(Arc::new(HashMap::new()));
				}),
			)
		} else {
			None
		};

		let top_bar = rect()
			.width(Size::fill())
			.cont()
			.cross_align(Alignment::Center)
			.child(browse_button)
			.child(open_mods_button)
			.child(update_button)
			.child(check_updates_btn)
			.maybe_child(update_all_btn)
			.child(rect().width(Size::flex(1.0)))
			.child(count_badge);

		let filter_bar = rect()
			.width(Size::fill())
			.cont()
			.cross_align(Alignment::Center)
			.child(segment(search_input, 2.5))
			.child(segment(ty_selector, 1.2))
			.child(segment(filters, 1.1));

		let header = rect()
			.width(Size::fill())
			.spacing(theme.gap2)
			.padding(Gaps::new(0.0, 0.0, theme.gap, 0.0))
			.child(top_bar)
			.child(filter_bar);

		let resolution_error = data
			.data
			.read()
			.last_resolution_errors
			.get(&*self.config_state.id.read())
			.map(|x| ResolutionErrorView {
				error: PtrEq(Arc::new(x.clone())),
			});

		rect()
			.expanded()
			.spacing(theme.gap)
			.padding(theme.gap2)
			.child(header)
			.maybe_child(resolution_error)
			.child(items)
	}
}

#[derive(PartialEq, Clone)]
enum Filter {
	Configured,
	Dependencies,
	All,
	Enabled,
	Disabled,
}

struct ContentItemElem {
	item: ContentItem,
	packages: Arc<HashMap<ArcPkgReq, anyhow::Result<PkgInfo>>>,
	config_state: ConfigState,
	is_open: bool,
	open_toggle: EventHandler<()>,
	on_edit: Option<EventHandler<()>>,
	scan_version: usize,
	updates: Arc<HashMap<String, String>>,
}

impl ContentItemElem {
	fn base_height(is_modpack: bool) -> f32 {
		if is_modpack { 74.0 } else { 58.0 }
	}

	fn height(item: &ContentItem, is_open: bool) -> f32 {
		let mut height = Self::base_height(item.is_modpack());
		if is_open {
			let len = if item.is_modpack() {
				item.locked_packages.0.len()
			} else {
				item.locked_addons.0.len()
			};
			height += len as f32 * SubItem::height();
		}

		height
	}
}

impl PartialEq for ContentItemElem {
	fn eq(&self, other: &Self) -> bool {
		self.item == other.item
			&& self.is_open == other.is_open
			&& self.scan_version == other.scan_version
			&& Arc::ptr_eq(&self.packages, &other.packages)
			&& Arc::ptr_eq(&self.updates, &other.updates)
	}
}

impl Component for ContentItemElem {
	fn render(&self) -> impl IntoElement {
		let theme = use_theme();
		let front_state = use_front_state();
		let back_state = use_consume::<BackState>();
		let update = use_mutation(Mutation::new(UpdateInstance::new(back_state.clone())));

		let is_hovered = use_state(|| false);

		let info = self
			.item
			.req()
			.and_then(|req| self.packages.get(req).and_then(|x| x.as_ref().ok()));

		let default_icon = icon("box", 32.0).into_element();

		let (ico, name, id) = match &self.item.ty {
			ContentItemType::Modpack { req } | ContentItemType::Package { req } => {
				if let Some(info) = info {
					let ico = info
						.meta
						.icon
						.as_deref()
						.map(|x| {
							img(x)
								.width(Size::px(40.0))
								.height(Size::px(40.0))
								.corner_radius(theme.round)
								.into_element()
						})
						.unwrap_or(default_icon);

					(
						ico,
						self.item.get_name(&self.packages).to_string(),
						Some(self.item.id.to_string()),
					)
				} else if let Some(Err(err)) = self.packages.get(req) {
					(
						icon("error", 24.0).color(theme.error).into_element(),
						err.root_cause().to_string(),
						Some(self.item.id.to_string()),
					)
				} else {
					(
						CircularLoader::new().into_element(),
						"Загрузка...".into(),
						Some(self.item.id.to_string()),
					)
				}
			}
			ContentItemType::Addon => {
				let meta = self
					.item
					.addon_path
					.as_deref()
					.and_then(get_addon_metadata_cached);

				let raw_id = self.item.id.to_string();
				let clean_id = raw_id
					.strip_suffix(".disabled")
					.unwrap_or(&raw_id)
					.to_string();

				let (ico, display_title, subtitle) = if let Some(meta) = meta {
					let ico = if let Some(icon_path) = meta.icon_path {
						ImageViewer::new(PathBuf::from(icon_path))
							.width(Size::px(36.0))
							.height(Size::px(36.0))
							.corner_radius(theme.round)
							.into_element()
					} else {
						self.item
							.addon_ty
							.map(get_package_kind_icon)
							.map(|x| icon(x, 24.0).into_element())
							.unwrap_or(default_icon)
					};

					let title = meta.title.filter(|t| !t.trim().is_empty());
					if let Some(title) = title {
						(ico, title, Some(clean_id))
					} else {
						(ico, clean_id, None)
					}
				} else {
					let ico = self
						.item
						.addon_ty
						.map(get_package_kind_icon)
						.map(|x| icon(x, 24.0).into_element())
						.unwrap_or(default_icon);
					(ico, clean_id, None)
				};

				(ico, display_title, subtitle)
			}
		};

		// Helper handlers for addons
		let instance_id = self.config_state.id.read().clone();
		let addon_path_for_toggle = self.item.addon_path.clone();
		let is_currently_enabled = self.item.is_enabled;
		let front_state_toggle = front_state.clone();
		let on_addon_toggle = EventHandler::new(move |_: ()| {
			let Some(path) = addon_path_for_toggle.clone() else {
				return;
			};
			let instance_id = instance_id.clone();
			let front_state2 = front_state_toggle.clone();
			spawn(async move {
				let res = tokio::task::spawn_blocking(move || {
					let file_name = path.file_name()?.to_string_lossy().to_string();
					if is_currently_enabled {
						let new_name = format!("{file_name}.disabled");
						let new_path = path.with_file_name(new_name);
						std::fs::rename(&path, &new_path).ok()
					} else {
						let new_name = file_name.strip_suffix(".disabled")?;
						let new_path = path.with_file_name(new_name);
						std::fs::rename(&path, &new_path).ok()
					}
				})
				.await;

				if let Ok(Some(_)) = res {
					invalidate_matching::<FetchInstanceAddons>(instance_id).await;
				} else {
					front_state2
						.write()
						.toast(Toast::error("Не удалось изменить статус файла", None));
				}
			});
		});

		let instance_id_del = self.config_state.id.read().clone();
		let addon_path_for_delete = self.item.addon_path.clone();
		let file_title = self.item.id.to_string();
		let front_state_del = front_state.clone();
		let on_addon_delete = move || {
			let Some(path) = addon_path_for_delete.clone() else {
				return;
			};
			let instance_id = instance_id_del.clone();
			let front_state2 = front_state_del.clone();
			let title = file_title.clone();
			spawn(async move {
				let res = tokio::task::spawn_blocking(move || {
					if path.is_dir() {
						std::fs::remove_dir_all(&path).is_ok()
					} else {
						std::fs::remove_file(&path).is_ok()
					}
				})
				.await;

				if let Ok(true) = res {
					front_state2
						.write()
						.toast(Toast::success(&format!("Файл удален: {title}")));
					invalidate_matching::<FetchInstanceAddons>(instance_id).await;
				} else {
					front_state2
						.write()
						.toast(Toast::error("Не удалось удалить файл", None));
				}
			});
		};

		let header_size = 16.0 + theme.gap * 2.0;
		let config_state2 = self.config_state.clone();
		let item2 = self.item.clone();
		let on_edit = self.on_edit.clone();
		let front_state2 = front_state.clone();
		let on_addon_toggle_menu = on_addon_toggle.clone();
		let on_addon_delete_menu = on_addon_delete.clone();

		let more_dropdown = Dropdown::new(
			Selected::Single(ItemMoreDropdown::More),
			Rc::new(move |selected| match selected.single() {
				ItemMoreDropdown::More => {}
				ItemMoreDropdown::Info => {
					if let Some(req) = item2.req() {
						front_state2.write().navigate(Page::Package(req.clone()));
					}
				}
				ItemMoreDropdown::Toggle => {
					on_addon_toggle_menu.call(());
				}
				ItemMoreDropdown::OpenFolder => {
					if let Some(path) = &item2.addon_path {
						let dir = path.parent().unwrap_or(path);
						let _ = open::that(dir);
					}
				}
				ItemMoreDropdown::Remove => match &item2.ty {
					ContentItemType::Modpack { .. } => {}
					ContentItemType::Addon => {
						on_addon_delete_menu();
					}
					ContentItemType::Package { req } => {
						config_state2.packages.clone().write().remove_package(req);
						if let Some(on_edit) = &on_edit {
							on_edit.call(());
						}
					}
				},
			}),
		)
		.custom_header(SelectOption::new(
			ItemMoreDropdown::More,
			"",
			Some("ellipsis"),
		))
		.header_width(Size::px(header_size))
		.hide_arrow()
		.options_width(180.0)
		.options_position(Position::new_absolute().right(header_size + theme.gap))
		.maybe_child(self.item.is_package() || self.item.is_modpack(), || {
			SelectOption::new(ItemMoreDropdown::Info, "Инфо", Some("popout"))
		})
		.maybe_child(self.item.is_addon(), || {
			let toggle_text = if self.item.is_enabled {
				"Выключить"
			} else {
				"Включить"
			};
			let toggle_icon = if self.item.is_enabled {
				"cross"
			} else {
				"check"
			};
			SelectOption::new(ItemMoreDropdown::Toggle, toggle_text, Some(toggle_icon))
		})
		.maybe_child(self.item.addon_path.is_some(), || {
			SelectOption::new(
				ItemMoreDropdown::OpenFolder,
				"Показать в папке",
				Some("folder"),
			)
		})
		.maybe_child(
			(self.item.is_package() && self.item.is_configured && !self.item.is_derived)
				|| self.item.is_addon(),
			|| SelectOption::new(ItemMoreDropdown::Remove, "Удалить", Some("trash")),
		);

		let mut badges = Vec::new();

		if let ContentItemType::Package { req } = &self.item.ty {
			badges.push(
				InstalledPackageVersion {
					configured: req.content_version.optional().map(|x| x.to_string()),
					installed: self.item.locked_version.clone(),
				}
				.into_element(),
			);
		}

		if let Some(new_ver) = self.updates.get(&self.item.id.to_string()) {
			badges.push(
				badge("arrow_up", theme.success, &theme)
					.background(theme.success_bg)
					.tip(&front_state, &format!("Доступно обновление: {new_ver}"))
					.into_element(),
			);
		}

		if self.item.is_configured && !(self.item.is_locked && self.item.files_exist) {
			badges.push(
				badge("warning", theme.warning, &theme)
					.tip(
						&front_state,
						"Элемент еще не установлен на диск. Обновите инстанс.",
					)
					.into_element(),
			);
		}
		if self.item.is_derived {
			badges.push(
				badge("diagram", theme.template, &theme)
					.background(theme.template_bg)
					.tip(&front_state, "Унаследовано из шаблона")
					.into_element(),
			);
		}
		if !self.item.is_configured && self.item.is_locked {
			badges.push(
				badge("scale", theme.fg3, &theme)
					.tip(&front_state, "Зависимость другого пакета")
					.into_element(),
			);
		}
		if let Some(kind) = self.item.get_addon_ty(&self.packages) {
			badges.push(
				badge(get_package_kind_icon(kind), theme.fg3, &theme)
					.tip(&front_state, kind.to_string_pretty())
					.into_element(),
			);
		}

		if !self.item.is_enabled {
			badges.push(
				badge("cross", theme.warning, &theme)
					.tip(&front_state, "Отключен")
					.into_element(),
			);
		}

		if self.item.is_modpack()
			&& self.config_state.ty == ConfigKind::Instance
			&& !self.config_state.is_new
		{
			let id = self.config_state.id.read().clone();
			let update_button = icon_button("cycle", &theme)
				.active(&theme)
				.on_press(move |_| {
					update.mutate(UpdateInstanceKeys {
						id: id.clone(),
						mode: UpdateInstanceMode::Modpack,
						force: false,
					});
				});
			let update_button = rect()
				.tip(&front_state, "Обновить сборку и все пакеты")
				.child(update_button);
			badges.push(update_button.into_element());
		}

		if self.item.addon_path.is_some() {
			let on_toggle_switch = on_addon_toggle.clone();
			let switch_elem = rect()
				.tip(
					&front_state,
					if self.item.is_enabled {
						"Выключить"
					} else {
						"Включить"
					},
				)
				.child(Switch {
					enabled: self.item.is_enabled,
					on_toggle: on_toggle_switch,
				});
			badges.push(switch_elem.into_element());
		}

		if self.item.is_addon() {
			let on_del_btn = on_addon_delete.clone();
			let trash_btn = rect()
				.tip(&front_state, "Удалить")
				.child(icon_button("trash", &theme).on_press(move |_| on_del_btn()));
			badges.push(trash_btn.into_element());
		} else if self.item.is_package() && self.item.is_configured && !self.item.is_derived {
			let config_state_del = self.config_state.clone();
			let on_edit_del = self.on_edit.clone();
			let req_del = self.item.req().cloned();
			let trash_btn = rect().tip(&front_state, "Удалить пакет").child(
				icon_button("trash", &theme).on_press(move |_| {
					if let Some(req) = &req_del {
						config_state_del
							.packages
							.clone()
							.write()
							.remove_package(req);
						if let Some(on_edit) = &on_edit_del {
							on_edit.call(());
						}
					}
				}),
			);
			badges.push(trash_btn.into_element());
		}

		badges.push(more_dropdown.into_element());

		let has_subitems = (self.item.is_package() && !self.item.locked_addons.0.is_empty())
			|| (self.item.is_modpack() && !self.item.locked_packages.0.is_empty());

		if has_subitems {
			let open_toggle_btn = self.open_toggle.clone();
			let chevron_icon = if self.is_open { "minus" } else { "angle_down" };
			let chevron_btn = rect()
				.tip(
					&front_state,
					if self.is_open {
						"Скрыть файлы"
					} else {
						"Показать файлы"
					},
				)
				.child(
					icon_button(chevron_icon, &theme).on_press(move |_| open_toggle_btn.call(())),
				);
			badges.push(chevron_btn.into_element());
		}

		let header_height = Self::base_height(self.item.is_modpack());
		let open_toggle = self.open_toggle.clone();
		let border_color = if *is_hovered.read() {
			theme.item_select_border
		} else {
			theme.panel_border
		};

		let header = rect()
			.width(Size::fill())
			.height(Size::px(header_height))
			.cont()
			.corner_radius(theme.round)
			.border(theme.border(border_color))
			.panel_colorway(&theme, *is_hovered.read(), false)
			.hover(is_hovered)
			.maybe(!self.item.is_enabled, |this| this.opacity(0.65))
			.maybe(has_subitems, |this| {
				this.clickable().on_press(move |_| open_toggle.call(()))
			})
			.child(
				rect()
					.width(Size::px(header_height))
					.height(Size::fill())
					.center()
					.child(ico),
			)
			.child(
				rect()
					.width(Size::flex(3.5))
					.height(Size::fill())
					.cont()
					.vertical()
					.main_align(Alignment::Center)
					.cross_align(Alignment::Start)
					.spacing(2.0)
					.child(
						label()
							.text(name)
							.font_size(13.0)
							.font_weight(FontWeight::BOLD)
							.color(theme.fg)
							.max_lines(1),
					)
					.maybe(id.is_some(), |this| {
						this.child(
							label()
								.text(id.unwrap())
								.font_size(11.0)
								.color(theme.fg3)
								.max_lines(1),
						)
					}),
			)
			.child(
				rect()
					.width(Size::flex(4.5))
					.height(Size::fill())
					.cont()
					.main_align(Alignment::End)
					.cross_align(Alignment::Center)
					.padding(Gaps::new(0.0, theme.gap, 0.0, 0.0))
					.children(badges),
			);

		let subitems = rect()
			.width(Size::fill())
			.maybe(self.item.is_package(), |this| {
				this.children(
					self.item
						.locked_addons
						.0
						.iter()
						.map(|x| SubItem::from_addon(x, &theme).render(&theme, &front_state)),
				)
			})
			.maybe(self.item.is_modpack(), |this| {
				this.children(self.item.locked_packages.0.iter().map(|x| {
					SubItem::from_pkg(x, &self.packages, &theme).render(&theme, &front_state)
				}))
			});

		rect()
			.width(Size::fill())
			.margin(Gaps::new(0.0, 0.0, theme.gap, 0.0))
			.panel_colorway(&theme, false, false)
			.corner_radius(theme.round2)
			.child(header)
			.maybe(self.is_open, |this| this.child(subitems))
	}

	fn render_key(&self) -> DiffKey {
		let mut key = DefaultHasher::new();
		self.item.id.hash(&mut key);
		DiffKey::U64(key.finish())
	}
}

#[derive(PartialEq, Clone)]
enum ItemMoreDropdown {
	More,
	Info,
	Toggle,
	OpenFolder,
	Remove,
}

#[derive(PartialEq, Clone, PartialOrd, Eq, Ord, Debug)]
pub struct ContentItem {
	pub ty: ContentItemType,
	pub id: Arc<str>,
	pub is_configured: bool,
	pub is_locked: bool,
	pub is_derived: bool,
	pub files_exist: bool,
	pub locked_version: Option<String>,
	pub locked_addons: PtrEq<[Addon]>,
	pub locked_packages: PtrEq<[ArcPkgReq]>,
	pub addon_ty: Option<PackageKind>,
	pub addon_path: Option<PathBuf>,
	pub is_enabled: bool,
}

impl ContentItem {
	fn is_package(&self) -> bool {
		matches!(&self.ty, ContentItemType::Package { .. })
	}

	fn is_modpack(&self) -> bool {
		matches!(&self.ty, ContentItemType::Modpack { .. })
	}

	fn is_addon(&self) -> bool {
		matches!(&self.ty, ContentItemType::Addon)
	}

	fn req(&self) -> Option<&ArcPkgReq> {
		match &self.ty {
			ContentItemType::Package { req } | ContentItemType::Modpack { req } => Some(req),
			ContentItemType::Addon => None,
		}
	}

	fn get_name<'a>(&'a self, info: &'a HashMap<ArcPkgReq, anyhow::Result<PkgInfo>>) -> &'a str {
		if let Some(req) = self.req() {
			if let Some(Ok(info)) = info.get(req) {
				info.meta.name.as_deref().unwrap_or(&self.id)
			} else {
				&self.id
			}
		} else {
			&self.id
		}
	}

	fn get_addon_ty(
		&self,
		info: &HashMap<ArcPkgReq, anyhow::Result<PkgInfo>>,
	) -> Option<PackageKind> {
		if let Some(req) = self.req() {
			if let Some(Ok(info)) = info.get(req) {
				info.props.kinds.first().copied()
			} else {
				self.addon_ty
			}
		} else {
			self.addon_ty
		}
	}
}

#[derive(PartialEq, Clone, PartialOrd, Eq, Ord, Debug)]
pub enum ContentItemType {
	Modpack { req: ArcPkgReq },
	Package { req: ArcPkgReq },
	Addon,
}

struct SubItem {
	image: Element,
	name: String,
	req: Option<ArcPkgReq>,
}

impl SubItem {
	fn height() -> f32 {
		40.0
	}

	fn from_addon(addon: &Addon, theme: &Theme) -> Self {
		let path = addon
			.original_path
			.as_deref()
			.or_else(|| addon.target_paths.first().map(|p| p.as_path()));
		let img_elem = path
			.and_then(get_addon_metadata_cached)
			.and_then(|m| m.icon_path)
			.map(|icon_path| {
				ImageViewer::new(PathBuf::from(icon_path))
					.width(Size::px(20.0))
					.height(Size::px(20.0))
					.corner_radius(theme.round)
					.into_element()
			});

		let image = img_elem.unwrap_or_else(|| {
			let ico = get_package_kind_icon(PackageKind::from_addon_kind(addon.kind));
			icon(ico, 16.0).into_element()
		});

		Self {
			image,
			name: addon.file_name.clone(),
			req: None,
		}
	}

	fn from_pkg(
		req: &ArcPkgReq,
		info: &HashMap<ArcPkgReq, anyhow::Result<PkgInfo>>,
		theme: &Theme,
	) -> Self {
		let ico = info
			.get(req)
			.and_then(|x| x.as_ref().ok())
			.and_then(|x| x.meta.icon.as_deref())
			.map(|x| {
				img(x)
					.width(Size::px(24.0))
					.height(Size::px(24.0))
					.corner_radius(theme.round)
					.into_element()
			})
			.unwrap_or_else(|| icon("box", 16.0).into_element());

		let name = info
			.get(req)
			.and_then(|x| x.as_ref().ok())
			.and_then(|x| x.meta.name.clone())
			.unwrap_or_else(|| req.to_string_no_version());

		Self {
			image: ico,
			name,
			req: Some(req.clone()),
		}
	}

	fn render(self, theme: &Theme, front_state: &Shared<FrontState>) -> impl IntoElement {
		let height = Self::height();

		let ico = rect()
			.width(Size::px(height))
			.height(Size::px(height))
			.center()
			.child(self.image);

		let name = rect()
			.width(Size::flex(1.0))
			.height(Size::fill())
			.main_align(Alignment::Center)
			.child(self.name);

		let front_state = front_state.clone();
		let req = self.req.clone();
		rect()
			.width(Size::fill())
			.height(Size::px(height))
			.cont()
			.margin(Gaps::new(0.0, 0.0, 0.0, theme.gap2))
			.maybe(self.req.is_some(), |this| {
				this.clickable().on_press(move |_| {
					front_state
						.write()
						.navigate(Page::Package(req.clone().unwrap()));
				})
			})
			.child(ico)
			.child(name)
	}
}

fn build_items(
	lockfile: Option<&InstanceLockfile>,
	modpack: Option<&String>,
	configured_packages: &TemplatePackageConfiguration,
	parent_configs: &[TemplateConfig],
	addons: Option<&[Addon]>,
) -> (Vec<ContentItem>, Vec<ArcPkgReq>) {
	let mut items = Vec::new();
	let mut packages = HashSet::new();

	if let Some(modpack) = modpack {
		let lock_modpack = lockfile.and_then(|x| x.get_modpack());
		let req = PkgRequest::parse(modpack, PkgRequestSource::UserRequire).arc();
		let addons = if let Some(lockfile) = lockfile {
			PtrEq(
				lockfile
					.get_addons()
					.filter(|x| x.from_modpack)
					.map(|x| x.to_addon())
					.collect(),
			)
		} else {
			PtrEq(Arc::default())
		};

		let locked_packages = lock_modpack
			.map(|x| {
				PtrEq(
					x.packages
						.iter()
						.map(|x| PkgRequest::parse(x, PkgRequestSource::UserRequire).arc())
						.collect(),
				)
			})
			.unwrap_or_else(|| PtrEq(Arc::default()));

		items.push(ContentItem {
			ty: ContentItemType::Modpack { req: req.clone() },
			id: Arc::from(modpack.clone()),
			is_configured: true,
			is_locked: lock_modpack.is_some(),
			is_derived: false,
			files_exist: true,
			locked_version: None,
			locked_addons: addons,
			locked_packages,
			addon_ty: None,
			addon_path: None,
			is_enabled: true,
		});

		packages.insert(req);
		if let Some(lock_modpack) = lock_modpack {
			packages.extend(
				lock_modpack
					.packages
					.iter()
					.map(|x| PkgRequest::parse(x, PkgRequestSource::UserRequire).arc()),
			);
		}
	}

	if let Some(lockfile) = &lockfile {
		for (pkg, data) in lockfile.get_packages() {
			let req = PkgRequest::parse(pkg, PkgRequestSource::UserRequire)
				.with_content_version(
					data.content_version
						.as_deref()
						.map(VersionPattern::from)
						.unwrap_or_default(),
				)
				.arc();

			let mut files_exist = true;
			let mut first_addon_path = None;
			let addons = lockfile
				.get_addons()
				.filter(|x| x.is_from_package(pkg))
				.map(|x| x.to_addon())
				.inspect(|x| {
					if !x.exists() {
						files_exist = false;
					}
					if first_addon_path.is_none() {
						first_addon_path = x
							.original_path
							.clone()
							.or_else(|| x.target_paths.first().cloned());
					}
				});

			let locked_addons_list: Arc<[Addon]> = addons.collect();
			let is_enabled = !locked_addons_list.iter().any(|a| a.file_name.ends_with(".disabled"));

			items.push(ContentItem {
				ty: ContentItemType::Package { req: req.clone() },
				id: Arc::from(pkg.clone()),
				is_locked: true,
				is_configured: false,
				is_derived: false,
				locked_version: data.content_version.clone(),
				locked_addons: PtrEq(locked_addons_list),
				locked_packages: PtrEq(Arc::default()),
				files_exist,
				addon_ty: None,
				addon_path: first_addon_path,
				is_enabled,
			});
			packages.insert(req);
		}
	}

	let global_packages = configured_packages.iter_global().map(|x| (x, None));
	let client_packages = configured_packages
		.iter_side(Side::Client)
		.map(|x| (x, Some(Side::Client)));
	let server_packages = configured_packages
		.iter_side(Side::Server)
		.map(|x| (x, Some(Side::Server)));
	for (package, _side) in global_packages
		.chain(client_packages)
		.chain(server_packages)
	{
		let req = package.get_req().arc();
		let item = if let Some(pos) = items
			.iter()
			.position(|x| matches!(&x.ty, ContentItemType::Package {req: req2} if *req2 == req))
		{
			&mut items[pos]
		} else {
			items.push(ContentItem {
				ty: ContentItemType::Package { req: req.clone() },
				id: package.get_id().into(),
				is_configured: true,
				is_locked: false,
				is_derived: false,
				files_exist: false,
				locked_version: None,
				locked_addons: PtrEq(Arc::default()),
				locked_packages: PtrEq(Arc::default()),
				addon_ty: None,
				addon_path: None,
				is_enabled: true,
			});
			items.last_mut().unwrap()
		};
		item.is_configured = true;
		if let ContentItemType::Package { req: req2 } = &mut item.ty {
			*req2 = req.clone();
		}

		packages.insert(req);
	}

	let parent_packages =
		parent_configs
			.iter()
			.fold(TemplatePackageConfiguration::default(), |mut acc, cfg| {
				acc.merge(cfg.packages.clone());
				acc
			});

	let derived_global_packages = parent_packages.iter_global().map(|x| (x, None));
	let derived_client_packages = parent_packages
		.iter_side(Side::Client)
		.map(|x| (x, Some(Side::Client)));
	let derived_server_packages = parent_packages
		.iter_side(Side::Server)
		.map(|x| (x, Some(Side::Server)));
	for (package, _side) in derived_global_packages
		.chain(derived_client_packages)
		.chain(derived_server_packages)
	{
		let req = package.get_req().arc();
		if items
			.iter()
			.any(|x| matches!(&x.ty, ContentItemType::Package {req: req2} if *req2 == req))
		{
			continue;
		}

		let item = if let Some(pos) = items
			.iter()
			.position(|x| matches!(&x.ty, ContentItemType::Package {req: req2} if *req2 == req))
		{
			&mut items[pos]
		} else {
			items.push(ContentItem {
				ty: ContentItemType::Package { req: req.clone() },
				id: package.get_id().into(),
				is_configured: false,
				is_locked: false,
				is_derived: false,
				files_exist: false,
				locked_version: None,
				locked_addons: PtrEq(Arc::default()),
				locked_packages: PtrEq(Arc::default()),
				addon_ty: None,
				addon_path: None,
				is_enabled: true,
			});
			items.last_mut().unwrap()
		};

		if item.is_configured {
			continue;
		}

		item.is_configured = true;
		item.is_derived = true;

		if let ContentItemType::Package { req: req2 } = &mut item.ty {
			*req2 = req.clone();
		}

		packages.insert(req);
	}

	if let Some(addons) = addons {
		for addon in addons {
			let is_matching_locked = items.iter_mut().find(|item| {
				item.locked_addons.0.iter().any(|x| {
					if x.is_source(addon) {
						return true;
					}
					let x_name = x.file_name.trim_end_matches(".disabled");
					let a_name = addon.file_name.trim_end_matches(".disabled");
					x_name == a_name
				})
			});
			if let Some(item) = is_matching_locked {
				if addon.file_name.ends_with(".disabled") {
					item.is_enabled = false;
				}
				if item.addon_path.is_none() {
					item.addon_path = addon
						.original_path
						.clone()
						.or_else(|| addon.target_paths.first().cloned());
				}
				continue;
			}

			let addon_path = addon
				.original_path
				.clone()
				.or_else(|| addon.target_paths.first().cloned());

			let is_enabled = !addon.file_name.ends_with(".disabled");

			items.push(ContentItem {
				ty: ContentItemType::Addon,
				id: addon.file_name.clone().into(),
				is_configured: false,
				is_locked: false,
				is_derived: false,
				files_exist: true,
				locked_version: None,
				locked_addons: PtrEq(Arc::default()),
				locked_packages: PtrEq(Arc::default()),
				addon_ty: Some(PackageKind::from_addon_kind(addon.kind)),
				addon_path,
				is_enabled,
			});
		}
	}

	items.sort();

	(items, packages.into_iter().sorted().collect())
}

fn filter_sort_items(
	mut items: Vec<ContentItem>,
	info: &HashMap<ArcPkgReq, anyhow::Result<PkgInfo>>,
	filter: &Filter,
	search: &str,
	pkg_ty: Option<&PackageKind>,
) -> Vec<ContentItem> {
	let search = search.to_lowercase();

	items.retain(|x| {
		match filter {
			Filter::Dependencies => {
				if x.is_configured || x.ty == ContentItemType::Addon {
					return false;
				}
			}
			Filter::Configured => {
				if !x.is_configured && x.ty != ContentItemType::Addon {
					return false;
				}
			}
			Filter::Enabled => {
				if !x.is_enabled {
					return false;
				}
			}
			Filter::Disabled => {
				if x.is_enabled {
					return false;
				}
			}
			Filter::All => {}
		}

		if let Some(pkg_ty) = pkg_ty {
			if let Some(ty) = x.get_addon_ty(info) {
				if ty != *pkg_ty {
					return false;
				}
			} else {
				return false;
			}
		}

		if !search.is_empty() {
			let to_search = x.get_name(info);
			let matches_name = to_search.to_lowercase().contains(&search);
			let matches_id = x.id.to_lowercase().contains(&search);
			let matches_addon = x
				.addon_path
				.as_deref()
				.and_then(get_addon_metadata_cached)
				.and_then(|m| m.title)
				.map(|t| t.to_lowercase().contains(&search))
				.unwrap_or(false);
			if !matches_name && !matches_id && !matches_addon {
				return false;
			}
		}

		true
	});

	#[derive(PartialEq, Eq, PartialOrd, Ord)]
	enum SortableType {
		Modpack,
		Package,
		Addon,
	}

	#[derive(PartialEq, Eq, PartialOrd, Ord)]
	struct Sort {
		ty: SortableType,
		name: String,
	}

	items.sort_by_key(|x| {
		let ty = match &x.ty {
			ContentItemType::Modpack { .. } => SortableType::Modpack,
			ContentItemType::Package { .. } => SortableType::Package,
			ContentItemType::Addon => SortableType::Addon,
		};

		let name = if let ContentItemType::Addon = &x.ty {
			x.addon_path
				.as_deref()
				.and_then(get_addon_metadata_cached)
				.and_then(|m| m.title)
				.unwrap_or_else(|| x.id.to_string())
		} else {
			x.get_name(info).to_string()
		};

		Sort {
			ty,
			name: name.to_lowercase(),
		}
	});

	items
}

fn badge(ico: &str, color: impl Into<Color>, theme: &Theme) -> Rect {
	let color = color.into();
	rect()
		.corner_radius(theme.round)
		.color(color)
		.padding(5.0)
		.child(icon(ico, 16.0))
}
