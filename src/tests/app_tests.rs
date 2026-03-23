use crate::{
    app::{App, StartupData},
    models::Repo,
};
use std::{
    fs::{self, OpenOptions},
    sync::{
        atomic::{AtomicU64, Ordering},
        mpsc,
    },
    time::{SystemTime, UNIX_EPOCH},
};

static TEMP_LOG_COUNTER: AtomicU64 = AtomicU64::new(0);

struct TempLogPath {
    path: std::path::PathBuf,
}

impl TempLogPath {
    fn new() -> Self {
        Self {
            path: temp_log_path(),
        }
    }

    fn clone_path(&self) -> std::path::PathBuf {
        self.path.clone()
    }
}

impl Drop for TempLogPath {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

fn temp_log_path() -> std::path::PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let counter = TEMP_LOG_COUNTER.fetch_add(1, Ordering::Relaxed);
    let pid = std::process::id();
    let path = std::env::temp_dir().join(format!(
        "norenwake-app-test-{}-pid{}-{}.log",
        nanos, pid, counter
    ));
    OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&path)
        .unwrap();
    path
}

fn repo(name: &str) -> Repo {
    Repo {
        name: name.to_string(),
        full_name: format!("me/{}", name),
        clone_url: format!("https://github.com/me/{}.git", name),
        default_branch: "main".to_string(),
        private: false,
        fork: false,
        archived: false,
        updated_at: "2026-03-01T00:00:00Z".to_string(),
        description: None,
    }
}

#[test]
fn app_can_initialize_before_startup_data_arrives() {
    let log_path = TempLogPath::new();
    let (_tx, rx) = mpsc::channel();

    let app = App::new_with_startup_receiver(log_path.clone_path(), Vec::new(), Some(rx));

    assert!(app.is_startup_loading());
    assert!(app.repos.is_empty());
    assert_eq!(app.github_token, "");
    assert_eq!(app.github_login, "");
    assert_eq!(app.readme_preview_markdown, "起動処理を読み込んでいます...");
}

#[test]
fn startup_data_is_applied_after_background_loading() {
    let log_path = TempLogPath::new();
    let mut app = App::new_with_startup_receiver(log_path.clone_path(), Vec::new(), None);

    app.apply_startup_data(StartupData {
        github_token: "token".to_string(),
        github_login: "me".to_string(),
        repos: vec![repo("repo-a")],
        startup_tree_lines: vec!["src".to_string(), "README.md".to_string()],
    });

    assert_eq!(app.github_token, "token");
    assert_eq!(app.github_login, "me");
    assert_eq!(app.repos.len(), 1);
    assert_eq!(app.repos[0].name, "repo-a");
    assert_eq!(app.repo_state.list.selected(), Some(0));
    assert_eq!(app.tree_state.list.selected(), Some(0));
    assert_eq!(
        app.current_tree_lines(),
        &["src".to_string(), "README.md".to_string()]
    );
}

#[test]
fn startup_error_is_retained_when_background_channel_disconnects() {
    let log_path = TempLogPath::new();
    let (tx, rx) = mpsc::channel();
    let mut app = App::new_with_startup_receiver(log_path.clone_path(), Vec::new(), Some(rx));

    drop(tx);
    app.drain_startup_updates();

    assert!(!app.is_startup_loading());
    assert_eq!(
        app.startup_error.as_deref(),
        Some("起動処理チャネルが切断されました")
    );
    assert_eq!(app.readme_preview_title, "startup error");
    assert_eq!(
        app.readme_preview_markdown,
        "起動処理に失敗しました。log を確認してください"
    );
}
