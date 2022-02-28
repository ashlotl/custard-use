use crate::{
	dylib_management::runtime_compile,
	identify::{crate_name::CrateName, custard_name::CustardName},
};

use libloading::Library;

use std::{error::Error, fmt, path::Path};

#[derive(Clone)]
pub enum LibraryRecompile {
	Recompile,
	TryCached,
	InsistCached,
}

#[derive(Clone)]
pub enum DebugMode {
	Debug,
	Release,
}

pub trait SafeLibrary: fmt::Debug {
	fn new(name: CrateName, recompile: LibraryRecompile, debug: DebugMode) -> Result<Self, Box<dyn Error>>
	where
		Self: Sized;

	fn get_crate_name(&self) -> &CrateName;
	fn get_underlying_library(&self) -> &Library;
}

pub fn load_crate_as_library(name: CrateName, recompile: LibraryRecompile, debug: DebugMode) -> Result<libloading::Library, Box<dyn Error>> {
	let library_name = libloading::library_filename(name.get()).to_str().unwrap().to_owned().replace("-", "_");

	let path = format!("custard_dylib_cache/{}", library_name);

	let should_recompile = match recompile {
		LibraryRecompile::Recompile => true,
		LibraryRecompile::TryCached => !Path::exists(Path::new(&path)),
		_ => false,
	};

	if should_recompile {
		runtime_compile::compile(name.clone(), library_name.as_str(), debug)?;
	}

	Ok(unsafe { libloading::Library::new(path) }?)
}
