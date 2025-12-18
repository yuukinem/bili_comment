use std::fs;
use std::path::PathBuf;

use crate::models::LoginCredential;

use super::{ensure_dir, get_app_data_dir};

/// 获取凭证文件路径
fn get_credential_path() -> PathBuf {
    get_app_data_dir().join("credential.json")
}

/// 保存登录凭证
pub fn save_credential(credential: &LoginCredential) -> Result<(), String> {
    let path = get_credential_path();
    ensure_dir(&path).map_err(|e| format!("创建目录失败: {}", e))?;

    let json = serde_json::to_string_pretty(credential)
        .map_err(|e| format!("序列化失败: {}", e))?;

    fs::write(&path, json).map_err(|e| format!("写入文件失败: {}", e))?;

    log::info!("凭证已保存到 {:?}", path);
    Ok(())
}

/// 加载登录凭证
pub fn load_credential() -> Result<Option<LoginCredential>, String> {
    let path = get_credential_path();

    if !path.exists() {
        return Ok(None);
    }

    let json = fs::read_to_string(&path).map_err(|e| format!("读取文件失败: {}", e))?;

    let credential: LoginCredential =
        serde_json::from_str(&json).map_err(|e| format!("解析失败: {}", e))?;

    Ok(Some(credential))
}

/// 删除登录凭证
pub fn delete_credential() -> Result<(), String> {
    let path = get_credential_path();

    if path.exists() {
        fs::remove_file(&path).map_err(|e| format!("删除文件失败: {}", e))?;
    }

    Ok(())
}
