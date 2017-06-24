/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![allow(unrooted_must_root)]

use core::iter::FromIterator;
use core::str::FromStr;
use dom::bindings::codegen::Bindings::HTMLTemplateElementBinding::HTMLTemplateElementMethods;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{JS, Root};
use dom::bindings::str::DOMString;
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
use html5ever::{Attribute as HtmlAttribute, ExpandedName, LocalName, QualName};
use html5ever::buffer_queue::BufferQueue;
use html5ever::tendril::{SendTendril, StrTendril, Tendril};
use html5ever::tendril::fmt::UTF8;
use html5ever::tokenizer::{Tokenizer as HtmlTokenizer, TokenizerOpts, TokenizerResult};
use html5ever::tree_builder::{ElementFlags, NodeOrText as HtmlNodeOrText, NextParserState, QuirksMode, TreeSink};
use html5ever::tree_builder::{TreeBuilder, TreeBuilderOpts};
use servo_url::ServoUrl;
use std::ascii::AsciiExt;
use std::borrow::Cow;
use std::cell::Cell;
use std::collections::HashMap;
use std::collections::vec_deque::VecDeque;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use style::context::QuirksMode as ServoQuirksMode;

type ParseNodeId = usize;

#[derive(Clone, HeapSizeOf, JSTraceable)]
pub struct ParseNode {
    id: ParseNodeId,
    qual_name: Option<QualName>,
}

#[derive(HeapSizeOf, JSTraceable)]
enum NodeOrText {
    Node(ParseNode),
    Text(String),
}

#[derive(HeapSizeOf, JSTraceable)]
struct Attribute {
    name: QualName,
    value: String,
}

#[derive(HeapSizeOf, JSTraceable)]
enum ParseOperation {
    GetTemplateContents(ParseNodeId, ParseNodeId),
    CreateElement(ParseNodeId, QualName, Vec<Attribute>, u64),
    CreateComment(String, ParseNodeId),
    // sibling, node to be inserted
    AppendBeforeSibling(ParseNodeId, NodeOrText),
    // parent, node to be inserted
    Append(ParseNodeId, NodeOrText),
    AppendDoctypeToDocument(String, String, String),
    AddAttrsIfMissing(ParseNodeId, Vec<Attribute>),
    RemoveFromParent(ParseNodeId),
    MarkScriptAlreadyStarted(ParseNodeId),
    ReparentChildren(ParseNodeId, ParseNodeId),
    AssociateWithForm(ParseNodeId, ParseNodeId),
    CreatePI(ParseNodeId, String, String),
    Pop(ParseNodeId),
    SetQuirksMode(
        #[ignore_heap_size_of = "Defined in style"]
        ServoQuirksMode
    ),
}

#[derive(HeapSizeOf)]
enum FromHtmlTokenizerMsg {
    Result(#[ignore_heap_size_of = "Defined in html5ever"]TokenizerResult<ParseNode>)
}

#[derive(HeapSizeOf)]
enum ToHtmlTokenizerMsg {
    Feed(#[ignore_heap_size_of = "Defined in html5ever"]Arc<Mutex<Vec<SendTendril<UTF8>>>>),
    End,
    SetPlainTextState,
}

#[derive(HeapSizeOf)]
enum FromSinkMsg {
    ProcessOperation(ParseOperation),
    IsSameTree(ParseNodeId, ParseNodeId),
    HasParentNode(ParseNodeId),
}

#[derive(HeapSizeOf)]
enum ToSinkMsg {
    IsSameTree(bool),
    HasParentNode(bool),
}

fn create_buffer_queue(mut buffers: VecDeque<StrTendril>) -> BufferQueue {
    let mut buffer_queue = BufferQueue::new();
    while let Some(s) = buffers.pop_front() {
        buffer_queue.push_back(s);
    }
    buffer_queue
}

#[derive(HeapSizeOf, JSTraceable)]
#[must_root]
pub struct Tokenizer {
    document: JS<Document>,
    #[ignore_heap_size_of = "Defined in std"]
    html_tokenizer_receiver: Receiver<FromHtmlTokenizerMsg>,
    #[ignore_heap_size_of = "Defined in std"]
    html_tokenizer_sender: Sender<ToHtmlTokenizerMsg>,
    #[ignore_heap_size_of = "Defined in std"]
    sink_receiver: Receiver<FromSinkMsg>,
    #[ignore_heap_size_of = "Defined in std"]
    sink_sender: Sender<ToSinkMsg>,
    nodes: HashMap<ParseNodeId, JS<Node>>,
    url: ServoUrl,
}

impl Tokenizer {
    pub fn new(
            document: &Document,
            url: ServoUrl,
            fragment_context: Option<super::FragmentContext>)
            -> Self {
        // Messages from the Tokenizer (main thread) to HtmlTokenizer (parser thread)
        let (to_html_tokenizer_sender, html_tokenizer_receiver) = channel();
        // Messages from the HtmlTokenizer (parser thread) to Tokenizer (main thread)
        let (html_tokenizer_sender, from_html_tokenizer_receiver) = channel();
        // Messages from the Tokenizer (main thread) to Sink (parser thread)
        let (to_sink_sender, sink_receiver) = channel();
        // Messages from the Sink (parser thread) to Tokenizer (main thread)
        let (sink_sender, from_sink_receiver) = channel();

        let mut tokenizer = Tokenizer {
            document: JS::from_ref(document),
            html_tokenizer_receiver: from_html_tokenizer_receiver,
            html_tokenizer_sender: to_html_tokenizer_sender,
            sink_receiver: from_sink_receiver,
            sink_sender: to_sink_sender,
            nodes: HashMap::new(),
            url: url
        };
        tokenizer.insert_node(0, JS::from_ref(document.upcast()));

        let mut sink = Sink::new(sink_sender, sink_receiver);
        let mut ctxt_parse_node = None;
        let mut form_parse_node = None;
        let mut fragment_context_is_some = false;
        if let Some(fc) = fragment_context {
            let node = sink.new_parse_node();
            tokenizer.insert_node(node.id, JS::from_ref(fc.context_elem));
            ctxt_parse_node = Some(node);

            form_parse_node = fc.form_elem.map(|form_elem| {
                let node = sink.new_parse_node();
                tokenizer.insert_node(node.id, JS::from_ref(form_elem));
                node
            });
            fragment_context_is_some = true;
        };

        // Create new thread for HtmlTokenizer. This is where parser actions
        // will be generated from the input provided. These parser actions are then passed
        // onto the main thread to be executed.
        thread::Builder::new().name(String::from("HTML Parser")).spawn(move || {
            run(sink,
                fragment_context_is_some,
                ctxt_parse_node,
                form_parse_node,
                html_tokenizer_sender,
                html_tokenizer_receiver);
        }).expect("HTML Parser thread spawning failed");

        tokenizer
    }

    #[allow(unsafe_code)]
    pub fn feed(&mut self, input: &mut BufferQueue) -> Result<(), Root<HTMLScriptElement>> {
        enum Message {
            Sink(FromSinkMsg),
            HtmlTokenizer(FromHtmlTokenizerMsg)
        }

        let mut send_tendrils = Vec::new();
        while let Some(str) = input.pop_front() {
            send_tendrils.push(SendTendril::from(str));
        }
        let send_tendrils = Arc::new(Mutex::new(send_tendrils));
        // Send message to parser thread, asking it to started reading from the shared input.
        // Parser operation messages will be sent to main thread as they are evaluated.
        self.html_tokenizer_sender.send(ToHtmlTokenizerMsg::Feed(send_tendrils.clone())).unwrap();

        loop {
            let message = {
                let sink_receiver = &self.sink_receiver;
                let html_tokenizer_receiver = &self.html_tokenizer_receiver;
                select! {
                msg = sink_receiver.recv() =>
                    Message::Sink(msg.expect("Unexpected sink channel panic in script")),
                msg = html_tokenizer_receiver.recv() =>
                    Message::HtmlTokenizer(msg.expect("Unexpected html tokenizer channel panic in script"))
                }
            };

            match message {
                Message::Sink(FromSinkMsg::ProcessOperation(parse_op)) => {
                    self.process_operation(parse_op)
                },
                Message::Sink(FromSinkMsg::IsSameTree(ref x_id, ref y_id)) => {
                    let x = self.get_node(x_id);
                    let y = self.get_node(y_id);

                    let x = x.downcast::<Element>().expect("Element node expected");
                    let y = y.downcast::<Element>().expect("Element node expected");
                    self.sink_sender.send(ToSinkMsg::IsSameTree(x.is_in_same_home_subtree(y))).unwrap();
                },
                Message::Sink(FromSinkMsg::HasParentNode(ref id)) => {
                    let res = self.get_node(id).GetParentNode().is_some();
                    self.sink_sender.send(ToSinkMsg::HasParentNode(res)).unwrap();
                },
                Message::HtmlTokenizer(FromHtmlTokenizerMsg::Result(TokenizerResult::Done)) => return Ok(()),
                Message::HtmlTokenizer(FromHtmlTokenizerMsg::Result(TokenizerResult::Script(script))) => {
                    let script = self.get_node(&script.id);
                    let buffers = (*send_tendrils.lock().unwrap()).iter().
                        map(|st| StrTendril::from(st.clone())).collect::<VecDeque<StrTendril>>();
                    let buffer_queue = create_buffer_queue(buffers);
                    *input = buffer_queue;
                    return Err(Root::from_ref(script.downcast().unwrap()));
                }
            };
        }
    }

    pub fn end(&mut self) {
        self.html_tokenizer_sender.send(ToHtmlTokenizerMsg::End).unwrap();
    }

    pub fn url(&self) -> &ServoUrl {
        &self.url
    }

    pub fn set_plaintext_state(&mut self) {
        self.html_tokenizer_sender.send(ToHtmlTokenizerMsg::SetPlainTextState).unwrap();
    }

    fn insert_node(&mut self, id: ParseNodeId, node: JS<Node>) {
        assert!(self.nodes.insert(id, node).is_none());
    }

    fn get_node<'a>(&'a self, id: &ParseNodeId) -> &'a JS<Node> {
        self.nodes.get(id).expect("Node not found!")
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
            ParseOperation::CreateElement(id, name, attrs, current_line) => {
                let is = attrs.iter()
                              .find(|attr| attr.name.local.eq_str_ignore_ascii_case("is"))
                              .map(|attr| LocalName::from(&*attr.value));

                let elem = Element::create(name,
                                           is,
                                           &*self.document,
                                           ElementCreator::ParserCreated(current_line),
                                           CustomElementCreationMode::Synchronous);
                for attr in attrs {
                    elem.set_attribute_from_parser(attr.name, DOMString::from(attr.value), None);
                }

                self.insert_node(id, JS::from_ref(elem.upcast()));
            }
            ParseOperation::CreateComment(text, id) => {
                let comment = Comment::new(DOMString::from(text), document);
                self.insert_node(id, JS::from_ref(&comment.upcast()));
            }
            ParseOperation::AppendBeforeSibling(sibling, node) => {
                let node = match node {
                    NodeOrText::Node(n) => HtmlNodeOrText::AppendNode(JS::from_ref(&**self.get_node(&n.id))),
                    NodeOrText::Text(text) => HtmlNodeOrText::AppendText(
                        Tendril::from_str(&text).expect("String should convert to Tendril")
                    )
                };
                let sibling = &**self.get_node(&sibling);
                let parent = &*sibling.GetParentNode().expect("append_before_sibling called on node without parent");

                super::insert(parent, Some(sibling), node);
            }
            ParseOperation::Append(parent, node) => {
                let node = match node {
                    NodeOrText::Node(n) => HtmlNodeOrText::AppendNode(JS::from_ref(&**self.get_node(&n.id))),
                    NodeOrText::Text(text) => HtmlNodeOrText::AppendText(
                        Tendril::from_str(&text).expect("String should convert to Tendril")
                    )
                };

                let parent = &**self.get_node(&parent);
                super::insert(parent, None, node);
            }
            ParseOperation::AppendDoctypeToDocument(name, public_id, system_id) => {
                let doctype = DocumentType::new(
                    DOMString::from(String::from(name)), Some(DOMString::from(public_id)),
                    Some(DOMString::from(system_id)), document);

                document.upcast::<Node>().AppendChild(doctype.upcast()).expect("Appending failed");
            }
            ParseOperation::AddAttrsIfMissing(target_id, attrs) => {
                let elem = self.get_node(&target_id).downcast::<Element>()
                    .expect("tried to set attrs on non-Element in HTML parsing");
                for attr in attrs {
                    elem.set_attribute_from_parser(attr.name, DOMString::from(attr.value), None);
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
                    DOMString::from(target),
                    DOMString::from(data),
                    document);
                self.insert_node(node, JS::from_ref(pi.upcast()));
            }
            ParseOperation::SetQuirksMode(mode) => {
                document.set_quirks_mode(mode);
            }
        }
    }
}

fn run(sink: Sink,
       fragment_context_is_some: bool,
       ctxt_parse_node: Option<ParseNode>,
       form_parse_node: Option<ParseNode>,
       sender: Sender<FromHtmlTokenizerMsg>,
       receiver: Receiver<ToHtmlTokenizerMsg>) {
    let options = TreeBuilderOpts {
        ignore_missing_rules: true,
        .. Default::default()
    };

    let mut html_tokenizer = if fragment_context_is_some {
        let tb = TreeBuilder::new_for_fragment(
            sink,
            ctxt_parse_node.unwrap(),
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

    loop {
        match receiver.recv().expect("Unexpected sink channel panic in Html parser thread") {
            ToHtmlTokenizerMsg::Feed(data) => {
                let mut data = data.lock().unwrap();
                let input = (*data).iter().
                    map(|st| StrTendril::from(st.clone())).collect::<VecDeque<StrTendril>>();
                let mut buffer_queue = create_buffer_queue(input);
                let res = html_tokenizer.feed(&mut buffer_queue);

                // Gather changes to 'buffer_queue' and place them in 'data',
                // so that when control is transferred back to Tokenizer's feed method
                // (with the send call a few lines below), its 'input' parameter is
                // updated accordingly.
                let mut data_ = Vec::new();
                while let Some(str) = buffer_queue.pop_front() {
                    data_.push(SendTendril::from(str));
                }
                *data = data_;
                sender.send(FromHtmlTokenizerMsg::Result(res)).unwrap();
            },
            ToHtmlTokenizerMsg::End => {
                html_tokenizer.end();
                break;
            },
            ToHtmlTokenizerMsg::SetPlainTextState => html_tokenizer.set_plaintext_state()
        };
    }
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

pub struct Sink {
    current_line: u64,
    parse_node_data: HashMap<ParseNodeId, ParseNodeData>,
    next_parse_node_id: Cell<ParseNodeId>,
    document_node: ParseNode,
    sender: Sender<FromSinkMsg>,
    receiver: Receiver<ToSinkMsg>,
}

impl Sink {
    fn new(sender: Sender<FromSinkMsg>, receiver: Receiver<ToSinkMsg>) -> Sink {
        let mut sink = Sink {
            current_line: 1,
            parse_node_data: HashMap::new(),
            next_parse_node_id: Cell::new(1),
            document_node: ParseNode {
                id: 0,
                qual_name: None,
            },
            sender: sender,
            receiver: receiver,
        };
        let data = ParseNodeData::default();
        sink.insert_parse_node_data(0, data);
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

    fn send_op(&self, op: ParseOperation) {
        self.sender.send(FromSinkMsg::ProcessOperation(op)).unwrap();
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
        self.send_op(ParseOperation::GetTemplateContents(target.id, node.id));
        node
    }

    fn same_node(&self, x: &Self::Handle, y: &Self::Handle) -> bool {
        x.id == y.id
    }

    fn elem_name<'a>(&self, target: &'a Self::Handle) -> ExpandedName<'a> {
        target.qual_name.as_ref().expect("Expected qual name of node!").expanded()
    }

    fn same_tree(&self, x: &Self::Handle, y: &Self::Handle) -> bool {
        self.sender.send(FromSinkMsg::IsSameTree(x.id, y.id)).unwrap();
        match self.receiver.recv().expect("Unexpected sink channel panic in html parser thread.") {
            ToSinkMsg::IsSameTree(result) => result,
            _ => unreachable!(),
        }
    }

    fn create_element(&mut self, name: QualName, html_attrs: Vec<HtmlAttribute>, _flags: ElementFlags)
        -> Self::Handle {
        let mut node = self.new_parse_node();
        node.qual_name = Some(name.clone());
        {
            let mut node_data = self.get_parse_node_data_mut(&node.id);
            node_data.is_integration_point = html_attrs.iter()
            .any(|attr| {
                let attr_value = &String::from(attr.value.clone());
                (attr.name.local == local_name!("encoding") && attr.name.ns == ns!()) &&
                (attr_value.eq_ignore_ascii_case("text/html") ||
                attr_value.eq_ignore_ascii_case("application/xhtml+xml"))
            });
        }
        let attrs = Vec::from_iter(
            html_attrs.into_iter().map(|attr| Attribute { name: attr.name, value: String::from(attr.value) })
        );

        self.send_op(ParseOperation::CreateElement(node.id, name, attrs, self.current_line));
        node
    }

    fn create_comment(&mut self, text: StrTendril) -> Self::Handle {
        let node = self.new_parse_node();
        self.send_op(ParseOperation::CreateComment(String::from(text), node.id));
        node
    }

    fn create_pi(&mut self, target: StrTendril, data: StrTendril) -> ParseNode {
        let node = self.new_parse_node();
        self.send_op(ParseOperation::CreatePI(node.id, String::from(target), String::from(data)));
        node
    }

    fn has_parent_node(&self, node: &Self::Handle) -> bool {
        self.sender.send(FromSinkMsg::HasParentNode(node.id)).unwrap();
        match self.receiver.recv().expect("Unexpected sink channel panic in html parser thread.") {
            ToSinkMsg::HasParentNode(result) => result,
            _ => unreachable!(),
        }
    }

    fn associate_with_form(&mut self, target: &Self::Handle, form: &Self::Handle) {
        self.send_op(ParseOperation::AssociateWithForm(target.id, form.id));
    }

    fn append_before_sibling(&mut self,
                             sibling: &Self::Handle,
                             new_node: HtmlNodeOrText<Self::Handle>) {
        let new_node = match new_node {
            HtmlNodeOrText::AppendNode(node) => NodeOrText::Node(node),
            HtmlNodeOrText::AppendText(text) => NodeOrText::Text(String::from(text))
        };
        self.send_op(ParseOperation::AppendBeforeSibling(sibling.id, new_node));
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
        self.send_op(ParseOperation::SetQuirksMode(mode));
    }

    fn append(&mut self, parent: &Self::Handle, child: HtmlNodeOrText<Self::Handle>) {
        let child = match child {
            HtmlNodeOrText::AppendNode(node) => NodeOrText::Node(node),
            HtmlNodeOrText::AppendText(text) => NodeOrText::Text(String::from(text))
        };
        self.send_op(ParseOperation::Append(parent.id, child));
    }

    fn append_doctype_to_document(&mut self, name: StrTendril, public_id: StrTendril,
                                  system_id: StrTendril) {
        self.send_op(ParseOperation::AppendDoctypeToDocument(
            String::from(name),
            String::from(public_id),
            String::from(system_id)
        ));
    }

    fn add_attrs_if_missing(&mut self, target: &Self::Handle, html_attrs: Vec<HtmlAttribute>) {
        let attrs = Vec::from_iter(
            html_attrs.into_iter().map(|attr| Attribute { name: attr.name, value: String::from(attr.value) })
        );
        self.send_op(ParseOperation::AddAttrsIfMissing(target.id, attrs));
    }

    fn remove_from_parent(&mut self, target: &Self::Handle) {
        self.send_op(ParseOperation::RemoveFromParent(target.id));
    }

    fn mark_script_already_started(&mut self, node: &Self::Handle) {
        self.send_op(ParseOperation::MarkScriptAlreadyStarted(node.id));
    }

    fn complete_script(&mut self, _: &Self::Handle) -> NextParserState {
        panic!("complete_script should not be called here!");
    }

    fn reparent_children(&mut self, parent: &Self::Handle, new_parent: &Self::Handle) {
        self.send_op(ParseOperation::ReparentChildren(parent.id, new_parent.id));
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
        self.send_op(ParseOperation::Pop(node.id));
    }
}
