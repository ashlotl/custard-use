use crate::{concurrency::fulfiller::Fulfiller, errors::tasks_result::TasksResult};

use threadpool::ThreadPool;

use std::sync::{Arc, RwLock};

#[derive(Debug)]
pub(crate) struct FulfillerChain {
	chain: Vec<Arc<Fulfiller>>,
}

impl FulfillerChain {
	fn run(&self, pool: ThreadPool, tasks_result: Arc<RwLock<TasksResult>>) {
		for fulfiller in 0..self.chain.len() {
			self.chain[fulfiller].run_task(fulfiller != 0, pool.clone(), tasks_result.clone());
		}
	}

	pub(crate) fn attempt_to_run(self: Arc<Self>, pool: ThreadPool, tasks_result: Arc<RwLock<TasksResult>>) {
		if self.chain[0].prerequisites_complete() {
			let pool_inner = pool.clone();
			let pool = pool.clone();
			pool.execute(move || {
				self.run(pool_inner, tasks_result);
			});
		}
	}
}
