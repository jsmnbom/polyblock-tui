use crate::Instance;
use anyhow::{anyhow, Context};
use chrono::{DateTime, Utc};
use log::{debug, /*trace*/};
//use notify::{op::Op, raw_watcher, RawEvent, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    io::{BufReader, BufWriter},
    path::PathBuf,
    //process::{Child, Command},
    //sync::mpsc::channel,
};

use super::VersionManifestVersion;
use crate::util;

#[cfg(target_os = "linux")]
use which::which;

#[cfg(target_os = "windows")]
use winreg::{enums::HKEY_LOCAL_MACHINE, RegKey};

type Other = serde_json::Map<String, serde_json::Value>;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LauncherProfile {
    name: String,
    last_version_id: String,
    last_used: DateTime<Utc>,
    created: DateTime<Utc>,
    r#type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    game_dir: Option<String>,
    #[serde(flatten)]
    other: Other,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LauncherSettings {
    enable_historical: bool,
    enable_snapshots: bool,
    enable_releases: bool,
    profile_sorting: String,
    #[serde(flatten)]
    other: Other,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LauncherConfig {
    profiles: HashMap<String, LauncherProfile>,
    settings: LauncherSettings,
    #[serde(flatten)]
    other: Other,
}

#[derive(Clone)]
pub struct Launcher {
    work_directory: PathBuf,
    pub libraries_directory: PathBuf,
    pub versions_directory: PathBuf,
    cache_directory: PathBuf,
    launcher_profiles_path: PathBuf,
    launcher_exec: PathBuf,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct LauncherVersion {
    downloads: HashMap<String, LauncherVersionDownload>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct LauncherVersionDownload {
    sha1: String,
    url: String,
}

impl VersionManifestVersion {
    pub fn launcher_version_json<P: Into<PathBuf>>(&self, versions_directory: P) -> PathBuf {
        versions_directory
            .into()
            .join(&self.id)
            .join(format!("{}.json", self.id))
    }
    pub fn launcher_version_jar<P: Into<PathBuf>>(&self, versions_directory: P) -> PathBuf {
        versions_directory
            .into()
            .join(&self.id)
            .join(format!("{}.jar", self.id))
    }
}

// TODO: Find launcher path on mac
#[cfg(target_os = "linux")]
pub fn default_launcher_exec() -> ::anyhow::Result<PathBuf> {
    Ok(which("minecraft-launcher")
        .context("Could not find 'minecraft-launcher' in path")?
        .to_path_buf())
}

#[cfg(target_os = "windows")]
pub fn default_launcher_exec() -> ::anyhow::Result<PathBuf> {
    let search = &[
        r"SOFTWARE\Mojang\InstalledProducts\Minecraft Launcher",
        r"SOFTWARE\WOW6432Node\Mojang\InstalledProducts\Minecraft Launcher",
    ];

    debug!("Search registry for minecraft launcher.");

    let install_locations: Vec<String> = search
        .iter()
        .map(|s| {
            RegKey::predef(HKEY_LOCAL_MACHINE)
                .open_subkey(s)
                .ok()
                .map(|key| key.get_value("InstallLocation").ok())
                .flatten()
        })
        .flatten()
        .collect();

    debug!("Found: {:?} using first one.", install_locations);

    let path = PathBuf::from(install_locations.get(0).context("Minecraft launcher not installed! You need to install the minecraft launcher to use polyblock. If you have it installed in another location than default use --launcher to point to it.")?);

    Ok(path.join("MinecraftLauncher.exe"))
}

impl Launcher {
    pub fn new<P: Into<PathBuf>>(
        work_directory: P,
        cache_directory: P,
        launcher: Option<P>,
    ) -> ::anyhow::Result<Self> {
        let launcher_exec = match launcher {
            Some(path) => path.into(),
            None => default_launcher_exec()?,
        };

        debug!("Using launcher executable: {:?}", launcher_exec);

        let work_directory = work_directory.into();

        let launcher_profiles_path = work_directory.join("launcher_profiles.json");

        Ok(Self {
            launcher_profiles_path,
            launcher_exec: launcher_exec,
            cache_directory: cache_directory.into(),
            libraries_directory: work_directory.join("libraries"),
            versions_directory: work_directory.join("versions"),
            work_directory: work_directory,
        })
    }

    // pub fn launch(&self) -> ::anyhow::Result<Child> {
    //     Ok(Command::new(&self.launcher_exec)
    //         .current_dir(&self.cache_directory)
    //         .arg("--workDir")
    //         .arg(&self.work_directory)
    //         .spawn()
    //         .context("Could not launch minecraft launcher. Is the launcher executable path set correctly?")?)
    // }

    // pub fn launch_instance(&self, instance: &Instance) -> ::anyhow::Result<()> {
    //     self.ensure_profile(instance)?;
    //     let _ = self.launch()?;

    //     Ok(())
    // }

    // fn wait_for_init(&self) -> ::anyhow::Result<()> {
    //     debug!("Attaching a watcher on {:?}", &self.work_directory);
    //     let (tx, rx) = channel();
    //     let mut watcher = raw_watcher(tx).context("Failed to set up watcher.")?;
    //     watcher
    //         .watch(&self.work_directory, RecursiveMode::Recursive)
    //         .context("Failed to start watcher.")?;

    //     let versions_directory = self.work_directory.join("versions");

    //     loop {
    //         match rx.recv() {
    //             Ok(RawEvent {
    //                 path: Some(path),
    //                 op: Ok(op),
    //                 cookie: _,
    //             }) => {
    //                 trace!("Watcher: {:?} {:?}", path, op);

    //                 if path == versions_directory && op == Op::CREATE {
    //                     debug!("Found version directory. Detaching watcher.");
    //                     watcher
    //                         .unwatch(&self.work_directory)
    //                         .context("Failed to stop watcher.")?;
    //                     break;
    //                 }
    //             }
    //             Ok(_event) => panic!("Broken event received by watcher!"),
    //             Err(e) => Err(e).context("Failed to receive watch event.")?,
    //         }
    //     }

    //     Ok(())
    // }

    fn read(&self) -> ::anyhow::Result<LauncherConfig> {
        debug!("Reading launcher profiles.");
        let file = fs::File::open(&self.launcher_profiles_path)
            .context("Failed to open launcher profiles file.")?;
        let reader = BufReader::new(file);
        Ok(serde_json::from_reader(reader).context("Failed to read launcher profiles.")?)
    }

    fn write(&self, config: LauncherConfig) -> ::anyhow::Result<()> {
        debug!("Writing launcher profiles.");
        let file = fs::File::create(&self.launcher_profiles_path)
            .context("Failed to create launcher profiles file.")?;
        let writer = BufWriter::new(file);
        Ok(serde_json::to_writer_pretty(writer, &config)
            .context("Failed to write launcher profiles.")?)
    }

    pub fn ensure_profile(&self, instance: &Instance) -> ::anyhow::Result<()> {
        debug!("Ensuring launcher profile for {:?}", instance);
        //TODO: WHat if the launcher profiles file wasn't created yet?
        let mut config = self.read()?;

        let uuid_key = instance.uuid.to_simple().to_string();

        config
            .profiles
            .entry(uuid_key)
            .and_modify(|profile| {
                profile.name = instance.name.clone();
                profile.last_version_id = instance.full_version_id();
                profile.last_used = Utc::now();
            })
            .or_insert(LauncherProfile {
                name: instance.name.clone(),
                last_version_id: instance.full_version_id(),
                last_used: Utc::now(),
                r#type: "custom".to_string(),
                created: Utc::now(),
                game_dir: Some(instance.directory().to_str().unwrap().to_owned()),
                other: Default::default(),
            });

        self.write(config)?;
        Ok(())
    }

    pub fn remove_profile(&self, instance: &Instance) -> ::anyhow::Result<()> {
        debug!("Removing launcher profile (instance: {:?})", instance);
        let mut config = self.read()?;

        let uuid_key = instance.uuid.to_simple().to_string();

        config.profiles.remove(&uuid_key);

        self.write(config)?;
        Ok(())
    }

    pub async fn download_version(
        &self,
        pb: &util::Progress,
        version: &VersionManifestVersion,
    ) -> ::anyhow::Result<()> {
        let version_json = self.download_version_json(pb, version).await?;
        self.download_version_jar(pb, version, version_json).await?;
        Ok(())
    }

    async fn download_version_json(
        &self,
        pb: &util::Progress,
        version: &VersionManifestVersion,
    ) -> ::anyhow::Result<LauncherVersion> {
        let version_json_path = version.launcher_version_json(&self.versions_directory);

        pb.reset().await;
        pb.set_msg("Downloading version json.").await;

        if !version_json_path.exists() {
            debug!(
                "Version json for version {} missing. Downloading...",
                version.id
            );
            util::download_file_with_progress(pb, &version.url, &version_json_path).await?;
        }

        let version_json_file = fs::File::open(&version_json_path)?;
        let version_json_reader = BufReader::new(version_json_file);
        Ok(serde_json::from_reader(version_json_reader)?)
    }

    async fn download_version_jar(
        &self,
        pb: &util::Progress,
        version: &VersionManifestVersion,
        version_json: LauncherVersion,
    ) -> ::anyhow::Result<()> {
        let version_jar_path = version.launcher_version_jar(&self.versions_directory);
        let download = version_json.downloads.get("client").unwrap();

        pb.reset().await;
        pb.set_msg("Downloading version jar.").await;

        if !version_jar_path.exists() {
            debug!(
                "Version jar for version {} missing. Downloading...",
                version.id
            );

            util::download_file_with_progress(pb, &download.url, &version_jar_path).await?;
        }

        pb.reset().await;
        pb.set_msg("Checking version jar.").await;

        let hash = util::sha1_file_with_progress(pb, &version_jar_path).await?;
        if hash != download.sha1 {
            return Err(anyhow!(
                "Version jar for version {} does not match its sha1.",
                version.id
            ));
        }

        Ok(())
    }
}
