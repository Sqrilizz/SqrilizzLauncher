use sqrilizz::instance::tracking::RunningInstanceEntry;

use crate::{
	ops::launch::FetchRunningInstances,
	prelude::*,
	routing::Page,
	state::{BackEvent, use_front_state},
	util::assets::get_instance_icon,
};

#[derive(PartialEq)]
pub struct RunningInstances;

impl Component for RunningInstances {
	fn render(&self) -> impl IntoElement {
		let theme = use_theme();
		let front_state = use_front_state();
		let back_state = use_consume::<BackState>();

		let running_instances = use_query(FetchRunningInstances::new(back_state.clone()));

		let front_state2 = front_state.clone();
		use_future(move || {
			let mut event_rx = front_state2.read().subscribe_events();
			async move {
				while let Ok(ev) = event_rx.recv().await {
					if let BackEvent::UpdateRunningInstances = ev {
						invalidate_all::<FetchRunningInstances>().await;
					}
				}
			}
		});

		let instances = match &*running_instances.read().state() {
			QueryStateData::Settled { res: Ok(res), .. } => res.clone(),
			_ => Vec::new(),
		};

		if instances.is_empty() {
			return rect().into_element();
		}

		let items: Vec<Element> = instances
			.into_iter()
			.map(|entry| RunningInstance { entry }.into_element())
			.collect();

		rect()
			.height(Size::fill())
			.horizontal()
			.cross_align(Alignment::Center)
			.spacing(theme.gap)
			.children(items)
			.into_element()
	}
}

struct RunningInstance {
	entry: RunningInstanceEntry,
}

impl PartialEq for RunningInstance {
	fn eq(&self, other: &Self) -> bool {
		self.entry.pid == other.entry.pid && self.entry.instance_id == other.entry.instance_id
	}
}

impl Component for RunningInstance {
	fn render(&self) -> impl IntoElement {
		let theme = use_theme();
		let front_state = use_front_state();
		let is_hovered = use_state(|| false);

		let inst_id = self.entry.instance_id.clone();
		let front_state_click = front_state.clone();
		let on_press = move |_| {
			front_state_click
				.write()
				.navigate(Page::Instance(inst_id.clone()));
		};

		let bg = if *is_hovered.read() {
			theme.item_hover
		} else {
			theme.item
		};

		rect()
			.height(Size::px(28.0))
			.padding(Gaps::new(2.0, 8.0, 2.0, 8.0))
			.background(bg)
			.border(theme.border(theme.panel_border))
			.corner_radius(theme.round)
			.horizontal()
			.cross_align(Alignment::Center)
			.spacing(6.0)
			.hover(is_hovered)
			.on_press(on_press)
			.child(
				ImageViewer::new(get_instance_icon(None))
					.width(Size::px(16.0))
					.height(Size::px(16.0))
					.corner_radius(theme.round),
			)
			.child(
				label()
					.text(self.entry.instance_id.clone())
					.font_size(12.0)
					.color(theme.fg),
			)
			.into_element()
	}
}
