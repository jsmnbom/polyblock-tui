use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{
        Block, BorderType, Borders, Clear, Gauge, List, Paragraph, Row, Table, TableState, Text,
    },
    Frame, Terminal,
};

use crate::{App, RouteId};

pub fn get_help(routeId: &RouteId) -> Vec<Text> {
    let raw = match routeId {
        RouteId::Home => vec![
            ("←↑→↓", "move cursor"),
            ("⏎", "select"),
            ("ctrl+N", "new instance"),
            ("ctrl+R", "remove instance"),
            ("ESC", "quit"),
        ],
        RouteId::InstanceMenu => vec![("←↑→↓", "move cursor"), ("⏎", "select"), ("ESC", "back")],
    };

    raw.into_iter()
        .map(|(key, text)| {
            vec![
                Text::styled(key, Style::default().modifier(Modifier::BOLD)),
                Text::raw(" "),
                Text::raw(text),
            ]
        })
        .collect::<Vec<_>>()
        .join(&Text::raw("   "))
}
