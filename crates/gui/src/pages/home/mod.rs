use std::rc::Rc;

use crate::{
	components::{
		footer::FooterItem,
		input::{
			select::Selected,
			tabs::TopTabs,
			text::{TextInput, search_bar},
		},
	},
	ops::instance::{FetchItems, InstanceItemInfo, InstancesAndTemplates},
	pages::{config::ConfiguredItem, home::item::InstanceListItem},
	prelude::*,
	state::ModalType,
};
use sqrilizz::{config_crate::ConfigKind, shared::Side};

use crate::pages::package::browse::BrowseFilters;
use crate::routing::Page;
use sqrilizz::shared::{loaders::Loader, pkg::PackageKind};

pub mod item;

#[derive(PartialEq)]
pub struct HomePage;

impl Component for HomePage {
	fn render(&self) -> impl IntoElement {
		let theme = use_theme();
		let back_state = use_consume::<BackState>();
		let front_state = use_front_state();
		let items_query = use_query(FetchItems::new(back_state.clone()));

		let tab = use_state(|| Tab::Instances);
		let filter = use_state::<Option<Side>>(|| None);
		let search = use_state(String::new);
		let selected = use_state::<Option<InstanceItemInfo>>(|| None);

		let front_state2 = front_state.clone();
		use_side_effect(move || {
			if let Some(selected) = selected.read().clone() {
				front_state2
					.write()
					.set_footer(FooterItem::InstanceOrTemplate(selected));
			}
		});

		let items_gap = 20.0;
		let items = items_query.read();
		let items = match &*items.state() {
			QueryStateData::Pending
			| QueryStateData::Loading { res: _ }
			| QueryStateData::Settled { res: Err(..), .. } => InstancesAndTemplates {
				instances: Vec::new(),
				templates: Vec::new(),
			},
			QueryStateData::Settled { res: Ok(res), .. } => res.clone(),
		};

		let items = match &*tab.read() {
			Tab::Instances => &items.instances,
			Tab::Templates => &items.templates,
		};

		let add_placeholder_ty = match &*tab.read() {
			Tab::Instances => ConfigKind::Instance,
			Tab::Templates => ConfigKind::Template,
		};
		let add_placeholder = InstanceListItem::add_placeholder(add_placeholder_ty, selected);

		let items = items
			.iter()
			.filter(|x| {
				if let Some(filter) = &*filter.read()
					&& x.side != Some(*filter)
				{
					return false;
				}

				if !search.read().is_empty() {
					let search = search.read().to_lowercase();
					let name = x.name.as_deref().unwrap_or_default().to_lowercase();
					let id = x.id.to_lowercase();
					if !name.contains(&search) && !id.contains(&search) {
						return false;
					}
				}

				true
			})
			.map(|x| InstanceListItem::new(x.clone(), selected))
			.chain(std::iter::once(add_placeholder));

		let items_elem = grid(4, items).gap(items_gap);

		let items_elem = rect().child(items_elem).width(Size::fill());

		let front_state2 = front_state.clone();
		let add_dropdown = Dropdown::new(
			Selected::Single(AddOption::Add),
			Rc::new(move |selected| match selected.single() {
				AddOption::Add => {}
				AddOption::Instance => {
					front_state2
						.write()
						.set_modal(Some(ModalType::Configuration(ConfiguredItem {
							id: None,
							ty: ConfigKind::Instance,
							is_new: true,
						})))
				}
				AddOption::ModrinthModpack => {
					front_state2
						.write()
						.navigate(Page::Packages(Some(BrowseFilters {
							instance_id: None,
							mc_versions: Vec::new(),
							loader: Loader::Any,
							kind: Some(PackageKind::Modpack),
							repo: Some("modrinth".into()),
						})));
				}
				AddOption::ImportLauncher => {
					front_state2
						.write()
						.set_modal(Some(ModalType::ImportLauncher));
				}
				AddOption::ImportModpackFile => {
					let front_state2 = front_state2.clone();
					let back_state2 = back_state.clone();
					spawn(async move {
						let picked = rfd::AsyncFileDialog::new()
							.add_filter("Modpack (.mrpack / .zip)", &["mrpack", "zip"])
							.pick_file()
							.await;
						let Some(handle) = picked else {
							return;
						};
						let path = handle.path().to_path_buf();
						front_state2
							.write()
							.toast(Toast::info("Импорт модпака...", None));
						match crate::ops::modpack_io::import_modpack_file(&path, &back_state2).await
						{
							Ok(name) => {
								front_state2.write().toast(Toast::success(&format!(
									"Модпак «{name}» успешно импортирован!"
								)));
							}
							Err(e) => {
								front_state2
									.write()
									.toast(Toast::error(&format!("Ошибка импорта: {e:#}"), None));
							}
						}
					});
				}
				AddOption::Template => {
					front_state2
						.write()
						.set_modal(Some(ModalType::Configuration(ConfiguredItem {
							id: None,
							ty: ConfigKind::Template,
							is_new: true,
						})))
				}
			}),
		)
		.custom_header(SelectOption::new(AddOption::Add, "Создать", Some("plus")))
		.header_width(Size::px(112.0))
		.options_width(210.0)
		.hide_arrow()
		.panel_colorway()
		.child(SelectOption::new(
			AddOption::Instance,
			"Новый инстанс",
			Some("box_dot"),
		))
		.child(SelectOption::new(
			AddOption::ModrinthModpack,
			"Новая сборка",
			Some("box_plus"),
		))
		.child(SelectOption::new(
			AddOption::ImportLauncher,
			"Импорт из лаунчера",
			Some("download"),
		))
		.child(SelectOption::new(
			AddOption::ImportModpackFile,
			"Импорт файла (.mrpack / .zip)",
			Some("box_dot"),
		))
		.child(SelectOption::new(
			AddOption::Template,
			"Шаблон",
			Some("diagram"),
		));

		let tabs = TopTabs::from_state(tab)
			.child(SelectOption::new(
				Tab::Instances,
				"Инстансы",
				Some("box_dot"),
			))
			.child(SelectOption::new(
				Tab::Templates,
				"Шаблоны",
				Some("diagram"),
			));

		let bar_left = rect()
			.width(Size::flex(1.0))
			.height(Size::fill())
			.cont()
			.cross_align(Alignment::Center)
			.child(add_dropdown)
			.child(segment(tabs, 1.0));

		let search_bar = search_bar(
			TextInput::new(search).placeholder("Поиск инстансов..."),
			&theme,
		);

		let bar_center = rect()
			.width(Size::flex(1.0))
			.height(Size::fill())
			.center()
			.child(search_bar);

		let filters = TopTabs::from_state(filter)
			.child(SelectOption::new(None, "Все", Some("asterisk")))
			.child(SelectOption::new(
				Some(Side::Client),
				"Клиент",
				Some("controller"),
			))
			.child(SelectOption::new(
				Some(Side::Server),
				"Сервер",
				Some("server"),
			));

		let bar_right = rect()
			.width(Size::flex(1.0))
			.height(Size::fill())
			.center()
			.child(filters);

		let bar_elem = rect()
			.width(Size::fill())
			.height(Size::px(32.0))
			.cont()
			.padding((3.0, items_gap))
			.child(bar_left)
			.child(bar_center)
			.child(bar_right);

		let view = rect().flex().child(bar_elem).child(items_elem);

		let view = ScrollView::new().expanded().child(view);

		rect().expanded().padding(items_gap).child(view)
	}
}

#[derive(PartialEq, Clone)]
enum Tab {
	Instances,
	Templates,
}

#[derive(PartialEq, Clone)]
enum AddOption {
	Add,
	Instance,
	ModrinthModpack,
	ImportLauncher,
	ImportModpackFile,
	Template,
}
