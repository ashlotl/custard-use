use thiserror::Error;

use crate::identify::task_name::FullTaskName;
#[derive(Debug, Error)]
#[error("None of the cycles this node belongs to contain an entrypoint.")]
pub struct CustardUnreachableTaskError {
	pub offending_task: FullTaskName,
}
