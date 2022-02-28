use crate::{
	composition::{loaded::loaded_datachunk::LoadedDatachunk, unloaded::unloaded_crate::UnloadedCrate},
	concurrency::fulfiller::Fulfiller,
	dylib_management::safe_library::{
		safe_library::{DebugMode, LibraryRecompile, SafeLibrary},
		user_library::UserLibrary,
	},
	errors::load_errors::custard_composition_requires_core_crate_error::CustardCompositionRequiresCoreCrateError,
	identify::{crate_name::CrateName, datachunk_name::DatachunkName, task_name::TaskName},
};

use std::{collections::BTreeMap, error::Error, sync::Arc};

#[derive(Debug)]
pub struct LoadedCrate {
	pub(crate) datachunks: BTreeMap<DatachunkName, LoadedDatachunk>,
	pub(crate) tasks: BTreeMap<TaskName, Arc<Fulfiller>>,

	library: UserLibrary,
}

impl LoadedCrate {
	pub fn new(name: &CrateName, unloaded_crate: &UnloadedCrate, recompile: LibraryRecompile, debug: DebugMode) -> Result<Self, Box<dyn Error>> {
		let core_library = match &unloaded_crate.lib {
			Some(v) => v,
			None => return Err(Box::new(CustardCompositionRequiresCoreCrateError { offending_crate: name.clone() })),
		};
		let user_library = UserLibrary::new(name.clone(), recompile, debug)?;
		let mut datachunks = BTreeMap::new();
		// let mut tasks = BTreeMap::new();
		for (datachunk_name, unloaded_datachunk) in &unloaded_crate.datachunks {
			datachunks.insert(datachunk_name, LoadedDatachunk::new(unloaded_datachunk, &user_library, &core_library)?);
		}

		println!("{:#?}", datachunks);

		unimplemented!(); //TODO: unimplemented
	}
}
