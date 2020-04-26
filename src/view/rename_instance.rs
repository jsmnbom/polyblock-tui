use tui::layout::Rect;

use super::common;
use crate::{
    ui::{RenderState, UiFrame},
    App, Instance, IoEvent, Key,
};

#[derive(Clone)]
pub struct State {
    pub instance: Option<Instance>,
    pub name_input: String,
    error: Option<String>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            instance: None,
            name_input: String::new(),
            error: None,
        }
    }
}

pub fn get_help(_app: &App) -> Vec<(&'static str, &'static str)> {
    vec![
        ("ESC", "cancel"),
        // TODO: ("←/→", "move cursor"),
        ("⏎", "continue"),
    ]
}

pub fn handle_key(key: Key, app: &mut App) {
    match key {
        Key::Char(c) => {
            app.rename_instance.name_input.push(c);
        }
        Key::Backspace => {
            app.rename_instance.name_input.pop();
        }
        Key::Enter => {
            if app.rename_instance.error.is_none() {
                app.dispatch(IoEvent::RenameInstance);
                app.pop_route();
            }
        }
        _ => {}
    }

    if app.rename_instance.name_input.is_empty() {
        app.rename_instance.error = Some("You must enter a name!".to_string())
    } else if app
        .instances
        .inner
        .keys()
        .collect::<Vec<&String>>()
        .contains(&&app.rename_instance.name_input)
    {
        app.rename_instance.error = Some("An instance with that name already exists!".to_string())
    } else {
        app.rename_instance.error = None;
    }
}

pub fn draw(f: &mut UiFrame<'_>, app: &App, chunk: Rect) -> RenderState {
    common::draw_input_dialog(
        f,
        chunk,
        "Enter new instance name",
        &app.rename_instance.name_input,
        app.rename_instance.error.as_deref(),
    )
}
