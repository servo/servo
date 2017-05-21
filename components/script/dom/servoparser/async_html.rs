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
    is_integration_point: bool,
}

impl Default for ParseNodeData {
    fn default() -> ParseNodeData {
        ParseNodeData {
            parent: None,
            target: None,
            data: None,
            contents: None,
            is_integration_point: false,
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
        let data: ParseNodeData = Default::default();
        assert!(sink.parse_node_data.borrow_mut().insert(0, data).is_none());
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
        let data;
        {
            let parse_node_data = self.parse_node_data.borrow();
            data = parse_node_data.get(&target.id).expect("Expected data of parse node").contents.clone();
        }
        if data.is_some() {
            return data.clone().unwrap();
        }

        let node = self.new_parse_node();
        let mut parse_node_data = self.parse_node_data.borrow_mut();
        let mut data = parse_node_data.get_mut(&target.id).expect("Expected data of parse node");
        data.contents = Some(node.clone());
        self.enqueue(ParseOperation::GetTemplateContents(target.id, node.id));
        node
    }

    fn same_node(&self, x: &Self::Handle, y: &Self::Handle) -> bool {
        x.id == y.id
    }

    fn elem_name<'a>(&self, target: &'a Self::Handle) -> ExpandedName<'a> {
        match target.qual_name {
            Some(ref qual_name) => qual_name.expanded(),
            None => panic!("Expected qual name of node!"),
        }
    }

    fn same_tree(&self, x: &Self::Handle, y: &Self::Handle) -> bool {
        let nodes = self.nodes.borrow();
        let (id_x, id_y) = (x.id, y.id);

        let x = nodes.get(&id_x).expect("Node not found");
        // Update x's parent if no longer in tree
        if !x.is_in_doc() {
            self.parse_node_data.borrow_mut().get_mut(&id_x).expect("Node data not found").parent = None;
        }

        let y = nodes.get(&id_y).expect("Node not found");
        // Update y's parent if no longer in tree
        if !y.is_in_doc() {
            self.parse_node_data.borrow_mut().get_mut(&id_y).expect("Node data not found").parent = None;
        }

        let x = x.downcast::<Element>().expect("Element node expected");
        let y = y.downcast::<Element>().expect("Element node expected");
        x.is_in_same_home_subtree(y)
    }

    fn create_element(&mut self, name: QualName, attrs: Vec<Attribute>, _flags: ElementFlags)
        -> Self::Handle {
        let mut node = self.new_parse_node();
        node.qual_name = Some(name.clone());

        let mut parse_node_data = self.parse_node_data.borrow_mut();
        let mut node_data = parse_node_data.get_mut(&node.id).expect("Element does not exist!");
        node_data.is_integration_point = attrs.iter()
        .any(|attr| {
            let attr_value = &String::from(attr.value.clone());
            (attr.name.local == local_name!("encoding") && attr.name.ns == ns!()) &&
            (attr_value.eq_ignore_ascii_case("text/html") ||
            attr_value.eq_ignore_ascii_case("application/xhtml+xml"))
        });
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
        let mut parse_node_data = self.parse_node_data.borrow_mut();
        let parent_id;
        {
            let data = parse_node_data.get(&sibling.id).expect("Element does not exist!");
            if !data.parent.is_some() {
                panic!("append_before_sibling called on node without parent!");
            }
            parent_id = data.parent.unwrap();
        }

        // Get sibling's parent and set new_node's parent
        if let NodeOrText::AppendNode(ref n) = new_node {
            parse_node_data.get_mut(&n.id).unwrap().parent = Some(parent_id);
        }

        let new_node = match new_node {
            NodeOrText::AppendNode(node) => ParserNodeOrText::Node(node.id),
            NodeOrText::AppendText(text) => ParserNodeOrText::Text(text.into()),
        };

        self.enqueue(ParseOperation::Insert(parent_id, Some(sibling.id), new_node));
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
            NodeOrText::AppendNode(node) => {
                let mut parse_node_data = self.parse_node_data.borrow_mut();
                let data = parse_node_data.get_mut(&node.id).expect("Element does not exist!");
                data.parent = Some(parent.id);
                ParserNodeOrText::Node(node.id)
            },
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

    fn complete_script(&mut self, _: &Self::Handle) -> NextParserState {
        panic!("complete_script should not be called here!");
    }

    fn reparent_children(&mut self, parent: &Self::Handle, new_parent: &Self::Handle) {
        for data in self.parse_node_data.borrow_mut().values_mut() {
            if data.parent == Some(parent.id) {
                data.parent = Some(new_parent.id);
            }
        }
        self.enqueue(ParseOperation::ReparentChildren(parent.id, new_parent.id));
    }

    /// https://html.spec.whatwg.org/multipage/#html-integration-point
    /// Specifically, the <annotation-xml> cases.
    fn is_mathml_annotation_xml_integration_point(&self, handle: &Self::Handle) -> bool {
        let parse_node_data = self.parse_node_data.borrow();
        let node_data = parse_node_data.get(&handle.id).expect("Element does not exist!");
        node_data.is_integration_point
    }

    fn set_current_line(&mut self, line_number: u64) {
        self.current_line = line_number;
    }

    fn pop(&mut self, node: &Self::Handle) {
        self.enqueue(ParseOperation::Pop(node.id));
    }
}
