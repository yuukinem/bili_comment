use base64::{engine::general_purpose::STANDARD, Engine};
use image::Luma;
use qrcode::QrCode;
use reqwest::Response;
use serde::Deserialize;

use super::{BiliApiError, BILI_CLIENT};
use crate::models::{LoginCredential, LoginPollResult, LoginStatus, QrCodeData, UserInfo};

const QRCODE_GENERATE_URL: &str =
    "https://passport.bilibili.com/x/passport-login/web/qrcode/generate";
const QRCODE_POLL_URL: &str = "https://passport.bilibili.com/x/passport-login/web/qrcode/poll";
const USER_INFO_URL: &str = "https://api.bilibili.com/x/web-interface/nav";

/// B站 API 响应结构
#[derive(Debug, Deserialize)]
struct BiliResponse<T> {
    code: i32,
    #[serde(default)]
    message: String,
    data: Option<T>,
}

/// 二维码生成响应
#[derive(Debug, Deserialize)]
struct QrCodeGenerateData {
    url: String,
    qrcode_key: String,
}

/// 登录轮询响应
#[derive(Debug, Deserialize)]
struct QrCodePollData {
    #[serde(default)]
    url: String,
    #[serde(default)]
    #[allow(dead_code)]
    refresh_token: String,
    #[serde(default)]
    #[allow(dead_code)]
    timestamp: i64,
    code: i32,
    #[serde(default)]
    message: String,
}

/// 用户信息响应
#[derive(Debug, Deserialize)]
struct NavData {
    #[serde(rename = "isLogin")]
    is_login: bool,
    mid: Option<u64>,
    uname: Option<String>,
    face: Option<String>,
}

/// 获取登录二维码
pub async fn get_qrcode() -> Result<QrCodeData, BiliApiError> {
    let client = &BILI_CLIENT;
    let headers = client.build_headers();

    let response: Response = client
        .client()
        .get(QRCODE_GENERATE_URL)
        .headers(headers)
        .send()
        .await?;

    let resp: BiliResponse<QrCodeGenerateData> = response.json().await?;

    if resp.code != 0 {
        return Err(BiliApiError::ApiError {
            code: resp.code,
            message: resp.message,
        });
    }

    let data = resp.data.ok_or_else(|| BiliApiError::ParseError("缺少数据".to_string()))?;

    // 生成二维码图片
    let code = QrCode::new(data.url.as_bytes())
        .map_err(|e| BiliApiError::Other(format!("二维码生成失败: {}", e)))?;

    let image = code.render::<Luma<u8>>().build();

    // 转换为 PNG 并 base64 编码
    let mut png_bytes: Vec<u8> = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut png_bytes);
    image
        .write_to(&mut cursor, image::ImageFormat::Png)
        .map_err(|e| BiliApiError::Other(format!("图片编码失败: {}", e)))?;

    let image_base64 = format!("data:image/png;base64,{}", STANDARD.encode(&png_bytes));

    Ok(QrCodeData {
        url: data.url,
        qrcode_key: data.qrcode_key,
        image_base64,
    })
}

/// 轮询二维码登录状态
pub async fn poll_qrcode_status(qrcode_key: &str) -> Result<LoginPollResult, BiliApiError> {
    let client = &BILI_CLIENT;
    let headers = client.build_headers();

    let response: Response = client
        .client()
        .get(QRCODE_POLL_URL)
        .headers(headers)
        .query(&[("qrcode_key", qrcode_key)])
        .send()
        .await?;

    // 获取 Set-Cookie 头
    let cookies: Vec<(String, String)> = response
        .cookies()
        .map(|c| (c.name().to_string(), c.value().to_string()))
        .collect();

    let body: BiliResponse<QrCodePollData> = response.json().await?;

    if body.code != 0 {
        return Err(BiliApiError::ApiError {
            code: body.code,
            message: body.message,
        });
    }

    let data = body.data.ok_or_else(|| BiliApiError::ParseError("缺少数据".to_string()))?;

    // 根据 code 判断状态
    let (status, message) = match data.code {
        0 => {
            // 登录成功，解析 Cookie
            if let Some(credential) = parse_login_cookies(&cookies, &data.url) {
                client.set_credential(Some(credential.clone()));
                // 保存凭证
                if let Err(e) = crate::storage::credential::save_credential(&credential) {
                    log::error!("保存凭证失败: {}", e);
                }
            }
            (LoginStatus::Confirmed, "登录成功".to_string())
        }
        86038 => (LoginStatus::Expired, "二维码已过期".to_string()),
        86090 => (LoginStatus::Scanned, "已扫码，请在手机上确认".to_string()),
        86101 => (LoginStatus::Waiting, "等待扫码".to_string()),
        _ => (LoginStatus::Error, data.message),
    };

    Ok(LoginPollResult { status, message })
}

/// 从 Cookie 和 URL 中解析登录凭证
fn parse_login_cookies(cookies: &[(String, String)], url: &str) -> Option<LoginCredential> {
    let mut sessdata = None;
    let mut bili_jct = None;
    let mut dedeuserid = None;

    for (name, value) in cookies {
        match name.as_str() {
            "SESSDATA" => sessdata = Some(value.clone()),
            "bili_jct" => bili_jct = Some(value.clone()),
            "DedeUserID" => dedeuserid = Some(value.clone()),
            _ => {}
        }
    }

    // 也尝试从 URL 参数中解析
    if let Ok(parsed_url) = url::Url::parse(url) {
        for (key, value) in parsed_url.query_pairs() {
            match key.as_ref() {
                "SESSDATA" => sessdata = sessdata.or(Some(value.to_string())),
                "bili_jct" => bili_jct = bili_jct.or(Some(value.to_string())),
                "DedeUserID" => dedeuserid = dedeuserid.or(Some(value.to_string())),
                _ => {}
            }
        }
    }

    if let (Some(sessdata), Some(bili_jct), Some(dedeuserid)) = (sessdata, bili_jct, dedeuserid) {
        Some(LoginCredential {
            sessdata,
            bili_jct,
            dedeuserid,
            expires_at: chrono::Utc::now().timestamp() + 30 * 24 * 3600, // 30天后过期
        })
    } else {
        None
    }
}

/// 获取当前用户信息
pub async fn get_user_info() -> Result<Option<UserInfo>, BiliApiError> {
    let client = &BILI_CLIENT;

    if !client.is_logged_in() {
        return Ok(None);
    }

    let headers = client.build_headers();

    let response: Response = client
        .client()
        .get(USER_INFO_URL)
        .headers(headers)
        .send()
        .await?;

    let resp: BiliResponse<NavData> = response.json().await?;

    if resp.code != 0 {
        return Err(BiliApiError::ApiError {
            code: resp.code,
            message: resp.message,
        });
    }

    let data = resp.data.ok_or_else(|| BiliApiError::ParseError("缺少数据".to_string()))?;

    if !data.is_login {
        return Ok(None);
    }

    Ok(Some(UserInfo {
        mid: data.mid.unwrap_or(0),
        uname: data.uname.unwrap_or_default(),
        face: data.face.unwrap_or_default(),
        is_login: true,
    }))
}

/// 退出登录
pub fn logout() {
    BILI_CLIENT.set_credential(None);
    let _ = crate::storage::credential::delete_credential();
}

/// 初始化时加载已保存的凭证
pub fn init_credential() {
    if let Ok(Some(credential)) = crate::storage::credential::load_credential() {
        // 检查是否过期
        if credential.expires_at > chrono::Utc::now().timestamp() {
            BILI_CLIENT.set_credential(Some(credential));
            log::info!("已加载保存的登录凭证");
        } else {
            log::info!("保存的登录凭证已过期");
            let _ = crate::storage::credential::delete_credential();
        }
    }
}
