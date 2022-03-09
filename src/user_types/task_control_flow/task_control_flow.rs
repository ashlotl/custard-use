use std::{error::Error, ops::Try};

pub enum TaskControlFlow {
	Continue,
	Err(Box<dyn Error>),
	Reload,
	ReloadCrate,
	StopAll,
	StopThis,
}

impl Try for TaskControlFlow {
	type Ok = ();
	type Error = Box<dyn Error>;

	fn into_result(self) -> Result<Self::Ok, Self::Error> {
		match self {
			Self::Err(v) => Err(v),
			_ => Ok(()),
		}
	}

	fn from_error(error: Self::Error) -> Self {
		Self::Err(error)
	}

	fn from_ok(_ok: Self::Ok) -> Self {
		Self::Continue
	}
}
