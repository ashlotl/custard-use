use crate::concurrency::fulfiller::Fulfiller;

use std::sync::Arc;

pub struct OCTaskChain {
	tasks: Vec<Arc<Fulfiller>>,
}
