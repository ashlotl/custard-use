use crate::{
	dylib_management::runtime_compile,
	identify::{crate_name::CrateName, custard_name::CustardName},
};

use libloading::Library;
use log::info;

use std::{cell::RefCell, collections::BTreeMap, error::Error, fmt, fs, path::Path, rc::Rc};

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

pub trait SafeLibrary: fmt::Debug + LibraryDrop {
	fn new(name: CrateName, recompile: LibraryRecompile, debug: DebugMode, drop_list: Rc<RefCell<Vec<Library>>>) -> Result<Self, Box<dyn Error>>
	where
		Self: Sized;

	fn get_crate_name(&self) -> &CrateName;
	fn get_underlying_library(&self) -> &Library;
	unsafe fn get_underlying_library_mut(&mut self) -> &mut Option<Library>;
}

pub trait LibraryDrop {
	fn get_library_drop_list(&self) -> Rc<RefCell<Vec<Library>>>;
	fn on_drop(&mut self)
	where
		Self: SafeLibrary,
	{
		let library_drop_list = self.get_library_drop_list();
		unsafe {
			let lib = self.get_underlying_library_mut().take().unwrap();
			library_drop_list.borrow_mut().push(lib);
		}
	}
}

static mut LIBRARY_USAGE: Option<BTreeMap<CrateName, u32>> = None;

pub unsafe fn increment_library_usage(name: CrateName) -> u32 {
	match &mut LIBRARY_USAGE {
		Some(usage) => {
			let got = *usage.get(&name).unwrap_or(&0);
			usage.insert(name, got + 1);
			got
		}
		None => {
			LIBRARY_USAGE = Some(BTreeMap::new());
			increment_library_usage(name)
		}
	}
}

pub fn load_crate_as_library(name: CrateName, recompile: LibraryRecompile, debug: DebugMode) -> Result<libloading::Library, Box<dyn Error>> {
	let library_name = libloading::library_filename(name.get()).to_str().unwrap().to_owned().replace("-", "_");

	let usage = unsafe { increment_library_usage(name.clone()) };

	let old_path = format!("custard_dylib_cache/{}", library_name);

	let path = format!("{}{}", old_path, usage);

	let should_recompile = match recompile {
		LibraryRecompile::Recompile => true,
		LibraryRecompile::TryCached => !Path::exists(Path::new(&path)),
		_ => false,
	};

	if should_recompile {
		runtime_compile::compile(name.clone(), library_name.as_str(), debug)?;
	}

	if fs::try_exists(&old_path)? {
		fs::copy(old_path, &path)?;
	}

	info!("Instantiating new library: {}", path);
	let ret = Ok(unsafe { libloading::Library::new(path.clone()) }?);
	info!("Instantiated library: {}", path);

	ret
}
