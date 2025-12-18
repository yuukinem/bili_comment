use std::fs;
use std::path::PathBuf;

use crate::models::CommentTemplate;

use super::{ensure_dir, get_app_data_dir};

/// 获取模板文件路径
fn get_templates_path() -> PathBuf {
    get_app_data_dir().join("templates.json")
}

/// 加载所有模板
pub fn load_templates() -> Vec<CommentTemplate> {
    let path = get_templates_path();

    if !path.exists() {
        return Vec::new();
    }

    fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

/// 保存所有模板
pub fn save_templates(templates: &[CommentTemplate]) -> Result<(), String> {
    let path = get_templates_path();
    ensure_dir(&path).map_err(|e| format!("创建目录失败: {}", e))?;

    let json = serde_json::to_string_pretty(templates)
        .map_err(|e| format!("序列化失败: {}", e))?;

    fs::write(&path, json).map_err(|e| format!("写入文件失败: {}", e))?;

    Ok(())
}

/// 添加模板
pub fn add_template(template: CommentTemplate) -> Result<(), String> {
    let mut templates = load_templates();
    templates.push(template);
    save_templates(&templates)
}

/// 更新模板
pub fn update_template(template: CommentTemplate) -> Result<(), String> {
    let mut templates = load_templates();

    if let Some(pos) = templates.iter().position(|t| t.id == template.id) {
        templates[pos] = template;
        save_templates(&templates)
    } else {
        Err("模板不存在".to_string())
    }
}

/// 删除模板
pub fn delete_template(id: &str) -> Result<(), String> {
    let mut templates = load_templates();

    if let Some(pos) = templates.iter().position(|t| t.id == id) {
        templates.remove(pos);
        save_templates(&templates)
    } else {
        Err("模板不存在".to_string())
    }
}
