use mopa::mopafy;

use std::fmt::Debug;

pub trait Datachunk: Debug + Send + Sync + mopa::Any {}
mopafy!(Datachunk);
