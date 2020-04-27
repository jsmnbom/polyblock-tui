use async_trait::async_trait;
use tui::{
    layout::{Constraint, Rect},
    widgets::Row,
};

use super::*;
use crate::{forge, util, Instance, IoEvent};

#[derive(Clone)]
pub enum InnerState {
    ForgeNotice,
    FetchVersionManifests,
    ChooseForgeVersion,
    Install,
}

pub struct State {
    pub inner: InnerState,
    pub instance: Option<Instance>,
    pub selected: usize,
    pub chosen_forge_version: Option<forge::VersionManifestVersion>,
    pub progress_main: Option<util::Progress>,
    pub progress_sub: Option<util::Progress>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            inner: InnerState::ForgeNotice,
            instance: None,
            selected: 0,
            chosen_forge_version: None,
            progress_main: None,
            progress_sub: None,
        }
    }
}

impl State {
    pub fn new(instance: Instance) -> Self {
        Self {
            instance: Some(instance),
            ..Default::default()
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
        match app.state.add_forge.inner {
            InnerState::ForgeNotice => vec![("⏎", "continue"), ("ESC", "cancel")],
            InnerState::FetchVersionManifests => Vec::new(),
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
        match app.state.add_forge.inner {
            InnerState::ForgeNotice => match key {
                Key::Enter => {
                    app.dispatch(IoEvent::AddForgeFetchVersionManifests);
                    app.state.add_forge.inner = InnerState::FetchVersionManifests;
                }
                _ => {}
            },
            InnerState::FetchVersionManifests => {}
            InnerState::ChooseForgeVersion => {
                let versions_len = app
                    .forge_version_manifest
                    .as_ref()
                    .unwrap()
                    .versions
                    .iter()
                    .filter(|version| {
                        version.game_version
                            == app.state.add_forge.instance.as_ref().unwrap().version_id
                    })
                    .count();
                match key {
                    Key::Up => {
                        app.state.add_forge.selected =
                            util::wrap_dec(app.state.add_forge.selected, versions_len)
                    }
                    Key::Down => {
                        app.state.add_forge.selected =
                            util::wrap_inc(app.state.add_forge.selected, versions_len)
                    }
                    Key::PageUp => {
                        app.state.add_forge.selected =
                            util::wrap_sub(app.state.add_forge.selected, versions_len, 25)
                    }
                    Key::PageDown => {
                        app.state.add_forge.selected =
                            util::wrap_add(app.state.add_forge.selected, versions_len, 25)
                    }
                    Key::Enter => {
                        app.state.add_forge.chosen_forge_version = Some(
                            app.forge_version_manifest
                                .as_ref()
                                .unwrap()
                                .versions
                                .iter()
                                .filter(|version| {
                                    version.game_version
                                        == app.state.add_forge.instance.as_ref().unwrap().version_id
                                })
                                .collect::<Vec<_>>()
                                .get(app.state.add_forge.selected)
                                .unwrap()
                                .clone()
                                .clone(),
                        );
                        app.dispatch(IoEvent::AddForge);
                        app.state.add_forge.inner = InnerState::Install;
                        app.state.add_forge.selected = 0;
                    }
                    _ => {}
                }
            }
            InnerState::Install => {}
        }
    }
    async fn draw(&self, f: &mut UiFrame<'_>, app: &mut App, chunk: Rect) {
        match &app.state.add_forge.inner {
            InnerState::ForgeNotice => draw_forge_notice(f, app, chunk),
            InnerState::FetchVersionManifests => {
                draw_loading(f, app, chunk, "Loading version manifests...").await
            }
            InnerState::ChooseForgeVersion => draw_choose_forge_version(f, app, chunk),
            InnerState::Install => draw_loading(f, app, chunk, "Updating your instance.").await,
        }
    }
}

async fn draw_loading(f: &mut UiFrame<'_>, app: &mut App, chunk: Rect, msg: &str) {
    common::draw_loading_dialog(
        f,
        chunk,
        msg,
        &[
            app.state.add_forge.progress_main.as_ref(),
            app.state.add_forge.progress_sub.as_ref(),
        ],
    )
    .await
}

pub fn draw_forge_notice(f: &mut UiFrame<'_>, _app: &mut App, chunk: Rect) {
    let text = "Forge is an open source project that mostly relies on ad revenue.
By using Polyblock you bypass viewing these ads.
Please strongly consider supporting the creator of Forge LexManos' Patreon.
https://www.patreon.com/LexManos";
    common::draw_button_dialog(f, chunk, 10, text, vec!["[ Ok ]"], 0)
}

pub fn draw_choose_forge_version(f: &mut UiFrame<'_>, app: &mut App, chunk: Rect) {
    let rect = util::centered_rect_percentage(90, 75, chunk);

    let versions = &(app.forge_version_manifest.as_ref().unwrap().versions);
    // TODO: sort

    let offset = app
        .state
        .add_forge
        .selected
        .saturating_sub((rect.height / 2) as usize)
        .min(versions.len().saturating_sub((rect.height / 2) as usize));

    let rows: Vec<_> = versions
        .iter()
        .filter(|version| {
            version.game_version == app.state.add_forge.instance.as_ref().unwrap().version_id
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
        Some(app.state.add_forge.selected - offset),
    )
}
