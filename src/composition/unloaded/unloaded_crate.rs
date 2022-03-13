use crate::{
	composition::unloaded::{unloaded_datachunk::UnloadedDatachunk, unloaded_task::UnloadedTask},
	dylib_management::safe_library::core_library::CoreLibrary,
	identify::{datachunk_name::DatachunkName, task_name::TaskName},
};

use serde::Deserialize;

use std::{collections::BTreeMap, rc::Rc};

#[derive(Debug, Deserialize)]
pub struct UnloadedCrate {
	pub(crate) datachunks: BTreeMap<DatachunkName, UnloadedDatachunk>,
	pub(crate) tasks: BTreeMap<TaskName, UnloadedTask>,
	#[serde(skip)]
	#[serde(default)]
	pub(crate) lib: Option<Rc<CoreLibrary<'static>>>,
}

impl PartialEq for UnloadedCrate {
	fn eq(&self, other: &Self) -> bool {
		self.datachunks == other.datachunks && self.tasks == other.tasks
	}
}
