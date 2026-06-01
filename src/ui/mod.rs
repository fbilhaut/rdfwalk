mod util;
mod widgets;
mod browser;
mod bookmarks;
mod search;
mod sparql;
mod types;
mod status;


use ratatui::{layout::{Constraint, Direction, Layout}, Frame};
use crate::app::{App, View};


pub fn render(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(1)])
        .split(f.area());

    match app.view {
        View::Browser   => browser::render(f, app, chunks[0]),
        View::Types     => types::render(f, app, chunks[0]),
        View::Sparql    => sparql::render(f, app, chunks[0]),
        View::Search    => search::render(f, app, chunks[0]),
        View::Bookmarks => bookmarks::render(f, app, chunks[0]),
    }

    status::render(f, app, chunks[1]);
}
