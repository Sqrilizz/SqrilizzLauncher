use freya::prelude::spawn;

use crate::ops::{
	account::FetchAccounts,
	instance::{FetchInstanceConfig, FetchInstanceFiles, FetchItems},
	invalidate_all, invalidate_matching,
	launch::FetchRunningInstances,
	packages::{FetchInstanceAddons, FetchInstanceLockfile},
};

/// Backend dependency that can be invalidated
#[derive(Clone)]
pub enum BackDependency {
	Items,
	#[allow(dead_code)]
	RunningInstances,
	Accounts,
	InstanceContent(String),
}

impl BackDependency {
	/// Invalidates this dependency across the app
	pub fn invalidate(&self) {
		match self {
			Self::Items => {
				spawn(invalidate_all::<FetchItems>());
			}
			Self::RunningInstances => {
				spawn(invalidate_all::<FetchRunningInstances>());
			}
			Self::Accounts => {
				spawn(invalidate_all::<FetchAccounts>());
			}
			Self::InstanceContent(id) => {
				let id1 = id.clone();
				let id2 = id.clone();
				let id3 = id.clone();
				let id4 = id.clone();
				spawn(async move {
					invalidate_matching::<FetchInstanceFiles>(id1).await;
					invalidate_matching::<FetchInstanceConfig>(id2).await;
					invalidate_matching::<FetchInstanceAddons>(id3).await;
					invalidate_matching::<FetchInstanceLockfile>(id4).await;
				});
			}
		}
	}
}
