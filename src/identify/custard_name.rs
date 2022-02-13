use serde::Deserialize;

use std::fmt::Debug;

pub trait CustardName<'a>: Clone + Debug + Deserialize<'a> + Ord + PartialEq {
	fn get(&self) -> &str;
}

pub trait FullCustardName<'a>: Clone + Deserialize<'a> + Ord + PartialEq {
	fn get_crate_name<T>(&self) -> &T
	where
		T: CustardName<'a>;
}
