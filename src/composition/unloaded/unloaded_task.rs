use serde::Deserialize;

use crate::{concurrency::access::Access, identify::task_name::FullTaskName};

#[derive(Debug, Deserialize, PartialEq)]
pub struct UnloadedTask {
	pub type_name: String,
	pub deserialize_path: String,

	pub parents: Vec<FullTaskName>,
	pub accesses: Vec<Access>,
	pub entrypoint: bool,
}
