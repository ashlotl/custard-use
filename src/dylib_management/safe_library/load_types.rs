use std::error::Error;

use crate::user_types::{datachunk::Datachunk, task::Task};

pub type DatachunkLoadFn = extern "C" fn(Box<String>) -> Box<FFIResult<Box<dyn Datachunk>, Box<dyn Error>>>;
pub type TaskLoadFn = extern "C" fn(Box<String>) -> Box<FFIResult<Task, Box<dyn Error + Send + Sync>>>;

#[repr(C)]
#[derive(Clone)]
pub enum FFIResult<T, E> {
	Ok(T),
	Err(E),
}

impl<T, E> FFIResult<T, E> {
	pub fn from_rust(r: Result<T, E>) -> Self {
		match r {
			Ok(v) => FFIResult::Ok(v),
			Err(e) => FFIResult::Err(e),
		}
	}

	pub fn into_rust(self) -> Result<T, E> {
		match self {
			FFIResult::Ok(v) => Ok(v),
			FFIResult::Err(e) => Err(e),
		}
	}
}

#[cfg(test)]
mod tests {

	use std::error::Error;

	//make sure no non-FFI-safe types are in use
	use serde::Deserialize;

	use crate::{
		dylib_management::safe_library::load_types::{DatachunkLoadFn, FFIResult, TaskLoadFn},
		identify::task_name::FullTaskName,
		user_types::{
			datachunk::Datachunk,
			task::{Task, TaskClosureType, TaskData, TaskImpl},
			task_control_flow::task_control_flow::TaskControlFlow,
		},
	};

	#[derive(Debug, Deserialize)]
	pub struct TestDatachunk();

	impl Datachunk for TestDatachunk {}

	#[no_mangle]
	#[allow(non_snake_case)]
	#[deny(improper_ctypes_definitions)]
	pub extern "C" fn datachunk_load_fn_test(from: Box<String>) -> Box<FFIResult<Box<dyn Datachunk>, Box<dyn Error>>> {
		let created: Result<TestDatachunk, ron::Error> = ron::from_str(from.as_str());

		match created {
			Ok(v) => {
				return Box::new(FFIResult::Ok(Box::new(v)));
			}
			Err(e) => {
				return Box::new(FFIResult::Err(Box::new(e)));
			}
		}
	}

	#[allow(unused)]
	const DATACHUNK_MATCH_TYPE: DatachunkLoadFn = datachunk_load_fn_test;

	#[derive(Debug, Deserialize)]
	pub struct TestTask();

	#[derive(Debug)]
	pub struct TestTaskImpl();

	impl TaskImpl for TestTaskImpl {
		fn run(&self, _: &dyn TaskData, _: FullTaskName) -> TaskClosureType {
			unimplemented!();
		}
		fn handle_control_flow_update(&self, _: &dyn TaskData, _: &FullTaskName, _: &FullTaskName, _: &TaskControlFlow) -> bool {
			unimplemented!();
		}
	}

	#[no_mangle]
	#[allow(non_snake_case)]
	#[deny(improper_ctypes_definitions)]
	pub extern "C" fn task_load_fn_test(_from: Box<String>) -> Box<FFIResult<Task, Box<dyn Error + Send + Sync>>> {
		unimplemented!()
	}

	#[allow(unused)]
	const TASK_MATCH_TYPE: TaskLoadFn = task_load_fn_test;
}
