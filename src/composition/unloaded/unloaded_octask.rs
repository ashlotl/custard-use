use serde::Deserialize;

use crate::{concurrency::access::Access, identify::task_name::FullTaskName};

#[derive(Debug, Deserialize)]
pub struct UnloadedOCTask {
	pub parents: Vec<FullTaskName>,
	pub accesses: Vec<Access>,
	pub entrypoint: bool,
}
