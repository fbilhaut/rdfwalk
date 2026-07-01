use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Renders a text input box. When `cursor` is `Some(byte_offset)` the terminal
/// cursor is positioned at that offset inside the text (accounting for `\n`
/// and soft-wrapping at the inner widget width).
///
/// The text is hard-wrapped ourselves (`wrap_chars`) rather than relying on
/// `Paragraph`'s built-in `Wrap`, which reflows on word boundaries — that would
/// make the wrapped layout diverge from the simple per-character column count
/// used below to place the cursor, and the cursor would drift on any input
/// long enough to wrap.
pub fn text_input(f: &mut Frame, text: &str, cursor: Option<usize>, active: bool, title: &str, area: Rect) {
    let border_style = if active { Style::default().fg(Color::Yellow) } else { Style::default() };
    let block = Block::default().title(title).borders(Borders::ALL).border_style(border_style);
    let inner_w = (area.width.saturating_sub(2) as usize).max(1);
    let para = Paragraph::new(wrap_chars(text, inner_w))
        .block(block)
        .style(if active { Style::default().fg(Color::Yellow) } else { Style::default().fg(Color::Gray) });
    f.render_widget(para, area);

    if let Some(byte_offset) = cursor {
        let (row, col) = cursor_position(text, byte_offset, inner_w);
        let cx = (area.x + 1 + col as u16).min(area.x + area.width.saturating_sub(2));
        let cy = (area.y + 1 + row).min(area.y + area.height.saturating_sub(2));
        f.set_cursor_position((cx, cy));
    }
}

/// Hard-wraps `text` at `width` characters per line, preserving explicit `\n`.
/// Must stay in lockstep with `cursor_position`.
fn wrap_chars(text: &str, width: usize) -> String {
    let mut out = String::with_capacity(text.len());
    for (i, line) in text.split('\n').enumerate() {
        if i > 0 { out.push('\n'); }
        let mut col = 0;
        for ch in line.chars() {
            if col == width {
                out.push('\n');
                col = 0;
            }
            out.push(ch);
            col += 1;
        }
    }
    out
}

/// Computes the (row, column) of `byte_offset` within `text` under the same
/// hard-wrapping rule as `wrap_chars`.
fn cursor_position(text: &str, byte_offset: usize, width: usize) -> (u16, usize) {
    let before = &text[..byte_offset.min(text.len())];
    let mut row = 0u16;
    let mut col = 0usize;
    for ch in before.chars() {
        if ch == '\n' {
            row += 1;
            col = 0;
        } else {
            if col == width {
                row += 1;
                col = 0;
            }
            col += 1;
        }
    }
    // `wrap_chars` wraps *before* placing a character once `col` reaches `width`,
    // so a cursor sitting right at that boundary belongs at the start of the next row.
    if col == width {
        row += 1;
        col = 0;
    }
    (row, col)
}
