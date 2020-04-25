use crossterm::{cursor::MoveTo, execute};
use std::io::{self, Write};
use tui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Text},
};
use unicode_width::UnicodeWidthStr;

use crate::{
    ui::{RenderState, UiFrame},
    util, App, Instance, IoEvent, Key,
};

#[derive(Clone)]
pub struct State {
    pub instance: Option<Instance>,
    pub name_input: String,
    error: Option<String>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            instance: None,
            name_input: String::new(),
            error: None,
        }
    }
}

pub fn get_help(_app: &App) -> Vec<(&'static str, &'static str)> {
    vec![
        ("ESC", "cancel"),
        // TODO: ("←/→", "move cursor"),
        ("⏎", "continue"),
    ]
}

pub fn handle_key(key: Key, app: &mut App) {
    match key {
        Key::Char(c) => {
            app.rename_instance.name_input.push(c);
        }
        Key::Backspace => {
            app.rename_instance.name_input.pop();
        }
        Key::Enter => {
            app.dispatch(IoEvent::RenameInstance);
            app.pop_route();
        }
        _ => {}
    }

    if app.rename_instance.name_input.is_empty() {
        app.rename_instance.error = Some("You must enter a name!".to_string())
    } else if app
        .instances
        .inner
        .keys()
        .collect::<Vec<&String>>()
        .contains(&&app.rename_instance.name_input)
    {
        app.rename_instance.error = Some("An instance with that name already exists!".to_string())
    } else {
        app.rename_instance.error = None;
    }
}

pub fn draw(f: &mut UiFrame<'_>, app: &App, chunk: Rect) -> RenderState {
    let state = &app.rename_instance;

    let mut rect = util::centered_rect_percentage_dir(Direction::Horizontal, 30, chunk);
    rect.y = (rect.height / 2) - 2;
    rect.height = if state.error.is_some() { 4 } else { 3 };

    let mut text = vec![Text::raw(&state.name_input)];
    if let Some(error) = &state.error {
        text.push(Text::raw("\n\r"));
        text.push(Text::styled(error, Style::default().fg(Color::Red)));
    }
    let input = Paragraph::new(text.iter())
        .style(Style::default().fg(Color::Yellow))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Plain)
                .title("Enter new instance name"),
        );

    execute!(
        io::stdout(),
        MoveTo(
            rect.x + 1 + ((&state.name_input).width() as u16),
            rect.y + 1
        )
    )
    .ok();

    f.render_widget(Clear, rect);
    f.render_widget(input, rect);

    RenderState::default().show_cursor()
}
