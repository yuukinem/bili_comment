use serde::{Deserialize, Serialize};

/// 用户信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub mid: u64,
    pub uname: String,
    pub face: String,
    pub is_login: bool,
}

/// 登录凭证 (需要加密存储)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginCredential {
    pub sessdata: String,
    pub bili_jct: String,
    pub dedeuserid: String,
    pub expires_at: i64,
}

impl LoginCredential {
    /// 转换为 Cookie 字符串
    pub fn to_cookie_string(&self) -> String {
        format!(
            "SESSDATA={}; bili_jct={}; DedeUserID={}",
            self.sessdata, self.bili_jct, self.dedeuserid
        )
    }
}

/// 二维码数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QrCodeData {
    pub url: String,
    pub qrcode_key: String,
    pub image_base64: String,
}

/// 登录轮询结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginPollResult {
    pub status: LoginStatus,
    pub message: String,
}

/// 登录状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum LoginStatus {
    Waiting,
    Scanned,
    Confirmed,
    Expired,
    Error,
}

/// 视频信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoItem {
    pub aid: u64,
    pub bvid: String,
    pub title: String,
    pub author: String,
    pub mid: u64,
    pub pic: String,
    pub play: u64,
    pub danmaku: u64,
    pub pubdate: i64,
    pub duration: String,
    pub description: String,
}

/// 搜索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub page: u32,
    pub page_size: u32,
    pub total: u32,
    pub items: Vec<VideoItem>,
}

/// 评论模板
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentTemplate {
    pub id: String,
    pub name: String,
    pub content: String,
    pub created_at: i64,
    pub updated_at: i64,
}

/// 评论任务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentTask {
    pub id: String,
    pub video: VideoItem,
    pub content: String,
    pub status: TaskStatus,
    pub error_msg: Option<String>,
    pub created_at: i64,
    pub completed_at: Option<i64>,
}

/// 任务状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Pending,
    Running,
    Success,
    Failed,
    Cancelled,
}

/// 评论结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentResult {
    pub success: bool,
    pub rpid: Option<u64>,
    pub error_msg: Option<String>,
}

/// 批量任务状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchStatus {
    pub batch_id: String,
    pub total: usize,
    pub completed: usize,
    pub success: usize,
    pub failed: usize,
    pub tasks: Vec<CommentTask>,
}
