use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{
        Block, BorderType, Borders, Clear, Gauge, List, Paragraph, Row, Table, TableState, Text,
    },
    Frame, Terminal,
};

use crate::Key;
use crate::{App, RouteId};

pub fn handle(key: Key, app: &mut App) {
    let routes = app.get_current_routes();
    let route = routes.last().unwrap();

    if key == Key::Esc {
        if app.pop_route_stack().is_none() {
            app.quit();
        }
    } else {
        match route.id {
            RouteId::Home => match key {
                Key::Up => {
                    app.home_selected_instance = if app.home_selected_instance == 0 {
                        4 - 1
                    } else {
                        app.home_selected_instance - 1
                    };
                }
                Key::Down => {
                    app.home_selected_instance = if app.home_selected_instance >= 4 - 1 {
                        0
                    } else {
                        app.home_selected_instance + 1
                    };
                }
                Key::Enter => {
                    app.push_route_stack(RouteId::InstanceMenu, true);
                }
                _ => {}
            },
            _ => unimplemented!(),
        }
    }
}
