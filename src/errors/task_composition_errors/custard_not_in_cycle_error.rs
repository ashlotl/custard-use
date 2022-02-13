use thiserror::Error;

use crate::identify::task_name::FullTaskName;
#[derive(Debug, Error)]
#[error("Node must belong to a cycle.")]
pub struct CustardNotInCycleError {
	pub offending_task: FullTaskName,
}
