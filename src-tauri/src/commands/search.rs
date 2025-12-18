use crate::api::search;
use crate::models::SearchResult;

/// 搜索视频
#[tauri::command]
pub async fn search_videos(
    keyword: String,
    page: u32,
    page_size: u32,
    order: Option<String>,
) -> Result<SearchResult, String> {
    search::search_videos(&keyword, page, page_size, order.as_deref())
        .await
        .map_err(|e| e.to_user_message())
}
