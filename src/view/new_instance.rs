use crossterm::{cursor::MoveTo, execute};
use log::debug;
use std::io::{self, Write};
use tui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, Clear, Gauge, Paragraph, Row, Table, TableState, Text},
};
use unicode_width::UnicodeWidthStr;

use crate::{
    forge, minecraft,
    ui::{RenderState, UiFrame},
    util, App, IoEvent, Key,
};

#[derive(Clone)]
pub enum InnerState {
    EnterName,
    FetchMinecraftVersionManifest,
    ChooseMinecraftVersion,
    ChooseForge,
    FetchForgeVersionManifest,
    ChooseForgeVersion,
    Install,
}

#[derive(Clone)]
pub struct State {
    pub inner: InnerState,
    pub name_input: String,
    error: Option<String>,
    selected: usize,
    pub chosen_minecraft_version: Option<minecraft::VersionManifestVersion>,
    pub chosen_forge_version: Option<forge::VersionManifestVersion>,
    pub progress_main: util::Progress,
    pub progress_sub: util::Progress,
}

impl Default for State {
    fn default() -> Self {
        Self {
            inner: InnerState::EnterName,
            name_input: String::new(),
            error: None,
            selected: 0,
            chosen_minecraft_version: None,
            chosen_forge_version: None,
            progress_main: util::Progress::new(),
            progress_sub: util::Progress::new(),
        }
    }
}

pub fn get_help(app: &App) -> Vec<(&'static str, &'static str)> {
    match app.new_instance.inner {
        InnerState::EnterName => vec![
            // TODO: ("←/→", "move cursor"),
            ("⏎", "continue"),
            ("ESC", "cancel"),
        ],
        InnerState::FetchMinecraftVersionManifest => Vec::new(),
        InnerState::ChooseMinecraftVersion => vec![
            ("↑↓", "choose version"),
            ("PgUp/PgDn", "move cursor 25"),
            ("⏎", "select"),
            ("ESC", "cancel"),
        ],
        InnerState::ChooseForge => vec![
            ("←/→", "choose option"),
            ("Y", "yes"),
            ("N", "no"),
            ("⏎", "select"),
            ("ESC", "cancel"),
        ],
        InnerState::FetchForgeVersionManifest => Vec::new(),
        InnerState::ChooseForgeVersion => vec![
            ("↑↓", "choose version"),
            ("PgUp/PgDn", "move cursor 25"),
            ("⏎", "select"),
            ("ESC", "cancel"),
        ],
        InnerState::Install => Vec::new(),
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
                Key::Enter => {
                    app.new_instance.chosen_minecraft_version = Some(
                        app.minecraft_version_manifest
                            .as_ref()
                            .unwrap()
                            .versions
                            .get(app.new_instance.selected)
                            .unwrap()
                            .clone(),
                    );
                    app.new_instance.selected = 0;
                    app.new_instance.inner = InnerState::ChooseForge;
                }
                _ => {}
            }
        }
        InnerState::ChooseForge => match key {
            Key::Left => app.new_instance.selected = util::wrap_dec(app.new_instance.selected, 2),
            Key::Right => app.new_instance.selected = util::wrap_inc(app.new_instance.selected, 2),
            Key::Char('y') => {
                app.dispatch(IoEvent::FetchForgeVersionManifest);
                app.new_instance.inner = InnerState::FetchForgeVersionManifest;
                app.new_instance.selected = 0;
            }
            Key::Char('n') => {
                app.dispatch(IoEvent::CreateNewInstance);
                app.new_instance.inner = InnerState::Install;
                app.new_instance.selected = 0;
            }
            Key::Enter => {
                if app.new_instance.selected == 0 {
                    app.dispatch(IoEvent::FetchForgeVersionManifest);
                    app.new_instance.inner = InnerState::FetchForgeVersionManifest;
                } else {
                    app.dispatch(IoEvent::CreateNewInstance);
                    app.new_instance.inner = InnerState::Install;
                }
                app.new_instance.selected = 0;
            }
            _ => {}
        },
        InnerState::FetchForgeVersionManifest => {}
        InnerState::ChooseForgeVersion => {
            let versions_len = app
                .forge_version_manifest
                .as_ref()
                .unwrap()
                .versions
                .iter()
                .filter(|version| {
                    version.game_version
                        == app
                            .new_instance
                            .chosen_minecraft_version
                            .as_ref()
                            .unwrap()
                            .id
                })
                .count();
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
                Key::Enter => {
                    app.new_instance.chosen_forge_version = Some(
                        app.forge_version_manifest
                            .as_ref()
                            .unwrap()
                            .versions
                            .iter()
                            .filter(|version| {
                                version.game_version
                                    == app
                                        .new_instance
                                        .chosen_minecraft_version
                                        .as_ref()
                                        .unwrap()
                                        .id
                            })
                            .collect::<Vec<_>>()
                            .get(app.new_instance.selected)
                            .unwrap()
                            .clone()
                            .clone(),
                    );
                    app.dispatch(IoEvent::CreateNewInstance);
                    app.new_instance.inner = InnerState::Install;
                    app.new_instance.selected = 0;
                }
                _ => {}
            }
        }
        InnerState::Install => {}
    }
}

pub async fn draw(f: &mut UiFrame<'_>, app: &App, chunk: Rect) -> RenderState {
    match &app.new_instance.inner {
        InnerState::EnterName => draw_enter_name(f, app, chunk),
        InnerState::FetchMinecraftVersionManifest => {
            app.new_instance
                .progress_main
                .set_msg("Loading minecraft version manifest...")
                .await;
            draw_loading(f, app, chunk).await
        }
        InnerState::ChooseMinecraftVersion => draw_choose_minecraft_version(f, app, chunk),
        InnerState::ChooseForge => draw_choose_forge(f, app, chunk),
        InnerState::FetchForgeVersionManifest => {
            app.new_instance
                .progress_main
                .set_msg("Loading forge version manifest...")
                .await;
            draw_loading(f, app, chunk).await
        }
        InnerState::ChooseForgeVersion => draw_choose_forge_version(f, app, chunk),
        InnerState::Install => {
            app.new_instance
                .progress_main
                .set_msg("Creating your new instance")
                .await;
            draw_loading(f, app, chunk).await
        }
    }
}

fn draw_enter_name(f: &mut UiFrame<'_>, app: &App, chunk: Rect) -> RenderState {
    let state = &app.new_instance;

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

async fn draw_loading(f: &mut UiFrame<'_>, app: &App, chunk: Rect) -> RenderState {
    debug!("About to draw loading");
    let rect = util::centered_rect_percentage_dir(Direction::Horizontal, 50, chunk);
    debug!("rect1 {:?}", rect);
    let rect = util::centered_rect_dir(Direction::Vertical, 7, rect);
    debug!("rect2 {:?}", rect);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ]
            .as_ref(),
        )
        .margin(2)
        .split(rect);

    let block_widget = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Plain);

    let text = vec![Text::raw(app.new_instance.progress_main.get_msg().await)];
    let text_widget = Paragraph::new(text.iter())
        .style(Style::default().fg(Color::Yellow))
        .alignment(Alignment::Center);

    let ratio = app.new_instance.progress_main.get().await;
    let label = format!("{:0}%", (ratio * 100.0));
    let gauge_widget = Gauge::default()
        .style(
            Style::default()
                .fg(Color::White)
                .bg(Color::Black)
                .modifier(Modifier::ITALIC),
        )
        .label(&label)
        .ratio(ratio);

    f.render_widget(Clear, rect);
    f.render_widget(block_widget, rect);
    f.render_widget(text_widget, layout[0]);
    f.render_widget(gauge_widget, layout[2]);

    RenderState::default()
}

pub fn draw_choose_minecraft_version(f: &mut UiFrame<'_>, app: &App, chunk: Rect) -> RenderState {
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
            .border_type(BorderType::Plain),
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

pub fn draw_choose_forge(f: &mut UiFrame<'_>, app: &App, chunk: Rect) -> RenderState {
    let mut rect = util::centered_rect_percentage_dir(Direction::Horizontal, 60, chunk);
    rect.y = (rect.height / 2) - 5;
    rect.height = 10;

    let block_widget = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Plain);
    f.render_widget(Clear, rect);
    f.render_widget(block_widget, rect);
    rect.x += 2;
    rect.width -= 4;

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(1),
                Constraint::Min(5),
                Constraint::Length(1),
                Constraint::Length(2),
            ]
            .as_ref(),
        )
        .split(rect);

    let text = vec![Text::raw("You will need a forge version to be able to install mods. Using the recommended version is usually a good idea unless you know you need another version. Would you like to install forge for this instance?")];
    let text_widget = Paragraph::new(text.iter())
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Yellow))
        .wrap(true);

    f.render_widget(text_widget, layout[1]);

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
            if app.new_instance.selected == 0 {
                Style::default().fg(Color::Blue).modifier(Modifier::BOLD)
            } else {
                Style::default().modifier(Modifier::DIM)
            },
        ),
        button_layout[1],
    );
    f.render_widget(
        Paragraph::new(vec![Text::raw("[ No ]")].iter()).style(if app.new_instance.selected != 0 {
            Style::default().fg(Color::Blue).modifier(Modifier::BOLD)
        } else {
            Style::default().modifier(Modifier::DIM)
        }),
        button_layout[3],
    );

    RenderState::default()
}

pub fn draw_choose_forge_version(f: &mut UiFrame<'_>, app: &App, chunk: Rect) -> RenderState {
    let rect = util::centered_rect_percentage(90, 75, chunk);

    let versions = &(app.forge_version_manifest.as_ref().unwrap().versions);

    let offset = app
        .new_instance
        .selected
        .saturating_sub((rect.height / 2) as usize)
        .min(versions.len().saturating_sub((rect.height / 2) as usize));

    let rows: Vec<_> = versions
        .iter()
        .filter(|version| {
            version.game_version
                == app
                    .new_instance
                    .chosen_minecraft_version
                    .as_ref()
                    .unwrap()
                    .id
        })
        .skip(offset)
        .take(rect.height as usize)
        .map(|version| {
            Row::Data(
                vec![
                    format!(
                        "{}{}{}",
                        version.name.trim_start_matches("forge-"),
                        if version.latest { " (latest)" } else { "" },
                        if version.recommended {
                            " (recommended)"
                        } else {
                            ""
                        }
                    ),
                    version.date_modified.format("%b %e %Y").to_string(),
                ]
                .into_iter(),
            )
        })
        .collect();

    let table = Table::new(["   Version id", "Release date"].iter(), rows.into_iter())
        .block(
            Block::default()
                .title("Choose forge version")
                .borders(Borders::ALL)
                .border_type(BorderType::Plain),
        )
        .header_style(Style::default().fg(Color::Yellow).modifier(Modifier::BOLD))
        .widths(&[Constraint::Percentage(50), Constraint::Percentage(50)])
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
