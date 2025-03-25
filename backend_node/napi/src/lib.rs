use napi::bindgen_prelude::*;
use napi_derive::napi;
use scraper::{Html, Selector};
use std::collections::HashSet;

#[napi]
fn get_media_urls(
    page: String,
    absolute_only: bool,
    domain: Option<String>,
) -> Result<Vec<String>> {
    // Разбираем HTML-контент
    let document = Html::parse_document(&page);
    let selector = Selector::parse("img, video").unwrap();

    // Создаем множество для хранения URL
    let mut urls = HashSet::new();

    for media in document.select(&selector) {
        if let Some(src) = media.value().attr("src") {
            // Проверяем, является ли ссылка абсолютной
            if !absolute_only || src.starts_with("http://") || src.starts_with("https://") {
                // Добавляем домен к относительным ссылкам
                if let Some(ref domain) = domain {
                    if !src.starts_with("http") {
                        urls.insert(format!(
                            "{}/{}",
                            domain.trim_end_matches('/'),
                            src.trim_start_matches('/')
                        ));
                    } else {
                        urls.insert(src.to_string());
                    }
                } else {
                    urls.insert(src.to_string());
                }
            }
        }
    }

    // Конвертируем HashSet в вектор строк
    Ok(urls.into_iter().collect())
}
