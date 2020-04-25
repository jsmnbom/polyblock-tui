use tui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Text},
};

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
    let rect = util::centered_rect_percentage_dir(Direction::Horizontal, 60, chunk);
    let rect = util::centered_rect_dir(Direction::Vertical, 10, rect);

    let block_widget = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Plain);
    f.render_widget(Clear, rect);
    f.render_widget(block_widget, rect);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(4),
                Constraint::Length(1),
                Constraint::Length(1),
            ]
            .as_ref(),
        )
        .split(rect);

    let raw_text = format!(
        "Are you sure to want to remove this instance? This will also remove the directory {}.",
        app.remove_instance
            .instance
            .as_ref()
            .unwrap()
            .directory()
            .display()
    );
    let text = vec![Text::raw(&raw_text)];
    let text_widget = Paragraph::new(text.iter())
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Yellow))
        .wrap(true);

    f.render_widget(text_widget, layout[0]);

    let button_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Length((layout[2].width - 16) / 2),
                Constraint::Length(7),
                Constraint::Length(3),
                Constraint::Length(6),
                Constraint::Length((layout[2].width / 16) / 2),
            ]
            .as_ref(),
        )
        .split(layout[2]);

    f.render_widget(
        Paragraph::new(vec![Text::raw("[ Yes ]")].iter()).style(
            if app.remove_instance.selected == 0 {
                Style::default().fg(Color::Blue).modifier(Modifier::BOLD)
            } else {
                Style::default().modifier(Modifier::DIM)
            },
        ),
        button_layout[1],
    );
    f.render_widget(
        Paragraph::new(vec![Text::raw("[ No ]")].iter()).style(
            if app.remove_instance.selected != 0 {
                Style::default().fg(Color::Blue).modifier(Modifier::BOLD)
            } else {
                Style::default().modifier(Modifier::DIM)
            },
        ),
        button_layout[3],
    );

    RenderState::default()
}
