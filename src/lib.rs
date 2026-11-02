//! The `pylint_django` rules as a ferrolint plugin.

mod migrations;
mod rules;
mod tree;

use ferrolint_plugin_sdk::RawDiagnostic;
use ferrolint_plugin_sdk::prelude::*;

use tree::Tree;

/// What `pylint_django` adds to `good-names`.
const GOOD_NAMES: [&str; 9] = [
    "pk",
    "qs",
    "urlpatterns",
    "register",
    "app_name",
    "handler400",
    "handler403",
    "handler404",
    "handler500",
];

#[derive(Default)]
struct Django;

fn severity(code: &str) -> Severity {
    match code.as_bytes().first() {
        Some(b'E' | b'F') => Severity::Error,
        Some(b'W') => Severity::Warning,
        _ => Severity::Info,
    }
}

impl Checker for Django {
    fn info() -> Info {
        let mut info = Info::new("django", env!("CARGO_PKG_VERSION"))
            .contribute("good-names", &GOOD_NAMES)
            .suppressing()
            .project_wide();
        for (code, name, summary) in rules::RULES {
            info = info.rule(
                Rule::new(code, name, summary)
                    .severity(severity(code))
                    .pylint(code, name),
            );
        }
        info
    }

    fn start_file(&mut self, cx: &mut Context<'_>) {
        let tree = Tree::new(cx.nodes(), cx.source());
        let mut found = Vec::new();
        if plugin_loaded(cx.option("load-plugins")) {
            found.extend(rules::check(&tree));
        }
        let options: Vec<(String, String)> = cx
            .option("load-plugins")
            .map(|list| ("load-plugins".to_owned(), list.to_owned()))
            .into_iter()
            .collect();
        if migrations::loaded(&options) {
            found.extend(migrations::backwards(&tree, cx.path()));
        }
        for diagnostic in found {
            cx.report(diagnostic);
        }
    }

    fn check_project(files: &[(&File, Vec<Node<'_>>)]) -> Vec<(String, Diagnostic)> {
        migrations::field_defaults(files)
    }

    fn suppress(file: &File, nodes: &[Node<'_>], findings: &[RawDiagnostic]) -> Vec<u32> {
        let list = file
            .options
            .iter()
            .find(|(key, _)| key == "load-plugins")
            .map(|(_, value)| value.as_str());
        if !plugin_loaded(list) {
            return Vec::new();
        }
        let tree = Tree::new(nodes, &file.source);
        findings
            .iter()
            .enumerate()
            .filter(|(_, finding)| exempt(&tree, file, finding))
            .filter_map(|(at, _)| u32::try_from(at).ok())
            .collect()
    }
}

/// Whether `load-plugins` names `pylint_django`.
fn plugin_loaded(list: Option<&str>) -> bool {
    list.is_some_and(|list| list.split(',').any(|name| name.trim() == "pylint_django"))
}

/// Whether `pylint_django` leaves this finding of the host out.
fn exempt(tree: &Tree<'_>, file: &File, finding: &RawDiagnostic) -> bool {
    let at = finding.range.start.offset;
    if finding.message.starts_with("Too many public methods") {
        return tree
            .nodes
            .iter()
            .filter(|n| n.kind == NodeKind::ClassDef)
            .any(|class| tree.name_range(class).start.offset == at && exempt_class(tree, class));
    }
    if !finding.message.contains(" name \"") {
        return false;
    }
    let Some(name) = tree
        .nodes
        .iter()
        .find(|n| n.kind == NodeKind::Name && n.range.start.offset == at)
    else {
        return false;
    };
    let enclosing = tree.nodes.iter().rfind(|n| {
        matches!(n.kind, NodeKind::ClassDef | NodeKind::FunctionDef)
            && n.id < name.id
            && n.last >= name.id
    });
    match (name.name, enclosing) {
        (Some("application"), None) => {
            file.path.ends_with("wsgi.py") || file.path.ends_with("asgi.py")
        }
        (Some("js"), Some(class)) => {
            class.kind == NodeKind::ClassDef && class.name == Some("Media")
        }
        _ => false,
    }
}

/// A `TestCase` (or a class nested in one) and a `ModelAdmin` are out of
/// `too-many-public-methods`.
// shortcut: a base counts by its name, since its ancestry is in another
// file; upgrade with a cross-file class lookup.
fn exempt_class(tree: &Tree<'_>, class: &Node<'_>) -> bool {
    let name = class.name.unwrap_or_default();
    let nested = tree
        .nodes
        .iter()
        .any(|n| n.kind == NodeKind::ClassDef && n.id < class.id && n.last >= class.id);
    let test_case = name.ends_with("Test") || nested;
    let admin = name.ends_with("Admin") && !nested;
    tree.arguments(class).iter().any(|base| {
        tree.base_is(base, &|qualified| {
            (admin && qualified.ends_with(".ModelAdmin"))
                || (test_case
                    && (rules::exempt_test_case(qualified) || from_test_module(qualified)))
        })
    })
}

/// Whether a base is imported from a test module, such as
/// `app.tests.test_models.BaseTest` or `project_tests.testcases.Case`.
// shortcut: a base imported from a test module counts as a `TestCase`, since
// its ancestry is in another file; upgrade with a cross-file class lookup.
fn from_test_module(qualified: &str) -> bool {
    if qualified.starts_with("django.") || qualified.starts_with("rest_framework.") {
        return false;
    }
    let mut parts = qualified.split('.');
    parts.next_back();
    parts.any(|part| part.starts_with("test") || part.ends_with("tests"))
}

ferrolint_plugin_sdk::export_checker!(Django);
