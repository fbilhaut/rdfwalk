use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};


pub fn text_input(f: &mut Frame, text: &str, active: bool, title: &str, area: Rect) {
    let border_style = if active { Style::default().fg(Color::Yellow) } else { Style::default() };
    let block = Block::default().title(title).borders(Borders::ALL).border_style(border_style);
    let para = Paragraph::new(text)
        .block(block)
        .style(if active { Style::default().fg(Color::Yellow) } else { Style::default().fg(Color::Gray) })
        .wrap(Wrap { trim: false });
    f.render_widget(para, area);
}