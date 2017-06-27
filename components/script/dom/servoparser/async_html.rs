/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![allow(unrooted_must_root)]

use dom::bindings::codegen::Bindings::HTMLTemplateElementBinding::HTMLTemplateElementMethods;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{JS, MutNullableJS, Root};
use dom::bindings::str::DOMString;
use dom::bindings::trace::JSTraceable;
use dom::comment::Comment;
use dom::document::Document;
use dom::documenttype::DocumentType;
use dom::element::{CustomElementCreationMode, Element, ElementCreator};
use dom::htmlformelement::{FormControlElementHelpers, HTMLFormElement};
use dom::htmlscriptelement::HTMLScriptElement;
use dom::htmltemplateelement::HTMLTemplateElement;
use dom::node::Node;
use dom::processinginstruction::ProcessingInstruction;
use dom::virtualmethods::vtable_for;
use html5ever::{Attribute, LocalName, QualName, ExpandedName};
use html5ever::buffer_queue::BufferQueue;
use html5ever::tendril::StrTendril;
use html5ever::tokenizer::{Tokenizer as HtmlTokenizer, TokenizerOpts, TokenizerResult};
use html5ever::tree_builder::{NodeOrText, TreeSink, NextParserState, QuirksMode, ElementFlags};
use html5ever::tree_builder::{Tracer as HtmlTracer, TreeBuilder, TreeBuilderOpts};
use js::jsapi::JSTracer;
use servo_url::ServoUrl;
use std::ascii::AsciiExt;
use std::borrow::Cow;
use std::cell::Cell;
use std::collections::HashMap;
use style::context::QuirksMode as ServoQuirksMode;

#[derive(HeapSizeOf, JSTraceable)]
#[must_root]
pub struct Tokenizer {
    #[ignore_heap_size_of = "Defined in html5ever"]
    inner: HtmlTokenizer<TreeBuilder<ParseNode, Sink>>,
}

impl Tokenizer {
    pub fn new(
            document: &Document,
            url: ServoUrl,
            fragment_context: Option<super::FragmentContext>)
            -> Self {
        let mut sink = Sink::new(url, document);

        let options = TreeBuilderOpts {
            ignore_missing_rules: true,
            .. Default::default()
        };

        let inner = if let Some(fc) = fragment_context {
            let ctxt_parse_node = sink.new_parse_node();
            sink.nodes.insert(ctxt_parse_node.id, JS::from_ref(fc.context_elem));

            let form_parse_node = fc.form_elem.map(|form_elem| {
                let node = sink.new_parse_node();
                sink.nodes.insert(node.id, JS::from_ref(form_elem));
                node
            });
            let tb = TreeBuilder::new_for_fragment(
                sink,
                ctxt_parse_node,
                form_parse_node,
                options);

            let tok_options = TokenizerOpts {
                initial_state: Some(tb.tokenizer_state_for_context_elem()),
                .. Default::default()
            };

            HtmlTokenizer::new(tb, tok_options)
        } else {
            HtmlTokenizer::new(TreeBuilder::new(sink, options), Default::default())
        };

        Tokenizer {
            inner: inner,
        }
    }

    pub fn feed(&mut self, input: &mut BufferQueue) -> Result<(), Root<HTMLScriptElement>> {
        match self.inner.feed(input) {
            TokenizerResult::Done => Ok(()),
            TokenizerResult::Script(script) => {
                let nodes = &self.inner.sink.sink.nodes;
                let script = nodes.get(&script.id).unwrap();
                Err(Root::from_ref(script.downcast().unwrap()))
            },
        }
    }

    pub fn end(&mut self) {
        self.inner.end();
    }

    pub fn url(&self) -> &ServoUrl {
        &self.inner.sink.sink.base_url
    }

    pub fn set_plaintext_state(&mut self) {
        self.inner.set_plaintext_state();
    }
}

#[allow(unsafe_code)]
unsafe impl JSTraceable for HtmlTokenizer<TreeBuilder<ParseNode, Sink>> {
    unsafe fn trace(&self, trc: *mut JSTracer) {
        struct Tracer(*mut JSTracer);
        let tracer = Tracer(trc);

        impl HtmlTracer for Tracer {
            type Handle = ParseNode;
            #[allow(unrooted_must_root)]
            fn trace_handle(&self, node: &ParseNode) {
                unsafe { node.trace(self.0); }
            }
        }

        let tree_builder = &self.sink;
        tree_builder.trace_handles(&tracer);
        tree_builder.sink.trace(trc);
    }
}

type ParseNodeId = usize;

#[derive(JSTraceable, Clone, HeapSizeOf)]
pub struct ParseNode {
    id: ParseNodeId,
    qual_name: Option<QualName>,
}

#[derive(JSTraceable, HeapSizeOf)]
struct ParseNodeData {
    contents: Option<ParseNode>,
    is_integration_point: bool,
}

impl Default for ParseNodeData {
    fn default() -> ParseNodeData {
        ParseNodeData {
            contents: None,
            is_integration_point: false,
        }
    }
}

enum ParseOperation {
    GetTemplateContents(ParseNodeId, ParseNodeId),
    CreateElement(ParseNodeId, QualName, Vec<Attribute>),
    CreateComment(StrTendril, ParseNodeId),
    // sibling, node to be inserted
    AppendBeforeSibling(ParseNodeId, NodeOrText<ParseNode>),
    // parent, node to be inserted
    Append(ParseNodeId, NodeOrText<ParseNode>),
    AppendDoctypeToDocument(StrTendril, StrTendril, StrTendril),
    AddAttrsIfMissing(ParseNodeId, Vec<Attribute>),
    RemoveFromParent(ParseNodeId),
    MarkScriptAlreadyStarted(ParseNodeId),
    ReparentChildren(ParseNodeId, ParseNodeId),
    AssociateWithForm(ParseNodeId, ParseNodeId),
    CreatePI(ParseNodeId, StrTendril, StrTendril),
    Pop(ParseNodeId),
}

#[derive(JSTraceable, HeapSizeOf)]
#[must_root]
pub struct Sink {
    base_url: ServoUrl,
    document: JS<Document>,
    current_line: u64,
    script: MutNullableJS<HTMLScriptElement>,
    parse_node_data: HashMap<ParseNodeId, ParseNodeData>,
    next_parse_node_id: Cell<ParseNodeId>,
    nodes: HashMap<ParseNodeId, JS<Node>>,
    document_node: ParseNode,
}

impl Sink {
    fn new(base_url: ServoUrl, document: &Document) -> Sink {
        let mut sink = Sink {
            base_url: base_url,
            document: JS::from_ref(document),
            current_line: 1,
            script: Default::default(),
            parse_node_data: HashMap::new(),
            next_parse_node_id: Cell::new(1),
            nodes: HashMap::new(),
            document_node: ParseNode {
                id: 0,
                qual_name: None,
            }
        };
        let data = ParseNodeData::default();
        sink.insert_parse_node_data(0, data);
        sink.insert_node(0, JS::from_ref(document.upcast()));
        sink
    }

    fn new_parse_node(&mut self) -> ParseNode {
        let id = self.next_parse_node_id.get();
        let data = ParseNodeData::default();
        self.insert_parse_node_data(id, data);
        self.next_parse_node_id.set(id + 1);
        ParseNode {
            id: id,
            qual_name: None,
        }
    }

    fn insert_node(&mut self, id: ParseNodeId, node: JS<Node>) {
        assert!(self.nodes.insert(id, node).is_none());
    }

    fn get_node<'a>(&'a self, id: &ParseNodeId) -> &'a JS<Node> {
        self.nodes.get(id).expect("Node not found!")
    }

    fn insert_parse_node_data(&mut self, id: ParseNodeId, data: ParseNodeData) {
        assert!(self.parse_node_data.insert(id, data).is_none());
    }

    fn get_parse_node_data<'a>(&'a self, id: &'a ParseNodeId) -> &'a ParseNodeData {
        self.parse_node_data.get(id).expect("Parse Node data not found!")
    }

    fn get_parse_node_data_mut<'a>(&'a mut self, id: &'a ParseNodeId) -> &'a mut ParseNodeData {
        self.parse_node_data.get_mut(id).expect("Parse Node data not found!")
    }

    fn process_operation(&mut self, op: ParseOperation) {
        let document = Root::from_ref(&**self.get_node(&0));
        let document = document.downcast::<Document>().expect("Document node should be downcasted!");
        match op {
            ParseOperation::GetTemplateContents(target, contents) => {
                let target = Root::from_ref(&**self.get_node(&target));
                let template = target.downcast::<HTMLTemplateElement>().expect(
                    "Tried to extract contents from non-template element while parsing");
                self.insert_node(contents, JS::from_ref(template.Content().upcast()));
            }
            ParseOperation::CreateElement(id, name, attrs) => {
                let is = attrs.iter()
                              .find(|attr| attr.name.local.eq_str_ignore_ascii_case("is"))
                              .map(|attr| LocalName::from(&*attr.value));

                let elem = Element::create(name,
                                           is,
                                           &*self.document,
                                           ElementCreator::ParserCreated(self.current_line),
                                           CustomElementCreationMode::Synchronous);
                for attr in attrs {
                    elem.set_attribute_from_parser(attr.name, DOMString::from(String::from(attr.value)), None);
                }

                self.insert_node(id, JS::from_ref(elem.upcast()));
            }
            ParseOperation::CreateComment(text, id) => {
                let comment = Comment::new(DOMString::from(String::from(text)), document);
                self.insert_node(id, JS::from_ref(&comment.upcast()));
            }
            ParseOperation::AppendBeforeSibling(sibling, node) => {
                let node = match node {
                    NodeOrText::AppendNode(n) => NodeOrText::AppendNode(JS::from_ref(&**self.get_node(&n.id))),
                    NodeOrText::AppendText(text) => NodeOrText::AppendText(text)
                };
                let sibling = &**self.get_node(&sibling);
                let parent = &*sibling.GetParentNode().expect("append_before_sibling called on node without parent");

                super::insert(parent, Some(sibling), node);
            }
            ParseOperation::Append(parent, node) => {
                let node = match node {
                    NodeOrText::AppendNode(n) => NodeOrText::AppendNode(JS::from_ref(&**self.get_node(&n.id))),
                    NodeOrText::AppendText(text) => NodeOrText::AppendText(text)
                };

                let parent = &**self.get_node(&parent);
                super::insert(parent, None, node);
            }
            ParseOperation::AppendDoctypeToDocument(name, public_id, system_id) => {
                let doctype = DocumentType::new(
                    DOMString::from(String::from(name)), Some(DOMString::from(String::from(public_id))),
                    Some(DOMString::from(String::from(system_id))), document);

                document.upcast::<Node>().AppendChild(doctype.upcast()).expect("Appending failed");
            }
            ParseOperation::AddAttrsIfMissing(target_id, attrs) => {
                let elem = self.get_node(&target_id).downcast::<Element>()
                    .expect("tried to set attrs on non-Element in HTML parsing");
                for attr in attrs {
                    elem.set_attribute_from_parser(attr.name, DOMString::from(String::from(attr.value)), None);
                }
            }
            ParseOperation::RemoveFromParent(target) => {
                if let Some(ref parent) = self.get_node(&target).GetParentNode() {
                    parent.RemoveChild(&**self.get_node(&target)).unwrap();
                }
            }
            ParseOperation::MarkScriptAlreadyStarted(node) => {
                let script = self.get_node(&node).downcast::<HTMLScriptElement>();
                script.map(|script| script.set_already_started(true));
            }
            ParseOperation::ReparentChildren(parent, new_parent) => {
                let parent = self.get_node(&parent);
                let new_parent = self.get_node(&new_parent);
                while let Some(child) = parent.GetFirstChild() {
                    new_parent.AppendChild(&child).unwrap();
                }
            }
            ParseOperation::AssociateWithForm(target, form) => {
                let form = self.get_node(&form);
                let form = Root::downcast::<HTMLFormElement>(Root::from_ref(&**form))
                    .expect("Owner must be a form element");

                let node = self.get_node(&target);
                let elem = node.downcast::<Element>();
                let control = elem.and_then(|e| e.as_maybe_form_control());

                if let Some(control) = control {
                    control.set_form_owner_from_parser(&form);
                } else {
                    // TODO remove this code when keygen is implemented.
                    assert!(node.NodeName() == "KEYGEN", "Unknown form-associatable element");
                }
            }
            ParseOperation::Pop(node) => {
                vtable_for(self.get_node(&node)).pop();
            }
            ParseOperation::CreatePI(node, target, data) => {
                let pi = ProcessingInstruction::new(
                        DOMString::from(String::from(target)),
                        DOMString::from(String::from(data)),
                        document);
                self.insert_node(node, JS::from_ref(pi.upcast()));
            }
        }
    }
}

#[allow(unrooted_must_root)]
impl TreeSink for Sink {
    type Output = Self;
    fn finish(self) -> Self { self }

    type Handle = ParseNode;

    fn get_document(&mut self) -> Self::Handle {
        self.document_node.clone()
    }

    fn get_template_contents(&mut self, target: &Self::Handle) -> Self::Handle {
        if let Some(ref contents) = self.get_parse_node_data(&target.id).contents {
            return contents.clone();
        }
        let node = self.new_parse_node();
        {
            let mut data = self.get_parse_node_data_mut(&target.id);
            data.contents = Some(node.clone());
        }
        self.process_operation(ParseOperation::GetTemplateContents(target.id, node.id));
        node
    }

    fn same_node(&self, x: &Self::Handle, y: &Self::Handle) -> bool {
        x.id == y.id
    }

    fn elem_name<'a>(&self, target: &'a Self::Handle) -> ExpandedName<'a> {
        target.qual_name.as_ref().expect("Expected qual name of node!").expanded()
    }

    fn same_tree(&self, x: &Self::Handle, y: &Self::Handle) -> bool {
        let x = self.get_node(&x.id);
        let y = self.get_node(&y.id);

        let x = x.downcast::<Element>().expect("Element node expected");
        let y = y.downcast::<Element>().expect("Element node expected");
        x.is_in_same_home_subtree(y)
    }

    fn create_element(&mut self, name: QualName, attrs: Vec<Attribute>, _flags: ElementFlags)
        -> Self::Handle {
        let mut node = self.new_parse_node();
        node.qual_name = Some(name.clone());
        {
            let mut node_data = self.get_parse_node_data_mut(&node.id);
            node_data.is_integration_point = attrs.iter()
            .any(|attr| {
                let attr_value = &String::from(attr.value.clone());
                (attr.name.local == local_name!("encoding") && attr.name.ns == ns!()) &&
                (attr_value.eq_ignore_ascii_case("text/html") ||
                attr_value.eq_ignore_ascii_case("application/xhtml+xml"))
            });
        }
        self.process_operation(ParseOperation::CreateElement(node.id, name, attrs));
        node
    }

    fn create_comment(&mut self, text: StrTendril) -> Self::Handle {
        let node = self.new_parse_node();
        self.process_operation(ParseOperation::CreateComment(text, node.id));
        node
    }

    fn create_pi(&mut self, target: StrTendril, data: StrTendril) -> ParseNode {
        let node = self.new_parse_node();
        self.process_operation(ParseOperation::CreatePI(node.id, target, data));
        node
    }

    fn has_parent_node(&self, node: &Self::Handle) -> bool {
        self.get_node(&node.id).GetParentNode().is_some()
    }

    fn associate_with_form(&mut self, target: &Self::Handle, form: &Self::Handle) {
        self.process_operation(ParseOperation::AssociateWithForm(target.id, form.id));
    }

    fn append_before_sibling(&mut self,
                             sibling: &Self::Handle,
                             new_node: NodeOrText<Self::Handle>) {
        self.process_operation(ParseOperation::AppendBeforeSibling(sibling.id, new_node));
    }

    fn parse_error(&mut self, msg: Cow<'static, str>) {
        debug!("Parse error: {}", msg);
    }

    fn set_quirks_mode(&mut self, mode: QuirksMode) {
        let mode = match mode {
            QuirksMode::Quirks => ServoQuirksMode::Quirks,
            QuirksMode::LimitedQuirks => ServoQuirksMode::LimitedQuirks,
            QuirksMode::NoQuirks => ServoQuirksMode::NoQuirks,
        };
        self.document.set_quirks_mode(mode);
    }

    fn append(&mut self, parent: &Self::Handle, child: NodeOrText<Self::Handle>) {
        self.process_operation(ParseOperation::Append(parent.id, child));
    }

    fn append_doctype_to_document(&mut self, name: StrTendril, public_id: StrTendril,
                                  system_id: StrTendril) {
        self.process_operation(ParseOperation::AppendDoctypeToDocument(name, public_id, system_id));
    }

    fn add_attrs_if_missing(&mut self, target: &Self::Handle, attrs: Vec<Attribute>) {
        self.process_operation(ParseOperation::AddAttrsIfMissing(target.id, attrs));
    }

    fn remove_from_parent(&mut self, target: &Self::Handle) {
        self.process_operation(ParseOperation::RemoveFromParent(target.id));
    }

    fn mark_script_already_started(&mut self, node: &Self::Handle) {
        self.process_operation(ParseOperation::MarkScriptAlreadyStarted(node.id));
    }

    fn complete_script(&mut self, _: &Self::Handle) -> NextParserState {
        panic!("complete_script should not be called here!");
    }

    fn reparent_children(&mut self, parent: &Self::Handle, new_parent: &Self::Handle) {
        self.process_operation(ParseOperation::ReparentChildren(parent.id, new_parent.id));
    }

    /// https://html.spec.whatwg.org/multipage/#html-integration-point
    /// Specifically, the <annotation-xml> cases.
    fn is_mathml_annotation_xml_integration_point(&self, handle: &Self::Handle) -> bool {
        let node_data = self.get_parse_node_data(&handle.id);
        node_data.is_integration_point
    }

    fn set_current_line(&mut self, line_number: u64) {
        self.current_line = line_number;
    }

    fn pop(&mut self, node: &Self::Handle) {
        self.process_operation(ParseOperation::Pop(node.id));
    }
}
