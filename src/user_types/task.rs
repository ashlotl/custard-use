use mopa::mopafy;

use crate::{concurrency::possibly_poisoned_mutex::PossiblyPoisonedMutex, identify::task_name::FullTaskName, user_types::task_control_flow::task_control_flow::TaskControlFlow};

use std::{
	fmt::Debug,
	sync::{Arc, Mutex},
};

pub type TaskClosureOutput = TaskControlFlow;
pub type TaskClosureType = Box<TaskClosureTrait<dyn Taskable>>;

pub type TaskClosureTrait<T> = Mutex<dyn Fn(Arc<PossiblyPoisonedMutex<T>>) -> TaskClosureOutput + Send>;

pub trait Taskable: Debug + Send + mopa::Any {
	fn run(&mut self, this_task_name: FullTaskName) -> TaskClosureType;
	fn handle_control_flow_update(&mut self, this_task_name: &FullTaskName, other_task_name: &FullTaskName, control_flow: &TaskControlFlow) -> bool;
}
mopafy!(Taskable);

#[derive(Clone, Debug)]
pub struct Task {
	pub inner: Arc<PossiblyPoisonedMutex<dyn Taskable>>,
}
