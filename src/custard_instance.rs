use crate::{
	composition::{
		loaded::loaded_composition::{Checked, LoadedComposition},
		unloaded::unloaded_composition::UnloadedComposition,
	},
	concurrency::fulfiller::Fulfiller,
	dylib_management::safe_library::safe_library::{DebugMode, LibraryRecompile},
	identify::crate_name::CrateName,
	instance_control_flow::InstanceControlFlow,
};

use std::{
	cell::RefCell,
	collections::BTreeMap,
	rc::Rc,
	sync::{atomic::Ordering, Arc, Barrier},
};

#[derive(Clone)]
pub struct CustardInstanceSettings {
	pub root_composition_string: String,
	pub recompile: LibraryRecompile,
	pub debug_mode: DebugMode,
}

pub struct CustardInstance {
	settings: CustardInstanceSettings,
	unloaded_composition: UnloadedComposition,
	loaded_composition: Option<LoadedComposition>,
	#[allow(unused)]
	drop_list: Rc<RefCell<Vec<libloading::Library>>>,
}

unsafe impl Send for CustardInstance {}
unsafe impl Sync for CustardInstance {}

impl CustardInstance {
	pub fn new(settings: CustardInstanceSettings) -> Self {
		Self::new_with_barrier(settings, Arc::new(Barrier::new(2)))
	}

	pub fn new_with_barrier(settings: CustardInstanceSettings, barrier: Arc<Barrier>) -> Self {
		let drop_list = Rc::new(RefCell::new(vec![]));
		let root_composition_unloaded = UnloadedComposition::from_string(settings.root_composition_string.clone(), settings.recompile.clone(), settings.debug_mode.clone(), drop_list.clone()).unwrap();

		println!("Full UnloadedComposition: {:#?}", root_composition_unloaded);

		let checked = LoadedComposition::check(&root_composition_unloaded).unwrap();

		//tell lib to turn an unloaded composition (see above) into a loaded composition
		let root_composition = LoadedComposition::new(barrier, &root_composition_unloaded, settings.recompile.clone(), settings.debug_mode.clone(), drop_list.clone(), checked).unwrap();

		println!("LoadedComposition: {:#?}", root_composition);

		//return
		Self { settings, drop_list, unloaded_composition: root_composition_unloaded, loaded_composition: Some(root_composition) }
	}

	pub(crate) fn full_reload(self) {
		let settings = self.settings.clone();
		let barrier = self.loaded_composition.as_ref().unwrap().task_completion.1.clone();

		std::mem::drop(self);

		let ret = Self::new_with_barrier(settings, barrier);
		ret.run();
	}

	pub(crate) fn partial_reload(mut self, new_unloaded_composition: UnloadedComposition, checked: Checked) {
		//TODO: so, so, so much testing

		let old_composition = self.loaded_composition.take().unwrap();
		let mut old_crates = BTreeMap::new();

		println!("in partial reload");
		for (crate_name, old_crate) in old_composition.crates {
			let reload = old_crate.should_reload.load(Ordering::Relaxed);
			println!("Old crate: {:?}, {}", crate_name, reload);
			if !(!reload && self.unloaded_composition.crates.get(&crate_name) == new_unloaded_composition.crates.get(&crate_name)) {
				continue;
			}
			println!("kept");
			let mut old_tasks = BTreeMap::new();

			for (task_name, old_fulfiller) in old_crate.tasks {
				let mut_fulfiller = unsafe { &mut *(Arc::as_ptr(&old_fulfiller) as *mut Fulfiller) };
				let old_task = mut_fulfiller.task.take().unwrap();
				old_tasks.insert(task_name, old_task);
			}

			let mut old_datachunks = BTreeMap::new();

			for (datachunk_name, old_datachunk) in old_crate.datachunks {
				old_datachunks.insert(datachunk_name, old_datachunk.unwrap());
			}

			old_crates.insert(crate_name, (old_tasks, old_datachunks));
		}

		self.unloaded_composition = new_unloaded_composition;

		self.loaded_composition = Some(LoadedComposition::new_with_baggage(Arc::new(Barrier::new(2)), &self.unloaded_composition, self.settings.recompile.clone(), self.settings.debug_mode.clone(), self.drop_list.clone(), checked, old_crates).unwrap());
		println!("1");
		self.run();
		println!("2");
	}

	pub fn run(self) {
		let control_flow = self.loaded_composition.as_ref().unwrap().run();
		println!("2");

		match control_flow {
			InstanceControlFlow::Continue => {
				println!("Relaxed exit");
			}
			InstanceControlFlow::FullReload => self.full_reload(),
			InstanceControlFlow::PartialReload => {
				let prospective_composition = UnloadedComposition::from_string(self.settings.root_composition_string.clone(), self.settings.recompile.clone(), self.settings.debug_mode.clone(), self.drop_list.clone()).unwrap();
				match LoadedComposition::check(&prospective_composition) {
					Ok(v) => self.partial_reload(prospective_composition, v),
					Err(e) => {
						println!("{}", e);
						*self.loaded_composition.as_ref().unwrap().control_flow.lock().unwrap() = InstanceControlFlow::Continue;
						self.run();
					}
				};
			}
			InstanceControlFlow::Stop => {}
		}
	}
}

impl Drop for CustardInstance {
	fn drop(&mut self) {
		//ensure drop order
		std::mem::drop(self.loaded_composition.take());

		//and then drop_list
	}
}
