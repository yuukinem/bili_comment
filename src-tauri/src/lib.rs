// 模块声明
mod api;
mod commands;
mod models;
mod storage;

use commands::{auth, comment, search, template};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            // 启用日志
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            // 初始化已保存的登录凭证
            api::login::init_credential();

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // 登录命令
            auth::get_login_qrcode,
            auth::poll_login_status,
            auth::get_user_info,
            auth::logout,
            auth::check_login_valid,
            // 搜索命令
            search::search_videos,
            // 评论命令
            comment::send_comment,
            comment::batch_send_comments,
            comment::get_batch_status,
            comment::cancel_batch,
            comment::clear_batch,
            comment::get_comment_interval,
            // 模板命令
            template::get_templates,
            template::create_template,
            template::update_template,
            template::delete_template,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
