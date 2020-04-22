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

use crate::{ui::RenderState, util, App, IoEvent, Key};

#[derive(Clone)]
pub enum InnerState {
    EnterName,
    FetchMinecraftVersionManifest,
    ChooseMinecraftVersion,
    ChooseForge,
    FetchForgeVersionManifest,
    ChooseForgeVersion,
}

#[derive(Clone)]
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

pub fn get_help(app: &App) -> Vec<(&'static str, &'static str)> {
    match app.new_instance.inner {
        InnerState::EnterName => vec![("←→", "move cursor"), ("⏎", "continue"), ("ESC", "cancel")],
        InnerState::FetchMinecraftVersionManifest => Vec::new(),
        _ => unimplemented!(),
    }
}

pub fn handle_key(key: Key, app: &mut App) {
    match app.new_instance.inner {
        InnerState::EnterName => {
            match key {
                Key::Char(c) => {
                    app.new_instance.name_input.push(c);
                }
                Key::Backspace => {
                    app.new_instance.name_input.pop();
                }
                Key::Enter => {
                    // delay_for(Duration::from_secs(5)).await;
                    // if !app.new_instance.name_input.is_empty() {
                    //     app.new_instance.inner = InnerState::FetchMinecraftVersions;
                    // }
                    app.new_instance.inner = InnerState::FetchMinecraftVersionManifest;
                    app.dispatch(IoEvent::FetchMinecraftVersionManifest);
                }
                _ => {}
            }

            if app.new_instance.name_input.is_empty() {
                app.new_instance.error = Some("You must enter a name!".to_string())
            } else if app
                .instances
                .inner
                .keys()
                .collect::<Vec<&String>>()
                .contains(&&app.new_instance.name_input)
            {
                app.new_instance.error =
                    Some("An instance with that name already exists!".to_string())
            } else {
                app.new_instance.error = None;
            }
        }
        InnerState::FetchMinecraftVersionManifest => {}
        _ => unimplemented!(),
    }
}

pub fn draw<B: Backend>(f: &mut Frame<B>, app: &App, chunk: Rect) -> RenderState {
    match &app.new_instance.inner {
        InnerState::EnterName => draw_enter_name(f, app, chunk),
        InnerState::FetchMinecraftVersionManifest => {
            draw_loading("Loading minecraft version manifest...", f, app, chunk)
        }
        _ => unimplemented!(),
    }
}

fn draw_enter_name<B: Backend>(f: &mut Frame<B>, app: &App, chunk: Rect) -> RenderState {
    let state = &app.new_instance;

    let mut rect = util::centered_rect_percentage_dir(Direction::Horizontal, 30, chunk);
    rect.y = (rect.height / 2) - 3;
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

fn draw_loading<B: Backend>(msg: &str, f: &mut Frame<B>, _app: &App, chunk: Rect) -> RenderState {
    let mut rect = util::centered_rect_percentage_dir(Direction::Horizontal, 30, chunk);
    rect.y = (rect.height / 2) - 3;
    rect.height = 3;

    let text = vec![Text::raw(msg)];
    let input = Paragraph::new(text.iter())
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL));

    f.render_widget(Clear, rect);
    f.render_widget(input, rect);

    RenderState::default()
}
