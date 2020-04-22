use ::anyhow::anyhow;
use directories::ProjectDirs;
use log::debug;
use std::path::PathBuf;

use crate::Opt;

#[derive(Debug, Clone)]
pub struct FilePaths {
    pub minecraft_versions_cache: PathBuf,
    pub forge_versions_cache: PathBuf,
    pub config: PathBuf,
    pub instances: PathBuf,
}

#[derive(Debug, Clone)]
pub struct DirectoryPaths {
    pub data: PathBuf,
    pub cache: PathBuf,
    pub instances: PathBuf,
    pub launcher_work: PathBuf,
    pub launcher_cache: PathBuf,
    pub forge_version_manifests_cache: PathBuf,
}

#[derive(Debug, Clone)]
pub struct Paths {
    pub file: FilePaths,
    pub directory: DirectoryPaths,
}

impl Paths {
    pub fn new(opt: &Opt) -> ::anyhow::Result<Self> {
        let (data_directory, cache_directory) = {
            let project_dirs = ProjectDirs::from("jsmnbom.github.io", "polyblock", "polyblock-rs");
            (
                match &opt.data_directory {
                    Some(dir) => dir.clone().canonicalize()?,
                    None => project_dirs.as_ref().ok_or(anyhow!("Data directory could not be automatically found. Please set with --data-directory!"))?.data_local_dir().to_path_buf()
                },
                match &opt.cache_directory {
                    Some(dir) => dir.clone().canonicalize()?,
                    None => project_dirs.as_ref().ok_or(anyhow!("Cache directory could not be automatically found. Please set with --cache-directory!"))?.cache_dir().to_path_buf()
                },
            )
        };

        let launcher_work_directory = data_directory.join(".minecraft");

        let directory_paths = DirectoryPaths {
            instances: data_directory.join("instances"),
            forge_version_manifests_cache: cache_directory.join("forge_version_manifests"),
            launcher_cache: cache_directory.join("launcher"),
            launcher_work: launcher_work_directory,
            cache: cache_directory,
            data: data_directory,
        };

        debug!("Using directories: {:?}", directory_paths);

        Ok(Self {
            file: FilePaths {
                minecraft_versions_cache: directory_paths.cache.join("versions.json"),
                forge_versions_cache: directory_paths.cache.join("forge.json"),
                instances: directory_paths.data.join("instances.json"),
                config: directory_paths.data.join("config.json"),
            },
            directory: directory_paths,
        })
    }
}
