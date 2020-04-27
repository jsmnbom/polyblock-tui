use async_trait::async_trait;
use std::fmt;
use tui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, Clear, List, ListState, Text},
};

use super::*;
use crate::{util, Instance, IoEvent};

pub enum MenuOption {
    Play,
    PlayShowLog,            // TODO
    ManageMods,             // TODO
    ChangeMinecraftVersion, // TODO
    ChangeForgeVersion,
    AddForge,
    RemoveForge,
    OpenDirectory,
    Rename,
    Remove,
}

impl fmt::Display for MenuOption {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MenuOption::Play => write!(f, "Play"),
            MenuOption::PlayShowLog => write!(f, "Play (show log)"),
            MenuOption::ManageMods => write!(f, "Manage mods"),
            MenuOption::ChangeMinecraftVersion => write!(f, "Change minecraft version"),
            MenuOption::ChangeForgeVersion => write!(f, "Change forge version"),
            MenuOption::AddForge => write!(f, "Add forge"),
            MenuOption::RemoveForge => write!(f, "Remove forge"),
            MenuOption::OpenDirectory => write!(f, "Open directory"),
            MenuOption::Rename => write!(f, "Rename"),
            MenuOption::Remove => write!(f, "Remove"),
        }
    }
}

impl MenuOption {
    pub fn vanilla() -> Vec<Self> {
        vec![
            Self::Play,
            Self::PlayShowLog,
            Self::ChangeMinecraftVersion,
            Self::AddForge,
            Self::OpenDirectory,
            Self::Rename,
            Self::Remove,
        ]
    }

    pub fn forge() -> Vec<Self> {
        vec![
            Self::Play,
            Self::PlayShowLog,
            Self::ManageMods,
            Self::ChangeMinecraftVersion,
            Self::ChangeForgeVersion,
            Self::RemoveForge,
            Self::OpenDirectory,
            Self::Rename,
            Self::Remove,
        ]
    }
}

#[derive(Default)]
pub struct State {
    pub selected: usize,
    pub options: Vec<MenuOption>,
    pub instance: Option<Instance>,
}

impl State {
    pub fn new(instance: Instance) -> Self {
        Self {
            selected: 0,
            options: if instance.forge_name.is_some() {
                MenuOption::forge()
            } else {
                MenuOption::vanilla()
            },
            instance: Some(instance),
        }
    }
}

pub struct Impl {}

#[async_trait]
impl RouteImpl for Impl {
    fn is_modal(&self) -> bool {
        true
    }
    fn get_help(&self, _app: &App) -> Vec<(&'static str, &'static str)> {
        vec![("ESC", "back"), ("↑/↓", "move cursor"), ("⏎", "select")]
    }
    fn handle_key(&self, key: Key, app: &mut App) {
        match key {
            Key::Up => {
                app.state.instance_menu.selected = util::wrap_dec(
                    app.state.instance_menu.selected,
                    app.state.instance_menu.options.len(),
                )
            }
            Key::Down => {
                app.state.instance_menu.selected = util::wrap_inc(
                    app.state.instance_menu.selected,
                    app.state.instance_menu.options.len(),
                )
            }
            Key::Enter => match app.state.instance_menu.options[app.state.instance_menu.selected] {
                MenuOption::Play => app.dispatch(IoEvent::PlayThenQuit),
                MenuOption::OpenDirectory => {
                    let instance = app.state.instance_menu.instance.as_ref().unwrap();
                    let directory = instance.directory();
                    open::that(directory).unwrap();
                    app.pop_route();
                }
                MenuOption::Rename => {
                    let instance = app.state.instance_menu.instance.clone().unwrap();
                    app.state.rename_instance = rename_instance::State::new(instance);
                    app.pop_route();
                    app.push_route(Route::RenameInstance);
                }
                MenuOption::Remove => {
                    let instance = app.state.instance_menu.instance.clone().unwrap();
                    app.state.remove_instance = remove_instance::State::new(instance);
                    app.pop_route();
                    app.push_route(Route::RemoveInstance);
                }
                MenuOption::AddForge => {
                    let instance = app.state.instance_menu.instance.clone().unwrap();
                    app.state.add_forge = add_forge::State::new(instance);
                    app.pop_route();
                    app.push_route(Route::AddForge);
                }
                MenuOption::ChangeForgeVersion => {
                    let instance = app.state.instance_menu.instance.clone().unwrap();
                    app.state.add_forge = add_forge::State::new(instance);
                    app.pop_route();
                    app.push_route(Route::AddForge);
                }
                MenuOption::RemoveForge => {
                    app.dispatch(IoEvent::RemoveForge);
                }
                _ => {}
            },
            _ => {}
        }
    }
    async fn draw(&self, f: &mut UiFrame<'_>, app: &mut App, chunk: Rect) {
        let state = &app.state.instance_menu;
        let instance = state.instance.as_ref().unwrap();
        let instance_name = instance.name.clone();

        let items: Vec<String> = state.options.iter().map(|o| o.to_string()).collect();

        let rect = util::centered_rect(
            (items.len() + 2) as u16,
            (items
                .iter()
                .map(|s| s.len() + 3)
                .max()
                .unwrap()
                .max(instance_name.len())
                + 2) as u16,
            chunk,
        );

        let mut list_state = ListState::default();
        list_state.select(Some(state.selected));

        f.render_widget(Clear, rect);
        f.render_stateful_widget(
            List::new(items.iter().map(|s| Text::raw(s)))
                .block(
                    Block::default()
                        .title(&instance_name)
                        .borders(Borders::ALL)
                        .border_type(BorderType::Plain),
                )
                .style(Style::default())
                .highlight_style(Style::default().fg(Color::Blue).modifier(Modifier::BOLD))
                .highlight_symbol(">> "),
            rect,
            &mut list_state,
        );
    }
}
