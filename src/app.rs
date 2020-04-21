use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{
        Block, BorderType, Borders, Clear, Gauge, List, Paragraph, Row, Table, TableState, Text,
    },
    Terminal,
};

pub enum RouteId {
    Home,
    InstanceMenu,
}

pub struct Route {
    pub id: RouteId,
    modal: bool,
}

pub struct App {
    route_stack: Vec<Route>,
    pub home_selected_instance: usize,
    pub instance_menu_selected: usize,
    pub should_quit: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            route_stack: Vec::new(),
            home_selected_instance: 0,
            instance_menu_selected: 0,
            should_quit: false,
        }
    }

    pub fn push_route_stack(&mut self, id: RouteId, modal: bool) {
        self.route_stack.push(Route { id, modal });
    }

    pub fn pop_route_stack(&mut self) -> Option<Route> {
        if self.route_stack.len() == 1 {
            None
        } else {
            self.route_stack.pop()
        }
    }

    pub fn get_current_routes(&self) -> Vec<&Route> {
        let mut routes = Vec::new();
        for route in self.route_stack.iter().rev() {
            routes.push(route);
            if !route.modal {
                break;
            }
        }
        if routes.is_empty() {
            routes.push(&Route {
                id: RouteId::Home,
                modal: false,
            });
        }
        routes.reverse();
        routes
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }
}
