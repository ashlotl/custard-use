use std::{error::Error, rc::Rc};

#[derive(Clone, Debug)]
pub enum TaskControlFlow {
	Continue,
	Err(Rc<dyn Error>),
	FullReload,
	PartialReload,
	StopAll,
	StopThis,
}
