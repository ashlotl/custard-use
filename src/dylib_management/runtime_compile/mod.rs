use std::{error::Error, fs, process::Command};

use crate::identify::{crate_name::CrateName, custard_name::CustardName};

pub fn compile(name: CrateName, library_name: &str, debug: bool) -> Result<(), Box<dyn Error>> {
	let mut handle = Command::new("cargo").arg("build").arg("-p").arg(name.get()).spawn()?;
	handle.wait()?;
	let debug_release = if debug { "debug" } else { "release" };
	fs::copy(format!("target/{}/{}", debug_release, library_name), format!("custard_dylib_cache/{}", library_name))?;
	Ok(())
}
