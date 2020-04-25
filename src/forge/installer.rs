use anyhow::{anyhow, Context};
use futures::stream::StreamExt;
use log::{debug, trace, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ffi::OsString;
use std::fs::{create_dir_all, File};
use std::io::{BufWriter, Read};
use std::path::{Path, PathBuf};
use std::process::Command;

use super::manifest::{VersionManifestVersion, URL};
use crate::minecraft;
use crate::util;

type Other = serde_json::Map<String, serde_json::Value>;
type InstallProfileData = HashMap<String, InstallProfileDataPart>;

#[derive(Deserialize, Serialize, Debug, Clone)]
struct InstallProfileDataPart {
    client: String,
    server: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct InstallProfileProcessor {
    jar: String,
    classpath: Vec<String>,
    args: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
enum InstallProfileLibraryDownload {
    #[serde(rename = "artifact")]
    Artifact {
        path: String,
        url: String,
        sha1: Option<String>,
        size: u64,
    },
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct InstallProfileLibrary {
    name: String,
    downloads: InstallProfileLibraryDownload,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct InstallProfile {
    path: String,
    data: InstallProfileData,
    processors: Vec<InstallProfileProcessor>,
    libraries: Vec<InstallProfileLibrary>,
    #[serde(flatten)]
    other: Other,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct Manifest {
    name: String,
    forge_version: String,
    download_url: String,
    install_method: u64,
    filename: String,
    maven_version_string: String,
    version_json: String,
    libraries_install_location: String,
    install_profile_json: Option<String>,
    #[serde(flatten)]
    other: Other,
}

pub async fn install<P: AsRef<Path>>(
    main_pb: &util::Progress,
    sub_pb: &util::Progress,
    minecraft_version: &minecraft::VersionManifestVersion,
    version: VersionManifestVersion,
    forge_version_manifests_cache_directory: P,
    launcher: &minecraft::Launcher,
    java_home: Option<PathBuf>,
) -> ::anyhow::Result<()> {
    main_pb.inc_with_msg(1, "Locating java.").await;

    create_dir_all(&forge_version_manifests_cache_directory)
        .context("Failed to create forge version manifests cache directory!")?;

    // Find java executable
    let java_exec = util::java::find_exec(java_home)?;

    debug!("Using java at: {:?}", java_exec);

    main_pb
        .inc_with_msg(1, "Fetching forge manifest for version.")
        .await;

    let manifest = download_manifest(
        forge_version_manifests_cache_directory
            .as_ref()
            .join(format!("{}.json", &version.name)),
        &version,
    )
    .await?;

    if manifest.install_method != 3 {
        unimplemented!("Forge installation method other than 3 is unimplemented!");
    }

    let install_profile: InstallProfile =
        serde_json::from_str(&manifest.install_profile_json.unwrap()).with_context(|| {
            format!(
                "Failed to read forge manifest install profile for version {}",
                version.name
            )
        })?;

    main_pb
        .inc_with_msg(1, "Downloading minecraft version.")
        .await;
    launcher.download_version(sub_pb, minecraft_version).await?;
    sub_pb.reset().await;

    main_pb.inc_with_msg(1, "Writing forge version file.").await;
    write_version_file(
        minecraft_version.id.clone(),
        manifest.name,
        manifest.version_json,
        launcher.versions_directory.clone(),
    )?;

    main_pb.inc_with_msg(1, "Downloading libraries.").await;
    download_libraries(
        sub_pb,
        install_profile.libraries,
        launcher.libraries_directory.clone(),
    )
    .await?;
    sub_pb.reset().await;

    let forge_jar_maven: String = install_profile.data["PATCHED"].client.clone();
    let forge_jar_maven = &forge_jar_maven[1..forge_jar_maven.len() - 1];

    let forge_jar = launcher
        .libraries_directory
        .join(util::java::parse_maven(forge_jar_maven));

    let forge_jar_sha1: String = install_profile.data["PATCHED_SHA"].client.clone();
    let forge_jar_sha1 = &forge_jar_sha1[1..forge_jar_sha1.len() - 1];

    main_pb.inc_with_msg(1, "Installing forge.").await;

    if forge_jar.exists() {
        debug!("Patched forge already exists.");
        main_pb.inc_with_msg(1, "Checking forge.").await;
        let hash = util::sha1_file(&forge_jar)?;
        if hash == forge_jar_sha1 {
            debug!("Patched forge checksum ok.");
            return Ok(());
        } else {
            debug!("Patched forge checksum did not match.");
        }
    }

    debug!("Running forge install processors.");

    run_processors(
        sub_pb,
        java_exec,
        launcher.libraries_directory.clone(),
        install_profile.processors,
        install_profile.data,
        minecraft_version.launcher_version_jar(&launcher.versions_directory),
        launcher
            .libraries_directory
            .join(util::java::parse_maven(install_profile.path)),
    )
    .await?;
    sub_pb.reset().await;

    main_pb.inc_with_msg(1, "Checking forge.").await;
    let hash = util::sha1_file_with_progress(sub_pb, &forge_jar).await?;
    if hash != forge_jar_sha1 {
        return Err(anyhow!("Patched forge jar does not match checksum!"));
    }

    Ok(())
}

async fn download_manifest(
    manifest_path: PathBuf,
    version: &VersionManifestVersion,
) -> ::anyhow::Result<Manifest> {
    if !manifest_path.exists() {
        util::download_file(&format!("{}/{}", URL, version.name), &manifest_path).await?;
    }

    let mut manifest_file = File::open(&manifest_path)?;
    let mut manifest_str = String::new();
    manifest_file.read_to_string(&mut manifest_str)?;
    let manifest: Manifest = serde_json::from_str(&manifest_str)
        .with_context(|| format!("Failed to read forge manifest for version {}", version.name))?;

    Ok(manifest)
}

async fn download_libraries(
    pb: &util::Progress,
    libraries: Vec<InstallProfileLibrary>,
    libraries_directory: PathBuf,
) -> ::anyhow::Result<()> {
    pb.set_length(libraries.len() as u64).await;

    let results: Vec<::anyhow::Result<()>> =
        futures::stream::iter(libraries.into_iter().map(|library| {
            let name = library.name.clone();
            match library.downloads {
                InstallProfileLibraryDownload::Artifact {
                    path, url, sha1, ..
                } => {
                    let library_path = libraries_directory.join(path);
                    async move {
                        let r = download_library(name, library_path, url, sha1).await;
                        pb.inc(1).await;
                        r
                    }
                }
            }
        }))
        .buffer_unordered(8)
        .collect::<Vec<_>>()
        .await;

    results.into_iter().collect::<::anyhow::Result<_>>()
}

async fn download_library(
    name: String,
    path: PathBuf,
    url: String,
    sha1: Option<String>,
) -> ::anyhow::Result<()> {
    debug!("Downloading library {}", name);

    create_dir_all(&path.parent().unwrap())
        .with_context(|| format!("Failed to create libraries directory for {}!", name))?;

    if path.exists() {
        match sha1.clone() {
            Some(sha1) => {
                let hash = util::sha1_file(&path)
                    .with_context(|| format!("Failed to checksum library {}", name))?;
                if hash == sha1 {
                    debug!("{} exists - checksum ok.", name);
                    return Ok(());
                } else {
                    warn!(
                        "{} exists - checksum doesn't match. {} != {}",
                        name, hash, sha1
                    );
                }
            }
            None => {
                debug!("{} exists - has no checksum.", name);
                return Ok(());
            }
        }
    }

    if url == "" {
        warn!("Skipping download of {} as it has no URL.", name);
        return Ok(());
    }

    util::download_file(&url, &path).await?;

    match sha1 {
        Some(sha1) => {
            let hash = util::sha1_file(&path)
                .with_context(|| format!("Failed to checksum library {}", name))?;
            if hash == sha1 {
                debug!("{} downloaded - checksum ok.", name);
                return Ok(());
            } else {
                warn!(
                    "{} downloaded - checksum doesn't match. {} != {}",
                    name, hash, sha1
                );
            }
        }
        None => {
            debug!("{} downloaded - has no checksum.", name);
            return Ok(());
        }
    }

    Ok(())
}

async fn run_processors(
    pb: &util::Progress,
    java_exec: PathBuf,
    libraries_directory: PathBuf,
    processors: Vec<InstallProfileProcessor>,
    data: InstallProfileData,
    version_jar: PathBuf,
    path: PathBuf,
) -> ::anyhow::Result<()> {
    let classpath_divider = OsString::from(if cfg!(windows) { ";" } else { ":" });

    pb.set_length(processors.len() as u64).await;

    for processor in processors {
        debug!("Running processor: {}", processor.jar);
        // pb.set_message(&format!("Running processor from {}", processor.jar));

        let classpaths: Vec<PathBuf> = processor
            .classpath
            .into_iter()
            .map(|cp| libraries_directory.join(util::java::parse_maven(cp)))
            .collect();

        let args: Vec<OsString> = processor
            .args
            .into_iter()
            // Transform {variables}
            .map(|arg| {
                if arg.starts_with("{") && arg.ends_with("}") {
                    let var_name = &arg[1..arg.len() - 1];
                    if var_name == "MINECRAFT_JAR" || var_name == "BINPATCH" {
                        return Ok(var_name.to_owned());
                    }
                    data.get(var_name)
                        .map(|var| var.client.clone())
                        .ok_or(anyhow!(
                            "Invalid variable in forge install profile: {}",
                            var_name
                        ))
                } else {
                    Ok(arg)
                }
            })
            .collect::<::anyhow::Result<Vec<String>>>()?
            .into_iter()
            // Transform paths (maven, minecraft jar and binpatch)
            .map(|arg| {
                if arg == "MINECRAFT_JAR" {
                    version_jar.clone().into_os_string()
                } else if arg == "BINPATCH" {
                    let mut path = path.clone();
                    path.set_file_name(
                        path.file_name()
                            .unwrap()
                            .to_str()
                            .unwrap()
                            .replace(".jar", "-clientdata.lzma"),
                    );
                    path.into_os_string()
                } else if arg.starts_with("[") && arg.ends_with("]") {
                    let maven = &arg[1..arg.len() - 1];
                    libraries_directory
                        .join(util::java::parse_maven(maven))
                        .into_os_string()
                } else {
                    OsString::from(arg)
                }
            })
            .collect();

        trace!("Args are: {:?}", args);

        let jar = libraries_directory.join(util::java::parse_maven(processor.jar));

        let mut classpath = OsString::new();
        classpath.push(jar.clone().into_os_string());
        classpath.push(classpath_divider.clone());
        for i in 0..classpaths.len() {
            classpath.push(classpaths[i].clone().into_os_string());
            if i != classpaths.len() - 1 {
                classpath.push(classpath_divider.clone());
            }
        }

        trace!("Classpath is {:?}", classpath);

        let main_class = util::java::main_class(&jar)?;

        trace!("Main class is {}", main_class);

        let output = Command::new(java_exec.clone())
            .arg("-cp")
            .arg(classpath)
            .arg(main_class)
            .args(args)
            .output()?;

        trace!("Stdout: {}", String::from_utf8(output.stdout)?);
        trace!("Stderr: {}", String::from_utf8(output.stderr)?);

        if !output.status.success() {
            return Err(anyhow!("Processor failed to execute."));
        }

        pb.inc(1).await;
    }

    Ok(())
}

fn write_version_file(
    minecraft_version_id: String,
    forge_name: String,
    json: String,
    versions_directory: PathBuf,
) -> ::anyhow::Result<()> {
    let name = format!("{}-{}", minecraft_version_id, forge_name);
    let path = versions_directory
        .join(&name)
        .join(format!("{}.json", &name));
    if path.exists() {
        debug!("Version file exists.");
    } else {
        let mut data: HashMap<String, serde_json::Value> = serde_json::from_str(&json)?;
        data.remove("jar");
        data.insert(String::from("id"), serde_json::Value::String(name));

        create_dir_all(&path.parent().unwrap())?;
        debug!("Writing version file to {:?}", path);
        let file = File::create(&path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer(writer, &data)?;
    }

    Ok(())
}
