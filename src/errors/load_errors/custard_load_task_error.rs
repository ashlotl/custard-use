use crate::identify::crate_name::CrateName;

use std::error::Error;

#[derive(Debug, thiserror::Error)]
#[error("Error loading datachunk from crate: ")]
pub struct CustardLoadTaskError {
	pub crate_name: CrateName,
	pub type_name: String,
	pub wrapped_error: Box<dyn Error>,
}
