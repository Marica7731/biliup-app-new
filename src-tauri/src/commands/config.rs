use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::fs;
use tauri::AppHandle;
use tauri::Manager;
use tokio::sync::Mutex;
use tracing::info;

use crate::{AppData, models::ConfigRoot};
use crate::{models::TemplateConfig, utils::get_config_json_path};

#[derive(Debug, Serialize, Deserialize)]
pub struct TemplateCommandResponse {
    pub success: bool,
    pub message: String,
    pub template: Option<TemplateConfig>,
}

/// 加载配置文件
#[tauri::command]
pub async fn load_config(app: AppHandle) -> Result<ConfigRoot, String> {
    let data = app.state::<Mutex<AppData>>();

    Ok(data.lock().await.config.lock().await.clone())
}

/// 保存配置文件
#[tauri::command]
pub async fn save_config(app: AppHandle) -> Result<bool, String> {
    let data = app.state::<Mutex<AppData>>();

    data.lock()
        .await
        .config
        .lock()
        .await
        .save_to_file(&get_config_json_path().map_err(|e| format!("获取配置路径失败: {e}"))?)
        .map_err(|e| format!("保存配置失败: {e}"))?;
    Ok(true)
}

#[tauri::command]
pub async fn save_user_config(
    app: AppHandle,
    uid: u64,
    line: Option<String>,
    proxy: Option<String>,
    limit: u32,
    watermark: u8,
    auto_edit: u8,
) -> Result<bool, String> {
    let data = app.state::<Mutex<AppData>>();
    info!("用户({uid})配置已保存");

    data.lock()
        .await
        .config
        .lock()
        .await
        .save_user_config(uid, line, proxy, limit, watermark, auto_edit)
        .map_err(|e| format!("保存用户配置失败: {e}"))?;
    Ok(true)
}

#[tauri::command]
pub async fn save_global_config(
    app: AppHandle,
    max_curr: u32,
    auto_start: bool,
    auto_upload: bool,
    log_level: String,
) -> Result<bool, String> {
    let data = app.state::<Mutex<AppData>>();

    info!("全局配置已保存");

    data.lock().await.config.lock().await.save_global_config(
        max_curr,
        auto_start,
        auto_upload,
        log_level,
    );

    data.lock()
        .await
        .upload_service
        .set_max_concurrent(max_curr)
        .await;
    Ok(true)
}

#[tauri::command]
pub async fn set_config(app: AppHandle, config: ConfigRoot) -> Result<bool, String> {
    let data = app.state::<Mutex<AppData>>();
    let mut app_data = data.lock().await;

    *app_data.config.lock().await = config;

    app_data
        .config
        .lock()
        .await
        .save_to_file(&get_config_json_path().map_err(|e| format!("获取配置路径失败: {e}"))?)
        .map_err(|e| format!("保存配置失败: {e}"))?;

    let max_curr = app_data.config.lock().await.max_curr;
    app_data.upload_service.set_max_concurrent(max_curr).await;
    info!("配置已导入并保存");
    Ok(true)
}

#[tauri::command]
pub async fn export_config_to_path(
    app: AppHandle,
    path: String,
    include_cookie: bool,
) -> Result<bool, String> {
    let data = app.state::<Mutex<AppData>>();
    let app_data = data.lock().await;
    let config = app_data.config.lock().await.clone();

    let mut config_value =
        serde_json::to_value(config).map_err(|e| format!("序列化配置失败: {e}"))?;

    if !include_cookie {
        if let Some(config_map) = config_value.get_mut("config").and_then(|v| v.as_object_mut()) {
            for user_cfg in config_map.values_mut() {
                if let Some(user_obj) = user_cfg.get_mut("user").and_then(|v| v.as_object_mut()) {
                    user_obj.remove("cookie");
                }
            }
        }
    }

    let content = serde_json::to_string_pretty(&config_value)
        .map_err(|e| format!("生成导出文件失败: {e}"))?;
    fs::write(&path, content).map_err(|e| format!("写入导出文件失败: {e}"))?;
    Ok(true)
}

#[tauri::command]
pub async fn import_config_from_path(
    app: AppHandle,
    path: String,
    keep_existing_cookie: bool,
) -> Result<String, String> {
    let content = fs::read_to_string(&path).map_err(|e| format!("读取导入文件失败: {e}"))?;
    let mut imported_value: Value =
        serde_json::from_str(&content).map_err(|e| format!("解析导入文件失败: {e}"))?;

    let data = app.state::<Mutex<AppData>>();
    let mut app_data = data.lock().await;
    let current_config = app_data.config.lock().await.clone();
    let mut skipped_uids: Vec<String> = Vec::new();

    if keep_existing_cookie {
        if let Some(config_map) = imported_value.get_mut("config").and_then(|v| v.as_object_mut()) {
            let keys: Vec<String> = config_map.keys().cloned().collect();
            for uid_str in keys {
                let mut has_cookie = false;
                if let Some(user_cfg) = config_map.get(&uid_str) {
                    has_cookie = user_cfg
                        .get("user")
                        .and_then(|u| u.get("cookie"))
                        .map(|c| !c.is_null())
                        .unwrap_or(false);
                }

                if has_cookie {
                    continue;
                }

                let uid = uid_str
                    .parse::<u64>()
                    .map_err(|e| format!("UID 解析失败({uid_str}): {e}"))?;
                if let Some(current_user_cfg) = current_config.config.get(&uid) {
                    if let Some(user_cfg) = config_map.get_mut(&uid_str) {
                        let user_obj = user_cfg
                            .get_mut("user")
                            .and_then(|v| v.as_object_mut())
                            .ok_or_else(|| format!("导入配置用户字段异常: {uid_str}"))?;
                        user_obj.insert(
                            "cookie".to_string(),
                            json!(current_user_cfg.user.cookie.clone()),
                        );
                    }
                } else {
                    skipped_uids.push(uid_str.clone());
                    config_map.remove(&uid_str);
                }
            }
        }
    }

    let imported_config: ConfigRoot =
        serde_json::from_value(imported_value).map_err(|e| format!("导入配置格式错误: {e}"))?;
    *app_data.config.lock().await = imported_config;

    app_data
        .config
        .lock()
        .await
        .save_to_file(&get_config_json_path().map_err(|e| format!("获取配置路径失败: {e}"))?)
        .map_err(|e| format!("保存配置失败: {e}"))?;

    let max_curr = app_data.config.lock().await.max_curr;
    app_data.upload_service.set_max_concurrent(max_curr).await;

    if skipped_uids.is_empty() {
        Ok(String::new())
    } else {
        Ok(format!("部分用户缺少 Cookie 已跳过: {}", skipped_uids.join(", ")))
    }
}

#[tauri::command]
pub async fn delete_user_template(
    app: AppHandle,
    uid: u64,
    template_name: String,
) -> Result<TemplateCommandResponse, String> {
    let app_lock = app.state::<Mutex<AppData>>();
    let app_data = app_lock.lock().await;

    app_data
        .config
        .lock()
        .await
        .delete_user_template(uid, &template_name);
    info!("删除模板: {}", template_name);

    Ok(TemplateCommandResponse {
        success: true,
        message: "模板删除成功".to_string(),
        template: None,
    })
}

#[tauri::command]
pub async fn update_user_template(
    app: AppHandle,
    uid: u64,
    template_name: String,
    template: TemplateConfig,
) -> Result<TemplateCommandResponse, String> {
    let app_lock = app.state::<Mutex<AppData>>();
    let app_data = app_lock.lock().await;

    let updated = app_data
        .config
        .lock()
        .await
        .add_user_template(uid, &template_name, template);
    info!("更新模板: {}", template_name);

    Ok(TemplateCommandResponse {
        success: true,
        message: "模板更新成功".to_string(),
        template: Some(updated),
    })
}

#[tauri::command]
pub async fn add_user_template(
    app: AppHandle,
    uid: u64,
    template_name: String,
    template: TemplateConfig,
) -> Result<TemplateCommandResponse, String> {
    let app_lock = app.state::<Mutex<AppData>>();
    let app_data = app_lock.lock().await;

    let added = app_data
        .config
        .lock()
        .await
        .add_user_template(uid, &template_name, template);
    info!("添加模板: {}", template_name);

    Ok(TemplateCommandResponse {
        success: true,
        message: "模板添加成功".to_string(),
        template: Some(added),
    })
}
