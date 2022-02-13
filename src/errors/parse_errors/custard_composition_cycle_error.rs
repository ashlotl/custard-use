use thiserror::Error;

use crate::identify::crate_name::CrateName;

#[derive(Debug, Error)]
#[error("Composition has a dependency cycle")]
pub struct CustardCompositionCycleError {
	pub offending_crate: Option<CrateName>,
}
