use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders},
};

pub(super) const MONOKAI_BG: Color = Color::Rgb(39, 40, 34);
pub(super) const MONOKAI_PANEL: Color = Color::Rgb(49, 51, 44);
pub(super) const MONOKAI_PANEL_ALT: Color = Color::Rgb(42, 43, 37);
pub(super) const MONOKAI_BORDER: Color = Color::Rgb(117, 113, 94);
pub(super) const MONOKAI_ACTIVE: Color = Color::Rgb(249, 38, 114);
pub(super) const MONOKAI_TEXT: Color = Color::Rgb(248, 248, 242);
pub(super) const MONOKAI_MUTED: Color = Color::Rgb(146, 144, 132);
pub(super) const MONOKAI_WHITE: Color = Color::Rgb(245, 245, 240);
pub(super) const MONOKAI_GREEN: Color = Color::Rgb(166, 226, 46);
pub(super) const MONOKAI_ORANGE: Color = Color::Rgb(253, 151, 31);
pub(super) const MONOKAI_RED: Color = Color::Rgb(249, 38, 114);
pub(super) const MONOKAI_ROW: Color = Color::Rgb(62, 61, 50);
pub(super) const MONOKAI_DIM: Color = Color::Rgb(108, 106, 95);
pub(super) const MONO_UNFOCUSED_BG: Color = Color::Rgb(45, 45, 45);
pub(super) const MONO_UNFOCUSED_FG: Color = Color::Rgb(175, 175, 175);

pub(super) fn pane_block<'a>(title: &'a str, active: bool, focused: bool) -> Block<'a> {
    let border_color = if active {
        MONOKAI_ACTIVE
    } else {
        MONOKAI_BORDER
    };
    let title_color = if active {
        MONOKAI_ORANGE
    } else {
        MONOKAI_MUTED
    };
    Block::default()
        .borders(Borders::ALL)
        .border_style(focus_dim(
            Style::default().fg(border_color).bg(MONOKAI_PANEL),
            focused,
        ))
        .title(Span::styled(
            title,
            focus_dim(
                Style::default()
                    .fg(title_color)
                    .add_modifier(Modifier::BOLD),
                focused,
            ),
        ))
        .style(focus_dim(Style::default().bg(MONOKAI_PANEL), focused))
}

pub(super) fn spinner_block<'a>(spinner: &'a str) -> Block<'a> {
    Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(MONOKAI_ACTIVE).bg(MONOKAI_PANEL_ALT))
        .title(Line::from(vec![
            Span::styled(
                spinner,
                Style::default()
                    .fg(MONOKAI_ORANGE)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "  Processing...",
                Style::default()
                    .fg(MONOKAI_TEXT)
                    .add_modifier(Modifier::BOLD),
            ),
        ]))
        .style(Style::default().bg(MONOKAI_PANEL_ALT))
}

pub(super) fn focus_dim(style: Style, focused: bool) -> Style {
    if focused {
        style
    } else {
        let mut out = style.add_modifier(Modifier::DIM);
        if style.fg.is_some() {
            out = out.fg(MONO_UNFOCUSED_FG);
        }
        if style.bg.is_some() {
            out = out.bg(MONO_UNFOCUSED_BG);
        }
        out
    }
}

pub(super) fn focus_dim_line(mut line: Line<'static>, focused: bool) -> Line<'static> {
    if focused {
        return line;
    }
    line.style = focus_dim(line.style, focused);
    line.spans = line
        .spans
        .into_iter()
        .map(|mut span| {
            span.style = focus_dim(span.style, focused);
            span
        })
        .collect();
    line
}
