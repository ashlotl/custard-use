use crate::{composition::unloaded::unloaded_datachunk::UnloadedDatachunk, dylib_management::safe_library::safe_library::SafeLibrary, user_types::datachunk::Datachunk};

use std::{error::Error, sync::Arc};

#[derive(Debug)]
pub(crate) struct LoadedDatachunk {
	user_data: Arc<dyn Datachunk>,
}

impl LoadedDatachunk {
	pub fn new(unloaded_datachunk: &UnloadedDatachunk, library: &SafeLibrary) -> Result<Self, Box<dyn Error>> {
		Ok(Self { user_data: library.load_datachunk(unloaded_datachunk.type_name.as_str(), unloaded_datachunk.deserialize.as_str())? })
	}
}
