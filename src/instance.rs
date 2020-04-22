use ::anyhow::{anyhow, Context};
use array_tool::vec::Intersect;
use log::{debug, trace, warn};
use serde::{Deserialize, Serialize};
use std::{
    collections::{hash_map, HashMap, HashSet},
    fs,
    io::{BufReader, BufWriter},
    path::PathBuf,
};
use uuid::Uuid;

use crate::mods::{ModFileInfo, ModInfo};

#[derive(Debug, Clone, Default)]
pub struct Instance {
    pub name: String,
    pub version_id: String,
    pub forge_name: Option<String>,
    pub uuid: Uuid,
    pub mods: HashMap<PathBuf, ModInfo>,
    pub instances_directory: PathBuf,
}

impl Instance {
    fn from_file_instance<P: Into<PathBuf>>(
        file_instance: FileInstance,
        instances_directory: P,
    ) -> Self {
        Self {
            instances_directory: instances_directory.into(),
            name: file_instance.name,
            version_id: file_instance.version_id,
            forge_name: file_instance.forge_name,
            uuid: file_instance.uuid,
            mods: file_instance.mods,
        }
    }

    fn to_file_instance(self) -> FileInstance {
        FileInstance {
            name: self.name,
            version_id: self.version_id,
            forge_name: self.forge_name,
            uuid: self.uuid,
            mods: self.mods,
        }
    }

    pub fn full_version_id(&self) -> String {
        match &self.forge_name {
            Some(forge_name) => format!("{}-{}", self.version_id, forge_name),
            None => self.version_id.clone(),
        }
    }

    pub fn directory(&self) -> PathBuf {
        self.instances_directory
            .join(self.uuid.to_hyphenated().to_string())
    }

    pub fn mods_directory(&self) -> PathBuf {
        self.directory().join("mods")
    }

    pub fn update_mod_file_info(&mut self) -> ::anyhow::Result<()> {
        // TODO: recurse into directories (but do not follow symlinks -- that might be bad)
        // TODO: Lots of room for optimization here i'd think
        let mods_dir = self.mods_directory();

        let mut mod_paths: HashSet<PathBuf> = HashSet::new();
        for entry in fs::read_dir(&mods_dir)? {
            let entry = entry?;
            let path = entry.path();
            let file_path = path.strip_prefix(&mods_dir)?.to_path_buf();
            mod_paths.insert(file_path);
        }

        for key in self.mods.keys() {
            mod_paths.insert(key.clone());
        }

        for path in mod_paths.into_iter() {
            let full_path = mods_dir.join(&path);
            match self.mods.entry(path) {
                hash_map::Entry::Occupied(mut entry) => {
                    let mut info = entry.get_mut();
                    if full_path.is_file() {
                        if info.file.is_none() {
                            let file_info = ModFileInfo::from_file(&full_path)?;
                            info.file = Some(file_info);
                        }
                    } else {
                        warn!("{:?} was missing - removing its file data.", full_path);
                        if info.curse.is_none() {
                            entry.remove_entry();
                        } else {
                            info.file = None;
                        }
                    }
                }
                hash_map::Entry::Vacant(entry) => {
                    let file_info = ModFileInfo::from_file(&full_path)?;
                    entry.insert(ModInfo {
                        file: Some(file_info),
                        curse: None,
                    });
                }
            }
        }

        Ok(())
    }

    pub fn conflicting_mods(&self, other: &ModInfo) -> Vec<(&PathBuf, &ModInfo)> {
        let existing_mod_ids: Vec<&String> = self
            .mods
            .iter()
            .filter_map(|(_, m)| {
                m.file.as_ref().map(|m_file| {
                    m_file
                        .sub_mods
                        .iter()
                        .map(|m| &m.mod_id)
                        .collect::<Vec<&String>>()
                })
            })
            .flatten()
            .collect();

        let intersection = existing_mod_ids.intersect(
            other
                .file
                .as_ref()
                .unwrap()
                .sub_mods
                .iter()
                .map(|m| &m.mod_id)
                .collect::<Vec<&String>>(),
        );

        if intersection.len() > 0 {
            trace!("intersection: {:?}", intersection);

            self.mods_by_mod_ids(intersection)
        } else {
            Vec::new()
        }
    }

    pub fn mods_by_mod_ids<S: AsRef<str>>(&self, mod_ids: Vec<S>) -> Vec<(&PathBuf, &ModInfo)> {
        let mod_ids: Vec<&str> = mod_ids.iter().map(|s| s.as_ref()).collect();
        self.mods
            .iter()
            .filter_map(|(k, m)| {
                if let Some(m_file) = m.file.as_ref() {
                    if m_file
                        .sub_mods
                        .iter()
                        .any(|m| mod_ids.iter().any(|x| x == &&m.mod_id))
                    {
                        Some((k, m))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect()
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
struct FileInstance {
    pub name: String,
    pub version_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub forge_name: Option<String>,
    pub uuid: Uuid,
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub mods: HashMap<PathBuf, ModInfo>,
}

#[derive(Debug, Clone, Default)]
pub struct Instances {
    pub inner: HashMap<String, Instance>,
    path: PathBuf,
}

impl Instances {
    pub fn from_file<P: Into<PathBuf>>(path: P, instances_directory: P) -> ::anyhow::Result<Self> {
        let path = path.into();
        match fs::File::open(&path) {
            Ok(file) => {
                let reader = BufReader::new(file);
                let instances_directory = instances_directory.into();
                let mut data: HashMap<String, FileInstance> =
                    serde_json::from_reader(reader).context("Got malformed instance data.")?;
                let data = data
                    .drain()
                    .map(|(k, v)| (k, Instance::from_file_instance(v, &instances_directory)))
                    .collect();
                Ok(Self {
                    path: path,
                    inner: data,
                })
            }
            _ => {
                debug!("No config file found.");
                Ok(Self {
                    path: path,
                    ..Default::default()
                })
            }
        }
    }

    pub fn save(&self) -> ::anyhow::Result<()> {
        let writer = BufWriter::new(
            fs::File::create(&self.path).context("Could not create instance file.")?,
        );
        let data: HashMap<String, FileInstance> = self
            .inner
            .clone()
            .drain()
            .map(|(k, v)| (k, v.to_file_instance()))
            .collect();
        serde_json::to_writer(writer, &data).context("Tried to save malformed instance data.")?;
        Ok(())
    }

    pub fn find_instance_by_name(&self, instance_name: &str) -> ::anyhow::Result<&Instance> {
        Ok(self
            .inner
            .get(instance_name)
            .ok_or(anyhow!("Could not find instance with that name."))?)
    }

    pub fn find_instance_by_name_mut(
        &mut self,
        instance_name: &str,
    ) -> ::anyhow::Result<&mut Instance> {
        Ok(self
            .inner
            .get_mut(instance_name)
            .ok_or(anyhow!("Could not find instance with that name."))?)
    }
}
