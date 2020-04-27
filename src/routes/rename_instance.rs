use async_trait::async_trait;
use tui::layout::Rect;

use super::*;
use crate::{Instance, IoEvent};

#[derive(Default)]
pub struct State {
    pub instance: Option<Instance>,
    pub name_input: String,
    pub error: Option<String>,
}

pub struct Impl {}

impl State {
    pub fn new(instance: Instance) -> Self {
        Self {
            name_input: instance.name.clone(),
            instance: Some(instance),
            error: None,
        }
    }
}

#[async_trait]
impl RouteImpl for Impl {
    fn is_modal(&self) -> bool {
        true
    }
    fn get_help(&self, _app: &App) -> Vec<(&'static str, &'static str)> {
        vec![
            ("ESC", "cancel"),
            // TODO: ("←/→", "move cursor"),
            ("⏎", "continue"),
        ]
    }
    fn handle_key(&self, key: Key, app: &mut App) {
        match key {
            Key::Char(c) => {
                app.state.rename_instance.name_input.push(c);
            }
            Key::Backspace => {
                app.state.rename_instance.name_input.pop();
            }
            Key::Enter => {
                if app.state.rename_instance.error.is_none() {
                    app.dispatch(IoEvent::RenameInstance);
                }
            }
            _ => {}
        }

        let mut state = &mut app.state.rename_instance;
        if state.name_input.is_empty() {
            state.error = Some("You must enter a name!".to_string())
        } else if app
            .instances
            .inner
            .keys()
            .collect::<Vec<&String>>()
            .contains(&&state.name_input)
        {
            state.error = Some("An instance with that name already exists!".to_string())
        } else {
            state.error = None;
        }
    }
    async fn draw(&self, f: &mut UiFrame<'_>, app: &mut App, chunk: Rect) {
        app.show_cursor();
        let state = &app.state.rename_instance;
        common::draw_input_dialog(
            f,
            chunk,
            "Enter new instance name",
            &state.name_input,
            state.error.as_deref(),
        )
    }
}
