use mopa::mopafy;

use crate::{concurrency::possibly_poisoned_mutex::PossiblyPoisonedMutex, identify::task_name::FullTaskName, user_types::task_control_flow::task_control_flow::TaskControlFlow};

use std::{
	fmt::Debug,
	sync::{Arc, Mutex},
};

pub type TaskClosureOutput = TaskControlFlow;
pub type TaskClosureType = Box<TaskClosureTrait<dyn TaskData>>;

pub type TaskClosureTrait<T> = Mutex<dyn Fn(Arc<PossiblyPoisonedMutex<T>>) -> TaskClosureOutput + Send>;

pub trait TaskData: Debug + mopa::Any + Send {}
mopafy!(TaskData);

pub trait TaskImpl: Debug + Send {
	fn run(&self, task_data: &dyn TaskData, task_name: FullTaskName) -> TaskClosureType;
	fn handle_control_flow_update(&self, task_data: &dyn TaskData, this_task_name: &FullTaskName, other_task_name: &FullTaskName, control_flow: &TaskControlFlow) -> bool;
}

#[derive(Clone, Debug)]
pub struct Task {
	pub task_data: Arc<PossiblyPoisonedMutex<dyn TaskData>>,
	pub task_impl: Arc<PossiblyPoisonedMutex<dyn TaskImpl>>,
}
