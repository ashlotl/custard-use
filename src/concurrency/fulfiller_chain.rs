use crate::{concurrency::fulfiller::Fulfiller, errors::tasks_result::TasksResult, identify::task_name::FullTaskName};

use threadpool::ThreadPool;

use std::sync::{Arc, RwLock, Weak};

#[derive(Debug)]
pub struct FulfillerChain {
	pub first_name: FullTaskName,
	pub chain: Vec<Weak<Fulfiller>>,
}

impl FulfillerChain {
	pub fn run(&self, pool: ThreadPool, tasks_result: Arc<RwLock<TasksResult>>) {
		for fulfiller_i in 0..self.chain.len() {
			let fulfiller = match self.chain[fulfiller_i].upgrade() {
				Some(v) => v,
				None => return, //program is exiting
			};
			fulfiller.run_task(&fulfiller.task.name, pool.clone(), tasks_result.clone());
		}
	}

	pub(crate) fn attempt_to_run(self: Arc<Self>, pool: ThreadPool, tasks_result: Arc<RwLock<TasksResult>>) {
		let first_fulfiller = match self.chain[0].upgrade() {
			Some(v) => v,
			None => return, //program is exiting
		};
		if first_fulfiller.prerequisites_complete() {
			let pool_inner = pool.clone();
			let pool = pool.clone();
			pool.execute(move || {
				self.run(pool_inner, tasks_result);
			});
		}
	}
}
