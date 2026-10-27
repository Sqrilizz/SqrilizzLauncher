use sqrilizz::config_crate::template::TemplateConfig;

use crate::{
	components::input::derived_value,
	data::InstanceIcon,
	prelude::*,
	state::use_launcher_data,
	util::{
		PtrEq,
		assets::pick_image_file,
	},
};

static OPTIONS: &[&str] = &[
	"builtin:/icons/fabric.png",
	"builtin:/icons/folia.png",
	"builtin:/icons/forge.png",
	"builtin:/icons/minecraft.png",
	"builtin:/icons/neoforge.png",
	"builtin:/icons/paper.png",
	"builtin:/icons/quilt.png",
	"builtin:/icons/sponge.png",
];

#[derive(PartialEq, Clone)]
pub struct IconSelector {
	pub icon: State<Option<String>>,
	pub parent_configs: PtrEq<[TemplateConfig]>,
}

impl ComponentOwned for IconSelector {
	fn render(self) -> impl IntoElement {
		let theme = use_theme();
		let data_hook = use_launcher_data();

		let mut is_open = use_state(|| false);
		let is_hovered = use_state(|| false);

		let derived = derived_value(self.icon.read().as_ref(), &self.parent_configs.0, |x| {
			x.instance.icon.as_ref()
		})
		.cloned();

		let size = 128.0;

		let final_icon = self.icon.read().as_ref().or(derived.as_ref()).cloned();
		let preview = crate::util::gif::AnimatedIconViewer::new(final_icon)
			.width(Size::percent(85.0))
			.height(Size::percent(85.0))
			.image_cover(ImageCover::Center)
			.corner_radius(theme.round);

		let selector_width = 340.0;
		let cols = 5;
		let option_gap = 8.0;
		let option_size = (selector_width - theme.gap * 2.0) / cols as f32 - option_gap;

		let mut icon_choices: Vec<Option<String>> = vec![None];

		let saved = data_hook.data.read().saved_instance_icons.clone();
		for item in saved {
			match item {
				InstanceIcon::File(p) => {
					let s = p.to_string_lossy().to_string();
					if !icon_choices.iter().any(|c| c.as_deref() == Some(&s)) {
						icon_choices.push(Some(s));
					}
				}
			}
		}

		if let Some(curr) = self.icon.read().as_ref() {
			if !icon_choices.iter().any(|c| c.as_deref() == Some(curr))
				&& !OPTIONS.contains(&curr.as_str())
			{
				icon_choices.push(Some(curr.clone()));
			}
		}

		for opt in OPTIONS {
			icon_choices.push(Some(opt.to_string()));
		}

		let selected_pick = self.icon;
		let data_hook_pick = data_hook.clone();
		let pick_button = icon_text_button("upload", "Выбрать своё фото / файл...", &theme)
			.width(Size::fill())
			.on_press(move |e: Event<PressEventData>| {
				e.stop_propagation();
				let mut selected = selected_pick;
				let mut data_hook = data_hook_pick.clone();
				spawn(async move {
					if let Some(path) = pick_image_file().await {
						let path_str = path.to_string_lossy().to_string();
						selected.set(Some(path_str));
						let mut data = data_hook.data.write();
						let entry = InstanceIcon::File(path);
						if !data.saved_instance_icons.contains(&entry) {
							data.saved_instance_icons.push(entry);
						}
						drop(data);
						data_hook.save();
					}
				});
			});

		let selected_plus = self.icon;
		let data_hook_plus = data_hook.clone();
		let plus_tile = rect()
			.width(Size::px(option_size))
			.height(Size::px(option_size))
			.item_colorway(&theme, false, false)
			.corner_radius(theme.round2)
			.center()
			.clickable()
			.cursor(CursorIcon::Pointer)
			.on_press(move |e: Event<PressEventData>| {
				e.stop_propagation();
				let mut selected = selected_plus;
				let mut data_hook = data_hook_plus.clone();
				spawn(async move {
					if let Some(path) = pick_image_file().await {
						let path_str = path.to_string_lossy().to_string();
						selected.set(Some(path_str));
						let mut data = data_hook.data.write();
						let entry = InstanceIcon::File(path);
						if !data.saved_instance_icons.contains(&entry) {
							data.saved_instance_icons.push(entry);
						}
						drop(data);
						data_hook.save();
					}
				});
			})
			.child(icon("plus", 18.0));

		let mut grid_items: Vec<Element> = Vec::new();
		let mut is_first = true;

		for choice in icon_choices {
			let is_selected = choice.as_deref() == self.icon.read().as_deref();
			let is_derived = choice.as_deref() == derived.as_deref() && choice.is_some();

			let image = crate::util::gif::AnimatedIconViewer::new(choice.clone())
				.width(Size::percent(75.0))
				.height(Size::percent(75.0))
				.image_cover(ImageCover::Center)
				.corner_radius(theme.round);

			let mut selected = self.icon;
			let choice_val = choice.clone();

			let tile = rect()
				.width(Size::px(option_size))
				.height(Size::px(option_size))
				.item_colorway(&theme, false, is_selected)
				.maybe(is_derived, |this| this.derived_colorway(&theme))
				.corner_radius(theme.round2)
				.center()
				.clickable()
				.cursor(CursorIcon::Pointer)
				.on_press(move |e: Event<PressEventData>| {
					e.stop_propagation();
					selected.set(choice_val.clone());
				})
				.child(image)
				.into_element();

			grid_items.push(tile);

			if is_first {
				grid_items.push(plus_tile.clone().into_element());
				is_first = false;
			}
		}

		let options = ScrollView::new().child(grid(cols, grid_items).gap(option_gap));

		let mut is_open_header = is_open;
		let header = rect()
			.horizontal()
			.width(Size::fill())
			.cross_align(Alignment::Center)
			.child(
				label()
					.text("Иконка")
					.color(theme.fg)
					.font_size(14.0)
					.font_weight(FontWeight::BOLD),
			)
			.child(rect().width(Size::fill()))
			.child(
				rect()
					.clickable()
					.cursor(CursorIcon::Pointer)
					.padding(4.0)
					.corner_radius(theme.round)
					.on_press(move |e: Event<PressEventData>| {
						e.stop_propagation();
						is_open_header.set(false);
					})
					.child(icon("cross", 14.0)),
			);

		let selector = rect()
			.position(Position::new_absolute().left(size + 8.0))
			.width(Size::px(selector_width))
			.height(Size::px(280.0))
			.background(theme.panel)
			.border(theme.border(theme.panel_border))
			.layer(Layer::Overlay)
			.corner_radius(theme.round2)
			.padding(theme.gap)
			.spacing(theme.gap)
			.clickable()
			.on_press(|e: Event<PressEventData>| e.stop_propagation())
			.child(header)
			.child(pick_button)
			.child(
				rect()
					.width(Size::fill())
					.height(Size::fill())
					.child(options),
			);

		rect()
			.width(Size::px(size))
			.height(Size::px(size))
			.item_colorway(&theme, *is_hovered.read(), false)
			.maybe(derived.is_some(), |this| this.derived_colorway(&theme))
			.hover(is_hovered)
			.clickable()
			.cursor(CursorIcon::Pointer)
			.corner_radius(theme.round2)
			.center()
			.on_press(move |_| is_open.toggle())
			.child(preview)
			.maybe(*is_open.read(), |this| this.child(selector))
	}
}
