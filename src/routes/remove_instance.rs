use async_trait::async_trait;
use tui::layout::Rect;

use super::*;
use crate::{util, Instance, IoEvent};

#[derive(Clone)]
pub struct State {
    pub instance: Option<Instance>,
    pub selected: usize,
}

impl Default for State {
    fn default() -> Self {
        Self {
            instance: None,
            selected: 1,
        }
    }
}

impl State {
    pub fn new(instance: Instance) -> Self {
        Self {
            instance: Some(instance),
            ..Default::default()
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
        vec![
            ("←/→", "choose option"),
            ("Y", "yes"),
            ("N", "no"),
            ("⏎", "select"),
            ("ESC", "cancel"),
        ]
    }
    fn handle_key(&self, key: Key, app: &mut App) {
        match key {
            Key::Left => {
                app.state.remove_instance.selected =
                    util::wrap_dec(app.state.remove_instance.selected, 2)
            }
            Key::Right => {
                app.state.remove_instance.selected =
                    util::wrap_inc(app.state.remove_instance.selected, 2)
            }
            Key::Char('y') => {
                app.dispatch(IoEvent::RemoveInstance);
                app.pop_route();
            }
            Key::Char('n') => {
                app.pop_route();
            }
            Key::Enter => {
                if app.state.remove_instance.selected == 0 {
                    app.dispatch(IoEvent::RemoveInstance);
                }
                app.pop_route();
            }
            _ => {}
        }
    }
    async fn draw(&self, f: &mut UiFrame<'_>, app: &mut App, chunk: Rect) {
        let state = &app.state.remove_instance;
        common::draw_button_dialog(
            f,
            chunk,
            10,
            &format!(
                "Are you sure to want to remove this instance? This will also remove the directory {}.",
                state
                    .instance
                    .as_ref()
                    .unwrap()
                    .directory()
                    .display()
            ),
            vec!["[ Yes ]", "[ No ]"],
            state.selected,
        )
    }
}
