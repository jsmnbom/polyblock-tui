use anyhow::{anyhow, Context};
use futures::stream::StreamExt;
use log::{debug, trace, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ffi::OsString;
use std::fs::{create_dir_all, File};
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

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
    // steps: &mut progress::StepProgress,
    minecraft_version: &minecraft::VersionManifestVersion,
    version: VersionManifestVersion,
    forge_version_manifests_cache_directory: P,
    launcher: &minecraft::Launcher,
    java_home: Option<PathBuf>,
) -> ::anyhow::Result<()> {
    // steps.add(vec![
    //     "Locating java.",
    //     "Fetching forge manifest for version.",
    //     "Downloading minecraft version.",
    //     "Writing forge version file.",
    //     "Downloading libraries",
    //     "Installing forge",
    // ]);
    // steps.inc();

    create_dir_all(&forge_version_manifests_cache_directory)
        .context("Failed to create forge version manifests cache directory!")?;

    // Find java executable
    let java_exec = util::java::find_exec(java_home)?;

    debug!("Using java at: {:?}", java_exec);

    // steps.inc();

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

    // steps.inc();
    launcher.download_version(minecraft_version).await?;

    // steps.inc();
    write_version_file(
        minecraft_version.id.clone(),
        manifest.name,
        manifest.version_json,
        launcher.versions_directory.clone(),
    )?;

    // steps.inc();
    download_libraries(
        install_profile.libraries,
        launcher.libraries_directory.clone(),
    )
    .await?;

    let forge_jar_maven: String = install_profile.data["PATCHED"].client.clone();
    let forge_jar_maven = &forge_jar_maven[1..forge_jar_maven.len() - 1];

    let forge_jar = launcher
        .libraries_directory
        .join(util::java::parse_maven(forge_jar_maven));

    let forge_jar_sha1: String = install_profile.data["PATCHED_SHA"].client.clone();
    let forge_jar_sha1 = &forge_jar_sha1[1..forge_jar_sha1.len() - 1];

    print_forge_notice();

    // steps.inc();

    if forge_jar.exists() {
        debug!("Patched forge already exists.");
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
        java_exec,
        launcher.libraries_directory.clone(),
        install_profile.processors,
        install_profile.data,
        minecraft_version.launcher_version_jar(&launcher.versions_directory),
        launcher
            .libraries_directory
            .join(util::java::parse_maven(install_profile.path)),
    )?;

    let hash = util::sha1_file(&forge_jar)?;
    if hash != forge_jar_sha1 {
        return Err(anyhow!("Patched forge jar does not match checksum!"));
    }

    Ok(())
}

fn print_forge_notice() {
    println!(
        "Forge is an open source project that mostly relies on ad revenue.
By using Polyblock you bypass viewing these ads.
Please strongly consider supporting the creator of Forge LexManos' Patreon.
https://www.patreon.com/LexManos"
    );
}

async fn download_manifest(
    manifest_path: PathBuf,
    version: &VersionManifestVersion,
) -> ::anyhow::Result<Manifest> {
    // let pb = progress::create_bar("", None, Some("Checking"));

    if !manifest_path.exists() {
        util::download_file(
            // &pb,
            &format!("{}/{}", URL, version.name),
            &manifest_path,
        )
        .await?;
    }

    let manifest_file = File::open(&manifest_path)?;
    let manifest_reader = BufReader::new(manifest_file);
    let manifest: Manifest = serde_json::from_reader(manifest_reader)
        .with_context(|| format!("Failed to read forge manifest for version {}", version.name))?;

    // pb.finish_and_clear();

    Ok(manifest)
}

async fn download_libraries(
    libraries: Vec<InstallProfileLibrary>,
    libraries_directory: PathBuf,
) -> ::anyhow::Result<()> {
    // let dmp = Arc::new(progress::DynamicMultiProgress::new());

    let results: Vec<::anyhow::Result<()>> =
        futures::stream::iter(libraries.into_iter().map(|library| {
            let name = library.name.clone();
            match library.downloads {
                InstallProfileLibraryDownload::Artifact {
                    path, url, sha1, ..
                } => {
                    let library_path = libraries_directory.join(path);
                    // let dmp = dmp.clone();
                    async move {
                        // let pb = dmp.add_new(&name, None, Some("Checking"));
                        download_library(name, library_path, url, sha1).await
                    }
                }
            }
        }))
        .buffer_unordered(8)
        .collect::<Vec<_>>()
        .await;

    // dmp.finish();

    results.into_iter().collect::<::anyhow::Result<_>>()
}

async fn download_library(
    // pb: ProgressBar,
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

fn run_processors(
    java_exec: PathBuf,
    libraries_directory: PathBuf,
    processors: Vec<InstallProfileProcessor>,
    data: InstallProfileData,
    version_jar: PathBuf,
    path: PathBuf,
) -> ::anyhow::Result<()> {
    let classpath_divider = OsString::from(if cfg!(windows) { ";" } else { ":" });

    // let pb = progress::create_bar("", Some(processors.len() as u64), None);

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

        // pb.inc(1);
    }

    // pb.finish_and_clear();

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
