use std::{
	cell::RefCell,
	collections::{BTreeMap, BTreeSet},
	error::Error,
	rc::Rc,
	sync::{atomic::AtomicUsize, Arc, Barrier, Mutex, Weak},
};

use threadpool::ThreadPool;

use crate::{
	composition::{
		loaded::{loaded_crate::LoadedCrate, loaded_datachunk::LoadedDatachunk, loaded_task::LoadedTask},
		unloaded::{unloaded_composition::UnloadedComposition, unloaded_task::UnloadedTask},
	},
	concurrency::{fulfiller::Fulfiller, fulfiller_chain::FulfillerChain},
	dylib_management::safe_library::safe_library::{DebugMode, LibraryRecompile},
	errors::{
		datachunk_errors::custard_datachunk_access_error::CustardDatachunkAccessError,
		task_composition_errors::{custard_not_in_cycle_error::CustardNotInCycleError, custard_unreachable_task_error::CustardUnreachableTaskError},
	},
	identify::{
		crate_name::CrateName,
		datachunk_name::DatachunkName,
		task_name::{FullTaskName, TaskName},
	},
	instance_control_flow::InstanceControlFlow,
};

pub struct Checked {
	#[allow(unused)]
	a: (), //make this impossible to instantiate outside of crate
}

#[derive(Debug)]
pub struct LoadedComposition {
	pub(crate) crates: BTreeMap<CrateName, LoadedCrate>,
	pub(crate) fulfiller_chains: Arc<Vec<Arc<FulfillerChain>>>,
	pub(crate) task_completion: (Arc<AtomicUsize>, Arc<Barrier>),
	pub(crate) control_flow: Arc<Mutex<InstanceControlFlow>>,
}

impl LoadedComposition {
	pub fn new(barrier: Arc<Barrier>, composition: &UnloadedComposition, recompile: LibraryRecompile, debug: DebugMode, drop_list: Rc<RefCell<Vec<libloading::Library>>>, _checked: Checked) -> Result<Self, Box<dyn Error>> {
		Self::new_with_baggage(barrier, composition, recompile, debug, drop_list, _checked, BTreeMap::new())
	}

	pub(crate) fn new_with_baggage(barrier: Arc<Barrier>, composition: &UnloadedComposition, recompile: LibraryRecompile, debug: DebugMode, drop_list: Rc<RefCell<Vec<libloading::Library>>>, _checked: Checked, mut old_crates: BTreeMap<CrateName, (BTreeMap<TaskName, LoadedTask>, BTreeMap<DatachunkName, LoadedDatachunk>)>) -> Result<Self, Box<dyn Error>> {
		let mut task_count = 0;
		for (_, unloaded_crate_contents) in &composition.crates {
			for (_, _) in &unloaded_crate_contents.tasks {
				task_count += 1;
			}
		}
		let mut ret = Self {
			crates: BTreeMap::new(),
			fulfiller_chains: Arc::new(vec![]),
			task_completion: (Arc::new(AtomicUsize::new(task_count)), barrier),
			control_flow: Arc::new(Mutex::new(InstanceControlFlow::Continue)),
		};
		for (crate_name, unloaded_crate_contents) in &composition.crates {
			let old_crate = old_crates.get_mut(crate_name);
			println!("old crate: {:#?}", old_crate);
			let loaded_crate_contents = LoadedCrate::new(crate_name, unloaded_crate_contents, recompile.clone(), debug.clone(), drop_list.clone(), old_crate)?;
			ret.crates.insert(crate_name.clone(), loaded_crate_contents);
		}

		ret.connect_fulfillers(composition)?;
		ret.create_fulfiller_chains(composition)?;
		ret.attach_fulfiller_chains()?;

		Ok(ret)
	}

	pub fn attach_fulfiller_chains(&mut self) -> Result<(), Box<dyn Error>> {
		for chain in &*self.fulfiller_chains {
			let first_node = self.crates.get(&chain.first_name.crate_name).unwrap().tasks.get(&chain.first_name.task_name).unwrap();
			for prerequisite in &first_node.prerequisites {
				let unsafe_borrow = unsafe { &mut *(&prerequisite.upgrade().unwrap().children_chains as *const _ as *mut Vec<Weak<FulfillerChain>>) };
				unsafe_borrow.push(Arc::downgrade(chain));
			}
		}
		Ok(())
	}

	fn ancestor_check(composition: &UnloadedComposition) -> Result<(), Box<dyn Error>> {
		for (crate_name, crate_contents) in &composition.crates {
			for (task_name, _task_contents) in &crate_contents.tasks {
				let mut found = false;
				let mut entrypoint_exists = false;

				let mut traversal_list = vec![];
				let start_node = FullTaskName { crate_name: crate_name.clone(), task_name: task_name.clone() };
				composition.traverse_until(
					&start_node,
					&mut traversal_list,
					Rc::new(RefCell::new(|_current, contents: &UnloadedTask| -> bool {
						if contents.entrypoint {
							entrypoint_exists = true;
						}
						if contents.parents.contains(&start_node) {
							found = true;
							return true;
						}
						return false;
					})),
				);

				if !found {
					return Err(Box::new(CustardNotInCycleError { offending_task: start_node.clone() }));
				}

				if !entrypoint_exists {
					return Err(Box::new(CustardUnreachableTaskError { offending_task: start_node.clone() }));
				}
			}
		}

		Ok(())
	}

	fn connect_fulfillers(&mut self, composition: &UnloadedComposition) -> Result<(), Box<dyn Error>> {
		for (crate_name, unloaded_crate) in &composition.crates {
			let loaded_crate = match self.crates.get(crate_name) {
				Some(v) => v,
				None => unreachable!(),
			};
			for (task_name, unloaded_task) in &unloaded_crate.tasks {
				#[allow(unused_mut)]
				let mut loaded_task = match loaded_crate.tasks.get(task_name) {
					Some(v) => v,
					None => unreachable!(),
				};
				let loaded_parents = unloaded_task
					.parents
					.iter()
					.map(|parent_name| {
						let loaded_parent_crate = match self.crates.get(&parent_name.crate_name) {
							Some(v) => v,
							None => unreachable!(),
						};
						let loaded_parent_task = match loaded_parent_crate.tasks.get(&parent_name.task_name) {
							Some(v) => v,
							None => unreachable!(),
						};
						Arc::downgrade(loaded_parent_task)
					})
					.collect();

				unsafe {
					*(&loaded_task.prerequisites as *const _ as *mut Vec<Weak<Fulfiller>>) = loaded_parents;
				}
			}
		}
		Ok(())
	}

	fn create_fulfiller_chains(&mut self, composition: &UnloadedComposition) -> Result<(), Box<dyn Error>> {
		//TODO: tests
		let mut chains = vec![];
		let mut traversed = BTreeSet::new();

		loop {
			let mut chain = vec![];
			let mut chain_names = vec![];
			let mut last_node = match composition.get_best_last_node_for_fulfiller_chain(&traversed) {
				Some(v) => v,
				None => break,
			};

			loop {
				if traversed.contains(&last_node) {
					break;
				}
				traversed.insert(last_node.clone());
				let last_node_contents = composition.crates.get(&last_node.crate_name).unwrap().tasks.get(&last_node.task_name).unwrap();
				chain.push(Arc::downgrade(self.crates.get(&last_node.crate_name).unwrap().tasks.get(&last_node.task_name).unwrap()));
				chain_names.push(last_node);

				if last_node_contents.parents.len() > 1 {
					break;
				}

				last_node = last_node_contents.parents[0].clone();
			}

			chain.reverse();
			let fulfiller_chain = FulfillerChain { first_name: chain_names.last().unwrap().clone(), chain };
			chains.push(Arc::new(fulfiller_chain));
		}

		self.fulfiller_chains = Arc::new(chains);

		Ok(())
	}

	fn cross_access_check(composition: &UnloadedComposition) -> Result<(), Box<dyn Error>> {
		for (crate_name, crate_contents) in &composition.crates {
			for (task_name, task_contents) in &crate_contents.tasks {
				for (other_crate_name, other_crate_contents) in &composition.crates {
					if crate_name == other_crate_name {
						continue;
					}
					for (other_task_name, other_task_contents) in &other_crate_contents.tasks {
						if task_name == other_task_name {
							continue;
						}
						if !composition.are_unsynchronized(FullTaskName { crate_name: crate_name.clone(), task_name: task_name.clone() }, FullTaskName { crate_name: other_crate_name.clone(), task_name: other_task_name.clone() }) {
							continue;
						}
						for access in &task_contents.accesses {
							for other_access in &other_task_contents.accesses {
								if access.of == other_access.of {
									if !access.mut_immut.commensurable(&other_access.mut_immut) {
										return Err(Box::new(CustardDatachunkAccessError {
											task_a: FullTaskName { crate_name: crate_name.clone(), task_name: task_name.clone() },
											task_b: FullTaskName { crate_name: other_crate_name.clone(), task_name: other_task_name.clone() },
											datachunk: access.of.clone(),
										}));
									}
								}
							}
						}
					}
				}
			}
		}
		Ok(())
	}

	pub fn check(unchecked: &UnloadedComposition) -> Result<Checked, Arc<dyn Error>> {
		Self::cross_access_check(unchecked)?;
		Self::ancestor_check(unchecked)?;
		Ok(Checked { a: () })
	}

	pub fn run(&self) -> InstanceControlFlow {
		let pool = ThreadPool::new(8); //TODO: make thread count and maybe other attributes configurable

		for chain in &*self.fulfiller_chains {
			chain.clone().attempt_to_run(self.task_completion.clone(), pool.clone(), self.fulfiller_chains.clone(), self.control_flow.clone());
		}

		println!("2");

		self.task_completion.1.wait();
		println!("tasks completed");

		let control_flow = self.control_flow.lock().unwrap().clone();

		control_flow
	}
}
