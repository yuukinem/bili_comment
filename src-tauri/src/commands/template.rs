use uuid::Uuid;

use crate::models::CommentTemplate;
use crate::storage::template;

/// 获取所有模板
#[tauri::command]
pub fn get_templates() -> Vec<CommentTemplate> {
    template::load_templates()
}

/// 创建模板
#[tauri::command]
pub fn create_template(name: String, content: String) -> Result<CommentTemplate, String> {
    let now = chrono::Utc::now().timestamp();
    let template = CommentTemplate {
        id: Uuid::new_v4().to_string(),
        name,
        content,
        created_at: now,
        updated_at: now,
    };

    template::add_template(template.clone())?;
    Ok(template)
}

/// 更新模板
#[tauri::command]
pub fn update_template(id: String, name: String, content: String) -> Result<CommentTemplate, String> {
    let templates = template::load_templates();
    let existing = templates
        .iter()
        .find(|t| t.id == id)
        .ok_or_else(|| "模板不存在".to_string())?;

    let updated = CommentTemplate {
        id,
        name,
        content,
        created_at: existing.created_at,
        updated_at: chrono::Utc::now().timestamp(),
    };

    template::update_template(updated.clone())?;
    Ok(updated)
}

/// 删除模板
#[tauri::command]
pub fn delete_template(id: String) -> Result<(), String> {
    template::delete_template(&id)
}
