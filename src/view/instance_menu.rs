use std::fmt;
use tui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, Clear, List, ListState, Text},
};

use crate::{
    ui::{RenderState, UiFrame},
    util, App, Instance, IoEvent, Key, RouteId,
};

pub enum MenuOption {
    Play,
    PlayShowLog,            // TODO
    ManageMods,             // TODO
    ChangeMinecraftVersion, // TODO
    ChangeForgeVersion,     // TODO
    AddForge,               // TODO
    RemoveForge,            // TODO
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

pub fn get_help(_app: &App) -> Vec<(&'static str, &'static str)> {
    vec![("ESC", "back"), ("↑/↓", "move cursor"), ("⏎", "select")]
}

pub fn handle_key(key: Key, app: &mut App) {
    match key {
        Key::Up => {
            app.instance_menu.selected =
                util::wrap_dec(app.instance_menu.selected, app.instance_menu.options.len())
        }
        Key::Down => {
            app.instance_menu.selected =
                util::wrap_inc(app.instance_menu.selected, app.instance_menu.options.len())
        }
        Key::Enter => match app.instance_menu.options[app.instance_menu.selected] {
            MenuOption::Play => app.dispatch(IoEvent::PlayThenQuit),
            MenuOption::OpenDirectory => {
                let instance = app.instance_menu.instance.as_ref().unwrap();
                let directory = instance.directory();
                open::that(directory).unwrap();
                app.pop_route();
            }
            MenuOption::Rename => {
                let instance = app.instance_menu.instance.clone().unwrap();
                app.rename_instance = Default::default();
                app.rename_instance.instance = Some(instance.clone());
                app.rename_instance.name_input = instance.name.clone();
                app.pop_route();
                app.push_route(RouteId::RenameInstance, true);
            }
            MenuOption::Remove => {
                let instance = app.instance_menu.instance.clone().unwrap();
                app.remove_instance = Default::default();
                app.remove_instance.instance = Some(instance.clone());
                app.pop_route();
                app.push_route(RouteId::RemoveInstance, true);
            }
            _ => {}
        },
        _ => {}
    }
}

pub fn draw(f: &mut UiFrame<'_>, app: &App, chunk: Rect) -> RenderState {
    let state = &app.instance_menu;
    let instance = state.instance.as_ref().unwrap();
    let instance_name = instance.name.clone();

    let items: Vec<String> = app
        .instance_menu
        .options
        .iter()
        .map(|o| o.to_string())
        .collect();

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

    RenderState::default()
}
