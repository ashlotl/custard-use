use std::sync::{Mutex, MutexGuard};

#[derive(Debug)]
pub struct PossiblyPoisonedMutex<T: ?Sized> {
	inner: Mutex<T>,
}

impl<T: ?Sized> PossiblyPoisonedMutex<T> {
	pub fn new(inner: Mutex<T>) -> Self
	where
		T: Sized,
	{
		Self { inner }
	}

	pub fn lock(&self) -> MutexGuard<T> {
		match self.inner.lock() {
			Ok(v) => v,
			Err(e) => e.into_inner(),
		}
	}
}
