pub mod app;
pub mod git_ops;
pub mod github;
pub mod logging;
pub mod models;
mod self_update;
#[cfg(test)]
pub mod tests;
pub mod ui;
pub mod util;

use anyhow::Result;
use app::App;
use logging::{append_log_line, ensure_log_file, tail_log_lines};
use models::MAX_LOG_LINES;
#[cfg(test)]
pub(crate) use self_update::update_bat_content;
pub use self_update::{run_self_update, should_handle_update_subcommand};
use std::fs;
use util::{app_data_dir, now_string};

pub fn run() -> Result<()> {
    let log_dir = app_data_dir()?;
    fs::create_dir_all(&log_dir)?;
    let obsolete_config = log_dir.join("config.toml");
    if obsolete_config.exists() {
        let _ = fs::remove_file(&obsolete_config);
    }
    let log_path = log_dir.join("log.txt");
    ensure_log_file(&log_path)?;
    append_log_line(&log_path, "----------------------------------------")?;
    append_log_line(
        &log_path,
        &format!("[{}] アプリを開始しました", now_string()),
    )?;
    append_log_line(
        &log_path,
        &format!("[{}] 起動処理を background で開始しました", now_string()),
    )?;
    let log_lines = tail_log_lines(&log_path, MAX_LOG_LINES)?;

    let mut app = App::new(log_path, log_lines);
    app.run_tui()
}
