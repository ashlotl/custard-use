use mopa::mopafy;

use std::fmt::Debug;

pub trait Datachunk: Debug + mopa::Any + Send + Sync {}
mopafy!(Datachunk);
