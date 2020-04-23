use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{minecraft, view, App};

#[derive(Debug)]
pub enum IoEvent {
    FetchMinecraftVersionManifest,
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
                let data_file_path = {
                    let app = self.app.lock().await;
                    app.paths.file.minecraft_versions_cache.clone()
                };

                let manifest =
                    minecraft::VersionManifest::fetch(&self.client, &data_file_path).await?;
                let mut app = self.app.lock().await;
                app.minecraft_version_manifest = Some(manifest);
                app.new_instance.inner = view::new_instance::InnerState::ChooseMinecraftVersion;
            }
        }

        Ok(())
    }
}
