mod launcher;
mod manifest;

pub use launcher::{default_launcher_exec, Launcher};
pub use manifest::{VersionManifest, VersionManifestVersion, VersionManifestVersionType};

pub fn version_ident(version: &VersionManifestVersion) -> &'static str {
    if version.id == "3D Shareware v1.34" {
        "April Fools' joke"
    } else if version.id == "20w14infinite" {
        "April Fools' joke"
    } else if version.id.starts_with("rd") {
        "Pre-classic"
    } else if version.id.starts_with("c") {
        "Classic"
    } else if version.id.starts_with("inf") {
        "Infdev"
    } else if version.id.starts_with("a") {
        "Alpha"
    } else if version.id.starts_with("b") {
        "Beta"
    } else if version.r#type == VersionManifestVersionType::Snapshot {
        "Snapshot"
    } else if version.r#type == VersionManifestVersionType::Release {
        "Release"
    } else {
        ""
    }
}
