use std::sync::{Arc, RwLock, Weak};

use threadpool::ThreadPool;

use crate::{
	composition::loaded::loaded_octask::LoadedOCTask,
	concurrency::{fulfiller_chain::FulfillerChain, ready::Ready},
	errors::tasks_result::TasksResult,
	user_types::task::TaskClosureType,
};

#[derive(Debug)]
pub(crate) struct Fulfiller {
	children_chains: Vec<Weak<FulfillerChain>>,
	done: Ready,
	prerequisites: Vec<Weak<Fulfiller>>,
	task: LoadedOCTask,
}

impl Fulfiller {
	pub(crate) fn prerequisites_complete(&self) -> bool {
		for prerequisite in &self.prerequisites {
			if !self.done.load_prerequisite(&prerequisite.upgrade().unwrap().done) {
				return false;
			}
		}
		true
	}

	pub(crate) fn run_task(&self, not_first: bool, pool: ThreadPool, tasks_result: Arc<RwLock<TasksResult>>) {
		if not_first {
			assert_eq!(self.prerequisites_complete(), true)
		}
		if let Err(e) = unsafe { (*(&self.task.closure as *const TaskClosureType as *mut TaskClosureType))(tasks_result.clone()) } {
			let mut g = tasks_result.write().unwrap();
			g.errors.insert(self.task.name.clone(), e);
		}
		self.done.release();
		for child_chain in &self.children_chains {
			child_chain.upgrade().unwrap().attempt_to_run(pool.clone(), tasks_result.clone());
		}
	}
}
