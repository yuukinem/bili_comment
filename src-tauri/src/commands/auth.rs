use crate::api::login;
use crate::models::{LoginPollResult, QrCodeData, UserInfo};

/// 获取登录二维码
#[tauri::command]
pub async fn get_login_qrcode() -> Result<QrCodeData, String> {
    login::get_qrcode().await.map_err(|e| e.to_user_message())
}

/// 轮询登录状态
#[tauri::command]
pub async fn poll_login_status(qrcode_key: String) -> Result<LoginPollResult, String> {
    login::poll_qrcode_status(&qrcode_key)
        .await
        .map_err(|e| e.to_user_message())
}

/// 获取当前用户信息
#[tauri::command]
pub async fn get_user_info() -> Result<Option<UserInfo>, String> {
    login::get_user_info()
        .await
        .map_err(|e| e.to_user_message())
}

/// 退出登录
#[tauri::command]
pub fn logout() {
    login::logout();
}

/// 检查登录状态
#[tauri::command]
pub async fn check_login_valid() -> Result<bool, String> {
    match login::get_user_info().await {
        Ok(Some(_)) => Ok(true),
        Ok(None) => Ok(false),
        Err(_) => Ok(false),
    }
}
