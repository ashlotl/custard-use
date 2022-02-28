use crate::{
	dylib_management::safe_library::{
		load_types::{DatachunkLoadFn, FFIResult, FFISafeString},
		safe_library::{self, DebugMode, LibraryRecompile, SafeLibrary},
	},
	errors::load_errors::custard_load_datachunk_error::{CustardLoadDatachunkError},
	identify::crate_name::CrateName,
	user_types::datachunk::Datachunk,
};

use libloading::Symbol;

use std::error::Error;

#[derive(Debug)]
pub(crate) struct UserLibrary {
	name: CrateName,
	lib: libloading::Library,
}

impl SafeLibrary for UserLibrary {
	fn new(name: CrateName, recompile: LibraryRecompile, debug: DebugMode) -> Result<Self, Box<dyn Error>> {
		let lib = safe_library::load_crate_as_library(name.clone(), recompile, debug)?;
		Ok(Self { name, lib })
	}

	fn get_crate_name(&self) -> &CrateName {
		&self.name
	}

	fn get_underlying_library(&self) -> &libloading::Library {
		&self.lib
	}
}

impl UserLibrary {
	pub fn load_datachunk(&self, type_name: &str, deserialize_str: &str) -> Result<Box<dyn Datachunk>, Box<dyn Error>> {
		let load_fn: Symbol<DatachunkLoadFn> = match unsafe { self.lib.get(format!("__custard_datachunk__{}", type_name).as_bytes()) } {
			Ok(v) => v,
			Err(e) => return Err(Box::new(CustardLoadDatachunkError { crate_name: self.name.clone(), type_name: type_name.to_owned(), wrapped_error: Box::new(e) })),
		};

		let cstring = FFISafeString::from_rust(deserialize_str.to_owned());

		let ret = load_fn(cstring);

		match *ret {
			FFIResult::Ok(v) => {
				return Ok(v);
			}
			FFIResult::Err(e) => {
				return Err(Box::new(CustardLoadDatachunkError { crate_name: self.name.clone(), type_name: type_name.to_owned(), wrapped_error: e }));
			}
		};
	}
}
