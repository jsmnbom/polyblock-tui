use ::anyhow::Context;
use reqwest;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AddonFile {
    pub release_type: u64,
    pub file_name: String,
    pub game_version: Vec<String>,
    pub download_url: String,
    pub project_id: u64,
    pub id: u64,
}

impl AddonFile {
    pub async fn fetch_files(project_id: u64) -> ::anyhow::Result<Vec<AddonFile>> {
        let client = reqwest::Client::new();
        let url = format!(
            "https://addons-ecs.forgesvc.net/api/v2/addon/{}/files",
            project_id
        );
        let response = client
            .get(&url)
            .send()
            .await
            .context("Failed to get addon files.")?
            .error_for_status()
            .context("Failed to get addon files.")?;
    
        #[derive(Deserialize, Serialize, Debug, Clone)]
        #[serde(rename_all = "camelCase")]
        pub struct RawAddonFile {
            release_type: u64,
            file_name: String,
            game_version: Vec<String>,
            download_url: String,
            id: u64,
        }
    
        let files: Vec<RawAddonFile> = response
            .json()
            .await
            .context("Failed to decode addon files.")?;
    
        let files: Vec<AddonFile> = files
            .into_iter()
            .map(|file| AddonFile {
                release_type: file.release_type,
                file_name: file.file_name,
                game_version: file.game_version,
                download_url: file.download_url,
                id: file.id,
                project_id,
            })
            .collect();
    
        Ok(files)
    }
}
