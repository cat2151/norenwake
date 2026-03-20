use super::*;

impl App {
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
