use super::*;

impl App {
    pub(super) fn sync_log_view_from_file(&mut self) {
        let Ok(text) = fs::read_to_string(&self.log_path) else {
            return;
        };
        let all_lines: Vec<String> = text.lines().map(ToString::to_string).collect();
        let total = all_lines.len();
        if total < self.log_file_line_count {
            self.log_state.lines = if total > MAX_LOG_LINES {
                all_lines[total - MAX_LOG_LINES..].to_vec()
            } else {
                all_lines
            };
            self.log_state.scroll = self
                .log_state
                .lines
                .len()
                .saturating_sub(1)
                .min(u16::MAX as usize) as u16;
            self.log_file_line_count = total;
            return;
        }
        if total == self.log_file_line_count {
            return;
        }
        for line in all_lines.iter().skip(self.log_file_line_count) {
            self.log_state.on_new_line(line.clone());
        }
        self.log_file_line_count = total;
    }

    pub(super) fn refresh_selected_repo_readme_preview(&mut self) -> Result<()> {
        if self.preview.is_some() {
            let readme = self.load_preview_readme()?;
            self.readme_preview_title = readme.title;
            self.readme_preview_markdown = if readme.markdown.trim().is_empty() {
                format!("{} は空です", self.readme_preview_title)
            } else {
                readme.markdown
            };
            return Ok(());
        }
        let Some(repo) = self.selected_repo().cloned() else {
            self.readme_preview_title = "README.ja.md".to_string();
            self.readme_preview_markdown = "README.ja.md preview はありません".to_string();
            return Ok(());
        };
        let readme = self.load_cached_or_fetch_readme(&repo)?;
        self.readme_preview_title = readme.title;
        self.readme_preview_markdown = if readme.markdown.trim().is_empty() {
            format!("{} は空です", self.readme_preview_title)
        } else {
            readme.markdown
        };
        Ok(())
    }

    pub(super) fn load_preview_readme(&self) -> Result<CachedReadme> {
        let preview = self
            .preview
            .as_ref()
            .ok_or_else(|| anyhow!("preview がありません"))?;
        let readme_ja_path = preview.preview_dir.join("README.ja.md");
        let readme_md_path = preview.preview_dir.join("README.md");
        if readme_ja_path.exists() {
            return Self::read_cached_readme("README.ja.md", &readme_ja_path);
        }
        if readme_md_path.exists() {
            return Self::read_cached_readme("README.md", &readme_md_path);
        }
        Ok(CachedReadme {
            title: "README.ja.md".to_string(),
            markdown: "README.ja.md / README.md が見つかりません".to_string(),
        })
    }

    pub(super) fn load_cached_or_fetch_readme(&mut self, repo: &Repo) -> Result<CachedReadme> {
        let cache_root = cache_dir()?;
        fs::create_dir_all(&cache_root)?;
        let repo_cache_dir = cache_root.join(&repo.name);
        fs::create_dir_all(&repo_cache_dir)?;
        let readme_ja_path = repo_cache_dir.join("README.ja.md");
        let readme_md_path = repo_cache_dir.join("README.md");
        let current = self.cache_index.repos.get(&repo.name);
        if current == Some(&repo.updated_at) {
            if readme_ja_path.exists() {
                return Self::read_cached_readme("README.ja.md", &readme_ja_path);
            }
            if readme_md_path.exists() {
                return Self::read_cached_readme("README.md", &readme_md_path);
            }
        }
        match fetch_repo_readme_ja(&self.github_token, &repo.full_name) {
            Ok(readme) => {
                let (write_path, stale_path) = if readme.file_name == "README.ja.md" {
                    (&readme_ja_path, &readme_md_path)
                } else {
                    (&readme_md_path, &readme_ja_path)
                };
                fs::write(write_path, &readme.markdown).with_context(|| {
                    format!(
                        "cache README の書き込みに失敗しました: {}",
                        write_path.display()
                    )
                })?;
                if stale_path.exists() {
                    let _ = fs::remove_file(stale_path);
                }
                self.cache_index
                    .repos
                    .insert(repo.name.clone(), repo.updated_at.clone());
                Ok(CachedReadme {
                    title: readme.file_name.to_string(),
                    markdown: readme.markdown,
                })
            }
            Err(_) => {
                if readme_ja_path.exists() {
                    return Self::read_cached_readme("README.ja.md", &readme_ja_path);
                }
                if readme_md_path.exists() {
                    return Self::read_cached_readme("README.md", &readme_md_path);
                }
                Ok(CachedReadme {
                    title: "README.ja.md".to_string(),
                    markdown: "README.ja.md / README.md を取得できませんでした".to_string(),
                })
            }
        }
    }

    pub(super) fn read_cached_readme(title: &str, path: &std::path::Path) -> Result<CachedReadme> {
        let markdown = fs::read_to_string(path).with_context(|| {
            format!("cache README の読み込みに失敗しました: {}", path.display())
        })?;
        Ok(CachedReadme {
            title: title.to_string(),
            markdown,
        })
    }

    pub(super) fn load_cache_index() -> Result<CacheIndex> {
        let path = cache_index_path()?;
        if !path.exists() {
            return Ok(CacheIndex::default());
        }
        let text = fs::read_to_string(&path)
            .with_context(|| format!("cache index の読み込みに失敗しました: {}", path.display()))?;
        Ok(serde_json::from_str(&text).unwrap_or_default())
    }

    pub(super) fn save_cache_index(&self) -> Result<()> {
        let path = cache_index_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let text = serde_json::to_string_pretty(&self.cache_index)?;
        fs::write(&path, text)
            .with_context(|| format!("cache index の書き込みに失敗しました: {}", path.display()))
    }

    pub(super) fn spawn_readme_prefetch(
        repos: Vec<Repo>,
        token: String,
        known_updates: BTreeMap<String, String>,
    ) -> Option<Receiver<PrefetchUpdate>> {
        let cache_root = cache_dir().ok()?;
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let _ = fs::create_dir_all(&cache_root);
            for repo in repos {
                let repo_dir = cache_root.join(&repo.name);
                let readme_ja_path = repo_dir.join("README.ja.md");
                let readme_md_path = repo_dir.join("README.md");
                let has_cache = readme_ja_path.exists() || readme_md_path.exists();
                let is_fresh = known_updates.get(&repo.name) == Some(&repo.updated_at) && has_cache;
                if is_fresh {
                    continue;
                }
                if fs::create_dir_all(&repo_dir).is_err() {
                    continue;
                }
                if let Ok(readme) = fetch_repo_readme_ja(&token, &repo.full_name) {
                    let (write_path, stale_path) = if readme.file_name == "README.ja.md" {
                        (&readme_ja_path, &readme_md_path)
                    } else {
                        (&readme_md_path, &readme_ja_path)
                    };
                    if fs::write(write_path, readme.markdown).is_ok() {
                        if stale_path.exists() {
                            let _ = fs::remove_file(stale_path);
                        }
                        let _ = tx.send(PrefetchUpdate {
                            repo_name: repo.name,
                            updated_at: repo.updated_at,
                        });
                    }
                }
            }
        });
        Some(rx)
    }

    pub(super) fn drain_prefetch_updates(&mut self) {
        let Some(rx) = &self.prefetch_rx else {
            return;
        };
        let mut updates = Vec::new();
        while let Ok(update) = rx.try_recv() {
            updates.push(update);
        }
        for update in updates {
            self.cache_index
                .repos
                .insert(update.repo_name.clone(), update.updated_at);
            if self.selected_repo().map(|repo| repo.name.as_str())
                == Some(update.repo_name.as_str())
            {
                let _ = self.refresh_selected_repo_readme_preview();
            }
        }
    }
}
