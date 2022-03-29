use custard_macros::display_from_debug;
use ron::Error;
use thiserror::Error;

#[derive(Debug, Error)]
pub struct CustardRonCompositionParseError {
	pub error: Error,
	pub relevant_ron: String,
}

display_from_debug!(CustardRonCompositionParseError);
