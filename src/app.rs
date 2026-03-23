mod cache;
mod input;
mod startup;
mod workflow;

use anyhow::{anyhow, Context, Result};
use arboard::Clipboard;
use crossterm::{
    event::{
        self, DisableFocusChange, DisableMouseCapture, EnableFocusChange, EnableMouseCapture,
        Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent,
        MouseEventKind,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    Terminal,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    io::stdout,
    process::Command,
    sync::mpsc::{self, Receiver},
    time::{Duration, Instant},
};

use crate::{
    git_ops::{
        build_remote_safety_lines, build_target_origin_push_url, build_target_origin_url,
        collect_files, ensure_safe_remotes, git_remote_push_url, git_remote_url,
        rename_preview_dir, render_delta_for_file, run_cmd_logged, update_readme_ja,
    },
    github::{check_repo_name_available, fetch_repo_readme_ja},
    logging::append_log_line,
    models::{
        ActivePane, LogState, Mode, PreviewState, Repo, RepoListState, StatusItem, TreeListState,
        DEFAULT_NEW_REPO, MAX_LOG_LINES,
    },
    ui::draw_ui,
    util::{cache_dir, cache_index_path, now_string, work_dir},
};

enum PendingAction {
    CloneSelectedRepo,
    PushToOrigin,
    ConfirmNewRepoName,
    CommitChanges,
}

struct LayoutRects {
    repos: Rect,
    dir_tree: Option<Rect>,
    log: Rect,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct CacheIndex {
    repos: BTreeMap<String, String>,
}

struct PrefetchUpdate {
    repo_name: String,
    updated_at: String,
}

struct CachedReadme {
    title: String,
    markdown: String,
}

pub(crate) struct StartupData {
    pub github_token: String,
    pub github_login: String,
    pub repos: Vec<Repo>,
    pub startup_tree_lines: Vec<String>,
}

pub struct App {
    pub repos: Vec<Repo>,
    pub repo_state: RepoListState,
    pub new_repo_name: String,
    pub edit_buffer: String,
    pub name_confirmed: bool,
    pub preview: Option<PreviewState>,
    pub status_items: Vec<StatusItem>,
    pub dry_run_lines: Vec<String>,
    pub mode: Mode,
    pub should_quit: bool,
    pub committed: bool,
    pub pushed: bool,
    pub last_commit_message: Option<String>,
    pub log_path: std::path::PathBuf,
    pub log_state: LogState,
    pub show_help: bool,
    pub github_token: String,
    pub github_login: String,
    pub repo_name_available: Option<bool>,
    pub remote_safety_lines: Vec<String>,
    pub active_pane: ActivePane,
    pub tree_state: TreeListState,
    pub startup_tree_lines: Vec<String>,
    pub busy_message: Option<String>,
    pub busy_started_at: Option<Instant>,
    pub is_focused: bool,
    pub frame_area: Rect,
    pub repo_filter: String,
    pub filter_state: RepoListState,
    pub readme_preview_title: String,
    pub readme_preview_markdown: String,
    pub(crate) startup_error: Option<String>,
    cache_index: CacheIndex,
    prefetch_rx: Option<Receiver<PrefetchUpdate>>,
    startup_rx: Option<Receiver<std::result::Result<StartupData, String>>>,
    filter_restore_state: Option<RepoListState>,
    pending_action: Option<PendingAction>,
    log_file_line_count: usize,
}

impl App {
    pub fn new(log_path: std::path::PathBuf, log_lines: Vec<String>) -> Self {
        let startup_rx = Self::spawn_startup_loader(log_path.clone());
        Self::new_with_startup_receiver(log_path, log_lines, startup_rx)
    }

    pub(crate) fn new_with_startup_receiver(
        log_path: std::path::PathBuf,
        log_lines: Vec<String>,
        startup_rx: Option<Receiver<std::result::Result<StartupData, String>>>,
    ) -> Self {
        let cache_index = Self::load_cache_index().unwrap_or_default();
        let log_file_line_count = fs::read_to_string(&log_path)
            .map(|text| text.lines().count())
            .unwrap_or(log_lines.len());
        let mut app = Self {
            repo_state: RepoListState::new(false),
            repos: Vec::new(),
            new_repo_name: DEFAULT_NEW_REPO.to_string(),
            edit_buffer: DEFAULT_NEW_REPO.to_string(),
            name_confirmed: false,
            preview: None,
            status_items: vec![
                StatusItem::new(false, "移動 : h / l / ← / → で pane を移動します"),
                StatusItem::new(
                    false,
                    "clone : repos で Enter を押して選択した repo を clone します",
                ),
                StatusItem::new(false, "new repo name : n で新しい repo 名を編集します"),
                StatusItem::new(false, "commit : c で local commit を実行します"),
                StatusItem::new(false, "push : Shift + P で検証して push します"),
            ],
            dry_run_lines: Vec::new(),
            mode: Mode::Normal,
            should_quit: false,
            committed: false,
            pushed: false,
            last_commit_message: None,
            log_path,
            log_state: LogState::new(log_lines),
            show_help: false,
            github_token: String::new(),
            github_login: String::new(),
            repo_name_available: None,
            remote_safety_lines: vec!["origin の安全性は未評価です".to_string()],
            active_pane: ActivePane::Repos,
            tree_state: TreeListState::new(false),
            startup_tree_lines: Vec::new(),
            busy_message: None,
            busy_started_at: None,
            is_focused: true,
            frame_area: Rect::new(0, 0, 0, 0),
            repo_filter: String::new(),
            filter_state: RepoListState::new(false),
            readme_preview_title: "README.ja.md".to_string(),
            readme_preview_markdown: "起動処理を読み込んでいます...".to_string(),
            startup_error: None,
            cache_index,
            prefetch_rx: None,
            startup_rx,
            filter_restore_state: None,
            pending_action: None,
            log_file_line_count,
        };
        app.tree_state.sync_len(app.current_tree_lines().len());
        app.refresh_status_items();
        if !app.is_startup_loading() {
            let _ = app.refresh_selected_repo_readme_preview();
        }
        app
    }

    pub(crate) fn is_startup_loading(&self) -> bool {
        self.startup_rx.is_some()
    }

    pub(crate) fn apply_startup_data(&mut self, data: StartupData) {
        self.github_token = data.github_token;
        self.github_login = data.github_login;
        self.repos = data.repos;
        self.startup_tree_lines = data.startup_tree_lines;
        self.startup_error = None;
        self.repo_state = RepoListState::new(!self.repos.is_empty());
        self.filter_state = RepoListState::new(!self.filtered_repo_indices().is_empty());
        self.ensure_tree_state_initialized();
    }

    pub(crate) fn set_startup_error(&mut self, message: impl Into<String>) {
        let message = message.into();
        self.startup_error = Some(message);
        self.readme_preview_title = "startup error".to_string();
        self.readme_preview_markdown = "起動処理に失敗しました。log を確認してください".to_string();
    }

    fn ensure_tree_state_initialized(&mut self) {
        let tree_len = self.current_tree_lines().len();
        self.tree_state.sync_len(tree_len);
        if self.tree_state.list.selected().is_none() && tree_len > 0 {
            self.tree_state = TreeListState::new(true);
        }
    }

    pub fn run_tui(&mut self) -> Result<()> {
        self.log("TUI の初期化が完了しました");
        enable_raw_mode()?;
        let mut out = stdout();
        execute!(
            out,
            EnterAlternateScreen,
            EnableMouseCapture,
            EnableFocusChange
        )?;
        let backend = CrosstermBackend::new(out);
        let mut terminal = Terminal::new(backend)?;
        let res = self.run_loop(&mut terminal);
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture,
            DisableFocusChange
        )?;
        terminal.show_cursor()?;
        let _ = self.save_cache_index();
        res
    }

    fn run_loop(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    ) -> Result<()> {
        loop {
            self.drain_startup_updates();
            self.drain_prefetch_updates();
            self.sync_log_view_from_file();
            self.normalize_active_pane();
            self.refresh_status_items();
            self.frame_area = terminal.size()?.into();
            terminal.draw(|f| draw_ui(f, self))?;
            if let Some(action) = self.pending_action.take() {
                if let Err(err) = self.run_pending_action(action) {
                    self.log(format!("error: {:#}", err));
                }
                self.busy_message = None;
                self.busy_started_at = None;
                continue;
            }
            if self.should_quit {
                self.log("アプリを終了します");
                return Ok(());
            }
            if event::poll(Duration::from_millis(200))? {
                match event::read()? {
                    Event::Key(key) => {
                        if let Err(err) = self.handle_key(key) {
                            self.log(format!("error: {:#}", err));
                        }
                    }
                    Event::Mouse(mouse) => {
                        let size = terminal.size()?;
                        self.handle_mouse(mouse, size.into());
                    }
                    Event::FocusGained => self.is_focused = true,
                    Event::FocusLost => self.is_focused = false,
                    _ => {}
                }
            }
        }
    }

    pub fn log<S: Into<String>>(&mut self, msg: S) {
        let line = msg.into();
        let stamped = format!("[{}] {}", now_string(), line);
        if append_log_line(&self.log_path, &stamped).is_ok() {
            self.log_file_line_count = self.log_file_line_count.saturating_add(1);
        }
        self.log_state.on_new_line(stamped);
    }

    fn selected_repo(&self) -> Option<&Repo> {
        self.repos.get(self.repo_state.selected)
    }

    pub fn filtered_repo_indices(&self) -> Vec<usize> {
        if self.repo_filter.trim().is_empty() {
            return (0..self.repos.len()).collect();
        }
        let terms = self
            .repo_filter
            .split_whitespace()
            .map(|term| term.to_lowercase())
            .collect::<Vec<_>>();
        self.repos
            .iter()
            .enumerate()
            .filter_map(|(index, repo)| {
                let haystack = repo.full_name.to_lowercase();
                if terms.iter().all(|term| haystack.contains(term)) {
                    Some(index)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn current_tree_lines(&self) -> &[String] {
        if let Some(preview) = &self.preview {
            &preview.file_list
        } else {
            &self.startup_tree_lines
        }
    }

    fn refresh_status_items(&mut self) {
        if self.status_items.len() != 5 {
            self.status_items = vec![
                StatusItem::new(false, "移動 : h / l / ← / → で pane を移動します"),
                StatusItem::new(
                    false,
                    "clone : repos で Enter を押して選択した repo を clone します",
                ),
                StatusItem::new(false, "new repo name : n で新しい repo 名を編集します"),
                StatusItem::new(false, "commit : c で local commit を実行します"),
                StatusItem::new(false, "push : Shift + P で検証して push します"),
            ];
        }
        self.status_items[0].done = false;
        self.status_items[1].done = self.preview.is_some();
        self.status_items[2].done = self.name_confirmed && self.repo_name_available == Some(true);
        self.status_items[3].done = self.committed;
        self.status_items[4].done = self.pushed;
    }
}
