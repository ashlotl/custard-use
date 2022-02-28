use std::ops::{Deref, DerefMut};

#[derive(Debug)]
#[repr(C)]
pub struct MutableArc<T: ?Sized> {
	data: *mut T,
	mutable: bool,
}

unsafe impl<T> Send for MutableArc<T> {}
unsafe impl<T> Sync for MutableArc<T> {}

impl<T: ?Sized> MutableArc<T> {
	pub fn new(data: *mut T, mutable: bool) -> Self {
		Self { data, mutable }
	}
}

impl<T> Deref for MutableArc<T> {
	type Target = T;
	fn deref(&self) -> &Self::Target {
		unsafe { &*self.data }
	}
}

impl<T> DerefMut for MutableArc<T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		if !self.mutable {
			panic!("Deref of immutable MutableArc");
		}
		unsafe { &mut *self.data }
	}
}
