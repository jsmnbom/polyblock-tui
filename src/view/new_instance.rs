use crossterm::{cursor::MoveTo, execute};
use std::io::{self, Write};
use tui::{
    backend::Backend,
    layout::{Direction, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Paragraph, Text},
    Frame,
};
use unicode_width::UnicodeWidthStr;

use crate::{ui::RenderState, util, App, Key};

pub enum InnerState {
    EnterName,
    FetchMinecraftVersions,
    ChooseMinecraftVersion,
    ChooseForge,
    FetchForgeVersions,
    ChooseForgeVersion,
}

pub struct State {
    pub inner: InnerState,
    name_input: String,
    error: Option<String>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            inner: InnerState::EnterName,
            name_input: String::new(),
            error: None,
        }
    }
}

pub fn get_help(_app: &App) -> Vec<(&'static str, &'static str)> {
    vec![("←→", "move cursor"), ("⏎", "continue"), ("ESC", "cancel")]
}

pub fn handle_key(key: Key, app: &mut App) {
    let state = &mut app.new_instance;

    match &state.inner {
        InnerState::EnterName => {
            let existing_names: Vec<&String> = app.instances.inner.keys().collect();

            match key {
                Key::Char(c) => {
                    state.name_input.push(c);
                }
                Key::Backspace => {
                    state.name_input.pop();
                }
                Key::Enter => {
                    // delay_for(Duration::from_secs(5)).await;
                    // if !state.name_input.is_empty() {
                    //     state.inner = InnerState::FetchMinecraftVersions;
                    // }
                }
                _ => {}
            }

            if state.name_input.is_empty() {
                state.error = Some("You must enter a name!".to_string())
            } else if existing_names.contains(&&state.name_input) {
                state.error = Some("An instance with that name already exists!".to_string())
            } else {
                state.error = None;
            }
        }
        _ => unimplemented!(),
    }
}

pub fn draw<B: Backend>(f: &mut Frame<B>, app: &App, chunk: Rect) -> RenderState {
    match &app.new_instance.inner {
        InnerState::EnterName => draw_enter_name(f, app, chunk),
        _ => unimplemented!(),
    }
}

fn draw_enter_name<B: Backend>(f: &mut Frame<B>, app: &App, chunk: Rect) -> RenderState {
    let state = &app.new_instance;

    let mut rect = util::centered_rect_percentage_dir(Direction::Horizontal, 30, chunk);
    rect.y = (rect.height / 2) - 3;
    rect.height = if state.error.is_some() { 4 } else { 3 };

    // app.hide_cursor = false;

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
                .title_style(Style::default())
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
