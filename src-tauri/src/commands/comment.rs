use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::api::comment;
use crate::models::{BatchStatus, CommentResult, CommentTask, TaskStatus, VideoItem};

/// 批量任务存储
static BATCH_TASKS: once_cell::sync::Lazy<Arc<Mutex<HashMap<String, BatchStatus>>>> =
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

/// 取消标记存储
static CANCEL_FLAGS: once_cell::sync::Lazy<Arc<Mutex<HashMap<String, bool>>>> =
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

/// 安全截取字符串 (处理中文等多字节字符)
fn truncate_str(s: &str, max_chars: usize) -> String {
    let char_count = s.chars().count();
    if char_count > max_chars {
        format!("{}...", s.chars().take(max_chars).collect::<String>())
    } else {
        s.to_string()
    }
}

/// 发送单条评论
#[tauri::command]
pub async fn send_comment(_bvid: String, aid: u64, content: String) -> Result<CommentResult, String> {
    log::info!("📝 Command: 发送单条评论 aid={}", aid);
    comment::send_comment(aid, &content)
        .await
        .map_err(|e| e.to_user_message())
}

/// 批量发送评论 (启动异步任务)
#[tauri::command]
pub async fn batch_send_comments(videos: Vec<VideoItem>, content: String) -> Result<String, String> {
    let batch_id = Uuid::new_v4().to_string();

    log::info!(
        "🚀 Command: 启动批量评论任务 batch_id={}, 视频数={}",
        batch_id,
        videos.len()
    );

    // 创建任务列表
    let tasks: Vec<CommentTask> = videos
        .iter()
        .map(|video| CommentTask {
            id: Uuid::new_v4().to_string(),
            video: video.clone(),
            content: content.clone(),
            status: TaskStatus::Pending,
            error_msg: None,
            created_at: chrono::Utc::now().timestamp(),
            completed_at: None,
        })
        .collect();

    let batch_status = BatchStatus {
        batch_id: batch_id.clone(),
        total: tasks.len(),
        completed: 0,
        success: 0,
        failed: 0,
        tasks,
    };

    // 保存批次状态
    {
        let mut batches = BATCH_TASKS.lock();
        batches.insert(batch_id.clone(), batch_status);
    }

    // 初始化取消标记
    {
        let mut flags = CANCEL_FLAGS.lock();
        flags.insert(batch_id.clone(), false);
    }

    // 启动异步执行任务
    let batch_id_clone = batch_id.clone();
    tokio::spawn(async move {
        execute_batch_tasks(batch_id_clone).await;
    });

    Ok(batch_id)
}

/// 执行批量任务
async fn execute_batch_tasks(batch_id: String) {
    let task_count = {
        let batches = BATCH_TASKS.lock();
        batches.get(&batch_id).map(|b| b.tasks.len()).unwrap_or(0)
    };

    log::info!("📋 批量任务开始执行: batch_id={}, 任务数={}", batch_id, task_count);

    for i in 0..task_count {
        // 检查取消标记
        {
            let flags = CANCEL_FLAGS.lock();
            if flags.get(&batch_id).copied().unwrap_or(false) {
                log::warn!("⏹️ 批量任务被取消: batch_id={}, 已完成={}/{}", batch_id, i, task_count);
                // 将剩余任务标记为取消
                let mut batches = BATCH_TASKS.lock();
                if let Some(batch) = batches.get_mut(&batch_id) {
                    for task in batch.tasks.iter_mut().skip(i) {
                        if task.status == TaskStatus::Pending {
                            task.status = TaskStatus::Cancelled;
                        }
                    }
                }
                break;
            }
        }

        // 获取当前任务信息
        let (aid, content, title) = {
            let mut batches = BATCH_TASKS.lock();
            if let Some(batch) = batches.get_mut(&batch_id) {
                batch.tasks[i].status = TaskStatus::Running;
                let title = batch.tasks[i].video.title.clone();
                (batch.tasks[i].video.aid, batch.tasks[i].content.clone(), title)
            } else {
                break;
            }
        };

        log::info!(
            "▶️ 执行任务 [{}/{}]: aid={}, 标题=\"{}\"",
            i + 1,
            task_count,
            aid,
            truncate_str(&title, 20)
        );

        // 执行评论 (带频率限制)
        let result = comment::send_comment_with_rate_limit(aid, &content).await;

        // 更新任务状态
        {
            let mut batches = BATCH_TASKS.lock();
            if let Some(batch) = batches.get_mut(&batch_id) {
                match result {
                    Ok(r) if r.success => {
                        batch.tasks[i].status = TaskStatus::Success;
                        batch.success += 1;
                    }
                    Ok(r) => {
                        batch.tasks[i].status = TaskStatus::Failed;
                        batch.tasks[i].error_msg = r.error_msg;
                        batch.failed += 1;
                    }
                    Err(e) => {
                        batch.tasks[i].status = TaskStatus::Failed;
                        batch.tasks[i].error_msg = Some(e.to_user_message());
                        batch.failed += 1;
                    }
                }
                batch.tasks[i].completed_at = Some(chrono::Utc::now().timestamp());
                batch.completed += 1;
            }
        }
    }

    // 获取最终统计
    let (success, failed, total) = {
        let batches = BATCH_TASKS.lock();
        if let Some(batch) = batches.get(&batch_id) {
            (batch.success, batch.failed, batch.total)
        } else {
            (0, 0, 0)
        }
    };

    log::info!(
        "🏁 批量任务完成: batch_id={}, 成功={}, 失败={}, 总计={}",
        batch_id,
        success,
        failed,
        total
    );

    // 清理取消标记
    {
        let mut flags = CANCEL_FLAGS.lock();
        flags.remove(&batch_id);
    }
}

/// 获取批量任务状态
#[tauri::command]
pub fn get_batch_status(batch_id: String) -> Result<BatchStatus, String> {
    let batches = BATCH_TASKS.lock();
    batches
        .get(&batch_id)
        .cloned()
        .ok_or_else(|| "批次不存在".to_string())
}

/// 取消批量任务
#[tauri::command]
pub fn cancel_batch(batch_id: String) -> Result<(), String> {
    log::info!("⏹️ Command: 取消批量任务 batch_id={}", batch_id);
    let mut flags = CANCEL_FLAGS.lock();
    flags.insert(batch_id, true);
    Ok(())
}

/// 清理已完成的批次
#[tauri::command]
pub fn clear_batch(batch_id: String) {
    let mut batches = BATCH_TASKS.lock();
    batches.remove(&batch_id);
}

/// 获取评论间隔时间
#[tauri::command]
pub fn get_comment_interval() -> u64 {
    comment::get_comment_interval()
}
