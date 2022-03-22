use std::{collections::BTreeSet, sync::Arc};

use crate::identify::crate_name::CrateName;

#[derive(Clone, Debug)]
pub enum InstanceControlFlow {
	Continue,
	FullReload,
	PartialReload(Arc<BTreeSet<CrateName>>),
	RecreateThreadpool,
	Stop,
}
