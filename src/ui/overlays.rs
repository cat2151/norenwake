use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Clear, Paragraph, Wrap},
    Frame,
};
use std::time::Instant;

use crate::app::App;

use super::{
    geometry::{centered_rect, expand_rect_within},
    theme::{
        pane_block, spinner_block, MONOKAI_ACTIVE, MONOKAI_BG, MONOKAI_GREEN, MONOKAI_MUTED,
        MONOKAI_PANEL_ALT, MONOKAI_RED, MONOKAI_TEXT, MONOKAI_WHITE,
    },
};

pub(super) fn draw_help(frame: &mut Frame, area: ratatui::layout::Rect) {
    let popup = centered_rect(72, 62, area);
    frame.render_widget(Clear, popup);
    let help = Paragraph::new(vec![
        Line::from("help"),
        Line::from(""),
        Line::from("h / l / ← / → : move between repos, dir tree, and log"),
        Line::from("j/k or ↑/↓    : move inside the active pane"),
        Line::from("PgUp/PgDn      : page within the active pane"),
        Line::from("/              : open repo filter"),
        Line::from("Enter          : clone selected repo (repos pane only)"),
        Line::from("n              : edit new repo name"),
        Line::from("c              : commit local change"),
        Line::from("Shift+P        : validate then confirm push"),
        Line::from("Shift+L        : copy full log"),
        Line::from("? / Esc        : close help"),
    ])
    .block(pane_block("help overlay", true, true))
    .style(Style::default().bg(MONOKAI_PANEL_ALT).fg(MONOKAI_TEXT));
    frame.render_widget(help, popup);
}

pub(super) fn draw_filter_overlay(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let popup = centered_rect(70, 74, area);
    let guard = expand_rect_within(popup, area, 1, 1, 0, 0);
    frame.render_widget(Clear, guard);
    frame.render_widget(
        Block::default().style(Style::default().bg(MONOKAI_PANEL_ALT)),
        guard,
    );
    frame.render_widget(Clear, popup);
    let filtered = app.filtered_repo_indices();
    let visible_rows = popup.height.saturating_sub(5).max(1) as usize;
    let lines = if filtered.is_empty() {
        vec![Line::from("一致する repo はありません")]
    } else {
        let total = filtered.len();
        let selected = app.filter_state.selected.min(total.saturating_sub(1));
        let half = visible_rows / 2;
        let mut start = selected.saturating_sub(half);
        if start + visible_rows > total {
            start = total.saturating_sub(visible_rows);
        }
        filtered
            .iter()
            .enumerate()
            .skip(start)
            .take(visible_rows)
            .filter_map(|(visible_index, repo_index)| {
                app.repos.get(*repo_index).map(|repo| (visible_index, repo))
            })
            .map(|(visible_index, repo)| {
                let style = if visible_index == app.filter_state.selected {
                    Style::default()
                        .fg(MONOKAI_ACTIVE)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(MONOKAI_WHITE)
                };
                Line::from(Span::styled(repo.name.clone(), style))
            })
            .collect()
    };
    let mut text = vec![
        Line::from(Span::styled(
            &app.repo_filter,
            Style::default()
                .fg(MONOKAI_TEXT)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "space 区切りで AND 検索します",
            Style::default().fg(MONOKAI_MUTED),
        )),
        Line::from(""),
    ];
    text.extend(lines);
    let dialog = Paragraph::new(text)
        .block(pane_block("filter repos", true, true))
        .style(Style::default().bg(MONOKAI_PANEL_ALT).fg(MONOKAI_TEXT))
        .wrap(Wrap { trim: false });
    frame.render_widget(dialog, popup);
}

pub(super) fn draw_name_editor(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let popup = centered_rect(56, 24, area);
    frame.render_widget(Clear, popup);
    let editor = Paragraph::new(vec![
        Line::from(vec![
            Span::styled(
                &app.edit_buffer,
                Style::default()
                    .fg(MONOKAI_TEXT)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "▌",
                Style::default()
                    .fg(MONOKAI_ACTIVE)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Enter で確定し、Esc で中止します",
            Style::default().fg(MONOKAI_MUTED),
        )),
        Line::from(Span::styled(
            "カーソルは末尾固定です。Backspace で削除します",
            Style::default().fg(MONOKAI_MUTED),
        )),
    ])
    .block(pane_block("new repo name", true, true))
    .style(Style::default().bg(MONOKAI_PANEL_ALT).fg(MONOKAI_TEXT));
    frame.render_widget(editor, popup);
}

pub(super) fn draw_confirm(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let popup = centered_rect(78, 70, area);
    let guard = expand_rect_within(popup, area, 1, 0, 0, 0);
    frame.render_widget(Clear, guard);
    frame.render_widget(
        Block::default().style(Style::default().bg(MONOKAI_PANEL_ALT)),
        guard,
    );
    frame.render_widget(Clear, popup);
    let lines = app
        .dry_run_lines
        .iter()
        .map(|line| {
            let color = if line.contains("OK:") || line.contains("検証に合格") {
                MONOKAI_GREEN
            } else if line.contains("NG:") {
                MONOKAI_RED
            } else {
                MONOKAI_TEXT
            };
            Line::from(Span::styled(line.clone(), Style::default().fg(color)))
        })
        .collect::<Vec<_>>();
    let confirm = Paragraph::new(lines)
        .block(pane_block("push confirm y/[N]", true, true))
        .style(Style::default().bg(MONOKAI_PANEL_ALT));
    frame.render_widget(confirm, popup);
}

pub(super) fn draw_busy_overlay(
    frame: &mut Frame,
    area: ratatui::layout::Rect,
    started_at: Option<Instant>,
    message: Option<&str>,
) {
    let popup = centered_rect(48, 16, area);
    frame.render_widget(Clear, popup);
    let body = message.unwrap_or("Processing...");
    let overlay = Paragraph::new(Text::from(body))
        .block(spinner_block(spinner_frame(started_at)))
        .style(Style::default().bg(MONOKAI_PANEL_ALT).fg(MONOKAI_TEXT))
        .wrap(Wrap { trim: false });
    frame.render_widget(overlay, popup);
}

pub(super) fn draw_dim_backdrop(frame: &mut Frame, area: ratatui::layout::Rect) {
    frame.render_widget(
        Block::default().style(Style::default().bg(MONOKAI_BG).add_modifier(Modifier::DIM)),
        area,
    );
}

pub(super) fn draw_focus_dim(frame: &mut Frame, area: ratatui::layout::Rect) {
    frame.render_widget(
        Block::default().style(Style::default().bg(MONOKAI_BG).add_modifier(Modifier::DIM)),
        area,
    );
}

fn spinner_frame(started_at: Option<Instant>) -> &'static str {
    let frames = ["-", "\\", "|", "/"];
    let index = started_at
        .map(|started| ((started.elapsed().as_millis() / 250) as usize) % frames.len())
        .unwrap_or(0);
    frames[index]
}
