use ::anyhow::Context;
use chrono::{DateTime, Utc};
use log::debug;
use reqwest;
use serde::{Deserialize, Serialize};
use std::{
    cmp::Reverse,
    fs,
    io::{BufReader, BufWriter},
    path::PathBuf,
};

use crate::util;

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
        pb: &util::Progress,
        client: &reqwest::Client,
        data_file_path: &PathBuf,
    ) -> ::anyhow::Result<Self> {
        // TODO: If less than 15 min since cache was saved, don't even bother the server at all

        // the endpoint provides no content-length data so a proper progress bar here is near impossible
        pb.set_length(5).await;

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

        pb.inc(1).await;

        let mut current_timestamp: Option<DateTime<Utc>> = None;

        if let Some(data) = cached_data {
            current_timestamp = Some(Self::fetch_timestamp(&client).await?);
            if current_timestamp.clone().unwrap() == data.timestamp {
                debug!("Timestamp is same - cache hit!");
                return Ok(data);
            }
        }

        pb.inc(1).await;

        let response = client
            .get(URL)
            .send()
            .await
            .context("Failed to get forge version manifest.")?
            .error_for_status()?;

        pb.inc(1).await;

        let mut versions: Vec<VersionManifestVersion> = response
            .json()
            .await
            .context("Failed to decode forge version manifest.")?;

        versions.sort_unstable_by_key(|version| Reverse(version.date_modified));

        pb.inc(1).await;

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

        pb.inc(1).await;

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
}
