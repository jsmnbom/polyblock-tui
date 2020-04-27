use async_trait::async_trait;
use tui::{
    layout::{Constraint, Rect},
    widgets::Row,
};

use super::*;
use crate::util;

#[derive(Default)]
pub struct State {
    selected: usize,
}

pub struct Impl {}

#[async_trait]
impl RouteImpl for Impl {
    fn is_modal(&self) -> bool {
        false
    }
    fn get_help(&self, _app: &App) -> Vec<(&'static str, &'static str)> {
        vec![
            ("ESC", "quit"),
            ("↑/↓", "choose instance"),
            ("⏎", "open menu"),
            ("ctrl+N", "new"),
            ("ctrl+R", "remove"),
            ("F2", "rename"),
        ]
    }
    fn handle_key(&self, key: Key, app: &mut App) {
        let mut state = &mut app.state.home;
        match key {
            Key::Up => state.selected = util::wrap_dec(state.selected, app.instances.inner.len()),
            Key::Down => state.selected = util::wrap_inc(state.selected, app.instances.inner.len()),
            Key::Enter => {
                let mut instances: Vec<_> = app.instances.inner.iter().collect();
                instances.sort_by(|x, y| x.0.cmp(&y.0));
                let instance = instances[state.selected];
                app.state.instance_menu = instance_menu::State::new(instance.1.clone());
                app.push_route(Route::InstanceMenu);
            }
            Key::Ctrl('n') => {
                app.state.new_instance = new_instance::State::default();
                app.push_route(Route::NewInstance);
            }
            Key::Ctrl('r') => {
                let mut instances: Vec<_> = app.instances.inner.iter().collect();
                instances.sort_by(|x, y| x.0.cmp(&y.0));
                let instance = instances[state.selected];
                app.state.remove_instance = remove_instance::State::new(instance.1.clone());
                app.push_route(Route::RemoveInstance);
            }
            Key::F2 => {
                let mut instances: Vec<_> = app.instances.inner.iter().collect();
                instances.sort_by(|x, y| x.0.cmp(&y.0));
                let instance = instances[state.selected];
                app.state.rename_instance = rename_instance::State::new(instance.1.clone());
                app.push_route(Route::RenameInstance);
            }
            _ => {}
        }
    }
    async fn draw(&self, f: &mut UiFrame<'_>, app: &mut App, chunk: Rect) {
        let state = &app.state.home;

        let mut instances: Vec<_> = app.instances.inner.iter().collect();

        instances.sort_by(|x, y| x.0.cmp(&y.0));

        let rows: Vec<_> = instances
            .into_iter()
            .map(|(name, instance)| {
                Row::Data(
                    vec![
                        name.clone(),
                        instance.version_id.clone(),
                        match instance.forge_name.as_ref() {
                            Some(modloader) => modloader.clone(),
                            None => String::from("(Vanilla)"),
                        },
                        if instance.mods.is_empty() {
                            String::from("")
                        } else {
                            format!("{} mods", instance.mods.len())
                        },
                    ]
                    .into_iter(),
                )
            })
            .collect();

        common::draw_table(
            f,
            chunk,
            &["   Name", "Minecraft version", "Modloader", "Mods"],
            rows,
            &[
                Constraint::Percentage(40),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
            ],
            None,
            Some(state.selected),
        );
    }
}
