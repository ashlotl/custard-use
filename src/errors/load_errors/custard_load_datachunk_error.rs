use std::error::Error;

use crate::identify::crate_name::CrateName;

use thiserror;

#[derive(Debug, thiserror::Error)]
#[error("String error: ")]
pub struct CustardFFIAmbiguousStringError {
	pub wrapped_error: String,
}

#[derive(Debug, thiserror::Error)]
#[error("Error loading datachunk from crate: ")]
pub struct CustardLoadDatachunkError {
	pub crate_name: CrateName,
	pub type_name: String,
	pub wrapped_error: Box<dyn Error>,
}
