use thiserror::Error;

/// B站 API 错误类型
#[derive(Error, Debug)]
pub enum BiliApiError {
    #[error("网络请求失败: {0}")]
    Network(#[from] reqwest::Error),

    #[error("未登录或登录已过期")]
    NotLoggedIn,

    #[error("B站返回错误: {code} - {message}")]
    ApiError { code: i32, message: String },

    #[error("请求过于频繁，请稍后再试")]
    RateLimited,

    #[error("评论失败: {0}")]
    CommentFailed(String),

    #[error("二维码已过期")]
    QrCodeExpired,

    #[error("数据解析失败: {0}")]
    ParseError(String),

    #[error("IO错误: {0}")]
    IoError(#[from] std::io::Error),

    #[error("{0}")]
    Other(String),
}

impl BiliApiError {
    /// 转换为用户友好的错误信息
    pub fn to_user_message(&self) -> String {
        match self {
            Self::NotLoggedIn => "请先登录".to_string(),
            Self::RateLimited => "操作过于频繁，请稍等几秒后重试".to_string(),
            Self::QrCodeExpired => "二维码已过期，请刷新".to_string(),
            Self::ApiError { code, message } => match *code {
                -101 => "账号未登录".to_string(),
                -111 => "csrf校验失败".to_string(),
                -400 => "请求错误".to_string(),
                -404 => "视频不存在".to_string(),
                12002 => "评论内容包含敏感词".to_string(),
                12009 => "评论发送太频繁".to_string(),
                12015 => "需要输入验证码".to_string(),
                12016 => "您的账号存在异常，请完成验证后重试".to_string(),
                12025 => "评论区已关闭".to_string(),
                _ => format!("错误 {}: {}", code, message),
            },
            _ => self.to_string(),
        }
    }
}

// 实现从 BiliApiError 到 String 的转换，方便 Tauri command 返回
impl From<BiliApiError> for String {
    fn from(err: BiliApiError) -> Self {
        err.to_user_message()
    }
}
