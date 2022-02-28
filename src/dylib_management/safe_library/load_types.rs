use std::{error::Error, ops::Try};

use crate::user_types::datachunk::Datachunk;

pub type DatachunkLoadFn = extern "C" fn(FFISafeString) -> Box<FFIResult<Box<dyn Datachunk>, Box<dyn Error>>>;

#[repr(C)]
pub struct FFISafeString {
	ptr: *mut u8,
	length: usize,
	capacity: usize,
}

impl FFISafeString {
	pub fn from_rust(mut input: String) -> Self {
		let ret = Self { ptr: input.as_mut_ptr(), length: input.len(), capacity: input.capacity() };
		std::mem::forget(input);
		ret
	}

	pub fn into_rust(self) -> String {
		let ret = unsafe { String::from_raw_parts(self.ptr, self.length, self.capacity) };
		ret
	}
}

#[repr(C)]
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
		dylib_management::safe_library::load_types::{DatachunkLoadFn, FFIResult, FFISafeString},
		user_types::datachunk::Datachunk,
	};

	#[derive(Debug, Deserialize)]
	pub struct TestDatachunk();

	impl Datachunk for TestDatachunk {}

	#[no_mangle]
	#[allow(non_snake_case)]
	#[deny(improper_ctypes_definitions)]
	pub extern "C" fn datachunk_load_fn_test(from: FFISafeString) -> Box<FFIResult<Box<dyn Datachunk>, Box<dyn Error>>> {
		let from: String = from.into_rust();
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
	const MATCH_TYPE: DatachunkLoadFn = datachunk_load_fn_test;
}
