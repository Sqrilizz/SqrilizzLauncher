use freya::components::ScrollView;

use sqrilizz::shared::Side;

use crate::{
	components::dialog::modal::Modal,
	data::{format_last_played, format_playtime},
	ops::instance::FetchItems,
	prelude::*,
	routing::Page,
	state::use_launcher_data,
	util::assets::get_instance_icon,
};

#[derive(PartialEq)]
pub struct PlaytimeStatsModal;

impl Component for PlaytimeStatsModal {
	fn render(&self) -> impl IntoElement {
		let theme = use_theme();
		let front_state = use_front_state();
		let back_state = use_consume::<BackState>();
		let data_hook = use_launcher_data();
		let items_query = use_query(FetchItems::new(back_state.clone()));

		let items_read = items_query.read();
		let instances = match &*items_read.state() {
			QueryStateData::Settled { res: Ok(res), .. } => res.instances.clone(),
			_ => Vec::new(),
		};

		let data = data_hook.data.read();
		let total_playtime = data.total_playtime();
		let total_launches = data.total_launches();
		let instances_count = instances.len();

		let mut sorted_instances = instances;
		sorted_instances.sort_by(|a, b| {
			let pt_a = data.get_playtime(&a.id);
			let pt_b = data.get_playtime(&b.id);
			pt_b.cmp(&pt_a).then_with(|| a.name().cmp(b.name()))
		});

		let max_playtime = sorted_instances
			.first()
			.map(|x| data.get_playtime(&x.id))
			.unwrap_or(0)
			.max(1);

		let front_state_close = front_state.clone();
		let close_fn = move |_| {
			front_state_close.write().set_modal(None);
		};

		let metric_card = |icon_name: &'static str, label_text: &'static str, val_text: String| {
			rect()
				.width(Size::flex(1.0))
				.height(Size::px(72.0))
				.background(theme.item)
				.corner_radius(theme.round)
				.border(theme.border(theme.item_border))
				.padding(theme.gap2)
				.horizontal()
				.cross_align(Alignment::Center)
				.spacing(theme.gap2)
				.child(
					rect()
						.width(Size::px(40.0))
						.height(Size::px(40.0))
						.center()
						.background(theme.item_hover)
						.corner_radius(theme.round)
						.child(icon(icon_name, 20.0)),
				)
				.child(
					rect()
						.width(Size::flex(1.0))
						.height(Size::fill())
						.main_align(Alignment::Center)
						.child(
							label()
								.text(label_text.to_string())
								.font_size(theme.font0)
								.color(theme.fg3),
						)
						.child(
							label()
								.text(val_text)
								.font_size(theme.font2)
								.font_weight(FontWeight::BOLD)
								.color(theme.fg),
						),
				)
		};

		let summary = rect()
			.width(Size::fill())
			.height(Size::auto())
			.horizontal()
			.spacing(theme.gap2)
			.child(metric_card(
				"clock",
				"Общее время",
				format_playtime(total_playtime),
			))
			.child(metric_card(
				"rocket",
				"Всего запусков",
				format!("{total_launches}"),
			))
			.child(metric_card(
				"box_highlight",
				"Всего сборок",
				format!("{instances_count}"),
			));

		let mut list = rect().width(Size::fill()).spacing(theme.gap);

		if sorted_instances.is_empty() {
			list = list.child(
				rect()
					.width(Size::fill())
					.height(Size::px(120.0))
					.center()
					.child(
						label()
							.text("Нет установленных сборок".to_string())
							.color(theme.fg3),
					),
			);
		} else {
			for inst in sorted_instances {
				let playtime = data.get_playtime(&inst.id);
				let launches = data.get_launches(&inst.id);
				let last_played = data.get_last_played(&inst.id).unwrap_or(0);
				let pct = if max_playtime > 0 {
					((playtime as f32 / max_playtime as f32) * 100.0).clamp(0.0, 100.0)
				} else {
					0.0
				};

				let inst_icon = if inst.icon.is_none() {
					icon("box_dot", 24.0).into_element()
				} else {
					let icon_source = get_instance_icon(inst.icon.as_deref());
					ImageViewer::new(icon_source)
						.width(Size::px(28.0))
						.height(Size::px(28.0))
						.corner_radius(theme.round)
						.into_element()
				};

				let vers_str = inst
					.version
					.as_ref()
					.map(|v| v.to_string())
					.unwrap_or_else(|| "Неизвестно".to_string());
				let sub_info = format!("{} • {}", vers_str, format_last_played(last_played));

				let front_state_nav = front_state.clone();
				let inst_id = inst.id.clone();
				let page = if inst.side == Some(Side::Server) {
					Page::ServerInstance(inst_id.clone())
				} else {
					Page::Instance(inst_id.clone())
				};
				let row = rect()
					.width(Size::fill())
					.height(Size::px(58.0))
					.background(theme.item)
					.corner_radius(theme.round)
					.border(theme.border(theme.item_border))
					.padding(Gaps::new(theme.gap, theme.gap2, theme.gap, theme.gap2))
					.horizontal()
					.cross_align(Alignment::Center)
					.spacing(theme.gap2)
					.cursor(CursorIcon::Pointer)
					.clickable()
					.on_press(move |_| {
						front_state_nav.write().set_modal(None);
						front_state_nav.write().navigate(page.clone());
					})
					.child(
						rect()
							.width(Size::px(36.0))
							.height(Size::px(36.0))
							.center()
							.child(inst_icon),
					)
					.child(
						rect()
							.width(Size::flex(1.0))
							.height(Size::fill())
							.main_align(Alignment::Center)
							.child(
								label()
									.text(inst.name().to_string())
									.font_weight(FontWeight::BOLD)
									.font_size(theme.font2)
									.color(theme.fg),
							)
							.child(
								label()
									.text(sub_info)
									.font_size(theme.font0)
									.color(theme.fg3),
							),
					)
					.child(
						rect()
							.width(Size::px(110.0))
							.height(Size::px(6.0))
							.background(theme.item_hover)
							.corner_radius(3.0)
							.overflow(Overflow::Clip)
							.child(
								rect()
									.width(Size::percent(pct))
									.height(Size::fill())
									.background(theme.primary)
									.corner_radius(3.0),
							),
					)
					.child(
						rect()
							.width(Size::px(130.0))
							.height(Size::fill())
							.main_align(Alignment::Center)
							.cross_align(Alignment::End)
							.child(
								label()
									.text(format_playtime(playtime))
									.font_weight(FontWeight::BOLD)
									.font_size(theme.font2)
									.color(theme.fg),
							)
							.child(
								label()
									.text(format!("{launches} зап."))
									.font_size(theme.font0)
									.color(theme.fg3),
							),
					);

				list = list.child(row);
			}
		}

		let scrollable_list = ScrollView::new().expanded().child(list);

		let body = rect()
			.width(Size::fill())
			.height(Size::fill())
			.spacing(theme.gap2)
			.padding(theme.gap2)
			.child(summary)
			.child(
				label()
					.text("Время по сборкам".to_string())
					.font_weight(FontWeight::BOLD)
					.font_size(theme.font2)
					.color(theme.fg),
			)
			.child(scrollable_list);

		Modal::new("Статистика времени".into(), "clock".into())
			.size_large()
			.maybe_child(true, || body)
			.on_close(close_fn)
	}
}
