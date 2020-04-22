use ::anyhow::{anyhow, Context};
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{BufReader, Read},
    path::Path,
};

use crate::util::java;

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub enum ModFileInfoSource {
    ModsToml,
    McModInfo,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct ModFileInfoMod {
    pub mod_id: String,
    pub name: Option<String>,
    pub version: Option<String>,
    pub description: Option<String>,
    pub authors: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModFileInfo {
    pub sub_mods: Vec<ModFileInfoMod>,
    pub source: ModFileInfoSource,
}

impl ModFileInfo {
    pub fn from_file<P: AsRef<Path>>(path: P) -> ::anyhow::Result<Self> {
        let path = path.as_ref();
        let file = File::open(path)?;

        // Open the mod file archive
        let mut archive = zip::ZipArchive::new(file)?;

        // Read mcmod.info if present
        if let Ok(file) = archive.by_name("mcmod.info") {
            let mut reader = BufReader::new(file);
            let mut s = String::new();
            let _ = reader.read_to_string(&mut s);
            return Self::from_mc_mod_info(&s);
        }

        // Read the manifest.mf file if present - needed to properly get version from mods.toml
        let manifest: Option<java::Manifest> =
            archive.by_name("META-INF/MANIFEST.MF").ok().map(|file| {
                let mut reader = BufReader::new(file);
                let mut s = String::new();
                let _ = reader.read_to_string(&mut s);
                java::parse_manifest(&s)
            });

        // Read the mods.toml if present
        if let Ok(file) = archive.by_name("META-INF/mods.toml") {
            let mut reader = BufReader::new(file);
            let mut s = String::new();
            let _ = reader.read_to_string(&mut s);
            return Self::from_mods_toml(&s, manifest);
        }

        Err(anyhow!("No mod info found."))
    }

    fn from_mc_mod_info(mc_mod_info_str: &str) -> ::anyhow::Result<Self> {
        #[derive(Debug, Deserialize)]
        struct McModInfo {
            #[serde(rename = "modid")]
            mod_id: String,
            name: Option<String>,
            version: Option<String>,
            description: Option<String>,
            #[serde(rename = "authorList")]
            authors: Vec<String>,
        }

        let data: Vec<McModInfo> =
            serde_json::from_str(mc_mod_info_str).context("Failed to read mcmod.info")?;

        let data: Vec<ModFileInfoMod> = data
            .into_iter()
            .map(|info| ModFileInfoMod {
                mod_id: info.mod_id,
                name: info.name,
                version: info.version,
                description: info.description.map(|d| d.trim().to_owned()),
                authors: if !info.authors.is_empty() {
                    Some(info.authors.join(", "))
                } else {
                    None
                },
            })
            .collect();

        Ok(Self {
            sub_mods: data,
            source: ModFileInfoSource::McModInfo,
        })
    }

    fn from_mods_toml(
        mods_toml_str: &str,
        manifest: Option<java::Manifest>,
    ) -> ::anyhow::Result<Self> {
        #[derive(Debug, Deserialize)]
        struct ModsTomlMod {
            #[serde(rename = "modId")]
            mod_id: String,
            version: Option<String>,
            #[serde(rename = "displayName")]
            display_name: String,
            description: String,
            authors: Option<String>,
        }

        #[derive(Debug, Deserialize)]
        struct ModsToml {
            authors: Option<String>,
            mods: Vec<ModsTomlMod>,
        }

        let mods_toml: ModsToml =
            toml::from_str(mods_toml_str).context("Failed to read mods.toml.")?;

        let root_authors = mods_toml.authors.as_ref();
        let manifest_implementation_version = manifest
            .map(|manifest| manifest.get("Implementation-Version").map(String::from))
            .flatten();

        Ok(Self {
            sub_mods: mods_toml
                .mods
                .into_iter()
                .map(|m| ModFileInfoMod {
                    mod_id: m.mod_id,
                    name: Some(m.display_name),
                    version: m
                        .version
                        .map(|v| {
                            if v == "${file.jarVersion}" {
                                manifest_implementation_version.clone()
                            } else {
                                Some(v.to_owned())
                            }
                        })
                        .flatten(),
                    description: Some(m.description.trim().to_owned()),
                    authors: match m.authors {
                        Some(authors) => Some(authors.clone()),
                        None => root_authors.cloned(),
                    },
                })
                .collect(),
            source: ModFileInfoSource::ModsToml,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn get_mod_info_post_1_13() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("resources/test/test-post-1.13-mod-manifest.jar");
        let info = ModFileInfo::from_file(&path).unwrap();
        assert_eq!(info.source, ModFileInfoSource::ModsToml);
        assert_eq!(info.sub_mods.len(), 1);
        assert_eq!(
            info.sub_mods[0],
            ModFileInfoMod {
                authors: Some("testauthor".to_owned()),
                version: Some("6.0.0.3".to_owned()),
                mod_id: "test".to_owned(),
                name: Some("Test mod".to_owned()),
                description: Some("Just a test mod\n".to_owned())
            }
        )
    }

    #[test]
    fn get_mod_info_pre_1_13() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("resources/test/test-pre-1.13-mod-manifest.jar");
        let info = ModFileInfo::from_file(&path).unwrap();
        assert_eq!(info.source, ModFileInfoSource::McModInfo);
        assert_eq!(info.sub_mods.len(), 1);
        assert_eq!(
            info.sub_mods[0],
            ModFileInfoMod {
                authors: Some("testauthor".to_owned()),
                version: Some("6.0.0.3".to_owned()),
                mod_id: "test".to_owned(),
                name: Some("Test mod".to_owned()),
                description: Some("Just a test mod".to_owned())
            }
        )
    }
}
