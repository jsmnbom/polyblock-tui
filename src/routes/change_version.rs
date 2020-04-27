use async_trait::async_trait;
use tui::{
    layout::{Constraint, Rect},
    widgets::Row,
};

use super::*;
use crate::{minecraft, util, Instance, IoEvent};

#[derive(Clone)]
pub enum InnerState {
    FetchVersionManifest,
    ForgeWarning,
    ChooseVersion,
    Install,
}

#[derive(Clone)]
pub struct State {
    pub inner: InnerState,
    pub instance: Option<Instance>,
    selected: usize,
    pub chosen_version: Option<minecraft::VersionManifestVersion>,
    pub progress: Option<util::Progress>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            inner: InnerState::FetchVersionManifest,
            selected: 0,
            chosen_version: None,
            progress: None,
            instance: None,
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
        match app.state.change_version.inner {
            InnerState::FetchVersionManifest => Vec::new(),
            InnerState::ForgeWarning => vec![("⏎", "continue"), ("ESC", "cancel")],
            InnerState::ChooseVersion => vec![
                ("ESC", "cancel"),
                ("↑↓", "choose version"),
                ("PgUp/PgDn", "move cursor 25"),
                ("⏎", "select"),
            ],
            InnerState::Install => Vec::new(),
        }
    }
    fn handle_key(&self, key: Key, app: &mut App) {
        match app.state.change_version.inner {
            InnerState::FetchVersionManifest => {}
            InnerState::ForgeWarning => match key {
                Key::Enter => {
                    app.state.change_version.inner = InnerState::ChooseVersion;
                }
                _ => {}
            },
            InnerState::ChooseVersion => {
                let versions_len = app
                    .minecraft_version_manifest
                    .as_ref()
                    .unwrap()
                    .versions
                    .len();
                match key {
                    Key::Up => {
                        app.state.change_version.selected =
                            util::wrap_dec(app.state.change_version.selected, versions_len)
                    }
                    Key::Down => {
                        app.state.change_version.selected =
                            util::wrap_inc(app.state.change_version.selected, versions_len)
                    }
                    Key::PageUp => {
                        app.state.change_version.selected =
                            util::wrap_sub(app.state.change_version.selected, versions_len, 25)
                    }
                    Key::PageDown => {
                        app.state.change_version.selected =
                            util::wrap_add(app.state.change_version.selected, versions_len, 25)
                    }
                    Key::Enter => {
                        app.state.change_version.chosen_version = Some(
                            app.minecraft_version_manifest
                                .as_ref()
                                .unwrap()
                                .versions
                                .get(app.state.change_version.selected)
                                .unwrap()
                                .clone(),
                        );
                        app.dispatch(IoEvent::ChangeVersion);
                        app.state.change_version.inner = InnerState::Install;
                    }
                    _ => {}
                }
            }
            InnerState::Install => {}
        }
    }
    async fn draw(&self, f: &mut UiFrame<'_>, app: &mut App, chunk: Rect) {
        match &app.state.change_version.inner {
            InnerState::FetchVersionManifest => {
                draw_loading(f, app, chunk, "Loading minecraft version manifest...").await
            }
            InnerState::ForgeWarning => draw_forge_warning(f, app, chunk),
            InnerState::ChooseVersion => draw_choose_minecraft_version(f, app, chunk),
            InnerState::Install => draw_loading(f, app, chunk, "Updating your instance.").await,
        }
    }
}

async fn draw_loading(f: &mut UiFrame<'_>, app: &mut App, chunk: Rect, msg: &str) {
    common::draw_loading_dialog(f, chunk, msg, &[app.state.change_version.progress.as_ref()]).await
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
        .change_version
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
        Some(app.state.change_version.selected - offset),
    )
}

pub fn draw_forge_warning(f: &mut UiFrame<'_>, _app: &mut App, chunk: Rect) {
    let text = "This instance currently has forge installed. Changing the minecraft version will remove forge. To reinstall it use the 'Add Forge' option in the menu.";
    common::draw_button_dialog(f, chunk, 10, text, vec!["[ Ok ]"], 0)
}
