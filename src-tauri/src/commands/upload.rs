use std::sync::Arc;

use crate::{
    AppData,
    models::{TemplateConfig, UploadTask, VideoInfo},
};
use serde_json::Value;
use tauri::{AppHandle, Manager};
use tokio::sync::Mutex;
use tracing::info;

/// 创建上传任务
#[tauri::command]
pub async fn create_upload_task(
    app: AppHandle,
    uid: u64,
    template: String,
    video: VideoInfo,
) -> Result<(), String> {
    let app_lock = app.state::<Mutex<AppData>>();
    let mut app_data = app_lock.lock().await;
    let user = app_data
        .clients
        .lock()
        .await
        .get(&uid)
        .ok_or("用户未登录或不存在")?
        .user
        .clone();
    let config_copy = Arc::clone(&app_data.config);
    let clients_copy = Arc::clone(&app_data.clients);
    let upload_service = &mut app_data.upload_service;

    upload_service
        .create_task(&user, &template, &video, config_copy, clients_copy)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// 开始上传
#[tauri::command]
pub async fn start_upload(app: AppHandle, task_id: String) -> Result<bool, String> {
    let app_lock = app.state::<Mutex<AppData>>();
    let mut app_data = app_lock.lock().await;
    let upload_service = &mut app_data.upload_service;

    upload_service
        .start_upload(&task_id)
        .await
        .map_err(|e| e.to_string())
}

/// 暂停上传
#[tauri::command]
pub async fn pause_upload(app: AppHandle, task_id: String) -> Result<bool, String> {
    let app_lock = app.state::<Mutex<AppData>>();
    let mut app_data = app_lock.lock().await;
    let upload_service = &mut app_data.upload_service;

    upload_service
        .pause_upload(&task_id)
        .await
        .map_err(|e| e.to_string())
}

/// 取消上传
#[tauri::command]
pub async fn cancel_upload(app: AppHandle, task_id: String) -> Result<bool, String> {
    let app_lock = app.state::<Mutex<AppData>>();
    let mut app_data = app_lock.lock().await;
    let upload_service = &mut app_data.upload_service;

    upload_service
        .cancel_upload(&task_id)
        .await
        .map_err(|e| e.to_string())
}

/// 获取上传队列
#[tauri::command]
pub async fn get_upload_queue(app: AppHandle) -> Result<Vec<UploadTask>, String> {
    let app_lock = app.state::<Mutex<AppData>>();
    let mut app_data = app_lock.lock().await;
    let upload_service = &mut app_data.upload_service;
    upload_service
        .get_upload_queue()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_upload_runtime_status(
    app: AppHandle,
) -> Result<crate::services::upload_service::UploadRuntimeStatus, String> {
    let app_lock = app.state::<Mutex<AppData>>();
    let app_data = app_lock.lock().await;
    Ok(app_data.upload_service.get_runtime_status().await)
}

/// 重新上传失败的任务
#[tauri::command]
pub async fn retry_upload(app: AppHandle, task_id: String) -> Result<bool, String> {
    let app_lock = app.state::<Mutex<AppData>>();
    let mut app_data = app_lock.lock().await;
    let upload_service = &mut app_data.upload_service;

    upload_service
        .retry_upload(&task_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn submit(app: AppHandle, uid: u64, form: TemplateConfig) -> Result<Value, String> {
    let app_lock = app.state::<Mutex<AppData>>();
    let app_data = app_lock.lock().await;
    let is_edit = form.aid.is_some();

    if !is_edit {
        // 将前端表单转换为B站API需要的格式
        let bilibili_form = form.into_bilibili_form();
        info!(
            "提交请求参数: uid={}, mode=web_add_v3, is_only_self={}, videos_count={}, aid=None",
            uid,
            bilibili_form.is_only_self,
            bilibili_form.videos.len()
        );
        let studio = bilibili_form.try_into_studio().map_err(|e| e.to_string())?;

        #[cfg(debug_assertions)]
        {
            use tracing::debug;

            let json_content = serde_json::to_string_pretty(&studio).unwrap();
            debug!("转换后的B站提交表单: {uid}\n{}", json_content);
        }

        let proxy = app_data
            .config
            .lock()
            .await
            .config
            .get(&uid)
            .and_then(|c| c.proxy.clone());

        match app_data
            .clients
            .lock()
            .await
            .get(&uid)
            .ok_or("用户未登录或不存在")?
            .bilibili
            .submit_by_web(&studio, proxy.as_deref())
            .await
        {
            Ok(resp) => {
                info!("添加稿件成功：{resp}");
                Ok(resp.data.ok_or("返回值错误").map_err(|e| e.to_string())?)
            }
            Err(e) => Err(e.to_string()),
        }
    } else {
        let bilibili_form = form.into_bilibili_form();
        info!(
            "提交请求参数: uid={}, mode=web_edit, is_only_self={}, videos_count={}, aid={:?}",
            uid,
            bilibili_form.is_only_self,
            bilibili_form.videos.len(),
            bilibili_form.aid
        );
        let studio = bilibili_form.try_into_studio().map_err(|e| e.to_string())?;
        match app_data
            .clients
            .lock()
            .await
            .get(&uid)
            .ok_or("用户未登录或不存在")?
            .bilibili
            .edit_by_web(&studio)
            .await
        {
            Ok(resp) => {
                info!("编辑稿件成功：{resp}");
                Ok(resp["data"].clone())
            }
            Err(e) => Err(e.to_string()),
        }
    }
}
