use std::error::Error;

use crate::identify::crate_name::CrateName;

use custard_macros::display_from_debug;
use thiserror;

#[derive(Debug, thiserror::Error)]
pub struct CustardLoadDatachunkError {
	pub crate_name: CrateName,
	pub type_name: String,
	pub wrapped_error: Box<dyn Error>,
}

display_from_debug!(CustardLoadDatachunkError);
