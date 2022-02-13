use crate::identify::custard_name::CustardName;

use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
pub struct CrateName {
	name: String,
}

impl CustardName<'_> for CrateName {
	fn get(&self) -> &str {
		self.name.as_str()
	}
}
