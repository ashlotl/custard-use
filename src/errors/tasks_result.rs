use std::{collections::BTreeMap, error::Error, sync::Arc};

use crate::identify::task_name::FullTaskName;

pub struct TasksResult {
	pub errors: BTreeMap<FullTaskName, Arc<dyn Error + Send + Sync>>,
}
