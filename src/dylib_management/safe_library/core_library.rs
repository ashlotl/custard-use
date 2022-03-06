use std::error::Error;

use crate::{
	dylib_management::safe_library::{
		load_types::FFIResult,
		safe_library::{self, DebugMode, LibraryRecompile, SafeLibrary},
	},
	identify::crate_name::CrateName,
};

pub type CompositionFunctionType = extern "C" fn() -> CompositionFunctionReturn;
pub type CompositionFunctionReturn = Box<FFIResult<String, Box<dyn Error>>>;

pub type UnloadedDatachunkContentsFunctionType = extern "C" fn(Box<String>) -> UnloadedDatachunkContentsFunctionReturnType;
pub type UnloadedDatachunkContentsFunctionReturnType = Box<FFIResult<String, Box<dyn Error>>>;

pub type UnloadedTaskContentsFunctionType = extern "C" fn(Box<String>) -> UnloadedTaskContentsFunctionReturnType;
pub type UnloadedTaskContentsFunctionReturnType = Box<FFIResult<String, Box<dyn Error>>>;

#[derive(Debug)]
pub(crate) struct CoreLibrarySymbols<'lib> {
	///The composition getter for the library. This usually wraps a call to `get_maybe_const_string` in `utils`.
	pub(crate) composition: libloading::Symbol<'lib, CompositionFunctionType>,
	pub(crate) unloaded_datachunk_contents: libloading::Symbol<'lib, UnloadedDatachunkContentsFunctionType>,
	pub(crate) unloaded_task_contents: libloading::Symbol<'lib, UnloadedTaskContentsFunctionType>,
}

#[derive(Debug)]
pub struct CoreLibrary<'lib> {
	name: CrateName,
	pub(crate) symbols: Option<CoreLibrarySymbols<'lib>>,
	lib: *mut libloading::Library, //leaked to avoid headaches
}

impl<'lib> SafeLibrary for CoreLibrary<'lib> {
	fn new(name: CrateName, recompile: LibraryRecompile, debug: DebugMode) -> Result<Self, Box<dyn Error>> {
		let lib = Box::leak(Box::new(safe_library::load_crate_as_library(name.clone(), recompile, debug)?)) as *mut libloading::Library;

		let mut ret = Self { name, lib, symbols: None };

		unsafe {
			let lib = &*(&*ret.lib as *const libloading::Library);
			let composition = lib.get(b"__custard_composition__")?;
			let unloaded_datachunk_contents = lib.get(b"__custard_unloaded_datachunk_contents__")?;
			let unloaded_task_contents = lib.get(b"__custard_unloaded_task_contents__")?;
			ret.symbols = Some(CoreLibrarySymbols { composition, unloaded_datachunk_contents, unloaded_task_contents });
			Ok(ret)
		}
	}

	fn get_crate_name(&self) -> &CrateName {
		&self.name
	}

	fn get_underlying_library(&self) -> &libloading::Library {
		unsafe { &*self.lib }
	}
}

#[cfg(test)]
mod tests {
	use std::error::Error;

	use crate::dylib_management::safe_library::{
		core_library::{CompositionFunctionType, UnloadedDatachunkContentsFunctionType, UnloadedTaskContentsFunctionType},
		load_types::FFIResult,
	};

	#[no_mangle]
	#[deny(improper_ctypes_definitions)]
	pub extern "C" fn ensure_composition_ffi_safe() -> Box<FFIResult<String, Box<dyn Error>>> {
		unreachable!()
	}

	#[no_mangle]
	#[deny(improper_ctypes_definitions)]
	pub extern "C" fn ensure_unloaded_datachunk_contents_ffi_safe(_s: Box<String>) -> Box<FFIResult<String, Box<dyn Error>>> {
		unreachable!()
	}

	#[no_mangle]
	#[deny(improper_ctypes_definitions)]
	pub extern "C" fn ensure_unloaded_task_contents_ffi_safe(_s: Box<String>) -> Box<FFIResult<String, Box<dyn Error>>> {
		unreachable!()
	}

	#[allow(unused)]
	const CHECK_COMPOSITION: CompositionFunctionType = ensure_composition_ffi_safe;

	#[allow(unused)]
	const CHECK_UNLOADED_DATACHUNK_CONTENTS: UnloadedDatachunkContentsFunctionType = ensure_unloaded_datachunk_contents_ffi_safe;

	#[allow(unused)]
	const CHECK_UNLOADED_TASK_CONTENTS: UnloadedTaskContentsFunctionType = ensure_unloaded_task_contents_ffi_safe;
}
