use crate::{
	composition::unloaded::{
		unloaded_crate::{UnloadedCrate},
		unloaded_octask::UnloadedOCTask,
	},
	dylib_management::safe_library::{
		core_library::CoreLibrary,
		safe_library::{DebugMode, LibraryRecompile, SafeLibrary},
	},
	errors::parse_errors::{custard_composition_cycle_error::CustardCompositionCycleError, custard_ron_parse_error::CustardRonCompositionParseError},
	identify::{crate_name::CrateName, task_name::FullTaskName},
};

use ron::{self};
use serde::Deserialize;

use std::{cell::RefCell, collections::BTreeMap, error::Error, rc::Rc};

#[derive(Debug, Deserialize)]
pub struct UnloadedComposition {
	pub(crate) crates: BTreeMap<CrateName, UnloadedCrate>,
	children: Vec<CrateName>,
}

impl UnloadedComposition {
	pub fn from_string(string: String, recompile: LibraryRecompile, debug: DebugMode) -> Result<Self, Box<dyn Error>> {
		let res: Result<UnloadedComposition, ron::Error> = ron::from_str(string.as_str());
		let mut to_return = match res {
			Ok(v) => v,
			Err(error) => return Err(Box::new(CustardRonCompositionParseError { error, relevant_ron: string })),
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
				let mut child_composition = Self::from_crate(to_return.children[child_i].clone(), recompile.clone(), debug.clone())?;

				traversal_tree.insert(Some(to_return.children[child_i].clone()), child_composition.children.clone());

				to_return.crates.append(&mut child_composition.crates);
				to_return.children.append(&mut child_composition.children); //mutable access, order relative to traversal_tree insert
			}
		}

		if match std::env::var("CUSTARD_ALLOW_DEPENDENCY_CYCLES") {
			Ok(v) => v != "true" && v != "1" && v != "yes",
			Err(std::env::VarError::NotPresent) => true,
			_ => false,
		} {
			//prevent a dependency cycle
			let mut traversal_tree_traversal_list = vec![];
			Self::recurse_traversal_tree(&traversal_tree, &mut traversal_tree_traversal_list, None)?;
		}

		Ok(to_return)
	}

	fn recurse_traversal_tree(traversal_tree: &BTreeMap<Option<CrateName>, Vec<CrateName>>, traversal_tree_traversal_list: &mut Vec<Option<CrateName>>, active_node: Option<CrateName>) -> Result<(), Box<dyn Error>> {
		traversal_tree_traversal_list.push(active_node.clone());
		for subnode in traversal_tree.get(&active_node).unwrap() {
			if traversal_tree_traversal_list.contains(&Some(subnode.clone())) {
				return Err(Box::new(CustardCompositionCycleError { offending_crate: active_node.clone() }));
			}
			Self::recurse_traversal_tree(traversal_tree, traversal_tree_traversal_list, Some(subnode.clone()))?;
		}
		traversal_tree_traversal_list.pop();
		Ok(())
	}

	fn from_crate(crate_name: CrateName, recompile: LibraryRecompile, debug: DebugMode) -> Result<Self, Box<dyn Error>> {
		let loaded = Rc::new(CoreLibrary::new(crate_name, recompile, debug)?);
		let composition_string = ((loaded.symbols.as_ref().unwrap().composition)()).into_rust()?.into_rust();
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

	pub fn get_unloaded_task(&self, find_name: &FullTaskName) -> Option<&UnloadedOCTask> {
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

	pub fn traverse_until<'a>(&'a self, current_node: &'a FullTaskName, traversal_list: &mut Vec<FullTaskName>, for_each: Rc<RefCell<dyn FnMut(&'a FullTaskName, &'a UnloadedOCTask) -> bool + 'a>>) -> bool {
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
