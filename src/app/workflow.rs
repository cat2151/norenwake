use super::*;

impl App {
    pub(super) fn clone_selected_repo(&mut self) -> Result<()> {
        let repo = self
            .selected_repo()
            .cloned()
            .ok_or_else(|| anyhow!("repo がありません"))?;
        let work_base = work_dir()?;
        fs::create_dir_all(&work_base)?;
        self.log(format!("clone を開始します: {}", repo.full_name));
        let preview_dir = work_base.join(DEFAULT_NEW_REPO);
        if preview_dir.exists() {
            fs::remove_dir_all(&preview_dir).with_context(|| {
                format!(
                    "既存 preview の削除に失敗しました: {}",
                    preview_dir.display()
                )
            })?;
        }
        run_cmd_logged(
            &self.log_path,
            &work_base,
            "git",
            &["clone", &repo.clone_url, DEFAULT_NEW_REPO],
        )?;
        ensure_safe_remotes(
            &self.log_path,
            &preview_dir,
            &repo.clone_url,
            &build_target_origin_url(&self.github_login, DEFAULT_NEW_REPO),
            &build_target_origin_push_url(&self.github_login, DEFAULT_NEW_REPO),
        )?;
        update_readme_ja(&preview_dir, &repo.name, DEFAULT_NEW_REPO)?;
        self.preview = Some(PreviewState {
            source_repo: repo.clone(),
            preview_dir,
            file_list: Vec::new(),
            readme_delta_ansi: String::new(),
            readme_delta: Vec::new(),
            origin_url: None,
        });
        self.new_repo_name = DEFAULT_NEW_REPO.to_string();
        self.edit_buffer = DEFAULT_NEW_REPO.to_string();
        self.name_confirmed = false;
        self.committed = false;
        self.pushed = false;
        self.last_commit_message = None;
        self.repo_name_available = Some(check_repo_name_available(
            &self.github_token,
            &self.github_login,
            DEFAULT_NEW_REPO,
        )?);
        self.refresh_preview_metadata()?;
        self.refresh_selected_repo_readme_preview()?;
        self.log(format!("clone が完了しました: {}", repo.full_name));
        Ok(())
    }

    pub(super) fn refresh_preview_metadata(&mut self) -> Result<()> {
        if let Some(preview) = self.preview.as_mut() {
            preview.file_list = collect_files(&preview.preview_dir, 300)?;
            let (ansi, plain) = render_delta_for_file(&preview.preview_dir, "README.ja.md")?;
            preview.readme_delta_ansi = ansi;
            preview.readme_delta = plain;
            preview.origin_url = git_remote_url(&preview.preview_dir, "origin")?;
            let origin_push = git_remote_push_url(&preview.preview_dir, "origin")?;
            let upstream = git_remote_url(&preview.preview_dir, "upstream")?;
            self.remote_safety_lines = build_remote_safety_lines(
                &preview.source_repo.clone_url,
                preview.origin_url.as_deref(),
                origin_push.as_deref(),
                upstream.as_deref(),
                &build_target_origin_url(&self.github_login, &self.new_repo_name),
                &build_target_origin_push_url(&self.github_login, &self.new_repo_name),
            );
        }
        self.tree_state.sync_len(self.current_tree_lines().len());
        Ok(())
    }

    pub(super) fn begin_edit_name(&mut self) -> Result<()> {
        if self.preview.is_none() {
            return Err(anyhow!("先に Enter で repo を clone してください"));
        }
        self.edit_buffer = self.new_repo_name.clone();
        self.mode = Mode::EditingName;
        self.log(format!(
            "新しい repo 名の編集を開始します: {}",
            self.edit_buffer
        ));
        Ok(())
    }

    pub(super) fn confirm_new_repo_name(&mut self) -> Result<()> {
        let repo_name = self.edit_buffer.trim().to_string();
        if repo_name.is_empty() {
            return Err(anyhow!("新しい repo 名が空です"));
        }
        let available =
            check_repo_name_available(&self.github_token, &self.github_login, &repo_name)?;
        self.repo_name_available = Some(available);
        if !available {
            self.name_confirmed = false;
            self.log(format!(
                "repo 名が衝突しています。別の名前にしてください: {}",
                repo_name
            ));
            return Err(anyhow!(
                "repo 名がすでに存在します。別の名前に変更してください"
            ));
        }
        let (current_dir, source_name, source_clone_url) = {
            let preview = self
                .preview
                .as_ref()
                .ok_or_else(|| anyhow!("先に clone が必要です"))?;
            (
                preview.preview_dir.clone(),
                preview.source_repo.name.clone(),
                preview.source_repo.clone_url.clone(),
            )
        };
        let renamed_dir = rename_preview_dir(&current_dir, &repo_name)?;
        ensure_safe_remotes(
            &self.log_path,
            &renamed_dir,
            &source_clone_url,
            &build_target_origin_url(&self.github_login, &repo_name),
            &build_target_origin_push_url(&self.github_login, &repo_name),
        )?;
        update_readme_ja(&renamed_dir, &source_name, &repo_name)?;
        if let Some(preview) = self.preview.as_mut() {
            preview.preview_dir = renamed_dir;
        }
        self.new_repo_name = repo_name.clone();
        self.name_confirmed = repo_name != DEFAULT_NEW_REPO;
        self.committed = false;
        self.pushed = false;
        self.last_commit_message = None;
        self.refresh_preview_metadata()?;
        self.refresh_selected_repo_readme_preview()?;
        self.log(format!(
            "新しい repo 名を確定しました: {}",
            self.new_repo_name
        ));
        Ok(())
    }

    pub(super) fn commit_changes(&mut self) -> Result<()> {
        if !self.name_confirmed {
            return Err(anyhow!(
                "新しい repo 名が未決定です。仮名のまま commit しないでください"
            ));
        }
        let preview = self
            .preview
            .as_ref()
            .ok_or_else(|| anyhow!("先に clone が必要です"))?;
        run_cmd_logged(&self.log_path, &preview.preview_dir, "git", &["add", "."])?;
        let status = Command::new("git")
            .current_dir(&preview.preview_dir)
            .args(["diff", "--cached", "--quiet"])
            .status()
            .context("`git diff --cached` の実行に失敗しました")?;
        if status.success() {
            return Err(anyhow!("commit 対象の staged change がありません"));
        }
        let message = format!("docs: initialize norenwake split as {}", self.new_repo_name);
        run_cmd_logged(
            &self.log_path,
            &preview.preview_dir,
            "git",
            &["commit", "-m", &message],
        )?;
        self.committed = true;
        self.pushed = false;
        self.last_commit_message = Some(message.clone());
        self.refresh_preview_metadata()?;
        self.log(format!("commit が完了しました: {}", message));
        Ok(())
    }

    pub(super) fn validate_before_push(&mut self) -> Result<()> {
        if self.preview.is_none() {
            return Err(anyhow!("Enter で repo を clone してください"));
        }
        if !self.name_confirmed {
            return Err(anyhow!("新しい repo 名が未決定です"));
        }
        if !self.committed {
            return Err(anyhow!("commit がまだです"));
        }
        if self.repo_name_available != Some(true) {
            return Err(anyhow!("repo 名が衝突しています"));
        }
        let preview = self.preview.as_ref().unwrap();
        let origin = git_remote_url(&preview.preview_dir, "origin")?;
        let origin_push = git_remote_push_url(&preview.preview_dir, "origin")?;
        let expected_origin = build_target_origin_url(&self.github_login, &self.new_repo_name);
        let expected_push = build_target_origin_push_url(&self.github_login, &self.new_repo_name);
        match origin {
            Some(ref url) if *url == preview.source_repo.clone_url => {
                return Err(anyhow!(
                    "ハードガードです。origin が暖簾分け元を向いているため push できません"
                ));
            }
            Some(ref url) if *url != expected_origin => {
                return Err(anyhow!("origin が想定 push 先を向いていません"));
            }
            None => return Err(anyhow!("origin が存在しません")),
            _ => {}
        }
        match origin_push {
            Some(ref url) if *url != expected_push => {
                return Err(anyhow!("push URL が想定の SSH 宛先を向いていません"));
            }
            None => return Err(anyhow!("push URL が存在しません")),
            _ => {}
        }
        let upstream = git_remote_url(&preview.preview_dir, "upstream")?;
        if upstream.is_some() {
            return Err(anyhow!(
                "upstream が残っています。origin のみで運用してください"
            ));
        }
        Ok(())
    }

    pub(super) fn prepare_push_confirm(&mut self) -> Result<()> {
        self.validate_before_push()?;
        let preview = self.preview.as_ref().unwrap();
        self.dry_run_lines = vec![
            "検証に合格しました".to_string(),
            format!("source repo: {}", preview.source_repo.full_name),
            format!("new repo:    {}", self.new_repo_name),
            "visibility:  public".to_string(),
            format!("branch:      {}", preview.source_repo.default_branch),
            format!("workdir:     {}", preview.preview_dir.display()),
            format!(
                "repo name validation: {}",
                if self.repo_name_available == Some(true) {
                    "OK: repo 名は未使用です"
                } else {
                    "NG: repo 名がすでに存在します"
                }
            ),
            format!(
                "origin target: {}",
                build_target_origin_url(&self.github_login, &self.new_repo_name)
            ),
            format!(
                "push target:   {}",
                build_target_origin_push_url(&self.github_login, &self.new_repo_name)
            ),
            format!(
                "origin current: {}",
                git_remote_url(&preview.preview_dir, "origin")?.unwrap_or_else(|| "(none)".to_string())
            ),
            format!(
                "push current:   {}",
                git_remote_push_url(&preview.preview_dir, "origin")?
                    .unwrap_or_else(|| "(none)".to_string())
            ),
            format!(
                "upstream current: {}",
                git_remote_url(&preview.preview_dir, "upstream")?
                    .unwrap_or_else(|| "(none)".to_string())
            ),
            "hard guard: origin が暖簾分け元を向いている場合と upstream が残っている場合は push を拒否します"
                .to_string(),
            "".to_string(),
            "README.ja.md diff preview:".to_string(),
        ];
        self.dry_run_lines
            .extend(preview.readme_delta.iter().take(20).cloned());
        self.dry_run_lines.push("".to_string());
        self.dry_run_lines
            .push("y で push し、N で中止します".to_string());
        self.mode = Mode::ConfirmPush;
        self.log("push 前の検証結果を表示しています");
        Ok(())
    }

    pub(super) fn execute_push(&mut self) -> Result<()> {
        self.validate_before_push()?;
        let preview = self
            .preview
            .as_ref()
            .ok_or_else(|| anyhow!("preview がありません"))?;
        let preview_dir = preview.preview_dir.clone();
        let source_clone_url = preview.source_repo.clone_url.clone();
        let expected_origin = build_target_origin_url(&self.github_login, &self.new_repo_name);
        let expected_push = build_target_origin_push_url(&self.github_login, &self.new_repo_name);
        self.log(format!("push を開始します: {}", self.new_repo_name));
        ensure_safe_remotes(
            &self.log_path,
            &preview_dir,
            &source_clone_url,
            &expected_origin,
            &expected_push,
        )?;
        if let Err(err) = run_cmd_logged(
            &self.log_path,
            &preview_dir,
            "gh",
            &["repo", "create", &self.new_repo_name, "--public"],
        ) {
            let message = format!("{:#}", err);
            if message.contains("already exists")
                || message.contains("name already exists on this account")
            {
                self.log("repo はすでに存在するため、作成をスキップします");
            } else {
                return Err(err);
            }
        }
        run_cmd_logged(
            &self.log_path,
            &preview_dir,
            "git",
            &["push", "-u", "origin", "--all"],
        )?;
        run_cmd_logged(
            &self.log_path,
            &preview_dir,
            "git",
            &["push", "origin", "--tags"],
        )?;
        self.pushed = true;
        self.log(format!("push が完了しました: {}", self.new_repo_name));
        Ok(())
    }

    pub(super) fn run_pending_action(&mut self, action: PendingAction) -> Result<()> {
        match action {
            PendingAction::CloneSelectedRepo => self.clone_selected_repo(),
            PendingAction::ConfirmNewRepoName => self.confirm_new_repo_name(),
            PendingAction::CommitChanges => self.commit_changes(),
            PendingAction::PushToOrigin => {
                let result = self.execute_push();
                if result.is_ok() {
                    self.mode = Mode::Normal;
                }
                result
            }
        }
    }
}
