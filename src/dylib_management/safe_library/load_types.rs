use std::{error::Error, sync::Arc};

use crate::user_types::{datachunk::Datachunk, task::Task};

pub type DatachunkLoadFn = extern "C" fn(Box<String>) -> Box<FFIResult<Box<dyn Datachunk>, Box<dyn Error>>>;
pub type TaskLoadFn = extern "C" fn(Box<String>) -> Box<FFIResult<Arc<dyn Task + Send + Sync>, Box<dyn Error + Send + Sync>>>;

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

	use std::{error::Error, sync::Arc};

	//make sure no non-FFI-safe types are in use
	use serde::Deserialize;

	use crate::{
		dylib_management::safe_library::load_types::{DatachunkLoadFn, FFIResult, TaskLoadFn},
		user_types::{
			datachunk::Datachunk,
			task::{Task, TaskClosureType},
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

	impl Task for TestTask {
		fn run(self: Arc<Self>) -> TaskClosureType {
			unimplemented!();
		}
	}

	#[no_mangle]
	#[allow(non_snake_case)]
	#[deny(improper_ctypes_definitions)]
	pub extern "C" fn task_load_fn_test(from: Box<String>) -> Box<FFIResult<Arc<dyn Task + Send + Sync>, Box<dyn Error + Send + Sync>>> {
		let created: Result<TestTask, ron::Error> = ron::from_str(from.as_str());

		match created {
			Ok(v) => {
				return Box::new(FFIResult::Ok(Arc::new(v)));
			}
			Err(e) => {
				return Box::new(FFIResult::Err(Box::new(e)));
			}
		}
	}

	#[allow(unused)]
	const TASK_MATCH_TYPE: TaskLoadFn = task_load_fn_test;
}
