use aidoku::{
	alloc::{format, vec, String, Vec},
	helpers::uri::encode_uri_component,
	imports::error::{AidokuError, Result},
	imports::net::Request,
	Chapter, ContentRating, DeepLinkResult, FilterValue, Listing, Manga, MangaPageResult,
	MangaStatus, Page, PageContent, PageContext, Viewer,
};

use crate::helper::*;

/// Helper to parse manga card elements from list / weekday / finish pages
fn parse_manga_cards(html: &aidoku::imports::html::Document) -> Vec<Manga> {
	let mut mangas: Vec<Manga> = Vec::new();
	let selector = "a[href*=\"/webtoon/list?titleId=\"], a[href*=\"/bestChallenge/list?titleId=\"]";
	if let Some(items) = html.select(selector) {
		for node in items {
			let href = node.attr("href").unwrap_or_default();
			let id = get_title_id(&href);
			if id.is_empty() || mangas.iter().any(|m| m.key == id) {
				continue;
			}

			let mut title = node.select_first("strong").and_then(|e| e.text()).unwrap_or_default();
			if title.is_empty() {
				title = node.select_first(".title").and_then(|e| e.text()).unwrap_or_default();
			}
			let cover = node.select_first("img").and_then(|e| e.attr("src"));
			let author = node.select_first(".desc, .author").and_then(|e| e.text());

			let full_url = if href.starts_with("http") {
				href
			} else {
				format!("{}{}", BASE_URL, href)
			};

			let authors = author.map(|a| vec![a]);

			mangas.push(Manga {
				key: id,
				cover,
				title,
				authors,
				url: Some(full_url),
				viewer: Viewer::Webtoon,
				..Default::default()
			});
		}
	}
	mangas
}

/// Helper to parse weekday list (mon, tue, wed, thu, fri, sat, sun)
fn parse_weekday_list(week: &str) -> Result<MangaPageResult> {
	let url = format!("{}/webtoon/weekday?week={}", BASE_URL, week);
	let html = request(&url)?.html()?;
	let entries = parse_manga_cards(&html);

	Ok(MangaPageResult {
		entries,
		has_next_page: false,
	})
}

/// Helper to parse finished webtoons with pagination
fn parse_finish_list(page: i32) -> Result<MangaPageResult> {
	let url = format!("{}/webtoon/finish?page={}&sort=UPDATE", BASE_URL, page);
	let html = request(&url)?.html()?;
	let entries = parse_manga_cards(&html);

	let next_btn = html.select_first("a.btn_next");
	let has_next_page = if let Some(btn) = next_btn {
		let class = btn.attr("class").unwrap_or_default();
		let href = btn.attr("href").unwrap_or_default();
		!class.contains("disabled") && !href.is_empty() && href != "#"
	} else {
		false
	};

	Ok(MangaPageResult {
		entries,
		has_next_page,
	})
}

/// Helper to parse Best Challenge series with pagination
fn parse_best_challenge_list(page: i32) -> Result<MangaPageResult> {
	if !show_best_challenge() {
		return Ok(MangaPageResult::default());
	}

	let url = format!(
		"{}/bestChallenge/genre?genre=ALL&sort=VIEW&page={}",
		BASE_URL, page
	);
	let html = request(&url)?.html()?;
	let entries = parse_manga_cards(&html);

	let next_btn = html.select_first("a.btn_next");
	let has_next_page = if let Some(btn) = next_btn {
		let class = btn.attr("class").unwrap_or_default();
		let href = btn.attr("href").unwrap_or_default();
		!class.contains("disabled") && !href.is_empty() && href != "#"
	} else {
		false
	};

	Ok(MangaPageResult {
		entries,
		has_next_page,
	})
}

/// Search manga by query or fallback to default
pub fn parse_search_manga_list(
	query: Option<String>,
	_page: i32,
	_filters: Vec<FilterValue>,
) -> Result<MangaPageResult> {
	if let Some(ref q) = query {
		if !q.trim().is_empty() {
			let encoded = encode_uri_component(q.trim());
			let url = format!("{}/search/result?keyword={}", BASE_URL, encoded);
			let html = request(&url)?.html()?;
			let entries = parse_manga_cards(&html);

			return Ok(MangaPageResult {
				entries,
				has_next_page: false,
			});
		}
	}

	// Default fallback to Monday webtoons
	parse_weekday_list("mon")
}

/// Handles all listings registered in source.json
pub fn parse_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
	match listing.id.as_str() {
		"mon" | "월요일" => parse_weekday_list("mon"),
		"tue" | "화요일" => parse_weekday_list("tue"),
		"wed" | "수요일" => parse_weekday_list("wed"),
		"thu" | "목요일" => parse_weekday_list("thu"),
		"fri" | "금요일" => parse_weekday_list("fri"),
		"sat" | "토요일" => parse_weekday_list("sat"),
		"sun" | "일요일" => parse_weekday_list("sun"),
		"completed" | "완결" => parse_finish_list(page),
		"best" | "베스트도전" => parse_best_challenge_list(page),
		"popular" | "인기순" => parse_finish_list(page),
		"update" | "업데이트순" => parse_finish_list(page),
		_ => parse_weekday_list("mon"),
	}
}

/// Parses webtoon details (title, cover, author, description, status, genre, 19+ check)
pub fn parse_manga_details(manga_id: &str, mut manga: Manga) -> Result<Manga> {
	let url = get_manga_url(manga_id);
	let html = match request(&url)?.html() {
		Ok(h) => h,
		Err(e) => return Err(e.into()),
	};

	let is_login_page = html.select_first("title")
		.and_then(|t| t.text())
		.map(|s| s.contains("NAVER 로그인") || s.contains("로그인"))
		.unwrap_or(false)
		|| html.select_first("form#frmNIDLogin").is_some();

	if is_login_page {
		manga.content_rating = ContentRating::NSFW;
		manga.tags = Some(vec![String::from("19세 이상 이용가")]);
		if manga.description.is_none() {
			manga.description = Some(String::from(
				"이 작품은 19세 이상 이용가입니다. 소스 설정(⚙️)에서 [네이버 로그인]을 완료해주세요.",
			));
		}
		manga.url = Some(url);
		manga.viewer = Viewer::Webtoon;
		return Ok(manga);
	}

	let title = html
		.select_first("meta[property=\"og:title\"]")
		.and_then(|e| e.attr("content"))
		.or_else(|| html.select_first(".info_area .title, .title_area strong").and_then(|e| e.text()));
	if let Some(t) = title {
		manga.title = t;
	}

	let cover = html
		.select_first("meta[property=\"og:image\"]")
		.and_then(|e| e.attr("content"));
	if cover.is_some() {
		manga.cover = cover;
	}

	let description = html
		.select_first("meta[property=\"og:description\"]")
		.and_then(|e| e.attr("content"))
		.or_else(|| html.select_first(".summary, .desc").and_then(|e| e.text()));
	if description.is_some() {
		manga.description = description;
	}

	let author = html
		.select_first(".author, .info_area .author, .writer, .info .author_area")
		.and_then(|e| e.text());
	if let Some(a) = author {
		manga.authors = Some(vec![a.clone()]);
		manga.artists = Some(vec![a]);
	}

	let mut tags: Vec<String> = Vec::new();
	if let Some(genre_items) = html.select(".genre dd span, .genre dd li, .tag_item") {
		for node in genre_items {
			if let Some(g_text) = node.text() {
				let trimmed = g_text.trim();
				if !trimmed.is_empty() && !tags.iter().any(|t| t == trimmed) {
					tags.push(String::from(trimmed));
				}
			}
		}
	}
	if !tags.is_empty() {
		manga.tags = Some(tags.clone());
	}

	let is_adult = tags.iter().any(|t| t.contains("19") || t.contains("성인"));
	manga.content_rating = if is_adult {
		ContentRating::NSFW
	} else {
		ContentRating::Safe
	};

	let status_text = html
		.select_first(".week_day .list_detail, .detail .week_day")
		.and_then(|e| e.text())
		.unwrap_or_default();
	manga.status = if status_text.contains("완결") {
		MangaStatus::Completed
	} else if status_text.contains("휴재") {
		MangaStatus::Hiatus
	} else {
		MangaStatus::Ongoing
	};

	manga.url = Some(url);
	manga.viewer = Viewer::Webtoon;

	Ok(manga)
}

/// Parses all chapters for a given manga by paginating through list pages
pub fn parse_chapter_list(manga_id: &str) -> Result<Vec<Chapter>> {
	let mut chapters: Vec<Chapter> = Vec::new();
	let mut page = 1;

	loop {
		let url = if let Some(clean_id) = manga_id.strip_suffix("-best") {
			format!(
				"{}/bestChallenge/list?titleId={}&sortOrder=DESC&page={}",
				BASE_URL, clean_id, page
			)
		} else {
			format!(
				"{}/webtoon/list?titleId={}&sortOrder=DESC&page={}",
				BASE_URL, manga_id, page
			)
		};

		let req = match request(&url) {
			Ok(r) => r,
			Err(_) => break,
		};
		let html = match req.html() {
			Ok(h) => h,
			Err(_) => break,
		};

		// Check for 19+ adult login redirect
		let is_login_page = html.select_first("title")
			.and_then(|t| t.text())
			.map(|s| s.contains("NAVER 로그인") || s.contains("로그인"))
			.unwrap_or(false)
			|| html.select_first("form#frmNIDLogin").is_some()
			|| html.select_first("input[name='dynamicKey']").is_some();

		if is_login_page {
			if !crate::auth::is_logged_in() {
				return Err(AidokuError::message(
					"🔒 19세 성인인증 작품입니다. 소스 설정(⚙️)에서 [네이버 로그인]을 완료해주세요.",
				));
			} else {
				return Err(AidokuError::message(
					"🔒 네이버 로그인 세션이 만료되었습니다. 소스 설정(⚙️)에서 다시 로그인해주세요.",
				));
			}
		}

		let ep_links = match html.select("a[href*=\"detail?\"]") {
			Some(l) => l,
			None => break,
		};

		let mut found_new = false;
		for node in ep_links {
			let href = node.attr("href").unwrap_or_default();
			if !href.contains("detail?") {
				continue;
			}
			let chapter_id = get_chapter_id(&href);
			if chapter_id.is_empty() || chapters.iter().any(|c| c.key == chapter_id) {
				continue;
			}
			found_new = true;

			let mut raw_title = node.select_first(".name").and_then(|e| e.text()).unwrap_or_default();
			if raw_title.is_empty() {
				raw_title = node.select_first("strong.title, .title").and_then(|e| e.text()).unwrap_or_default();
			}

			let fallback_no = chapter_id.parse::<f32>().unwrap_or(-1.0);
			let chapter_num = extract_chapter_number(&raw_title, fallback_no);
			let date_text = node.select_first(".date").and_then(|e| e.text()).unwrap_or_default();
			let date_uploaded = parse_korean_date(&date_text);

			let full_chapter_url = if href.starts_with("http") {
				href
			} else {
				format!("{}{}", BASE_URL, href)
			};

			chapters.push(Chapter {
				key: chapter_id,
				title: Some(raw_title),
				chapter_number: Some(chapter_num),
				date_uploaded,
				url: Some(full_chapter_url),
				language: Some(String::from("ko")),
				..Default::default()
			});
		}

		if !found_new {
			break;
		}

		// Check pagination button
		let next_btn = html.select_first("a.btn_next");
		if let Some(btn) = next_btn {
			let next_class = btn.attr("class").unwrap_or_default();
			let next_href = btn.attr("href").unwrap_or_default();
			if next_class.contains("disabled") || next_href.is_empty() || next_href == "#" {
				break;
			}
		} else {
			break;
		}

		page += 1;
		if page > 1000 {
			break;
		}
	}

	Ok(chapters)
}

/// Unified manga update function (details + chapters)
pub fn parse_manga_update(
	mut manga: Manga,
	needs_details: bool,
	needs_chapters: bool,
) -> Result<Manga> {
	let manga_id = manga.key.clone();
	if needs_details {
		manga = parse_manga_details(&manga_id, manga)?;
	}
	if needs_chapters {
		let chapters = parse_chapter_list(&manga_id)?;
		manga.chapters = Some(chapters);
	}
	Ok(manga)
}

/// Parses all cut images of a specific episode
pub fn parse_page_list(manga_id: &str, chapter_id: &str) -> Result<Vec<Page>> {
	let url = get_chapter_url(chapter_id, manga_id);
	let html = request(&url)?.html()?;

	let is_login_page = html.select_first("title")
		.and_then(|t| t.text())
		.map(|s| s.contains("NAVER 로그인") || s.contains("로그인"))
		.unwrap_or(false)
		|| html.select_first("form#frmNIDLogin").is_some();

	if is_login_page {
		return Err(AidokuError::message(
			"🔒 19세 성인인증이 필요합니다. 소스 설정에서 [네이버 로그인]을 해주세요.",
		));
	}

	let mut pages: Vec<Page> = Vec::new();
	let mut img_nodes = html.select("img.toon_image");
	if img_nodes.is_none() || img_nodes.as_ref().map(|l| l.is_empty()).unwrap_or(true) {
		img_nodes = html.select("div.wt_viewer img, #toon_layer img, div.toon_view_area img");
	}

	if let Some(items) = img_nodes {
		for node in items {
			let mut img_url = node.attr("data-src").unwrap_or_default();
			if img_url.is_empty() || img_url.contains("bg_transparency.png") {
				img_url = node.attr("src").unwrap_or_default();
			}
			if img_url.is_empty() || img_url.contains("bg_transparency.png") {
				continue;
			}

			pages.push(Page {
				content: PageContent::url(img_url),
				..Default::default()
			});
		}
	}

	Ok(pages)
}

/// Handles image request modification (injected Referer + User-Agent + Cookie)
pub fn parse_image_request(url: String, _context: Option<PageContext>) -> Result<Request> {
	let mut req = Request::get(&url)?
		.header("Referer", "https://comic.naver.com/")
		.header("User-Agent", &get_user_agent());
	if let Some(cookie_str) = crate::auth::get_cookie_header() {
		req = req.header("Cookie", &cookie_str);
	}
	Ok(req)
}

/// Handles deep linking for comic.naver.com URLs
pub fn parse_deep_link(url: String) -> Result<Option<DeepLinkResult>> {
	let manga_key = get_title_id(&url);
	if manga_key.is_empty() {
		return Ok(None);
	}
	let chapter_id = get_chapter_id(&url);
	if !chapter_id.is_empty() {
		Ok(Some(DeepLinkResult::Chapter {
			manga_key,
			key: chapter_id,
		}))
	} else {
		Ok(Some(DeepLinkResult::Manga {
			key: manga_key,
		}))
	}
}
