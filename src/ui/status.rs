use ratatui::{Frame, layout::Rect, style::{Color, Style}, widgets::Paragraph};
use crate::app::{App, View};


pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let view_hint = match app.view {
        View::Browser   => "[T]ypes  [S]PARQL  [F]ind  [M]arks  [b] Bookmark  [c] Copy triple  [Tab] Next section  [↑/↓] Navigate  [Enter] Open  [←/→] History  [Q]uit",
        View::Types     => "[S]PARQL  [F]ind  [M]arks  [↑/↓] Navigate  [Enter] Browse  [Q]uit",
        View::Sparql    => "[Esc/B]rowser  [Enter] Run  [Tab] Toggle input/results  [Q]uit",
        View::Search    => "[Esc/B]rowser  [T]ypes  [S]PARQL  [M]arks  [Enter] Search  [Tab] Toggle input/results  [↑/↓+Enter] Browse  [Q]uit",
        View::Bookmarks => "[B]rowser  [T]ypes  [S]PARQL  [F]ind  [↑/↓] Navigate  [Enter] Browse  [Del] Remove  [Q]uit",
    };
    let status = if app.status.is_empty() {
        view_hint.to_string()
    } else {
        format!("{}  |  {}", app.status, view_hint)
    };
    let p = Paragraph::new(status).style(Style::default().bg(Color::DarkGray).fg(Color::White));
    f.render_widget(p, area);
}
