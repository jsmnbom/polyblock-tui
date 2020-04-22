use anyhow::Context;
use chrono::{DateTime, Utc};
use log::debug;
use reqwest;
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::PathBuf,
};

const URL: &'static str = "https://launchermeta.mojang.com/mc/game/version_manifest.json";

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VersionManifestVersionType {
    Snapshot,
    Release,
    OldBeta,
    OldAlpha,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VersionManifestLatest {
    pub release: String,
    pub snapshot: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct VersionManifestVersion {
    pub id: String,
    pub r#type: VersionManifestVersionType,
    pub url: String,
    pub time: DateTime<Utc>,
    pub release_time: DateTime<Utc>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VersionManifest {
    pub latest: VersionManifestLatest,
    pub versions: Vec<VersionManifestVersion>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Data {
    pub manifest: VersionManifest,
    pub etag: String,
}

impl VersionManifest {
    pub async fn fetch(data_file_path: &PathBuf) -> ::anyhow::Result<Self> {
        // TODO: If less than 15 min since cache was saved, don't even bother the server at all
        let client = reqwest::Client::new();

        let cached_data = match File::open(data_file_path) {
            Ok(file) => {
                debug!("Version manifest cache file found.");
                let reader = BufReader::new(file);
                let data: Data = serde_json::from_reader(reader)?;
                Some(data)
            }
            _ => {
                debug!("Version manifest cache file not found.");
                None
            }
        };

        let mut builder = client.get(URL);
        if let Some(data) = cached_data.clone() {
            builder = builder.header(reqwest::header::IF_NONE_MATCH, data.etag)
        }
        let response = builder
            .send()
            .await
            .context("Failed to get minecraft version manifest.")?
            .error_for_status()?;

        let etag: Option<String> = response
            .headers()
            .get(reqwest::header::ETAG)
            .as_ref()
            .map(|etag| etag.to_str().unwrap().to_owned());

        let manifest = match response.status() {
            reqwest::StatusCode::NOT_MODIFIED => {
                debug!("Not modified - cache hit.");
                cached_data.clone().unwrap().manifest
            }
            _ => response
                .json()
                .await
                .context("Failed to decode minecraft version manifest.")?,
        };

        if cached_data.is_none()
            || (etag.is_some() && etag.clone().unwrap() != cached_data.unwrap().etag)
        {
            debug!("Updating cache.");
            let data = Data {
                manifest: manifest.clone(),
                etag: etag.unwrap(),
            };

            let writer = BufWriter::new(
                File::create(&data_file_path)
                    .context("Failed to create minecraft version manifest cache.")?,
            );
            serde_json::to_writer(writer, &data)?;
        }

        Ok(manifest)
    }
}
