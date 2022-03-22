use crate::{
	composition::{
		loaded::{loaded_datachunk::LoadedDatachunk, loaded_task::LoadedTask},
		unloaded::unloaded_crate::UnloadedCrate,
	},
	concurrency::{fulfiller::Fulfiller, possibly_poisoned_mutex::PossiblyPoisonedMutex, ready::Ready},
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
	sync::{Arc, Mutex},
};

#[derive(Debug)]
pub struct LoadedCrate {
	pub(crate) datachunks: BTreeMap<DatachunkName, Option<LoadedDatachunk>>,
	pub(crate) tasks: BTreeMap<TaskName, Arc<Fulfiller>>,
}

impl LoadedCrate {
	pub fn new(name: &CrateName, unloaded_crate: &UnloadedCrate, recompile: LibraryRecompile, debug: DebugMode, drop_list: Rc<RefCell<Vec<libloading::Library>>>, mut old_crate: Option<&mut (BTreeMap<TaskName, LoadedTask>, BTreeMap<DatachunkName, LoadedDatachunk>)>) -> Result<Self, Box<dyn Error>> {
		let core_library = match &unloaded_crate.lib {
			Some(v) => v,
			None => return Err(Box::new(CustardCompositionRequiresCoreCrateError { offending_crate: name.clone() })),
		};
		let user_library = UserLibrary::new(name.clone(), recompile, debug, drop_list)?;
		let mut datachunks = BTreeMap::new();

		for (datachunk_name, unloaded_datachunk) in &unloaded_crate.datachunks {
			datachunks.insert(datachunk_name.clone(), {
				let mut datachunk = None;
				if let Some(old_crate) = &mut old_crate {
					if let Some(old_datachunk) = old_crate.1.remove(datachunk_name) {
						datachunk = Some(old_datachunk);
					}
				}
				match datachunk {
					Some(v) => Some(v),
					None => Some(LoadedDatachunk::new(unloaded_datachunk, &user_library, &core_library)?),
				}
			});
		}

		println!("{:#?}", datachunks);

		let mut fulfillers = BTreeMap::new();

		for (task_name, unloaded_task) in &unloaded_crate.tasks {
			let full_name = FullTaskName { crate_name: name.clone(), task_name: task_name.clone() };
			let fulfiller = Fulfiller {
				cease: Mutex::new(false),
				error: Mutex::new(false),
				children_chains: vec![],
				done: Ready::new(unloaded_task.entrypoint),
				prerequisites: vec![],
				task: {
					let mut task = None;
					if let Some(old_crate) = &mut old_crate {
						if let Some(old_task) = old_crate.0.remove(task_name) {
							task = Some(old_task);
						}
					}
					match task {
						Some(v) => Some(v),
						None => Some(LoadedTask::new(full_name, unloaded_task, &user_library, &core_library)?),
					}
				},
			};
			fulfillers.insert(task_name.clone(), Arc::new(fulfiller));
		}

		Ok(Self { datachunks, tasks: fulfillers })
	}
}
