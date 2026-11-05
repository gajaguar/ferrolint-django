//! `W5198` reads only the latest migration of each app, as Pylint does with
//! `pylint_django.checkers.migrations` loaded (a run over the same tree).

// A helper that panics is how a test fails, as `clippy.toml` allows inside
// `#[test]` functions themselves.
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use std::fs;
use std::path::Path;
use std::process::Command;

fn migration(name: &str) -> String {
    format!(
        "from django.db import migrations, models\n\n\nclass Migration(migrations.Migration):\n    operations = [\n        migrations.AddField(model_name=\"m\", name=\"{name}\", field=models.IntegerField(default=1)),\n        migrations.AddField(model_name=\"m\", name=\"n\", field=models.IntegerField(null=True)),\n        AddField(model_name=\"m\", name=\"b\", field=models.IntegerField(default=1)),\n        migrations.AddField(model_name=\"m\", name=\"x\", field=field(default=1)),\n    ]\n"
    )
}

fn write(root: &Path, name: &str, text: &str) {
    let path = root.join(name);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

/// The `code` findings of a run over `.`, as `path:line:column`, sorted.
fn found(root: &Path, code: &str) -> Vec<String> {
    let output = Command::new(common::ferrolint())
        .args([".", "--no-cache", "--format", "json"])
        .current_dir(root)
        .env("VIRTUAL_ENV", root.join("venv"))
        .env("HOME", root.join(".home"))
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let mut out: Vec<String> = json["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|d| d["code"] == code)
        .map(|d| {
            format!(
                "{}:{}:{}",
                d["path"].as_str().unwrap().trim_start_matches("./"),
                d["range"]["start"]["line"],
                d["range"]["start"]["column"]
            )
        })
        .collect();
    out.sort();
    out
}

#[test]
fn new_field_defaults_are_read_in_the_latest_migration_only() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    common::venv(root, &common::plugin());
    fs::create_dir(root.join(".home")).unwrap();
    write(
        root,
        ".pylintrc",
        "[MASTER]\nload-plugins=pylint_django.checkers.migrations\n[MESSAGES CONTROL]\ndisable=all\nenable=W5198\n",
    );
    for file in [
        "app1/migrations/0001_a.py",
        "app1/migrations/0002_b.py",
        "app2/migrations/0001_c.py",
        "app3/sub/migrations/0004_e.py",
        "app3/sub/migrations/0005_d.py",
    ] {
        write(root, file, &migration(file));
        for dir in Path::new(file).parent().unwrap().ancestors() {
            if !dir.as_os_str().is_empty() {
                write(root, &format!("{}/__init__.py", dir.display()), "");
            }
        }
    }
    assert_eq!(
        found(root, "W5198"),
        [
            "app1/migrations/0002_b.py:6:9",
            "app1/migrations/0002_b.py:9:9",
            "app2/migrations/0001_c.py:6:9",
            "app2/migrations/0001_c.py:9:9",
            "app3/sub/migrations/0005_d.py:6:9",
            "app3/sub/migrations/0005_d.py:9:9",
        ]
    );
    // Without the sub-plugin the rule is silent.
    write(
        root,
        ".pylintrc",
        "[MESSAGES CONTROL]\ndisable=all\nenable=W5198\n",
    );
    assert_eq!(found(root, "W5198"), Vec::<String>::new());
}

#[test]
fn a_model_is_known_by_the_host_through_a_base_in_another_file() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    common::venv(root, &common::plugin());
    fs::create_dir(root.join(".home")).unwrap();
    write(
        root,
        ".pylintrc",
        "[MASTER]\nload-plugins=pylint_django\n[MESSAGES CONTROL]\ndisable=all\nenable=W5102\n",
    );
    // The host reads the MRO from source, so Django has to be in the project.
    for init in ["django", "django/db"] {
        write(root, &format!("{init}/__init__.py"), "");
    }
    write(
        root,
        "django/db/models/__init__.py",
        "class Model:\n    pass\n",
    );
    write(root, "app/__init__.py", "");
    write(
        root,
        "app/base.py",
        "from django.db import models\n\n\nclass Base(models.Model):\n    class Meta:\n        abstract = True\n",
    );
    write(
        root,
        "app/models.py",
        "from app.base import Base\n\n\nclass Pet(Base):\n    def __unicode__(self):\n        return \"pet\"\n",
    );
    // `Base` has no Django name: only the host's MRO says it is a model.
    assert_eq!(found(root, "W5102"), ["app/models.py:4:7"]);
}
