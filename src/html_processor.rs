use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

use crate::Parser;

use html5ever::{LocalName, QualName};
#[expect(unused_imports)]
use html5ever::{namespace_url, ns};
use kuchikikiki::traits::TendrilSink;
use kuchikikiki::{Attribute, ExpandedName, NodeData, NodeRef};

const ZWSP_CODEPOINT: u32 = 0x200B;
const ZWSP: &str = "\u{200B}";

const PARENT_STYLE: &str = "word-break: keep-all; overflow-wrap: anywhere;";

/// Separator inserted at semantic boundaries.
#[non_exhaustive]
#[derive(Clone, Debug)]
pub enum Separator {
    /// Insert the provided string at each boundary.
    Text(String),
    /// Insert a cloned node at each boundary.
    Node(NodeRef),
}

impl Default for Separator {
    fn default() -> Self {
        Self::Text(ZWSP.to_string())
    }
}

/// Options for [`HTMLProcessor`].
#[non_exhaustive]
#[derive(Debug)]
pub struct HTMLProcessorOptions {
    /// Optional class name added to the containing element.
    pub class_name: Option<String>,
    /// Separator to insert at semantic boundaries.
    pub separator: Option<Separator>,
}

impl Default for HTMLProcessorOptions {
    fn default() -> Self {
        Self {
            class_name: None,
            separator: Some(Separator::default()),
        }
    }
}

static DOM_ACTIONS: LazyLock<HashMap<&'static str, DomAction>> = LazyLock::new(dom_actions);

static BLOCK_ELEMENTS: LazyLock<HashSet<&'static str>> = LazyLock::new(default_block_elements);

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum DomAction {
    Inline,
    Block,
    Skip,
    Break,
    NoBreak,
    BreakOpportunity,
}

fn dom_actions() -> HashMap<&'static str, DomAction> {
    use DomAction::{Break, BreakOpportunity, Inline, NoBreak, Skip};
    HashMap::from([
        ("AREA", Skip),
        ("BASE", Skip),
        ("BASEFONT", Skip),
        ("DATALIST", Skip),
        ("HEAD", Skip),
        ("LINK", Skip),
        ("META", Skip),
        ("NOEMBED", Skip),
        ("NOFRAMES", Skip),
        ("PARAM", Skip),
        ("RP", Skip),
        ("SCRIPT", Skip),
        ("STYLE", Skip),
        ("TEMPLATE", Skip),
        ("TITLE", Skip),
        ("NOSCRIPT", Skip),
        ("HR", Break),
        ("LISTING", Skip),
        ("PLAINTEXT", Skip),
        ("PRE", Skip),
        ("XMP", Skip),
        ("BR", Break),
        ("RT", Skip),
        ("WBR", BreakOpportunity),
        ("INPUT", Skip),
        ("SELECT", Skip),
        ("BUTTON", Skip),
        ("TEXTAREA", Skip),
        ("ABBR", Skip),
        ("CODE", Skip),
        ("IFRAME", Skip),
        ("TIME", Skip),
        ("VAR", Skip),
        ("NOBR", NoBreak),
        ("SPAN", Inline),
    ])
}

fn default_block_elements() -> HashSet<&'static str> {
    HashSet::from([
        "HTML",
        "BODY",
        "ADDRESS",
        "BLOCKQUOTE",
        "CENTER",
        "DIALOG",
        "DIV",
        "FIGURE",
        "FIGCAPTION",
        "FOOTER",
        "FORM",
        "HEADER",
        "LEGEND",
        "LISTING",
        "MAIN",
        "P",
        "ARTICLE",
        "ASIDE",
        "H1",
        "H2",
        "H3",
        "H4",
        "H5",
        "H6",
        "HGROUP",
        "NAV",
        "SECTION",
        "DIR",
        "DD",
        "DL",
        "DT",
        "MENU",
        "OL",
        "UL",
        "LI",
        "TABLE",
        "CAPTION",
        "COL",
        "TR",
        "TD",
        "TH",
        "FIELDSET",
        "DETAILS",
        "SUMMARY",
        "MARQUEE",
    ])
}

fn action_for_element(node: &NodeRef) -> DomAction {
    let Some(element) = node.as_element() else {
        return DomAction::Inline;
    };
    let name = element.name.local.to_string().to_uppercase();
    if let Some(action) = DOM_ACTIONS.get(name.as_str()) {
        return *action;
    }
    if BLOCK_ELEMENTS.contains(name.as_str()) {
        DomAction::Block
    } else {
        DomAction::Inline
    }
}

/// HTML processor that applies `BudouX` boundaries to a DOM.
#[derive(Debug)]
pub struct HTMLProcessor {
    parser: Parser,
    class_name: Option<String>,
    separator: Separator,
}

impl HTMLProcessor {
    /// Create a new HTML processor from a [`Parser`].
    #[must_use]
    pub fn new(parser: Parser, options: Option<HTMLProcessorOptions>) -> Self {
        let options = options.unwrap_or_default();
        Self {
            parser,
            class_name: options.class_name,
            separator: options.separator.unwrap_or_default(),
        }
    }

    /// Apply `BudouX` boundaries to an HTML string.
    #[must_use]
    pub fn apply_to_html_string(&self, html: &str) -> String {
        if html.is_empty() {
            return String::new();
        }
        let document = kuchikikiki::parse_html()
            .one(format!("<!doctype html><html><body>{html}</body></html>"));
        let body = match document.select_first("body") {
            Ok(body) => body.as_node().clone(),
            Err(()) => return html.to_string(),
        };

        let children: Vec<NodeRef> = body.children().collect();
        let child_count = children.len();
        let has_text_child = children.iter().any(|child| child.as_text().is_some());

        let target = if child_count == 1 && !has_text_child {
            let Some(first_child) = body.first_child() else {
                return html.to_string();
            };
            first_child
        } else {
            let wrapper = new_element("span");
            for node in children {
                node.detach();
                wrapper.append(node);
            }
            body.append(wrapper.clone());
            wrapper
        };

        self.apply_to_element(&target);

        target.to_string()
    }

    /// Apply `BudouX` boundaries to a DOM element.
    pub fn apply_to_element(&self, element: &NodeRef) {
        let mut blocks = Vec::new();
        self.collect_blocks(element, None, &mut blocks);
        for block in blocks {
            if !block.nodes.is_empty() {
                self.apply_to_paragraph(block);
            }
        }
    }

    fn collect_blocks(
        &self,
        element: &NodeRef,
        parent: Option<Paragraph>,
        output: &mut Vec<Paragraph>,
    ) {
        let action = action_for_element(element);
        if action == DomAction::Skip {
            return;
        }
        if action == DomAction::Break {
            if let Some(mut parent) = parent
                && !parent.nodes.is_empty()
            {
                parent.set_has_break_opportunity_after();
                output.push(parent);
            }
            return;
        }
        if action == DomAction::BreakOpportunity {
            if let Some(mut parent) = parent {
                parent.set_has_break_opportunity_after();
            }
            return;
        }

        let is_new_block = parent.is_none() || action == DomAction::Block;
        let mut block = if is_new_block {
            Paragraph::new(element.clone())
        } else {
            let Some(block) = parent else {
                return;
            };
            block
        };

        for child in element.children() {
            match child.data() {
                NodeData::Element(_) => {
                    self.collect_blocks(&child, Some(block.clone()), output);
                }
                NodeData::Text(_) => {
                    if action == DomAction::NoBreak {
                        if let Some(text) = child.as_text() {
                            block
                                .nodes
                                .push(NodeOrText::from_string(text.borrow().clone()));
                        }
                    } else {
                        block.nodes.push(NodeOrText::from_node(child.clone()));
                    }
                }
                _ => {}
            }
        }

        if is_new_block && !block.nodes.is_empty() {
            output.push(block);
        }
    }

    fn apply_to_paragraph(&self, mut paragraph: Paragraph) {
        if !paragraph.nodes.iter().any(NodeOrText::can_split) {
            return;
        }
        let text = paragraph.text();
        if text.trim().is_empty() {
            return;
        }
        let boundaries = self.parser.parse_boundaries(&text);
        if boundaries.is_empty() {
            return;
        }
        let adjusted = paragraph.exclude_forced_opportunities(boundaries);
        if adjusted.is_empty() {
            return;
        }
        let mut boundaries = adjusted;
        boundaries.push(text.chars().count() + 1);

        self.split_nodes(&mut paragraph.nodes, &boundaries);
        self.apply_block_style(&paragraph.element);
    }

    fn split_nodes(&self, nodes: &mut [NodeOrText], boundaries: &[usize]) {
        let mut boundary_index = 0usize;
        let mut boundary = boundaries[0];
        let mut node_start = 0usize;
        let mut last_node_can_split = false;

        for node in nodes.iter_mut() {
            let Some(node_text) = node.text() else {
                continue;
            };
            let node_len = node_text.chars().count();
            let node_end = node_start + node_len;

            if !node.can_split() {
                if last_node_can_split && boundary == node_start {
                    node.add_boundary_at_end();
                }
                while boundary < node_end {
                    boundary_index += 1;
                    boundary = boundaries[boundary_index];
                }
                last_node_can_split = false;
                node_start = node_end;
                continue;
            }

            last_node_can_split = true;
            if boundary >= node_end {
                node_start = node_end;
                continue;
            }

            let mut chunk_start = 0usize;
            while boundary < node_end {
                let boundary_in_node = boundary - node_start;
                node.push_chunk(slice_chars(&node_text, chunk_start, boundary_in_node));
                chunk_start = boundary_in_node;
                boundary_index += 1;
                boundary = boundaries[boundary_index];
            }
            node.push_chunk(slice_chars(&node_text, chunk_start, node_len));
            node_start = node_end;
        }

        for node in nodes.iter_mut() {
            node.split(&self.separator);
        }
    }

    fn apply_block_style(&self, element: &NodeRef) {
        if let Some(class_name) = &self.class_name {
            if let Some(el) = element.as_element() {
                let mut attrs = el.attributes.borrow_mut();
                let existing = attrs.get("class").unwrap_or("").to_string();
                let new_value = if existing.is_empty() {
                    class_name.clone()
                } else if existing.split_whitespace().any(|item| item == class_name) {
                    existing
                } else {
                    format!("{existing} {class_name}")
                };
                attrs.insert("class", new_value);
            }
        } else if let Some(el) = element.as_element() {
            let mut attrs = el.attributes.borrow_mut();
            let existing = attrs.get("style").unwrap_or("").trim().to_string();
            let new_value = if existing.is_empty() {
                PARENT_STYLE.to_string()
            } else if existing.contains(PARENT_STYLE) {
                existing
            } else {
                format!("{existing} {PARENT_STYLE}")
            };
            attrs.insert("style", new_value);
        }
    }
}

#[derive(Clone)]
struct NodeOrText {
    node: NodeOrTextInner,
    chunks: Vec<String>,
    has_break_opportunity_after: bool,
}

#[derive(Clone)]
enum NodeOrTextInner {
    Node(NodeRef),
    Text(String),
}

impl NodeOrText {
    const fn from_node(node: NodeRef) -> Self {
        Self {
            node: NodeOrTextInner::Node(node),
            chunks: Vec::new(),
            has_break_opportunity_after: false,
        }
    }

    const fn from_string(text: String) -> Self {
        Self {
            node: NodeOrTextInner::Text(text),
            chunks: Vec::new(),
            has_break_opportunity_after: false,
        }
    }

    const fn can_split(&self) -> bool {
        matches!(self.node, NodeOrTextInner::Node(_))
    }

    fn text(&self) -> Option<String> {
        match &self.node {
            NodeOrTextInner::Text(text) => Some(text.clone()),
            NodeOrTextInner::Node(node) => node.as_text().map(|text| text.borrow().clone()),
        }
    }

    fn length(&self) -> usize {
        self.text().map_or(0, |text| text.chars().count())
    }

    fn push_chunk(&mut self, value: String) {
        self.chunks.push(value);
    }

    fn add_boundary_at_end(&mut self) {
        if self.chunks.is_empty()
            && let Some(text) = self.text()
        {
            self.chunks.push(text);
        }
        self.chunks.push(String::new());
    }

    fn split(&self, separator: &Separator) {
        if self.chunks.len() <= 1 {
            return;
        }
        match &self.node {
            NodeOrTextInner::Text(_) => (),
            NodeOrTextInner::Node(node) => match separator {
                Separator::Text(sep) => {
                    if let Some(text_ref) = node.as_text() {
                        *text_ref.borrow_mut() = self.chunks.join(sep);
                    }
                }
                Separator::Node(sep_node) => {
                    let mut nodes = Vec::new();
                    for chunk in &self.chunks {
                        if !chunk.is_empty() {
                            nodes.push(NodeRef::new_text(chunk.clone()));
                        }
                        nodes.push(clone_subtree(sep_node));
                    }
                    nodes.pop();
                    if let Some(parent) = node.parent() {
                        for new_node in nodes.iter().rev() {
                            parent.insert_after(new_node.clone());
                        }
                        node.detach();
                    }
                }
            },
        }
    }
}

#[derive(Clone)]
struct Paragraph {
    element: NodeRef,
    nodes: Vec<NodeOrText>,
}

impl Paragraph {
    const fn new(element: NodeRef) -> Self {
        Self {
            element,
            nodes: Vec::new(),
        }
    }

    fn text(&self) -> String {
        self.nodes
            .iter()
            .filter_map(NodeOrText::text)
            .collect::<String>()
    }

    fn set_has_break_opportunity_after(&mut self) {
        if let Some(last) = self.nodes.last_mut() {
            last.has_break_opportunity_after = true;
        }
    }

    fn get_forced_opportunities(&self) -> Vec<usize> {
        let mut opportunities = Vec::new();
        let mut len = 0usize;
        for node in &self.nodes {
            if node.can_split()
                && let Some(text) = node.text()
            {
                for (idx, ch) in text.chars().enumerate() {
                    if ch as u32 == ZWSP_CODEPOINT {
                        opportunities.push(len + idx + 1);
                    }
                }
            }
            len += node.length();
            if node.has_break_opportunity_after {
                opportunities.push(len);
            }
        }
        opportunities
    }

    fn exclude_forced_opportunities(&self, boundaries: Vec<usize>) -> Vec<usize> {
        let forced = self.get_forced_opportunities();
        if forced.is_empty() {
            return boundaries;
        }
        let set: HashSet<usize> = forced.into_iter().collect();
        boundaries
            .into_iter()
            .filter(|b| !set.contains(b))
            .collect()
    }
}

/// `BudouX` parser with HTML processing support.
#[derive(Debug)]
pub struct HTMLProcessingParser {
    parser: Parser,
    processor: HTMLProcessor,
}

impl HTMLProcessingParser {
    #[must_use]
    /// Create a new HTML processing parser.
    pub fn new(parser: Parser, options: Option<HTMLProcessorOptions>) -> Self {
        let processor = HTMLProcessor::new(parser.clone(), options);
        Self { parser, processor }
    }

    #[must_use]
    /// Split a sentence into semantic chunks.
    pub fn parse(&self, sentence: &str) -> Vec<String> {
        self.parser.parse(sentence)
    }

    #[must_use]
    /// Return the boundary indices for the sentence.
    pub fn parse_boundaries(&self, sentence: &str) -> Vec<usize> {
        self.parser.parse_boundaries(sentence)
    }

    /// Apply `BudouX` boundaries to a DOM element.
    pub fn apply_to_element(&self, element: &NodeRef) {
        self.processor.apply_to_element(element);
    }

    #[must_use]
    /// Apply `BudouX` boundaries to an HTML string.
    pub fn translate_html_string(&self, html: &str) -> String {
        self.processor.apply_to_html_string(html)
    }
}

fn new_element(tag: &str) -> NodeRef {
    NodeRef::new_element(
        QualName::new(None, ns!(html), LocalName::from(tag)),
        Vec::<(ExpandedName, Attribute)>::new(),
    )
}

fn slice_chars(input: &str, start: usize, end: usize) -> String {
    input.chars().skip(start).take(end - start).collect()
}

fn clone_subtree(node: &NodeRef) -> NodeRef {
    let cloned = match node.data() {
        NodeData::Document(_) => NodeRef::new_document(),
        NodeData::DocumentFragment => NodeRef::new(NodeData::DocumentFragment),
        NodeData::Doctype(value) => NodeRef::new_doctype(
            value.name.clone(),
            value.public_id.clone(),
            value.system_id.clone(),
        ),
        NodeData::Comment(value) => NodeRef::new_comment(value.borrow().clone()),
        NodeData::ProcessingInstruction(value) => {
            let (target, data) = value.borrow().clone();
            NodeRef::new_processing_instruction(target, data)
        }
        NodeData::Text(value) => NodeRef::new_text(value.borrow().clone()),
        NodeData::Element(element) => {
            let name = element.name.clone();
            let attrs = element
                .attributes
                .borrow()
                .map
                .iter()
                .map(|(k, v)| {
                    (
                        ExpandedName::new(k.ns.clone(), k.local.clone()),
                        Attribute {
                            prefix: v.prefix.clone(),
                            value: v.value.clone(),
                        },
                    )
                })
                .collect::<Vec<_>>();
            NodeRef::new_element(name, attrs)
        }
    };

    let children: Vec<NodeRef> = node.children().collect();
    for child in children {
        cloned.append(clone_subtree(&child));
    }

    cloned
}
