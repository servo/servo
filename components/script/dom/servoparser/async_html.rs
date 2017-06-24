/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![allow(unrooted_must_root)]

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
    GetTemplateContents { target: ParseNodeId, contents: ParseNodeId },

    CreateElement {
        node: ParseNodeId,
        name: QualName,
        attrs: Vec<Attribute>,
        current_line: u64
    },

    CreateComment { text: String, node: ParseNodeId },
    AppendBeforeSibling { sibling: ParseNodeId, node: NodeOrText },
    Append { parent: ParseNodeId, node: NodeOrText },

    AppendDoctypeToDocument {
        name: String,
        public_id: String,
        system_id: String
    },

    AddAttrsIfMissing { target: ParseNodeId, attrs: Vec<Attribute> },
    RemoveFromParent { target: ParseNodeId },
    MarkScriptAlreadyStarted { node: ParseNodeId },
    ReparentChildren { parent: ParseNodeId, new_parent: ParseNodeId },
    AssociateWithForm { target: ParseNodeId, form: ParseNodeId },

    CreatePI {
        node: ParseNodeId,
        target: String,
        data: String
    },

    Pop { node: ParseNodeId },

    SetQuirksMode {
        #[ignore_heap_size_of = "Defined in style"]
        mode: ServoQuirksMode
    },
}

#[derive(HeapSizeOf)]
enum ToTokenizerMsg {
    // From HtmlTokenizer
    TokenizerResultDone {
        #[ignore_heap_size_of = "Defined in html5ever"]
        updated_input: VecDeque<SendTendril<UTF8>>
    },
    TokenizerResultScript {
        script: ParseNode,
        #[ignore_heap_size_of = "Defined in html5ever"]
        updated_input: VecDeque<SendTendril<UTF8>>
    },
    End, // Sent to Tokenizer to signify HtmlTokenizer's end method has returned

    // From Sink
    ProcessOperation(ParseOperation),
    IsSameTree(ParseNodeId, ParseNodeId),
    HasParentNode(ParseNodeId),
}

#[derive(HeapSizeOf)]
enum ToHtmlTokenizerMsg {
    Feed {
        #[ignore_heap_size_of = "Defined in html5ever"]
        input: VecDeque<SendTendril<UTF8>>
    },
    End,
    SetPlainTextState,
}

// Responses to the queries asked by the the Sink to the Tokenizer,
// using the messages types in FromSinkMsg.
#[derive(HeapSizeOf)]
enum ToSinkMsg {
    IsSameTree(bool),
    HasParentNode(bool),
}

fn create_buffer_queue(mut buffers: VecDeque<SendTendril<UTF8>>) -> BufferQueue {
    let mut buffer_queue = BufferQueue::new();
    while let Some(st) = buffers.pop_front() {
        buffer_queue.push_back(StrTendril::from(st));
    }
    buffer_queue
}

// The async HTML Tokenizer consists of two separate types working together: the Tokenizer
// (defined below), which lives on the main thread, and the HtmlTokenizer, defined in html5ever, which
// lives on the parser thread.
// Steps:
// 1. A call to Tokenizer::new will spin up a new parser thread, creating an HtmlTokenizer instance,
//    which starts listening for messages from Tokenizer.
// 2. Upon receiving an input from ServoParser, the Tokenizer forwards it to HtmlTokenizer, where it starts
//    creating the necessary tree actions based on the input.
// 3. HtmlTokenizer sends these tree actions to the Tokenizer as soon as it creates them. The Tokenizer
//    then executes the received actions.
//
//    _____________                           _______________
//   |             |    ToHtmlTokenizerMsg   |               |
//   |             |------------------------>|               |
//   |             |                         |               |
//   |             |      ToTokenizerMsg     | HtmlTokenizer |
//   |             |<------------------------|               |
//   |  Tokenizer  |                         |               |
//   |             |      ToTokenizerMsg     |    ________   |
//   |             |<------------------------|---|        |  |
//   |             |                         |   |  Sink  |  |
//   |             |        ToSinkMsg        |   |        |  |
//   |             |-------------------------|-->|________|  |
//   |_____________|                         |_______________|
//
#[derive(HeapSizeOf, JSTraceable)]
#[must_root]
pub struct Tokenizer {
    document: JS<Document>,
    #[ignore_heap_size_of = "Defined in std"]
    receiver: Receiver<ToTokenizerMsg>,
    #[ignore_heap_size_of = "Defined in std"]
    html_tokenizer_sender: Sender<ToHtmlTokenizerMsg>,
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
        // Messages from the Tokenizer (main thread) to Sink (parser thread)
        let (to_sink_sender, sink_receiver) = channel();
        // Messages from HtmlTokenizer and Sink (parser thread) to Tokenizer (main thread)
        let (to_tokenizer_sender, tokenizer_receiver) = channel();

        let mut tokenizer = Tokenizer {
            document: JS::from_ref(document),
            receiver: tokenizer_receiver,
            html_tokenizer_sender: to_html_tokenizer_sender,
            sink_sender: to_sink_sender,
            nodes: HashMap::new(),
            url: url
        };
        tokenizer.insert_node(0, JS::from_ref(document.upcast()));

        let mut sink = Sink::new(to_tokenizer_sender.clone(), sink_receiver);
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
                to_tokenizer_sender,
                html_tokenizer_receiver);
        }).expect("HTML Parser thread spawning failed");

        tokenizer
    }

    pub fn feed(&mut self, input: &mut BufferQueue) -> Result<(), Root<HTMLScriptElement>> {
        let mut send_tendrils = VecDeque::new();
        while let Some(str) = input.pop_front() {
            send_tendrils.push_back(SendTendril::from(str));
        }

        // Send message to parser thread, asking it to start reading from the input.
        // Parser operation messages will be sent to main thread as they are evaluated.
        self.html_tokenizer_sender.send(ToHtmlTokenizerMsg::Feed { input: send_tendrils }).unwrap();

        loop {
            match self.receiver.recv().expect("Unexpected channel panic in main thread.") {
                ToTokenizerMsg::ProcessOperation(parse_op) => self.process_operation(parse_op),
                ToTokenizerMsg::IsSameTree(ref x_id, ref y_id) => {
                    let x = self.get_node(x_id);
                    let y = self.get_node(y_id);

                    let x = x.downcast::<Element>().expect("Element node expected");
                    let y = y.downcast::<Element>().expect("Element node expected");
                    self.sink_sender.send(ToSinkMsg::IsSameTree(x.is_in_same_home_subtree(y))).unwrap();
                },
                ToTokenizerMsg::HasParentNode(ref id) => {
                    let res = self.get_node(id).GetParentNode().is_some();
                    self.sink_sender.send(ToSinkMsg::HasParentNode(res)).unwrap();
                },
                ToTokenizerMsg::TokenizerResultDone { updated_input } => {
                    let buffer_queue = create_buffer_queue(updated_input);
                    *input = buffer_queue;
                    return Ok(());
                },
                ToTokenizerMsg::TokenizerResultScript { script, updated_input } => {
                    let buffer_queue = create_buffer_queue(updated_input);
                    *input = buffer_queue;
                    let script = self.get_node(&script.id);
                    return Err(Root::from_ref(script.downcast().unwrap()));
                }
                ToTokenizerMsg::End => unreachable!(),
            };
        }
    }

    pub fn end(&mut self) {
        self.html_tokenizer_sender.send(ToHtmlTokenizerMsg::End).unwrap();
        loop {
            match self.receiver.recv().expect("Unexpected channel panic in main thread.") {
                ToTokenizerMsg::ProcessOperation(parse_op) => self.process_operation(parse_op),
                ToTokenizerMsg::IsSameTree(ref x_id, ref y_id) => {
                    let x = self.get_node(x_id);
                    let y = self.get_node(y_id);

                    let x = x.downcast::<Element>().expect("Element node expected");
                    let y = y.downcast::<Element>().expect("Element node expected");
                    self.sink_sender.send(ToSinkMsg::IsSameTree(x.is_in_same_home_subtree(y))).unwrap();
                },
                ToTokenizerMsg::HasParentNode(ref id) => {
                    let res = self.get_node(id).GetParentNode().is_some();
                    self.sink_sender.send(ToSinkMsg::HasParentNode(res)).unwrap();
                },
                ToTokenizerMsg::End => return,
                _ => unreachable!(),
            };
        }
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
            ParseOperation::GetTemplateContents { target, contents } => {
                let target = Root::from_ref(&**self.get_node(&target));
                let template = target.downcast::<HTMLTemplateElement>().expect(
                    "Tried to extract contents from non-template element while parsing");
                self.insert_node(contents, JS::from_ref(template.Content().upcast()));
            }
            ParseOperation::CreateElement { node, name, attrs, current_line } => {
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

                self.insert_node(node, JS::from_ref(elem.upcast()));
            }
            ParseOperation::CreateComment { text, node } => {
                let comment = Comment::new(DOMString::from(text), document);
                self.insert_node(node, JS::from_ref(&comment.upcast()));
            }
            ParseOperation::AppendBeforeSibling { sibling, node } => {
                let node = match node {
                    NodeOrText::Node(n) => HtmlNodeOrText::AppendNode(JS::from_ref(&**self.get_node(&n.id))),
                    NodeOrText::Text(text) => HtmlNodeOrText::AppendText(
                        Tendril::from(text)
                    )
                };
                let sibling = &**self.get_node(&sibling);
                let parent = &*sibling.GetParentNode().expect("append_before_sibling called on node without parent");

                super::insert(parent, Some(sibling), node);
            }
            ParseOperation::Append { parent, node } => {
                let node = match node {
                    NodeOrText::Node(n) => HtmlNodeOrText::AppendNode(JS::from_ref(&**self.get_node(&n.id))),
                    NodeOrText::Text(text) => HtmlNodeOrText::AppendText(
                        Tendril::from(text)
                    )
                };

                let parent = &**self.get_node(&parent);
                super::insert(parent, None, node);
            }
            ParseOperation::AppendDoctypeToDocument { name, public_id, system_id } => {
                let doctype = DocumentType::new(
                    DOMString::from(String::from(name)), Some(DOMString::from(public_id)),
                    Some(DOMString::from(system_id)), document);

                document.upcast::<Node>().AppendChild(doctype.upcast()).expect("Appending failed");
            }
            ParseOperation::AddAttrsIfMissing { target, attrs } => {
                let elem = self.get_node(&target).downcast::<Element>()
                    .expect("tried to set attrs on non-Element in HTML parsing");
                for attr in attrs {
                    elem.set_attribute_from_parser(attr.name, DOMString::from(attr.value), None);
                }
            }
            ParseOperation::RemoveFromParent { target } => {
                if let Some(ref parent) = self.get_node(&target).GetParentNode() {
                    parent.RemoveChild(&**self.get_node(&target)).unwrap();
                }
            }
            ParseOperation::MarkScriptAlreadyStarted { node } => {
                let script = self.get_node(&node).downcast::<HTMLScriptElement>();
                script.map(|script| script.set_already_started(true));
            }
            ParseOperation::ReparentChildren { parent, new_parent } => {
                let parent = self.get_node(&parent);
                let new_parent = self.get_node(&new_parent);
                while let Some(child) = parent.GetFirstChild() {
                    new_parent.AppendChild(&child).unwrap();
                }
            }
            ParseOperation::AssociateWithForm { target, form } => {
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
            ParseOperation::Pop { node } => {
                vtable_for(self.get_node(&node)).pop();
            }
            ParseOperation::CreatePI { node, target, data } => {
                let pi = ProcessingInstruction::new(
                    DOMString::from(target),
                    DOMString::from(data),
                    document);
                self.insert_node(node, JS::from_ref(pi.upcast()));
            }
            ParseOperation::SetQuirksMode { mode } => {
                document.set_quirks_mode(mode);
            }
        }
    }
}

fn run(sink: Sink,
       fragment_context_is_some: bool,
       ctxt_parse_node: Option<ParseNode>,
       form_parse_node: Option<ParseNode>,
       sender: Sender<ToTokenizerMsg>,
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
        match receiver.recv().expect("Unexpected channel panic in html parser thread") {
            ToHtmlTokenizerMsg::Feed { input } => {
                let mut input = create_buffer_queue(input);
                let res = html_tokenizer.feed(&mut input);

                // Gather changes to 'input' and place them in 'updated_input',
                // which will be sent to the main thread to update feed method's 'input'
                let mut updated_input = VecDeque::new();
                while let Some(st) = input.pop_front() {
                    updated_input.push_back(SendTendril::from(st));
                }

                let res = match res {
                    TokenizerResult::Done => ToTokenizerMsg::TokenizerResultDone { updated_input },
                    TokenizerResult::Script(script) => ToTokenizerMsg::TokenizerResultScript { script, updated_input }
                };
                sender.send(res).unwrap();
            },
            ToHtmlTokenizerMsg::End => {
                html_tokenizer.end();
                sender.send(ToTokenizerMsg::End).unwrap();
                break;
            },
            ToHtmlTokenizerMsg::SetPlainTextState => html_tokenizer.set_plaintext_state()
        };
    }
}

#[derive(JSTraceable, HeapSizeOf, Default)]
struct ParseNodeData {
    contents: Option<ParseNode>,
    is_integration_point: bool,
}

pub struct Sink {
    current_line: u64,
    parse_node_data: HashMap<ParseNodeId, ParseNodeData>,
    next_parse_node_id: Cell<ParseNodeId>,
    document_node: ParseNode,
    sender: Sender<ToTokenizerMsg>,
    receiver: Receiver<ToSinkMsg>,
}

impl Sink {
    fn new(sender: Sender<ToTokenizerMsg>, receiver: Receiver<ToSinkMsg>) -> Sink {
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
        self.sender.send(ToTokenizerMsg::ProcessOperation(op)).unwrap();
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
        self.send_op(ParseOperation::GetTemplateContents { target: target.id, contents: node.id });
        node
    }

    fn same_node(&self, x: &Self::Handle, y: &Self::Handle) -> bool {
        x.id == y.id
    }

    fn elem_name<'a>(&self, target: &'a Self::Handle) -> ExpandedName<'a> {
        target.qual_name.as_ref().expect("Expected qual name of node!").expanded()
    }

    fn same_tree(&self, x: &Self::Handle, y: &Self::Handle) -> bool {
        self.sender.send(ToTokenizerMsg::IsSameTree(x.id, y.id)).unwrap();
        match self.receiver.recv().expect("Unexpected channel panic in html parser thread.") {
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
        let attrs = html_attrs.into_iter()
            .map(|attr| Attribute { name: attr.name, value: String::from(attr.value) }).collect();

        self.send_op(ParseOperation::CreateElement {
            node: node.id,
            name,
            attrs,
            current_line: self.current_line
        });
        node
    }

    fn create_comment(&mut self, text: StrTendril) -> Self::Handle {
        let node = self.new_parse_node();
        self.send_op(ParseOperation::CreateComment { text: String::from(text), node: node.id });
        node
    }

    fn create_pi(&mut self, target: StrTendril, data: StrTendril) -> ParseNode {
        let node = self.new_parse_node();
        self.send_op(ParseOperation::CreatePI {
            node: node.id,
            target: String::from(target),
            data: String::from(data)
        });
        node
    }

    fn has_parent_node(&self, node: &Self::Handle) -> bool {
        self.sender.send(ToTokenizerMsg::HasParentNode(node.id)).unwrap();
        match self.receiver.recv().expect("Unexpected channel panic in html parser thread.") {
            ToSinkMsg::HasParentNode(result) => result,
            _ => unreachable!(),
        }
    }

    fn associate_with_form(&mut self, target: &Self::Handle, form: &Self::Handle) {
        self.send_op(ParseOperation::AssociateWithForm {
            target: target.id,
            form: form.id
        });
    }

    fn append_before_sibling(&mut self,
                             sibling: &Self::Handle,
                             new_node: HtmlNodeOrText<Self::Handle>) {
        let new_node = match new_node {
            HtmlNodeOrText::AppendNode(node) => NodeOrText::Node(node),
            HtmlNodeOrText::AppendText(text) => NodeOrText::Text(String::from(text))
        };
        self.send_op(ParseOperation::AppendBeforeSibling { sibling: sibling.id, node: new_node });
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
        self.send_op(ParseOperation::SetQuirksMode { mode });
    }

    fn append(&mut self, parent: &Self::Handle, child: HtmlNodeOrText<Self::Handle>) {
        let child = match child {
            HtmlNodeOrText::AppendNode(node) => NodeOrText::Node(node),
            HtmlNodeOrText::AppendText(text) => NodeOrText::Text(String::from(text))
        };
        self.send_op(ParseOperation::Append { parent: parent.id, node: child });
    }

    fn append_doctype_to_document(&mut self, name: StrTendril, public_id: StrTendril,
                                  system_id: StrTendril) {
        self.send_op(ParseOperation::AppendDoctypeToDocument {
            name: String::from(name),
            public_id: String::from(public_id),
            system_id: String::from(system_id)
        });
    }

    fn add_attrs_if_missing(&mut self, target: &Self::Handle, html_attrs: Vec<HtmlAttribute>) {
        let attrs = html_attrs.into_iter()
            .map(|attr| Attribute { name: attr.name, value: String::from(attr.value) }).collect();
        self.send_op(ParseOperation::AddAttrsIfMissing { target: target.id, attrs });
    }

    fn remove_from_parent(&mut self, target: &Self::Handle) {
        self.send_op(ParseOperation::RemoveFromParent { target: target.id });
    }

    fn mark_script_already_started(&mut self, node: &Self::Handle) {
        self.send_op(ParseOperation::MarkScriptAlreadyStarted { node: node.id });
    }

    fn complete_script(&mut self, _: &Self::Handle) -> NextParserState {
        panic!("complete_script should not be called here!");
    }

    fn reparent_children(&mut self, parent: &Self::Handle, new_parent: &Self::Handle) {
        self.send_op(ParseOperation::ReparentChildren { parent: parent.id, new_parent: new_parent.id });
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
        self.send_op(ParseOperation::Pop { node: node.id });
    }
}
