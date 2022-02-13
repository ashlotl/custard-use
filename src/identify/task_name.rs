use crate::identify::{crate_name::CrateName, custard_name::CustardName};

use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
pub struct FullTaskName {
	pub crate_name: CrateName,
	pub task_name: TaskName,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
pub struct TaskName {
	name: String,
}

impl CustardName<'_> for TaskName {
	fn get(&self) -> &str {
		self.name.as_str()
	}
}
