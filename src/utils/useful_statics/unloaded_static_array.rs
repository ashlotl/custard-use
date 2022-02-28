use std::fmt::Debug;

use crate::errors::load_errors::custard_unloaded_static_array_does_not_contain_element_error::CustardUnloadedStaticArrayDoesNotContainElementError;

pub struct UnloadedStaticArray<A: Clone + Debug + PartialEq, B, const N: usize> {
	pub elems: [(A, B); N],
}

impl<A: Clone + Debug + PartialEq, B, const N: usize> UnloadedStaticArray<A, B, N> {
	pub fn get(&self, target_key: &A) -> Result<&B, CustardUnloadedStaticArrayDoesNotContainElementError<A>> {
		for (key, value) in &self.elems {
			if key == target_key {
				return Ok(value);
			}
		}
		Err(CustardUnloadedStaticArrayDoesNotContainElementError { offending_key: target_key.clone() })
	}
}
