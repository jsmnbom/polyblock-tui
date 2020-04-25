use tui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, Row, Table, TableState},
};

use crate::{
    ui::{RenderState, UiFrame},
    util, App, Key, RouteId,
};

#[derive(Default)]
pub struct State {
    pub selected: usize,
}

pub fn get_help(_app: &App) -> Vec<(&'static str, &'static str)> {
    vec![
        ("ESC", "quit"),
        ("↑/↓", "choose instance"),
        ("⏎", "open menu"),
        ("ctrl+N", "new"),
        ("ctrl+R", "remove"),
        ("F2", "rename"),
    ]
}

pub fn handle_key(key: Key, app: &mut App) {
    match key {
        Key::Up => app.home.selected = util::wrap_dec(app.home.selected, app.instances.inner.len()),
        Key::Down => {
            app.home.selected = util::wrap_inc(app.home.selected, app.instances.inner.len())
        }
        Key::Enter => {
            let mut instances: Vec<_> = app.instances.inner.iter().collect();
            instances.sort_by(|x, y| x.0.cmp(&y.0));
            let instance = instances[app.home.selected];
            app.instance_menu = Default::default();
            app.instance_menu.instance = Some(instance.1.clone());

            app.instance_menu.options = if instance.1.forge_name.is_none() {
                super::instance_menu::MenuOption::vanilla()
            } else {
                super::instance_menu::MenuOption::forge()
            };
            app.push_route(RouteId::InstanceMenu, true);
        }
        Key::Ctrl('n') => {
            app.new_instance = Default::default();
            app.push_route(RouteId::NewInstance, true);
        }
        Key::Ctrl('r') => {
            let mut instances: Vec<_> = app.instances.inner.iter().collect();
            instances.sort_by(|x, y| x.0.cmp(&y.0));
            let instance = instances[app.home.selected];
            app.remove_instance = Default::default();
            app.remove_instance.instance = Some(instance.1.clone());
            app.push_route(RouteId::RemoveInstance, true);
        }
        Key::F2 => {
            let mut instances: Vec<_> = app.instances.inner.iter().collect();
            instances.sort_by(|x, y| x.0.cmp(&y.0));
            let instance = instances[app.home.selected];
            app.rename_instance = Default::default();
            app.rename_instance.instance = Some(instance.1.clone());
            app.rename_instance.name_input = instance.1.name.clone();
            app.push_route(RouteId::RenameInstance, true);
        }
        _ => {}
    }
}

pub fn draw(f: &mut UiFrame<'_>, app: &App, chunk: Rect) -> RenderState {
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

    let table = Table::new(
        ["   Name", "Minecraft version", "Modloader", "Mods"].iter(),
        rows.into_iter(),
    )
    .block(
        Block::default()
            .title("Polyblock - choose instance")
            .borders(Borders::ALL)
            .border_type(BorderType::Plain),
    )
    .header_style(Style::default().fg(Color::Yellow).modifier(Modifier::BOLD))
    .widths(&[
        Constraint::Percentage(40),
        Constraint::Percentage(20),
        Constraint::Percentage(20),
        Constraint::Percentage(20),
    ])
    .style(Style::default())
    .highlight_style(Style::default().fg(Color::Blue).modifier(Modifier::BOLD))
    .highlight_symbol(">> ")
    .column_spacing(1)
    .header_gap(0);

    let mut state = TableState::default();
    state.select(Some(app.home.selected));

    f.render_stateful_widget(table, chunk, &mut state);

    RenderState::default()
}
