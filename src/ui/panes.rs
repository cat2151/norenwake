use ansi_to_tui::IntoText as _;
use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, Paragraph, Wrap},
    Frame,
};
use tui_markdown::from_str;

use crate::{app::App, models::ActivePane};

use super::theme::{
    focus_dim, focus_dim_line, pane_block, MONOKAI_ACTIVE, MONOKAI_DIM, MONOKAI_GREEN,
    MONOKAI_MUTED, MONOKAI_ORANGE, MONOKAI_PANEL, MONOKAI_PANEL_ALT, MONOKAI_RED, MONOKAI_ROW,
    MONOKAI_TEXT, MONOKAI_WHITE,
};

pub(super) fn draw_app_title(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let title = Paragraph::new("norenwake ～リポジトリ暖簾分け～").style(focus_dim(
        Style::default()
            .fg(MONOKAI_ORANGE)
            .add_modifier(Modifier::BOLD),
        app.is_focused,
    ));
    frame.render_widget(title, area);
}

pub(super) fn draw_todo(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let lines = app
        .status_items
        .iter()
        .map(|item| {
            let is_navigation_guide = item.label.starts_with("移動 :");
            let (prefix, color) = if is_navigation_guide {
                ("↔ ", MONOKAI_ORANGE)
            } else if item.done {
                ("✔ ", MONOKAI_GREEN)
            } else {
                ("☐ ", MONOKAI_MUTED)
            };
            Line::from(vec![
                Span::styled(
                    prefix,
                    focus_dim(
                        Style::default().fg(color).add_modifier(Modifier::BOLD),
                        app.is_focused,
                    ),
                ),
                Span::styled(
                    item.label.clone(),
                    focus_dim(Style::default().fg(MONOKAI_TEXT), app.is_focused),
                ),
            ])
        })
        .collect::<Vec<_>>();
    let todo = Paragraph::new(lines)
        .block(pane_block("TODO", false, app.is_focused))
        .style(focus_dim(
            Style::default().bg(MONOKAI_PANEL).fg(MONOKAI_TEXT),
            app.is_focused,
        ))
        .wrap(Wrap { trim: false });
    frame.render_widget(todo, area);
}

pub(super) fn draw_repo_list(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let active = app.active_pane == ActivePane::Repos;
    let filtered = app.filtered_repo_indices();
    let items: Vec<ListItem> = filtered
        .iter()
        .enumerate()
        .filter_map(|(visible_index, repo_index)| {
            app.repos
                .get(*repo_index)
                .map(|repo| (visible_index, *repo_index, repo))
        })
        .map(|(_visible_index, repo_index, repo)| {
            let is_selected = repo_index == app.repo_state.selected;
            let style = if active && is_selected {
                Style::default()
                    .fg(MONOKAI_ACTIVE)
                    .add_modifier(Modifier::BOLD)
            } else if active {
                Style::default().fg(MONOKAI_WHITE)
            } else {
                Style::default().fg(MONOKAI_DIM).add_modifier(Modifier::DIM)
            };
            ListItem::new(Line::from(Span::styled(
                &repo.name,
                focus_dim(style, app.is_focused),
            )))
        })
        .collect();

    let mut state = app.repo_state.list;
    frame.render_stateful_widget(
        List::new(items)
            .block(pane_block("repos", active, app.is_focused))
            .style(focus_dim(
                Style::default().bg(MONOKAI_PANEL).fg(MONOKAI_TEXT),
                app.is_focused,
            ))
            .highlight_symbol(if active { "› " } else { "  " })
            .highlight_style(focus_dim(Style::default().bg(MONOKAI_ROW), app.is_focused)),
        area,
        &mut state,
    );
}

pub(super) fn draw_readme_preview(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let text = from_str(&app.readme_preview_markdown);
    let preview = Paragraph::new(text)
        .block(pane_block(&app.readme_preview_title, false, app.is_focused))
        .style(focus_dim(
            Style::default().bg(MONOKAI_PANEL_ALT).fg(MONOKAI_TEXT),
            app.is_focused,
        ))
        .wrap(Wrap { trim: false });
    frame.render_widget(preview, area);
}

pub(super) fn draw_file_list(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let lines = app.current_tree_lines().to_vec();
    let items: Vec<ListItem> = lines
        .into_iter()
        .map(|line| {
            ListItem::new(Line::from(Span::styled(
                line,
                focus_dim(Style::default().fg(MONOKAI_TEXT), app.is_focused),
            )))
        })
        .collect();
    let mut state = app.tree_state.list;
    frame.render_stateful_widget(
        List::new(items)
            .block(pane_block(
                "dir tree",
                app.active_pane == ActivePane::DirTree,
                app.is_focused,
            ))
            .style(focus_dim(
                Style::default().bg(MONOKAI_PANEL).fg(MONOKAI_TEXT),
                app.is_focused,
            ))
            .highlight_symbol(if app.active_pane == ActivePane::DirTree {
                "› "
            } else {
                "  "
            })
            .highlight_style(focus_dim(Style::default().bg(MONOKAI_ROW), app.is_focused)),
        area,
        &mut state,
    );
}

pub(super) fn draw_validation(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let mut lines: Vec<Line> = vec![Line::from(Span::styled(
        format!("repo name: {}", app.new_repo_name),
        focus_dim(Style::default().fg(MONOKAI_TEXT), app.is_focused),
    ))];
    let (available_label, available_color) = match app.repo_name_available {
        Some(true) => ("name available: OK", MONOKAI_GREEN),
        Some(false) => ("name available: NG", MONOKAI_RED),
        None => ("name available: pending", MONOKAI_MUTED),
    };
    lines.push(Line::from(Span::styled(
        available_label,
        focus_dim(Style::default().fg(available_color), app.is_focused),
    )));
    lines.push(Line::from(""));
    lines.extend(app.remote_safety_lines.iter().map(|line| {
        let color = if line.contains("OK:") {
            MONOKAI_GREEN
        } else if line.contains("NG:") {
            MONOKAI_RED
        } else {
            MONOKAI_TEXT
        };
        Line::from(Span::styled(
            line.clone(),
            focus_dim(Style::default().fg(color), app.is_focused),
        ))
    }));
    if let Some(preview) = app.preview.as_ref() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "README.ja.md delta:",
            focus_dim(
                Style::default()
                    .fg(MONOKAI_ORANGE)
                    .add_modifier(Modifier::BOLD),
                app.is_focused,
            ),
        )));
        if preview.readme_delta_ansi.trim().is_empty() {
            lines.push(Line::from(Span::styled(
                "(差分はありません)",
                focus_dim(Style::default().fg(MONOKAI_MUTED), app.is_focused),
            )));
        } else if let Ok(delta_text) = preview.readme_delta_ansi.as_str().into_text() {
            lines.extend(
                delta_text
                    .lines
                    .into_iter()
                    .take(20)
                    .map(|line| focus_dim_line(line, app.is_focused)),
            );
        } else {
            lines.extend(preview.readme_delta.iter().take(20).map(|line| {
                let color = if line.trim_start().starts_with('+') {
                    MONOKAI_GREEN
                } else if line.trim_start().starts_with('-') {
                    MONOKAI_RED
                } else {
                    MONOKAI_TEXT
                };
                Line::from(Span::styled(
                    line.clone(),
                    focus_dim(Style::default().fg(color), app.is_focused),
                ))
            }));
        }
    }
    let validation = Paragraph::new(lines)
        .block(pane_block("validation", false, app.is_focused))
        .style(focus_dim(
            Style::default().bg(MONOKAI_PANEL).fg(MONOKAI_TEXT),
            app.is_focused,
        ))
        .wrap(Wrap { trim: false });
    frame.render_widget(validation, area);
}

pub(super) fn draw_log(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let max_scroll = app
        .log_state
        .lines
        .len()
        .saturating_sub(area.height.saturating_sub(2) as usize)
        .min(u16::MAX as usize) as u16;
    let log = Paragraph::new(app.log_state.lines.join("\n"))
        .block(pane_block(
            "log",
            app.active_pane == ActivePane::Log,
            app.is_focused,
        ))
        .style(focus_dim(
            Style::default().bg(MONOKAI_PANEL).fg(MONOKAI_TEXT),
            app.active_pane == ActivePane::Log && app.is_focused,
        ))
        .wrap(Wrap { trim: false })
        .scroll((app.log_state.scroll.min(max_scroll), 0));
    frame.render_widget(log, area);
}

pub(super) fn draw_footer(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let footer = Paragraph::new(
        "q: quit   /: filter repos   h/l/←/→: pane move   Enter: clone   n: rename   c: commit   Shift+P: push",
    )
    .block(pane_block("keys", false, app.is_focused))
    .style(focus_dim(
        Style::default().bg(MONOKAI_PANEL).fg(MONOKAI_MUTED),
        app.is_focused,
    ));
    frame.render_widget(footer, area);
}
