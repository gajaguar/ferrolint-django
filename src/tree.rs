//! Reads over the packed node list: children, class bases, imports and the
//! positions of names, with no help from the host, so `check` and `suppress`
//! (which has no semantic model) share it.

use std::collections::HashMap;

use ferrolint_plugin_sdk::prelude::*;

/// The host's MRO of a base expression, class first, or none.
pub type Mro<'a> = dyn Fn(&Node<'_>) -> Option<Vec<String>> + 'a;

pub struct Tree<'a> {
    pub nodes: &'a [Node<'a>],
    pub source: &'a str,
    /// What each imported name stands for: `HttpResponse` is
    /// `django.http.HttpResponse` after `from django.http import
    /// HttpResponse`.
    imports: HashMap<&'a str, String>,
}

impl<'a> Tree<'a> {
    pub fn new(nodes: &'a [Node<'a>], source: &'a str) -> Self {
        let mut tree = Self {
            nodes,
            source,
            imports: HashMap::new(),
        };
        tree.read_imports();
        tree
    }

    // shortcut: every import in the file counts, in any scope; upgrade if a
    // function-local import shadows a module-level name.
    fn read_imports(&mut self) {
        for node in self.nodes {
            match node.kind {
                NodeKind::ImportFrom => {
                    let module = node.detail.unwrap_or_default();
                    for alias in self.children(node) {
                        let Some(name) = alias.name.filter(|name| *name != "*") else {
                            continue;
                        };
                        let key = alias.detail.unwrap_or(name);
                        self.imports
                            .entry(key)
                            .or_insert(format!("{module}.{name}"));
                    }
                }
                NodeKind::Import => {
                    for alias in self.children(node) {
                        let Some(name) = alias.name else { continue };
                        let key = alias
                            .detail
                            .unwrap_or_else(|| name.split('.').next().unwrap_or(name));
                        let target = if alias.detail.is_some() { name } else { key };
                        self.imports.entry(key).or_insert(target.to_owned());
                    }
                }
                _ => {}
            }
        }
    }

    pub fn children(&self, node: &Node<'_>) -> Vec<&'a Node<'a>> {
        let mut next = node.id + 1;
        let mut out = Vec::new();
        while next <= node.last {
            let child = &self.nodes[next as usize];
            out.push(child);
            next = child.last + 1;
        }
        out
    }

    pub fn text(&self, node: &Node<'_>) -> &'a str {
        let range = node.range;
        self.source
            .get(range.start.offset as usize..range.end.offset as usize)
            .unwrap_or("")
    }

    /// The statements of a class or function body.
    pub fn body(&self, node: &Node<'_>) -> Vec<&'a Node<'a>> {
        self.children(node)
            .into_iter()
            .filter(|child| Context::is_statement(child))
            .collect()
    }

    /// The positional arguments of a call, or the bases of a class.
    pub fn arguments(&self, node: &Node<'_>) -> Vec<&'a Node<'a>> {
        self.arguments_node(node)
            .map(|arguments| {
                self.children(arguments)
                    .into_iter()
                    .filter(|child| child.kind != NodeKind::Keyword)
                    .collect()
            })
            .unwrap_or_default()
    }

    /// The keywords of a call, or of a class statement.
    pub fn keywords(&self, node: &Node<'_>) -> Vec<&'a Node<'a>> {
        self.arguments_node(node)
            .map(|arguments| {
                self.children(arguments)
                    .into_iter()
                    .filter(|child| child.kind == NodeKind::Keyword)
                    .collect()
            })
            .unwrap_or_default()
    }

    fn arguments_node(&self, node: &Node<'_>) -> Option<&'a Node<'a>> {
        self.children(node)
            .into_iter()
            .find(|child| child.kind == NodeKind::Arguments)
    }

    /// The dotted name an expression of names and attributes stands for,
    /// following imports. A name that is not imported stands for itself.
    pub fn qualified(&self, node: &Node<'_>) -> Option<String> {
        match node.kind {
            NodeKind::Name => {
                let name = node.name?;
                Some(
                    self.imports
                        .get(name)
                        .cloned()
                        .unwrap_or_else(|| name.to_owned()),
                )
            }
            NodeKind::Attribute => {
                let value = self.children(node).into_iter().next()?;
                Some(format!("{}.{}", self.qualified(value)?, node.name?))
            }
            _ => None,
        }
    }

    /// The only class of the file with this name.
    pub fn class_named(&self, name: &str) -> Option<&'a Node<'a>> {
        let mut found = self
            .nodes
            .iter()
            .filter(|node| node.kind == NodeKind::ClassDef && node.name == Some(name));
        let class = found.next()?;
        found.next().is_none().then_some(class)
    }

    /// The class of this file that a base names.
    pub fn class_of(&self, base: &Node<'_>) -> Option<&'a Node<'a>> {
        (base.kind == NodeKind::Name)
            .then(|| self.class_named(base.name?))
            .flatten()
    }

    /// Whether `base`, or a class of this file it names, is one of the
    /// classes `pred` accepts by qualified name.
    pub fn base_is(&self, base: &Node<'_>, pred: &dyn Fn(&str) -> bool) -> bool {
        self.base_is_at(base, pred, 0)
    }

    fn base_is_at(&self, base: &Node<'_>, pred: &dyn Fn(&str) -> bool, depth: u8) -> bool {
        if self.qualified(base).is_some_and(|name| pred(&name)) {
            return true;
        }
        depth < 16
            && self.class_of(base).is_some_and(|class| {
                self.arguments(class)
                    .iter()
                    .any(|base| self.base_is_at(base, pred, depth + 1))
            })
    }

    /// Whether the class derives from a Django class whose last name is one of
    /// `names`. `mro` is the host's MRO of a base expression, when it has one;
    /// it decides for that base. shortcut: with no answer a base is a Django
    /// class by its name, since the real ancestry is in other files; the
    /// fallback stays for a host that cannot tell.
    pub fn derives(&self, class: &Node<'_>, names: &[&str], mro: &Mro<'_>) -> bool {
        self.derives_at(class, names, mro, 0)
    }

    fn derives_at(&self, class: &Node<'_>, names: &[&str], mro: &Mro<'_>, depth: u8) -> bool {
        depth < 16
            && self.arguments(class).iter().any(|base| {
                if let Some(chain) = mro(base) {
                    return chain
                        .iter()
                        .any(|q| q.starts_with("django.") && names.contains(&last(q)));
                }
                names.contains(&last(self.text(base)))
                    || self
                        .class_of(base)
                        .is_some_and(|c| self.derives_at(c, names, mro, depth + 1))
            })
    }

    /// The classes of this file that `class` inherits from, nearest first.
    pub fn ancestors(&self, class: &Node<'_>) -> Vec<&'a Node<'a>> {
        let mut found: Vec<&Node<'_>> = Vec::new();
        self.ancestors_at(class, &mut found, 0);
        found
    }

    fn ancestors_at(&self, class: &Node<'_>, found: &mut Vec<&'a Node<'a>>, depth: u8) {
        if depth >= 16 {
            return;
        }
        for base in self.arguments(class) {
            if let Some(c) = self.class_of(base)
                && !found.iter().any(|f| f.id == c.id)
            {
                found.push(c);
                self.ancestors_at(c, found, depth + 1);
            }
        }
    }

    /// The range of a class's name: the nodes' range covers the decorators
    /// and the `class` keyword too.
    pub fn name_range(&self, class: &Node<'_>) -> Range {
        let after = self
            .children(class)
            .into_iter()
            .filter(|child| child.kind == NodeKind::Decorator)
            .map(|decorator| decorator.range.end.offset)
            .next_back()
            .unwrap_or(class.range.start.offset) as usize;
        let rest = self.source.get(after..).unwrap_or("");
        let name = class.name.unwrap_or("");
        let keyword = find_keyword(rest, "class").unwrap_or(0);
        let from = keyword + "class".len();
        let start = after + from + rest[from..].find(name).unwrap_or(0);
        self.range_between(start, start + name.len())
    }

    /// The range of the bytes `start..end`, whose line and column are found
    /// from the start of the source.
    pub fn range_between(&self, start: usize, end: usize) -> Range {
        Range {
            start: self.position(start),
            end: self.position(end),
        }
    }

    fn position(&self, offset: usize) -> Position {
        let before = &self.source[..offset.min(self.source.len())];
        let line = before.matches('\n').count() + 1;
        let column = before
            .rsplit('\n')
            .next()
            .map_or(0, |last| last.chars().count())
            + 1;
        Position {
            offset: u32::try_from(offset).unwrap_or(u32::MAX),
            line: u32::try_from(line).unwrap_or(u32::MAX),
            column: u32::try_from(column).unwrap_or(u32::MAX),
        }
    }
}

/// The last dotted segment of `text`.
pub fn last(text: &str) -> &str {
    text.rsplit('.').next().unwrap_or(text)
}

/// The index of `word` as a whole word in `text`.
fn find_keyword(text: &str, word: &str) -> Option<usize> {
    let mut from = 0;
    while let Some(at) = text[from..].find(word) {
        let start = from + at;
        let end = start + word.len();
        let boundary = |c: Option<char>| c.is_none_or(|c| !(c.is_alphanumeric() || c == '_'));
        if boundary(text[..start].chars().next_back()) && boundary(text[end..].chars().next()) {
            return Some(start);
        }
        from = end;
    }
    None
}
