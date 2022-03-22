use crate::{
	concurrency::{
		fulfiller::{Fulfiller, Quit},
		possibly_poisoned_mutex::PossiblyPoisonedMutex,
	},
	identify::task_name::FullTaskName,
	instance_control_flow::InstanceControlFlow,
};

use threadpool::ThreadPool;

use std::sync::{Arc, Weak};

#[derive(Debug)]
pub struct FulfillerChain {
	pub first_name: FullTaskName,
	pub chain: Vec<Weak<Fulfiller>>,
}

impl FulfillerChain {
	/// The start at parameter is intended for use with moving threads, such as in the case of a panic.
	pub(super) fn run(self: &Arc<Self>, quit: Arc<Quit>, pool: ThreadPool, all_chains: Arc<Vec<Arc<Self>>>, instance_control_flow: Arc<PossiblyPoisonedMutex<InstanceControlFlow>>) {
		for fulfiller_i in 0..self.chain.len() {
			let fulfiller = match self.chain[fulfiller_i].upgrade() {
				Some(v) => v,
				None => return, //program is exiting
			};
			fulfiller.run_task(pool.clone(), quit.clone(), all_chains.clone(), instance_control_flow.clone())
		}
	}

	pub(crate) fn attempt_to_run(self: Arc<Self>, quit: Arc<Quit>, pool: ThreadPool, all_chains: Arc<Vec<Arc<Self>>>, instance_control_flow: Arc<PossiblyPoisonedMutex<InstanceControlFlow>>) {
		let first_fulfiller = {
			match self.chain[0].upgrade() {
				Some(v) => v,
				None => {
					println!("warning: empty fulfiller");
					return;
				} //program is exiting
			}
		};

		// if barrier.2.load(Ordering::Relaxed) {
		// 	println!("due to a panic this is exiting");
		// 	return;
		// }

		if first_fulfiller.prerequisites_complete() {
			let pool_inner = pool.clone();
			pool.execute(move || {
				self.clone().run(quit.clone(), pool_inner, all_chains, instance_control_flow);
			});
		}
	}
}
