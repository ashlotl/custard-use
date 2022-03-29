use custard_macros::display_from_debug;

use crate::identify::crate_name::CrateName;

use std::error::Error;

#[derive(Debug, thiserror::Error)]
pub struct CustardLoadTaskError {
	pub crate_name: CrateName,
	pub type_name: String,
	pub wrapped_error: Box<dyn Error>,
}

display_from_debug!(CustardLoadTaskError);
