use std::{
	cell::RefCell,
	panic::{self, AssertUnwindSafe},
	rc::Rc,
	sync::{
		atomic::{AtomicBool, AtomicUsize, Ordering},
		Arc, Barrier, Mutex, Weak,
	},
};

use threadpool::ThreadPool;

use crate::{
	composition::loaded::loaded_task::LoadedTask,
	concurrency::{fulfiller_chain::FulfillerChain, ready::Ready},
	errors::run_errors::custard_task_panic_error::CustardTaskPanicError,
	identify::task_name::FullTaskName,
	instance_control_flow::InstanceControlFlow,
	user_types::task_control_flow::task_control_flow::TaskControlFlow,
};

#[derive(Debug)]
pub struct Fulfiller {
	pub cease: AtomicBool,
	pub children_chains: Vec<Weak<FulfillerChain>>,
	pub done: Ready,
	pub prerequisites: Vec<Weak<Fulfiller>>,
	pub task: Option<LoadedTask>,
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
						let user_task = fulfiller.task.as_ref().unwrap().user_data.clone();
						let task_impl_res = user_task.task_impl.lock();
						let user_data_res = user_task.task_data.lock();
						let ret = match (task_impl_res, user_data_res) {
							(Ok(task_impl), Ok(user_data)) => task_impl.handle_control_flow_update(&*user_data, current_task_name, &fulfiller.task.as_ref().unwrap().name, task_result),
							_ => {
								println!("poisoned mutex");
								true
							}
						};
						ret
					}
					&(TaskControlFlow::FullReload | TaskControlFlow::PartialReload | TaskControlFlow::StopAll) => true,
				};
				if cease {
					fulfiller.cease.store(true, Ordering::Relaxed);
					let completed = barrier.0.fetch_sub(1, Ordering::Relaxed);
					if completed == 1 {
						barrier.1.wait(); //return to this call in CustardInstance
					}
				}
			}
		}
	}

	pub(crate) fn run_task(&self, pool: ThreadPool, barrier: (Arc<AtomicUsize>, Arc<Barrier>), all_chains: Arc<Vec<Arc<FulfillerChain>>>, instance_control_flow: Arc<Mutex<InstanceControlFlow>>) {
		if !self.prerequisites_complete() {
			return;
		}
		let cease = self.cease.load(Ordering::Relaxed);
		if !cease {
			println!("not cease");
		}
		if !cease {
			let closure_result: TaskControlFlow;
			let safe_closure_result = AssertUnwindSafe(RefCell::new(Some(TaskControlFlow::Continue)));
			let safe_self = AssertUnwindSafe(self);
			let panic_result = panic::catch_unwind(|| {
				let user_task = &safe_self.task;
				*safe_closure_result.borrow_mut() = Some((user_task.as_ref().unwrap().closure.lock().unwrap())(user_task.as_ref().unwrap().user_data.task_data.clone()));
			});

			match panic_result {
				Err(e) => {
					let panic_error = CustardTaskPanicError { offending_task: self.task.as_ref().unwrap().name.clone(), error: e };
					println!("{}", panic_error);
					closure_result = TaskControlFlow::Err(Rc::new(panic_error));
				}
				Ok(_) => {
					closure_result = safe_closure_result.take().unwrap();
				}
			}

			match &closure_result {
				TaskControlFlow::Continue => {}
				_ => {
					match &closure_result {
						TaskControlFlow::FullReload => *instance_control_flow.lock().unwrap() = InstanceControlFlow::FullReload,
						TaskControlFlow::PartialReload => *instance_control_flow.lock().unwrap() = InstanceControlFlow::PartialReload,
						TaskControlFlow::Err(e) => println!("{}", e),
						TaskControlFlow::StopAll => *instance_control_flow.lock().unwrap() = InstanceControlFlow::Stop,
						_ => {}
					}

					Self::notify_tasks_of_control_flow_change(&self.task.as_ref().unwrap().name, &closure_result, &all_chains, barrier.clone());
				}
			};
		}
		self.done.release();

		for child_chain in &self.children_chains {
			child_chain.upgrade().unwrap().attempt_to_run(barrier.clone(), pool.clone(), all_chains.clone(), instance_control_flow.clone());
		}
	}
}
