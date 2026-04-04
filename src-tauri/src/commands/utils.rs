use biliup::bilibili::BiliBili;
use serde_json::{Value, json};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::str::FromStr;
use std::{fs, fs::File, io::Read, path::Path, path::PathBuf};
use tauri::Manager;
use tokio::sync::Mutex;
use tokio::time::{Duration, sleep};
use tracing::{debug, error, info, warn};

use crate::utils::crypto::encode_base64;
use crate::utils::file_utils::{self, FileEntry};
use crate::{AppData, models::TemplateConfig};

#[tauri::command]
pub async fn get_current_version() -> Result<String, String> {
    Ok(env!("CARGO_PKG_VERSION").to_string())
}

#[tauri::command]
pub async fn fetch_culua_search(query: String) -> Result<Value, String> {
    let client = reqwest::Client::new();
    let response = client
        .get("https://www.culua.com/api/search")
        .query(&[
            ("page", "1"),
            ("pageSize", "50"),
            ("source", "all"),
            ("fields", "title,artist"),
            ("sort", "pubdate_desc"),
            ("q", query.trim()),
        ])
        .send()
        .await
        .map_err(|e| format!("请求 culua 搜索失败: {e}"))?;

    response
        .json::<Value>()
        .await
        .map_err(|e| format!("解析 culua 搜索结果失败: {e}"))
}

/// 获取文件大小
#[tauri::command]
pub async fn get_file_size(file_path: String) -> Result<u64, String> {
    let path = Path::new(&file_path);
    file_utils::get_file_size(path).map_err(|e| format!("获取文件大小失败: {e}"))
}

/// 递归读取目录
#[tauri::command]
pub async fn read_dir_recursive(
    dir_path: String,
    include_subdirs: bool,
    max_depth: Option<u32>,
) -> Result<Vec<FileEntry>, String> {
    let path = Path::new(&dir_path);
    file_utils::read_dir_recursive(path, include_subdirs, max_depth)
        .map_err(|e| format!("读取目录失败: {e}"))
}

/// 读取文本文件内容
#[tauri::command]
pub async fn read_text_file(file_path: String) -> Result<String, String> {
    let bytes = fs::read(&file_path).map_err(|e| format!("读取文本文件失败: {e}"))?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

/// 读取文件并返回 base64
#[tauri::command]
pub async fn read_file_base64(file_path: String) -> Result<String, String> {
    let bytes = fs::read(&file_path).map_err(|e| format!("读取文件失败: {e}"))?;
    Ok(encode_base64(&bytes))
}

/// 上传封面并进行返回url
#[tauri::command]
pub async fn upload_cover(app: tauri::AppHandle, uid: u64, file: String) -> Result<String, String> {
    let app_lock = app.state::<Mutex<AppData>>();
    let app_data = app_lock.lock().await;

    let mut cover_file = File::open(file).map_err(|e| format!("打开文件失败: {e}"))?;
    let mut cover_buf = vec![];

    cover_file
        .read_to_end(&mut cover_buf)
        .map_err(|e| format!("读取文件失败: {e}"))?;

    match app_data
        .clients
        .lock()
        .await
        .get(&uid)
        .ok_or("用户未登录或不存在")?
        .bilibili
        .cover_up(&cover_buf)
        .await
    {
        Ok(url) => {
            info!("封面上传成功: {}", url);
            Ok(url)
        }
        Err(e) => Err(e.to_string()),
    }
}

/// 下载封面并进行base64编码
#[tauri::command]
pub async fn download_cover(
    app: tauri::AppHandle,
    uid: u64,
    url: String,
) -> Result<String, String> {
    let app_lock = app.state::<Mutex<AppData>>();
    let app_data = app_lock.lock().await;

    match app_data
        .clients
        .lock()
        .await
        .get(&uid)
        .ok_or("用户未登录或不存在")?
        .bilibili
        .client
        .get(&url)
        .send()
        .await
    {
        Ok(res) => {
            let bytes = res.bytes().await.map_err(|e| e.to_string())?;
            Ok(encode_base64(&bytes))
        }
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn get_type_list(app: tauri::AppHandle, uid: u64) -> Result<Value, String> {
    let app_lock = app.state::<Mutex<AppData>>();
    let app_data = app_lock.lock().await;

    match app_data
        .clients
        .lock()
        .await
        .get(&uid)
        .ok_or("用户未登录或不存在")?
        .bilibili
        .archive_pre()
        .await
    {
        Ok(res) => {
            debug!("获取分区列表成功: {}", res);
            Ok(res["data"]["typelist"].clone())
        }
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn get_topic_list(app: tauri::AppHandle, uid: u64) -> Result<Value, String> {
    let app_lock = app.state::<Mutex<AppData>>();
    let app_data = app_lock.lock().await;

    match app_data
        .clients
        .lock()
        .await
        .get(&uid)
        .ok_or("用户未登录或不存在")?
        .bilibili
        .client
        .get("https://member.bilibili.com/x/vupre/web/topic/type?pn=0&ps=999")
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<Value>()
        .await
    {
        Ok(res) => {
            debug!("获取话题列表成功: {}", res);
            Ok(res["data"]["topics"].clone())
        }
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn search_topics(
    app: tauri::AppHandle,
    uid: u64,
    query: String,
) -> Result<Value, String> {
    let app_lock = app.state::<Mutex<AppData>>();
    let app_data = app_lock.lock().await;

    match app_data.clients.lock().await.get(&uid).ok_or("用户未登录或不存在")?
            .bilibili
            .client
            .get(format!("https://member.bilibili.com/x/vupre/web/topic/search?keywords={query}&page_size=50&offset=0&t={}", chrono::Utc::now().timestamp()))
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json::<Value>()
            .await
        {
            Ok(res) => {
                debug!("搜索话题成功: {}", res);
                Ok(res["data"]["result"]["topics"].clone())
            },
            Err(e) => Err(e.to_string()),
        }
}

#[tauri::command]
pub async fn get_season_list(app: tauri::AppHandle, uid: u64) -> Result<Value, String> {
    let app_lock = app.state::<Mutex<AppData>>();
    let app_data = app_lock.lock().await;

    match app_data.clients.lock().await.get(&uid).ok_or("用户未登录或不存在")?
            .bilibili
            .client
            .get(format!("https://member.bilibili.com/x2/creative/web/seasons?pn=1&ps=50&order=desc&sort=mtime&filter=1&t={}", chrono::Utc::now().timestamp()))
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json::<Value>()
            .await
        {
            Ok(res) => {
                debug!("获取合集列表成功: {}", res);
                let mut season_vec = Vec::new();

                let seasons = res["data"]["seasons"].as_array()
                    .ok_or("用户没有创建合集").unwrap_or(&season_vec).to_owned();
                for season in &seasons {
                    let season_id = season["season"]["id"].as_u64().unwrap_or(0);
                    let season_title = season["season"]["title"].as_str().unwrap_or("").to_string();
                    let mut sections_vec = Vec::new();

                    if let Some(sections) = season["sections"]["sections"].as_array() {
                        for section in sections {
                            let section_id = section["id"].as_u64().unwrap_or(0);
                            let section_title = section["title"]
                                .as_str()
                                .unwrap_or(&season_title)
                                .to_string();

                            sections_vec.push(serde_json::json!({
                                "section_id": if section_id != 0 { Some(section_id) } else { None },
                                "title": section_title,
                            }));
                        }
                    }

                    let default_section_id = sections_vec
                        .first()
                        .and_then(|item| item["section_id"].as_u64())
                        .unwrap_or(0);

                    season_vec.push(serde_json::json!({
                        "season_id": if season_id != 0 { Some(season_id) } else { None },
                        "section_id": if default_section_id != 0 { Some(default_section_id) } else { None },
                        "title": season_title,
                        "sections": sections_vec,
                    }));
                }

                Ok(serde_json::json!({
                    "seasons": season_vec,
                }))
            },
            Err(e) => Err(e.to_string()),
        }
}

#[tauri::command]
pub async fn get_video_detail(
    app: tauri::AppHandle,
    uid: u64,
    video_id: String,
) -> Result<TemplateConfig, String> {
    let vid = biliup::uploader::bilibili::Vid::from_str(&video_id)
        .map_err(|e| format!("解析视频 ID 失败: {e}"))?;

    let app_lock = app.state::<Mutex<AppData>>();
    let app_data = app_lock.lock().await;

    let true_desc = match app_data.clients.lock().await.get(&uid) {
        Some(client) => {
            match client
                .bilibili
                .client
                .get(format!(
                    "https://api.bilibili.com/x/web-interface/view?{vid}",
                ))
                .send()
                .await
            {
                Ok(response) => match response.json::<Value>().await {
                    Ok(res) => res["data"]["desc"].as_str().unwrap_or("").to_string(),
                    Err(e) => {
                        error!("解析稿件描述响应失败: {:?}", e);
                        "".to_string()
                    }
                },
                Err(e) => {
                    error!("获取稿件描述请求失败: {:?}", e);
                    "".to_string()
                }
            }
        }
        None => {
            error!("用户未登录或不存在，无法获取稿件描述");
            "".to_string()
        }
    };

    let proxy: Option<String> = app_data
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
        .video_data(&vid, proxy.as_deref())
        .await
    {
        Ok(res) => {
            let mut template_config =
                TemplateConfig::from_bilibili_res(res).map_err(|e| e.to_string())?;
            if !true_desc.is_empty() {
                template_config.desc = true_desc;
            }
            Ok(template_config)
        }
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn get_video_season(app: tauri::AppHandle, uid: u64, aid: u64) -> Result<u64, String> {
    let app_lock = app.state::<Mutex<AppData>>();
    let app_data = app_lock.lock().await;

    match app_data
        .clients
        .lock()
        .await
        .get(&uid)
        .ok_or("用户未登录或不存在")?
        .bilibili
        .client
        .get(format!(
            "https://member.bilibili.com/x2/creative/web/season/aid?id={}&t={}",
            aid,
            chrono::Utc::now().timestamp()
        ))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<Value>()
        .await
    {
        Ok(res) => {
            debug!("获取稿件合集信息成功: {}", res);
            Ok(res["data"]["id"].as_u64().unwrap_or(0))
        }
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn switch_season(
    app: tauri::AppHandle,
    uid: u64,
    aid: u64,
    cid: u64,
    season_id: u64,
    section_id: u64,
    title: String,
    add: bool,
) -> Result<bool, String> {
    let app_lock = app.state::<Mutex<AppData>>();
    let app_data = app_lock.lock().await;

    fn get_csrf(bilibili: &BiliBili) -> Result<String, String> {
        let csrf = bilibili
            .login_info
            .cookie_info
            .get("cookies")
            .and_then(|c| c.as_array())
            .ok_or("Cookie错误")?
            .iter()
            .filter_map(|c| c.as_object())
            .find(|c| c["name"] == "bili_jct")
            .ok_or("JCT错误")?
            .get("value")
            .and_then(|v| v.as_str())
            .ok_or("CSRF错误")?;
        Ok(csrf.to_string())
    }

    let csrf = get_csrf(
        &app_data
            .clients
            .lock()
            .await
            .get(&uid)
            .ok_or("用户未登录或不存在")?
            .bilibili,
    )
    .map_err(|e| e.to_string())?;

    let retry_waits: [u64; 6] = [2, 3, 5, 8, 12, 15];
    let max_attempts = retry_waits.len();
    let mut last_err = String::new();

    for idx in 0..max_attempts {
        let attempt = idx + 1;
        let req_result = if add {
            app_data
                .clients
                .lock()
                .await
                .get(&uid)
                .ok_or("用户未登录或不存在")?
                .bilibili
                .client
                .post(format!(
                    "https://member.bilibili.com/x2/creative/web/season/section/episodes/add?t={}&csrf={}",
                    chrono::Utc::now().timestamp(),
                    csrf
                ))
                .json(&json!({
                    "episodes": [
                        {
                            "title": title,
                            "aid": aid,
                            "cid": cid
                        }
                    ],
                    "sectionId": section_id,
                    "csrf": csrf
                }))
                .send()
                .await
        } else {
            app_data
                .clients
                .lock()
                .await
                .get(&uid)
                .ok_or("用户未登录或不存在")?
                .bilibili
                .client
                .post(format!(
                    "https://member.bilibili.com/x2/creative/web/season/switch?t={}&csrf={}",
                    chrono::Utc::now().timestamp(),
                    csrf
                ))
                .json(&json!({
                    "season_id": if season_id != 0 { Some(season_id) } else { None },
                    "section_id": if section_id != 0 { Some(section_id) } else { None },
                    "title": title,
                    "aid": aid,
                    "cid": cid,
                    "csrf": csrf
                }))
                .send()
                .await
        };

        match req_result {
            Ok(resp) => match resp.json::<Value>().await {
                Ok(res) => {
                    let code = res["code"].as_i64().unwrap_or(-1);
                    if code == 0 {
                        debug!(
                            "设置合集成功(第{}次): {}",
                            attempt,
                            serde_json::to_string(&res).unwrap_or_default()
                        );
                        return Ok(true);
                    }
                    last_err = format!(
                        "code={}, msg={}, data={}",
                        code,
                        res["message"].as_str().unwrap_or(""),
                        serde_json::to_string(&res["data"]).unwrap_or_default()
                    );
                    warn!("设置合集失败(第{}次): {}", attempt, last_err);
                }
                Err(e) => {
                    last_err = format!("解析合集接口响应失败: {}", e);
                    warn!("设置合集失败(第{}次): {}", attempt, last_err);
                }
            },
            Err(e) => {
                last_err = format!("请求合集接口失败: {}", e);
                warn!("设置合集失败(第{}次): {}", attempt, last_err);
            }
        }

        if attempt < max_attempts {
            sleep(Duration::from_secs(retry_waits[idx])).await;
        }
    }

    Err(format!(
        "设置合集重试{}次仍失败: aid={}, cid={}, season_id={}, section_id={}, add={}, last_error={}",
        max_attempts, aid, cid, season_id, section_id, add, last_err
    ))
}

/// 导出日志
#[tauri::command]
pub async fn export_logs(export_path: String, date_filter: Option<String>) -> Result<String, String> {
    use std::fs;
    use std::io::Write;
    use zip::ZipWriter;
    use chrono::{DateTime, Local, NaiveDate};

    let log_dir = crate::utils::get_log_path().map_err(|e| format!("获取日志目录失败: {e}"))?;
    let zip_file = fs::File::create(&export_path).map_err(|e| format!("创建ZIP文件失败: {e}"))?;
    let mut zip = ZipWriter::new(zip_file);
    let options: zip::write::FileOptions<'_, ()> = zip::write::FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o644);

    // 添加日志文件
    let parsed_filter = date_filter
        .as_ref()
        .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());

    let mut matched_files: Vec<String> = Vec::new();
    let mut read_failed_files: Vec<String> = Vec::new();

    if let Ok(entries) = fs::read_dir(&log_dir) {
        for entry in entries.flatten() {
            if let Some(extension) = entry.path().extension() {
                if extension == "log" {
                    let file_name = entry.file_name().to_string_lossy().to_string();
                    if let Some(filter_date) = parsed_filter.as_ref() {
                        let date_text = date_filter.as_deref().unwrap_or("");
                        let mut include = file_name.contains(&format!("biliup-{}_", date_text));
                        if !include {
                            if let Ok(meta) = fs::metadata(entry.path()) {
                                if let Ok(modified) = meta.modified() {
                                    let local_time: DateTime<Local> = modified.into();
                                    include = local_time.date_naive() == *filter_date;
                                }
                            }
                        }
                        if !include {
                            continue;
                        }
                    }
                    if let Ok(content) = fs::read(entry.path()) {
                        zip.start_file(&file_name, options)
                            .map_err(|e| format!("创建ZIP条目失败: {e}"))?;
                        zip.write_all(&content)
                            .map_err(|e| format!("写入ZIP文件失败: {e}"))?;
                        matched_files.push(file_name.clone());
                    } else {
                        read_failed_files.push(file_name.clone());
                    }
                }
            }
        }
    }

    let manifest = format!(
        "date_filter={}\nmatched_files={}\n{}\nread_failed_files={}\n{}\n",
        date_filter.clone().unwrap_or_else(|| "ALL".to_string()),
        matched_files.len(),
        matched_files.join("\n"),
        read_failed_files.len(),
        read_failed_files.join("\n")
    );
    zip.start_file("export_manifest.txt", options)
        .map_err(|e| format!("创建ZIP条目失败: {e}"))?;
    zip.write_all(manifest.as_bytes())
        .map_err(|e| format!("写入ZIP文件失败: {e}"))?;

    zip.finish().map_err(|e| format!("完成ZIP文件失败: {e}"))?;

    Ok(export_path)
}

#[tauri::command]
pub async fn export_logs_since(export_path: String, since_ts: i64) -> Result<String, String> {
    use chrono::{DateTime, Local, TimeZone};
    use std::fs;
    use std::io::Write;
    use zip::ZipWriter;

    let log_dir = crate::utils::get_log_path().map_err(|e| format!("获取日志目录失败: {e}"))?;
    let zip_file = fs::File::create(&export_path).map_err(|e| format!("创建ZIP文件失败: {e}"))?;
    let mut zip = ZipWriter::new(zip_file);
    let options: zip::write::FileOptions<'_, ()> = zip::write::FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o644);

    let since_dt = Local
        .timestamp_opt(since_ts, 0)
        .single()
        .ok_or("启动时间戳无效")?;

    let mut matched_files: Vec<String> = Vec::new();
    let mut read_failed_files: Vec<String> = Vec::new();

    if let Ok(entries) = fs::read_dir(&log_dir) {
        for entry in entries.flatten() {
            if entry.path().extension().and_then(|s| s.to_str()) != Some("log") {
                continue;
            }

            let file_name = entry.file_name().to_string_lossy().to_string();
            let include = fs::metadata(entry.path())
                .ok()
                .and_then(|meta| meta.modified().ok())
                .map(|modified| {
                    let local_time: DateTime<Local> = modified.into();
                    local_time >= since_dt
                })
                .unwrap_or(false);

            if !include {
                continue;
            }

            if let Ok(content) = fs::read(entry.path()) {
                zip.start_file(&file_name, options)
                    .map_err(|e| format!("创建ZIP条目失败: {e}"))?;
                zip.write_all(&content)
                    .map_err(|e| format!("写入ZIP文件失败: {e}"))?;
                matched_files.push(file_name);
            } else {
                read_failed_files.push(file_name);
            }
        }
    }

    let manifest = format!(
        "since={}\nmatched_files={}\n{}\nread_failed_files={}\n{}\n",
        since_dt.format("%Y-%m-%d %H:%M:%S"),
        matched_files.len(),
        matched_files.join("\n"),
        read_failed_files.len(),
        read_failed_files.join("\n")
    );
    zip.start_file("export_manifest.txt", options)
        .map_err(|e| format!("创建ZIP条目失败: {e}"))?;
    zip.write_all(manifest.as_bytes())
        .map_err(|e| format!("写入ZIP文件失败: {e}"))?;

    zip.finish().map_err(|e| format!("完成ZIP文件失败: {e}"))?;
    Ok(export_path)
}

#[tauri::command]
pub async fn clear_old_logs(before_ts: Option<i64>) -> Result<u32, String> {
    use chrono::{DateTime, Local, TimeZone};
    use std::fs;

    let log_dir = crate::utils::get_log_path().map_err(|e| format!("获取日志目录失败: {e}"))?;
    let before_dt = before_ts.and_then(|ts| Local.timestamp_opt(ts, 0).single());
    let mut removed = 0u32;

    if let Ok(entries) = fs::read_dir(&log_dir) {
        for entry in entries.flatten() {
            if entry.path().extension().and_then(|s| s.to_str()) != Some("log") {
                continue;
            }

            let should_remove = if let Some(limit) = before_dt {
                fs::metadata(entry.path())
                    .ok()
                    .and_then(|meta| meta.modified().ok())
                    .map(|modified| {
                        let local_time: DateTime<Local> = modified.into();
                        local_time < limit
                    })
                    .unwrap_or(false)
            } else {
                true
            };

            if should_remove && fs::remove_file(entry.path()).is_ok() {
                removed += 1;
            }
        }
    }

    Ok(removed)
}

#[tauri::command]
pub async fn export_current_session_log(export_path: String, since_ts: i64) -> Result<String, String> {
    use chrono::{DateTime, Local, TimeZone};
    use std::fs;
    use std::io::Write;

    let log_dir = crate::utils::get_log_path().map_err(|e| format!("获取日志目录失败: {e}"))?;
    let since_dt = Local
        .timestamp_opt(since_ts, 0)
        .single()
        .ok_or("启动时间戳无效")?;

    let mut matched: Vec<(String, Vec<u8>)> = Vec::new();
    if let Ok(entries) = fs::read_dir(&log_dir) {
        for entry in entries.flatten() {
            if entry.path().extension().and_then(|s| s.to_str()) != Some("log") {
                continue;
            }
            let include = fs::metadata(entry.path())
                .ok()
                .and_then(|meta| meta.modified().ok())
                .map(|modified| {
                    let local_time: DateTime<Local> = modified.into();
                    local_time >= since_dt
                })
                .unwrap_or(false);
            if !include {
                continue;
            }
            if let Ok(content) = fs::read(entry.path()) {
                matched.push((entry.file_name().to_string_lossy().to_string(), content));
            }
        }
    }

    if matched.is_empty() {
        return Err("未找到本次启动日志".to_string());
    }

    matched.sort_by(|a, b| a.0.cmp(&b.0));
    let mut output = fs::File::create(&export_path).map_err(|e| format!("创建日志文件失败: {e}"))?;
    for (index, (name, content)) in matched.iter().enumerate() {
        if matched.len() > 1 {
            let header = format!("===== {} =====\n", name);
            output
                .write_all(header.as_bytes())
                .map_err(|e| format!("写入日志失败: {e}"))?;
        }
        output
            .write_all(content)
            .map_err(|e| format!("写入日志失败: {e}"))?;
        if index + 1 < matched.len() {
            output
                .write_all(b"\n\n")
                .map_err(|e| format!("写入日志失败: {e}"))?;
        }
    }

    Ok(export_path)
}

#[tauri::command]
pub async fn store_local_cover(
    uid: u64,
    key: String,
    source_path: String,
    is_folder: bool,
) -> Result<String, String> {
    let source = PathBuf::from(&source_path);
    if !source.exists() {
        return Err("封面文件不存在".to_string());
    }

    let ext = source
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("jpg")
        .to_string();

    let mut hasher = DefaultHasher::new();
    key.hash(&mut hasher);
    let hashed_key = format!("{:016x}", hasher.finish());

    let cover_dir = crate::utils::get_config_dir()
        .map_err(|e| format!("获取配置目录失败: {e}"))?
        .join("covers")
        .join(uid.to_string());
    fs::create_dir_all(&cover_dir).map_err(|e| format!("创建封面目录失败: {e}"))?;

    let target_name = if is_folder {
        format!("folder_{}.{}", hashed_key, ext)
    } else {
        format!("template_{}.{}", hashed_key, ext)
    };
    let target_path = cover_dir.join(target_name);
    info!(
        "保存本地封面: uid={}, is_folder={}, source={}, target={}",
        uid,
        is_folder,
        source.display(),
        target_path.display()
    );
    fs::copy(&source, &target_path).map_err(|e| format!("保存封面文件失败: {e}"))?;
    if !target_path.exists() {
        return Err("封面文件保存后未找到目标文件".to_string());
    }
    Ok(target_path.to_string_lossy().to_string())
}

/// 检查更新
#[tauri::command]
pub async fn check_update() -> Result<Option<String>, String> {
    use reqwest;
    use serde_json::Value;

    let client = reqwest::Client::new();
    let response = client
        .get("https://api.github.com/repos/biliup/biliup-app-new/releases/latest")
        .header("User-Agent", "biliup-app")
        .send()
        .await
        .map_err(|e| format!("网络请求失败: {e}"))?;

    let release_info: Value = response
        .json()
        .await
        .map_err(|e| format!("解析响应失败: {e}"))?;

    let latest_tag = release_info["tag_name"]
        .as_str()
        .ok_or("无法获取最新版本标签")?;

    info!("最新版本：{latest_tag}");
    // 解析版本号 (格式: app-va.b.c)
    let latest_version = latest_tag.strip_prefix("app-v").ok_or("版本标签格式错误")?;

    let current_version = env!("CARGO_PKG_VERSION");

    if is_newer_version(latest_version, current_version)? {
        Ok(Some(latest_tag.to_string()))
    } else {
        Ok(None)
    }
}

/// 比较版本号
fn is_newer_version(latest: &str, current: &str) -> Result<bool, String> {
    let parse_version = |v: &str| -> Result<Vec<u32>, String> {
        v.split('.')
            .map(|part| {
                part.parse::<u32>()
                    .map_err(|_| "版本号格式错误".to_string())
            })
            .collect()
    };

    let latest_parts = parse_version(latest)?;
    let current_parts = parse_version(current)?;

    for (latest_part, current_part) in latest_parts.iter().zip(current_parts.iter()) {
        if latest_part > current_part {
            return Ok(true);
        } else if latest_part < current_part {
            return Ok(false);
        }
    }

    // 如果前面的部分都相等，比较长度
    Ok(latest_parts.len() > current_parts.len())
}

/// 检查更新
#[tauri::command]
pub async fn console_log(
    _app: tauri::AppHandle,
    level: String,
    messages: Vec<String>,
) -> Result<(), String> {
    let message = messages.join(" ");
    match level.as_str() {
        "log" => info!("Webconsole: {}", message),
        "error" => error!("Webconsole: {}", message),
        "warn" => warn!("Webconsole: {}", message),
        _ => info!("Webconsole: {}", message),
    }
    Ok(())
}
