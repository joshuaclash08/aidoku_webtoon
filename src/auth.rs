use aidoku::{
	alloc::String,
	imports::defaults::{defaults_get, defaults_set, DefaultValue},
	HashMap, Result,
};

const COOKIE_KEY: &str = "naver_cookies";

/// Save cookies captured from in-app Naver login
pub fn handle_login(cookies: HashMap<String, String>) -> Result<bool> {
	if cookies.is_empty() {
		return Ok(false);
	}
	let mut cookie_str = String::new();
	let mut first = true;
	for (name, value) in cookies.iter() {
		if !first {
			cookie_str.push_str("; ");
		}
		cookie_str.push_str(name);
		cookie_str.push('=');
		cookie_str.push_str(value);
		first = false;
	}
	defaults_set(COOKIE_KEY, DefaultValue::String(cookie_str));
	Ok(true)
}

/// Check if user has authenticated Naver cookies
pub fn is_logged_in() -> bool {
	if let Some(cookie_str) = defaults_get::<String>(COOKIE_KEY) {
		!cookie_str.trim().is_empty()
	} else {
		false
	}
}

/// Retrieve formatted Cookie header string
pub fn get_cookie_header() -> Option<String> {
	defaults_get::<String>(COOKIE_KEY).filter(|s| !s.trim().is_empty())
}

/// Logout and clear saved cookies
pub fn logout() {
	defaults_set(COOKIE_KEY, DefaultValue::Null);
}
