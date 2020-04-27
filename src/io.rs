use ::anyhow::Context;
use log::debug;
use std::{fs, sync::Arc};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{forge, minecraft, routes, util, App, Instance};

#[derive(Debug)]
pub enum IoEvent {
    NewInstanceFetchMinecraftVersionManifest,
    NewInstanceFetchForgeVersionManifest,
    NewInstance,
    RemoveInstance,
    RenameInstance,
    PlayThenQuit,
    AddForgeFetchVersionManifests,
    AddForge,
    RemoveForge,
}

#[derive(Clone)]
pub struct Io<'a> {
    app: &'a Arc<RwLock<App>>,
    pub client: reqwest::Client,
}

impl<'a> Io<'a> {
    pub fn new(app: &'a Arc<RwLock<App>>, client: reqwest::Client) -> Self {
        Io { app, client }
    }

    pub async fn handle_io_event(&mut self, io_event: IoEvent) -> ::anyhow::Result<()> {
        use IoEvent::*;

        match io_event {
            NewInstanceFetchMinecraftVersionManifest => {
                let exists = { self.app.read().await.minecraft_version_manifest.is_some() };
                if !exists {
                    let (data_file_path, pb) = {
                        let mut app = self.app.write().await;
                        let pb = util::Progress::new();
                        app.state.new_instance.progress_main = Some(pb.clone());
                        (app.paths.file.minecraft_versions_cache.clone(), pb)
                    };

                    let manifest =
                        minecraft::VersionManifest::fetch(&pb, &self.client, &data_file_path)
                            .await?;

                    self.app.write().await.minecraft_version_manifest = Some(manifest);
                }
                self.app.write().await.state.new_instance.inner =
                    routes::new_instance::InnerState::ChooseMinecraftVersion;
            }
            NewInstanceFetchForgeVersionManifest => {
                let exists = { self.app.read().await.forge_version_manifest.is_some() };
                if !exists {
                    let (data_file_path, pb) = {
                        let mut app = self.app.write().await;
                        let pb = util::Progress::new();
                        app.state.new_instance.progress_main = Some(pb.clone());
                        (app.paths.file.forge_versions_cache.clone(), pb)
                    };

                    let manifest =
                        forge::VersionManifest::fetch(&pb, &self.client, &data_file_path).await?;
                    self.app.write().await.forge_version_manifest = Some(manifest);
                }
                self.app.write().await.state.new_instance.inner =
                    routes::new_instance::InnerState::ChooseForgeVersion;
            }
            AddForgeFetchVersionManifests => {
                let exists = { self.app.read().await.minecraft_version_manifest.is_some() };
                if !exists {
                    let (data_file_path, pb) = {
                        let mut app = self.app.write().await;
                        let pb = util::Progress::new();
                        pb.set_msg("Fetching minecraft version manifest...").await;
                        app.state.add_forge.progress_main = Some(pb.clone());
                        (app.paths.file.minecraft_versions_cache.clone(), pb)
                    };

                    let manifest =
                        minecraft::VersionManifest::fetch(&pb, &self.client, &data_file_path)
                            .await?;

                    self.app.write().await.minecraft_version_manifest = Some(manifest);
                }
                let exists = { self.app.read().await.forge_version_manifest.is_some() };
                if !exists {
                    let (data_file_path, pb) = {
                        let mut app = self.app.write().await;
                        let pb = util::Progress::new();
                        pb.set_msg("Fetching forge version manifest...").await;
                        app.state.add_forge.progress_main = Some(pb.clone());
                        (app.paths.file.forge_versions_cache.clone(), pb)
                    };

                    let manifest =
                        forge::VersionManifest::fetch(&pb, &self.client, &data_file_path).await?;
                    self.app.write().await.forge_version_manifest = Some(manifest);
                }

                self.app.write().await.state.add_forge.inner =
                    routes::add_forge::InnerState::ChooseForgeVersion;
            }
            NewInstance => {
                let (name, instance) = {
                    let (minecraft_version, forge_version, name, instances_directory) = {
                        let app = self.app.read().await;
                        (
                            app.state
                                .new_instance
                                .chosen_minecraft_version
                                .clone()
                                .unwrap(),
                            app.state.new_instance.chosen_forge_version.clone(),
                            app.state.new_instance.name_input.clone(),
                            app.paths.directory.instances.clone(),
                        )
                    };
                    let main_pb = {
                        let mut app = self.app.write().await;
                        let pb = util::Progress::new();
                        app.state.new_instance.progress_main = Some(pb.clone());
                        pb
                    };

                    let forge = if let Some(forge_version) = forge_version {
                        main_pb.set_length(10).await;
                        let sub_pb = {
                            let mut app = self.app.write().await;
                            let pb = util::Progress::new();
                            app.state.new_instance.progress_sub = Some(pb.clone());
                            pb
                        };

                        let (forge_version_manifests_cache, launcher, java_home_overwrite) = {
                            let app = self.app.read().await;
                            (
                                app.paths.directory.forge_version_manifests_cache.clone(),
                                app.launcher.clone(),
                                app.java_home_overwrite.clone(),
                            )
                        };
                        forge::install(
                            &main_pb,
                            &sub_pb,
                            &minecraft_version,
                            forge_version.clone(),
                            &forge_version_manifests_cache,
                            &launcher,
                            java_home_overwrite,
                        )
                        .await
                        .context("Failed to install forge")?;

                        Some(forge_version)
                    } else {
                        main_pb.set_length(3).await;
                        None
                    };
                    let uuid = Uuid::new_v4();

                    let instance = Instance {
                        name: name.clone(),
                        uuid,
                        version_id: minecraft_version.id.clone(),
                        forge_name: forge.map(|f| f.name),
                        instances_directory,
                        ..Default::default()
                    };

                    main_pb
                        .inc_with_msg(1, "Creating instance directory.")
                        .await;

                    fs::create_dir_all(&instance.directory())
                        .context("Failed to create instance directory!")?;

                    (name, instance)
                };
                let mut app = self.app.write().await;
                let main_pb = &app.state.new_instance.progress_main.as_ref().unwrap();

                main_pb.inc_with_msg(1, "Ensuring launcher profile.").await;

                app.launcher.ensure_profile(&instance)?;

                main_pb.inc_with_msg(1, "Saving instance").await;

                app.instances.inner.insert(name, instance);
                app.instances.save()?;

                app.pop_route();
            }
            AddForge => {
                let (minecraft_version, forge_version) = {
                    let app = self.app.read().await;
                    let minecraft_version_id = app
                        .state
                        .add_forge
                        .instance
                        .as_ref()
                        .unwrap()
                        .clone()
                        .version_id;
                    (
                        app.minecraft_version_manifest
                            .as_ref()
                            .unwrap()
                            .versions
                            .iter()
                            .find(|v| v.id == minecraft_version_id)
                            .unwrap()
                            .clone(),
                        app.state
                            .add_forge
                            .chosen_forge_version
                            .as_ref()
                            .unwrap()
                            .clone(),
                    )
                };
                let (main_pb, sub_pb) = {
                    let mut app = self.app.write().await;
                    let main_pb = util::Progress::new();
                    main_pb.set_length(9).await;
                    app.state.add_forge.progress_main = Some(main_pb.clone());
                    let sub_pb = util::Progress::new();
                    app.state.add_forge.progress_sub = Some(sub_pb.clone());
                    (main_pb, sub_pb)
                };

                let (forge_version_manifests_cache, launcher, java_home_overwrite) = {
                    let app = self.app.read().await;
                    (
                        app.paths.directory.forge_version_manifests_cache.clone(),
                        app.launcher.clone(),
                        app.java_home_overwrite.clone(),
                    )
                };
                forge::install(
                    &main_pb,
                    &sub_pb,
                    &minecraft_version,
                    forge_version.clone(),
                    &forge_version_manifests_cache,
                    &launcher,
                    java_home_overwrite,
                )
                .await
                .context("Failed to install forge")?;

                let mut app = self.app.write().await;
                let main_pb = &app.state.add_forge.progress_main.as_ref().unwrap();

                let mut instance = app.state.add_forge.instance.as_ref().unwrap().clone();
                instance.forge_name = Some(forge_version.name);

                main_pb.inc_with_msg(1, "Ensuring launcher profile.").await;
                app.launcher.ensure_profile(&instance)?;

                main_pb.inc_with_msg(1, "Saving instance").await;
                app.instances.inner.insert(instance.name.clone(), instance);
                app.instances.save()?;

                app.pop_route();
            }
            RemoveInstance => {
                let instance = {
                    let app = self.app.read().await;
                    let instance = app.state.remove_instance.instance.clone().unwrap();

                    debug!("Removing launcher profile.");
                    app.launcher.remove_profile(&instance)?;

                    debug!("Removing data folder.");
                    let _ = fs::remove_dir_all(&instance.directory());
                    instance
                };

                let mut app = self.app.write().await;
                debug!("Removing from config.");
                app.instances.inner.remove(&instance.name);
                app.instances.save()?;
                app.pop_route();
            }
            RenameInstance => {
                let mut app = self.app.write().await;
                let mut instance = app.state.rename_instance.instance.clone().unwrap();

                let old_name = instance.name;
                instance.name = app.state.rename_instance.name_input.clone();

                app.launcher.ensure_profile(&instance)?;

                app.instances.inner.remove(&old_name);
                app.instances.inner.insert(instance.name.clone(), instance);
                app.instances.save()?;
                app.pop_route();
            }
            RemoveForge => {
                let mut app = self.app.write().await;
                let mut instance = app.state.instance_menu.instance.clone().unwrap();
                instance.forge_name = None;

                app.launcher.ensure_profile(&instance)?;

                app.instances
                    .inner
                    .insert(instance.name.clone(), instance);
                app.instances.save()?;
                app.pop_route();
            }
            PlayThenQuit => {
                {
                    let app = self.app.read().await;
                    let instance = app.state.instance_menu.instance.clone().unwrap();

                    app.launcher.launch_instance(&instance)?;
                }
                let mut app = self.app.write().await;
                app.quit();
            }
        }

        Ok(())
    }
}
