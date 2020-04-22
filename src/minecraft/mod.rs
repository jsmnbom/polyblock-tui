mod launcher;
mod manifest;

pub use launcher::{default_launcher_exec, Launcher};
pub use manifest::{VersionManifest, VersionManifestVersion, VersionManifestVersionType};

pub fn version_ident(version: &VersionManifestVersion) -> &'static str {
    if version.id == "3D Shareware v1.34" {
        "(April Fools' joke)"
    } else if version.id == "20w14infinite" {
        "(April Fools' joke)"
    } else if version.id.starts_with("rd") {
        "(pre-classic)"
    } else if version.id.starts_with("c") {
        "(classic)"
    } else if version.id.starts_with("inf") {
        "(infdev)"
    } else if version.id.starts_with("a") {
        "(alpha)"
    } else if version.id.starts_with("b") {
        "(beta)"
    } else if version.r#type == VersionManifestVersionType::Snapshot {
        "(snapshot)"
    } else if version.r#type == VersionManifestVersionType::Release {
        "(release)"
    } else {
        ""
    }
}
