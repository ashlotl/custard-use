use crate::dylib_management::safe_library::core_crate::CoreCrate;

#[derive(Debug)]
pub(crate) enum LibraryType<'a> {
	CoreLibrary(CoreCrate<'a>),
	UserLibrary,
}
