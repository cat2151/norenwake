use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub(super) fn centered_rect(percent_x: u16, percent_y: u16, rect: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(rect);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

pub(super) fn expand_rect_within(
    rect: Rect,
    bounds: Rect,
    left: u16,
    right: u16,
    top: u16,
    bottom: u16,
) -> Rect {
    let x = rect.x.saturating_sub(left).max(bounds.x);
    let y = rect.y.saturating_sub(top).max(bounds.y);
    let right_edge = rect
        .x
        .saturating_add(rect.width)
        .saturating_add(right)
        .min(bounds.x.saturating_add(bounds.width));
    let bottom_edge = rect
        .y
        .saturating_add(rect.height)
        .saturating_add(bottom)
        .min(bounds.y.saturating_add(bounds.height));
    Rect::new(
        x,
        y,
        right_edge.saturating_sub(x),
        bottom_edge.saturating_sub(y),
    )
}
