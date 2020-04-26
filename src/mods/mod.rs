use serde::{Deserialize, Serialize};
use std::path::Path;

mod curse;
mod file_info;

pub use curse::AddonFile;
pub use file_info::{ModFileInfo, ModFileInfoSource};

use crate::util;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ModInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<ModFileInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub curse: Option<AddonFile>,
}

impl ModInfo {
    pub fn from_file<P: AsRef<Path>>(path: P) -> ::anyhow::Result<Self> {
        Ok(Self {
            file: Some(ModFileInfo::from_file(path.as_ref())?),
            curse: None,
        })
    }

    // pub fn print(&self, title: &str, full: bool) {
    //     // TODO: Handle if file is missing
    //     let file_info = self.file.as_ref();
    //     let curse_info = self.curse.as_ref();

    //     print!("{}", title);

    //     if let Some(file_info) = file_info {
    //         print!(
    //             " ({} found in {})",
    //             file_info.sub_mods.len(),
    //             match file_info.source {
    //                 ModFileInfoSource::McModInfo => "mcmod.info",
    //                 ModFileInfoSource::ModsToml => "mods.toml",
    //             }
    //         );
    //     }

    //     if let Some(curse_info) = curse_info {
    //         if full {
    //             print!(
    //                 "\n  Curseforge ids:\n    project: {}\n    file: {}",
    //                 curse_info.project_id, curse_info.id
    //             );
    //         } else {
    //             print!(" (curseforge: {}/{})", curse_info.project_id, curse_info.id)
    //         }
    //     }

    //     println!();

    //     if file_info.is_none() {
    //         println!("  WARNING: Missing file (consider using --repair to fix or remove the mod entirely).");
    //     }

    //     if let Some(file_info) = file_info {
    //         let all_versions: Vec<Option<&String>> = file_info
    //             .sub_mods
    //             .iter()
    //             .map(|m| m.version.as_ref())
    //             .collect();
    //         let versions_same = util::is_all_same(&all_versions);
    //         if versions_same {
    //             if let Some(Some(versions)) = all_versions.get(0) {
    //                 println!("  version: {}", versions);
    //             }
    //         }

    //         for m in file_info.sub_mods.iter() {
    //             print!("- {}", m.mod_id);
    //             if let Some(name) = &m.name {
    //                 println!(" ({})", name);
    //             }
    //             if !versions_same {
    //                 if let Some(version) = &m.version {
    //                     println!("    version: {}", version);
    //                 }
    //             }
    //             if full {
    //                 if let Some(authors) = &m.authors {
    //                     println!("    author(s): {}", authors);
    //                 }
    //             }
    //         }
    //     }
    // }
}
