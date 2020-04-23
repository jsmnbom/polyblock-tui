use std::fmt;
use tui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, Clear, List, ListState, Text},
    Frame,
};

use crate::{ui::RenderState, util, App, Key};

pub enum Option {
    Play,
    ManageMods,
    ChangeMinecraftVersion,
    ChangeForgeVersion,
    AddForge,
    RemoveForge,
    OpenDirectory,
    Rename,
    Remove,
}

impl fmt::Display for Option {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Option::Play => write!(f, "Play"),
            Option::ManageMods => write!(f, "Manage mods"),
            Option::ChangeMinecraftVersion => write!(f, "Change minecraft version"),
            Option::ChangeForgeVersion => write!(f, "Change forge version"),
            Option::AddForge => write!(f, "Add forge"),
            Option::RemoveForge => write!(f, "Remove forge"),
            Option::OpenDirectory => write!(f, "Open directory"),
            Option::Rename => write!(f, "Rename"),
            Option::Remove => write!(f, "Remove"),
        }
    }
}

impl Option {
    pub fn vanilla() -> Vec<Self> {
        vec![
            Self::Play,
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
    pub options: Vec<Option>,
    pub instance_name: String,
}

pub fn get_help(_app: &App) -> Vec<(&'static str, &'static str)> {
    vec![("↑/↓", "move cursor"), ("⏎", "select"), ("ESC", "back")]
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
        Key::Enter => {}
        _ => {}
    }
}

pub fn draw<B: Backend>(f: &mut Frame<B>, app: &App, chunk: Rect) -> RenderState {
    let state = &app.instance_menu;

    let instance_name = &state.instance_name;

    let items: Vec<String> = state.options.iter().map(|o| o.to_string()).collect();

    let list = List::new(items.iter().map(|s| Text::raw(s)))
        .block(
            Block::default()
                .title(instance_name)
                .borders(Borders::ALL)
                .border_type(BorderType::Plain),
        )
        .style(Style::default())
        .highlight_style(Style::default().fg(Color::Blue).modifier(Modifier::BOLD))
        .highlight_symbol(">> ");

    let mut list_state = ListState::default();
    list_state.select(Some(state.selected));

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

    f.render_widget(Clear, rect);
    f.render_stateful_widget(list, rect, &mut list_state);

    RenderState::default()
}
