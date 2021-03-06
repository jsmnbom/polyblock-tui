use async_trait::async_trait;
use tui::{
    layout::{Constraint, Rect},
    widgets::Row,
};

use super::*;
use crate::{forge, minecraft, util, IoEvent};

#[derive(Clone)]
pub enum InnerState {
    EnterName,
    FetchMinecraftVersionManifest,
    ChooseMinecraftVersion,
    ChooseForge,
    ForgeNotice,
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
    pub progress_main: Option<util::Progress>,
    pub progress_sub: Option<util::Progress>,
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
            progress_main: None,
            progress_sub: None,
        }
    }
}

pub struct Impl {}

#[async_trait]
impl RouteImpl for Impl {
    fn is_modal(&self) -> bool {
        true
    }
    fn get_help(&self, app: &App) -> Vec<(&'static str, &'static str)> {
        match app.state.new_instance.inner {
            InnerState::EnterName => vec![
                ("ESC", "cancel"),
                // TODO: ("←/→", "move cursor"),
                ("⏎", "continue"),
            ],
            InnerState::FetchMinecraftVersionManifest => Vec::new(),
            InnerState::ChooseMinecraftVersion => vec![
                ("ESC", "cancel"),
                ("↑↓", "choose version"),
                ("PgUp/PgDn", "move cursor 25"),
                ("⏎", "select"),
            ],
            InnerState::ChooseForge => vec![
                ("ESC", "cancel"),
                ("←/→", "choose option"),
                ("Y", "yes"),
                ("N", "no"),
                ("⏎", "select"),
            ],
            InnerState::ForgeNotice => vec![("⏎", "continue"), ("ESC", "cancel")],
            InnerState::FetchForgeVersionManifest => Vec::new(),
            InnerState::ChooseForgeVersion => vec![
                ("ESC", "cancel"),
                ("↑↓", "choose version"),
                ("PgUp/PgDn", "move cursor 25"),
                ("⏎", "select"),
            ],
            InnerState::Install => Vec::new(),
        }
    }
    fn handle_key(&self, key: Key, app: &mut App) {
        match app.state.new_instance.inner {
            InnerState::EnterName => {
                match key {
                    Key::Char(c) => {
                        app.state.new_instance.name_input.push(c);
                    }
                    Key::Backspace => {
                        app.state.new_instance.name_input.pop();
                    }
                    Key::Enter => {
                        if app.state.new_instance.error.is_none() {
                            app.dispatch(IoEvent::NewInstanceFetchMinecraftVersionManifest);
                            app.state.new_instance.inner =
                                InnerState::FetchMinecraftVersionManifest;
                        }
                    }
                    _ => {}
                }

                if app.state.new_instance.name_input.is_empty() {
                    app.state.new_instance.error = Some("You must enter a name!".to_string())
                } else if app
                    .instances
                    .inner
                    .keys()
                    .collect::<Vec<&String>>()
                    .contains(&&app.state.new_instance.name_input)
                {
                    app.state.new_instance.error =
                        Some("An instance with that name already exists!".to_string())
                } else {
                    app.state.new_instance.error = None;
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
                        app.state.new_instance.selected =
                            util::wrap_dec(app.state.new_instance.selected, versions_len)
                    }
                    Key::Down => {
                        app.state.new_instance.selected =
                            util::wrap_inc(app.state.new_instance.selected, versions_len)
                    }
                    Key::PageUp => {
                        app.state.new_instance.selected =
                            util::wrap_sub(app.state.new_instance.selected, versions_len, 25)
                    }
                    Key::PageDown => {
                        app.state.new_instance.selected =
                            util::wrap_add(app.state.new_instance.selected, versions_len, 25)
                    }
                    Key::Enter => {
                        app.state.new_instance.chosen_minecraft_version = Some(
                            app.minecraft_version_manifest
                                .as_ref()
                                .unwrap()
                                .versions
                                .get(app.state.new_instance.selected)
                                .unwrap()
                                .clone(),
                        );
                        app.state.new_instance.selected = 0;
                        app.state.new_instance.inner = InnerState::ChooseForge;
                    }
                    _ => {}
                }
            }
            InnerState::ChooseForge => match key {
                Key::Left => {
                    app.state.new_instance.selected =
                        util::wrap_dec(app.state.new_instance.selected, 2)
                }
                Key::Right => {
                    app.state.new_instance.selected =
                        util::wrap_inc(app.state.new_instance.selected, 2)
                }
                Key::Char('y') => {
                    app.state.new_instance.inner = InnerState::ForgeNotice;
                    app.state.new_instance.selected = 0;
                }
                Key::Char('n') => {
                    app.dispatch(IoEvent::NewInstance);
                    app.state.new_instance.inner = InnerState::Install;
                    app.state.new_instance.selected = 0;
                }
                Key::Enter => {
                    if app.state.new_instance.selected == 0 {
                        app.state.new_instance.inner = InnerState::ForgeNotice;
                    } else {
                        app.dispatch(IoEvent::NewInstance);
                        app.state.new_instance.inner = InnerState::Install;
                    }
                    app.state.new_instance.selected = 0;
                }
                _ => {}
            },
            InnerState::ForgeNotice => match key {
                Key::Enter => {
                    app.dispatch(IoEvent::NewInstanceFetchForgeVersionManifest);
                    app.state.new_instance.inner = InnerState::FetchForgeVersionManifest;
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
                                .state
                                .new_instance
                                .chosen_minecraft_version
                                .as_ref()
                                .unwrap()
                                .id
                    })
                    .count();
                match key {
                    Key::Up => {
                        app.state.new_instance.selected =
                            util::wrap_dec(app.state.new_instance.selected, versions_len)
                    }
                    Key::Down => {
                        app.state.new_instance.selected =
                            util::wrap_inc(app.state.new_instance.selected, versions_len)
                    }
                    Key::PageUp => {
                        app.state.new_instance.selected =
                            util::wrap_sub(app.state.new_instance.selected, versions_len, 25)
                    }
                    Key::PageDown => {
                        app.state.new_instance.selected =
                            util::wrap_add(app.state.new_instance.selected, versions_len, 25)
                    }
                    Key::Enter => {
                        app.state.new_instance.chosen_forge_version = Some(
                            app.forge_version_manifest
                                .as_ref()
                                .unwrap()
                                .versions
                                .iter()
                                .filter(|version| {
                                    version.game_version
                                        == app
                                            .state
                                            .new_instance
                                            .chosen_minecraft_version
                                            .as_ref()
                                            .unwrap()
                                            .id
                                })
                                .collect::<Vec<_>>()
                                .get(app.state.new_instance.selected)
                                .unwrap()
                                .clone()
                                .clone(),
                        );
                        app.dispatch(IoEvent::NewInstance);
                        app.state.new_instance.inner = InnerState::Install;
                        app.state.new_instance.selected = 0;
                    }
                    _ => {}
                }
            }
            InnerState::Install => {}
        }
    }
    async fn draw(&self, f: &mut UiFrame<'_>, app: &mut App, chunk: Rect) {
        match &app.state.new_instance.inner {
            InnerState::EnterName => draw_enter_name(f, app, chunk),
            InnerState::FetchMinecraftVersionManifest => {
                draw_loading(f, app, chunk, "Loading minecraft version manifest...").await
            }
            InnerState::ChooseMinecraftVersion => draw_choose_minecraft_version(f, app, chunk),
            InnerState::ChooseForge => draw_choose_forge(f, app, chunk),
            InnerState::ForgeNotice => draw_forge_notice(f, app, chunk),
            InnerState::FetchForgeVersionManifest => {
                draw_loading(f, app, chunk, "Loading forge version manifest...").await
            }
            InnerState::ChooseForgeVersion => draw_choose_forge_version(f, app, chunk),
            InnerState::Install => draw_loading(f, app, chunk, "Creating your new instance").await,
        }
    }
}

async fn draw_loading(f: &mut UiFrame<'_>, app: &mut App, chunk: Rect, msg: &str) {
    common::draw_loading_dialog(
        f,
        chunk,
        msg,
        &[
            app.state.new_instance.progress_main.as_ref(),
            app.state.new_instance.progress_sub.as_ref(),
        ],
    )
    .await
}

fn draw_enter_name(f: &mut UiFrame<'_>, app: &mut App, chunk: Rect) {
    app.show_cursor();
    common::draw_input_dialog(
        f,
        chunk,
        "Enter new instance name",
        &app.state.new_instance.name_input,
        app.state.new_instance.error.as_deref(),
    )
}

pub fn draw_choose_minecraft_version(f: &mut UiFrame<'_>, app: &mut App, chunk: Rect) {
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
        .state
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

    common::draw_table(
        f,
        rect,
        &["   Version id", "Type", "Release date"],
        rows,
        &[
            Constraint::Percentage(40),
            Constraint::Percentage(30),
            Constraint::Percentage(30),
        ],
        Some("Choose minecraft version"),
        Some(app.state.new_instance.selected - offset),
    )
}

pub fn draw_forge_notice(f: &mut UiFrame<'_>, _app: &mut App, chunk: Rect) {
    let text = "Forge is an open source project that mostly relies on ad revenue.
By using Polyblock you bypass viewing these ads.
Please strongly consider supporting the creator of Forge LexManos' Patreon.
https://www.patreon.com/LexManos";
    common::draw_button_dialog(f, chunk, 10, text, vec!["[ Ok ]"], 0)
}

pub fn draw_choose_forge(f: &mut UiFrame<'_>, app: &mut App, chunk: Rect) {
    let text = "You will need a forge version to be able to install mods. Using the recommended version is usually a good idea unless you know you need another version. Would you like to install forge for this instance?";
    common::draw_button_dialog(
        f,
        chunk,
        10,
        text,
        vec!["[ Yes ]", "[ No ]"],
        app.state.new_instance.selected,
    )
}

pub fn draw_choose_forge_version(f: &mut UiFrame<'_>, app: &mut App, chunk: Rect) {
    let rect = util::centered_rect_percentage(90, 75, chunk);

    let versions = &(app.forge_version_manifest.as_ref().unwrap().versions);
    // TODO: sort

    let offset = app
        .state
        .new_instance
        .selected
        .saturating_sub((rect.height / 2) as usize)
        .min(versions.len().saturating_sub((rect.height / 2) as usize));

    let rows: Vec<_> = versions
        .iter()
        .filter(|version| {
            version.game_version
                == app
                    .state
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

    common::draw_table(
        f,
        rect,
        &["   Version id", "Release date"],
        rows,
        &[Constraint::Percentage(50), Constraint::Percentage(50)],
        Some("Choose forge version"),
        Some(app.state.new_instance.selected - offset),
    )
}
