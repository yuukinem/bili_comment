use reqwest::Response;
use serde::Deserialize;
use std::time::Duration;
use tokio::time::sleep;

use super::{BiliApiError, BILI_CLIENT};
use crate::models::CommentResult;

const COMMENT_ADD_URL: &str = "https://api.bilibili.com/x/v2/reply/add";

/// 评论间隔时间 (秒)
const COMMENT_INTERVAL_SECS: u64 = 5;

/// 安全截取字符串 (处理中文等多字节字符)
fn truncate_str(s: &str, max_chars: usize) -> String {
    let char_count = s.chars().count();
    if char_count > max_chars {
        format!("{}...", s.chars().take(max_chars).collect::<String>())
    } else {
        s.to_string()
    }
}

/// B站 API 响应结构
#[derive(Debug, Deserialize)]
struct BiliResponse<T> {
    code: i32,
    #[serde(default)]
    message: String,
    data: Option<T>,
}

/// 评论响应数据
#[derive(Debug, Deserialize)]
struct CommentData {
    rpid: Option<u64>,
}

/// 发送评论
pub async fn send_comment(aid: u64, content: &str) -> Result<CommentResult, BiliApiError> {
    let client = &BILI_CLIENT;

    if !client.is_logged_in() {
        log::warn!("⚠️ 发送评论失败: 用户未登录");
        return Err(BiliApiError::NotLoggedIn);
    }

    let csrf = client
        .get_csrf()
        .ok_or_else(|| BiliApiError::NotLoggedIn)?;

    let headers = client.build_headers();

    let content_preview = truncate_str(content, 30);
    log::info!("💬 发送评论: aid={}, 内容=\"{}\"", aid, content_preview);

    let params = [
        ("oid", aid.to_string()),
        ("type", "1".to_string()), // 1 = 视频
        ("message", content.to_string()),
        ("csrf", csrf),
    ];

    let response: Response = client
        .client()
        .post(COMMENT_ADD_URL)
        .headers(headers)
        .form(&params)
        .send()
        .await?;

    let resp: BiliResponse<CommentData> = response.json().await?;

    if resp.code != 0 {
        log::error!(
            "❌ 评论失败: aid={}, code={}, message={}",
            aid,
            resp.code,
            resp.message
        );
        return Ok(CommentResult {
            success: false,
            rpid: None,
            error_msg: Some(
                BiliApiError::ApiError {
                    code: resp.code,
                    message: resp.message,
                }
                .to_user_message(),
            ),
        });
    }

    let rpid = resp.data.and_then(|d| d.rpid);
    log::info!("✅ 评论成功: aid={}, rpid={:?}", aid, rpid);

    Ok(CommentResult {
        success: true,
        rpid,
        error_msg: None,
    })
}

/// 发送评论 (带频率限制)
pub async fn send_comment_with_rate_limit(
    aid: u64,
    content: &str,
) -> Result<CommentResult, BiliApiError> {
    log::debug!("⏳ 等待 {} 秒后发送评论...", COMMENT_INTERVAL_SECS);
    // 先等待间隔时间
    sleep(Duration::from_secs(COMMENT_INTERVAL_SECS)).await;
    send_comment(aid, content).await
}

/// 获取评论间隔时间
pub fn get_comment_interval() -> u64 {
    COMMENT_INTERVAL_SECS
}
