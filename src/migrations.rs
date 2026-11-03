//! The `pylint_django.checkers.migrations` rules: `W5197` per file and
//! `W5198` over the latest migration of each app.

use ferrolint_plugin_sdk::prelude::*;

use crate::tree::Tree;

pub const BACKWARDS: &str = "W5197";
pub const FIELD_DEFAULT: &str = "W5198";

/// Whether `load-plugins` names the migrations sub-plugin.
pub fn loaded(options: &[(String, String)]) -> bool {
    options
        .iter()
        .find(|(name, _)| name == "load-plugins")
        .is_some_and(|(_, list)| {
            list.split(',')
                .any(|name| name.trim() == "pylint_django.checkers.migrations")
        })
}

/// Pylint's `is_migrations_module`. shortcut: the whole path is searched for
/// `migrations`, as Pylint does with the path it was given, which may be
/// absolute here and relative there; upgrade if a project root is named so.
pub fn migrations_path(path: &str) -> bool {
    path.contains("migrations") && !path.ends_with("__init__.py")
}

/// The calls of a migration that the sub-plugin reads. Pylint takes a call
/// only when its frame (the nearest `def`, `class` or `lambda`) is a direct
/// child of the module.
fn frame_calls<'a>(tree: &Tree<'a>) -> Vec<&'a Node<'a>> {
    let frame = |n: &Node<'_>| {
        matches!(
            n.kind,
            NodeKind::FunctionDef | NodeKind::ClassDef | NodeKind::Lambda
        )
    };
    tree.nodes
        .iter()
        .filter(|call| call.kind == NodeKind::Call)
        .filter(|call| {
            tree.nodes
                .iter()
                .rfind(|n| frame(n) && n.id < call.id && n.last >= call.id)
                .is_some_and(|frame| frame.parent == Some(0))
        })
        .collect()
}

fn has_keyword(tree: &Tree<'_>, call: &Node<'_>, name: &str) -> bool {
    tree.keywords(call).iter().any(|k| k.name == Some(name))
}

/// `W5197`: a `RunPython` call with fewer than two positional arguments and
/// no `reverse_code`.
pub fn backwards(tree: &Tree<'_>, path: &str) -> Vec<Diagnostic> {
    if !migrations_path(path) {
        return Vec::new();
    }
    frame_calls(tree)
        .into_iter()
        .filter(|call| {
            tree.children(call)
                .first()
                .is_some_and(|func| tree.text(func).ends_with("RunPython"))
                && tree.arguments(call).len() < 2
                && !has_keyword(tree, call, "reverse_code")
        })
        .map(|call| {
            Diagnostic::new(
                BACKWARDS,
                "Always include backwards migration callable",
                call.range,
            )
        })
        .collect()
}

/// Whether `call` is `x.AddField(..., field=F(..., default=...))`, the shape
/// `W5198` reports. `F` is not checked to be a field type, as in Pylint.
fn adds_field_with_default(tree: &Tree<'_>, call: &Node<'_>) -> bool {
    tree.children(call)
        .first()
        .is_some_and(|func| func.kind == NodeKind::Attribute && func.name == Some("AddField"))
        && tree
            .keywords(call)
            .iter()
            .filter(|k| k.name == Some("field"))
            .filter_map(|k| tree.children(k).into_iter().next())
            .any(|field| field.kind == NodeKind::Call && has_keyword(tree, field, "default"))
}

/// W5198's findings, as `(path, diagnostic)`: the `AddField` calls with a
/// default in the latest migration of each app, which is the greatest path
/// among the migrations that share what precedes the first `migrations` in
/// it, as the sub-plugin picks it.
pub fn field_defaults(files: &[(&File, Vec<Node<'_>>)]) -> Vec<(String, Diagnostic)> {
    let mut migrations: Vec<&(&File, Vec<Node<'_>>)> = files
        .iter()
        .filter(|(file, _)| loaded(&file.options) && migrations_path(&file.path))
        .collect();
    migrations.sort_by(|a, b| b.0.path.cmp(&a.0.path));
    let mut latest = Vec::new();
    let mut last_space = "";
    for entry in migrations {
        let space = entry.0.path.split("migrations").next().unwrap_or_default();
        if space != last_space {
            last_space = space;
            latest.push(entry);
        }
    }
    let mut out = Vec::new();
    for (file, nodes) in latest {
        let tree = Tree::new(nodes, &file.source);
        for call in frame_calls(&tree) {
            if adds_field_with_default(&tree, call) {
                let message = format!("{} AddField with default value", file.module);
                out.push((
                    file.path.clone(),
                    Diagnostic::new(FIELD_DEFAULT, message, call.range),
                ));
            }
        }
    }
    out
}
