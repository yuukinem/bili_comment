// 数据存储模块

pub mod credential;
pub mod template;

use std::path::PathBuf;

/// 获取应用数据目录
pub fn get_app_data_dir() -> PathBuf {
    let base = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("bili-comment")
}

/// 确保目录存在
pub fn ensure_dir(path: &PathBuf) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    Ok(())
}
