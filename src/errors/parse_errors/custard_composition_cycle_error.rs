use custard_macros::display_from_debug;
use thiserror::Error;

use crate::identify::crate_name::CrateName;

#[derive(Debug, Error)]
pub struct CustardCompositionCycleError {
	pub offending_crate: Option<CrateName>,
}

display_from_debug!(CustardCompositionCycleError);
