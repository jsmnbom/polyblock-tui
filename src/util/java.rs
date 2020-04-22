use ::anyhow::{anyhow, Context};
use std::{
    collections::HashMap,
    io::Read,
    path::{Path, PathBuf},
};
use which::which;

#[cfg(target_os = "windows")]
use crate::minecraft;
#[cfg(target_os = "windows")]
use log::debug;
#[cfg(target_os = "windows")]
use std::fs::read_dir;
#[cfg(target_os = "windows")]
use winreg::{enums::HKEY_LOCAL_MACHINE, RegKey};

pub type Manifest = HashMap<String, String>;

fn find_home_in_path() -> ::anyhow::Result<PathBuf> {
    let java = which("java")
        .context("Failed to find 'java' in path")?
        .canonicalize()?;
    Ok(java.parent().unwrap().parent().unwrap().to_path_buf())
}

#[cfg(target_os = "linux")]
fn find_home() -> ::anyhow::Result<PathBuf> {
    return find_home_in_path();
}

#[cfg(target_os = "windows")]
fn find_launcher_runtime() -> ::anyhow::Result<PathBuf> {
    let launcher_exec = minecraft::default_launcher_exec()?;
    let launcher_path = launcher_exec.parent().unwrap();
    let runtime_path = launcher_path.join("runtime");
    if !runtime_path.is_dir() {
        return Err(anyhow!("Runtime dir non-existant."));
    }
    for entry in read_dir(runtime_path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let bin = path.join("bin");
            let java = bin.join("java.exe");
            if java.exists() {
                return Ok(path);
            }
        }
    }
    Err(anyhow!("No appropriate runtime found."))
}

#[cfg(target_os = "windows")]
fn find_home() -> ::anyhow::Result<PathBuf> {
    debug!("Trying to locate minecraft launcher runtime.");
    let launcher_runtime = find_launcher_runtime();
    debug!("Launcher runtime: {:?}", launcher_runtime);
    if let Some(launcher_runtime) = launcher_runtime.ok() {
        return Ok(launcher_runtime);
    }

    let search = &[
        r"SOFTWARE\JavaSoft\JDK",
        r"SOFTWARE\JavaSoft\Java Development Kit",
        r"SOFTWARE\JavaSoft\Java Runtime Environment",
    ];

    let homes: Vec<String> = search
        .iter()
        .map(|s| {
            RegKey::predef(HKEY_LOCAL_MACHINE)
                .open_subkey(s)
                .ok()
                .map(|key| {
                    key.get_value("CurrentVersion")
                        .ok()
                        .map(|current_version: String| {
                            key.open_subkey(current_version)
                                .ok()
                                .map(|version_key| version_key.get_value("JavaHome").ok())
                        })
                        .flatten()
                })
                .flatten()
        })
        .flatten()
        .flatten()
        .collect();

    let home = homes.get(0).ok_or(anyhow!("No home found."))?;

    Ok(PathBuf::from(home))
}

pub fn find_exec<P: Into<PathBuf>>(home: Option<P>) -> ::anyhow::Result<PathBuf> {
    let home = match home {
        Some(home) => home.into(),
        None => find_home().context("Could not find java home. Please make sure java is installed. If you're sure it's installed, set JAVA_HOME env var to point to it. On windows try playing a game with the minecraft launcher to automatically install java.")?
    };

    let path = home
        .join("bin")
        .join(if cfg!(windows) { "java.exe" } else { "java" });

    if path.is_file() {
        Ok(path)
    } else {
        Err(anyhow!(
            "Java executable at {:?} doesn't exist or isn't a file.",
            path
        ))
    }
}

pub fn main_class<P: AsRef<Path>>(jar_path: P) -> ::anyhow::Result<String> {
    let jar = std::fs::File::open(jar_path.as_ref())?;

    let mut archive = zip::ZipArchive::new(jar)?;

    let mut file = archive.by_name("META-INF/MANIFEST.MF").with_context(|| {
        format!(
            "Failed to find meta-inf/manifest.mf in {:?}",
            jar_path.as_ref()
        )
    })?;

    let mut manifest_str = String::new();
    file.read_to_string(&mut manifest_str)?;

    let manifest = parse_manifest(&manifest_str);

    let main_class = manifest
        .get("Main-Class")
        .ok_or(anyhow!("Failed to find main class."))?;

    Ok(main_class.to_owned())
}

pub fn parse_manifest(manifest_str: &str) -> Manifest {
    manifest_str
        .lines()
        .filter_map(|line| {
            let mut parts = line.splitn(2, ":");
            match (parts.next(), parts.next()) {
                (Some(s1), Some(s2)) => Some((s1.trim().to_owned(), s2.trim().to_owned())),
                _ => None,
            }
        })
        .collect()
}

/// Transform from maven string to a path
///
/// de.oceanlabs.mcp:mcp_config:1.14.4-20190829.143755:mappings@txt
/// turns into
/// de/oceanlabs/mcp/mcp_config/1.14.4-20190829.143755/mcp_config-1.14.4-20190829.143755-mappings.txt
///
/// net.sf.jopt-simple:jopt-simple:5.0.4
/// turns into
/// net/sf/jopt-simple/jopt-simple/5.0.4/jopt-simple-5.0.4.jar
pub fn parse_maven<S: Into<String>>(name: S) -> PathBuf {
    let name = name.into();

    let ext_parts: Vec<&str> = name.split("@").collect();
    let path_parts: Vec<&str> = ext_parts[0].split(":").collect();
    let file = path_parts[1];
    let version = path_parts[2];
    let extra = match path_parts.get(3) {
        Some(extra) => format!("-{}", extra),
        None => String::new(),
    };
    let ext = *ext_parts.get(1).unwrap_or(&"jar");

    PathBuf::from(path_parts[0].replace(".", "/")) // TODO: USE windows path separators
        .join(file)
        .join(version)
        .join(format!("{}-{}{}.{}", file, version, extra, ext))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_manifest() {
        let manifest = parse_manifest(
            "Manifest-Version: 1.0
Implementation-Title: test-0.4
Main-Class: test.test.testclass",
        );
        assert_eq!(manifest["Manifest-Version"], "1.0");
        assert_eq!(manifest["Implementation-Title"], "test-0.4");
        assert_eq!(manifest["Main-Class"], "test.test.testclass");
    }

    #[test]
    fn test_parse_maven_with_ext_and_extra() {
        let path = parse_maven("de.oceanlabs.mcp:mcp_config:1.14.4-20190829.143755:mappings@txt");
        assert_eq!(path, PathBuf::from("de/oceanlabs/mcp/mcp_config/1.14.4-20190829.143755/mcp_config-1.14.4-20190829.143755-mappings.txt"));
    }

    #[test]
    fn test_parse_maven() {
        let path = parse_maven("net.sf.jopt-simple:jopt-simple:5.0.4");
        assert_eq!(
            path,
            PathBuf::from("net/sf/jopt-simple/jopt-simple/5.0.4/jopt-simple-5.0.4.jar")
        );
    }
    #[test]
    fn test_parse_maven_with_ext() {
        let path = parse_maven("de.oceanlabs.mcp:mcp_config:1.14.4-20190829.143755@zip");
        assert_eq!(
            path,
            PathBuf::from("de/oceanlabs/mcp/mcp_config/1.14.4-20190829.143755/mcp_config-1.14.4-20190829.143755.zip")
        );
    }
}
