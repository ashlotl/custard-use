pub(super) struct UserCrate<'a> {
	///The composition getter for the library. This usually wraps a call to `get_maybe_const_string` in `utils`.
	pub(crate) composition: libloading::Symbol<'a, fn() -> String>,
}
