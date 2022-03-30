use mopa::mopafy;

use std::fmt::Debug;

use crate::utils::mutable_arc::MutableArc;

pub type DatachunkObject = MutableArc<dyn Datachunkable>;

pub trait Datachunkable: Debug + mopa::Any + Send + Sync {}
mopafy!(Datachunkable);
