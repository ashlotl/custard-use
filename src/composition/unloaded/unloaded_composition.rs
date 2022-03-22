use crate::{
	composition::unloaded::{unloaded_crate::UnloadedCrate, unloaded_task::UnloadedTask},
	dylib_management::safe_library::{
		core_library::CoreLibrary,
		safe_library::{DebugMode, LibraryRecompile, SafeLibrary},
	},
	errors::parse_errors::{custard_composition_cycle_error::CustardCompositionCycleError, custard_ron_parse_error::CustardRonCompositionParseError},
	identify::{crate_name::CrateName, task_name::FullTaskName},
};

use ron::{self};
use serde::Deserialize;

use std::{
	cell::RefCell,
	collections::{BTreeMap, BTreeSet},
	error::Error,
	rc::Rc,
};

/// Set this variable to allow crate dependencies to cycle. This is not reccomended for dependency management reasons.
const ENVIRONMENT_VAR_STR_ALLOW_DEPENDENCY_CYCLES: &'static str = "CUSTARD_ALLOW_DEPENDENCY_CYCLES";

/// Stores the fundamental information about a composition before user crates are dynamically loaded.
#[derive(Debug, Deserialize)]
pub struct UnloadedComposition {
	pub(crate) crates: BTreeMap<CrateName, UnloadedCrate>,
	children: Vec<CrateName>,
}

impl UnloadedComposition {
	/// Create an unloaded composition from the deserializable string. Note that this requires a drop list as an argument (creating an unloaded composition requires dynamically loading core crates), so you will have to be careful to pass in a value that is only dropped once the composition and any memory it may have allocated has been freed. As a consequence this is very unsafe:
	/// ```ignore
	/// let mut some_string: String;
	///
	/// {
	/// 	let drop_list = Rc::new(RefCell::new(vec![]));
	///
	/// 	{
	/// 		let composition = from_string(to_deserialize, recompile, debug, drop_list).unwrap();
	///
	/// 		some_string = composition.get_best_last_node_for_fulfiller_chain(traversal_set).unwrap().crate_name.get().to_owned();//some random carelessness allows some_string to now internally have a pointer to memory owned by a dynamic library
	/// 	}//composition is dropped before drop_list--this is good
	///
	/// }//drop_list is dropped, and here problems start
	///
	/// some_other_func(some_string);//almost certainly segfaults because some_string no longer points to memory that belongs to us
	/// ```
	/// To save on headaches all of this is handled by [CustardInstance](crate::custard_instance::CustardInstance), but this method and struct are left public because it may be useful at runtime to study the present composition, and check it before a reload (keep in mind, reloads--see [CustardInstance](crate::custard_instance::CustardInstance)--run the risk of panicking if not carefully checked before initiation).
	pub unsafe fn from_string(to_deserialize: String, recompile: LibraryRecompile, debug: DebugMode, drop_list: Rc<RefCell<Vec<libloading::Library>>>) -> Result<Self, Box<dyn Error>> {
		let res: Result<UnloadedComposition, ron::Error> = ron::from_str(to_deserialize.as_str());
		let mut to_return = match res {
			Ok(v) => v,
			Err(error) => return Err(Box::new(CustardRonCompositionParseError { error, relevant_ron: to_deserialize })),
		};

		let mut traversal_tree = BTreeMap::<Option<CrateName>, Vec<CrateName>>::new();

		traversal_tree.insert(None, to_return.children.clone());

		let mut should_break = false;

		while !should_break {
			for child_i in 0..to_return.children.len() {
				should_break = false;

				if traversal_tree.contains_key(&Some(to_return.children[child_i].clone())) {
					should_break = true;
					continue;
				}
				let mut child_composition = Self::from_crate(to_return.children[child_i].clone(), recompile.clone(), debug.clone(), drop_list.clone())?;

				traversal_tree.insert(Some(to_return.children[child_i].clone()), child_composition.children.clone());

				to_return.crates.append(&mut child_composition.crates);
				to_return.children.append(&mut child_composition.children); //mutable access, order relative to traversal_tree insert
			}
		}

		if match std::env::var(ENVIRONMENT_VAR_STR_ALLOW_DEPENDENCY_CYCLES) {
			Ok(v) => v != "true" && v != "1" && v != "yes",
			Err(std::env::VarError::NotPresent) => true,
			_ => false,
		} {
			//prevent a dependency cycle
			let mut traversal_tree_traversal_list = vec![];
			Self::recurse_crate_traversal_tree(&traversal_tree, &mut traversal_tree_traversal_list, None)?;
		}

		Ok(to_return)
	}

	/// Determine if two tasks are unsynchronized. This is determined by doing a search of their ancestors and seeing if they reach a common ancestor (in which case they are unsynchronized) before each other (in which case they must be synchronized).
	pub fn are_tasks_unsynchronized(&self, task_name: FullTaskName, other_task_name: FullTaskName) -> bool {
		let mut first_ab = true;
		let mut found_ab = false;

		let mut traversal_list = vec![];

		self.traverse_until(
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

		self.traverse_until(
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

	fn recurse_crate_traversal_tree(traversal_tree: &BTreeMap<Option<CrateName>, Vec<CrateName>>, traversal_tree_traversal_list: &mut Vec<Option<CrateName>>, active_node: Option<CrateName>) -> Result<(), Box<dyn Error>> {
		traversal_tree_traversal_list.push(active_node.clone());
		for subnode in traversal_tree.get(&active_node).unwrap() {
			if traversal_tree_traversal_list.contains(&Some(subnode.clone())) {
				return Err(Box::new(CustardCompositionCycleError { offending_crate: active_node.clone() }));
			}
			Self::recurse_crate_traversal_tree(traversal_tree, traversal_tree_traversal_list, Some(subnode.clone()))?;
		}
		traversal_tree_traversal_list.pop();
		Ok(())
	}

	fn from_crate(crate_name: CrateName, recompile: LibraryRecompile, debug: DebugMode, drop_list: Rc<RefCell<Vec<libloading::Library>>>) -> Result<Self, Box<dyn Error>> {
		println!("A");
		let loaded = Rc::new(CoreLibrary::new(crate_name, recompile, debug, drop_list)?);
		let composition_string = ((loaded.symbols.as_ref().unwrap().composition)()).into_rust()?;
		let res: Result<UnloadedComposition, ron::Error> = ron::from_str(composition_string.as_str());

		return match res {
			Ok(mut v) => {
				for (_, crate_contents) in &mut v.crates {
					crate_contents.lib = Some(loaded.clone());
				}
				Ok(v)
			}
			Err(error) => return Err(Box::new(CustardRonCompositionParseError { error, relevant_ron: composition_string })),
		};
	}

	pub fn get_best_last_node_for_fulfiller_chain(&self, traversed: &BTreeSet<FullTaskName>) -> Option<FullTaskName> {
		let mut candidates: Vec<(FullTaskName, usize)> = vec![];
		for (crate_name, unloaded_crate) in &self.crates {
			for (task_name, _) in &unloaded_crate.tasks {
				let full_name = FullTaskName { crate_name: crate_name.clone(), task_name: task_name.clone() };
				if !traversed.contains(&full_name) {
					let child_count = self.get_children_of(&full_name, |_, _| true).len();
					if candidates.len() == 0 {
						candidates.push((full_name, child_count));
					} else {
						for i in 0..candidates.len() {
							if candidates[i].1 <= child_count {
								candidates.insert(i, (full_name, child_count));
								break;
							}
						}
					}
				}
			}
		}
		if candidates.len() > 0 {
			return Some(candidates[0].0.clone());
		}
		None
	}

	pub fn get_children_of(&self, parent_name: &FullTaskName, rule: impl Fn(&FullTaskName, &UnloadedTask) -> bool) -> Vec<FullTaskName> {
		let mut ret = vec![];
		for (crate_name, unloaded_crate) in &self.crates {
			for (task_name, unloaded_task) in &unloaded_crate.tasks {
				let full_name = FullTaskName { crate_name: crate_name.clone(), task_name: task_name.clone() };
				if unloaded_task.parents.contains(parent_name) && rule(&full_name, unloaded_task) {
					ret.push(full_name);
				}
			}
		}
		ret
	}

	pub fn get_unloaded_task(&self, find_name: &FullTaskName) -> Option<&UnloadedTask> {
		for (crate_name, crate_contents) in &self.crates {
			if crate_name != &find_name.crate_name {
				continue;
			}
			for (task_name, task_contents) in &crate_contents.tasks {
				if task_name == &find_name.task_name {
					return Some(task_contents);
				}
			}
		}
		return None;
	}

	pub fn traverse_until<'a>(&'a self, current_node: &'a FullTaskName, traversal_list: &mut Vec<FullTaskName>, for_each: Rc<RefCell<dyn FnMut(&'a FullTaskName, &'a UnloadedTask) -> bool + 'a>>) -> bool {
		if traversal_list.contains(current_node) {
			return false;
		}

		let unloaded_task = self.get_unloaded_task(current_node).unwrap();
		if (for_each.borrow_mut())(current_node, unloaded_task) {
			return true;
		}
		traversal_list.push(current_node.clone());
		for parent in &unloaded_task.parents {
			if self.traverse_until(parent, traversal_list, for_each.clone()) {
				return true;
			}
		}
		return false;
	}
}
