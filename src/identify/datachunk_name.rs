use crate::identify::{crate_name::CrateName, custard_name::CustardName};

use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
pub struct FullDatachunkName {
	pub crate_name: CrateName,
	pub datachunk_name: DatachunkName,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
pub struct DatachunkName {
	name: String,
}

impl CustardName<'_> for DatachunkName {
	fn get(&self) -> &str {
		self.name.as_str()
	}
}
