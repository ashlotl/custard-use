use std::{error::Error, sync::Arc};

use crate::user_types::datachunk::Datachunk;

pub type DatachunkLoadFn = fn(&str) -> Result<Arc<dyn Datachunk>, Box<dyn Error>>;
