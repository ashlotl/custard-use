pub type CompositionFunctionReturn = String;

#[derive(Debug)]
pub(crate) struct CoreCrate<'a> {
	///The composition getter for the library. This usually wraps a call to `get_maybe_const_string` in `utils`.
	pub(crate) composition: libloading::Symbol<'a, fn() -> CompositionFunctionReturn>,
}
