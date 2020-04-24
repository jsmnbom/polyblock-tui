use ::anyhow::Context;
use std::{fs, sync::Arc};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::{forge, minecraft, view, App, Instance};

#[derive(Debug)]
pub enum IoEvent {
    FetchMinecraftVersionManifest,
    FetchForgeVersionManifest,
    CreateNewInstance,
}

#[derive(Clone)]
pub struct Io<'a> {
    app: &'a Arc<Mutex<App>>,
    pub client: reqwest::Client,
}

impl<'a> Io<'a> {
    pub fn new(app: &'a Arc<Mutex<App>>, client: reqwest::Client) -> Self {
        Io { app, client }
    }

    pub async fn handle_io_event(&mut self, io_event: IoEvent) -> ::anyhow::Result<()> {
        use IoEvent::*;

        match io_event {
            FetchMinecraftVersionManifest => {
                if self.app.lock().await.minecraft_version_manifest.is_none() {
                    let (data_file_path, pb) = {
                        let app = self.app.lock().await;
                        (
                            app.paths.file.minecraft_versions_cache.clone(),
                            app.new_instance.progress_main.clone(),
                        )
                    };
                    pb.reset().await;

                    let manifest =
                        minecraft::VersionManifest::fetch(&pb, &self.client, &data_file_path)
                            .await?;

                    self.app.lock().await.minecraft_version_manifest = Some(manifest);
                }
                self.app.lock().await.new_instance.inner =
                    view::new_instance::InnerState::ChooseMinecraftVersion;
            }
            FetchForgeVersionManifest => {
                if self.app.lock().await.forge_version_manifest.is_none() {
                    let (data_file_path, pb) = {
                        let app = self.app.lock().await;
                        (
                            app.paths.file.forge_versions_cache.clone(),
                            app.new_instance.progress_main.clone(),
                        )
                    };
                    pb.reset().await;

                    let manifest =
                        forge::VersionManifest::fetch(&pb, &self.client, &data_file_path).await?;
                    self.app.lock().await.forge_version_manifest = Some(manifest);
                }
                self.app.lock().await.new_instance.inner =
                    view::new_instance::InnerState::ChooseForgeVersion;
            }
            CreateNewInstance => {
                let minecraft_version = self
                    .app
                    .lock()
                    .await
                    .new_instance
                    .chosen_minecraft_version
                    .clone()
                    .unwrap();
                let forge_version = self
                    .app
                    .lock()
                    .await
                    .new_instance
                    .chosen_forge_version
                    .clone();

                let forge = if let Some(forge_version) = forge_version {
                    // forge::install(
                    //     version,
                    //     forge_version.clone(),
                    //     &paths.directory.forge_version_manifests_cache,
                    //     &launcher,
                    //     java_home,
                    // )
                    // .await
                    // .context("Failed to install forge")?;

                    Some(forge_version)
                } else {
                    None
                };

                let mut app = self.app.lock().await;
                let uuid = Uuid::new_v4();
                let name = app.new_instance.name_input.clone();

                let instance = Instance {
                    name: name.clone(),
                    uuid,
                    version_id: minecraft_version.id.clone(),
                    forge_name: forge.map(|f| f.name),
                    instances_directory: app.paths.directory.instances.clone(),
                    ..Default::default()
                };

                fs::create_dir_all(&instance.directory())
                    .context("Failed to create instance directory!")?;

                app.launcher.ensure_profile(&instance)?;

                app.instances.inner.insert(name, instance);
                app.instances.save()?;

                app.pop_route();
            }
        }

        Ok(())
    }
}
