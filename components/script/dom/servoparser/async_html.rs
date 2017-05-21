/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![allow(unrooted_must_root)]

use core::ops::Deref;
use core::str::FromStr;
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::HTMLTemplateElementBinding::HTMLTemplateElementMethods;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{JS, MutNullableJS, Root};
use dom::bindings::str::DOMString;
use dom::bindings::trace::JSTraceable;
use dom::comment::Comment;
use dom::document::Document;
use dom::documenttype::DocumentType;
use dom::element::{Element, ElementCreator};
use dom::htmlformelement::{FormControlElementHelpers, HTMLFormElement};
use dom::htmlscriptelement::HTMLScriptElement;
use dom::htmltemplateelement::HTMLTemplateElement;
use dom::node::Node;
use dom::processinginstruction::ProcessingInstruction;
use dom::virtualmethods::vtable_for;
use html5ever::{Attribute, QualName, ExpandedName};
use html5ever::buffer_queue::BufferQueue;
use html5ever::tendril::{Tendril, StrTendril};
use html5ever::tokenizer::{Tokenizer as HtmlTokenizer, TokenizerOpts, TokenizerResult};
use html5ever::tree_builder::{NodeOrText, TreeSink, NextParserState, QuirksMode, ElementFlags};
use html5ever::tree_builder::{Tracer as HtmlTracer, TreeBuilder, TreeBuilderOpts};
use js::jsapi::JSTracer;
use servo_url::ServoUrl;
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
        let sink = Sink::new(url, document);

        let options = TreeBuilderOpts {
            ignore_missing_rules: true,
            .. Default::default()
        };

        let inner = if let Some(fc) = fragment_context {
            let ctxt_parse_node = sink.new_parse_node();
            sink.nodes.borrow_mut().insert(ctxt_parse_node.id, JS::from_ref(fc.context_elem));

            let form_parse_node = fc.form_elem.map(|form_elem| {
                let node = sink.new_parse_node();
                sink.nodes.borrow_mut().insert(node.id, JS::from_ref(form_elem));
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
                let nodes = self.inner.sink.sink.nodes.borrow();
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

#[derive(JSTraceable, Clone, HeapSizeOf)]
pub struct ParseNode {
    id: usize,
    qual_name: Option<QualName>,
}

enum ParserNodeOrText {
    Node(usize),
    Text(String),
}

#[derive(JSTraceable, HeapSizeOf)]
struct ParseNodeData {
    parent: Option<usize>,
    target: Option<String>,
    data: Option<String>,
    contents: Option<ParseNode>,
}

impl Default for ParseNodeData {
    fn default() -> ParseNodeData {
        ParseNodeData {
            parent: None,
            target: None,
            data: None,
            contents: None,
        }
    }
}

enum ParseOperation {
    GetTemplateContents(usize, usize),
    CreateElement(usize, QualName, Vec<Attribute>),
    CreateComment(String, usize),
    Insert(usize, Option<usize>, ParserNodeOrText),
    AppendDoctypeToDocument(String, String, String),
    AddAttrsIfMissing(usize, Vec<Attribute>),
    RemoveFromParent(usize),
    MarkScriptAlreadyStarted(usize),
    CompleteScript(usize),
    ReparentChildren(usize, usize),
    AssociateWithForm(usize, usize),
    CreatePI(usize),
    Pop(usize),
}

#[derive(JSTraceable, HeapSizeOf)]
#[must_root]
pub struct Sink {
    base_url: ServoUrl,
    document: JS<Document>,
    current_line: u64,
    script: MutNullableJS<HTMLScriptElement>,
    parse_node_data: DOMRefCell<HashMap<usize, ParseNodeData>>,
    next_parse_node_id: Cell<usize>,
    nodes: DOMRefCell<HashMap<usize, JS<Node>>>,
    document_node: ParseNode,
}

impl Sink {
    fn new(base_url: ServoUrl, document: &Document) -> Sink {
        let sink = Sink {
            base_url: base_url,
            document: JS::from_ref(document),
            current_line: 1,
            script: Default::default(),
            parse_node_data: DOMRefCell::new(HashMap::new()),
            next_parse_node_id: Cell::new(1),
            nodes: DOMRefCell::new(HashMap::new()),
            document_node: ParseNode {
                id: 0,
                qual_name: None,
            }
        };
        sink.nodes.borrow_mut().insert(0, JS::from_ref(document.upcast()));
        sink
    }

    fn new_parse_node(&self) -> ParseNode {
        let id = self.next_parse_node_id.get();
        let data: ParseNodeData = Default::default();
        assert!(self.parse_node_data.borrow_mut().insert(id, data).is_none());
        self.next_parse_node_id.set(id + 1);
        ParseNode {
            id: id,
            qual_name: None,
        }
    }

    fn enqueue(&self, op: ParseOperation) {
        self.process_operation(op);
    }

    fn root(&self, node: &ParseNode) -> usize {
        let mut id = node.id;
        while let Some(parent_id) = self.parse_node_data.borrow().get(&id)
                                    .expect("self.parse_node_data should exist!").parent {
            id = parent_id
        }
        id
    }

    fn process_operation(&self, op: ParseOperation) {
        let document = Root::from_ref(&**self.nodes.borrow().get(&0).expect("document node should exist!"));
        let document = document.downcast::<Document>().expect("Document node should be downcasted!");
        match op {
            ParseOperation::GetTemplateContents(target, contents) => {
                let target = Root::from_ref(&**self.nodes.borrow().get(&target).unwrap());
                let template = target.downcast::<HTMLTemplateElement>().expect(
                    "Tried to extract contents from non-template element while parsing");
                assert!(self.nodes.borrow_mut().insert(contents, JS::from_ref(template.Content().upcast())).is_none());
            }
            ParseOperation::CreateElement(id, name, attrs) => {
                let elem = Element::create(name, &*self.document,
                                           ElementCreator::ParserCreated(self.current_line));
                for attr in attrs {
                    elem.set_attribute_from_parser(attr.name, DOMString::from(String::from(attr.value)), None);
                }

                self.nodes.borrow_mut().insert(id, JS::from_ref(elem.upcast()));
            }
            ParseOperation::CreateComment(text, id) => {
                let comment = Comment::new(DOMString::from(text), document);
                assert!(self.nodes.borrow_mut().insert(id, JS::from_ref(&comment.upcast())).is_none());
            }
            ParseOperation::Insert(parent, reference_child, new_node) => {
                let nodes = self.nodes.borrow();
                let parent = nodes.get(&parent).unwrap();
                let reference_child = reference_child.map(|n| nodes.get(&n).unwrap().deref());

                let node = match new_node {
                    ParserNodeOrText::Node(ref id) => {
                        let deref_node = nodes.get(id).unwrap().deref();
                        NodeOrText::AppendNode(JS::from_ref(deref_node))
                    },
                    ParserNodeOrText::Text(ref string) => {
                        match Tendril::from_str(string) {
                            Ok(tendril) => NodeOrText::AppendText(tendril),
                            Err(_) => panic!("Error while converting from string to tendril!"),
                        }
                    }
                };

                super::insert(parent, reference_child, node);
            }
            ParseOperation::AppendDoctypeToDocument(name, public_id, system_id) => {
                let doctype = DocumentType::new(
                    DOMString::from(name), Some(DOMString::from(public_id)),
                    Some(DOMString::from(system_id)), document);

                document.upcast::<Node>().AppendChild(doctype.upcast()).expect("Appending failed");
            }
            ParseOperation::AddAttrsIfMissing(target_id, attrs) => {
                match self.nodes.borrow().get(&target_id) {
                    Some(target) => {
                        let elem = target.downcast::<Element>()
                            .expect("tried to set attrs on non-Element in HTML parsing");
                        for attr in attrs {
                            elem.set_attribute_from_parser(attr.name, DOMString::from(String::from(attr.value)), None);
                        }
                    },
                    None => panic!("Node not found!")
                }
            }
            ParseOperation::RemoveFromParent(target) => {
                match self.nodes.borrow().get(&target) {
                    Some(target) => {
                        if let Some(ref parent) = target.GetParentNode() {
                            parent.RemoveChild(&*target).unwrap();
                        }
                    }
                    None => panic!("Node not found!")
                }
            }
            ParseOperation::MarkScriptAlreadyStarted(node) => {
                match self.nodes.borrow().get(&node) {
                    Some(node) => {
                        let script = node.downcast::<HTMLScriptElement>();
                        script.map(|script| script.set_already_started(true));
                    }
                    None => panic!("Node not found!")
                }
            }
            ParseOperation::CompleteScript(node) => {
            }
            ParseOperation::ReparentChildren(parent, new_parent) => {
                let nodes = self.nodes.borrow();
                let parent = nodes.get(&parent).unwrap();
                let new_parent = nodes.get(&new_parent).unwrap();
                while let Some(ref child) = parent.GetFirstChild() {
                    new_parent.AppendChild(&child).unwrap();
                }
            }
            ParseOperation::AssociateWithForm(target, form) => {
                let nodes = self.nodes.borrow();
                let node = nodes.get(&target).unwrap();
                let form = nodes.get(&form).unwrap();

                let form = Root::downcast::<HTMLFormElement>(Root::from_ref(&**form))
                    .expect("Owner must be a form element");

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
                let nodes = self.nodes.borrow();
                let node = nodes.get(&node).unwrap();
                let node = Root::from_ref(&**node);
                vtable_for(&node).pop();
            }
            ParseOperation::CreatePI(node) => {
                let parse_node_data = self.parse_node_data.borrow();
                let data = parse_node_data.get(&node).unwrap();
                let pi = ProcessingInstruction::new(
                    DOMString::from(data.target.clone().unwrap()),
                    DOMString::from(data.data.clone().unwrap()),
                    document);
                self.nodes.borrow_mut().insert(node, JS::from_ref(pi.upcast()));
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
        let mut parse_node_data = self.parse_node_data.borrow_mut();
        let mut data = parse_node_data.get_mut(&target.id).expect("Expected data of parse node");
        let contents = match data.contents {
            Some(ref node) => node.clone(),
            None => {
                let node = self.new_parse_node();
                data.contents = Some(node.clone());
                node
            }
        };
        self.enqueue(ParseOperation::GetTemplateContents(target.id, contents.id));
        contents
    }

    fn same_node(&self, x: &Self::Handle, y: &Self::Handle) -> bool {
        x.id == y.id
    }

    fn elem_name<'a>(&self, target: &'a Self::Handle) -> ExpandedName<'a> {
        match target.qual_name {
            Some(ref qual_name) => ExpandedName {
                                       ns: &qual_name.ns,
                                       local: &qual_name.local,
                                   },
            None => panic!("Expected qual name of node!"),
        }
    }

    fn same_tree(&self, x: &Self::Handle, y: &Self::Handle) -> bool {
        self.root(&x) == self.root(&y)
    }

    fn create_element(&mut self, name: QualName, attrs: Vec<Attribute>, _flags: ElementFlags)
        -> Self::Handle {
        let mut node = self.new_parse_node();
        node.qual_name = Some(name.clone());
        self.enqueue(ParseOperation::CreateElement(node.id, name, attrs));
        node
    }

    fn create_comment(&mut self, text: StrTendril) -> Self::Handle {
        let node = self.new_parse_node();
        self.enqueue(ParseOperation::CreateComment(String::from(text), node.id));
        node
    }

    fn create_pi(&mut self, target: StrTendril, data: StrTendril) -> ParseNode {
        let node = self.new_parse_node();
        let mut parse_node_data = self.parse_node_data.borrow_mut();
        let mut node_data = parse_node_data.get_mut(&node.id).expect("Element does not exist!");
        node_data.target = Some(String::from(target));
        node_data.data = Some(String::from(data));
        self.enqueue(ParseOperation::CreatePI(node.id));
        node
    }

    fn has_parent_node(&self, node: &Self::Handle) -> bool {
        let parse_node_data = self.parse_node_data.borrow();
        let data = parse_node_data.get(&node.id).expect("Element does not exist!");
        data.parent.is_some()
    }

    fn associate_with_form(&mut self, target: &Self::Handle, form: &Self::Handle) {
        self.enqueue(ParseOperation::AssociateWithForm(target.id, form.id));
    }

    fn append_before_sibling(&mut self,
            sibling: &Self::Handle,
            new_node: NodeOrText<Self::Handle>) {
        match self.parse_node_data.borrow().get(&sibling.id) {
            Some(ref data) => {
                match data.parent {
                    Some(ref parent_id) => {
                        // Get sibling's parent and set new_node's parent
                        if let NodeOrText::AppendNode(ref n) = new_node {
                            self.parse_node_data.borrow_mut().get_mut(&n.id).unwrap().parent = Some(parent_id.clone());
                        }

                        let new_node = match new_node {
                            NodeOrText::AppendNode(node) => ParserNodeOrText::Node(node.id),
                            NodeOrText::AppendText(text) => ParserNodeOrText::Text(text.into()),
                        };

                        self.enqueue(ParseOperation::Insert(*parent_id, Some(sibling.id), new_node));
                    },
                    None => panic!("append_before_sibling called on node without parent!"),
                }
            },
            None => panic!("Element does not exist!"),
        }
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
        let new_node = match child {
            NodeOrText::AppendNode(node) => ParserNodeOrText::Node(node.id),
            NodeOrText::AppendText(text) => ParserNodeOrText::Text(text.into()),
        };
        self.enqueue(ParseOperation::Insert(parent.id, None, new_node));
    }

    fn append_doctype_to_document(&mut self, name: StrTendril, public_id: StrTendril,
                                  system_id: StrTendril) {
        self.enqueue(ParseOperation::AppendDoctypeToDocument(name.into(), public_id.into(), system_id.into()));
    }

    fn add_attrs_if_missing(&mut self, target: &Self::Handle, attrs: Vec<Attribute>) {
        self.enqueue(ParseOperation::AddAttrsIfMissing(target.id, attrs));
    }

    fn remove_from_parent(&mut self, target: &Self::Handle) {
        let mut parse_node_data = self.parse_node_data.borrow_mut();
        let data = parse_node_data.get_mut(&target.id).expect("Element does not exist!");
        data.parent = None;
        self.enqueue(ParseOperation::RemoveFromParent(target.id));
    }

    fn mark_script_already_started(&mut self, node: &Self::Handle) {
        self.enqueue(ParseOperation::MarkScriptAlreadyStarted(node.id));
    }

    // TODO: Handle this
    fn complete_script(&mut self, node: &Self::Handle) -> NextParserState {
        // if let Some(script) = node.downcast() {
        //     self.script.set(Some(script));
        //     NextParserState::Suspend
        // } else {
        //     NextParserState::Continue
        // }
        NextParserState::Suspend
    }

    fn reparent_children(&mut self, node: &Self::Handle, new_parent: &Self::Handle) {
        for data in self.parse_node_data.borrow_mut().values_mut() {
            if data.parent == Some(node.id) {
                data.parent = Some(new_parent.id);
            }
        }
        self.enqueue(ParseOperation::ReparentChildren(node.id, new_parent.id));
    }

    /// https://html.spec.whatwg.org/multipage/#html-integration-point
    /// Specifically, the <annotation-xml> cases.
    fn is_mathml_annotation_xml_integration_point(&self, handle: &Self::Handle) -> bool {
        // let elem = handle.downcast::<Element>().unwrap();
        // elem.get_attribute(&ns!(), &local_name!("encoding")).map_or(false, |attr| {
        //     attr.value().eq_ignore_ascii_case("text/html")
        //         || attr.value().eq_ignore_ascii_case("application/xhtml+xml")
        // })
        true
    }

    fn set_current_line(&mut self, line_number: u64) {
        self.current_line = line_number;
    }

    fn pop(&mut self, node: &Self::Handle) {
        self.enqueue(ParseOperation::Pop(node.id));
    }
}
