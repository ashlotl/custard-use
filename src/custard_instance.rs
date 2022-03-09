use crate::{
	composition::{loaded::loaded_composition::LoadedComposition, unloaded::unloaded_composition::UnloadedComposition},
	dylib_management::safe_library::safe_library::{DebugMode, LibraryRecompile},
};

use std::{
	cell::RefCell,
	error::Error,
	rc::Rc,
	sync::{Arc, Barrier},
};

#[derive(Clone)]
pub struct CustardInstanceSettings {
	pub root_composition_string: String,
	pub recompile: LibraryRecompile,
	pub debug_mode: DebugMode,
}

pub struct CustardInstance {
	settings: CustardInstanceSettings,
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

		//tell lib to turn an unloaded composition (see above) into a loaded composition
		let root_composition = LoadedComposition::check(barrier, root_composition_unloaded, settings.recompile.clone(), settings.debug_mode.clone(), drop_list.clone()).unwrap();

		println!("LoadedComposition: {:#?}", root_composition);

		//return
		Self { settings, drop_list, loaded_composition: Some(root_composition) }
	}

	pub(crate) fn reload(self) -> Result<(), Box<dyn Error>> {
		let settings = self.settings.clone();
		let barrier = self.loaded_composition.as_ref().unwrap().task_completion.1.clone();

		std::mem::drop(self);

		let ret = Self::new_with_barrier(settings, barrier);
		ret.run();

		Ok(())
	}

	pub fn run(self) {
		let reload_enqueued = self.loaded_composition.as_ref().unwrap().run();

		if reload_enqueued {
			//TODO: partial reload ("crate reload")
			self.reload().unwrap();
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
