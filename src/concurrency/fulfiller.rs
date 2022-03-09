use std::{
	any::Any,
	cell::RefCell,
	panic::{self, AssertUnwindSafe},
	sync::{
		atomic::{AtomicBool, AtomicUsize, Ordering},
		Arc, Barrier, Weak,
	},
};

use threadpool::ThreadPool;

use crate::{
	composition::loaded::loaded_task::LoadedTask,
	concurrency::{fulfiller_chain::FulfillerChain, ready::Ready},
	errors::run_errors::custard_task_panic_error::CustardTaskPanicError,
	identify::task_name::FullTaskName,
	user_types::{task::Task, task_control_flow::task_control_flow::TaskControlFlow},
};

#[derive(Debug)]
pub struct Fulfiller {
	pub cease: AtomicBool,
	pub children_chains: Vec<Weak<FulfillerChain>>,
	pub done: Ready,
	pub prerequisites: Vec<Weak<Fulfiller>>,
	pub task: LoadedTask,
}

impl Fulfiller {
	pub(crate) fn prerequisites_complete(&self) -> bool {
		for prerequisite in &self.prerequisites {
			let upgraded = &prerequisite.upgrade();
			let done = match upgraded {
				Some(v) => &v.done,
				None => return true,
			};
			if !self.done.load_prerequisite(done) {
				return false;
			}
		}
		true
	}

	fn notify_tasks_of_control_flow_change(current_task_name: &FullTaskName, task_result: &TaskControlFlow, all_chains: &Vec<Arc<FulfillerChain>>, barrier: (Arc<AtomicUsize>, Arc<Barrier>)) {
		for chain in all_chains {
			for fulfiller in &chain.chain {
				let upgraded = fulfiller.upgrade();
				let fulfiller = upgraded.as_ref().unwrap();
				let cease = match task_result {
					&TaskControlFlow::Continue => unreachable!(),
					&TaskControlFlow::Err(_) | &TaskControlFlow::StopThis => {
						let user_task = fulfiller.task.user_data.clone();
						let task_impl = user_task.task_impl.lock().unwrap();
						let user_data = user_task.task_data.lock().unwrap();
						task_impl.handle_control_flow_update(&*user_data, current_task_name, &fulfiller.task.name, task_result)
					}
					&TaskControlFlow::Reload => true,
					&TaskControlFlow::ReloadCrate => &current_task_name.crate_name == &fulfiller.task.name.crate_name,
					&TaskControlFlow::StopAll => true,
				};
				if cease {
					fulfiller.cease.store(true, Ordering::Relaxed);
					let completed = barrier.0.fetch_sub(1, Ordering::Relaxed);
					if completed == 1 {
						barrier.1.wait();
					}
				}
			}
		}
	}

	pub(crate) fn run_task(&self, pool: ThreadPool, barrier: (Arc<AtomicUsize>, Arc<Barrier>), all_chains: Arc<Vec<Arc<FulfillerChain>>>, should_reload: Arc<AtomicBool>) {
		if !self.prerequisites_complete() {
			return;
		}
		if !self.cease.load(Ordering::Relaxed) {
			let closure_result: TaskControlFlow;
			let safe_closure_result = AssertUnwindSafe(RefCell::new(Some(TaskControlFlow::Continue)));
			let safe_self = AssertUnwindSafe(self);
			let panic_result = panic::catch_unwind(|| {
				let user_task = &safe_self.task;
				*safe_closure_result.borrow_mut() = Some((user_task.closure.lock().unwrap())(user_task.user_data.task_data.clone()));
			});

			match panic_result {
				Err(e) => unsafe {
					let panic_error = CustardTaskPanicError { offending_task: self.task.name.clone(), error: Box::from_raw(Box::leak(e) as *mut _ as *mut (dyn Any + Send + Sync)) }; //TODO: seems sketchy
					println!("{}", panic_error);
					closure_result = TaskControlFlow::Err(Box::new(panic_error));
				},
				Ok(_) => {
					closure_result = safe_closure_result.take().unwrap();
				}
			}

			match &closure_result {
				TaskControlFlow::Continue => {}
				_ => {
					if let TaskControlFlow::Reload = &closure_result {
						should_reload.store(true, Ordering::Relaxed);
					}

					Self::notify_tasks_of_control_flow_change(&self.task.name, &closure_result, &all_chains, barrier.clone());

					self.cease.store(true, Ordering::Relaxed);
					let completed = barrier.0.fetch_sub(1, Ordering::Relaxed);
					if completed == 1 {
						barrier.1.wait();
					}
				}
			};
		}
		self.done.release();
		for child_chain in &self.children_chains {
			child_chain.upgrade().unwrap().attempt_to_run(barrier.clone(), pool.clone(), all_chains.clone(), should_reload.clone());
		}
	}
}
