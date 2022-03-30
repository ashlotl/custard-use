use mopa::mopafy;

use crate::{composition::loaded::datachunk_getter::DatachunkGetter, concurrency::possibly_poisoned_mutex::PossiblyPoisonedMutex, identify::task_name::FullTaskName, user_types::task_control_flow::task_control_flow::TaskControlFlow};

use std::{
	fmt::Debug,
	sync::{Arc, Mutex},
};

pub type TaskClosureOutput = TaskControlFlow;
pub type TaskClosureType = Box<TaskClosureTrait<dyn Taskable>>;
pub type TaskClosureTrait<T> = Mutex<dyn FnMut(Arc<PossiblyPoisonedMutex<T>>) -> TaskClosureOutput + Send>;

pub type TaskObject = Arc<PossiblyPoisonedMutex<dyn Taskable>>;

pub trait Taskable: Debug + Send + mopa::Any {
	fn run(&mut self, this_task_name: FullTaskName, datachunk_getter: Arc<DatachunkGetter>) -> TaskClosureType;
	fn handle_control_flow_update(&mut self, this_task_name: &FullTaskName, other_task_name: &FullTaskName, control_flow: &TaskControlFlow) -> bool;
}
mopafy!(Taskable);
