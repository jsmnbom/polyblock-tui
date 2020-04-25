use std::path::PathBuf;
use std::sync::mpsc::Sender;
use std::sync::Mutex;

use crate::{forge, minecraft, view, Instances, IoEvent, Opt, Paths};

#[derive(Clone)]
pub enum RouteId {
    Home,
    InstanceMenu,
    NewInstance,
    RemoveInstance,
    RenameInstance,
}

#[derive(Clone)]
pub struct Route {
    pub id: RouteId,
    modal: bool,
}

pub struct App {
    // We need a mutex so we can send the app into the io thread - even if the io_tx is never actually used there
    io_tx: Mutex<Sender<IoEvent>>,
    route_stack: Vec<Route>,

    pub home: view::home::State,
    pub instance_menu: view::instance_menu::State,
    pub new_instance: view::new_instance::State,
    pub remove_instance: view::remove_instance::State,
    pub rename_instance: view::rename_instance::State,

    pub should_quit: bool,
    pub hide_cursor: bool,

    pub instances: Instances,
    pub paths: Paths,
    pub launcher: minecraft::Launcher,
    pub java_home_overwrite: Option<PathBuf>,

    pub minecraft_version_manifest: Option<minecraft::VersionManifest>,
    pub forge_version_manifest: Option<forge::VersionManifest>,
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
            io_tx: Mutex::new(io_tx),
            route_stack: vec![Route {
                id: RouteId::Home,
                modal: false,
            }],
            home: Default::default(),
            instance_menu: Default::default(),
            new_instance: Default::default(),
            remove_instance: Default::default(),
            rename_instance: Default::default(),
            should_quit: false,
            paths,
            instances,
            launcher,
            java_home_overwrite: opt.java_home.clone(),
            hide_cursor: true,
            minecraft_version_manifest: None,
            forge_version_manifest: None,
        })
    }

    /// Send a io event to the io thread
    pub fn dispatch(&mut self, action: IoEvent) {
        self.io_tx.lock().unwrap().send(action).unwrap();
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
