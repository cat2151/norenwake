pub mod app;
pub mod git_ops;
pub mod github;
pub mod logging;
pub mod models;
#[cfg(test)]
pub mod tests;
pub mod ui;
pub mod util;

use anyhow::Result;
use app::App;
use github::{fetch_authenticated_login, fetch_repos, get_github_token};
use logging::{append_log_line, ensure_log_file, tail_log_lines};
use models::MAX_LOG_LINES;
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
    let log_lines = tail_log_lines(&log_path, MAX_LOG_LINES)?;

    let token = get_github_token()?;
    let github_login = fetch_authenticated_login(&token)?;
    append_log_line(
        &log_path,
        &format!(
            "[{}] `gh auth token` で GitHub token を取得しました",
            now_string()
        ),
    )?;
    append_log_line(
        &log_path,
        &format!("[{}] 認証済みログイン名: {}", now_string(), github_login),
    )?;
    let repos = fetch_repos(&token)?;
    append_log_line(
        &log_path,
        &format!(
            "[{}] リポジトリを {} 件取得しました",
            now_string(),
            repos.len()
        ),
    )?;

    let mut app = App::new(repos, log_path, log_lines, token, github_login);
    app.run_tui()
}
