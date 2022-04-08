use crate::identify::{crate_name::CrateName, custard_name::CustardName};

use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
pub struct FullTaskName {
	pub crate_name: CrateName,
	pub task_name: TaskName,
}

impl FullTaskName {
	pub fn new(crate_name: String, task_name: String) -> Self {
		Self {
			crate_name: CrateName::new(crate_name),
			task_name: TaskName::new(task_name),
		}
	}
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
pub struct TaskName {
	name: String,
}

impl CustardName<'_> for TaskName {
	fn new(val: String) -> Self {
		Self { name: val }
	}

	fn get(&self) -> &str {
		self.name.as_str()
	}
}
