use thiserror::Error;

use crate::identify::{datachunk_name::FullDatachunkName, task_name::FullTaskName};

#[derive(Debug, Error)]
#[error("Tasks have conflicting access of a Datachunk")]
pub struct CustardDatachunkAccessError {
	pub task_a: FullTaskName,
	pub task_b: FullTaskName,
	pub datachunk: FullDatachunkName,
}
