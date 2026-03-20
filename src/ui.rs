mod geometry;
mod overlays;
mod panes;
mod theme;

use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::Style,
    widgets::Block,
    Frame,
};

use crate::{app::App, models::Mode};

use overlays::{
    draw_busy_overlay, draw_confirm, draw_dim_backdrop, draw_filter_overlay, draw_focus_dim,
    draw_help, draw_name_editor,
};
use panes::{
    draw_app_title, draw_file_list, draw_footer, draw_log, draw_readme_preview, draw_repo_list,
    draw_todo, draw_validation,
};
use theme::MONOKAI_BG;

pub fn draw_ui(frame: &mut Frame, app: &App) {
    frame.render_widget(
        Block::default().style(Style::default().bg(MONOKAI_BG)),
        frame.area(),
    );

    let show_dir_tree = app.preview.is_some();
    let show_validation = show_dir_tree && app.name_confirmed;
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Percentage(48),
            Constraint::Percentage(47),
            Constraint::Length(3),
        ])
        .split(frame.area());

    draw_app_title(frame, app, root[0]);

    if !show_dir_tree {
        let top = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(18),
                Constraint::Percentage(30),
                Constraint::Percentage(52),
            ])
            .split(root[1]);
        draw_todo(frame, app, top[0]);
        draw_repo_list(frame, app, top[1]);
        draw_readme_preview(frame, app, top[2]);
    } else if show_validation {
        let top = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(14),
                Constraint::Percentage(19),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Percentage(27),
            ])
            .split(root[1]);
        draw_todo(frame, app, top[0]);
        draw_repo_list(frame, app, top[1]);
        draw_file_list(frame, app, top[2]);
        draw_validation(frame, app, top[3]);
        draw_readme_preview(frame, app, top[4]);
    } else {
        let top = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(16),
                Constraint::Percentage(22),
                Constraint::Percentage(24),
                Constraint::Percentage(38),
            ])
            .split(root[1]);
        draw_todo(frame, app, top[0]);
        draw_repo_list(frame, app, top[1]);
        draw_file_list(frame, app, top[2]);
        draw_readme_preview(frame, app, top[3]);
    }

    draw_log(frame, app, root[2]);
    draw_footer(frame, app, root[3]);

    if !app.is_focused {
        draw_focus_dim(frame, frame.area());
    }
    if app.show_help {
        draw_dim_backdrop(frame, frame.area());
        draw_help(frame, frame.area());
    }
    if app.mode == Mode::FilteringRepos {
        draw_dim_backdrop(frame, frame.area());
        draw_filter_overlay(frame, app, frame.area());
    }
    if app.mode == Mode::EditingName {
        draw_dim_backdrop(frame, frame.area());
        draw_name_editor(frame, app, frame.area());
    }
    if app.mode == Mode::ConfirmPush {
        draw_dim_backdrop(frame, frame.area());
        draw_confirm(frame, app, frame.area());
    }
    if app.busy_message.is_some() {
        draw_dim_backdrop(frame, frame.area());
        draw_busy_overlay(
            frame,
            frame.area(),
            app.busy_started_at,
            app.busy_message.as_deref(),
        );
    }
}
