use crate::{concurrency::fulfiller::Fulfiller, identify::task_name::FullTaskName};

use threadpool::ThreadPool;

use std::sync::{
	atomic::{AtomicBool, AtomicUsize},
	Arc, Barrier, Weak,
};

#[derive(Debug)]
pub struct FulfillerChain {
	pub first_name: FullTaskName,
	pub chain: Vec<Weak<Fulfiller>>,
}

impl FulfillerChain {
	pub fn run(&self, barrier: (Arc<AtomicUsize>, Arc<Barrier>), pool: ThreadPool, all_chains: Arc<Vec<Arc<Self>>>, should_reload: Arc<AtomicBool>) {
		for fulfiller_i in 0..self.chain.len() {
			let fulfiller = match self.chain[fulfiller_i].upgrade() {
				Some(v) => v,
				None => return, //program is exiting
			};
			fulfiller.run_task(pool.clone(), barrier.clone(), all_chains.clone(), should_reload.clone());
		}
	}

	pub(crate) fn attempt_to_run(self: Arc<Self>, barrier: (Arc<AtomicUsize>, Arc<Barrier>), pool: ThreadPool, all_chains: Arc<Vec<Arc<Self>>>, should_reload: Arc<AtomicBool>) {
		let first_fulfiller = {
			match self.chain[0].upgrade() {
				Some(v) => v,
				None => return, //program is exiting
			}
		};

		if first_fulfiller.prerequisites_complete() {
			let pool_inner = pool.clone();
			pool.execute(move || {
				self.clone().run(barrier.clone(), pool_inner, all_chains, should_reload);
			});
		}
	}
}
