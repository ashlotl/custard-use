use serde::Deserialize;

use std::fmt::Debug;

pub trait CustardName<'a>: Clone + Debug + Deserialize<'a> + Ord + PartialEq {
	fn new(val: String) -> Self;
	fn get(&self) -> &str;
}
