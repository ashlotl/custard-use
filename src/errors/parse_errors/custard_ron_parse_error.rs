use ron::Error;
use thiserror::Error;

#[derive(Debug, Error)]
#[error("Composition RON could not be parsed")]
pub struct CustardRonCompositionParseError {
	pub error: Error,
	pub relevant_ron: String,
}
