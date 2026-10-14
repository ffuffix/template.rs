use std::path::Path;
use std::process::{Command, Stdio};

use serde_json::Value;

/// The `cargo metadata` entry for the package whose manifest is `manifest`, or `None` when
/// `manifest` is a virtual workspace manifest or cargo can't read it.
pub(crate) fn package(manifest: &Path) -> Option<Value> {
    let output = Command::new("cargo")
        .args([
            "metadata",
            "--no-deps",
            "--format-version",
            "1",
            "--manifest-path",
        ])
        .arg(manifest)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()
        .filter(|output| output.status.success())?;
    let metadata: Value = serde_json::from_slice(&output.stdout).ok()?;
    let manifest = manifest.canonicalize().ok()?;
    metadata["packages"]
        .as_array()?
        .iter()
        .find(|package| {
            package["manifest_path"]
                .as_str()
                .and_then(|path| Path::new(path).canonicalize().ok())
                .is_some_and(|path| path == manifest)
        })
        .cloned()
}
