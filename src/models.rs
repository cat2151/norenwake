use ratatui::widgets::ListState;
use serde::Deserialize;
use std::path::PathBuf;

pub const APP_NAME: &str = "norenwake";
pub const DEFAULT_NEW_REPO: &str = "my-new-repo";
pub const README_MESSAGE: &str =
    "元repoからcloneして暖簾分けしました。暖簾分け断面までの履歴を持っています。";
pub const MAX_LOG_LINES: usize = 5000;

#[derive(Debug, Clone, Deserialize)]
pub struct Repo {
    pub name: String,
    pub full_name: String,
    pub clone_url: String,
    pub default_branch: String,
    pub private: bool,
    pub fork: bool,
    pub archived: bool,
    pub updated_at: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PreviewState {
    pub source_repo: Repo,
    pub preview_dir: PathBuf,
    pub file_list: Vec<String>,
    pub readme_delta_ansi: String,
    pub readme_delta: Vec<String>,
    pub origin_url: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    EditingName,
    ConfirmPush,
    FilteringRepos,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivePane {
    Repos,
    DirTree,
    Log,
}

impl ActivePane {
    pub fn next(self) -> Self {
        match self {
            Self::Repos => Self::DirTree,
            Self::DirTree => Self::Log,
            Self::Log => Self::Repos,
        }
    }

    pub fn previous(self) -> Self {
        match self {
            Self::Repos => Self::Log,
            Self::DirTree => Self::Repos,
            Self::Log => Self::DirTree,
        }
    }
}

#[derive(Debug, Clone)]
pub struct StatusItem {
    pub done: bool,
    pub label: String,
}

impl StatusItem {
    pub fn new(done: bool, label: impl Into<String>) -> Self {
        Self {
            done,
            label: label.into(),
        }
    }
}

#[derive(Debug)]
pub struct LogState {
    pub lines: Vec<String>,
    pub scroll: u16,
}

impl LogState {
    pub fn new(lines: Vec<String>) -> Self {
        Self { lines, scroll: 0 }
    }

    pub fn on_new_line(&mut self, line: String) {
        self.lines.push(line);
        if self.lines.len() > MAX_LOG_LINES {
            let keep_from = self.lines.len() - MAX_LOG_LINES;
            self.lines.drain(0..keep_from);
        }
        self.scroll = self.max_scroll();
    }

    fn max_scroll(&self) -> u16 {
        self.lines.len().saturating_sub(1).min(u16::MAX as usize) as u16
    }
}

#[derive(Debug, Clone)]
pub struct RepoListState {
    pub selected: usize,
    pub list: ListState,
}

impl RepoListState {
    pub fn new(has_items: bool) -> Self {
        let mut list = ListState::default();
        if has_items {
            list.select(Some(0));
        }
        Self { selected: 0, list }
    }

    pub fn move_selection(&mut self, delta: isize, len: usize) {
        if len == 0 {
            return;
        }
        let max = len.saturating_sub(1) as isize;
        let next = (self.selected as isize + delta).clamp(0, max) as usize;
        self.selected = next;
        self.list.select(Some(next));
    }
}

#[derive(Debug)]
pub struct TreeListState {
    pub selected: usize,
    pub list: ListState,
}

impl TreeListState {
    pub fn new(has_items: bool) -> Self {
        let mut list = ListState::default();
        if has_items {
            list.select(Some(0));
        }
        Self { selected: 0, list }
    }

    pub fn move_selection(&mut self, delta: isize, len: usize) {
        if len == 0 {
            self.selected = 0;
            self.list.select(None);
            return;
        }
        let max = len.saturating_sub(1) as isize;
        let next = (self.selected as isize + delta).clamp(0, max) as usize;
        self.selected = next;
        self.list.select(Some(next));
    }

    pub fn sync_len(&mut self, len: usize) {
        if len == 0 {
            self.selected = 0;
            self.list.select(None);
            return;
        }
        if self.selected >= len {
            self.selected = len - 1;
        }
        self.list.select(Some(self.selected));
    }
}
