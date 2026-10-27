use std::collections::HashSet;
use std::path::PathBuf;
use std::rc::Rc;

use sqrilizz::{
	config_crate::ConfigKind,
	pkg::search::{PackageMultiSearchResults, PackageSearchSession},
	pkg_crate::PackageMetaAndProps,
	shared::{
		loaders::Loader,
		pkg::{
			ArcPkgReq, PackageCategory, PackageKind, PackageSearchParameters, PkgRequest,
			PkgRequestSource,
		},
	},
};

use crate::{
	components::{
		footer::FooterItem,
		input::{
			select::{Dropdown, SelectOption, Selected},
			text::{TextInput, search_bar},
		},
		nav::page_buttons::PageButtons,
		pkg::{
			RepoSelector,
			filters::{PackageFilters, PackageTypeFilter},
		},
		tag::{installed_tag, repo_tag},
	},
	ops::{
		ConditionalQuery,
		instance::{FetchInstanceConfig, FetchItems, InstancesAndTemplates},
		packages::{
			FetchInstanceAddons, FetchInstanceLockfile, SearchPackages, SearchPackagesParams,
		},
	},
	pages::{config::ConfiguredItem, package::view::PackageView},
	prelude::*,
	util::addon_info::{
		extract_local_addon_metadata, get_addon_cache_dir, get_addon_metadata_cached,
		resolve_remote_addons_batch,
	},
};

const PAGE_SIZE: u8 = 16;

fn is_pkg_installed(
	req: &ArcPkgReq,
	meta_name: Option<&str>,
	installed_set: &HashSet<String>,
) -> bool {
	if installed_set.is_empty() {
		return false;
	}
	let id_lower = req.id.to_lowercase();
	if installed_set.contains(&id_lower) {
		return true;
	}
	let id_clean: String = id_lower.chars().filter(|c| c.is_alphanumeric()).collect();
	if !id_clean.is_empty() && installed_set.contains(&id_clean) {
		return true;
	}
	if let Some(slug) = &req.slug {
		let slug_lower = slug.to_lowercase();
		if installed_set.contains(&slug_lower) {
			return true;
		}
		let slug_clean: String = slug_lower.chars().filter(|c| c.is_alphanumeric()).collect();
		if !slug_clean.is_empty() && installed_set.contains(&slug_clean) {
			return true;
		}
	}
	if let Some(name) = meta_name {
		let name_lower = name.to_lowercase();
		if installed_set.contains(&name_lower) {
			return true;
		}
		let name_clean: String = name_lower.chars().filter(|c| c.is_alphanumeric()).collect();
		if !name_clean.is_empty() && installed_set.contains(&name_clean) {
			return true;
		}
	}
	if let Some(repo) = &req.repository {
		let repo_id = format!("{repo}:{}", req.id).to_lowercase();
		if installed_set.contains(&repo_id) {
			return true;
		}
		if let Some(slug) = &req.slug {
			let repo_slug = format!("{repo}:{slug}").to_lowercase();
			if installed_set.contains(&repo_slug) {
				return true;
			}
		}
	}
	false
}

#[derive(PartialEq)]
pub struct BrowsePackagesPage {
	pub filters: Option<BrowseFilters>,
}

impl Component for BrowsePackagesPage {
	fn render(&self) -> impl IntoElement {
		let theme = use_theme();
		let front_state = use_front_state();
		let back_state = use_consume::<BackState>();
		let search_session = use_state(|| PackageSearchSession::new(PAGE_SIZE));

		let active_instance =
			use_state(|| self.filters.as_ref().and_then(|x| x.instance_id.clone()));

		let search_state = PackageSearchState::new(&back_state, self.filters.clone());

		// Synchronize state if incoming filters change
		let mut search_state_sync = search_state.clone();
		let mut active_instance_sync = active_instance;
		let filters_prop = self.filters.clone();
		use_side_effect(move || {
			if let Some(f) = &filters_prop {
				active_instance_sync.set_if_modified(f.instance_id.clone());
				if !f.mc_versions.is_empty() {
					search_state_sync
						.mc_versions
						.set_if_modified(f.mc_versions.clone());
				}
				if f.loader != Loader::Vanilla && f.loader != Loader::Any {
					search_state_sync
						.loaders
						.set_if_modified(vec![f.loader.clone()]);
				}
				if let Some(kind) = f.kind {
					search_state_sync.ty.set_if_modified(kind);
				}
				if let Some(repo) = &f.repo {
					search_state_sync.repo.set_if_modified(Some(repo.clone()));
				}
			}
		});

		let items_query = use_query(FetchItems::new(back_state.clone()));
		let default_items = InstancesAndTemplates::default();
		let items_binding = items_query.read();
		let items_state = items_binding.state();
		let items = items_state.ok().unwrap_or(&default_items);

		let active_id = active_instance.read().clone();
		let current_instance = active_id
			.as_ref()
			.and_then(|id| items.instances.iter().find(|x| &x.id == id).cloned());

		let instance_config = use_query(ConditionalQuery::new(
			FetchInstanceConfig::capability(back_state.clone()),
			active_id.is_some(),
			|| active_id.clone().unwrap_or_default(),
		));

		let lockfile = use_query(ConditionalQuery::new(
			FetchInstanceLockfile::new(back_state.clone()),
			active_id.is_some(),
			|| active_id.clone().unwrap_or_default(),
		));

		let addons_query = use_query(ConditionalQuery::new(
			FetchInstanceAddons::new(back_state.clone()),
			active_id.is_some(),
			|| active_id.clone().unwrap_or_default(),
		));

		let back_state_scan = back_state.clone();
		let addons_scan = addons_query;
		use_side_effect(move || {
			if let Some(addons) = addons_scan.read().state().ok().cloned() {
				let unresolved: Vec<PathBuf> = addons
					.iter()
					.filter_map(|a| {
						a.original_path
							.clone()
							.or_else(|| a.target_paths.first().cloned())
					})
					.filter(|p| {
						get_addon_metadata_cached(p)
							.and_then(|m| m.project_id)
							.is_none()
					})
					.collect();
				if !unresolved.is_empty() {
					let client = back_state_scan.client.clone();
					tokio::spawn(async move {
						let cache_dir = get_addon_cache_dir();
						for p in &unresolved {
							extract_local_addon_metadata(p, &cache_dir);
						}
						resolve_remote_addons_batch(unresolved, cache_dir, client).await;
					});
				}
			}
		});

		let instance_config_state = instance_config.read();
		let lockfile_state = lockfile.read();
		let addons_state = addons_query.read();
		let installed_packages = use_memo(move || {
			let mut set = HashSet::new();
			let mut add_entry = |s: &str| {
				let trimmed = s.trim();
				if trimmed.is_empty() {
					return;
				}
				let lower = trimmed.to_lowercase();
				set.insert(lower.clone());
				let clean: String = lower.chars().filter(|c| c.is_alphanumeric()).collect();
				if !clean.is_empty() {
					set.insert(clean);
				}
			};

			if let Some(Some(configs)) = instance_config_state.state().ok() {
				for pkg in &configs.main.packages {
					let id = pkg.get_id();
					add_entry(id);
					let req = pkg.get_req();
					add_entry(&req.id);
					if let Some(slug) = &req.slug {
						add_entry(slug);
					}
					if let Some(repo) = &req.repository {
						add_entry(&format!("{repo}:{id}"));
						add_entry(&format!("{repo}:{}", req.id));
						if let Some(slug) = &req.slug {
							add_entry(&format!("{repo}:{slug}"));
						}
					}
				}
			}
			if let Some(lock) = lockfile_state.state().ok() {
				for (pkg, _) in lock.get_packages() {
					add_entry(pkg);
					let parsed = PkgRequest::parse(pkg, PkgRequestSource::UserRequire);
					add_entry(&parsed.id);
					if let Some(slug) = &parsed.slug {
						add_entry(slug);
					}
					if let Some(repo) = &parsed.repository {
						add_entry(&format!("{repo}:{pkg}"));
						add_entry(&format!("{repo}:{}", parsed.id));
						if let Some(slug) = &parsed.slug {
							add_entry(&format!("{repo}:{slug}"));
						}
					}
				}
			}
			if let Some(addons) = addons_state.state().ok() {
				let cache_dir = get_addon_cache_dir();
				for addon in addons {
					let clean_fname = addon
						.file_name
						.trim_end_matches(".disabled")
						.trim_end_matches(".jar")
						.trim_end_matches(".zip")
						.trim_end_matches(".mrpack");
					add_entry(clean_fname);

					if let Some((prefix, _)) = clean_fname.split_once('-') {
						add_entry(prefix);
					}
					if let Some((prefix, _)) = clean_fname.split_once('_') {
						add_entry(prefix);
					}
					if let Some((prefix, _)) = clean_fname.split_once('+') {
						add_entry(prefix);
					}

					let path = addon
						.original_path
						.as_ref()
						.or_else(|| addon.target_paths.first());
					if let Some(p) = path {
						let meta = get_addon_metadata_cached(p)
							.or_else(|| extract_local_addon_metadata(p, &cache_dir));
						if let Some(meta) = meta {
							if let Some(mid) = &meta.mod_id {
								add_entry(mid);
							}
							if let Some(title) = &meta.title {
								add_entry(title);
							}
							if let Some(slug) = &meta.slug {
								add_entry(slug);
							}
							if let Some(pid) = &meta.project_id {
								add_entry(pid);
							}
						}
					}
				}
			}
			set
		});

		let search_state2 = search_state.clone();
		let search = use_memo(move || search_state2.to_search_params());

		let search_state2 = search_state.clone();
		let results_query = use_query(Query::new(
			SearchPackagesParams {
				search: search.read().cloned(),
				session: Captured(search_session.peek().cloned()),
				repo: search_state2.repo.read().cloned(),
			},
			SearchPackages::new(back_state.clone()).toast(
				&back_state,
				None,
				"Не удалось загрузить пакеты",
			),
		));
		let results = use_state(|| PackageMultiSearchResults {
			results: Vec::new(),
			total_results: 0,
		});

		if back_state.repos().is_empty() {
			return placeholder("No package repositories available", &theme).font_size(18.0);
		}

		let results_query2 = results_query;
		let mut search_session2 = search_session;
		let mut results2 = results;
		use_side_effect(move || {
			if let Some(result) = results_query2.read().state().ok() {
				results2.set(result.0.clone());
				search_session2.set(result.1.clone());
			}
		});

		let selected_pkg = search_state.previewed;
		let active_instance_id_for_footer = active_instance.read().clone();
		let front_state_footer = front_state.clone();
		use_side_effect(move || {
			if let Some(req) = &*selected_pkg.read() {
				let target_item = active_instance_id_for_footer
					.as_ref()
					.map(|id| ConfiguredItem {
						id: Some(id.clone()),
						ty: ConfigKind::Instance,
						is_new: false,
					});
				front_state_footer
					.write()
					.set_footer(FooterItem::InstallPackage {
						req: req.clone(),
						target_item,
					});
			} else {
				front_state_footer.write().set_footer(FooterItem::None);
			}
		});

		let mut page2 = search_state.page;
		let page_buttons = PageButtons {
			page: (*search_state.page.read()).into(),
			total_pages: results.read().total_results.div_ceil(PAGE_SIZE as usize),
			on_set: (move |new_page| page2.set(new_page as u16)).into(),
		};

		let ty_selector = PackageTypeFilter {
			ty: search_state.ty,
			repo: search_state.repo.read().clone(),
		};

		let search = TextInput::new(search_state.search);

		let search_state_inst = search_state.clone();
		let active_instance_setter = active_instance;
		let instances_clone = items.instances.clone();

		let instance_dropdown = Dropdown::new(
			Selected::Single(active_instance.read().clone()),
			Rc::new(move |selected| {
				let mut active_setter = active_instance_setter;
				let mut search_inst = search_state_inst.clone();
				let new_id = selected.single();
				active_setter.set(new_id.clone());
				if let Some(id) = new_id {
					if let Some(inst) = instances_clone.iter().find(|x| x.id == id) {
						if let Some(v) = &inst.version {
							search_inst.mc_versions.set(vec![v.to_string()]);
						}
						if let Some(l) = &inst.loader {
							if *l != Loader::Vanilla && *l != Loader::Any {
								search_inst.loaders.set(vec![l.clone()]);
							} else {
								search_inst.loaders.set(Vec::new());
							}
						}
					}
				} else {
					search_inst.mc_versions.set(Vec::new());
					search_inst.loaders.set(Vec::new());
				}
				search_inst.page.set(0);
			}),
		)
		.panel_colorway()
		.header_width(Size::px(180.0))
		.child(SelectOption::new(None, "Все сборки", Some("box")).tip("Без фильтра по сборке"))
		.children(items.instances.iter().map(|inst| {
			let label = inst.name().to_string();
			let version_str = inst
				.version
				.as_ref()
				.map(|v| v.to_string())
				.unwrap_or_else(|| "без версии".into());
			let loader_str = inst
				.loader
				.as_ref()
				.map(|l| l.to_string())
				.unwrap_or_default();
			let tip_text = format!("{label} ({version_str} • {loader_str})");

			SelectOption::new(Some(inst.id.clone()), &label, Some("rocket")).tip(&tip_text)
		}));

		let mut active_instance_clear = active_instance;
		let mut search_state_clear = search_state.clone();
		let clear_instance_button = if active_instance.read().is_some() {
			Some(rect().tip(&front_state, "Сбросить фильтр сборки").child(
				icon_button("cross", &theme).on_press(move |_| {
					active_instance_clear.set(None);
					search_state_clear.mc_versions.set(Vec::new());
					search_state_clear.loaders.set(Vec::new());
					search_state_clear.page.set(0);
				}),
			))
		} else {
			None
		};

		let search_state2 = search_state.clone();
		let top_bar_right = rect()
			.width(Size::flex(3.5))
			.height(Size::fill())
			.cont()
			.child(
				rect()
					.height(Size::fill())
					.center()
					.horizontal()
					.spacing(theme.gap2)
					.child(instance_dropdown)
					.maybe_child(clear_instance_button),
			)
			.child(rect().height(Size::fill()).center().child(RepoSelector {
				repo: search_state.repo,
			}))
			.child(rect().height(Size::fill()).center().child(ty_selector))
			.child(
				rect()
					.width(Size::flex(1.0))
					.height(Size::fill())
					.center()
					.child(search_bar(search, &theme)),
			)
			.child(rect().height(Size::fill()).center().child(PackageFilters {
				repo: search_state.repo.read().clone(),
				loaders: search_state.loaders,
				mc_versions: search_state.mc_versions,
				categories: search_state.categories,
				on_reset: EventHandler::new(move |_| {
					search_state2.clone().reset();
				}),
			}));

		let top_bar = rect()
			.width(Size::fill())
			.height(Size::px(theme.input_height + theme.gap2 * 2.0))
			.padding(Gaps::new(theme.gap2, theme.gap2, theme.gap2, 0.0))
			.cont()
			.border(border_bottom(theme.border, theme.panel_border))
			.child(
				rect()
					.width(Size::flex(1.0))
					.height(Size::fill())
					.center()
					.child(page_buttons),
			)
			.child(top_bar_right);

		let is_loading =
			results_query.read().state().is_loading() || results_query.read().state().is_pending();
		let packages_data = results.read();
		let packages_view = ScrollView::new().expanded().spacing(theme.gap);
		let installed_set = installed_packages.read();
		let packages = if is_loading {
			packages_view.children(
				(0..PAGE_SIZE)
					.map(|_| skeleton(Size::fill(), Size::px(64.0), &theme).into_element()),
			)
		} else if packages_data.results.is_empty() {
			packages_view.child(placeholder("Ничего не найдено", &theme))
		} else {
			packages_view.children(packages_data.results.iter().map(|req| {
				let preview = search_session.peek().previews().get(req).map(|x| x.clone());
				let meta_name = preview.as_ref().and_then(|x| x.meta.name.as_deref());
				let is_installed = is_pkg_installed(req, meta_name, &installed_set);

				BrowseItem {
					req: req.clone(),
					preview,
					selected_package: selected_pkg,
					is_installed,
				}
				.into_element()
			}))
		};

		let instance_banner = if let Some(inst) = &current_instance {
			let ver_str = inst
				.version
				.as_ref()
				.map(|v| v.to_string())
				.unwrap_or_else(|| "Версия не указана".into());
			let loader_str = inst
				.loader
				.as_ref()
				.map(|l| l.to_string())
				.unwrap_or_default();
			let installed_count = {
				let pkg_count = if let Some(Some(c)) = instance_config.read().state().ok() {
					c.main.packages.len()
				} else {
					0
				};
				let addon_count = addons_query
					.read()
					.state()
					.ok()
					.map(|a| a.len())
					.unwrap_or(0);
				pkg_count.max(addon_count)
			};

			let banner_title = format!("Сборка: {}", inst.name());
			let banner_sub = format!("{ver_str} • {loader_str} • {installed_count} в сборке");
			Some(
				rect()
					.width(Size::fill())
					.padding(Gaps::new(theme.gap2, theme.gap2, theme.gap2, theme.gap2))
					.margin(Gaps::new(0.0, 0.0, theme.gap2, 0.0))
					.corner_radius(theme.round)
					.background(theme.panel)
					.border(border_bottom(theme.border, theme.panel_border))
					.horizontal()
					.cross_align(Alignment::Center)
					.spacing(theme.gap2)
					.child(icon("check", 14.0).color(theme.success))
					.child(
						rect()
							.width(Size::flex(1.0))
							.child(
								clip_text(&banner_title)
									.font_size(12.0)
									.font_weight(FontWeight::BOLD)
									.color(theme.fg),
							)
							.child(clip_text(&banner_sub).font_size(11.0).color(theme.fg3)),
					),
			)
		} else {
			None
		};

		let left_bar = rect()
			.width(Size::flex(1.0))
			.height(Size::fill())
			.vertical()
			.padding(theme.gap)
			.border(border_right(theme.border, theme.panel_border))
			.maybe_child(instance_banner)
			.child(packages);

		let preview = rect()
			.width(Size::flex(3.5))
			.height(Size::fill())
			.maybe(selected_pkg.read().is_some(), |this| {
				let req = selected_pkg.peek().cloned().unwrap();
				let prev = search_session.peek().previews().get(&req).map(|x| x.clone());
				let meta_name = prev.as_ref().and_then(|x| x.meta.name.as_deref());
				let is_installed = is_pkg_installed(&req, meta_name, &installed_packages.read());
				this.child(PackageView {
					req,
					fullscreen: false,
					is_installed,
				})
			})
			.maybe(selected_pkg.read().is_none(), |this| {
				this.child(placeholder("Select a package", &theme))
			});

		rect().expanded().child(top_bar).child(
			rect()
				.width(Size::fill())
				.height(Size::flex(1.0))
				.horizontal()
				.flex()
				.child(left_bar)
				.child(preview),
		)
	}
}

#[derive(Clone)]
struct PackageSearchState {
	repo: State<Option<String>>,
	previewed: State<Option<ArcPkgReq>>,
	page: State<u16>,
	search: State<String>,
	ty: State<PackageKind>,
	categories: State<Vec<PackageCategory>>,
	mc_versions: State<Vec<String>>,
	loaders: State<Vec<Loader>>,
}

impl PackageSearchState {
	fn new(back_state: &BackState, browse_filters: Option<BrowseFilters>) -> Self {
		let out = Self {
			repo: use_state(|| browse_filters.as_ref().and_then(|x| x.repo.clone())),
			previewed: use_state(|| None),
			page: use_state(|| 0),
			search: use_state(String::new),
			ty: use_state(|| {
				browse_filters
					.as_ref()
					.and_then(|x| x.kind)
					.unwrap_or(PackageKind::Mod)
			}),
			categories: use_state(Vec::new),
			mc_versions: use_state(|| {
				browse_filters
					.as_ref()
					.map(|x| x.mc_versions.clone())
					.unwrap_or_default()
			}),
			loaders: use_state(|| {
				browse_filters
					.as_ref()
					.filter(|x| x.loader != Loader::Vanilla && x.loader != Loader::Any)
					.map(|x| vec![x.loader.clone()])
					.unwrap_or_default()
			}),
		};

		let mut state2 = out.clone();
		let back_state = back_state.clone();
		use_side_effect(move || {
			state2.repo.read();
			state2.search.read();
			state2.ty.read();
			state2.categories.read();
			state2.mc_versions.read();
			state2.loaders.read();

			state2.page.set_if_modified(0);
			state2.previewed.set_if_modified(None);

			if let Some(repo) = &*state2.repo.peek()
				&& let Some(repo) = back_state.repos().get(repo)
			{
				if !repo.package_types.is_empty()
					&& !repo.package_types.contains(&*state2.ty.peek())
				{
					state2.ty.set(repo.package_types[0]);
				}

				if !repo.package_categories.is_empty()
					&& !state2
						.categories
						.peek()
						.iter()
						.all(|x| repo.package_categories.contains(x))
				{
					state2.categories.set(Vec::new());
				}
			}
		});

		out
	}

	fn to_search_params(&self) -> PackageSearchParameters {
		PackageSearchParameters {
			count: PAGE_SIZE,
			skip: *self.page.read() as usize * PAGE_SIZE as usize,
			search: Some(self.search.read().clone()).filter(|x| !x.is_empty()),
			types: vec![*self.ty.read()],
			minecraft_versions: self.mc_versions.read().cloned(),
			loaders: self.loaders.read().cloned(),
			categories: self.categories.read().cloned(),
		}
	}

	fn reset(&mut self) {
		self.page.set(0);
		self.search.set(String::new());
		self.categories.set(Vec::new());
		self.mc_versions.set(Vec::new());
		self.loaders.set(Vec::new());
	}
}

#[derive(PartialEq, Clone, Eq, Debug)]
pub struct BrowseFilters {
	pub instance_id: Option<String>,
	pub mc_versions: Vec<String>,
	pub loader: Loader,
	pub kind: Option<PackageKind>,
	pub repo: Option<String>,
}

#[derive(PartialEq)]
struct BrowseItem {
	req: ArcPkgReq,
	preview: Option<PackageMetaAndProps>,
	selected_package: State<Option<ArcPkgReq>>,
	is_installed: bool,
}

impl Component for BrowseItem {
	fn render(&self) -> impl IntoElement {
		let theme = use_theme();
		let back_state = use_consume::<BackState>();
		let is_hovered = use_state(|| false);

		let is_selected = self
			.selected_package
			.read()
			.as_ref()
			.is_some_and(|x| x == &self.req);

		let meta = self.preview.as_ref().map(|x| &x.meta);

		let name = meta.and_then(|x| x.name.as_deref()).unwrap_or(&self.req.id);

		let default_icon = icon("box", 32.0).into_element();
		let ico = meta
			.and_then(|x| x.icon.as_ref())
			.map(|x| {
				let default_icon = default_icon.clone();
				img(x)
					.error_renderer(move |_| default_icon.clone())
					.width(Size::px(40.0))
					.height(Size::px(40.0))
					.corner_radius(theme.round)
					.into_element()
			})
			.unwrap_or(default_icon);

		let description = meta.and_then(|x| x.description.as_deref()).map(|x| {
			rect()
				.width(Size::flex(1.0))
				.child(clip_text(x).color(theme.fg3).font_size(12.0))
		});

		let req = self.req.clone();
		let mut selected_package = self.selected_package;

		let repo = self.req.repository.as_deref().map(|x| {
			rect()
				.cross_align(Alignment::End)
				.margin(Gaps::new(0.0, theme.gap2 * 2.0, 0.0, 0.0))
				.child(repo_tag(x, true, &back_state, &theme))
		});

		let installed_badge = if self.is_installed {
			Some(
				rect()
					.cross_align(Alignment::End)
					.margin(Gaps::new(0.0, theme.gap2, 0.0, 0.0))
					.child(installed_tag(false, &theme)),
			)
		} else {
			None
		};

		let upper_details = rect()
			.width(Size::fill())
			.cont()
			.cross_align(Alignment::Center)
			.child(segment(name, 1.0))
			.maybe_child(installed_badge)
			.maybe_child(repo);
		let lower_details = rect()
			.width(Size::fill())
			.cont()
			.spacing(theme.gap2)
			.cross_align(Alignment::Center)
			.maybe_child(description);
		let details = rect()
			.width(Size::flex(1.0))
			.height(Size::fill())
			.center()
			.spacing(theme.gap)
			.cross_align(Alignment::Start)
			.child(upper_details)
			.child(lower_details);

		rect()
			.width(Size::fill())
			.height(Size::px(64.0))
			.hover(is_hovered)
			.clickable()
			.corner_radius(theme.round)
			.simple_colorway(&theme, *is_hovered.read(), is_selected)
			.cont()
			.spacing(0.0)
			.on_press(move |_| {
				selected_package.set_if_modified(Some(req.clone()));
			})
			.child(
				rect()
					.width(Size::px(64.0))
					.height(Size::fill())
					.center()
					.child(ico),
			)
			.child(details)
	}
}
