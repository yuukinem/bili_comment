use reqwest::Response;
use serde::Deserialize;
use serde_json::Value;
use std::time::Duration;
use tokio::time::sleep;

use super::{BiliApiError, BILI_CLIENT};
use crate::models::{SearchResult, VideoItem};

const SEARCH_URL: &str = "https://api.bilibili.com/x/web-interface/search/type";
const MAX_RETRIES: u32 = 5;
const RETRY_DELAY_MS: u64 = 500;

/// B站 API 响应结构
#[derive(Debug, Deserialize)]
struct BiliResponse<T> {
    code: i32,
    #[serde(default)]
    message: String,
    data: Option<T>,
}

/// 搜索响应数据
#[derive(Debug, Deserialize)]
struct SearchData {
    #[serde(default, rename = "numResults")]
    num_results: u32,
    #[serde(default, rename = "numPages")]
    #[allow(dead_code)]
    num_pages: u32,
    #[serde(default)]
    page: u32,
    #[serde(default)]
    pagesize: u32,
    result: Option<Vec<SearchResultItem>>,
}

/// 单个搜索结果项 - 使用宽松的类型
#[derive(Debug, Deserialize)]
struct SearchResultItem {
    #[serde(default)]
    aid: u64,
    #[serde(default)]
    bvid: String,
    #[serde(default)]
    title: String,
    #[serde(default)]
    author: String,
    #[serde(default)]
    mid: u64,
    #[serde(default)]
    pic: String,
    #[serde(default)]
    play: Value, // 可能是数字或字符串
    #[serde(default, rename = "video_review")]
    danmaku: Value, // 可能是数字或字符串
    #[serde(default)]
    pubdate: i64,
    #[serde(default)]
    duration: String,
    #[serde(default)]
    description: String,
}

/// 从 Value 提取数字
fn value_to_u64(v: &Value) -> u64 {
    match v {
        Value::Number(n) => n.as_u64().unwrap_or(0),
        Value::String(s) => s.parse().unwrap_or(0),
        _ => 0,
    }
}

/// 搜索视频
pub async fn search_videos(
    keyword: &str,
    page: u32,
    page_size: u32,
    order: Option<&str>,
) -> Result<SearchResult, BiliApiError> {
    let client = &BILI_CLIENT;
    let order = order.unwrap_or("totalrank");

    log::info!(
        "🔍 搜索视频: keyword={}, page={}, page_size={}, order={}",
        keyword,
        page,
        page_size,
        order
    );

    let mut last_error = None;

    for attempt in 1..=MAX_RETRIES {
        let headers = client.build_headers();

        let response: Response = match client
            .client()
            .get(SEARCH_URL)
            .headers(headers)
            .query(&[
                ("search_type", "video"),
                ("keyword", keyword),
                ("page", &page.to_string()),
                ("page_size", &page_size.to_string()),
                ("order", order),
            ])
            .send()
            .await
        {
            Ok(resp) => resp,
            Err(e) => {
                log::error!("❌ 搜索网络请求失败 (尝试 {}/{}): {}", attempt, MAX_RETRIES, e);
                last_error = Some(BiliApiError::from(e));
                if attempt < MAX_RETRIES {
                    sleep(Duration::from_millis(RETRY_DELAY_MS)).await;
                }
                continue;
            }
        };

        // 先获取响应文本
        let response_text = match response.text().await {
            Ok(text) => text,
            Err(e) => {
                log::error!("❌ 获取响应文本失败 (尝试 {}/{}): {}", attempt, MAX_RETRIES, e);
                last_error = Some(BiliApiError::ParseError(format!("获取响应文本失败: {}", e)));
                if attempt < MAX_RETRIES {
                    sleep(Duration::from_millis(RETRY_DELAY_MS)).await;
                }
                continue;
            }
        };

        // 检查是否返回了 HTML (风控页面)
        if response_text.trim_start().starts_with("<!DOCTYPE") || response_text.trim_start().starts_with("<html") {
            log::warn!("⚠️ B站返回了HTML页面 (风控/验证), 重试 ({}/{})", attempt, MAX_RETRIES);
            last_error = Some(BiliApiError::ParseError("B站返回了验证页面，请稍后重试".to_string()));
            if attempt < MAX_RETRIES {
                sleep(Duration::from_millis(RETRY_DELAY_MS * 2)).await; // HTML 情况等待更久
            }
            continue;
        }

        // 解析 JSON
        let resp: BiliResponse<SearchData> = match serde_json::from_str(&response_text) {
            Ok(r) => r,
            Err(e) => {
                log::error!("❌ 搜索响应解析失败 (尝试 {}/{}): {}", attempt, MAX_RETRIES, e);
                log::error!("📄 原始响应 (前300字符): {}", &response_text[..response_text.len().min(300)]);
                last_error = Some(BiliApiError::ParseError(format!("响应解析失败: {}", e)));
                if attempt < MAX_RETRIES {
                    sleep(Duration::from_millis(RETRY_DELAY_MS)).await;
                }
                continue;
            }
        };

        if resp.code != 0 {
            log::error!("❌ 搜索失败: code={}, message={}", resp.code, resp.message);
            return Err(BiliApiError::ApiError {
                code: resp.code,
                message: resp.message,
            });
        }

        let data = resp.data.ok_or_else(|| BiliApiError::ParseError("缺少数据".to_string()))?;

        let items: Vec<VideoItem> = data
            .result
            .unwrap_or_default()
            .into_iter()
            .map(|item| VideoItem {
                aid: item.aid,
                bvid: item.bvid,
                title: clean_html_tags(&item.title),
                author: item.author,
                mid: item.mid,
                pic: normalize_pic_url(&item.pic),
                play: value_to_u64(&item.play),
                danmaku: value_to_u64(&item.danmaku),
                pubdate: item.pubdate,
                duration: item.duration,
                description: item.description,
            })
            .collect();

        log::info!(
            "✅ 搜索成功: 找到 {} 条结果, 总计 {} 条, 第 {} 页{}",
            items.len(),
            data.num_results,
            data.page,
            if attempt > 1 { format!(" (第{}次尝试)", attempt) } else { String::new() }
        );

        return Ok(SearchResult {
            page: data.page,
            page_size: data.pagesize,
            total: data.num_results,
            items,
        });
    }

    // 所有重试都失败
    Err(last_error.unwrap_or_else(|| BiliApiError::Other("搜索失败".to_string())))
}

/// 清理 HTML 标签 (搜索结果中的高亮标签)
fn clean_html_tags(text: &str) -> String {
    text.replace("<em class=\"keyword\">", "")
        .replace("</em>", "")
}

/// 标准化图片 URL
fn normalize_pic_url(url: &str) -> String {
    if url.starts_with("//") {
        format!("https:{}", url)
    } else {
        url.to_string()
    }
}
