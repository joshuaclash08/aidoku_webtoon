use aidoku::{
	helpers::uri::encode_uri_component,
	prelude::format,
	std::defaults::defaults_get,
	std::net::Request,
	std::{String, Vec},
	Filter, FilterType,
};

pub const BASE_URL: &str = "https://m.comic.naver.com";

/// User-Agent for mobile requests
pub fn get_user_agent() -> String {
	String::from(
		"Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1",
	)
}

/// Request wrapper with User-Agent and Referer headers
pub fn request(url: &str) -> Request {
	Request::get(url)
		.header("Referer", "https://comic.naver.com/")
		.header("User-Agent", &get_user_agent())
}

/// Extracts titleId from a given webtoon URL
pub fn get_title_id(url: &str) -> String {
	if let Some(pos) = url.find("titleId=") {
		let after = &url[pos + 8..];
		let id_str = after.split('&').next().unwrap_or(after);
		if url.contains("bestChallenge") {
			format!("{}-best", id_str)
		} else {
			String::from(id_str)
		}
	} else {
		String::new()
	}
}

/// Extracts episode sequence number 'no' from a viewer or detail URL
pub fn get_chapter_id(url: &str) -> String {
	if !url.contains("detail?") {
		return String::new();
	}
	let query = url.split('?').nth(1).unwrap_or("");
	for param in query.split('&') {
		let clean = param.trim_start_matches("amp;");
		if let Some(val) = clean.strip_prefix("no=") {
			let clean_val = val.split('#').next().unwrap_or(val);
			return String::from(clean_val);
		}
	}
	String::new()
}

/// Returns full list URL for a manga
pub fn get_manga_url(manga_id: &str) -> String {
	if let Some(clean_id) = manga_id.strip_suffix("-best") {
		format!("{}/bestChallenge/list?titleId={}", BASE_URL, clean_id)
	} else {
		format!("{}/webtoon/list?titleId={}", BASE_URL, manga_id)
	}
}

/// Returns full viewer URL for a chapter
pub fn get_chapter_url(chapter_id: &str, manga_id: &str) -> String {
	if let Some(clean_id) = manga_id.strip_suffix("-best") {
		format!(
			"{}/bestChallenge/detail?titleId={}&no={}",
			BASE_URL, clean_id, chapter_id
		)
	} else {
		format!(
			"{}/webtoon/detail?titleId={}&no={}",
			BASE_URL, manga_id, chapter_id
		)
	}
}

/// Extracts numeric chapter value from title like "14화 단원 요약" or "2부 8화 생존자"
/// Falls back to the URL's 'no' parameter if no number found.
pub fn extract_chapter_number(title: &str, fallback_no: f32) -> f32 {
	// Look for pattern ending with '화'
	if let Some(hwa_idx) = title.find("화") {
		let before = &title[..hwa_idx];
		let mut num_str = String::new();
		for c in before.chars().rev() {
			if c.is_ascii_digit() || c == '.' {
				num_str.push(c);
			} else if !num_str.is_empty() {
				break;
			}
		}
		if !num_str.is_empty() {
			let reversed: String = num_str.chars().rev().collect();
			if let Ok(val) = reversed.parse::<f32>() {
				return val;
			}
		}
	}
	fallback_no
}

/// Parses "25.05.26" or "2025.05.26" date into approximate Unix epoch seconds
pub fn parse_korean_date(date_str: &str) -> f64 {
	let trimmed = date_str.trim().trim_end_matches('.');
	let parts: Vec<&str> = trimmed.split('.').collect();
	if parts.len() == 3 {
		let raw_year: i64 = parts[0].parse().unwrap_or(0);
		let year: i64 = if raw_year < 100 { 2000 + raw_year } else { raw_year };
		let month: i64 = parts[1].parse().unwrap_or(1);
		let day: i64 = parts[2].parse().unwrap_or(1);

		// Days from year 1970 to given year (approximate leap years)
		let mut days = (year - 1970) * 365 + (year - 1969) / 4;
		let days_in_month = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
		for m in 0..(month - 1) as usize {
			if m < 12 {
				days += days_in_month[m];
			}
		}
		// Leap year correction if past Feb
		if month > 2 && (year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)) {
			days += 1;
		}
		days += day - 1;

		(days * 86400) as f64
	} else {
		-1.0
	}
}

/// Returns the search status as a boolean and the search string if there is one
pub fn check_for_search(filters: Vec<Filter>) -> (String, bool) {
	let mut search_string = String::new();
	let mut search = false;

	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				if let Ok(filter_value) = filter.value.as_string() {
					search_string.push_str(
						encode_uri_component(filter_value.read().to_lowercase()).as_str(),
					);
					search = true;
					break;
				}
			}
			_ => continue,
		}
	}
	(search_string, search)
}

/// Check setting for best challenge
pub fn show_best_challenge() -> bool {
	defaults_get("showBestChallenge")
		.ok()
		.and_then(|v| v.as_bool().ok())
		.unwrap_or(true)
}
