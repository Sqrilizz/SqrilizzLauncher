use std::{rc::Rc, time::Duration};

use sqrilizz::{
	config_crate::instance::make_valid_instance_id,
	core::account::{Account, AccountKind},
};

use crate::{
	components::{
		account::get_account_image_with_fallback,
		input::{select::Selected, text::TextInput},
	},
	icons::microsoft_icon,
	ops::{
		account::{CreateAccount, DeleteAccount, FetchAccounts, LoginAccount, LogoutAccount},
		plugin_results::FetchAccountTypes,
	},
	prelude::*,
};

#[derive(PartialEq)]
pub struct AccountSettings;

impl Component for AccountSettings {
	fn render(&self) -> impl IntoElement {
		let theme = use_theme();
		let back_state = use_consume::<BackState>();
		let accounts_query = use_query(Query::new((), FetchAccounts::new(back_state.clone())));
		let create_mutation = use_mutation(Mutation::new(CreateAccount::new(back_state.clone())));

		let mut is_editing = use_state(|| false);

		let default = Vec::new();
		let accounts = accounts_query.read();
		let accounts = accounts.state();
		let accounts = accounts.ok().unwrap_or(&default);

		let mut is_editing2 = is_editing;
		let accounts = ScrollView::new()
			.expanded()
			.spacing(theme.gap2)
			.children(accounts.iter().map(|x| {
				AccountItem {
					account: NotEq(x.clone()),
					on_edit_submit: None,
				}
				.into_element()
			}))
			.maybe(!*is_editing.read(), |this| {
				this.child(rect().width(Size::fill()).center().child(
					icon_text_button("plus", "Добавить аккаунт", &theme).on_press(move |_| {
						is_editing2.set(true);
					}),
				))
			})
			.maybe(*is_editing.read(), |this| {
				this.child(
					AccountItem {
						account: NotEq(Account::new(
							AccountKind::Unknown("offline".into()),
							"".into(),
						)),
						on_edit_submit: Some(EventHandler::new(move |account| {
							if let Some(account) = account {
								create_mutation.mutate(NotEq(account));
							}
							is_editing.set(false);
						})),
					}
					.into_element(),
				)
			});

		rect().expanded().padding(theme.gap2).child(accounts)
	}
}

#[derive(PartialEq)]
struct AccountItem {
	account: NotEq<Account>,
	on_edit_submit: Option<EventHandler<Option<Account>>>,
}

impl Component for AccountItem {
	fn render(&self) -> impl IntoElement {
		let theme = use_theme();
		let front_state = use_front_state();
		let back_state = use_consume::<BackState>();
		let login_mutation = use_mutation(Mutation::new(LoginAccount::new(back_state.clone())));
		let logout_mutation = use_mutation(Mutation::new(LogoutAccount::new(back_state.clone())));
		let delete_mutation = use_mutation(Mutation::new(DeleteAccount::new(back_state.clone())));
		let account_types = use_query(Query::new((), FetchAccountTypes::new(back_state.clone())));

		let default = Vec::new();
		let account_types = account_types.read();
		let account_types = account_types.state();
		let account_types = account_types.ok().unwrap_or(&default);

		let editing_id = use_state(String::new);
		let editing_ty = use_state(|| AccountKind::Unknown("offline".into()));

		let mut editing_id2 = editing_id;
		use_side_effect(move || {
			let value = make_valid_instance_id(editing_id.read().as_str());
			editing_id2.set_if_modified(value);
		});

		let account = &self.account.0;
		let is_editing = self.on_edit_submit.is_some();

		let name = account.get_name().unwrap_or(account.get_id());

		let image = ImageViewer::new(get_account_image_with_fallback(
			account.get_uuid(),
			Some(account.get_name().unwrap_or(account.get_id())),
		))
		.asset_age(Duration::from_hours(1))
		.width(Size::px(32.0))
		.height(Size::px(32.0))
		.corner_radius(theme.round)
		.sampling_mode(SamplingMode::Nearest)
		.shiny_border(theme.round, &theme);
		let image = rect()
			.width(Size::px(64.0))
			.height(Size::px(64.0))
			.center()
			.child(image);

		let is_offline = matches!(account.get_kind(), AccountKind::Unknown(k) if k == "offline");
		let is_authenticated = is_offline || account.is_auth_valid(&back_state.paths.core);

		let editing_ty2 = editing_ty;
		let ty_dropdown =
			Dropdown::new(
				Selected::Single(editing_ty.read().cloned()),
				Rc::new(move |selected| {
					editing_ty2.clone().set(selected.single());
				}),
			)
			.panel_colorway()
			.child(SelectOption::new_custom_icon(
				AccountKind::Unknown("offline".into()),
				"Офлайн",
				icon("user", 16.0).into_element(),
			))
			.child(SelectOption::new_custom_icon(
				AccountKind::Microsoft { xbox_uid: None },
				"Microsoft",
				microsoft_icon(theme.fg).into_element(),
			))
			.child(SelectOption::new(
				AccountKind::Demo,
				"Demo",
				Some("controller"),
			))
			.children(account_types.iter().filter(|x| x.id != "offline").map(|x| {
				SelectOption::new(AccountKind::Unknown(x.id.clone()), &x.name, Some("star"))
			}));

		let edit_controls = rect()
			.width(Size::fill())
			.cont()
			.child(
				segment(
					TextInput::new(editing_id).placeholder("Введите никнейм"),
					1.0,
				)
				.tip(&front_state, "Никнейм игрока"),
			)
			.child(segment(ty_dropdown, 1.0));

		let details = rect()
			.width(Size::flex(1.0))
			.height(Size::fill())
			.main_align(Alignment::Center)
			.maybe(!is_editing, |this| this.child(name))
			.maybe(is_editing, |this| this.child(edit_controls))
			.maybe(is_authenticated || account.get_name().is_some(), |this| {
				this.child(label().text(account.get_id().to_string()).color(theme.fg3))
			})
			.maybe(!is_authenticated && !is_editing, |this| {
				this.child(label().text("Не авторизован").color(theme.fg3))
			});

		let id = account.get_id().clone();
		let action: EventHandler<_> = if is_editing {
			let on_edit_submit = self.on_edit_submit.clone().unwrap();
			let editing_id = editing_id;
			let editing_ty = editing_ty;
			let front_state = front_state.clone();
			(move |_: Event<PressEventData>| {
				if editing_id.read().is_empty() {
					front_state
						.write()
						.toast(Toast::error("Никнейм не может быть пустым", None));
					return;
				};
				let new_account =
					Account::new(editing_ty.read().clone(), editing_id.read().clone().into());
				on_edit_submit.call(Some(new_account));
			})
			.into()
		} else if is_authenticated {
			(move |_: Event<PressEventData>| {
				logout_mutation.mutate(id.to_string());
			})
			.into()
		} else {
			(move |_: Event<PressEventData>| {
				login_mutation.mutate(id.to_string());
			})
			.into()
		};

		let (ico, text) = if is_editing {
			("check", "Сохранить")
		} else if is_authenticated {
			("logout", "Выйти")
		} else {
			("login", "Войти")
		};

		let action_button = icon_text_button(ico, text, &theme).on_press(action);

		let id = account.get_id().clone();
		let uuid = account.get_uuid().map(|x| x.to_string());
		let more_dropdown = Dropdown::new(
			Selected::Single(MoreOption::More),
			Rc::new(move |selected| match selected.single() {
				MoreOption::More => {}
				MoreOption::Delete => {
					delete_mutation.mutate(id.to_string());
				}
				MoreOption::CopyUUID => {
					if let Some(uuid) = &uuid {
						let _ = Clipboard::set(uuid.clone());
					}
				}
			}),
		)
		.custom_header(SelectOption::simple_icon(MoreOption::More, "ellipsis"))
		.header_width(Size::auto())
		.options_width(128.0)
		.align_options_right()
		.hide_arrow()
		.child(SelectOption::new(
			MoreOption::Delete,
			"Удалить",
			Some("trash"),
		))
		.maybe_child(account.get_uuid().is_some(), || {
			SelectOption::new(MoreOption::CopyUUID, "Скопировать UUID", Some("copy"))
		});

		let on_edit_submit = self.on_edit_submit.clone();
		let controls = rect()
			.height(Size::fill())
			.cont()
			.main_align(Alignment::End)
			.cross_align(Alignment::Center)
			.padding(Gaps::new(0.0, theme.gap2, 0.0, 0.0))
			.child(action_button)
			.maybe(!is_editing, |this| this.child(more_dropdown))
			.maybe(is_editing, |this| {
				this.child(icon_button("delete", &theme).on_press(move |_| {
					on_edit_submit.as_ref().unwrap().call(None);
				}))
			});

		rect()
			.width(Size::fill())
			.height(Size::px(64.0))
			.panel_colorway(&theme, false, false)
			.corner_radius(theme.round)
			.cont()
			.child(image)
			.child(details)
			.child(controls)
	}
}

#[derive(Clone, PartialEq)]
enum MoreOption {
	More,
	Delete,
	CopyUUID,
}
