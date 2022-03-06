use std::fmt::Debug;

#[derive(Clone, Debug, thiserror::Error)]
#[error("failed to find unloaded datachunk/task element")]
pub struct CustardUnloadedStaticArrayDoesNotContainElementError<T: Debug> {
	pub offending_key: T,
}
