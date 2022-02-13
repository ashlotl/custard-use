use crate::{
	composition::unloaded::unloaded_octask::UnloadedOCTask,
	concurrency::access::Access,
	identify::task_name::FullTaskName,
	user_types::task::{Task, TaskClosureType},
};

#[derive(Debug)]
pub struct LoadedOCTask {
	pub(crate) name: FullTaskName,
	pub(crate) closure: TaskClosureType,
	accesses: Vec<Access>,
}

impl LoadedOCTask {
	pub(crate) fn new(name: FullTaskName, unloaded: UnloadedOCTask, mut user_data: Box<dyn Task>) -> Self {
		let closure = user_data.run();
		Self { name, closure, accesses: unloaded.accesses }
	}
}
