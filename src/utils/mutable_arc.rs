use std::sync::Arc;

#[derive(Debug)]
pub struct MutableArc<T: ?Sized> {
	data: Arc<T>,
}

impl<T: ?Sized> Clone for MutableArc<T> {
	fn clone(&self) -> Self {
		Self {
			data: self.data.clone(),
		}
	}
}

impl<T> MutableArc<T>
where
	T: ?Sized,
{
	pub fn new(data: Arc<T>) -> Self {
		Self { data }
	}

	pub fn get(&self) -> &T {
		&self.data
	}

	pub unsafe fn get_mut(&self) -> &mut T {
		&mut *(Arc::as_ptr(&self.data) as *mut T)
	}
}
