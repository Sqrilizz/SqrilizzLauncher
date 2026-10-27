use std::{fmt::Display, sync::Arc, time::Duration};

use sqrilizz::shared::output::{Message, MessageContents, MessageLevel};
use tokio::{
	sync::{Mutex, broadcast, mpsc},
	task::JoinHandle,
};

use crate::{simple_mutation, state::BackEvent};

/// Manager for long-running tasks
pub struct TaskManager {
	tasks: Vec<RunningTask>,
	event_tx: broadcast::Sender<BackEvent>,
	logger_tx: mpsc::Sender<Message>,
}

impl TaskManager {
	pub fn new(event_tx: broadcast::Sender<BackEvent>, logger_tx: mpsc::Sender<Message>) -> Self {
		Self {
			tasks: Vec::new(),
			event_tx,
			logger_tx,
		}
	}

	/// Gets the async task to update running tasks
	pub async fn get_run_task(this: Arc<Mutex<Self>>) -> ! {
		loop {
			let mut lock = this.lock().await;

			lock.update_tasks().await;

			std::mem::drop(lock);
			tokio::time::sleep(Duration::from_millis(15)).await;
		}
	}

	/// Registers a task with the task manager
	pub fn register_task(&mut self, task_id: Task, join_handle: JoinHandle<anyhow::Result<()>>) {
		self.tasks.push(RunningTask {
			id: task_id,
			join_handle: Some(join_handle),
		})
	}

	/// Updates running tasks
	pub async fn update_tasks(&mut self) {
		for task in &mut self.tasks {
			if let Some(join_handle) = task.join_handle.take() {
				if join_handle.is_finished() {
					let result = join_handle.await;
					if let Ok(Err(error)) = result {
						let _ = self.logger_tx.try_send(Message {
							contents: MessageContents::Error(format!("{error:?}")),
							level: MessageLevel::Important,
						});
						if task.id.is_long_running() {
							let _ = self.event_tx.send(BackEvent::ErrorToast(
								task.id.failure_message(),
								Some(format!("{error:?}")),
							));
						}
						let _ = self.event_tx.send(BackEvent::OutputEndTask {
							task: task.id.clone(),
							success: false,
						});
					} else if task.id.is_long_running() {
						let _ = self
							.event_tx
							.send(BackEvent::SuccessToast(task.id.success_message()));
						let _ = self.event_tx.send(BackEvent::OutputEndTask {
							task: task.id.clone(),
							success: true,
						});
					}
				} else {
					task.join_handle = Some(join_handle);
				}
			}
		}

		self.tasks.retain(|x| x.join_handle.is_some());
	}

	/// Checks if a task is currently running
	#[allow(dead_code)]
	pub fn is_running(&self, task_id: &Task) -> bool {
		self.tasks.iter().any(|x| x.id == *task_id)
	}

	/// Checks if an update task is currently running for an instance
	pub fn is_instance_updating(&self, instance_id: &str) -> bool {
		self.tasks.iter().any(|task| match &task.id {
			Task::UpdateInstance(id)
			| Task::UpdateInstancePackages(id)
			| Task::UpdateInstanceModpack(id) => id == instance_id,
			_ => false,
		})
	}

	/// Kills a task
	pub fn kill(&mut self, task_id: &Task) {
		self.tasks.retain(|task| {
			if task.id == *task_id {
				if let Some(join_handle) = &task.join_handle {
					join_handle.abort();
				}
				let _ = self.event_tx.send(BackEvent::OutputEndTask {
					task: task.id.clone(),
					success: false,
				});

				println!("Task {task_id:?} cancelled");

				false
			} else {
				true
			}
		});
	}
}

simple_mutation!(
	name = KillTask,
	ok = (),
	err = anyhow::Error,
	keys = Task,
	fn run(&self, keys: &Self::Keys) -> impl Future<Output = Result<Self::Ok, Self::Err>> {
		let back_state = self.back_state.clone();
		let task = keys.clone();

		async move {
			back_state.kill_task(&task).await;
			Ok(())
		}
	}
);

/// A single running task
#[derive(Debug)]
struct RunningTask {
	id: Task,
	join_handle: Option<JoinHandle<anyhow::Result<()>>>,
}

/// Different types of long-running tasks across the app
#[derive(PartialEq, Clone, Debug, Hash, Eq)]
pub enum Task {
	LaunchInstance(String),
	UpdateInstance(String),
	UpdateInstancePackages(String),
	UpdateInstanceModpack(String),
	DeleteInstance,
	InstallPackage,
	SearchPackages,
	#[allow(dead_code)]
	LoginAccount,
	LoginFirstAccount,
	Opening,
}

impl Task {
	pub fn can_cancel(&self) -> bool {
		match self {
			Self::LaunchInstance(_) => true,
			Self::UpdateInstance(_) => true,
			Self::UpdateInstancePackages(_) => true,
			Self::UpdateInstanceModpack(_) => true,
			Self::DeleteInstance => false,
			Self::InstallPackage => false,
			Self::SearchPackages => false,
			Self::LoginAccount => true,
			Self::LoginFirstAccount => true,
			Self::Opening => false,
			}
	}

	pub fn is_long_running(&self) -> bool {
		match self {
			Self::LaunchInstance(_) => true,
			Self::UpdateInstance(_) => true,
			Self::UpdateInstancePackages(_) => true,
			Self::UpdateInstanceModpack(_) => true,
			Self::DeleteInstance => false,
			Self::InstallPackage => true,
			Self::SearchPackages => false,
			Self::LoginAccount => true,
			Self::LoginFirstAccount => true,
			Self::Opening => false,
			}
	}

	pub fn success_message(&self) -> String {
		match self {
			Self::LaunchInstance(..) => "Launched!".to_string(),
			Self::UpdateInstance(..) => "Instance updated".to_string(),
			Self::UpdateInstancePackages(..) => "Packages updated".to_string(),
			Self::UpdateInstanceModpack(..) => "Modpack updated".to_string(),
			Self::DeleteInstance => "Instance deleted".into(),
			Self::InstallPackage => "Package installed".into(),
			Self::SearchPackages => "Packages searched".into(),
							Self::LoginAccount => "Logged in".into(),
			Self::LoginFirstAccount => "Logged in".into(),
			Self::Opening => "Opened".into(),
			}
	}

	pub fn failure_message(&self) -> String {
		match self {
			Self::LaunchInstance(id) => format!("Failed to launch instance {id}"),
			Self::UpdateInstance(id) => format!("Failed to update instance {id}"),
			Self::UpdateInstancePackages(id) => format!("Failed to update packages for {id}"),
			Self::UpdateInstanceModpack(id) => format!("Failed to update modpack for {id}"),
			Self::DeleteInstance => "Failed to delete instance".into(),
			Self::InstallPackage => "Failed to install package".into(),
			Self::SearchPackages => "Failed to search packages".into(),
							Self::LoginAccount => "Failed to log in".into(),
			Self::LoginFirstAccount => "Failed to log in".into(),
			Self::Opening => "Failed to open".into(),
			}
	}
}

impl Display for Task {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::LaunchInstance(id) => write!(f, "Launching instance {id}"),
			Self::UpdateInstance(id) => write!(f, "Updating instance {id}"),
			Self::UpdateInstancePackages(id) => write!(f, "Updating packages for {id}"),
			Self::UpdateInstanceModpack(id) => write!(f, "Updating modpack for {id}"),
			Self::DeleteInstance => write!(f, "Deleting instance"),
			Self::InstallPackage => write!(f, "Installing package"),
			Self::SearchPackages => write!(f, "Searching packages"),
							Self::LoginAccount => write!(f, "Logging in"),
			Self::LoginFirstAccount => write!(f, "Logging in"),
			Self::Opening => write!(f, "Opening"),
			}
	}
}
