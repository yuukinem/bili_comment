use once_cell::sync::Lazy;
use parking_lot::RwLock;
use reqwest::header::{HeaderMap, HeaderValue, COOKIE, REFERER, USER_AGENT};
use reqwest::Client;
use std::sync::Arc;

use crate::models::LoginCredential;

/// B站 API 客户端
pub struct BiliClient {
    client: Client,
    credential: Arc<RwLock<Option<LoginCredential>>>,
}

/// 全局客户端实例
pub static BILI_CLIENT: Lazy<BiliClient> = Lazy::new(|| BiliClient::new());

impl BiliClient {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            credential: Arc::new(RwLock::new(None)),
        }
    }

    /// 设置登录凭证
    pub fn set_credential(&self, credential: Option<LoginCredential>) {
        let mut cred = self.credential.write();
        *cred = credential;
    }

    /// 获取登录凭证
    #[allow(dead_code)]
    pub fn get_credential(&self) -> Option<LoginCredential> {
        self.credential.read().clone()
    }

    /// 检查是否已登录
    pub fn is_logged_in(&self) -> bool {
        self.credential.read().is_some()
    }

    /// 获取 CSRF token
    pub fn get_csrf(&self) -> Option<String> {
        self.credential.read().as_ref().map(|c| c.bili_jct.clone())
    }

    /// 构建请求头
    pub fn build_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();

        headers.insert(
            USER_AGENT,
            HeaderValue::from_static(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 \
                 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
            ),
        );

        headers.insert(
            REFERER,
            HeaderValue::from_static("https://www.bilibili.com"),
        );

        if let Some(cred) = self.credential.read().as_ref() {
            if let Ok(cookie) = HeaderValue::from_str(&cred.to_cookie_string()) {
                headers.insert(COOKIE, cookie);
            }
        }

        headers
    }

    /// 获取底层 HTTP 客户端
    pub fn client(&self) -> &Client {
        &self.client
    }
}

impl Default for BiliClient {
    fn default() -> Self {
        Self::new()
    }
}
