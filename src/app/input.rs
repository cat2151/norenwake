use super::*;

impl App {
    pub(super) fn handle_key(&mut self, key: KeyEvent) -> Result<()> {
        if key.kind == KeyEventKind::Release {
            return Ok(());
        }

        if self.show_help {
            if key.kind != KeyEventKind::Press {
                return Ok(());
            }
            match key.code {
                KeyCode::Char('?') | KeyCode::Esc | KeyCode::Enter => self.show_help = false,
                _ => {}
            }
            return Ok(());
        }
        match self.mode {
            Mode::EditingName => return self.handle_editing_name(key),
            Mode::ConfirmPush => return self.handle_confirm_push(key),
            Mode::FilteringRepos => return self.handle_filtering_repos(key),
            Mode::Normal => {}
        }

        let is_press = key.kind == KeyEventKind::Press;
        let is_repeat = key.kind == KeyEventKind::Repeat;
        match key.code {
            KeyCode::Char('q') if is_press => self.should_quit = true,
            KeyCode::Char('?') if is_press => self.show_help = true,
            KeyCode::Char('/') if is_press => {
                self.active_pane = ActivePane::Repos;
                self.mode = Mode::FilteringRepos;
                self.filter_restore_state = Some(self.repo_state.clone());
                self.filter_state = RepoListState::new(!self.filtered_repo_indices().is_empty());
            }
            KeyCode::Char('h') if is_press => self.move_active_pane_previous(),
            KeyCode::Char('l') if is_press => self.move_active_pane_next(),
            KeyCode::Left if is_press => self.move_active_pane_previous(),
            KeyCode::Right if is_press => self.move_active_pane_next(),
            KeyCode::Down | KeyCode::Char('j') if is_press || is_repeat => match self.active_pane {
                ActivePane::Repos => {
                    self.repo_state
                        .move_selection(1, self.filtered_repo_indices().len());
                    let _ = self.refresh_selected_repo_readme_preview();
                }
                ActivePane::DirTree => self
                    .tree_state
                    .move_selection(1, self.current_tree_lines().len()),
                ActivePane::Log => self.scroll_log_down(1),
            },
            KeyCode::Up | KeyCode::Char('k') if is_press || is_repeat => match self.active_pane {
                ActivePane::Repos => {
                    self.repo_state
                        .move_selection(-1, self.filtered_repo_indices().len());
                    let _ = self.refresh_selected_repo_readme_preview();
                }
                ActivePane::DirTree => self
                    .tree_state
                    .move_selection(-1, self.current_tree_lines().len()),
                ActivePane::Log => self.scroll_log_up(1),
            },
            KeyCode::Enter if is_press && self.active_pane == ActivePane::Repos => {
                self.active_pane = ActivePane::Repos;
                self.busy_message = Some("Cloning repository...".to_string());
                self.busy_started_at = Some(Instant::now());
                self.pending_action = Some(PendingAction::CloneSelectedRepo);
            }
            KeyCode::Char('n') if is_press => {
                self.active_pane = ActivePane::Repos;
                self.begin_edit_name()?;
            }
            KeyCode::Char('c') if is_press => {
                self.active_pane = ActivePane::Repos;
                self.busy_message = Some("commit を実行しています...".to_string());
                self.busy_started_at = Some(Instant::now());
                self.pending_action = Some(PendingAction::CommitChanges);
            }
            KeyCode::Char('L') if key.modifiers.contains(KeyModifiers::SHIFT) && is_press => {
                self.active_pane = ActivePane::Log;
                self.copy_logs_to_clipboard()?;
            }
            KeyCode::PageUp if is_press || is_repeat => match self.active_pane {
                ActivePane::Repos => {
                    self.repo_state
                        .move_selection(-10, self.filtered_repo_indices().len());
                    let _ = self.refresh_selected_repo_readme_preview();
                }
                ActivePane::DirTree => self
                    .tree_state
                    .move_selection(-10, self.current_tree_lines().len()),
                ActivePane::Log => self.scroll_log_up(10),
            },
            KeyCode::PageDown if is_press || is_repeat => match self.active_pane {
                ActivePane::Repos => {
                    self.repo_state
                        .move_selection(10, self.filtered_repo_indices().len());
                    let _ = self.refresh_selected_repo_readme_preview();
                }
                ActivePane::DirTree => self
                    .tree_state
                    .move_selection(10, self.current_tree_lines().len()),
                ActivePane::Log => self.scroll_log_down(10),
            },
            KeyCode::Char('u')
                if key.modifiers.contains(KeyModifiers::CONTROL) && (is_press || is_repeat) =>
            {
                self.active_pane = ActivePane::Log;
                self.scroll_log_up(10);
            }
            KeyCode::Char('d')
                if key.modifiers.contains(KeyModifiers::CONTROL) && (is_press || is_repeat) =>
            {
                self.active_pane = ActivePane::Log;
                self.scroll_log_down(10);
            }
            KeyCode::Char('P') if key.modifiers.contains(KeyModifiers::SHIFT) && is_press => {
                self.active_pane = ActivePane::Repos;
                self.prepare_push_confirm()?;
            }
            _ => {}
        }
        Ok(())
    }

    pub(super) fn handle_editing_name(&mut self, key: KeyEvent) -> Result<()> {
        if key.kind != KeyEventKind::Press {
            return Ok(());
        }
        match key.code {
            KeyCode::Esc => {
                self.mode = Mode::Normal;
                self.log("repo 名の変更を中止しました");
            }
            KeyCode::Enter => {
                self.mode = Mode::Normal;
                self.busy_message = Some("new repo name を確定しています...".to_string());
                self.busy_started_at = Some(Instant::now());
                self.pending_action = Some(PendingAction::ConfirmNewRepoName);
            }
            KeyCode::Backspace => {
                self.edit_buffer.pop();
            }
            KeyCode::Char(ch) => {
                self.edit_buffer.push(ch);
            }
            _ => {}
        }
        Ok(())
    }

    pub(super) fn handle_confirm_push(&mut self, key: KeyEvent) -> Result<()> {
        if key.kind != KeyEventKind::Press {
            return Ok(());
        }
        match key.code {
            KeyCode::Char('y') => {
                self.busy_message = Some("pushしています...".to_string());
                self.busy_started_at = Some(Instant::now());
                self.pending_action = Some(PendingAction::PushToOrigin);
            }
            KeyCode::Char('N') | KeyCode::Char('n') | KeyCode::Esc => {
                self.mode = Mode::Normal;
                self.log("push を中止しました");
            }
            _ => {}
        }
        Ok(())
    }

    pub(super) fn handle_filtering_repos(&mut self, key: KeyEvent) -> Result<()> {
        if key.kind != KeyEventKind::Press {
            return Ok(());
        }
        match key.code {
            KeyCode::Esc => {
                self.mode = Mode::Normal;
                self.repo_filter.clear();
                if let Some(state) = self.filter_restore_state.take() {
                    self.repo_state = state;
                }
                self.refresh_selected_repo_readme_preview()?;
            }
            KeyCode::Enter => {
                let indices = self.filtered_repo_indices();
                if !indices.is_empty() {
                    let visible_index = self
                        .filter_state
                        .selected
                        .min(indices.len().saturating_sub(1));
                    if let Some(repo_index) = indices.get(visible_index) {
                        self.repo_state.selected = *repo_index;
                        self.repo_state.list.select(Some(self.repo_state.selected));
                    }
                }
                self.repo_filter.clear();
                self.mode = Mode::Normal;
                self.filter_restore_state = None;
                self.center_repo_selection();
                self.refresh_selected_repo_readme_preview()?;
            }
            KeyCode::Backspace => {
                self.repo_filter.pop();
                self.sync_filter_state();
            }
            KeyCode::Down | KeyCode::Char('j') => self
                .filter_state
                .move_selection(1, self.filtered_repo_indices().len()),
            KeyCode::Up | KeyCode::Char('k') => self
                .filter_state
                .move_selection(-1, self.filtered_repo_indices().len()),
            KeyCode::PageUp => self
                .filter_state
                .move_selection(-10, self.filtered_repo_indices().len()),
            KeyCode::PageDown => self
                .filter_state
                .move_selection(10, self.filtered_repo_indices().len()),
            KeyCode::Char(ch) => {
                self.repo_filter.push(ch);
                self.sync_filter_state();
            }
            _ => {}
        }
        Ok(())
    }

    pub(super) fn copy_logs_to_clipboard(&mut self) -> Result<()> {
        let all = self.log_state.lines.join("\n");
        let mut clipboard = Clipboard::new().context("clipboard を開けません")?;
        clipboard
            .set_text(all)
            .context("clipboard へのコピーに失敗しました")?;
        self.log("ログ全体を clipboard へコピーしました");
        Ok(())
    }

    pub(super) fn sync_filter_state(&mut self) {
        let len = self.filtered_repo_indices().len();
        if len == 0 {
            self.filter_state = RepoListState::new(false);
            return;
        }
        if self.filter_state.selected >= len {
            self.filter_state.selected = len - 1;
        }
        self.filter_state
            .list
            .select(Some(self.filter_state.selected));
    }

    pub(super) fn handle_mouse(&mut self, mouse: MouseEvent, area: Rect) {
        if mouse.kind != MouseEventKind::Down(MouseButton::Left) {
            return;
        }
        let rects = self.layout_rects(area);
        let x = mouse.column;
        let y = mouse.row;
        if contains(rects.repos, x, y) {
            self.active_pane = ActivePane::Repos;
        } else if rects.dir_tree.is_some_and(|rect| contains(rect, x, y)) {
            self.active_pane = ActivePane::DirTree;
        } else if contains(rects.log, x, y) {
            self.active_pane = ActivePane::Log;
        }
    }

    pub(super) fn layout_rects(&self, area: Rect) -> LayoutRects {
        let show_dir_tree = self.has_dir_tree_pane();
        let show_validation = show_dir_tree && self.name_confirmed;
        let root = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Percentage(48),
                Constraint::Percentage(47),
                Constraint::Length(3),
            ])
            .split(area);
        let top = if !show_dir_tree {
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(18),
                    Constraint::Percentage(30),
                    Constraint::Percentage(52),
                ])
                .split(root[1])
        } else if show_validation {
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(14),
                    Constraint::Percentage(19),
                    Constraint::Percentage(20),
                    Constraint::Percentage(20),
                    Constraint::Percentage(27),
                ])
                .split(root[1])
        } else {
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(16),
                    Constraint::Percentage(22),
                    Constraint::Percentage(24),
                    Constraint::Percentage(38),
                ])
                .split(root[1])
        };
        let (repos, dir_tree) = if !show_dir_tree {
            (top[1], None)
        } else {
            (top[1], Some(top[2]))
        };
        LayoutRects {
            repos,
            dir_tree,
            log: root[2],
        }
    }

    pub(super) fn scroll_log_up(&mut self, n: u16) {
        self.log_state.scroll = self.log_state.scroll.saturating_sub(n);
    }

    pub(super) fn scroll_log_down(&mut self, n: u16) {
        let bottom_scroll = self.bottom_log_scroll();
        self.log_state.scroll = self.log_state.scroll.saturating_add(n);
        if self.log_state.scroll >= bottom_scroll {
            self.log_state.scroll = bottom_scroll;
        }
    }

    pub(super) fn bottom_log_scroll(&self) -> u16 {
        let total_lines = self.log_state.lines.len();
        let view_height = self.log_view_height() as usize;
        total_lines
            .saturating_sub(view_height)
            .min(u16::MAX as usize) as u16
    }

    pub(super) fn log_view_height(&self) -> u16 {
        self.layout_rects(self.frame_area)
            .log
            .height
            .saturating_sub(2)
            .max(1)
    }

    pub(super) fn center_repo_selection(&mut self) {
        let repos_rect = self.layout_rects(self.frame_area).repos;
        let visible_rows = repos_rect.height.saturating_sub(2) as usize;
        if visible_rows == 0 {
            return;
        }
        let half = visible_rows / 2;
        let offset = self.repo_state.selected.saturating_sub(half);
        *self.repo_state.list.offset_mut() = offset;
    }

    pub(super) fn has_dir_tree_pane(&self) -> bool {
        self.preview.is_some()
    }

    pub(super) fn normalize_active_pane(&mut self) {
        if !self.has_dir_tree_pane() && self.active_pane == ActivePane::DirTree {
            self.active_pane = ActivePane::Repos;
        }
    }

    pub(super) fn move_active_pane_next(&mut self) {
        self.active_pane = if self.has_dir_tree_pane() {
            self.active_pane.next()
        } else {
            match self.active_pane {
                ActivePane::Repos => ActivePane::Log,
                ActivePane::DirTree => ActivePane::Repos,
                ActivePane::Log => ActivePane::Repos,
            }
        };
    }

    pub(super) fn move_active_pane_previous(&mut self) {
        self.active_pane = if self.has_dir_tree_pane() {
            self.active_pane.previous()
        } else {
            match self.active_pane {
                ActivePane::Repos => ActivePane::Log,
                ActivePane::DirTree => ActivePane::Repos,
                ActivePane::Log => ActivePane::Repos,
            }
        };
    }
}

fn contains(rect: Rect, x: u16, y: u16) -> bool {
    x >= rect.x && x < rect.x + rect.width && y >= rect.y && y < rect.y + rect.height
}
