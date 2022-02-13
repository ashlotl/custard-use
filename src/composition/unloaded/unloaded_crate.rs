use crate::{
	composition::unloaded::{unloaded_datachunk::UnloadedDatachunk, unloaded_octask::UnloadedOCTask},
	identify::{datachunk_name::DatachunkName, task_name::TaskName},
};

use serde::Deserialize;

use std::collections::BTreeMap;

#[derive(Debug, Deserialize)]
pub struct UnloadedCrate {
	pub(crate) datachunks: BTreeMap<DatachunkName, UnloadedDatachunk>,
	pub(crate) tasks: BTreeMap<TaskName, UnloadedOCTask>,
}
