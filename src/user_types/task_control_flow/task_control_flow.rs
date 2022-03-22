use std::{collections::BTreeSet, error::Error, rc::Rc, sync::Arc};

use crate::identify::crate_name::CrateName;

#[derive(Clone, Debug)]
pub enum TaskControlFlow {
	Continue,
	Err(Rc<dyn Error>),
	FullReload,
	PartialReload(Arc<BTreeSet<CrateName>>),
	StopAll,
	StopThis,
}
