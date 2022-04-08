use crate::identify::{crate_name::CrateName, custard_name::CustardName};

use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
pub struct FullDatachunkName {
	pub crate_name: CrateName,
	pub datachunk_name: DatachunkName,
}

impl FullDatachunkName {
	pub fn new(crate_name: String, datachunk_name: String) -> Self {
		Self {
			crate_name: CrateName::new(crate_name),
			datachunk_name: DatachunkName::new(datachunk_name),
		}
	}
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
pub struct DatachunkName {
	name: String,
}

impl CustardName<'_> for DatachunkName {
	fn new(val: String) -> Self {
		Self { name: val }
	}

	fn get(&self) -> &str {
		self.name.as_str()
	}
}
