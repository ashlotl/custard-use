use std::fmt::{Debug, Formatter};

use serde::Deserialize;

#[derive(Deserialize, PartialEq)]
pub struct UnloadedDatachunk {
	pub type_name: String,
	pub deserialize_path: String,
}

impl Debug for UnloadedDatachunk {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		f.write_str("\n(field \"deserialize\" omitted)\n")?;
		f.debug_struct("UnloadedDatachunk").field("type_name", &self.type_name).finish()
	}
}
