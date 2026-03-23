use super::*;
use crate::{
    github::{fetch_authenticated_login, fetch_repos, get_github_token},
    util::now_string,
};

impl App {
    pub(super) fn spawn_startup_loader(
        log_path: std::path::PathBuf,
    ) -> Option<Receiver<std::result::Result<StartupData, String>>> {
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let result = (|| -> Result<StartupData> {
                let token = get_github_token()?;
                append_log_line(
                    &log_path,
                    &format!(
                        "[{}] `gh auth token` で GitHub token を取得しました",
                        now_string()
                    ),
                )?;
                let github_login = fetch_authenticated_login(&token)?;
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
                let startup_tree_lines = Self::load_startup_tree()?;
                append_log_line(
                    &log_path,
                    &format!(
                        "[{}] 起動時ディレクトリ tree を {} 行読み込みました",
                        now_string(),
                        startup_tree_lines.len()
                    ),
                )?;
                Ok(StartupData {
                    github_token: token,
                    github_login,
                    repos,
                    startup_tree_lines,
                })
            })()
            .map_err(|err| format!("{:#}", err));
            let _ = tx.send(result);
        });
        Some(rx)
    }

    pub(crate) fn drain_startup_updates(&mut self) {
        let Some(rx) = self.startup_rx.take() else {
            return;
        };
        match rx.try_recv() {
            Ok(Ok(data)) => {
                self.apply_startup_data(data);
                self.prefetch_rx = Self::spawn_readme_prefetch(
                    self.repos.clone(),
                    self.github_token.clone(),
                    self.cache_index.repos.clone(),
                );
                let _ = self.refresh_selected_repo_readme_preview();
            }
            Ok(Err(err)) => {
                self.set_startup_error(err.clone());
                self.log(format!("error: 起動処理に失敗しました: {}", err));
            }
            Err(mpsc::TryRecvError::Empty) => {
                self.startup_rx = Some(rx);
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                self.set_startup_error("起動処理チャネルが切断されました");
                self.log("error: 起動処理チャネルが切断されました");
            }
        }
    }

    pub(super) fn load_startup_tree() -> Result<Vec<String>> {
        let work_base = work_dir()?;
        if !work_base.exists() {
            return Ok(Vec::new());
        }
        let preview_dir = work_base.join(DEFAULT_NEW_REPO);
        if preview_dir.exists() {
            fs::remove_dir_all(&preview_dir).with_context(|| {
                format!(
                    "一時 preview の削除に失敗しました: {}",
                    preview_dir.display()
                )
            })?;
        }

        let mut candidates = Vec::new();
        if work_base.join(".git").exists() {
            let modified = std::fs::metadata(&work_base)
                .and_then(|meta| meta.modified())
                .ok();
            candidates.push((work_base.clone(), modified));
        }

        for entry in std::fs::read_dir(&work_base)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() && path.join(".git").exists() {
                let modified = entry.metadata().and_then(|meta| meta.modified()).ok();
                candidates.push((path, modified));
            }
        }

        candidates.sort_by_key(|(_, modified)| *modified);
        if let Some((path, _)) = candidates.pop() {
            return collect_files(&path, 300);
        }
        Ok(Vec::new())
    }
}
