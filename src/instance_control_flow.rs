use crate::identify::crate_name::CrateName;

use std::{collections::BTreeSet, sync::Arc};

#[derive(Clone, Debug)]
pub enum InstanceControlFlow {
	Continue,
	FullReload,
	PartialReload(Arc<BTreeSet<CrateName>>),
	RecreateThreadpool,
	Stop,
}
