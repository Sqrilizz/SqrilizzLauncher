use std::{rc::Rc, time::Duration};

use crate::{
	components::input::select::Selected,
	ops::account::FetchAccounts,
	pages::settings,
	prelude::*,
	state::{ModalType, use_launcher_data},
	util::assets::DEFAULT_SKIN,
};

pub mod auth;

#[derive(PartialEq)]
pub struct AccountSelector;

impl Component for AccountSelector {
	fn render(&self) -> impl IntoElement {
		let theme = use_theme();
		let front_state = use_front_state();
		let back_state = use_consume::<BackState>();
		let accounts_query = use_query(Query::new((), FetchAccounts::new(back_state.clone())));
		let data = use_launcher_data();

		let default = Vec::new();
		let accounts = accounts_query.read();
		let accounts = accounts.state();
		let accounts = accounts.ok().unwrap_or(&default);

		let data2 = data.clone();
		let current_account_id = data.data.read().current_account.clone();

		let no_account_img = ImageViewer::new(("default-skin", DEFAULT_SKIN))
			.width(Size::px(28.0))
			.height(Size::px(28.0))
			.corner_radius(4.0)
			.sampling_mode(SamplingMode::Nearest);

		Dropdown::new(
			Selected::Single(current_account_id),
			Rc::new(move |selected| {
				data2.data.clone().write().current_account = selected.single();
				data2.save();
			}),
		)
		.header_width(Size::px(240.0))
		.hide_arrow()
		.child(SelectOption::new_custom_icon(
			None,
			"Без аккаунта",
			no_account_img.into_element(),
		))
		.children(accounts.iter().map(|account| {
			let name = account.get_name().unwrap_or(account.get_id());

			let image = ImageViewer::new(get_account_image_with_fallback(
				account.get_uuid(),
				Some(account.get_name().unwrap_or(account.get_id())),
			))
			.asset_age(Duration::from_hours(1))
			.width(Size::px(28.0))
			.height(Size::px(28.0))
			.corner_radius(4.0)
			.sampling_mode(SamplingMode::Nearest);

			let front_state = front_state.clone();
			let action_button =
				icon_button("gear", &theme).on_press(move |e: Event<PressEventData>| {
					e.stop_propagation();
					front_state
						.write()
						.set_modal(Some(ModalType::Settings(settings::Tab::Accounts)));
				});

			SelectOption::new_custom_icon(
				Some(account.get_id().to_string()),
				name,
				image.into_element(),
			)
			.action_button(action_button.into_element())
		}))
	}
}

#[allow(dead_code)]
pub fn get_account_image(uuid: Option<&str>) -> ImageSource {
	get_account_image_with_fallback(uuid, None)
}

pub fn get_account_image_with_fallback(uuid: Option<&str>, username: Option<&str>) -> ImageSource {
	if let Some(uuid) = uuid {
		Url::parse(&format!("https://mc-heads.net/avatar/{uuid}/64"))
			.unwrap_or_else(|_| Url::parse("https://mc-heads.net/avatar/steve/64").unwrap())
			.into()
	} else if let Some(username) = username {
		Url::parse(&format!("https://mc-heads.net/avatar/{username}/64"))
			.unwrap_or_else(|_| Url::parse("https://mc-heads.net/avatar/steve/64").unwrap())
			.into()
	} else {
		("default-skin", DEFAULT_SKIN).into()
	}
}
