use std::{cell::RefCell, collections::BTreeMap, error::Error, rc::Rc, sync::Barrier};

use crate::{
	composition::{
		loaded::loaded_crate::LoadedCrate,
		unloaded::{unloaded_composition::UnloadedComposition, unloaded_octask::UnloadedOCTask},
	},
	errors::{
		datachunk_errors::custard_datachunk_access_error::CustardDatachunkAccessError,
		task_composition_errors::{custard_not_in_cycle_error::CustardNotInCycleError, custard_unreachable_task_error::CustardUnreachableTaskError},
	},
	identify::{crate_name::CrateName, task_name::FullTaskName},
};

#[derive(Debug)]
pub struct LoadedComposition<'lib> {
	crates: BTreeMap<CrateName, LoadedCrate<'lib>>,
}

impl<'lib> LoadedComposition<'lib> {
	fn new(composition: &UnloadedComposition, recompile: bool, debug: bool) -> Result<Self, Box<dyn Error>> {
		let mut ret = Self { crates: BTreeMap::new() };
		for (crate_name, unloaded_crate_contents) in &composition.crates {
			let loaded_crate_contents = LoadedCrate::new(crate_name, unloaded_crate_contents, recompile, debug)?;
			ret.crates.insert(crate_name.clone(), loaded_crate_contents);
		}
		//TODO: connect fulfillers
		Ok(ret)
	}

	fn ancestor_check(composition: &UnloadedComposition) -> Result<(), Box<dyn Error>> {
		for (crate_name, crate_contents) in &composition.crates {
			for (task_name, _task_contents) in &crate_contents.tasks {
				let mut first = true;
				let mut found = false;
				let mut entrypoint_exists = false;

				let mut traversal_list = vec![];
				let start_node = FullTaskName { crate_name: crate_name.clone(), task_name: task_name.clone() };
				composition.traverse_until(
					&start_node,
					&mut traversal_list,
					Rc::new(RefCell::new(|current, contents: &UnloadedOCTask| -> bool {
						if contents.entrypoint {
							entrypoint_exists = true;
						}
						if !first && current == &start_node {
							found = true;
							return true;
						}
						first = false;
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

	fn are_unsynchronized(composition: &UnloadedComposition, task_name: FullTaskName, other_task_name: FullTaskName) -> bool {
		let mut first_ab = true;
		let mut found_ab = false;

		let mut traversal_list = vec![];

		composition.traverse_until(
			&task_name.clone(),
			&mut traversal_list,
			Rc::new(RefCell::new(|current, _contents| -> bool {
				if !first_ab && current == &other_task_name {
					found_ab = true;
					return true;
				}
				first_ab = false;
				return false;
			})),
		);

		if !found_ab {
			return true;
		}

		traversal_list.remove(0); //We want to find task_name again. We don't need to remove other_task_name from the list because it doesn't exist within it

		let mut first_ba = true;
		let mut found_ba = false;

		composition.traverse_until(
			&other_task_name.clone(),
			&mut traversal_list,
			Rc::new(RefCell::new(|current, _contents| -> bool {
				if !first_ba && current == &task_name {
					found_ba = true;
					return true;
				}
				first_ba = false;
				return false;
			})),
		);

		!(found_ab && found_ba)
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
						if !Self::are_unsynchronized(composition, FullTaskName { crate_name: crate_name.clone(), task_name: task_name.clone() }, FullTaskName { crate_name: other_crate_name.clone(), task_name: other_task_name.clone() }) {
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

	pub fn check(unchecked: UnloadedComposition, recompile: bool, debug: bool) -> Result<Self, Box<dyn Error>> {
		Self::cross_access_check(&unchecked)?;
		Self::ancestor_check(&unchecked)?;
		Ok(Self::new(&unchecked, recompile, debug)?)
	}

	pub fn run(&self) -> Barrier {
		unimplemented!() //TODO: LoadedComposition::run is not fully implemented
	}
}
