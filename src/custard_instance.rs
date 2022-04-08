use log::{debug, error, info};

use crate::{
	composition::{
		loaded::{
			loaded_composition::{Checked, LoadedComposition},
			loaded_datachunk::LoadedDatachunk,
		},
		unloaded::unloaded_composition::UnloadedComposition,
	},
	concurrency::{
		fulfiller::{Fulfiller, Quit},
		possibly_poisoned_mutex::PossiblyPoisonedMutex,
	},
	dylib_management::safe_library::safe_library::{
		DebugMode, LibraryRecompile,
	},
	identify::{crate_name::CrateName, datachunk_name::DatachunkName},
	instance_control_flow::InstanceControlFlow,
};

use std::{
	cell::RefCell,
	collections::{BTreeMap, BTreeSet},
	rc::Rc,
	sync::{Arc, Mutex},
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
	/// Create a new `CustardInstance`. Use this method from an entrypoint crate.
	pub fn new(settings: CustardInstanceSettings) -> Self {
		Self::new_with_quit(settings, None)
	}

	/// Create a new `CustardInstance`, but manually specify the barrier. By default, the barrier is set to 2, so the entrypoint and the CustardInstance both have one reference via which they can respectively wait and release the program flow.
	pub fn new_with_quit(
		settings: CustardInstanceSettings,
		quit: Option<Arc<Quit>>,
	) -> Self {
		//Create a place where library memory can be cached until a full reload. This is slightly hazardous to deal with, as dropping it too early could segfault, and dropping it too late means it won't be cleaned up even in the case of a full reload.
		let drop_list = Rc::new(RefCell::new(vec![]));

		let root_composition_unloaded = unsafe {
			UnloadedComposition::from_string(
				settings.root_composition_string.clone(),
				settings.recompile.clone(),
				settings.debug_mode.clone(),
				drop_list.clone(),
			)
			.unwrap()
		};

		debug!("Full UnloadedComposition: {:#?}", root_composition_unloaded);

		let checked =
			LoadedComposition::check(&root_composition_unloaded).unwrap();

		let root_composition = LoadedComposition::new(
			quit,
			&root_composition_unloaded,
			settings.recompile.clone(),
			settings.debug_mode.clone(),
			drop_list.clone(),
			checked,
		)
		.unwrap();

		debug!("LoadedComposition: {:#?}", root_composition);

		//return
		Self {
			settings,
			drop_list,
			unloaded_composition: root_composition_unloaded,
			loaded_composition: Some(root_composition),
		}
	}

	/// Drop self and all dynamic libraries, saving only the settings and barrier for new instance.
	pub(crate) fn full_reload(self) {
		let settings = self.settings.clone();
		let quit = self
			.loaded_composition
			.as_ref()
			.unwrap()
			.task_completion
			.clone();

		std::mem::drop(self);

		let ret = Self::new_with_quit(settings, Some(quit));
		ret.run();
	}

	/// Replace loaded composition with a fresh one, but do not drop libraries. Because not all crates are reloaded, the `drop_list` is kept. Note that `reload_for_sure` is not the be all and end all of reloading. If a crate's old composition does not align with its new composition, it will be reloaded as well.
	pub(crate) fn partial_reload(
		mut self,
		new_unloaded_composition: UnloadedComposition,
		checked: Checked,
		reload_for_sure: Arc<BTreeSet<CrateName>>,
	) {
		//TODO: so, so, so much testing

		let old_composition = self.loaded_composition.take().unwrap();
		let mut old_crates = BTreeMap::new();

		for (crate_name, old_crate) in
			unsafe { old_composition.crates.get_mut() }
		{
			let crate_name = crate_name.clone();
			let reload = reload_for_sure.contains(&crate_name);
			let old_unloaded_crate =
				self.unloaded_composition.crates.get(&crate_name);
			let new_unloaded_crate =
				new_unloaded_composition.crates.get(&crate_name);
			if !(!reload && old_unloaded_crate == new_unloaded_crate) {
				let delta_tasks =
					new_unloaded_crate.as_ref().unwrap().tasks.len()
						- old_unloaded_crate.as_ref().unwrap().tasks.len();
				old_composition
					.task_completion
					.begin_fulfillers(delta_tasks as isize);
				continue;
			}
			let mut old_tasks = BTreeMap::new();

			for (task_name, old_fulfiller) in &old_crate.tasks {
				let task_name = task_name.clone();
				let mut_fulfiller = unsafe {
					&mut *(Arc::as_ptr(&old_fulfiller) as *mut Fulfiller)
				};
				let old_task = mut_fulfiller.task.take().unwrap();
				old_tasks.insert(task_name, old_task);
			}

			let mut old_datachunks = BTreeMap::new();

			for (datachunk_name, old_datachunk) in unsafe {
				&mut *(&old_crate.datachunks as *const _
					as *mut BTreeMap<DatachunkName, Option<LoadedDatachunk>>)
			} {
				let datachunk_name = datachunk_name.clone();
				old_datachunks
					.insert(datachunk_name, old_datachunk.take().unwrap());
			}

			old_crates.insert(crate_name, (old_tasks, old_datachunks));
		}

		self.unloaded_composition = new_unloaded_composition;

		self.loaded_composition = Some(
			LoadedComposition::new_with_baggage(
				Some(old_composition.task_completion),
				&self.unloaded_composition,
				self.settings.recompile.clone(),
				self.settings.debug_mode.clone(),
				self.drop_list.clone(),
				checked,
				old_crates,
			)
			.unwrap(),
		);

		info!("Partial reload: rerunning instance.");
		self.run();
	}

	/// Consume self, giving up control to the instance. Any errors past this point are unhandleable, so ensure that any reloads come only after thoroughly checking the new composition.
	pub fn run(mut self) {
		let control_flow = self.loaded_composition.as_ref().unwrap().run();
		let loaded_composition = self.loaded_composition.as_mut().unwrap();
		if unsafe { loaded_composition.task_completion.reset() } == 0 {
			return;
		}

		loaded_composition.control_flow = Arc::new(PossiblyPoisonedMutex::new(
			Mutex::new(InstanceControlFlow::Continue),
		));

		match control_flow {
			InstanceControlFlow::Continue => {
				println!("Relaxed exit");
			}
			InstanceControlFlow::FullReload => self.full_reload(),
			InstanceControlFlow::PartialReload(reload_for_sure) => {
				let prospective_composition = unsafe {
					UnloadedComposition::from_string(
						self.settings.root_composition_string.clone(),
						self.settings.recompile.clone(),
						self.settings.debug_mode.clone(),
						self.drop_list.clone(),
					)
					.unwrap()
				};
				match LoadedComposition::check(&prospective_composition) {
					Ok(v) => {
						self.partial_reload(
							prospective_composition,
							v,
							reload_for_sure,
						)
					}
					Err(e) => {
						error!("{}", e);
						*self
							.loaded_composition
							.as_ref()
							.unwrap()
							.control_flow
							.lock() = InstanceControlFlow::Continue;
						self.run();
					}
				};
			}
			InstanceControlFlow::RecreateThreadpool => {
				let comp_ref = self.loaded_composition.as_mut().unwrap();

				for chain in &*comp_ref.fulfiller_chains {
					for fulfiller in &chain.chain {
						if let Some(fulfiller) = fulfiller.upgrade() {
							if !*fulfiller.error.lock().unwrap() {
								*fulfiller.cease.lock().unwrap() = false;
							}
						}
					}
				}

				info!("Rerunning instance.");
				self.run();
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
