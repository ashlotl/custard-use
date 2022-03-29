use custard_macros::display_from_debug;
use thiserror::Error;

use crate::identify::crate_name::CrateName;

#[derive(Debug, Error)]
/// Do not fill the composition field unless the composition is in a core crate.
pub struct CustardCompositionRequiresCoreCrateError {
	pub offending_crate: CrateName,
}

display_from_debug!(CustardCompositionRequiresCoreCrateError);
