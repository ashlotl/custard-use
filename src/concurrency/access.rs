use crate::identify::datachunk_name::FullDatachunkName;

use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Access {
	pub of: FullDatachunkName,
	pub mut_immut: AccessType,
}

#[derive(PartialEq, Clone, Copy, Debug, Deserialize)]
pub enum AccessType {
	ImmutableAccess,
	MutableAccess,
}

impl AccessType {
	pub fn commensurable(&self, other: &Self) -> bool {
		if self == &Self::MutableAccess && other == &Self::MutableAccess {
			return false;
		}
		true
	}
}
