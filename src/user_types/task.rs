use std::{
	any::Any,
	fmt::Debug,
	sync::{Arc, RwLock},
};

use crate::errors::tasks_result::TasksResult;

pub type TaskClosureOutput = Result<(), Arc<dyn Any + Send + Sync + 'static>>;
pub type TaskClosureType = Box<dyn Send + Sync + TaskClosureTrait>;

pub trait TaskClosureTrait: Debug + FnMut(Arc<RwLock<TasksResult>>) -> TaskClosureOutput {}

pub trait Task: Debug + Send + Sync {
	fn run(&mut self) -> TaskClosureType;
}
