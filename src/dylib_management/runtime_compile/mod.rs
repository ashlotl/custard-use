use std::{error::Error, fs, process::Command};

use crate::{
	dylib_management::safe_library::safe_library::DebugMode,
	identify::{crate_name::CrateName, custard_name::CustardName},
};

pub fn compile(name: CrateName, library_name: &str, _debug: DebugMode) -> Result<(), Box<dyn Error>> {
	let mut handle = Command::new("cargo").arg("build").arg("-p").arg(name.get()).spawn()?;
	handle.wait()?;
	let debug_release = if let _debug = DebugMode::Debug { "debug" } else { "release" };
	fs::copy(format!("target/{}/{}", debug_release, library_name), format!("custard_dylib_cache/{}", library_name))?;
	Ok(())
}
