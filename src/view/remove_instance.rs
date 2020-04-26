use tui::layout::Rect;

use super::common;
use crate::{
    ui::{RenderState, UiFrame},
    util, App, Instance, IoEvent, Key,
};

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

pub fn get_help(_app: &App) -> Vec<(&'static str, &'static str)> {
    vec![
        ("←/→", "choose option"),
        ("Y", "yes"),
        ("N", "no"),
        ("⏎", "select"),
        ("ESC", "cancel"),
    ]
}

pub fn handle_key(key: Key, app: &mut App) {
    match key {
        Key::Left => app.remove_instance.selected = util::wrap_dec(app.remove_instance.selected, 2),
        Key::Right => {
            app.remove_instance.selected = util::wrap_inc(app.remove_instance.selected, 2)
        }
        Key::Char('y') => {
            app.dispatch(IoEvent::RemoveInstance);
            app.pop_route();
        }
        Key::Char('n') => {
            app.pop_route();
        }
        Key::Enter => {
            if app.remove_instance.selected == 0 {
                app.dispatch(IoEvent::RemoveInstance);
            }
            app.pop_route();
        }
        _ => {}
    }
}

pub fn draw(f: &mut UiFrame<'_>, app: &App, chunk: Rect) -> RenderState {
    common::draw_button_dialog(
        f,
        chunk,
        10,
        &format!(
            "Are you sure to want to remove this instance? This will also remove the directory {}.",
            app.remove_instance
                .instance
                .as_ref()
                .unwrap()
                .directory()
                .display()
        ),
        vec!["[ Yes ]", "[ No ]"],
        app.remove_instance.selected,
    )
}
