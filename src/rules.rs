//! The messages of `pylint_django` that the syntax decides.

use ferrolint_plugin_sdk::prelude::*;

use crate::tree::{Mro, Tree, last};

pub const NOT_CALLABLE: &str = "E5101";
pub const AUTH_USER: &str = "E5141";
pub const IMPORTED_USER: &str = "E5142";
pub const JSON_DUMPS: &str = "R5101";
pub const JSON_CONTENT_TYPE: &str = "R5102";
pub const REDUNDANT_TYPE: &str = "R5103";
pub const HAS_UNICODE: &str = "W5102";
pub const NO_EXPLICIT: &str = "W5103";
pub const EXCLUDE: &str = "W5104";
pub const FORMAT_HTML: &str = "W5150";
pub const MARK_SAFE: &str = "W5151";

const INTERPOLATION: &str = "Pass interpolated values as arguments to format_html() instead of building the format string; arguments are automatically HTML-escaped";

/// The rules of this file, as `(code, symbol, summary)`.
pub const RULES: [(&str, &str, &str); 13] = [
    (
        NOT_CALLABLE,
        "model-unicode-not-callable",
        "A model's `__unicode__` assigned something that is not callable.",
    ),
    (
        AUTH_USER,
        "hard-coded-auth-user",
        "The string `'auth.User'`.",
    ),
    (
        IMPORTED_USER,
        "imported-auth-user",
        "`User` imported from `django.contrib.auth.models`.",
    ),
    (
        JSON_DUMPS,
        "http-response-with-json-dumps",
        "`HttpResponse(json.dumps(data))`.",
    ),
    (
        JSON_CONTENT_TYPE,
        "http-response-with-content-type-json",
        "`HttpResponse(content_type='application/json')`.",
    ),
    (
        REDUNDANT_TYPE,
        "redundant-content-type-for-json-response",
        "`JsonResponse(content_type=...)`.",
    ),
    (
        HAS_UNICODE,
        "model-has-unicode",
        "A model that defines `__unicode__`.",
    ),
    (
        NO_EXPLICIT,
        "model-no-explicit-unicode",
        "A model that only inherits `__unicode__`.",
    ),
    (
        EXCLUDE,
        "modelform-uses-exclude",
        "A `ModelForm` whose `Meta` sets `exclude`.",
    ),
    (
        FORMAT_HTML,
        "format-html-interpolation",
        "`format_html()` with an interpolated format string.",
    ),
    (
        MARK_SAFE,
        "mark-safe-interpolation",
        "`mark_safe()` of an interpolated string.",
    ),
    (
        crate::migrations::BACKWARDS,
        "missing-backwards-migration-callable",
        "A `RunPython` migration without a backwards callable.",
    ),
    (
        crate::migrations::FIELD_DEFAULT,
        "new-db-field-with-default",
        "An `AddField` migration with a default, in the latest migration of an app.",
    ),
];

/// Every finding of the rules of this file, in `tree`.
pub fn check(tree: &Tree<'_>, mro: &Mro<'_>) -> Vec<Diagnostic> {
    let docstrings = docstrings(tree);
    let mut out = Vec::new();
    for node in tree.nodes {
        match node.kind {
            NodeKind::ClassDef => {
                model(tree, node, mro, &mut out);
                model_form(tree, node, mro, &mut out);
            }
            NodeKind::ImportFrom => imported_user(tree, node, &mut out),
            NodeKind::Call => call(tree, node, &mut out),
            NodeKind::StringLiteral if !docstrings.contains(&node.id) => {
                if string_value(tree.text(node)).as_deref() == Some("auth.User") {
                    out.push(auth_user(node.range));
                }
            }
            NodeKind::FString => f_string_literals(tree, node, &mut out),
            _ => {}
        }
    }
    out
}

fn auth_user(range: Range) -> Diagnostic {
    Diagnostic::new(AUTH_USER, "Hard-coded 'auth.User'", range)
}

/// The ids of the string literals that are a docstring, which Pylint's
/// `Const` visit does not see.
fn docstrings(tree: &Tree<'_>) -> Vec<u32> {
    let mut found = Vec::new();
    for node in tree.nodes {
        if matches!(
            node.kind,
            NodeKind::Module | NodeKind::ClassDef | NodeKind::FunctionDef
        ) && let Some(first) = tree.body(node).first()
            && first.kind == NodeKind::Expr
            && let Some(value) = tree.children(first).first()
            && value.kind == NodeKind::StringLiteral
        {
            found.push(value.id);
        }
    }
    found
}

/// The value of a string literal: its pieces joined, as implicit
/// concatenation does, or none for bytes or an f-string piece.
/// shortcut: escapes are not decoded, which the one value looked for
/// (`auth.User`) never needs.
fn string_value(text: &str) -> Option<String> {
    let mut rest = text.trim_start();
    let mut value = String::new();
    while !rest.is_empty() {
        let prefix = rest.find(['"', '\''])?;
        if rest[..prefix]
            .chars()
            .any(|c| matches!(c, 'b' | 'B' | 'f' | 'F'))
        {
            return None;
        }
        let quote = rest[prefix..].chars().next()?;
        let body = &rest[prefix..];
        let triple: String = std::iter::repeat_n(quote, 3).collect();
        let width = if body.starts_with(&triple) { 3 } else { 1 };
        let inner = &body[width..];
        let (mut end, mut chars) = (None, inner.char_indices());
        while let Some((at, c)) = chars.next() {
            if c == '\\' {
                chars.next();
            } else if inner[at..].starts_with(&body[..width]) {
                end = Some(at);
                break;
            }
        }
        let end = end?;
        value.push_str(&inner[..end]);
        rest = inner[end + width..].trim_start();
    }
    Some(value)
}

/// `E5141` in the literal pieces of an f-string: a piece between
/// interpolations that is exactly `auth.User`.
fn f_string_literals(tree: &Tree<'_>, node: &Node<'_>, out: &mut Vec<Diagnostic>) {
    let text = tree.text(node);
    let base = node.range.start.offset as usize;
    // Pieces of text outside the braces; `{{` and `}}` are literal braces,
    // which end a piece for this check, as an escape changes the value.
    let (mut depth, mut start) = (0_u32, None::<usize>);
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'{' if depth == 0 && bytes.get(i + 1) == Some(&b'{') => i += 1,
            b'{' => {
                if depth == 0 {
                    close_piece(tree, text, base, start.take(), i, out);
                }
                depth += 1;
            }
            b'}' if depth > 0 => {
                depth -= 1;
                if depth == 0 {
                    start = Some(i + 1);
                }
            }
            b'"' | b'\'' if depth == 0 && start.is_none() => {
                // The opening quote (after the prefix) starts the first piece.
                start = Some(i + 1);
            }
            _ => {}
        }
        i += 1;
    }
    let end = text.len().saturating_sub(1);
    close_piece(tree, text, base, start, end, out);
}

fn close_piece(
    tree: &Tree<'_>,
    text: &str,
    base: usize,
    start: Option<usize>,
    end: usize,
    out: &mut Vec<Diagnostic>,
) {
    let Some(start) = start else { return };
    if text.get(start..end) == Some("auth.User") {
        out.push(auth_user(tree.range_between(base + start, base + end)));
    }
}

fn imported_user(tree: &Tree<'_>, node: &Node<'_>, out: &mut Vec<Diagnostic>) {
    if node.detail == Some("django.contrib.auth.models")
        && tree
            .children(node)
            .iter()
            .any(|alias| matches!(alias.name, Some("*" | "User")))
    {
        out.push(Diagnostic::new(
            IMPORTED_USER,
            "User model imported from django.contrib.auth.models",
            node.range,
        ));
    }
}

/// `name = value` as a statement: the name and the value node.
fn assigned_name<'a>(tree: &Tree<'a>, stmt: &Node<'a>) -> Option<(&'a str, &'a Node<'a>)> {
    if stmt.kind != NodeKind::Assign {
        return None;
    }
    let children = tree.children(stmt);
    let (target, value) = (children.first()?, children.last()?);
    (target.kind == NodeKind::Name && children.len() >= 2).then_some((target.name?, value))
}

/// Whether a value is a literal, which no call can be made of.
fn literal(value: &Node<'_>) -> bool {
    use NodeKind as K;
    matches!(
        value.kind,
        K::StringLiteral
            | K::BytesLiteral
            | K::NumberLiteral
            | K::BooleanLiteral
            | K::NoneLiteral
            | K::EllipsisLiteral
            | K::FString
            | K::List
            | K::Tuple
            | K::Dict
            | K::Set
    )
}

fn meta_of<'a>(tree: &Tree<'a>, class: &Node<'a>) -> Option<&'a Node<'a>> {
    tree.body(class)
        .into_iter()
        .find(|s| s.kind == NodeKind::ClassDef && s.name == Some("Meta"))
}

/// `E5101`, `W5102` and `W5103` for a model.
fn model(tree: &Tree<'_>, class: &Node<'_>, mro: &Mro<'_>, out: &mut Vec<Diagnostic>) {
    if !tree.derives(class, &["Model"], mro) {
        return;
    }
    let name = class.name.unwrap_or_default();
    let at = tree.name_range(class);
    for stmt in tree.body(class) {
        if stmt.kind == NodeKind::ClassDef
            && stmt.name == Some("Meta")
            && tree.body(stmt).iter().any(|s| {
                assigned_name(tree, s).is_some_and(|(n, v)| n == "abstract" && truthy(tree, v))
            })
        {
            return;
        }
        if let Some(("__unicode__", value)) = assigned_name(tree, stmt) {
            if literal(value) {
                let message = format!("__unicode__ on a model must be callable ({name})");
                out.push(Diagnostic::new(NOT_CALLABLE, message, at));
            }
            return;
        }
        if stmt.kind == NodeKind::FunctionDef && stmt.name == Some("__unicode__") {
            let message =
                format!("Found __unicode__ method on model ({name}). Python3 uses __str__.");
            out.push(Diagnostic::new(HAS_UNICODE, message, at));
            return;
        }
    }
    inherits_unicode(tree, class, at, out);
}

fn truthy(tree: &Tree<'_>, value: &Node<'_>) -> bool {
    match value.kind {
        NodeKind::BooleanLiteral => value.detail == Some("True"),
        NodeKind::StringLiteral => string_value(tree.text(value)).is_none_or(|v| !v.is_empty()),
        _ => false,
    }
}

/// `W5103`: a model whose ancestors define `__unicode__`, or `__str__` under
/// `python_2_unicode_compatible`, and that does not define it.
fn inherits_unicode(tree: &Tree<'_>, class: &Node<'_>, at: Range, out: &mut Vec<Diagnostic>) {
    let methods = |c: &Node<'_>| -> Vec<String> {
        tree.body(c)
            .into_iter()
            .filter(|s| s.kind == NodeKind::FunctionDef)
            .filter_map(|s| s.name.map(str::to_owned))
            .collect()
    };
    let mut seen = methods(class);
    for ancestor in tree.ancestors(class) {
        let compat = tree.children(ancestor).iter().any(|d| {
            d.kind == NodeKind::Decorator
                && tree.children(d).first().is_some_and(|e| {
                    e.kind == NodeKind::Name && e.name == Some("python_2_unicode_compatible")
                })
        });
        for method in methods(ancestor) {
            if seen.contains(&method) {
                continue;
            }
            if method == "__unicode__" || (method == "__str__" && compat) {
                let name = class.name.unwrap_or_default();
                let message = format!("Model does not explicitly define __unicode__ ({name})");
                out.push(Diagnostic::new(NO_EXPLICIT, message, at));
                return;
            }
            seen.push(method);
        }
    }
}

/// `W5104`: a `ModelForm` whose `Meta` assigns `exclude`.
fn model_form(tree: &Tree<'_>, class: &Node<'_>, mro: &Mro<'_>, out: &mut Vec<Diagnostic>) {
    if !tree.derives(class, &["ModelForm"], mro) {
        return;
    }
    if let Some(meta) = meta_of(tree, class)
        && let Some(stmt) = tree
            .body(meta)
            .into_iter()
            .find(|s| assigned_name(tree, s).is_some_and(|(n, _)| n == "exclude"))
    {
        out.push(Diagnostic::new(
            EXCLUDE,
            "Use explicit fields instead of exclude in ModelForm",
            stmt.range,
        ));
    }
}

fn interpolated(tree: &Tree<'_>, expr: &Node<'_>) -> bool {
    use NodeKind as K;
    let constant = |e: &Node<'_>| {
        matches!(
            e.kind,
            K::StringLiteral
                | K::BytesLiteral
                | K::NumberLiteral
                | K::BooleanLiteral
                | K::NoneLiteral
                | K::EllipsisLiteral
        )
    };
    match expr.kind {
        K::FString => !tree.children(expr).is_empty(),
        K::BinOp => match expr.detail {
            Some("%") => true,
            Some("+") => {
                let operands = tree.children(expr);
                let (Some(left), Some(right)) = (operands.first(), operands.last()) else {
                    return false;
                };
                (left.kind == K::StringLiteral || right.kind == K::StringLiteral)
                    && (!constant(left) || !constant(right))
            }
            _ => false,
        },
        K::Call => tree
            .children(expr)
            .first()
            .is_some_and(|func| func.kind == K::Attribute && func.name == Some("format")),
        _ => false,
    }
}

/// `R5101`, `R5102`, `R5103`, `W5150` and `W5151`.
fn call(tree: &Tree<'_>, call: &Node<'_>, out: &mut Vec<Diagnostic>) {
    let Some(func) = tree.children(call).into_iter().next() else {
        return;
    };
    let func = tree.text(func);
    let args = tree.arguments(call);
    let keywords = tree.keywords(call);
    let keyword = |name: &str| keywords.iter().find(|k| k.name == Some(name));
    if func.ends_with("HttpResponse") {
        if let Some(first) = args.first()
            && first.kind == NodeKind::Call
            && tree
                .children(first)
                .first()
                .is_some_and(|inner| tree.text(inner) == "json.dumps")
        {
            out.push(Diagnostic::new(
                JSON_DUMPS,
                "Instead of HttpResponse(json.dumps(data)) use JsonResponse(data)",
                call.range,
            ));
        }
        if let Some(value) =
            keyword("content_type").and_then(|k| tree.children(k).into_iter().next())
            && tree.text(value).to_lowercase().contains("application/json")
        {
            out.push(Diagnostic::new(
                JSON_CONTENT_TYPE,
                "Instead of HttpResponse(content_type='application/json') use JsonResponse()",
                call.range,
            ));
        }
    }
    if func.ends_with("JsonResponse") && keyword("content_type").is_some() {
        out.push(Diagnostic::new(
            REDUNDANT_TYPE,
            "Redundant content_type parameter for JsonResponse()",
            call.range,
        ));
    }
    let (candidate, code) = if func.ends_with("format_html_join") && args.len() >= 2 {
        (args[1], FORMAT_HTML)
    } else if func.ends_with("format_html") && !args.is_empty() {
        (args[0], FORMAT_HTML)
    } else if func.ends_with("mark_safe") && !args.is_empty() {
        (args[0], MARK_SAFE)
    } else {
        return;
    };
    if interpolated(tree, candidate) {
        let message = if code == FORMAT_HTML {
            INTERPOLATION
        } else {
            "Avoid mark_safe() on a dynamically-built string; use format_html() with arguments so values are HTML-escaped"
        };
        out.push(Diagnostic::new(code, message, call.range));
    }
}

/// Whether a dotted name is the Django `TestCase` family `pylint_django`
/// exempts from `too-many-public-methods`.
pub fn exempt_test_case(qualified: &str) -> bool {
    let tail = last(qualified);
    tail.ends_with("TestCase")
        && !qualified.starts_with("unittest.")
        && !["Simple", "Transaction", "LiveServer", "IsolatedAsyncio"]
            .iter()
            .any(|kind| tail.contains(kind))
}
