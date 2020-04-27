use log::trace;
use std::{io::Stdout, time::Instant};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Paragraph, Text},
    Frame,
};

use crate::App;

pub type UiFrame<'a> = Frame<'a, CrosstermBackend<Stdout>>;

pub async fn draw_layout(f: &mut UiFrame<'_>, app: &mut App) -> ::anyhow::Result<()> {
    let instant = Instant::now();
    let routes: Vec<_> = app.get_current_routes().into_iter().cloned().collect();

    let parent_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)].as_ref())
        .margin(0)
        .split(f.size());

    for (i, route) in routes.iter().enumerate() {
        let implementation = route.get_impl();
        Some(implementation.draw(f, app, parent_layout[0]).await);

        if i == routes.len() - 1 {
            let raw_help = implementation.get_help(app);

            let help = raw_help
                .iter()
                .map(|(key, text)| {
                    vec![
                        Text::styled(key.clone(), Style::default().modifier(Modifier::BOLD)),
                        Text::raw(" "),
                        Text::raw(text.clone()),
                    ]
                })
                .collect::<Vec<_>>()
                .join(&Text::raw("   "));

            f.render_widget(
                Paragraph::new(help.iter()).style(Style::default().fg(Color::White)),
                parent_layout[1],
            );
        }
    }

    trace!("Drawing took {:?}", instant.elapsed());

    Ok(())
}
