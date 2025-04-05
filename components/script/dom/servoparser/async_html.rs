/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![cfg_attr(crown, allow(crown::unrooted_must_root))]

use std::borrow::Cow;
use std::cell::{Cell, Ref, RefCell, RefMut};
use std::collections::HashMap;
use std::collections::vec_deque::VecDeque;
use std::thread;

use crossbeam_channel::{Receiver, Sender, unbounded};
use html5ever::buffer_queue::BufferQueue;
use html5ever::tendril::fmt::UTF8;
use html5ever::tendril::{SendTendril, StrTendril, Tendril};
use html5ever::tokenizer::{Tokenizer as HtmlTokenizer, TokenizerOpts, TokenizerResult};
use html5ever::tree_builder::{
    ElementFlags, NextParserState, NodeOrText as HtmlNodeOrText, QuirksMode, TreeBuilder,
    TreeBuilderOpts, TreeSink,
};
use html5ever::{
    Attribute as HtmlAttribute, ExpandedName, QualName, local_name, namespace_url, ns,
};
use servo_url::ServoUrl;
use style::context::QuirksMode as ServoQuirksMode;

use crate::dom::bindings::codegen::Bindings::HTMLTemplateElementBinding::HTMLTemplateElementMethods;
use crate::dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::comment::Comment;
use crate::dom::document::Document;
use crate::dom::documenttype::DocumentType;
use crate::dom::element::{Element, ElementCreator};
use crate::dom::htmlformelement::{FormControlElementHelpers, HTMLFormElement};
use crate::dom::htmlscriptelement::HTMLScriptElement;
use crate::dom::htmltemplateelement::HTMLTemplateElement;
use crate::dom::node::Node;
use crate::dom::processinginstruction::ProcessingInstruction;
use crate::dom::servoparser::{ElementAttribute, ParsingAlgorithm, create_element_for_token};
use crate::dom::virtualmethods::vtable_for;
use crate::script_runtime::CanGc;

type ParseNodeId = usize;

#[derive(Clone, JSTraceable, MallocSizeOf)]
pub(crate) struct ParseNode {
    id: ParseNodeId,
    #[no_trace]
    qual_name: Option<QualName>,
}

#[derive(JSTraceable, MallocSizeOf)]
enum NodeOrText {
    Node(ParseNode),
    Text(String),
}

#[derive(JSTraceable, MallocSizeOf)]
struct Attribute {
    #[no_trace]
    name: QualName,
    value: String,
}

#[derive(JSTraceable, MallocSizeOf)]
enum ParseOperation {
    GetTemplateContents {
        target: ParseNodeId,
        contents: ParseNodeId,
    },

    CreateElement {
        node: ParseNodeId,
        #[no_trace]
        name: QualName,
        attrs: Vec<Attribute>,
        current_line: u64,
    },

    CreateComment {
        text: String,
        node: ParseNodeId,
    },
    AppendBeforeSibling {
        sibling: ParseNodeId,
        node: NodeOrText,
    },
    AppendBasedOnParentNode {
        element: ParseNodeId,
        prev_element: ParseNodeId,
        node: NodeOrText,
    },
    Append {
        parent: ParseNodeId,
        node: NodeOrText,
    },

    AppendDoctypeToDocument {
        name: String,
        public_id: String,
        system_id: String,
    },

    AddAttrsIfMissing {
        target: ParseNodeId,
        attrs: Vec<Attribute>,
    },
    RemoveFromParent {
        target: ParseNodeId,
    },
    MarkScriptAlreadyStarted {
        node: ParseNodeId,
    },
    ReparentChildren {
        parent: ParseNodeId,
        new_parent: ParseNodeId,
    },

    AssociateWithForm {
        target: ParseNodeId,
        form: ParseNodeId,
        element: ParseNodeId,
        prev_element: Option<ParseNodeId>,
    },

    CreatePI {
        node: ParseNodeId,
        target: String,
        data: String,
    },

    Pop {
        node: ParseNodeId,
    },

    SetQuirksMode {
        #[ignore_malloc_size_of = "Defined in style"]
        #[no_trace]
        mode: ServoQuirksMode,
    },
}

#[derive(MallocSizeOf)]
enum ToTokenizerMsg {
    // From HtmlTokenizer
    TokenizerResultDone {
        #[ignore_malloc_size_of = "Defined in html5ever"]
        updated_input: VecDeque<SendTendril<UTF8>>,
    },
    TokenizerResultScript {
        script: ParseNode,
        #[ignore_malloc_size_of = "Defined in html5ever"]
        updated_input: VecDeque<SendTendril<UTF8>>,
    },
    End, // Sent to Tokenizer to signify HtmlTokenizer's end method has returned

    // From Sink
    ProcessOperation(ParseOperation),
}

#[derive(MallocSizeOf)]
enum ToHtmlTokenizerMsg {
    Feed {
        #[ignore_malloc_size_of = "Defined in html5ever"]
        input: VecDeque<SendTendril<UTF8>>,
    },
    End,
    SetPlainTextState,
}

fn create_buffer_queue(mut buffers: VecDeque<SendTendril<UTF8>>) -> BufferQueue {
    let buffer_queue = BufferQueue::default();
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
//   |             |                         |               |
//   |             |                         |               |
//   |             |   ToHtmlTokenizerMsg    |               |
//   |             |------------------------>| HtmlTokenizer |
//   |             |                         |               |
//   |  Tokenizer  |     ToTokenizerMsg      |               |
//   |             |<------------------------|    ________   |
//   |             |                         |   |        |  |
//   |             |     ToTokenizerMsg      |   |  Sink  |  |
//   |             |<------------------------|---|        |  |
//   |             |                         |   |________|  |
//   |_____________|                         |_______________|
//
#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) struct Tokenizer {
    document: Dom<Document>,
    #[ignore_malloc_size_of = "Defined in std"]
    #[no_trace]
    receiver: Receiver<ToTokenizerMsg>,
    #[ignore_malloc_size_of = "Defined in std"]
    #[no_trace]
    html_tokenizer_sender: Sender<ToHtmlTokenizerMsg>,
    #[ignore_malloc_size_of = "Defined in std"]
    nodes: RefCell<HashMap<ParseNodeId, Dom<Node>>>,
    #[no_trace]
    url: ServoUrl,
    parsing_algorithm: ParsingAlgorithm,
}

impl Tokenizer {
    pub(crate) fn new(
        document: &Document,
        url: ServoUrl,
        fragment_context: Option<super::FragmentContext>,
    ) -> Self {
        // Messages from the Tokenizer (main thread) to HtmlTokenizer (parser thread)
        let (to_html_tokenizer_sender, html_tokenizer_receiver) = unbounded();
        // Messages from HtmlTokenizer and Sink (parser thread) to Tokenizer (main thread)
        let (to_tokenizer_sender, tokenizer_receiver) = unbounded();

        let algorithm = match fragment_context {
            Some(_) => ParsingAlgorithm::Fragment,
            None => ParsingAlgorithm::Normal,
        };

        let tokenizer = Tokenizer {
            document: Dom::from_ref(document),
            receiver: tokenizer_receiver,
            html_tokenizer_sender: to_html_tokenizer_sender,
            nodes: RefCell::new(HashMap::new()),
            url,
            parsing_algorithm: algorithm,
        };
        tokenizer.insert_node(0, Dom::from_ref(document.upcast()));

        let sink = Sink::new(to_tokenizer_sender.clone());
        let mut ctxt_parse_node = None;
        let mut form_parse_node = None;
        let mut fragment_context_is_some = false;
        if let Some(fc) = fragment_context {
            let node = sink.new_parse_node();
            tokenizer.insert_node(node.id, Dom::from_ref(fc.context_elem));
            ctxt_parse_node = Some(node);

            form_parse_node = fc.form_elem.map(|form_elem| {
                let node = sink.new_parse_node();
                tokenizer.insert_node(node.id, Dom::from_ref(form_elem));
                node
            });
            fragment_context_is_some = true;
        };

        // Create new thread for HtmlTokenizer. This is where parser actions
        // will be generated from the input provided. These parser actions are then passed
        // onto the main thread to be executed.
        let scripting_enabled = document.has_browsing_context();
        thread::Builder::new()
            .name(format!("Parse:{}", tokenizer.url.debug_compact()))
            .spawn(move || {
                run(
                    sink,
                    fragment_context_is_some,
                    ctxt_parse_node,
                    form_parse_node,
                    to_tokenizer_sender,
                    html_tokenizer_receiver,
                    scripting_enabled,
                );
            })
            .expect("HTML Parser thread spawning failed");

        tokenizer
    }

    pub(crate) fn feed(
        &self,
        input: &BufferQueue,
        can_gc: CanGc,
    ) -> TokenizerResult<DomRoot<HTMLScriptElement>> {
        let mut send_tendrils = VecDeque::new();
        while let Some(str) = input.pop_front() {
            send_tendrils.push_back(SendTendril::from(str));
        }

        // Send message to parser thread, asking it to start reading from the input.
        // Parser operation messages will be sent to main thread as they are evaluated.
        self.html_tokenizer_sender
            .send(ToHtmlTokenizerMsg::Feed {
                input: send_tendrils,
            })
            .unwrap();

        loop {
            match self
                .receiver
                .recv()
                .expect("Unexpected channel panic in main thread.")
            {
                ToTokenizerMsg::ProcessOperation(parse_op) => {
                    self.process_operation(parse_op, can_gc)
                },
                ToTokenizerMsg::TokenizerResultDone { updated_input } => {
                    let buffer_queue = create_buffer_queue(updated_input);
                    input.replace_with(buffer_queue);
                    return TokenizerResult::Done;
                },
                ToTokenizerMsg::TokenizerResultScript {
                    script,
                    updated_input,
                } => {
                    let buffer_queue = create_buffer_queue(updated_input);
                    input.replace_with(buffer_queue);
                    let script = self.get_node(&script.id);
                    return TokenizerResult::Script(DomRoot::from_ref(script.downcast().unwrap()));
                },
                _ => unreachable!(),
            };
        }
    }

    pub(crate) fn end(&self, can_gc: CanGc) {
        self.html_tokenizer_sender
            .send(ToHtmlTokenizerMsg::End)
            .unwrap();
        loop {
            match self
                .receiver
                .recv()
                .expect("Unexpected channel panic in main thread.")
            {
                ToTokenizerMsg::ProcessOperation(parse_op) => {
                    self.process_operation(parse_op, can_gc)
                },
                ToTokenizerMsg::TokenizerResultDone { updated_input: _ } |
                ToTokenizerMsg::TokenizerResultScript {
                    script: _,
                    updated_input: _,
                } => continue,
                ToTokenizerMsg::End => return,
            };
        }
    }

    pub(crate) fn url(&self) -> &ServoUrl {
        &self.url
    }

    pub(crate) fn set_plaintext_state(&self) {
        self.html_tokenizer_sender
            .send(ToHtmlTokenizerMsg::SetPlainTextState)
            .unwrap();
    }

    fn insert_node(&self, id: ParseNodeId, node: Dom<Node>) {
        assert!(self.nodes.borrow_mut().insert(id, node).is_none());
    }

    fn get_node<'a>(&'a self, id: &ParseNodeId) -> Ref<'a, Dom<Node>> {
        Ref::map(self.nodes.borrow(), |nodes| {
            nodes.get(id).expect("Node not found!")
        })
    }

    fn append_before_sibling(&self, sibling: ParseNodeId, node: NodeOrText, can_gc: CanGc) {
        let node = match node {
            NodeOrText::Node(n) => {
                HtmlNodeOrText::AppendNode(Dom::from_ref(&**self.get_node(&n.id)))
            },
            NodeOrText::Text(text) => HtmlNodeOrText::AppendText(Tendril::from(text)),
        };
        let sibling = &**self.get_node(&sibling);
        let parent = &*sibling
            .GetParentNode()
            .expect("append_before_sibling called on node without parent");

        super::insert(parent, Some(sibling), node, self.parsing_algorithm, can_gc);
    }

    fn append(&self, parent: ParseNodeId, node: NodeOrText, can_gc: CanGc) {
        let node = match node {
            NodeOrText::Node(n) => {
                HtmlNodeOrText::AppendNode(Dom::from_ref(&**self.get_node(&n.id)))
            },
            NodeOrText::Text(text) => HtmlNodeOrText::AppendText(Tendril::from(text)),
        };

        let parent = &**self.get_node(&parent);
        super::insert(parent, None, node, self.parsing_algorithm, can_gc);
    }

    fn has_parent_node(&self, node: ParseNodeId) -> bool {
        self.get_node(&node).GetParentNode().is_some()
    }

    fn same_tree(&self, x: ParseNodeId, y: ParseNodeId) -> bool {
        let x = self.get_node(&x);
        let y = self.get_node(&y);

        let x = x.downcast::<Element>().expect("Element node expected");
        let y = y.downcast::<Element>().expect("Element node expected");
        x.is_in_same_home_subtree(y)
    }

    fn process_operation(&self, op: ParseOperation, can_gc: CanGc) {
        let document = DomRoot::from_ref(&**self.get_node(&0));
        let document = document
            .downcast::<Document>()
            .expect("Document node should be downcasted!");
        match op {
            ParseOperation::GetTemplateContents { target, contents } => {
                let target = DomRoot::from_ref(&**self.get_node(&target));
                let template = target
                    .downcast::<HTMLTemplateElement>()
                    .expect("Tried to extract contents from non-template element while parsing");
                self.insert_node(contents, Dom::from_ref(template.Content(can_gc).upcast()));
            },
            ParseOperation::CreateElement {
                node,
                name,
                attrs,
                current_line,
            } => {
                let attrs = attrs
                    .into_iter()
                    .map(|attr| ElementAttribute::new(attr.name, DOMString::from(attr.value)))
                    .collect();
                let element = create_element_for_token(
                    name,
                    attrs,
                    &self.document,
                    ElementCreator::ParserCreated(current_line),
                    ParsingAlgorithm::Normal,
                    can_gc,
                );
                self.insert_node(node, Dom::from_ref(element.upcast()));
            },
            ParseOperation::CreateComment { text, node } => {
                let comment = Comment::new(DOMString::from(text), document, None, can_gc);
                self.insert_node(node, Dom::from_ref(comment.upcast()));
            },
            ParseOperation::AppendBeforeSibling { sibling, node } => {
                self.append_before_sibling(sibling, node, can_gc);
            },
            ParseOperation::Append { parent, node } => {
                self.append(parent, node, can_gc);
            },
            ParseOperation::AppendBasedOnParentNode {
                element,
                prev_element,
                node,
            } => {
                if self.has_parent_node(element) {
                    self.append_before_sibling(element, node, can_gc);
                } else {
                    self.append(prev_element, node, can_gc);
                }
            },
            ParseOperation::AppendDoctypeToDocument {
                name,
                public_id,
                system_id,
            } => {
                let doctype = DocumentType::new(
                    DOMString::from(name),
                    Some(DOMString::from(public_id)),
                    Some(DOMString::from(system_id)),
                    document,
                    can_gc,
                );

                document
                    .upcast::<Node>()
                    .AppendChild(doctype.upcast())
                    .expect("Appending failed");
            },
            ParseOperation::AddAttrsIfMissing { target, attrs } => {
                let node = self.get_node(&target);
                let elem = node
                    .downcast::<Element>()
                    .expect("tried to set attrs on non-Element in HTML parsing");
                for attr in attrs {
                    elem.set_attribute_from_parser(
                        attr.name,
                        DOMString::from(attr.value),
                        None,
                        can_gc,
                    );
                }
            },
            ParseOperation::RemoveFromParent { target } => {
                if let Some(ref parent) = self.get_node(&target).GetParentNode() {
                    parent.RemoveChild(&self.get_node(&target)).unwrap();
                }
            },
            ParseOperation::MarkScriptAlreadyStarted { node } => {
                let node = self.get_node(&node);
                let script = node.downcast::<HTMLScriptElement>();
                if let Some(script) = script {
                    script.set_already_started(true)
                }
            },
            ParseOperation::ReparentChildren { parent, new_parent } => {
                let parent = self.get_node(&parent);
                let new_parent = self.get_node(&new_parent);
                while let Some(child) = parent.GetFirstChild() {
                    new_parent.AppendChild(&child).unwrap();
                }
            },
            ParseOperation::AssociateWithForm {
                target,
                form,
                element,
                prev_element,
            } => {
                let tree_node = prev_element.map_or(element, |prev| {
                    if self.has_parent_node(element) {
                        element
                    } else {
                        prev
                    }
                });

                if !self.same_tree(tree_node, form) {
                    return;
                }
                let form = self.get_node(&form);
                let form = DomRoot::downcast::<HTMLFormElement>(DomRoot::from_ref(&**form))
                    .expect("Owner must be a form element");

                let node = self.get_node(&target);
                let elem = node.downcast::<Element>();
                let control = elem.and_then(|e| e.as_maybe_form_control());

                if let Some(control) = control {
                    control.set_form_owner_from_parser(&form, can_gc);
                }
            },
            ParseOperation::Pop { node } => {
                vtable_for(&self.get_node(&node)).pop();
            },
            ParseOperation::CreatePI { node, target, data } => {
                let pi = ProcessingInstruction::new(
                    DOMString::from(target),
                    DOMString::from(data),
                    document,
                    can_gc,
                );
                self.insert_node(node, Dom::from_ref(pi.upcast()));
            },
            ParseOperation::SetQuirksMode { mode } => {
                document.set_quirks_mode(mode);
            },
        }
    }
}

fn run(
    sink: Sink,
    fragment_context_is_some: bool,
    ctxt_parse_node: Option<ParseNode>,
    form_parse_node: Option<ParseNode>,
    sender: Sender<ToTokenizerMsg>,
    receiver: Receiver<ToHtmlTokenizerMsg>,
    scripting_enabled: bool,
) {
    let options = TreeBuilderOpts {
        ignore_missing_rules: true,
        scripting_enabled,
        ..Default::default()
    };

    let html_tokenizer = if fragment_context_is_some {
        let tb =
            TreeBuilder::new_for_fragment(sink, ctxt_parse_node.unwrap(), form_parse_node, options);

        let tok_options = TokenizerOpts {
            initial_state: Some(tb.tokenizer_state_for_context_elem()),
            ..Default::default()
        };

        HtmlTokenizer::new(tb, tok_options)
    } else {
        HtmlTokenizer::new(TreeBuilder::new(sink, options), Default::default())
    };

    loop {
        match receiver
            .recv()
            .expect("Unexpected channel panic in html parser thread")
        {
            ToHtmlTokenizerMsg::Feed { input } => {
                let input = create_buffer_queue(input);
                let res = html_tokenizer.feed(&input);

                // Gather changes to 'input' and place them in 'updated_input',
                // which will be sent to the main thread to update feed method's 'input'
                let mut updated_input = VecDeque::new();
                while let Some(st) = input.pop_front() {
                    updated_input.push_back(SendTendril::from(st));
                }

                let res = match res {
                    TokenizerResult::Done => ToTokenizerMsg::TokenizerResultDone { updated_input },
                    TokenizerResult::Script(script) => ToTokenizerMsg::TokenizerResultScript {
                        script,
                        updated_input,
                    },
                };
                sender.send(res).unwrap();
            },
            ToHtmlTokenizerMsg::End => {
                html_tokenizer.end();
                sender.send(ToTokenizerMsg::End).unwrap();
                break;
            },
            ToHtmlTokenizerMsg::SetPlainTextState => html_tokenizer.set_plaintext_state(),
        };
    }
}

#[derive(Default, JSTraceable, MallocSizeOf)]
struct ParseNodeData {
    contents: Option<ParseNode>,
    is_integration_point: bool,
}

pub(crate) struct Sink {
    current_line: Cell<u64>,
    parse_node_data: RefCell<HashMap<ParseNodeId, ParseNodeData>>,
    next_parse_node_id: Cell<ParseNodeId>,
    document_node: ParseNode,
    sender: Sender<ToTokenizerMsg>,
}

impl Sink {
    fn new(sender: Sender<ToTokenizerMsg>) -> Sink {
        let sink = Sink {
            current_line: Cell::new(1),
            parse_node_data: RefCell::new(HashMap::new()),
            next_parse_node_id: Cell::new(1),
            document_node: ParseNode {
                id: 0,
                qual_name: None,
            },
            sender,
        };
        let data = ParseNodeData::default();
        sink.insert_parse_node_data(0, data);
        sink
    }

    fn new_parse_node(&self) -> ParseNode {
        let id = self.next_parse_node_id.get();
        let data = ParseNodeData::default();
        self.insert_parse_node_data(id, data);
        self.next_parse_node_id.set(id + 1);
        ParseNode {
            id,
            qual_name: None,
        }
    }

    fn send_op(&self, op: ParseOperation) {
        self.sender
            .send(ToTokenizerMsg::ProcessOperation(op))
            .unwrap();
    }

    fn insert_parse_node_data(&self, id: ParseNodeId, data: ParseNodeData) {
        assert!(self.parse_node_data.borrow_mut().insert(id, data).is_none());
    }

    fn get_parse_node_data<'a>(&'a self, id: &'a ParseNodeId) -> Ref<'a, ParseNodeData> {
        Ref::map(self.parse_node_data.borrow(), |data| {
            data.get(id).expect("Parse Node data not found!")
        })
    }

    fn get_parse_node_data_mut<'a>(&'a self, id: &'a ParseNodeId) -> RefMut<'a, ParseNodeData> {
        RefMut::map(self.parse_node_data.borrow_mut(), |data| {
            data.get_mut(id).expect("Parse Node data not found!")
        })
    }
}

#[cfg_attr(crown, allow(crown::unrooted_must_root))]
impl TreeSink for Sink {
    type Output = Self;
    fn finish(self) -> Self {
        self
    }

    type Handle = ParseNode;
    type ElemName<'a>
        = ExpandedName<'a>
    where
        Self: 'a;

    fn get_document(&self) -> Self::Handle {
        self.document_node.clone()
    }

    fn get_template_contents(&self, target: &Self::Handle) -> Self::Handle {
        if let Some(ref contents) = self.get_parse_node_data(&target.id).contents {
            return contents.clone();
        }
        let node = self.new_parse_node();
        {
            let mut data = self.get_parse_node_data_mut(&target.id);
            data.contents = Some(node.clone());
        }
        self.send_op(ParseOperation::GetTemplateContents {
            target: target.id,
            contents: node.id,
        });
        node
    }

    fn same_node(&self, x: &Self::Handle, y: &Self::Handle) -> bool {
        x.id == y.id
    }

    fn elem_name<'a>(&self, target: &'a Self::Handle) -> ExpandedName<'a> {
        target
            .qual_name
            .as_ref()
            .expect("Expected qual name of node!")
            .expanded()
    }

    fn create_element(
        &self,
        name: QualName,
        html_attrs: Vec<HtmlAttribute>,
        _flags: ElementFlags,
    ) -> Self::Handle {
        let mut node = self.new_parse_node();
        node.qual_name = Some(name.clone());
        {
            let mut node_data = self.get_parse_node_data_mut(&node.id);
            node_data.is_integration_point = html_attrs.iter().any(|attr| {
                let attr_value = &String::from(attr.value.clone());
                (attr.name.local == local_name!("encoding") && attr.name.ns == ns!()) &&
                    (attr_value.eq_ignore_ascii_case("text/html") ||
                        attr_value.eq_ignore_ascii_case("application/xhtml+xml"))
            });
        }
        let attrs = html_attrs
            .into_iter()
            .map(|attr| Attribute {
                name: attr.name,
                value: String::from(attr.value),
            })
            .collect();

        self.send_op(ParseOperation::CreateElement {
            node: node.id,
            name,
            attrs,
            current_line: self.current_line.get(),
        });
        node
    }

    fn create_comment(&self, text: StrTendril) -> Self::Handle {
        let node = self.new_parse_node();
        self.send_op(ParseOperation::CreateComment {
            text: String::from(text),
            node: node.id,
        });
        node
    }

    fn create_pi(&self, target: StrTendril, data: StrTendril) -> ParseNode {
        let node = self.new_parse_node();
        self.send_op(ParseOperation::CreatePI {
            node: node.id,
            target: String::from(target),
            data: String::from(data),
        });
        node
    }

    fn associate_with_form(
        &self,
        target: &Self::Handle,
        form: &Self::Handle,
        nodes: (&Self::Handle, Option<&Self::Handle>),
    ) {
        let (element, prev_element) = nodes;
        self.send_op(ParseOperation::AssociateWithForm {
            target: target.id,
            form: form.id,
            element: element.id,
            prev_element: prev_element.map(|p| p.id),
        });
    }

    fn append_before_sibling(
        &self,
        sibling: &Self::Handle,
        new_node: HtmlNodeOrText<Self::Handle>,
    ) {
        let new_node = match new_node {
            HtmlNodeOrText::AppendNode(node) => NodeOrText::Node(node),
            HtmlNodeOrText::AppendText(text) => NodeOrText::Text(String::from(text)),
        };
        self.send_op(ParseOperation::AppendBeforeSibling {
            sibling: sibling.id,
            node: new_node,
        });
    }

    fn append_based_on_parent_node(
        &self,
        elem: &Self::Handle,
        prev_elem: &Self::Handle,
        child: HtmlNodeOrText<Self::Handle>,
    ) {
        let child = match child {
            HtmlNodeOrText::AppendNode(node) => NodeOrText::Node(node),
            HtmlNodeOrText::AppendText(text) => NodeOrText::Text(String::from(text)),
        };
        self.send_op(ParseOperation::AppendBasedOnParentNode {
            element: elem.id,
            prev_element: prev_elem.id,
            node: child,
        });
    }

    fn parse_error(&self, msg: Cow<'static, str>) {
        debug!("Parse error: {}", msg);
    }

    fn set_quirks_mode(&self, mode: QuirksMode) {
        let mode = match mode {
            QuirksMode::Quirks => ServoQuirksMode::Quirks,
            QuirksMode::LimitedQuirks => ServoQuirksMode::LimitedQuirks,
            QuirksMode::NoQuirks => ServoQuirksMode::NoQuirks,
        };
        self.send_op(ParseOperation::SetQuirksMode { mode });
    }

    fn append(&self, parent: &Self::Handle, child: HtmlNodeOrText<Self::Handle>) {
        let child = match child {
            HtmlNodeOrText::AppendNode(node) => NodeOrText::Node(node),
            HtmlNodeOrText::AppendText(text) => NodeOrText::Text(String::from(text)),
        };
        self.send_op(ParseOperation::Append {
            parent: parent.id,
            node: child,
        });
    }

    fn append_doctype_to_document(
        &self,
        name: StrTendril,
        public_id: StrTendril,
        system_id: StrTendril,
    ) {
        self.send_op(ParseOperation::AppendDoctypeToDocument {
            name: String::from(name),
            public_id: String::from(public_id),
            system_id: String::from(system_id),
        });
    }

    fn add_attrs_if_missing(&self, target: &Self::Handle, html_attrs: Vec<HtmlAttribute>) {
        let attrs = html_attrs
            .into_iter()
            .map(|attr| Attribute {
                name: attr.name,
                value: String::from(attr.value),
            })
            .collect();
        self.send_op(ParseOperation::AddAttrsIfMissing {
            target: target.id,
            attrs,
        });
    }

    fn remove_from_parent(&self, target: &Self::Handle) {
        self.send_op(ParseOperation::RemoveFromParent { target: target.id });
    }

    fn mark_script_already_started(&self, node: &Self::Handle) {
        self.send_op(ParseOperation::MarkScriptAlreadyStarted { node: node.id });
    }

    fn complete_script(&self, _: &Self::Handle) -> NextParserState {
        panic!("complete_script should not be called here!");
    }

    fn reparent_children(&self, parent: &Self::Handle, new_parent: &Self::Handle) {
        self.send_op(ParseOperation::ReparentChildren {
            parent: parent.id,
            new_parent: new_parent.id,
        });
    }

    /// <https://html.spec.whatwg.org/multipage/#html-integration-point>
    /// Specifically, the `<annotation-xml>` cases.
    fn is_mathml_annotation_xml_integration_point(&self, handle: &Self::Handle) -> bool {
        let node_data = self.get_parse_node_data(&handle.id);
        node_data.is_integration_point
    }

    fn set_current_line(&self, line_number: u64) {
        self.current_line.set(line_number);
    }

    fn pop(&self, node: &Self::Handle) {
        self.send_op(ParseOperation::Pop { node: node.id });
    }
}
