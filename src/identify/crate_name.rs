use crate::identify::custard_name::CustardName;

use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
pub struct CrateName {
	name: String,
}

impl CustardName<'_> for CrateName {
	fn new(val: String) -> Self {
		Self { name: val }
	}

	fn get(&self) -> &str {
		self.name.as_str()
	}
}
