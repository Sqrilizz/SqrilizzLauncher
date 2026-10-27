use std::{sync::Arc, time::Duration};

use crate::{
	components::console::{Console, ConsoleImpl},
	ops::{
		instance::{FetchInstanceLogs, FetchInstanceOutput},
		launch::WriteInstanceInput,
	},
	prelude::*,
	util::PtrEq,
};

#[derive(PartialEq)]
pub struct InstanceConsole {
	pub id: String,
	/// Whether this console belongs to a server, enabling command input
	pub is_server: bool,
}

impl Component for InstanceConsole {
	fn render(&self) -> impl IntoElement {
		let back_state = use_consume::<BackState>();
		let selected_log = use_state::<Option<String>>(|| None);
		let contents_query = use_query(FetchInstanceOutput::new(
			self.id.clone(),
			selected_log.read().clone(),
			back_state.clone(),
		));
		let logs = use_query(Query::new(
			self.id.clone(),
			FetchInstanceLogs::new(back_state.clone()),
		));
		let write_input = use_mutation(Mutation::new(WriteInstanceInput::new(back_state.clone())));

		let back_state2 = back_state.clone();
		let id2 = self.id.clone();
		let contents_query2 = contents_query;
		use_future(move || {
			let back_state2 = back_state2.clone();
			let id2 = id2.clone();
			let contents_query2 = contents_query2;
			async move {
				let mut last_modified = None;
				loop {
					tokio::time::sleep(Duration::from_millis(150)).await;

					let mut modified = None;
					if let Some(entry) = back_state2.running_instances.get_entry(&id2, None).await {
						if let Some(stdout_file) = &entry.stdout_file {
							let path = back_state2.paths.internal.join("stdio").join(stdout_file);
							if let Ok(metadata) = tokio::fs::metadata(&path).await {
								modified = metadata.modified().ok();
							}
						}
					}

					let latest_log = back_state2.paths.data.join("instances").join(&id2).join("logs").join("latest.log");
					if let Ok(metadata) = tokio::fs::metadata(&latest_log).await {
						let log_mod = metadata.modified().ok();
						if log_mod > modified {
							modified = log_mod;
						}
					}

					if last_modified != modified {
						last_modified = modified;
						contents_query2.invalidate();
					}
				}
			}
		});

		let contents = contents_query
			.read()
			.state()
			.ok()
			.cloned()
			.unwrap_or_default()
			.map(PtrEq);

		let logs = logs.read().state().ok().cloned().unwrap_or_default();

		let id = self.id.clone();
let console = Impl {
	id: self.id.clone(),
	is_server: self.is_server,
	contents,
			log_files: PtrEq(logs),
			selected_log,
			is_loading: !contents_query.read().state().is_ok(),
			write_input: (move |command| {
				write_input.mutate((id.clone(), command));
			})
			.into(),
		};

		Console { console }
	}
}

#[derive(PartialEq, Clone)]
struct Impl {
	id: String,
	is_server: bool,
	contents: Option<PtrEq<str>>,
	log_files: PtrEq<[String]>,
	selected_log: State<Option<String>>,
	is_loading: bool,
	write_input: EventHandler<String>,
}

impl ConsoleImpl for Impl {
	fn contents(&self) -> Option<Arc<str>> {
		self.contents.as_ref().map(|x| x.0.clone())
	}

	fn is_loading(&self) -> bool {
		self.is_loading
	}

	fn get_log_files(&self) -> impl Iterator<Item = &String> {
		self.log_files.0.iter()
	}

	fn get_log_file(&self) -> Option<String> {
		self.selected_log.read().clone()
	}

	fn set_log_file(&self, file: Option<String>) {
		self.selected_log.clone().set(file);
	}

	fn input_fn(&self) -> Option<impl Fn(String) -> bool + 'static> {
		if !self.is_server {
			return None::<Box<dyn Fn(String) -> bool>>;
		}

		let write_input = self.write_input.clone();
		Some(Box::new(move |mut text: String| {
			text.push('\n');
			write_input.call(text);
			true
		}))
	}
}
