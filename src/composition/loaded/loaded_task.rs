use crate::{
	composition::unloaded::unloaded_task::UnloadedTask,
	concurrency::access::Access,
	dylib_management::safe_library::{core_library::CoreLibrary, user_library::UserLibrary},
	identify::task_name::FullTaskName,
	user_types::task::{Task, TaskClosureType},
};

use std::{
	error::Error,
	fmt::{self, Formatter},
};

pub struct LoadedTask {
	pub name: FullTaskName,
	pub closure: TaskClosureType,
	pub user_data: Task,
	accesses: Vec<Access>,
}

impl fmt::Debug for LoadedTask {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		f.debug_struct("LoadedTask").field("accesses", &self.accesses).field("user_data", &self.user_data).finish_non_exhaustive()
	}
}

impl LoadedTask {
	pub fn new(name: FullTaskName, unloaded_task: &UnloadedTask, user_library: &UserLibrary, core_library: &CoreLibrary) -> Result<Self, Box<dyn Error>> {
		let fn_res = (core_library.symbols.as_ref().unwrap().unloaded_task_contents)(Box::new(unloaded_task.deserialize_path.clone()));

		let deserialize_str = (fn_res).into_rust()?;

		let accesses = unloaded_task.accesses.clone();

		let user_data = user_library.load_task(unloaded_task.type_name.as_str(), deserialize_str.as_str())?;

		let closure = {
			let task_impl = user_data.task_impl.lock();
			let task_data = user_data.task_data.lock();
			task_impl.run(&*task_data, name.clone())
		};

		let ret = Ok(Self { name, accesses, closure, user_data });
		ret
	}
}
