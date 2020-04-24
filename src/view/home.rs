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
        ("↑/↓", "choose instance"),
        ("⏎", "select"),
        ("ctrl+N", "new instance"),
        ("ctrl+R", "remove instance"),
        ("ESC", "quit"),
    ]
}

pub fn handle_key(key: Key, app: &mut App) {
    match key {
        Key::Up => app.home.selected = util::wrap_dec(app.home.selected, app.instances.inner.len()),
        Key::Down => {
            app.home.selected = util::wrap_inc(app.home.selected, app.instances.inner.len())
        }
        Key::Enter => {
            app.instance_menu.instance_name = "Instance name!".to_string();
            app.instance_menu.options = super::instance_menu::Option::forge();
            app.push_route(RouteId::InstanceMenu, true);
        }
        Key::Ctrl('n') => {
            app.new_instance = Default::default();
            app.push_route(RouteId::NewInstance, true);
        }
        _ => {}
    }
}

pub fn draw(f: &mut UiFrame<'_>, app: &App, chunk: Rect) -> RenderState {
    let rows: Vec<_> = app
        .instances
        .inner
        .iter()
        .map(|(name, instance)| {
            Row::Data(
                vec![
                    name.clone(),
                    instance.version_id.clone(),
                    match instance.forge_name.as_ref() {
                        Some(modloader) => modloader.clone(),
                        None => "(Vanilla)".to_string(),
                    },
                    if instance.mods.is_empty() {
                        String::new()
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
