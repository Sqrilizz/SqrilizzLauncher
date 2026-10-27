use std::rc::Rc;

use itertools::Itertools;
use sqrilizz::{
	pkg_crate::{
		declarative::DeclarativeAddonVersion, metadata::PackageMetadata,
		properties::PackageProperties,
	},
	shared::{
		loaders::Loader,
		pkg::{ArcPkgReq, PackageStability},
		versions::VersionPattern,
	},
};

use crate::{
	components::{
		input::select::{Dropdown, SelectOption, Selected},
		pkg::install::PackageInstallModal,
		tag::{loader_tag, text_tag},
	},
	ops::packages::FetchPackageContentVersions,
	prelude::*,
	util::{assets::get_loader_icon, PtrEq},
};

#[derive(PartialEq)]
pub struct PackageVersions {
	pub req: ArcPkgReq,
	pub meta: PtrEq<PackageMetadata>,
	pub props: PtrEq<PackageProperties>,
}

impl Component for PackageVersions {
	fn render(&self) -> impl IntoElement {
		let theme = use_theme();
		let front_state = use_front_state();
		let back_state = use_consume::<BackState>();
		let versions_query = use_query(Query::new(
			self.req.clone(),
			FetchPackageContentVersions::new(back_state.clone()),
		));
		let installing_version = use_state::<Option<String>>(|| None);
		let selected_mc_version = use_state::<Option<String>>(|| None);
		let selected_loader = use_state::<Option<Loader>>(|| None);

		let default_versions = Vec::new();
		let versions_state = versions_query.read();
		let versions_state = versions_state.state();
		let is_loading = versions_state.is_loading();
		let all_versions = versions_state.ok().cloned().unwrap_or(default_versions);

		let mut available_mc_versions = Vec::new();
		let mut seen_mc = std::collections::HashSet::new();
		let mut available_loaders = Vec::new();
		let mut seen_loaders = std::collections::HashSet::new();

		for v in &all_versions {
			if let Some(mc_vers) = &v.conditional_properties.minecraft_versions {
				for mv in mc_vers.iter() {
					let s = mv.to_string();
					if seen_mc.insert(s.clone()) {
						available_mc_versions.push(s);
					}
				}
			}
			if let Some(loaders) = &v.conditional_properties.loaders {
				for lm in loaders.iter() {
					for l in lm.get_matches() {
						if seen_loaders.insert(l.clone()) {
							available_loaders.push(l);
						}
					}
				}
			}
		}

		let current_mc = selected_mc_version.read().clone();
		let current_loader = selected_loader.read().clone();

		let filtered_versions: Vec<DeclarativeAddonVersion> = all_versions
			.into_iter()
			.filter(|v| {
				if let Some(target_mc) = &current_mc {
					let matches_mc = v
						.conditional_properties
						.minecraft_versions
						.as_ref()
						.map(|mvs| {
							mvs.iter().any(|pattern| {
								pattern.to_string() == *target_mc
									|| matches!(pattern, VersionPattern::Single(s) if s == target_mc)
									|| matches!(pattern, VersionPattern::Any)
							})
						})
						.unwrap_or(true);
					if !matches_mc {
						return false;
					}
				}
				if let Some(target_loader) = &current_loader {
					let matches_loader = v
						.conditional_properties
						.loaders
						.as_ref()
						.map(|lms| lms.iter().any(|lm| lm.matches(target_loader)))
						.unwrap_or(true);
					if !matches_loader {
						return false;
					}
				}
				true
			})
			.collect();

		let selected_mc2 = selected_mc_version;
		let mut mc_dropdown = Dropdown::new(
			Selected::Single(selected_mc_version.read().clone()),
			Rc::new(move |selected| {
				selected_mc2.clone().set(selected.single());
			}),
		)
		.panel_colorway()
		.loading(is_loading)
		.child(SelectOption::new_custom_icon(
			None,
			"All Game Versions",
			icon("minecraft", 14.0).into_element(),
		));

		for mc in &available_mc_versions {
			mc_dropdown = mc_dropdown.child(SelectOption::new_custom_icon(
				Some(mc.clone()),
				mc,
				icon("tag", 14.0).into_element(),
			));
		}

		let selected_loader2 = selected_loader;
		let mut loader_dropdown = Dropdown::new(
			Selected::Single(selected_loader.read().clone()),
			Rc::new(move |selected| {
				selected_loader2.clone().set(selected.single());
			}),
		)
		.panel_colorway()
		.loading(is_loading)
		.child(SelectOption::new_custom_icon(
			None,
			"All Loaders",
			icon("box_highlight", 14.0).into_element(),
		));

		for l in &available_loaders {
			loader_dropdown = loader_dropdown.child(SelectOption::new_custom_icon(
				Some(l.clone()),
				&l.to_string(),
				get_loader_icon(l).into_element(),
			));
		}

		let has_active_filters =
			selected_mc_version.read().is_some() || selected_loader.read().is_some();
		let filter_bar = rect()
			.width(Size::fill())
			.horizontal()
			.spacing(theme.gap2)
			.cross_align(Alignment::Center)
			.margin(Gaps::new(0.0, 0.0, theme.gap2, 0.0))
			.child(
				rect()
					.width(Size::flex(1.0))
					.child(mc_dropdown),
			)
			.child(
				rect()
					.width(Size::flex(1.0))
					.child(loader_dropdown),
			)
			.maybe(
				has_active_filters,
				|this| {
					let mut selected_mc_clear = selected_mc_version;
					let mut selected_loader_clear = selected_loader;
					this.child(
						rect()
							.tip(&front_state, "Reset filters")
							.child(
								icon_button("cross", &theme).on_press(move |_| {
									selected_mc_clear.set(None);
									selected_loader_clear.set(None);
								}),
							),
					)
				},
			);

		let content = if is_loading {
			rect()
				.expanded()
				.center()
				.child(CircularLoader::new().size(32.0))
				.into_element()
		} else if filtered_versions.is_empty() {
			placeholder("No versions match the selected filters", &theme).into_element()
		} else {
			let versions_for_scroll = filtered_versions.clone();
			VirtualScrollView::new(move |item, _| {
				if let Some(version) = versions_for_scroll.get(item.index) {
					Version {
						version: NotEq(version.clone()),
						installing_version,
					}
					.into_element()
				} else {
					rect().into_element()
				}
			})
			.item_size(48.0 + theme.gap)
			.length(filtered_versions.len())
			.expanded()
			.into_element()
		};

		let mut installing_version2 = installing_version;
		rect()
			.expanded()
			.padding(theme.gap2)
			.child(filter_bar)
			.child(
				rect()
					.width(Size::fill())
					.height(Size::flex(1.0))
					.child(content),
			)
			.maybe(
				installing_version.read().is_some(),
				|this| {
					this.child(PackageInstallModal {
						req: self
							.req
							.with_content_version(VersionPattern::Single(
								installing_version.read().clone().unwrap(),
							))
							.arc(),
						initial_item: None,
						meta: self.meta.clone(),
						props: self.props.clone(),
						on_close: (move |_| {
							installing_version2.set(None);
						})
						.into(),
					})
				},
			)
	}
}

#[derive(PartialEq)]
struct Version {
	version: NotEq<DeclarativeAddonVersion>,
	installing_version: State<Option<String>>,
}

impl Component for Version {
	fn render(&self) -> impl IntoElement {
		let theme = use_theme();
		let front_state = use_front_state();

		let (ico, ico_fg, ico_bg, ico_tip) = match self
			.version
			.0
			.conditional_properties
			.stability
			.unwrap_or(PackageStability::Stable)
		{
			PackageStability::Stable => ("tag", theme.success, theme.success_bg, "Stable"),
			PackageStability::Latest => ("warning", theme.warning, theme.error_bg, "Unstable"),
		};
		let ico = rect()
			.width(Size::px(48.0))
			.height(Size::px(48.0))
			.center()
			.child(
				rect()
					.width(Size::px(24.0))
					.height(Size::px(24.0))
					.center()
					.background(ico_bg)
					.corner_radius(theme.round)
					.tip(&front_state, ico_tip)
					.child(icon(ico, 12.0).color(ico_fg)),
			);

		let name = self
			.version
			.0
			.conditional_properties
			.content_versions
			.as_ref()
			.and_then(|x| x.first().map(|x| x.as_str()))
			.or(self.version.0.version.as_deref())
			.unwrap_or("idk");

		let title = self.version.0.filename.as_deref().unwrap_or(name);

		let mut installing_version = self.installing_version;
		let content_version = self
			.version
			.0
			.conditional_properties
			.content_versions
			.as_ref()
			.and_then(|x| x.first().cloned())
			.or_else(|| self.version.0.version.clone());
		let install = rect()
			.width(Size::px(48.0))
			.height(Size::px(48.0))
			.center()
			.child(
				rect()
					.width(Size::px(24.0))
					.height(Size::px(24.0))
					.center()
					.background(theme.primary_bg)
					.border(theme.border(theme.primary))
					.corner_radius(theme.round)
					.clickable()
					.on_press(move |_| {
						if let Some(version) = &content_version {
							installing_version.set(Some(version.clone()));
						}
					})
					.child(icon("download", 12.0).color(theme.primary)),
			);

		let versions =
			if let Some(versions) = &self.version.0.conditional_properties.minecraft_versions {
				let out = versions
					.iter()
					.map(|x| x.to_string())
					.map(|x| text_tag(&x, &theme))
					.take(3);
				let ellipsis = if versions.len() > 3 {
					Some(text_tag("...", &theme))
				} else {
					None
				};

				out.chain(ellipsis).collect_vec()
			} else {
				Vec::new()
			};
		let loaders = self
			.version
			.0
			.conditional_properties
			.loaders
			.iter()
			.flat_map(|x| x.iter())
			.flat_map(|x| x.get_matches())
			.unique()
			.map(|x| loader_tag(&x, true, &theme).into_element());

		let manual_indicator = if self.version.0.is_manual {
			Some(
				icon("file_cross", 12.0)
					.color(theme.warning)
					.tip(&front_state, "Must be downloaded manually"),
			)
		} else {
			None
		};

		rect()
			.width(Size::fill())
			.height(Size::px(48.0))
			.border(theme.border(theme.panel_border))
			.corner_radius(theme.round)
			.margin(Gaps::new(0.0, 0.0, theme.gap, 0.0))
			.cont()
			.child(ico)
			.child(
				segment(title, 1.0)
					.height(Size::fill())
					.main_align(Alignment::Center),
			)
			.child(
				rect()
					.height(Size::fill())
					.cont()
					.main_align(Alignment::End)
					.cross_align(Alignment::Center)
					.children(versions)
					.children(loaders)
					.maybe_child(manual_indicator),
			)
			.child(install)
	}
}

#[derive(PartialEq)]
pub struct InstalledPackageVersion {
	pub configured: Option<String>,
	pub installed: Option<String>,
}

impl Component for InstalledPackageVersion {
	fn render(&self) -> impl IntoElement {
		let theme = use_theme();
		let front_state = use_front_state();

		let (ico, contents) = if let Some(configured) = &self.configured {
			("lock", configured.clone())
		} else if let Some(installed) = &self.installed {
			("tag", installed.clone())
		} else {
			("tag", "None".to_string())
		};

		let tip = if let Some(configured) = &self.configured {
			if let Some(installed) = &self.installed {
				if configured != installed {
					format!("Requested version {configured}, installed version {installed}")
				} else {
					format!("Requested and installed version {configured}")
				}
			} else {
				format!("Requested version {configured}")
			}
		} else if let Some(installed) = &self.installed {
			format!("Installed version {installed}")
		} else {
			"None".to_string()
		};

		rect()
			.horizontal()
			.cross_align(Alignment::Center)
			.spacing(theme.gap)
			.child(icon(ico, 14.0).color(theme.fg2))
			.child(
				label()
					.text(contents)
					.color(theme.fg2)
					.tip(&front_state, &tip),
			)
	}
}
