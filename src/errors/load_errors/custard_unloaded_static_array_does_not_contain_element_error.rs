use std::fmt::{self, Debug, Display, Formatter};

#[derive(Clone, Debug, thiserror::Error)]
/// Some unloaded user type (`Task`, `Datachunk`) was not found statically. Static compilation-included lookups should only occur if dynamic runtime lookups fail.
pub struct CustardUnloadedStaticArrayDoesNotContainElementError<T: Debug> {
	pub offending_key: T,
}

impl<T: Debug> Display
	for CustardUnloadedStaticArrayDoesNotContainElementError<T>
{
	//TODO: this should work in display_from_debug
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		write!(f, "{:?}", self)
	}
}
