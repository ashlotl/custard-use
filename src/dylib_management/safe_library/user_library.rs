use crate::{
	dylib_management::safe_library::{
		load_types::{DatachunkLoadFn, FFIResult, TaskLoadFn},
		safe_library::{
			self, DebugMode, LibraryDrop, LibraryRecompile, SafeLibrary,
		},
	},
	errors::load_errors::{
		custard_load_datachunk_error::CustardLoadDatachunkError,
		custard_load_task_error::CustardLoadTaskError,
	},
	identify::crate_name::CrateName,
	user_types::{datachunk::DatachunkObject, task::TaskObject},
};

use libloading::Symbol;

use std::{cell::RefCell, error::Error, rc::Rc};

#[derive(Debug)]
pub struct UserLibrary {
	name: CrateName,
	lib: Option<libloading::Library>,
	drop_list: Rc<RefCell<Vec<libloading::Library>>>,
}

impl Drop for UserLibrary {
	fn drop(&mut self) {
		self.on_drop()
	}
}

impl LibraryDrop for UserLibrary {
	fn get_library_drop_list(&self) -> Rc<RefCell<Vec<libloading::Library>>> {
		self.drop_list.clone()
	}
}

impl SafeLibrary for UserLibrary {
	fn new(
		name: CrateName,
		recompile: LibraryRecompile,
		debug: DebugMode,
		drop_list: Rc<RefCell<Vec<libloading::Library>>>,
	) -> Result<Self, Box<dyn Error>> {
		let lib = Some(safe_library::load_crate_as_library(
			name.clone(),
			recompile,
			debug,
		)?);
		Ok(Self {
			name,
			lib,
			drop_list,
		})
	}

	fn get_crate_name(&self) -> &CrateName {
		&self.name
	}

	fn get_underlying_library(&self) -> &libloading::Library {
		self.lib.as_ref().unwrap()
	}

	unsafe fn get_underlying_library_mut(
		&mut self,
	) -> &mut Option<libloading::Library> {
		&mut self.lib
	}
}

impl UserLibrary {
	pub fn load_datachunk(
		&self,
		type_name: &str,
		deserialize_str: &str,
	) -> Result<DatachunkObject, Box<dyn Error>> {
		let load_fn: Symbol<DatachunkLoadFn> = match unsafe {
			self.lib
				.as_ref()
				.unwrap()
				.get(format!("__custard_datachunk__{}", type_name).as_bytes())
		} {
			Ok(v) => v,
			Err(e) => {
				return Err(Box::new(CustardLoadDatachunkError {
					crate_name: self.name.clone(),
					type_name: type_name.to_owned(),
					wrapped_error: Box::new(e),
				}))
			}
		};

		let ostring = Box::new(deserialize_str.to_owned());

		let ret = load_fn(ostring);

		match *ret {
			FFIResult::Ok(v) => {
				return Ok(v);
			}
			FFIResult::Err(e) => {
				return Err(Box::new(CustardLoadDatachunkError {
					crate_name: self.name.clone(),
					type_name: type_name.to_owned(),
					wrapped_error: e,
				}));
			}
		};
	}

	pub fn load_task(
		&self,
		type_name: &str,
		deserialize_str: &str,
	) -> Result<TaskObject, Box<dyn Error>> {
		let load_fn: Symbol<TaskLoadFn> = match unsafe {
			self.lib
				.as_ref()
				.unwrap()
				.get(format!("__custard_task__{}", type_name).as_bytes())
		} {
			Ok(v) => v,
			Err(e) => {
				return Err(Box::new(CustardLoadTaskError {
					crate_name: self.name.clone(),
					type_name: type_name.to_owned(),
					wrapped_error: Box::new(e),
				}))
			}
		};

		let ostring = Box::new(deserialize_str.to_owned());

		let ret = load_fn(ostring);
		match *ret {
			FFIResult::Ok(v) => {
				return Ok(v);
			}
			FFIResult::Err(e) => {
				return Err(Box::new(CustardLoadTaskError {
					crate_name: self.name.clone(),
					type_name: type_name.to_owned(),
					wrapped_error: e,
				}));
			}
		};
	}
}
