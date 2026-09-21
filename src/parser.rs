use aidoku::{
	error::Result,
	prelude::*,
	std::net::Request,
	std::{String, Vec},
	Chapter, DeepLink, Filter, Listing, Manga, MangaPageResult, MangaStatus, MangaViewer, Page,
};

use crate::helper::*;

/// Parses webtoon list based on search filter or default
pub fn parse_manga_list(filters: Vec<Filter>, _page: i32) -> Result<MangaPageResult> {
	let (query, search) = check_for_search(filters);

	if search {
		let url = format!("{}/search/result?keyword={}", BASE_URL, query);
		let html = request(&url).html()?;

		let mut mangas: Vec<Manga> = Vec::new();
		for item in html.select("a[href*=\"/webtoon/list?titleId=\"]").array() {
			let node = match item.as_node() {
				Ok(n) => n,
				Err(_) => continue,
			};
			let href = node.attr("href").read();
			let id = get_title_id(&href);
			if id.is_empty() {
				continue;
			}
			if mangas.iter().any(|m| m.id == id) {
				continue;
			}

			let mut title = node.select("strong").text().read();
			if title.is_empty() {
				title = node.select(".title").text().read();
			}
			let cover = node.select("img").attr("src").read();
			let author = node.select(".desc, .author").text().read();

			mangas.push(Manga {
				id,
				cover,
				title,
				author,
				url: format!("{}{}", BASE_URL, href),
				viewer: MangaViewer::Scroll,
				..Default::default()
			});
		}

		Ok(MangaPageResult {
			manga: mangas,
			has_more: false,
		})
	} else {
		parse_weekday_list("mon")
	}
}

/// Helper to parse weekday titles (mon, tue, wed, thu, fri, sat, sun)
fn parse_weekday_list(week: &str) -> Result<MangaPageResult> {
	let url = format!("{}/webtoon/weekday?week={}", BASE_URL, week);
	let html = request(&url).html()?;

	let mut mangas: Vec<Manga> = Vec::new();
	for item in html.select("a[href*=\"/webtoon/list?titleId=\"]").array() {
		let node = match item.as_node() {
			Ok(n) => n,
			Err(_) => continue,
		};
		let href = node.attr("href").read();
		let id = get_title_id(&href);
		if id.is_empty() {
			continue;
		}
		if mangas.iter().any(|m| m.id == id) {
			continue;
		}

		let mut title = node.select("strong").text().read();
		if title.is_empty() {
			title = node.select(".title").text().read();
		}
		let cover = node.select("img").attr("src").read();
		let author = node.select(".desc, .author").text().read();

		mangas.push(Manga {
			id,
			cover,
			title,
			author,
			url: format!("{}{}", BASE_URL, href),
			viewer: MangaViewer::Scroll,
			..Default::default()
		});
	}

	Ok(MangaPageResult {
		manga: mangas,
		has_more: false,
	})
}

/// Helper to parse finished webtoons with pagination
fn parse_finish_list(page: i32) -> Result<MangaPageResult> {
	let url = format!("{}/webtoon/finish?page={}&sort=UPDATE", BASE_URL, page);
	let html = request(&url).html()?;

	let mut mangas: Vec<Manga> = Vec::new();
	for item in html.select("a[href*=\"/webtoon/list?titleId=\"]").array() {
		let node = match item.as_node() {
			Ok(n) => n,
			Err(_) => continue,
		};
		let href = node.attr("href").read();
		let id = get_title_id(&href);
		if id.is_empty() {
			continue;
		}
		if mangas.iter().any(|m| m.id == id) {
			continue;
		}

		let mut title = node.select("strong").text().read();
		if title.is_empty() {
			title = node.select(".title").text().read();
		}
		let cover = node.select("img").attr("src").read();
		let author = node.select(".desc, .author").text().read();

		mangas.push(Manga {
			id,
			cover,
			title,
			author,
			url: format!("{}{}", BASE_URL, href),
			viewer: MangaViewer::Scroll,
			..Default::default()
		});
	}

	let next_btn = html.select("a.btn_next").first();
	let next_class = next_btn.attr("class").read();
	let next_href = next_btn.attr("href").read();
	let has_more = !next_class.contains("disabled") && !next_href.is_empty() && next_href != "#";

	Ok(MangaPageResult {
		manga: mangas,
		has_more,
	})
}

/// Helper to parse Best Challenge series with pagination
fn parse_best_challenge_list(page: i32) -> Result<MangaPageResult> {
	if !show_best_challenge() {
		return Ok(MangaPageResult {
			..Default::default()
		});
	}

	let url = format!(
		"{}/bestChallenge/genre?genre=ALL&sort=VIEW&page={}",
		BASE_URL, page
	);
	let html = request(&url).html()?;

	let mut mangas: Vec<Manga> = Vec::new();
	for item in html.select("a[href*=\"/bestChallenge/list?titleId=\"]").array() {
		let node = match item.as_node() {
			Ok(n) => n,
			Err(_) => continue,
		};
		let href = node.attr("href").read();
		let id = get_title_id(&href);
		if id.is_empty() {
			continue;
		}
		if mangas.iter().any(|m| m.id == id) {
			continue;
		}

		let mut title = node.select("strong").text().read();
		if title.is_empty() {
			title = node.select(".title").text().read();
		}
		let cover = node.select("img").attr("src").read();
		let author = node.select(".desc, .author").text().read();

		mangas.push(Manga {
			id,
			cover,
			title,
			author,
			url: format!("{}{}", BASE_URL, href),
			viewer: MangaViewer::Scroll,
			..Default::default()
		});
	}

	let next_btn = html.select("a.btn_next").first();
	let next_class = next_btn.attr("class").read();
	let next_href = next_btn.attr("href").read();
	let has_more = !next_class.contains("disabled") && !next_href.is_empty() && next_href != "#";

	Ok(MangaPageResult {
		manga: mangas,
		has_more,
	})
}

/// Handles all listings registered in source.json
pub fn parse_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
	match listing.name.as_str() {
		"월요일" => parse_weekday_list("mon"),
		"화요일" => parse_weekday_list("tue"),
		"수요일" => parse_weekday_list("wed"),
		"목요일" => parse_weekday_list("thu"),
		"금요일" => parse_weekday_list("fri"),
		"토요일" => parse_weekday_list("sat"),
		"일요일" => parse_weekday_list("sun"),
		"완결" => parse_finish_list(page),
		"베스트도전" => parse_best_challenge_list(page),
		"인기순" => parse_finish_list(page),
		"업데이트순" => parse_finish_list(page),
		_ => parse_weekday_list("mon"),
	}
}

/// Parses webtoon details (title, cover, author, description, status, genre)
pub fn parse_manga_details(manga_id: String) -> Result<Manga> {
	let url = get_manga_url(&manga_id);
	let html = request(&url).html()?;

	let mut title = html
		.select("meta[property=\"og:title\"]")
		.first()
		.attr("content")
		.read();
	if title.is_empty() {
		title = html.select(".info_area .title, .title_area strong").text().read();
	}

	let cover = html
		.select("meta[property=\"og:image\"]")
		.first()
		.attr("content")
		.read();

	let mut description = html
		.select("meta[property=\"og:description\"]")
		.first()
		.attr("content")
		.read();
	if description.is_empty() {
		description = html.select(".summary, .desc").text().read();
	}

	let author = html
		.select(".author, .info_area .author, .writer, .info .author_area")
		.text()
		.read();

	let mut categories: Vec<String> = Vec::new();
	for genre in html.select(".genre, .tag_item, .sub_info span").array() {
		if let Ok(genre_node) = genre.as_node() {
			let g_text = genre_node.text().read();
			let trimmed = g_text.trim();
			if !trimmed.is_empty() && !categories.contains(&String::from(trimmed)) {
				categories.push(String::from(trimmed));
			}
		}
	}

	let page_text = html.text().read();
	let status = if page_text.contains("완결") {
		MangaStatus::Completed
	} else if page_text.contains("휴재") {
		MangaStatus::Hiatus
	} else {
		MangaStatus::Ongoing
	};

	Ok(Manga {
		id: manga_id,
		cover,
		title,
		author: author.clone(),
		artist: author,
		description,
		url,
		categories,
		status,
		viewer: MangaViewer::Scroll,
		..Default::default()
	})
}

/// Parses all chapters for a given manga by paginating through list pages
pub fn parse_chapter_list(manga_id: String) -> Result<Vec<Chapter>> {
	let mut chapters: Vec<Chapter> = Vec::new();
	let mut page = 1;

	loop {
		let url = if let Some(clean_id) = manga_id.strip_suffix("-best") {
			format!(
				"{}/bestChallenge/list?titleId={}&page={}",
				BASE_URL, clean_id, page
			)
		} else {
			format!("{}/webtoon/list?titleId={}&page={}", BASE_URL, manga_id, page)
		};

		let html = match request(&url).html() {
			Ok(h) => h,
			Err(_) => break,
		};

		let ep_links = html.select("a[href*=\"no=\"]").array();
		if ep_links.is_empty() {
			break;
		}

		let mut found_new = false;
		for item in ep_links {
			let node = match item.as_node() {
				Ok(n) => n,
				Err(_) => continue,
			};
			let href = node.attr("href").read();
			let chapter_id = get_chapter_id(&href);
			if chapter_id.is_empty() {
				continue;
			}

			// Deduplicate
			if chapters.iter().any(|c| c.id == chapter_id) {
				continue;
			}
			found_new = true;

			let mut raw_title = node.select(".name").text().read();
			if raw_title.is_empty() {
				raw_title = node.select("strong.title, .title").text().read();
			}

			let fallback_no = chapter_id.parse::<f32>().unwrap_or(-1.0);
			let chapter_num = extract_chapter_number(&raw_title, fallback_no);
			let date_text = node.select(".date").text().read();
			let date_updated = parse_korean_date(&date_text);

			let full_chapter_url = if href.starts_with("http") {
				href
			} else {
				format!("{}{}", BASE_URL, href)
			};

			chapters.push(Chapter {
				id: chapter_id,
				title: raw_title,
				volume: -1.0,
				chapter: chapter_num,
				date_updated,
				url: full_chapter_url,
				lang: String::from("ko"),
				..Default::default()
			});
		}

		if !found_new {
			break;
		}

		// Check pagination button
		let next_btn = html.select("a.btn_next").first();
		let next_class = next_btn.attr("class").read();
		let next_href = next_btn.attr("href").read();

		if next_class.contains("disabled") || next_href.is_empty() || next_href == "#" {
			break;
		}

		page += 1;
		if page > 1000 {
			break;
		}
	}

	Ok(chapters)
}

/// Parses all cut images of a specific episode
pub fn parse_page_list(manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	let url = get_chapter_url(&chapter_id, &manga_id);
	let html = request(&url).html()?;

	let mut pages: Vec<Page> = Vec::new();
	let mut img_nodes = html.select("img.toon_image").array();
	if img_nodes.is_empty() {
		img_nodes = html.select("div.wt_viewer img, #toon_layer img").array();
	}

	for (index, item) in img_nodes.into_iter().enumerate() {
		let node = match item.as_node() {
			Ok(n) => n,
			Err(_) => continue,
		};

		let mut img_url = node.attr("data-src").read();
		if img_url.is_empty() || img_url.contains("bg_transparency.png") {
			img_url = node.attr("src").read();
		}
		if img_url.is_empty() || img_url.contains("bg_transparency.png") {
			continue;
		}

		pages.push(Page {
			index: index as i32,
			url: img_url,
			..Default::default()
		});
	}

	Ok(pages)
}

/// Injects Referer and User-Agent headers to prevent 403 Forbidden
pub fn modify_image_request(request: Request) {
	request
		.header("Referer", "https://comic.naver.com/")
		.header("User-Agent", &get_user_agent());
}

/// Handles deep linking for comic.naver.com URLs
pub fn handle_url(url: String) -> Result<DeepLink> {
	let title_id = get_title_id(&url);
	if !title_id.is_empty() {
		let manga = parse_manga_details(title_id)?;
		Ok(DeepLink {
			manga: Some(manga),
			chapter: None,
		})
	} else {
		Ok(DeepLink {
			manga: None,
			chapter: None,
		})
	}
}
