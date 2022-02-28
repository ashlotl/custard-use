use std::{error::Error, pin::Pin};



use crate::{
	dylib_management::safe_library::{
		load_types::{FFIResult, FFISafeString},
		safe_library::{self, DebugMode, LibraryRecompile, SafeLibrary},
	},
	identify::crate_name::CrateName,
};

pub type CompositionFunctionType = extern "C" fn() -> CompositionFunctionReturn;
pub type CompositionFunctionReturn = Box<FFIResult<FFISafeString, Box<dyn Error>>>;

pub type UnloadedDatachunkContentsFunctionType = extern "C" fn(FFISafeString) -> UnloadedDatachunkContentsFunctionReturnType;
pub type UnloadedDatachunkContentsFunctionReturnType = Box<FFIResult<FFISafeString, Box<dyn Error>>>;

#[derive(Debug)]
pub(crate) struct CoreLibrarySymbols<'lib> {
	///The composition getter for the library. This usually wraps a call to `get_maybe_const_string` in `utils`.
	pub(crate) composition: libloading::Symbol<'lib, CompositionFunctionType>,
	pub(crate) unloaded_datachunk_contents: libloading::Symbol<'lib, UnloadedDatachunkContentsFunctionType>,
}

#[derive(Debug)]
pub struct CoreLibrary<'lib> {
	name: CrateName,
	pub(crate) symbols: Option<CoreLibrarySymbols<'lib>>,
	lib: Pin<Box<libloading::Library>>,
}

impl<'lib> SafeLibrary for CoreLibrary<'lib> {
	fn new(name: CrateName, recompile: LibraryRecompile, debug: DebugMode) -> Result<Self, Box<dyn Error>> {
		let lib = Box::pin(safe_library::load_crate_as_library(name.clone(), recompile, debug)?);

		let mut ret = Self { name, lib, symbols: None };

		unsafe {
			let lib = &*(&*ret.lib as *const libloading::Library);
			let composition = lib.get(b"__custard_composition__")?;
			let unloaded_datachunk_contents = lib.get(b"__custard_unloaded_datachunk_contents__")?;
			ret.symbols = Some(CoreLibrarySymbols { composition, unloaded_datachunk_contents });
			Ok(ret)
		}
	}

	fn get_crate_name(&self) -> &CrateName {
		&self.name
	}

	fn get_underlying_library(&self) -> &libloading::Library {
		&self.lib
	}
}

#[cfg(test)]
mod tests {
	use std::error::Error;

	use crate::dylib_management::safe_library::{
		core_library::{CompositionFunctionType, UnloadedDatachunkContentsFunctionType},
		load_types::{FFIResult, FFISafeString},
	};

	#[no_mangle]
	#[deny(improper_ctypes_definitions)]
	pub extern "C" fn ensure_composition_ffi_safe() -> Box<FFIResult<FFISafeString, Box<dyn Error>>> {
		unreachable!()
	}

	#[no_mangle]
	#[deny(improper_ctypes_definitions)]
	pub extern "C" fn ensure_unloaded_datachunk_contenets_ffi_safe(_s: FFISafeString) -> Box<FFIResult<FFISafeString, Box<dyn Error>>> {
		unreachable!()
	}

	#[allow(unused)]
	const CHECK_COMPOSITION: CompositionFunctionType = ensure_composition_ffi_safe;

	#[allow(unused)]
	const CHECK_UNLOADED_DATACHUNK_CONTENTS: UnloadedDatachunkContentsFunctionType = ensure_unloaded_datachunk_contenets_ffi_safe;
}
