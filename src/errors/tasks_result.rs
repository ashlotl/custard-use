use std::{any::Any, collections::BTreeMap, sync::Arc};

use crate::identify::task_name::FullTaskName;

pub struct TasksResult {
	pub errors: BTreeMap<FullTaskName, Arc<dyn Any + Send + Sync + 'static>>,
}
