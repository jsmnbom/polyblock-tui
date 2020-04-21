use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{
        Block, BorderType, Borders, Clear, Gauge, List, ListState, Paragraph, Row, Table,
        TableState, Text,
    },
    Frame, Terminal,
};

use crate::help::get_help;
use crate::{App, RouteId};

pub fn draw_layout<B: Backend>(f: &mut Frame<B>, app: &App) {
    let parent_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)].as_ref())
        .margin(0)
        .split(f.size());

    let routes = app.get_current_routes();

    f.render_widget(
        Paragraph::new(get_help(&(&routes.last().unwrap().id)).iter())
            .style(Style::default().fg(Color::White)),
        parent_layout[1],
    );

    for route in routes {
        match route.id {
            RouteId::Home => draw_home(f, app, parent_layout[0]),
            RouteId::InstanceMenu => draw_instance_menu(f, app, parent_layout[0]),
            _ => unimplemented!(),
        }
    }
}

pub fn draw_home<B: Backend>(f: &mut Frame<B>, app: &App, chunk: Rect) {
    let table = Table::new(
        ["   Name", "Minecraft version", "Modloader", "Mods"].iter(),
        vec![
            Row::Data(["Test 1", "1.14.4", "(Vanilla)", ""].iter()),
            Row::Data(["Test 2", "1.15.2", "forge-28.0.22", "22 mods"].iter()),
            Row::Data(["Test 3", "1.12.2", "forge-23.2.32", "107+ mods"].iter()),
            Row::Data(["Test 4", "beta 1.8", "(Vanilla)"].iter()),
        ]
        .into_iter(),
    )
    .block(
        Block::default()
            .title("Choose instance")
            .borders(Borders::ALL)
            .border_type(BorderType::Thick),
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
    state.select(Some(app.home_selected_instance));

    f.render_stateful_widget(table, chunk, &mut state);
}

pub fn draw_instance_menu<B: Backend>(f: &mut Frame<B>, app: &App, chunk: Rect) {
    let instance_name = "Blah blah blah instance";

    let items = [
        "Play",
        "Manage mods",
        "Change minecraft version",
        "Change forge version",
        "Remove forge",
        "Rename",
        "Remove",
    ];

    let rect = centered_rect(
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

    let list = List::new(items.iter().map(|i| Text::raw(*i)))
        .block(Block::default().title(instance_name).borders(Borders::ALL))
        .style(Style::default())
        .highlight_style(Style::default().fg(Color::Blue).modifier(Modifier::BOLD))
        .highlight_symbol(">> ");

    let mut state = ListState::default();
    state.select(Some(app.instance_menu_selected));

    f.render_widget(Clear, rect);
    f.render_stateful_widget(list, rect, &mut state);
}

fn centered_rect(height: u16, width: u16, r: Rect) -> Rect {
    Rect {
        height,
        width,
        x: (r.width - width) / 2,
        y: (r.height - height) / 2,
    }
}
