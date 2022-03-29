use custard_macros::display_from_debug;
use thiserror::Error;

use crate::identify::task_name::FullTaskName;
#[derive(Debug, Error)]
/// None of the cycles this node belongs to contain an entrypoint.
pub struct CustardUnreachableTaskError {
	pub offending_task: FullTaskName,
}

display_from_debug!(CustardUnreachableTaskError);
