use std::fs;

pub fn get_maybe_const_string(path: &str, if_fail: &str) -> (String, bool) {
	if let Ok(v) = fs::read_to_string(path) {
		(v, true) //the file could be found
	} else {
		//the file couldn't be found. Perhaps we have a packaged appimage that forgot it?
		(if_fail.to_string(), false)
	}
}
