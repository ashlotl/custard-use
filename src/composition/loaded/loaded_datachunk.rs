use crate::{
	composition::unloaded::unloaded_datachunk::UnloadedDatachunk,
	dylib_management::safe_library::{core_library::CoreLibrary, user_library::UserLibrary},
	user_types::datachunk::Datachunk,
};

use std::error::Error;

#[derive(Debug)]
pub(crate) struct LoadedDatachunk {
	user_data: Box<dyn Datachunk>,
}

impl LoadedDatachunk {
	pub fn new(unloaded_datachunk: &UnloadedDatachunk, user_library: &UserLibrary, core_library: &CoreLibrary) -> Result<Self, Box<dyn Error>> {
		let deserialize_str = (core_library.symbols.as_ref().unwrap().unloaded_datachunk_contents)(Box::new(unloaded_datachunk.deserialize_path.clone())).into_rust()?;

		let ret = Ok(Self { user_data: user_library.load_datachunk(unloaded_datachunk.type_name.as_str(), deserialize_str.as_str())? });
		ret
	}
}
