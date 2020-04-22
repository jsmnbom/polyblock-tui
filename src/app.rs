use std::sync::mpsc::Sender;

use crate::{IoEvent, minecraft, view, Instances, Opt, Paths};

#[derive(Clone)]
pub enum RouteId {
    Home,
    InstanceMenu,
    NewInstance,
}

#[derive(Clone)]
pub struct Route {
    pub id: RouteId,
    modal: bool,
}

pub struct App {
    io_tx: Sender<IoEvent>,
    route_stack: Vec<Route>,
    pub home: view::home::State,
    pub instance_menu: view::instance_menu::State,
    pub new_instance: view::new_instance::State,
    pub should_quit: bool,
    pub instances: Instances,
    pub paths: Paths,
    pub launcher: minecraft::Launcher,
    pub hide_cursor: bool,
}

impl App {
    pub fn new(opt: &Opt, io_tx: Sender<IoEvent>) -> ::anyhow::Result<Self> {
        let paths = Paths::new(opt)?;
        let instances = Instances::from_file(&paths.file.instances, &paths.directory.instances)?;

        let launcher = minecraft::Launcher::new(
            &paths.directory.launcher_work,
            &paths.directory.launcher_cache,
            opt.launcher.as_ref(),
        )?;

        Ok(Self {
            io_tx,
            route_stack: vec![Route {
                id: RouteId::Home,
                modal: false,
            }],
            home: Default::default(),
            instance_menu: Default::default(),
            new_instance: Default::default(),
            should_quit: false,
            paths,
            instances,
            launcher,
            hide_cursor: true,
        })
    }

    /// Send a io event to the io thread
    pub fn dispatch(&mut self, action: IoEvent) {
        self.io_tx.send(action).unwrap();
    }

    pub fn push_route(&mut self, id: RouteId, modal: bool) {
        self.route_stack.push(Route { id, modal });
    }

    pub fn pop_route(&mut self) -> Option<Route> {
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
