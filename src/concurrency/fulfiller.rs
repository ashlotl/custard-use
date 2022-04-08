use crate::{
	composition::loaded::loaded_task::LoadedTask,
	concurrency::{
		fulfiller_chain::FulfillerChain,
		possibly_poisoned_mutex::PossiblyPoisonedMutex, ready::Ready,
	},
	errors::run_errors::custard_task_panic_error::CustardTaskPanicError,
	identify::task_name::FullTaskName,
	instance_control_flow::InstanceControlFlow,
	user_types::task_control_flow::task_control_flow::{
		TaskControlFlow, TaskHandlerState,
	},
};

use log::{error, info, warn};
use threadpool::ThreadPool;

use std::{
	cell::RefCell,
	panic::{self, AssertUnwindSafe},
	rc::Rc,
	sync::{Arc, Barrier, BarrierWaitResult, Mutex, Weak},
};

#[derive(Debug)]
pub struct Quit {
	///fulfillers that dont have errors and can be rerun in a reload
	nominal_count: Mutex<usize>,
	active_count: Mutex<usize>,
	barrier: Barrier,
}

impl Quit {
	pub fn new(active_count: usize) -> Self {
		Self {
			nominal_count: Mutex::new(active_count),
			active_count: Mutex::new(active_count),
			barrier: Barrier::new(2),
		}
	}

	pub(crate) fn begin_fulfillers(&self, num_to_add: isize) {
		let mut active_g = self.active_count.lock().unwrap();
		*active_g = ((*active_g as isize) + num_to_add) as usize;
	}

	pub(crate) fn cease_fulfiller(&self, fulfiller: &Fulfiller) {
		*fulfiller.cease.lock().unwrap() = true;

		let mut active_count = self.active_count.lock().unwrap();

		*active_count -= 1;
		info!("New count of active fulfillers: {}", *active_count);
		if *active_count == 0 {
			std::mem::drop(active_count);
			info!("Waiting for main thread to quit.");
			self.barrier.wait(); //return to this call in CustardInstance. Make sure this doesnt get called twice and set off a deadlock
		}
	}

	pub(crate) fn main_thread_wait(&self) -> BarrierWaitResult {
		self.barrier.wait()
	}

	pub(crate) unsafe fn reset(&self) -> usize {
		info!("Resetting active/nominal counts and barrier for reload");
		let nominal_count = *self.nominal_count.lock().unwrap();
		*self.active_count.lock().unwrap() = nominal_count;
		*((&self.barrier as *const _) as *mut Barrier) = Barrier::new(2);
		nominal_count
	}
}

#[derive(Debug)]
pub struct Fulfiller {
	pub cease: Mutex<bool>,
	pub error: Mutex<bool>,
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

	fn notify_tasks_of_control_flow_change(
		current_task_name: &FullTaskName,
		task_result: &TaskControlFlow,
		all_chains: &Vec<Arc<FulfillerChain>>,
		quit: Arc<Quit>,
	) {
		for chain in all_chains {
			for fulfiller in &chain.chain {
				let upgraded = fulfiller.upgrade();
				if upgraded.is_none() {
					continue;
				}
				let fulfiller = upgraded.as_ref().unwrap();

				if *fulfiller.cease.lock().unwrap() {
					continue;
				}

				let error_or_stop = || {
					if current_task_name
						== &fulfiller.task.as_ref().unwrap().name
					{
						return true;
					}
					let user_task =
						fulfiller.task.as_ref().unwrap().user_data.clone();
					let mut task_impl = user_task.lock();
					TaskHandlerState::Stop
						== task_impl.handle_control_flow_update(
							current_task_name,
							&fulfiller.task.as_ref().unwrap().name,
							task_result,
						)
				};

				let cease = match task_result {
					TaskControlFlow::Continue => unreachable!(),
					TaskControlFlow::Err(error) => {
						match error.downcast_ref::<CustardTaskPanicError>() {
							Some(_) => {
								warn!(
									"Exiting because a task panicked: {:?}",
									fulfiller.task.as_ref().unwrap().name
								);
								true
							}
							None => error_or_stop(),
						}
					}
					TaskControlFlow::StopThis => error_or_stop(),
					TaskControlFlow::FullReload
					| TaskControlFlow::PartialReload(_)
					| TaskControlFlow::StopAll => true,
				};
				if cease {
					quit.cease_fulfiller(&fulfiller);
					info!(
						"Fulfiller ceased: {:?}",
						fulfiller.task.as_ref().unwrap().name
					);
				}
			}
		}
	}

	/// Runs a task. Returns true if it panicked, so the remainder of the chain can occur in a different thread. Even in the case of an error, run_task will call `Ready::release()` to allow other tasks to interpret the error.
	pub(crate) fn run_task(
		&self,
		pool: ThreadPool,
		quit: Arc<Quit>,
		all_chains: Arc<Vec<Arc<FulfillerChain>>>,
		instance_control_flow: Arc<PossiblyPoisonedMutex<InstanceControlFlow>>,
	) {
		if !self.prerequisites_complete() {
			return;
		}

		let cease = { *self.cease.lock().unwrap() };

		if !cease {
			let closure_result: TaskControlFlow;
			let safe_closure_result =
				AssertUnwindSafe(RefCell::new(Some(TaskControlFlow::Continue)));
			let safe_self = AssertUnwindSafe(self);
			let panic_result = panic::catch_unwind(|| {
				let user_task = &safe_self.task;
				*safe_closure_result.borrow_mut() = Some((user_task
					.as_ref()
					.unwrap()
					.closure
					.as_ref()
					.unwrap()
					.lock()
					.unwrap())(
					user_task.as_ref().unwrap().user_data.clone(),
				));
			});

			match panic_result {
				Err(e) => {
					let panic_error = CustardTaskPanicError {
						offending_task: self
							.task
							.as_ref()
							.unwrap()
							.name
							.clone(),
						error: e,
					};

					closure_result = TaskControlFlow::Err(Rc::new(panic_error));
					*instance_control_flow.lock() =
						InstanceControlFlow::RecreateThreadpool;
				}
				Ok(_) => {
					closure_result = safe_closure_result.take().unwrap();
				}
			}

			match &closure_result {
				TaskControlFlow::Continue => {}
				_ => {
					match &closure_result {
						TaskControlFlow::FullReload => {
							*instance_control_flow.lock() =
								InstanceControlFlow::FullReload
						}
						TaskControlFlow::PartialReload(v) => {
							*instance_control_flow.lock() =
								InstanceControlFlow::PartialReload(v.clone())
						}
						TaskControlFlow::Err(e) => {
							*self.error.lock().unwrap() = true;
							*quit.nominal_count.lock().unwrap() -= 1;
							error!("Task error: {}", e);
						}
						TaskControlFlow::StopAll => {
							*instance_control_flow.lock() =
								InstanceControlFlow::Stop
						}
						_ => {}
					}

					Self::notify_tasks_of_control_flow_change(
						&self.task.as_ref().unwrap().name,
						&closure_result,
						&all_chains,
						quit.clone(),
					);
				}
			};
		}
		self.done.release();

		for child_chain in &self.children_chains {
			child_chain.upgrade().unwrap().attempt_to_run(
				quit.clone(),
				pool.clone(),
				all_chains.clone(),
				instance_control_flow.clone(),
			);
		}
	}
}
