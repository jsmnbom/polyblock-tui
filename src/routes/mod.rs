pub mod add_forge;
pub mod change_version;
mod common;
pub mod home;
pub mod instance_menu;
pub mod new_instance;
pub mod remove_instance;
pub mod rename_instance;

use async_trait::async_trait;
use tui::layout::Rect;

use crate::{ui::UiFrame, App, Key};

#[async_trait]
pub trait RouteImpl {
    fn is_modal(&self) -> bool;
    fn get_help(&self, app: &App) -> Vec<(&'static str, &'static str)>;
    fn handle_key(&self, key: Key, app: &mut App);
    async fn draw(&self, f: &mut UiFrame<'_>, app: &mut App, chunk: Rect);
}

#[derive(Clone)]
pub enum Route {
    Home,
    RenameInstance,
    RemoveInstance,
    NewInstance,
    InstanceMenu,
    AddForge,
    ChangeVersion,
}

impl Route {
    pub fn get_impl(&self) -> Box<dyn RouteImpl> {
        use Route::*;

        match self {
            Home => Box::new(home::Impl {}),
            RenameInstance => Box::new(rename_instance::Impl {}),
            RemoveInstance => Box::new(remove_instance::Impl {}),
            NewInstance => Box::new(new_instance::Impl {}),
            InstanceMenu => Box::new(instance_menu::Impl {}),
            AddForge => Box::new(add_forge::Impl {}),
            ChangeVersion => Box::new(change_version::Impl {}),
        }
    }
}

#[derive(Default)]
pub struct State {
    pub home: home::State,
    pub rename_instance: rename_instance::State,
    pub remove_instance: remove_instance::State,
    pub new_instance: new_instance::State,
    pub instance_menu: instance_menu::State,
    pub add_forge: add_forge::State,
    pub change_version: change_version::State,
}
