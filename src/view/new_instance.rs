use crossterm::{cursor::MoveTo, execute};
use std::io::{self, Write};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Row, Table, TableState, Text},
    Frame,
};
use unicode_width::UnicodeWidthStr;

use crate::{minecraft, ui::RenderState, util, App, IoEvent, Key};

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
    selected: usize,
    choose_minecraft_version_rows: Vec<Row<std::vec::IntoIter<std::string::String>>>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            inner: InnerState::EnterName,
            name_input: String::new(),
            error: None,
            selected: 0,
            choose_minecraft_version_rows: Vec::new(),
        }
    }
}

pub fn get_help(app: &App) -> Vec<(&'static str, &'static str)> {
    match app.new_instance.inner {
        InnerState::EnterName => vec![("←→", "move cursor"), ("⏎", "continue"), ("ESC", "cancel")],
        InnerState::FetchMinecraftVersionManifest => Vec::new(),
        InnerState::ChooseMinecraftVersion => vec![
            ("↑↓", "move cursor"),
            ("PgUp/PgDn", "move cursor 25"),
            ("⏎", "select"),
            ("ESC", "cancel"),
        ],
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
                    app.dispatch(IoEvent::FetchMinecraftVersionManifest);
                    app.new_instance.inner = InnerState::FetchMinecraftVersionManifest;
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
        InnerState::ChooseMinecraftVersion => {
            let versions_len = app
                .minecraft_version_manifest
                .as_ref()
                .unwrap()
                .versions
                .len();
            match key {
                Key::Up => {
                    app.new_instance.selected =
                        util::wrap_dec(app.new_instance.selected, versions_len)
                }
                Key::Down => {
                    app.new_instance.selected =
                        util::wrap_inc(app.new_instance.selected, versions_len)
                }
                Key::PageUp => {
                    app.new_instance.selected =
                        util::wrap_sub(app.new_instance.selected, versions_len, 25)
                }
                Key::PageDown => {
                    app.new_instance.selected =
                        util::wrap_add(app.new_instance.selected, versions_len, 25)
                }
                _ => {}
            }
        }
        _ => unimplemented!(),
    }
}

pub fn draw<B: Backend>(f: &mut Frame<B>, app: &App, chunk: Rect) -> RenderState {
    match &app.new_instance.inner {
        InnerState::EnterName => draw_enter_name(f, app, chunk),
        InnerState::FetchMinecraftVersionManifest => {
            draw_loading("Loading minecraft version manifest...", f, app, chunk)
        }
        InnerState::ChooseMinecraftVersion => draw_choose_minecraft_version(f, app, chunk),
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
                .border_type(BorderType::Double)
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
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Double),
        );

    f.render_widget(Clear, rect);
    f.render_widget(input, rect);

    RenderState::default()
}

pub fn draw_choose_minecraft_version<'a, B: Backend>(
    f: &mut Frame<B>,
    app: &App,
    chunk: Rect,
) -> RenderState {
    let rect = util::centered_rect_percentage(90, 75, chunk);

    let versions = &(app.minecraft_version_manifest.as_ref().unwrap().versions);
    // .iter()
    // .filter(|version| match version.r#type {
    //     minecraft::VersionManifestVersionType::Release => true,
    //     minecraft::VersionManifestVersionType::Snapshot /* if snapshots */ => true,
    //     minecraft::VersionManifestVersionType::OldBeta /* if historical */ => true,
    //     minecraft::VersionManifestVersionType::OldAlpha /* if historical */ => true,
    //     _ => false,
    // })
    // .collect::<Vec<_>>();

    let offset = app
        .new_instance
        .selected
        .saturating_sub((rect.height / 2) as usize)
        .min(versions.len().saturating_sub((rect.height / 2) as usize));

    let rows: Vec<_> = versions
        .iter()
        .skip(offset)
        .take(rect.height as usize)
        .map(|version| {
            Row::Data(
                vec![
                    version.id.clone(),
                    minecraft::version_ident(&version).to_string(),
                    version.release_time.format("%b %e %Y").to_string(),
                ]
                .into_iter(),
            )
        })
        .collect();

    let table = Table::new(
        ["   Version id", "Type", "Release date"].iter(),
        rows.into_iter(),
    )
    .block(
        Block::default()
            .title("Choose minecraft version")
            .borders(Borders::ALL)
            .border_type(BorderType::Double),
    )
    .header_style(Style::default().fg(Color::Yellow).modifier(Modifier::BOLD))
    .widths(&[
        Constraint::Percentage(40),
        Constraint::Percentage(30),
        Constraint::Percentage(30),
    ])
    .style(Style::default())
    .highlight_style(Style::default().fg(Color::Blue).modifier(Modifier::BOLD))
    .highlight_symbol(">> ")
    .column_spacing(1)
    .header_gap(0);

    let mut state = TableState::default();
    state.select(Some(app.new_instance.selected - offset));

    f.render_widget(Clear, rect);
    f.render_stateful_widget(table, rect, &mut state);

    RenderState::default()
}
