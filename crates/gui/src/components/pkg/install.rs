use std::rc::Rc;

use sqrilizz::{
	config_crate::{ConfigKind, instance::make_valid_instance_id},
	pkg_crate::{metadata::PackageMetadata, properties::PackageProperties},
	shared::{
		pkg::{ArcPkgReq, PackageKind},
		versions::VersionPattern,
	},
};

use crate::{
	components::{
		dialog::modal::{MODAL_MEDIUM_HEIGHT, MODAL_MEDIUM_WIDTH, Modal, ModalButton},
		input::{
			select::{Dropdown, SelectOption, Selected},
			tabs::TopTabs,
			text::TextInput,
		},
	},
	ops::{
		ConditionalQuery,
		instance::{FetchInstanceConfig, FetchItems, InstanceItemInfo, InstancesAndTemplates},
		packages::{
			CheckPackageCompatability, CheckPackageCompatabilityKeys, FetchInstanceAddons,
			FetchPackageContentVersions, InstallPackage, PackageCompatabilityError, UninstallPackage,
			PackageInstallLocation,
		},
	},
	pages::config::ConfiguredItem,
	prelude::*,
	util::{PtrEq, assets::get_instance_icon},
};

#[derive(PartialEq)]
pub struct PackageInstallModal {
	pub req: ArcPkgReq,
	pub initial_item: Option<ConfiguredItem>,
	pub meta: PtrEq<PackageMetadata>,
	pub props: PtrEq<PackageProperties>,
	pub on_close: EventHandler<()>,
}

impl Component for PackageInstallModal {
	fn render(&self) -> impl IntoElement {
		let theme = use_theme();
		let back_state = use_consume::<BackState>();
		let install_package = use_mutation(Mutation::new(
			InstallPackage::new(back_state.clone()).toast(
				&back_state,
				Some("Package installed"),
				"Failed to install package",
			),
		));
		let uninstall_package = use_mutation(Mutation::new(
			UninstallPackage::new(back_state.clone()).toast(
				&back_state,
				Some("Пакет удалён из сборки"),
				"Не удалось удалить пакет",
			),
		));
		let items_query = use_query(FetchItems::new(back_state.clone()));

		let is_modpack = self.props.0.kinds.contains(&PackageKind::Modpack);

		let name = self
			.meta
			.0
			.name
			.clone()
			.unwrap_or_else(|| self.req.to_string_no_version());
		let initial_id = make_valid_instance_id(&name);

		let initial_tab = if is_modpack {
			Tab::ModpackInstance
		} else if let Some(item) = &self.initial_item {
			match item.ty {
				ConfigKind::Template | ConfigKind::BaseTemplate => Tab::Template,
				_ => Tab::Instance,
			}
		} else {
			Tab::Instance
		};
		let tab = use_state(|| initial_tab);
		let selected_item = use_state::<Option<ConfiguredItem>>(|| self.initial_item.clone());
		let new_instance_id = use_state(|| initial_id);
		let mut new_instance_id2 = new_instance_id.clone();
		use_side_effect(move || {
			let new_value = make_valid_instance_id(&*new_instance_id2.read());
			new_instance_id2.set_if_modified(new_value);
		});

		let initial_version = match &self.req.content_version {
			VersionPattern::Single(v) => Some(v.clone()),
			_ => None,
		};
		let selected_version = use_state(|| initial_version);

		let versions_query = use_query(Query::new(
			self.req.clone(),
			FetchPackageContentVersions::new(back_state.clone()),
		));

		let current_req = if let Some(v) = selected_version.read().clone() {
			self.req
				.with_content_version(VersionPattern::Single(v))
				.arc()
		} else {
			self.req
				.with_content_version(VersionPattern::Any)
				.arc()
		};

		let compatability_check = use_query(Query::new(
			selected_item
				.read()
				.cloned()
				.map(|x| CheckPackageCompatabilityKeys {
					item: x,
					package: current_req.clone(),
				}),
			CheckPackageCompatability::new(back_state.clone()),
		));
		let compatability_err = compatability_check.read().state().ok().cloned().flatten();

		let prev_tab = use_state(|| tab.peek().clone());
		let tab2 = tab;
		let mut selected_item2 = selected_item;
		let mut prev_tab2 = prev_tab;
		use_side_effect(move || {
			let current_tab = tab2.read().clone();
			if current_tab != *prev_tab2.peek() {
				prev_tab2.set(current_tab);
				selected_item2.set(None);
			}
		});

		let tabs = TopTabs::from_state(tab).children(
			Tab::get_tabs(is_modpack)
				.iter()
				.map(|x| SelectOption::new(x.clone(), x.name(is_modpack), Some(x.icon()))),
		);

		let default_versions = Vec::new();
		let versions = versions_query.read();
		let versions = versions.state();
		let is_versions_loading = versions.is_loading();
		let versions = versions.ok().cloned().unwrap_or(default_versions);

		let selected_version2 = selected_version;
		let mut version_dropdown = Dropdown::new(
			Selected::Single(selected_version.read().clone()),
			Rc::new(move |selected| {
				selected_version2.clone().set(selected.single());
			}),
		)
		.panel_colorway()
		.loading(is_versions_loading)
		.child(SelectOption::new_custom_icon(
			None,
			"Latest (Best compatible)",
			icon("star", 14.0).into_element(),
		));

		let mut seen_ids = std::collections::HashSet::new();
		for v in &versions {
			let id = v
				.conditional_properties
				.content_versions
				.as_ref()
				.and_then(|x| x.first().cloned())
				.or_else(|| v.version.clone());
			if let Some(id) = id {
				if !seen_ids.insert(id.clone()) {
					continue;
				}
				let title = if let Some(filename) = &v.filename {
					let short_name = filename
						.trim_end_matches(".jar")
						.trim_end_matches(".mrpack")
						.trim_end_matches(".zip");
					if short_name != id {
						format!("{short_name} ({id})")
					} else {
						id.clone()
					}
				} else {
					id.clone()
				};
				version_dropdown = version_dropdown.child(SelectOption::new_custom_icon(
					Some(id),
					&title,
					icon("tag", 14.0).into_element(),
				));
			}
		}

		let version_indicator = rect()
			.width(Size::fill())
			.height(Size::px(48.0))
			.padding(Gaps::new(0.0, theme.gap2, 0.0, theme.gap2))
			.horizontal()
			.spacing(theme.gap2)
			.cross_align(Alignment::Center)
			.border(border_bottom(theme.border, theme.panel_border))
			.child(
				rect()
					.horizontal()
					.spacing(theme.gap)
					.cross_align(Alignment::Center)
					.child(icon("tag", 14.0).color(theme.fg2))
					.child(label().text("Version:").color(theme.fg2)),
			)
			.child(rect().width(Size::flex(1.0)).child(version_dropdown));

		let default_items = InstancesAndTemplates::default();
		let items = items_query.read();
		let items = items.state();
		let items = items.ok().unwrap_or(&default_items);

		let tab_contents = match &*tab.read() {
			Tab::Instance => {
				let out = grid(
					3,
					items.instances.iter().map(|x| Item {
						item: x.clone(),
						selected: selected_item,
					}),
				)
				.gap(theme.gap2);

				ScrollView::new().expanded().child(out).into_element()
			}
			Tab::Template => {
				let out = grid(
					3,
					items.templates.iter().map(|x| Item {
						item: x.clone(),
						selected: selected_item,
					}),
				)
				.gap(theme.gap2);

				ScrollView::new().expanded().child(out).into_element()
			}
			Tab::ModpackInstance => rect()
				.width(Size::fill())
				.padding(theme.gap2)
				.child(field(
					"ID for new instance",
					"hashtag",
					&theme,
					TextInput::new(new_instance_id),
				))
				.into_element(),
		};

		let base_error = rect()
			.width(Size::fill())
			.position(Position::new_absolute().bottom(theme.gap))
			.margin((0.0, theme.gap2))
			.layer(Layer::Relative(12))
			.padding(theme.gap2)
			.cont()
			.main_align(Alignment::Start)
			.cross_align(Alignment::Center)
			.corner_radius(theme.round);
		let error = if let Some(error) = &compatability_err {
			let message = match error {
				PackageCompatabilityError::WrongMinecraftVersion => {
					"Package does not support this Minecraft version"
				}
				PackageCompatabilityError::WrongLoader => "Package does not support this loader",
			};
			Some(
				base_error
					.color(theme.warning)
					.background(theme.error_bg)
					.border(theme.border(theme.warning))
					.child(icon("warning", 16.0))
					.child(message),
			)
		} else if compatability_check.read().state().is_loading() {
			Some(
				base_error
					.background(theme.panel)
					.border(theme.border(theme.panel_border))
					.child(CircularLoader::new().size(16.0))
					.child("Checking package compatibility..."),
			)
		} else {
			None
		};

		let selected_inst_id = selected_item
			.read()
			.as_ref()
			.filter(|x| x.ty == ConfigKind::Instance)
			.and_then(|x| x.id.clone());

		let inst_config_query = use_query(ConditionalQuery::new(
			FetchInstanceConfig::capability(back_state.clone()),
			selected_inst_id.is_some(),
			|| selected_inst_id.clone().unwrap_or_default(),
		));

		let inst_addons_query = use_query(ConditionalQuery::new(
			FetchInstanceAddons::new(back_state.clone()),
			selected_inst_id.is_some(),
			|| selected_inst_id.clone().unwrap_or_default(),
		));

		let is_already_installed = {
			let req_id = self.req.id.to_lowercase();
			let req_slug = self.req.slug.as_ref().map(|s| s.to_lowercase());
			let req_name = self.meta.0.name.as_deref().map(|n| n.to_lowercase());
			let id_clean: String = req_id.chars().filter(|c| c.is_alphanumeric()).collect();
			let slug_clean: Option<String> = req_slug
				.as_ref()
				.map(|s| s.chars().filter(|c| c.is_alphanumeric()).collect());
			let name_clean: Option<String> = req_name
				.as_ref()
				.map(|s| s.chars().filter(|c| c.is_alphanumeric()).collect());

			let in_config = if let Some(Some(cfg)) = inst_config_query.read().state().ok() {
				cfg.main.packages.iter().any(|p| {
					let pid = p.get_id().to_lowercase();
					let preq = p.get_req();
					pid == req_id
						|| preq.id.to_lowercase() == req_id
						|| req_slug
							.as_ref()
							.is_some_and(|s| s == &pid || s == &preq.id.to_lowercase())
				})
			} else {
				false
			};

			let in_addons = if let Some(addons) = inst_addons_query.read().state().ok() {
				addons.iter().any(|a| {
					let clean_fname = a
						.file_name
						.trim_end_matches(".disabled")
						.trim_end_matches(".jar")
						.trim_end_matches(".zip")
						.to_lowercase();
					let fname_clean: String = clean_fname
						.chars()
						.filter(|c| c.is_alphanumeric())
						.collect();

					if clean_fname.contains(&req_id)
						|| (!id_clean.is_empty() && fname_clean.contains(&id_clean))
					{
						return true;
					}
					if let Some(s) = &req_slug {
						if clean_fname.contains(s)
							|| slug_clean
								.as_ref()
								.is_some_and(|sc| fname_clean.contains(sc))
						{
							return true;
						}
					}
					if let Some(n) = &req_name {
						if clean_fname.contains(n)
							|| name_clean
								.as_ref()
								.is_some_and(|nc| fname_clean.contains(nc))
						{
							return true;
						}
					}
					let path = a.original_path.as_ref().or_else(|| a.target_paths.first());
					if let Some(p) = path {
						if let Some(meta) = crate::util::addon_info::get_addon_metadata_cached(p) {
							if let Some(mid) = &meta.mod_id {
								let mid_lower = mid.to_lowercase();
								if mid_lower == req_id
									|| req_slug.as_ref().is_some_and(|s| s == &mid_lower)
								{
									return true;
								}
							}
							if let Some(slug) = &meta.slug {
								let slug_lower = slug.to_lowercase();
								if slug_lower == req_id
									|| req_slug.as_ref().is_some_and(|s| s == &slug_lower)
								{
									return true;
								}
							}
							if let Some(pid) = &meta.project_id {
								if pid.to_lowercase() == req_id {
									return true;
								}
							}
						}
					}
					false
				})
			} else {
				false
			};

			in_config || in_addons
		};

		let already_installed_alert = if is_already_installed {
			Some(
				rect()
					.width(Size::fill())
					.padding(Gaps::new(theme.gap, theme.gap2, theme.gap, theme.gap2))
					.margin(Gaps::new(0.0, theme.gap2, 0.0, theme.gap2))
					.horizontal()
					.spacing(theme.gap)
					.cross_align(Alignment::Center)
					.corner_radius(theme.round)
					.background(theme.success_bg)
					.border(theme.border(theme.success))
					.child(icon("check", 14.0).color(theme.success))
					.child(
						label()
							.text("Этот пакет уже установлен в выбранной сборке")
							.color(theme.success)
							.font_size(12.0)
							.font_weight(FontWeight::BOLD),
					),
			)
		} else {
			None
		};

		let contents = rect()
			.flex()
			.child(tabs)
			.child(version_indicator)
			.maybe_child(already_installed_alert)
			.child(
				rect()
					.width(Size::fill())
					.height(Size::flex(1.0))
					.child(tab_contents),
			)
			.maybe_child(error);

		let is_save_ready = match &*tab.read() {
			Tab::ModpackInstance => !new_instance_id.read().is_empty(),
			_ => selected_item.read().is_some() && compatability_err.is_none(),
		};

		let req = self.req.clone();
		let tab = tab;
		let selected_item = selected_item;
		let new_instance_id = new_instance_id;
		let selected_version = selected_version;
		let on_close = self.on_close.clone();
		let req_del = self.req.clone();
		let tab_del = tab;
		let selected_item_del = selected_item;
		let on_close_del = self.on_close.clone();
		Modal::new(format!("Install {name}"), "download".into())
			.size(MODAL_MEDIUM_WIDTH, MODAL_MEDIUM_HEIGHT)
			.maybe_child(true, || contents)
			.on_close(self.on_close.clone())
			.cancel_button()
			.maybe_button(is_already_installed, ModalButton {
				title: "Удалить из сборки".into(),
				icon: "trash".into(),
				on_click: (move |_| {
					let location = match &*tab_del.read() {
						Tab::Instance => {
							let Some(selected) = selected_item_del.read().clone() else {
								return;
							};
							if is_modpack {
								PackageInstallLocation::InstanceModpack(selected.id.unwrap().into())
							} else {
								PackageInstallLocation::Instance(selected.id.unwrap().into())
							}
						}
						Tab::Template => {
							let Some(selected) = selected_item_del.read().clone() else {
								return;
							};
							if selected.ty == ConfigKind::BaseTemplate {
								PackageInstallLocation::BaseTemplate(None)
							} else if is_modpack {
								PackageInstallLocation::TemplateModpack(selected.id.unwrap().into())
							} else {
								PackageInstallLocation::Template(selected.id.unwrap().into(), None)
							}
						}
						Tab::ModpackInstance => return,
					};

					let on_close = on_close_del.clone();
					let req = req_del.clone();
					spawn_forever(async move {
						uninstall_package.mutate_async((req, location)).await;
						on_close.call(());
					});
				}).into(),
				active: false,
				danger: true,
			})
			.button(ModalButton {
				title: if is_already_installed {
					"Обновить".into()
				} else {
					"Установить".into()
				},
				icon: "download".into(),
				on_click: (move |_| {
					let location = match &*tab.read() {
						Tab::Instance => {
							let Some(selected) = selected_item.read().clone() else {
								return;
							};
							if is_modpack {
								PackageInstallLocation::InstanceModpack(selected.id.unwrap().into())
							} else {
								PackageInstallLocation::Instance(selected.id.unwrap().into())
							}
						}
						Tab::Template => {
							let Some(selected) = selected_item.read().clone() else {
								return;
							};
							if selected.ty == ConfigKind::BaseTemplate {
								PackageInstallLocation::BaseTemplate(None)
							} else if is_modpack {
								PackageInstallLocation::TemplateModpack(selected.id.unwrap().into())
							} else {
								PackageInstallLocation::Template(selected.id.unwrap().into(), None)
							}
						}
						Tab::ModpackInstance => PackageInstallLocation::NewInstanceModpack(
							new_instance_id.read().clone().into(),
						),
					};

					let on_close = on_close.clone();
					let final_req = if let Some(ver) = selected_version.read().clone() {
						req.with_content_version(VersionPattern::Single(ver)).arc()
					} else {
						req.with_content_version(VersionPattern::Any).arc()
					};
					spawn_forever(async move {
						install_package.mutate_async((final_req, location)).await;
						on_close.call(());
					});
				})
				.into(),
				active: is_save_ready,
				danger: false,
			})
	}
}

#[derive(PartialEq)]
struct Item {
	item: InstanceItemInfo,
	selected: State<Option<ConfiguredItem>>,
}

impl Component for Item {
	fn render(&self) -> impl IntoElement {
		let theme = use_theme();
		let item = self.item.clone();
		let mut selected = self.selected;

		let icon = ImageViewer::new(get_instance_icon(self.item.icon.as_deref()))
			.width(Size::px(24.0))
			.height(Size::px(24.0))
			.into_element();
		let name = self.item.name().to_string();

		let is_selected = selected
			.read()
			.as_ref()
			.is_some_and(|x| x.id.as_deref() == Some(&self.item.id));

		button(&theme)
			.width(Size::fill())
			.height(Size::px(48.0))
			.border_fill(if is_selected {
				theme.primary
			} else {
				theme.panel_border
			})
			.background(if is_selected {
				theme.primary_bg
			} else {
				theme.panel
			})
			.hover_background(if is_selected {
				theme.primary_bg
			} else {
				theme.item
			})
			.on_press(move |_| {
				selected.set(Some(item.get_config_item()));
			})
			.child(
				rect()
					.expanded()
					.horizontal()
					.spacing(theme.gap)
					.cross_align(Alignment::Center)
					.child(icon)
					.child(
						clip_text(&name)
							.color(if is_selected { theme.primary } else { theme.fg })
							.font_size(13.0)
							.font_weight(FontWeight::BOLD),
					),
			)
	}
}

#[derive(PartialEq, Clone, Copy)]
enum Tab {
	Instance,
	Template,
	ModpackInstance,
}

impl Tab {
	fn name(&self, is_modpack: bool) -> &'static str {
		match self {
			Self::Instance => {
				if is_modpack {
					"Update Instance"
				} else {
					"Instance"
				}
			}
			Self::Template => "Template",
			Self::ModpackInstance => "New Instance",
		}
	}

	fn icon(&self) -> &'static str {
		match self {
			Self::Instance => "rocket",
			Self::Template => "properties",
			Self::ModpackInstance => "plus",
		}
	}

	fn get_tabs(is_modpack: bool) -> &'static [Self] {
		if is_modpack {
			&[Self::ModpackInstance, Self::Instance, Self::Template]
		} else {
			&[Self::Instance, Self::Template]
		}
	}
}
