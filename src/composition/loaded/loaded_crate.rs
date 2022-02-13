use crate::{
	composition::{
		loaded::{loaded_composition::LoadedComposition, loaded_datachunk::LoadedDatachunk},
		unloaded::unloaded_crate::UnloadedCrate,
	},
	concurrency::fulfiller::Fulfiller,
	dylib_management::safe_library::safe_library::SafeLibrary,
	identify::{crate_name::CrateName, datachunk_name::DatachunkName, task_name::TaskName},
};

use std::{collections::BTreeMap, error::Error, sync::Arc};

#[derive(Debug)]
pub struct LoadedCrate<'lib> {
	pub(crate) datachunks: BTreeMap<DatachunkName, LoadedDatachunk>,
	pub(crate) tasks: BTreeMap<TaskName, Arc<Fulfiller>>,

	library: SafeLibrary<'lib>,
}

impl<'lib> LoadedCrate<'lib> {
	pub fn new(name: &CrateName, unloaded_crate: &UnloadedCrate) -> Result<Self, Box<dyn Error>> {
		let library = SafeLibrary::new(false, name.clone())?;
		let mut datachunks = BTreeMap::new();
		// let mut tasks = BTreeMap::new();
		for (datachunk_name, unloaded_datachunk) in &unloaded_crate.datachunks {
			datachunks.insert(datachunk_name, LoadedDatachunk::new(unloaded_datachunk, &library)?);
		}
		unimplemented!(); //TODO: unimplemented
	}
}
