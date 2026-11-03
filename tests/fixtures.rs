//! Runs the built plugin, through the ferrolint binary, over the fixtures and compares with their `.expected` files:
//! one `line:code:message` per finding of a `pylint_django` rule.
// The binary is `FERROLINT`, else the sibling checkout's build, else the
// `ferrolint` on PATH (`uv tool install ferrolint==1.1.0`).

// A helper that panics is how a test fails, as `clippy.toml` allows inside
// `#[test]` functions themselves.
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use std::fs;
use std::path::Path;
use std::process::Command;

use common::{ferrolint, plugin, root, venv};

/// The findings a fixture shows, as `line:code:message`.
fn findings(dir: &Path, fixture: &str) -> Vec<String> {
    let output = Command::new(ferrolint())
        .args([
            format!("{fixture}.py").as_str(),
            "--format",
            "json",
            "--no-cache",
        ])
        .current_dir(dir)
        .env("VIRTUAL_ENV", dir.join("venv"))
        .env("HOME", dir.join(".home"))
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let mut out: Vec<String> = json
        .as_array()
        .or_else(|| json["diagnostics"].as_array())
        .expect("a list of findings")
        .iter()
        .filter_map(|d| {
            let code = d["code"].as_str()?;
            let ours =
                code.len() == 5 && code.starts_with(['E', 'W', 'R']) && code[1..].starts_with('5')
                    || code == "R0904"
                    || code == "C0103";
            ours.then(|| {
                format!(
                    "{}:{code}:{}",
                    d["range"]["start"]["line"],
                    d["message"].as_str().unwrap()
                )
            })
        })
        .collect();
    out.sort();
    out
}

#[test]
fn every_fixture_gives_its_expected_findings() {
    let wasm = plugin();
    let fixtures = root().join("tests/fixtures");
    let mut names: Vec<String> = fs::read_dir(&fixtures)
        .unwrap()
        .flatten()
        .filter_map(|e| e.path().file_stem()?.to_str().map(str::to_owned))
        .filter(|name| fixtures.join(format!("{name}.expected")).exists())
        .collect();
    names.sort();
    names.dedup();
    assert!(names.len() >= 4, "{names:?}");
    let mut wrong = Vec::new();
    for name in names {
        let dir = tempfile::tempdir().unwrap();
        venv(dir.path(), &wasm);
        fs::create_dir(dir.path().join(".home")).unwrap();
        fs::copy(
            fixtures.join(format!("{name}.py")),
            dir.path().join(format!("{name}.py")),
        )
        .unwrap();
        fs::write(
            dir.path().join(".pylintrc"),
            "[MASTER]\nload-plugins=pylint_django,pylint_django.checkers.migrations\n\
             [DESIGN]\nmax-public-methods=2\n",
        )
        .unwrap();
        let mut expected: Vec<String> =
            fs::read_to_string(fixtures.join(format!("{name}.expected")))
                .unwrap()
                .lines()
                .map(str::to_owned)
                .collect();
        expected.sort();
        let actual = findings(dir.path(), &name);
        if actual != expected {
            wrong.push(format!(
                "{name}:\n  expected {expected:#?}\n  actual {actual:#?}"
            ));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}
