use custard_macros::display_from_debug;
use thiserror::Error;

use crate::identify::task_name::FullTaskName;
#[derive(Debug, Error)]
pub struct CustardNotInCycleError {
	pub offending_task: FullTaskName,
}

display_from_debug!(CustardNotInCycleError);
