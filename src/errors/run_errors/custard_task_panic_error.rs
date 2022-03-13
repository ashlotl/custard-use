use crate::identify::task_name::FullTaskName;

use custard_macros::display_from_debug;

use thiserror::Error;

use std::any::Any;

#[derive(Debug, Error)]
pub struct CustardTaskPanicError {
	pub offending_task: FullTaskName,
	pub error: Box<dyn Any + Send>,
}
display_from_debug!(CustardTaskPanicError);
