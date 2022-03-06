use crate::{
	dylib_management::safe_library::{
		load_types::{DatachunkLoadFn, FFIResult, TaskLoadFn},
		safe_library::{self, DebugMode, LibraryRecompile, SafeLibrary},
	},
	errors::load_errors::{custard_load_datachunk_error::CustardLoadDatachunkError, custard_load_task_error::CustardLoadTaskError},
	identify::crate_name::CrateName,
	user_types::{datachunk::Datachunk, task::Task},
};

use libloading::Symbol;

use std::{error::Error, sync::Arc};

#[derive(Debug)]
pub struct UserLibrary {
	name: CrateName,
	lib: *mut libloading::Library,
}

impl SafeLibrary for UserLibrary {
	fn new(name: CrateName, recompile: LibraryRecompile, debug: DebugMode) -> Result<Self, Box<dyn Error>> {
		let lib = safe_library::load_crate_as_library(name.clone(), recompile, debug)?;
		let boxed = Box::new(lib);
		let leaked = Box::leak(boxed);
		Ok(Self { name, lib: leaked as *mut libloading::Library })
	}

	fn get_crate_name(&self) -> &CrateName {
		&self.name
	}

	fn get_underlying_library(&self) -> &libloading::Library {
		unsafe { &(*self.lib) }
	}
}

impl UserLibrary {
	pub fn load_datachunk(&self, type_name: &str, deserialize_str: &str) -> Result<Box<dyn Datachunk>, Box<dyn Error>> {
		let load_fn: Symbol<DatachunkLoadFn> = match unsafe { (*self.lib).get(format!("__custard_datachunk__{}", type_name).as_bytes()) } {
			Ok(v) => v,
			Err(e) => return Err(Box::new(CustardLoadDatachunkError { crate_name: self.name.clone(), type_name: type_name.to_owned(), wrapped_error: Box::new(e) })),
		};

		let ostring = Box::new(deserialize_str.to_owned());

		let ret = load_fn(ostring);

		match *ret {
			FFIResult::Ok(v) => {
				return Ok(v);
			}
			FFIResult::Err(e) => {
				return Err(Box::new(CustardLoadDatachunkError { crate_name: self.name.clone(), type_name: type_name.to_owned(), wrapped_error: e }));
			}
		};
	}

	pub fn load_task(&self, type_name: &str, deserialize_str: &str) -> Result<Arc<dyn Task + Send + Sync>, Box<dyn Error>> {
		let load_fn: Symbol<TaskLoadFn> = match unsafe { (*self.lib).get(format!("__custard_task__{}", type_name).as_bytes()) } {
			Ok(v) => v,
			Err(e) => return Err(Box::new(CustardLoadTaskError { crate_name: self.name.clone(), type_name: type_name.to_owned(), wrapped_error: Box::new(e) })),
		};

		let ostring = Box::new(deserialize_str.to_owned());

		let ret = load_fn(ostring);
		match *ret {
			FFIResult::Ok(v) => {
				return Ok(v);
			}
			FFIResult::Err(e) => {
				return Err(Box::new(CustardLoadTaskError { crate_name: self.name.clone(), type_name: type_name.to_owned(), wrapped_error: e }));
			}
		};
	}
}
