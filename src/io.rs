use ::anyhow::Context;
use log::debug;
use std::{fs, sync::Arc};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{cleanup_terminal, forge, minecraft, util, view, App, Instance};

#[derive(Debug)]
pub enum IoEvent {
    FetchMinecraftVersionManifest,
    FetchForgeVersionManifest,
    CreateNewInstance,
    RemoveInstance,
    RenameInstance,
    PlayThenQuit,
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
            FetchMinecraftVersionManifest => {
                let exists = { self.app.read().await.minecraft_version_manifest.is_none() };
                if exists {
                    let (data_file_path, pb) = {
                        let mut app = self.app.write().await;
                        let pb = util::Progress::new();
                        app.new_instance.progress_main = Some(pb.clone());
                        (app.paths.file.minecraft_versions_cache.clone(), pb)
                    };

                    let manifest =
                        minecraft::VersionManifest::fetch(&pb, &self.client, &data_file_path)
                            .await?;

                    self.app.write().await.minecraft_version_manifest = Some(manifest);
                }
                self.app.write().await.new_instance.inner =
                    view::new_instance::InnerState::ChooseMinecraftVersion;
            }
            FetchForgeVersionManifest => {
                let exists = { self.app.read().await.forge_version_manifest.is_none() };
                if exists {
                    let (data_file_path, pb) = {
                        let mut app = self.app.write().await;
                        let pb = util::Progress::new();
                        app.new_instance.progress_main = Some(pb.clone());
                        (app.paths.file.forge_versions_cache.clone(), pb)
                    };

                    let manifest =
                        forge::VersionManifest::fetch(&pb, &self.client, &data_file_path).await?;
                    self.app.write().await.forge_version_manifest = Some(manifest);
                }
                self.app.write().await.new_instance.inner =
                    view::new_instance::InnerState::ChooseForgeVersion;
            }
            CreateNewInstance => {
                let (name, instance) = {
                    let (minecraft_version, forge_version, name, instances_directory) = {
                        let app = self.app.read().await;
                        (
                            app.new_instance.chosen_minecraft_version.clone().unwrap(),
                            app.new_instance.chosen_forge_version.clone(),
                            app.new_instance.name_input.clone(),
                            app.paths.directory.instances.clone(),
                        )
                    };
                    let main_pb = {
                        let mut app = self.app.write().await;
                        let pb = util::Progress::new();
                        app.new_instance.progress_main = Some(pb.clone());
                        pb
                    };

                    let forge = if let Some(forge_version) = forge_version {
                        main_pb.set_length(10).await;
                        let sub_pb = {
                            let mut app = self.app.write().await;
                            let pb = util::Progress::new();
                            app.new_instance.progress_sub = Some(pb.clone());
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
                let main_pb = &app.new_instance.progress_main.as_ref().unwrap();

                main_pb.inc_with_msg(1, "Ensuring launcher profile.").await;

                app.launcher.ensure_profile(&instance)?;

                main_pb.inc_with_msg(1, "Saving instance").await;

                app.instances.inner.insert(name, instance);
                app.instances.save()?;

                app.pop_route();
            }
            RemoveInstance => {
                let instance = {
                    let app = self.app.read().await;
                    let instance = app.remove_instance.instance.clone().unwrap();

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
            }
            RenameInstance => {
                let mut app = self.app.write().await;
                let mut instance = app.rename_instance.instance.clone().unwrap();

                let old_name = instance.name;
                instance.name = app.rename_instance.name_input.clone();

                app.instances.inner.remove(&old_name);
                app.instances
                    .inner
                    .insert(instance.name.clone(), instance.clone());
                app.instances.save()?;

                app.launcher.ensure_profile(&instance)?;
            }
            PlayThenQuit => {
                let app = self.app.read().await;
                let instance = app.instance_menu.instance.clone().unwrap();

                app.launcher.launch_instance(&instance)?;
                // TODO: Signal to main thread to exit instead of doing it here on the io thread
                cleanup_terminal();
                std::process::exit(0);
            }
        }

        Ok(())
    }
}
