use log::trace;
use std::{io::Stdout, time::Instant};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Paragraph, Text},
    Frame,
};

use crate::{view, App, RouteId};

pub type UiFrame<'a> = Frame<'a, CrosstermBackend<Stdout>>;

pub struct RenderState {
    hide_cursor: bool,
}

impl Default for RenderState {
    fn default() -> Self {
        Self { hide_cursor: true }
    }
}

impl RenderState {
    pub fn show_cursor(mut self) -> Self {
        self.hide_cursor = false;
        self
    }
}

pub async fn draw_layout(f: &mut UiFrame<'_>, app: &mut App) {
    let instant = Instant::now();
    let routes = app.get_current_routes();

    let parent_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)].as_ref())
        .margin(0)
        .split(f.size());

    let raw_help = match routes.last().unwrap().id {
        RouteId::Home => view::home::get_help(app),
        RouteId::InstanceMenu => view::instance_menu::get_help(app),
        RouteId::NewInstance => view::new_instance::get_help(app),
        RouteId::RemoveInstance => view::remove_instance::get_help(app),
        RouteId::RenameInstance => view::rename_instance::get_help(app),
    };

    let help = raw_help
        .iter()
        .map(|(key, text)| {
            vec![
                Text::styled(key.clone(), Style::default().modifier(Modifier::BOLD)),
                Text::raw(" "),
                Text::raw(text.clone()),
            ]
        })
        .collect::<Vec<_>>()
        .join(&Text::raw("   "));

    f.render_widget(
        Paragraph::new(help.iter()).style(Style::default().fg(Color::White)),
        parent_layout[1],
    );

    let mut render_state = None;
    for route in routes.iter() {
        render_state = Some(match route.id {
            RouteId::Home => view::home::draw(f, app, parent_layout[0]),
            RouteId::InstanceMenu => view::instance_menu::draw(f, app, parent_layout[0]),
            RouteId::NewInstance => view::new_instance::draw(f, app, parent_layout[0]).await,
            RouteId::RemoveInstance => view::remove_instance::draw(f, app, parent_layout[0]),
            RouteId::RenameInstance => view::rename_instance::draw(f, app, parent_layout[0]),
        });
    }
    app.hide_cursor = render_state.unwrap().hide_cursor;

    trace!("Drawing took {:?}", instant.elapsed());
}
