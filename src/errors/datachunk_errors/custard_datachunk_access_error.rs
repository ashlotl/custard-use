use custard_macros::display_from_debug;
use thiserror::Error;

use crate::identify::{
	datachunk_name::FullDatachunkName, task_name::FullTaskName,
};

#[derive(Debug, Error)]
pub struct CustardDatachunkAccessError {
	pub task_a: FullTaskName,
	pub task_b: FullTaskName,
	pub datachunk: FullDatachunkName,
}

display_from_debug!(CustardDatachunkAccessError);
