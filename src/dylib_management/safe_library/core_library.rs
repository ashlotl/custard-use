use std::{cell::RefCell, error::Error, rc::Rc};

use log::info;

use crate::{
	dylib_management::safe_library::{
		load_types::FFIResult,
		safe_library::{self, DebugMode, LibraryDrop, LibraryRecompile, SafeLibrary},
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
	lib: Option<libloading::Library>,
	drop_list: Rc<RefCell<Vec<libloading::Library>>>,
}

impl<'lib> Drop for CoreLibrary<'lib> {
	fn drop(&mut self) {
		self.on_drop()
	}
}

impl<'lib> LibraryDrop for CoreLibrary<'lib> {
	fn get_library_drop_list(&self) -> Rc<RefCell<Vec<libloading::Library>>> {
		self.drop_list.clone()
	}
}

impl<'lib> SafeLibrary for CoreLibrary<'lib> {
	fn new(name: CrateName, recompile: LibraryRecompile, debug: DebugMode, drop_list: Rc<RefCell<Vec<libloading::Library>>>) -> Result<Self, Box<dyn Error>> {
		println!("constructing CoreLibrary");
		let lib = Some(safe_library::load_crate_as_library(name.clone(), recompile, debug)?);

		println!("done loading");

		let mut ret = Self { name, lib, symbols: None, drop_list };

		unsafe {
			let lib = &*(ret.lib.as_ref().unwrap() as *const libloading::Library);

			info!("Loading library symbols.");

			info!("Getting composition getter.");
			let composition = lib.get(b"__custard_composition__")?;
			info!("Got composition getter.");

			info!("Getting unloaded datachunk getter.");
			let unloaded_datachunk_contents = lib.get(b"__custard_unloaded_datachunk_contents__")?;
			info!("Got unloaded datachunk getter.");

			info!("Getting unloaded task getter.");
			let unloaded_task_contents = lib.get(b"__custard_unloaded_task_contents__")?;
			info!("Got unloaded task getter.");

			ret.symbols = Some(CoreLibrarySymbols { composition, unloaded_datachunk_contents, unloaded_task_contents });

			info!("Loaded library symbols.");
			Ok(ret)
		}
	}

	fn get_crate_name(&self) -> &CrateName {
		&self.name
	}

	fn get_underlying_library(&self) -> &libloading::Library {
		self.lib.as_ref().unwrap()
	}

	unsafe fn get_underlying_library_mut(&mut self) -> &mut Option<libloading::Library> {
		&mut self.lib
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
