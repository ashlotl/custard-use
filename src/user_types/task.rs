use std::{
	error::Error,
	fmt::Debug,
	sync::{Arc, RwLock},
};

use crate::errors::tasks_result::TasksResult;

pub type TaskClosureOutput = Result<(), Arc<dyn Error + Send + Sync>>;
pub type TaskClosureType = Box<TaskClosureTrait>;

pub type TaskClosureTrait = dyn Fn(Arc<RwLock<TasksResult>>) -> TaskClosureOutput + Send + Sync;

pub trait Task: Debug + Send + Sync {
	fn run(self: Arc<Self>) -> TaskClosureType;
}
