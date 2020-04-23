use ::anyhow::{anyhow, Context};
use chrono::{DateTime, Utc};
use log::debug;
use reqwest;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{BufReader, BufWriter},
    path::PathBuf,
};

use crate::minecraft;

pub const URL: &str = "https://addons-ecs.forgesvc.net/api/v2/minecraft/modloader";

const TIMESTAMP_URL: &str = "https://addons-ecs.forgesvc.net/api/v2/minecraft/modloader/timestamp";

#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct VersionManifestVersion {
    pub name: String,
    pub game_version: String,
    pub latest: bool,
    pub recommended: bool,
    pub date_modified: DateTime<Utc>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct VersionManifest {
    pub versions: Vec<VersionManifestVersion>,
    timestamp: DateTime<Utc>,
}

impl VersionManifest {
    pub async fn fetch(
        client: &reqwest::Client,
        data_file_path: &PathBuf,
    ) -> ::anyhow::Result<Self> {
        // TODO: If less than 15 min since cache was saved, don't even bother the server at all

        let cached_data = match fs::File::open(data_file_path) {
            Ok(file) => {
                debug!("Forge version manifest cache file found.");
                let reader = BufReader::new(file);
                let data: Self = serde_json::from_reader(reader)?;
                Some(data)
            }
            _ => {
                debug!("Forge version manifest cache file not found.");
                None
            }
        };

        let mut current_timestamp: Option<DateTime<Utc>> = None;

        if let Some(data) = cached_data {
            current_timestamp = Some(Self::fetch_timestamp(&client).await?);
            if current_timestamp.clone().unwrap() == data.timestamp {
                debug!("Timestamp is same - cache hit!");
                return Ok(data);
            }
        }

        let response = client
            .get(URL)
            .send()
            .await
            .context("Failed to get forge version manifest.")?
            .error_for_status()?;
        let versions: Vec<VersionManifestVersion> = response
            .json()
            .await
            .context("Failed to decode forge version manifest.")?;

        if current_timestamp.is_none() {
            current_timestamp = Some(Self::fetch_timestamp(&client).await?);
        }

        debug!("Updating cache.");
        let data = Self {
            versions: versions,
            timestamp: current_timestamp.unwrap(),
        };

        let writer = BufWriter::new(
            fs::File::create(&data_file_path)
                .context("Failed to create forge version manifest cache.")?,
        );
        serde_json::to_writer(writer, &data)?;

        Ok(data)
    }

    async fn fetch_timestamp(client: &reqwest::Client) -> ::anyhow::Result<DateTime<Utc>> {
        let response = client
            .get(TIMESTAMP_URL)
            .send()
            .await
            .context("Failed to get forge version manifest timestamp")?
            .error_for_status()?;

        Ok(response
            .json()
            .await
            .context("Failed to decode forge version manifest timestamp")?)
    }

    pub fn find_version_from_name(
        &self,
        forge_name: &str,
    ) -> ::anyhow::Result<VersionManifestVersion> {
        Ok(self
            .versions
            .iter()
            .find(|forge_version| forge_version.name == forge_name)
            .ok_or(anyhow!("Could not find forge version."))
            .map(Clone::clone)?)
    }

    pub fn find_version(
        &self,
        version_str: &str,
        minecraft_version_id: &str,
    ) -> ::anyhow::Result<VersionManifestVersion> {
        let version_str = version_str.trim_start_matches("forge-");
        Ok(self
            .versions
            .iter()
            .find(|forge_version| {
                forge_version.name.trim_start_matches("forge-") == version_str
                    || (forge_version.game_version == minecraft_version_id
                        && ((version_str == "recommended" && forge_version.recommended)
                            || (version_str == "latest" && forge_version.latest)))
            })
            .ok_or(anyhow!(
                "Specified forge version could not be found for version of minecraft."
            ))
            .map(Clone::clone)?)
    }
}
