use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{minecraft, App};

#[derive(Debug)]
pub enum IoEvent {
    FetchMinecraftVersionManifest,
}

#[derive(Clone)]
pub struct Io<'a> {
    app: &'a Arc<Mutex<App>>,
}

impl<'a> Io<'a> {
    pub fn new(app: &'a Arc<Mutex<App>>) -> Self {
        Io { app }
    }

    pub async fn handle_io_event(&mut self, io_event: IoEvent) -> ::anyhow::Result<()> {
        use IoEvent::*;

        match io_event {
            FetchMinecraftVersionManifest => {
                let data_file_path = {
                    self.app
                        .lock()
                        .await
                        .paths
                        .file
                        .minecraft_versions_cache
                        .clone()
                };

                let manifest = minecraft::VersionManifest::fetch(&data_file_path).await?;
                let mut app = self.app.lock().await;
                app.minecraft_version_manifest = Some(manifest);
            }
        }

        Ok(())
    }
}
