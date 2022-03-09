use crate::{
	composition::{
		loaded::{loaded_datachunk::LoadedDatachunk, loaded_task::LoadedTask},
		unloaded::unloaded_crate::UnloadedCrate,
	},
	concurrency::{fulfiller::Fulfiller, ready::Ready},
	dylib_management::safe_library::{
		safe_library::{DebugMode, LibraryRecompile, SafeLibrary},
		user_library::UserLibrary,
	},
	errors::load_errors::custard_composition_requires_core_crate_error::CustardCompositionRequiresCoreCrateError,
	identify::{
		crate_name::CrateName,
		datachunk_name::DatachunkName,
		task_name::{FullTaskName, TaskName},
	},
};

use std::{
	cell::RefCell,
	collections::BTreeMap,
	error::Error,
	rc::Rc,
	sync::{atomic::AtomicBool, Arc},
};

#[derive(Debug)]
pub struct LoadedCrate {
	pub(crate) datachunks: BTreeMap<DatachunkName, LoadedDatachunk>,
	pub(crate) tasks: BTreeMap<TaskName, Arc<Fulfiller>>,

	library: UserLibrary,
}

impl LoadedCrate {
	pub fn new(name: &CrateName, unloaded_crate: &UnloadedCrate, recompile: LibraryRecompile, debug: DebugMode, drop_list: Rc<RefCell<Vec<libloading::Library>>>) -> Result<Self, Box<dyn Error>> {
		let core_library = match &unloaded_crate.lib {
			Some(v) => v,
			None => return Err(Box::new(CustardCompositionRequiresCoreCrateError { offending_crate: name.clone() })),
		};
		let user_library = UserLibrary::new(name.clone(), recompile, debug, drop_list)?;
		let mut datachunks = BTreeMap::new();
		// let mut tasks = BTreeMap::new();
		for (datachunk_name, unloaded_datachunk) in &unloaded_crate.datachunks {
			datachunks.insert(datachunk_name.clone(), LoadedDatachunk::new(unloaded_datachunk, &user_library, &core_library)?);
		}

		println!("{:#?}", datachunks);

		let mut fulfillers = BTreeMap::new();

		for (task_name, unloaded_task) in &unloaded_crate.tasks {
			let full_name = FullTaskName { crate_name: name.clone(), task_name: task_name.clone() };
			let fulfiller = Fulfiller {
				cease: AtomicBool::new(false),
				children_chains: vec![],
				done: Ready::new(unloaded_task.entrypoint),
				prerequisites: vec![],
				task: LoadedTask::new(full_name, unloaded_task, &user_library, &core_library)?,
			};
			fulfillers.insert(task_name.clone(), Arc::new(fulfiller));
		}

		Ok(Self { datachunks, tasks: fulfillers, library: user_library })
	}
}
