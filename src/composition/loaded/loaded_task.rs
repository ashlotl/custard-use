use crate::{
	composition::{
		loaded::{datachunk_getter::DatachunkGetter, loaded_crate::LoadedCrate},
		unloaded::unloaded_task::UnloadedTask,
	},
	concurrency::access::Access,
	dylib_management::safe_library::{core_library::CoreLibrary, user_library::UserLibrary},
	errors::run_errors::custard_task_panic_error::CustardTaskPanicError,
	identify::{crate_name::CrateName, task_name::FullTaskName},
	user_types::task::{TaskClosureType, TaskObject},
	utils::mutable_arc::MutableArc,
};

use std::{
	collections::BTreeMap,
	error::Error,
	fmt::{self, Formatter},
	panic::{self, AssertUnwindSafe},
	sync::Arc,
};

pub struct LoadedTask {
	pub name: FullTaskName,
	pub closure: Option<TaskClosureType>,
	pub user_data: TaskObject,
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

		let ret = Ok(Self { name, accesses, closure: None, user_data });
		ret
	}

	pub fn load_closure(&mut self, crate_table: MutableArc<BTreeMap<CrateName, LoadedCrate>>) -> Result<(), Box<dyn Error>> {
		self.closure = {
			let user_data = AssertUnwindSafe(self.user_data.clone());
			let datachunk_getter = AssertUnwindSafe(Arc::new(DatachunkGetter::new(crate_table, self.accesses.clone())));
			match panic::catch_unwind(|| {
				let mut task_impl = user_data.lock();
				task_impl.run(self.name.clone(), datachunk_getter.clone())
			}) {
				Ok(v) => Some(v),
				Err(e) => return Err(Box::new(CustardTaskPanicError { offending_task: self.name.clone(), error: e })),
			}
		};
		Ok(())
	}
}
