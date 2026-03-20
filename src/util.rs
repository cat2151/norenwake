use anyhow::{anyhow, Result};
use dirs::data_local_dir;
use std::path::PathBuf;
use time::{macros::format_description, OffsetDateTime, UtcOffset};

use crate::models::APP_NAME;

pub fn now_string() -> String {
    let format = format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");
    let now = OffsetDateTime::now_utc();
    let local = UtcOffset::current_local_offset()
        .ok()
        .map(|offset| now.to_offset(offset))
        .unwrap_or(now);
    local
        .format(&format)
        .unwrap_or_else(|_| "1970-01-01 00:00:00".to_string())
}

pub fn app_data_dir() -> Result<PathBuf> {
    let base = data_local_dir().ok_or_else(|| anyhow!("local data dir を解決できません"))?;
    Ok(base.join(APP_NAME))
}

pub fn work_dir() -> Result<PathBuf> {
    Ok(app_data_dir()?.join("work"))
}

pub fn cache_dir() -> Result<PathBuf> {
    Ok(app_data_dir()?.join("caches"))
}

pub fn cache_index_path() -> Result<PathBuf> {
    Ok(cache_dir()?.join("caches.json"))
}
