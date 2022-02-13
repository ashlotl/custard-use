use std::{error::Error, sync::Arc};

use crate::{
	composition::loaded::loaded_datachunk::LoadedDatachunk,
	dylib_management::safe_library::{core_crate::CoreCrate, library_type::LibraryType, load_types::DatachunkLoadFn},
	identify::{crate_name::CrateName, custard_name::CustardName},
	user_types::datachunk::Datachunk,
};

use libloading::{Library, Symbol};

#[derive(Debug)]
pub(crate) struct SafeLibrary<'a> {
	pub(crate) structure: LibraryType<'a>,
	lib: Library,
}

impl<'a> SafeLibrary<'a> {
	pub fn new(is_core: bool, name: CrateName) -> Result<Self, Box<dyn Error>> {
		let library_name = libloading::library_filename(name.get()).to_str().unwrap().to_owned();
		let path = format!("custard_dylib_cache/{}", library_name.replace("-", "_"));
		println!("Loading library: {}", path);
		let lib = unsafe { libloading::Library::new(path) }?;
		println!("done");
		let structure = if is_core { LibraryType::CoreLibrary(CoreCrate { composition: unsafe { (&*(&lib as *const Library)).get("composition".as_bytes())? } }) } else { LibraryType::UserLibrary };
		Ok(Self { structure, lib })
	}

	pub fn load_datachunk(&self, type_name: &str, deserialize_str: &str) -> Result<Arc<dyn Datachunk>, Box<dyn Error>> {
		let load_fn: Symbol<DatachunkLoadFn> = unsafe { self.lib.get(format!("__custard_datachunk__{}", type_name).as_bytes())? };
		Ok(load_fn(deserialize_str)?)
	}
}
