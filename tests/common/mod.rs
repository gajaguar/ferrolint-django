//! Shared by the integration tests: the built plugin, laid out as pip would.

#![allow(dead_code, clippy::unwrap_used, clippy::expect_used)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub fn ferrolint() -> PathBuf {
    if let Some(path) = std::env::var_os("FERROLINT") {
        return path.into();
    }
    let target = root().join("../ferrolint/target");
    ["release", "debug"]
        .iter()
        .map(|profile| target.join(profile).join("ferrolint"))
        .find(|path| path.exists())
        .expect("run `make ferrolint-bin` or set FERROLINT")
}

/// The component, built once for the whole test run.
pub fn plugin() -> Vec<u8> {
    let status = Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".into()))
        .args(["build", "--locked", "--release", "-j", "5"])
        .args(["--target", "wasm32-wasip2"])
        .current_dir(root())
        .status()
        .expect("cargo runs");
    assert!(status.success());
    fs::read(root().join("target/wasm32-wasip2/release/ferrolint_django.wasm")).expect("wasm")
}

/// A virtualenv whose `site-packages` holds the plugin as pip lays it out.
pub fn venv(dir: &Path, wasm: &[u8]) {
    let site = dir.join("venv/lib/python3.12/site-packages");
    let info = site.join("ferrolint_django-0.1.0.dist-info");
    fs::create_dir_all(&info).unwrap();
    fs::create_dir_all(site.join("ferrolint_django")).unwrap();
    fs::write(
        info.join("entry_points.txt"),
        "[ferrolint.plugins]\npylint_django = ferrolint_django\n\
         pylint_django.checkers.migrations = ferrolint_django\n",
    )
    .unwrap();
    fs::write(site.join("ferrolint_django/plugin.wasm"), wasm).unwrap();
}
