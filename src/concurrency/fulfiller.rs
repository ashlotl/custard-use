use std::sync::{Arc, RwLock, Weak};

use threadpool::ThreadPool;

use crate::{
	composition::loaded::loaded_task::LoadedTask,
	concurrency::{fulfiller_chain::FulfillerChain, ready::Ready},
	errors::tasks_result::TasksResult,
	identify::task_name::FullTaskName,
};

#[derive(Debug)]
pub struct Fulfiller {
	pub children_chains: Vec<Weak<FulfillerChain>>,
	pub done: Ready,
	pub prerequisites: Vec<Weak<Fulfiller>>,
	pub task: LoadedTask,
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

	pub(crate) fn run_task(&self, name: &FullTaskName, pool: ThreadPool, tasks_result: Arc<RwLock<TasksResult>>) {
		if !self.prerequisites_complete() {
			return;
		}
		if let Err(e) = (self.task.closure)(tasks_result.clone()) {
			let mut g = tasks_result.write().unwrap();
			g.errors.insert(name.clone(), e);
		}
		self.done.release();
		for child_chain in &self.children_chains {
			child_chain.upgrade().unwrap().attempt_to_run(pool.clone(), tasks_result.clone());
		}
	}
}
