/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![cfg_attr(crown, expect(crown::unrooted_must_root))]

use std::borrow::Cow;
use std::cell::{Cell, Ref, RefCell, RefMut};
use std::collections::vec_deque::VecDeque;
use std::rc::Rc;
use std::thread;

use crossbeam_channel::{Receiver, Sender, unbounded};
use html5ever::buffer_queue::BufferQueue;
use html5ever::tendril::fmt::UTF8;
use html5ever::tendril::{SendTendril, StrTendril, Tendril};
use html5ever::tokenizer::{Tokenizer as HtmlTokenizer, TokenizerOpts};
use html5ever::tree_builder::{
    ElementFlags, NodeOrText as HtmlNodeOrText, QuirksMode, TreeBuilder, TreeBuilderOpts, TreeSink,
};
use html5ever::{Attribute as HtmlAttribute, ExpandedName, QualName, local_name, ns};
use markup5ever::TokenizerResult;
use rustc_hash::FxHashMap;
use servo_url::ServoUrl;
use style::context::QuirksMode as ServoQuirksMode;

use crate::dom::bindings::codegen::Bindings::HTMLTemplateElementBinding::HTMLTemplateElementMethods;
use crate::dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::comment::Comment;
use crate::dom::customelementregistry::CustomElementReactionStack;
use crate::dom::document::Document;
use crate::dom::documenttype::DocumentType;
use crate::dom::element::{Element, ElementCreator};
use crate::dom::html::htmlformelement::{FormControlElementHelpers, HTMLFormElement};
use crate::dom::html::htmlscriptelement::HTMLScriptElement;
use crate::dom::html::htmltemplateelement::HTMLTemplateElement;
use crate::dom::node::Node;
use crate::dom::processinginstruction::ProcessingInstruction;
use crate::dom::servoparser::{
    ElementAttribute, ParsingAlgorithm, attach_declarative_shadow_inner, create_element_for_token,
};
use crate::dom::virtualmethods::vtable_for;
use crate::script_runtime::CanGc;

type ParseNodeId = usize;

#[derive(Clone, Debug, JSTraceable, MallocSizeOf)]
pub(crate) struct ParseNode {
    id: ParseNodeId,
    #[no_trace]
    qual_name: Option<QualName>,
}

#[derive(Debug, JSTraceable, MallocSizeOf)]
enum NodeOrText {
    Node(ParseNode),
    Text(String),
}

#[derive(Debug, JSTraceable, MallocSizeOf)]
struct Attribute {
    #[no_trace]
    name: QualName,
    value: String,
}

#[derive(Debug, JSTraceable, MallocSizeOf)]
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
    AttachDeclarativeShadowRoot {
        location: ParseNodeId,
        template: ParseNodeId,
        attributes: Vec<Attribute>,
        /// Used to notify the parser thread whether or not attaching the shadow root succeeded
        #[no_trace]
        sender: Sender<bool>,
    },
}

#[derive(MallocSizeOf)]
enum FromParserThreadMsg {
    TokenizerResultDone {
        updated_input: VecDeque<SendTendril<UTF8>>,
    },
    TokenizerResultScript {
        script: ParseNode,
        updated_input: VecDeque<SendTendril<UTF8>>,
    },
    EncodingIndicator(SendTendril<UTF8>),
    /// Sent to main thread to signify that the parser thread's end method has returned.
    End,
    ProcessOperation(ParseOperation),
}

#[derive(MallocSizeOf)]
enum ToParserThreadMsg {
    Feed { input: VecDeque<SendTendril<UTF8>> },
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

// The async HTML Tokenizer consists of two separate types threads working together:
// the main thread, which communicates with the rest of script, and the parser thread, which
// feeds input to the tokenizer from html5ever.
//
// Steps:
// 1. A call to Tokenizer::new will spin up a new parser thread, which starts listening for messages from Tokenizer.
// 2. Upon receiving an input from ServoParser, the tokenizer forwards it to the parser thread, where it starts
//    creating the necessary tree actions based on the input.
// 3. The parser thread sends these tree actions to the main thread as soon as it creates them. The main thread
//    then executes the received actions.
//
//    _____________                           _______________
//   |             |                         |               |
//   |             |                         |               |
//   |             |   ToParserThreadMsg     |               |
//   |             |------------------------>| Parser Thread |
//   |    Main     |                         |               |
//   |   Thread    |   FromParserThreadMsg   |               |
//   |             |<------------------------|    ________   |
//   |             |                         |   |        |  |
//   |             |   FromParserThreadMsg   |   |  Sink  |  |
//   |             |<------------------------|---|        |  |
//   |             |                         |   |________|  |
//   |_____________|                         |_______________|
//
#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) struct Tokenizer {
    document: Dom<Document>,
    #[no_trace]
    from_parser_thread_receiver: Receiver<FromParserThreadMsg>,
    /// Sender from the main thread to the parser thread.
    #[no_trace]
    to_parser_thread_sender: Sender<ToParserThreadMsg>,
    nodes: RefCell<FxHashMap<ParseNodeId, Dom<Node>>>,
    #[no_trace]
    url: ServoUrl,
    parsing_algorithm: ParsingAlgorithm,
    #[conditional_malloc_size_of]
    custom_element_reaction_stack: Rc<CustomElementReactionStack>,
    current_line: Cell<u64>,
    has_ended: Cell<bool>,
}

impl Tokenizer {
    pub(crate) fn new(
        document: &Document,
        url: ServoUrl,
        fragment_context: Option<super::FragmentContext>,
    ) -> Self {
        // Messages from the main thread to the parser thread
        let (to_parser_thread_sender, from_main_thread_receiver) = unbounded();
        // Messages from the parser thread to the main thread
        let (to_main_thread_sender, from_parser_thread_receiver) = unbounded();

        let algorithm = match fragment_context {
            Some(_) => ParsingAlgorithm::Fragment,
            None => ParsingAlgorithm::Normal,
        };

        let custom_element_reaction_stack = document.custom_element_reaction_stack();
        let tokenizer = Tokenizer {
            document: Dom::from_ref(document),
            from_parser_thread_receiver,
            to_parser_thread_sender,
            nodes: RefCell::new(FxHashMap::default()),
            url,
            parsing_algorithm: algorithm,
            custom_element_reaction_stack,
            current_line: Cell::new(1),
            has_ended: Cell::new(false),
        };
        tokenizer.insert_node(0, Dom::from_ref(document.upcast()));

        let sink = Sink::new(
            to_main_thread_sender.clone(),
            document.allow_declarative_shadow_roots(),
        );
        let mut form_parse_node = None;
        let mut parser_fragment_context = None;
        if let Some(fragment_context) = fragment_context {
            let node = sink.new_parse_node();
            tokenizer.insert_node(node.id, Dom::from_ref(fragment_context.context_elem));
            parser_fragment_context =
                Some((node, fragment_context.context_element_allows_scripting));

            form_parse_node = fragment_context.form_elem.map(|form_elem| {
                let node = sink.new_parse_node();
                tokenizer.insert_node(node.id, Dom::from_ref(form_elem));
                node
            });
        };

        // Create new thread for parser. This is where parser actions
        // will be generated from the input provided. These parser actions are then passed
        // onto the main thread to be executed.
        let scripting_enabled = document.has_browsing_context();
        thread::Builder::new()
            .name(format!("Parse:{}", tokenizer.url.debug_compact()))
            .spawn(move || {
                run(
                    sink,
                    parser_fragment_context,
                    form_parse_node,
                    to_main_thread_sender,
                    from_main_thread_receiver,
                    scripting_enabled,
                );
            })
            .expect("HTML Parser thread spawning failed");

        tokenizer
    }

    pub(crate) fn feed(
        &self,
        input: &BufferQueue,
        cx: &mut js::context::JSContext,
    ) -> TokenizerResult<DomRoot<HTMLScriptElement>> {
        let mut send_tendrils = VecDeque::new();
        while let Some(str) = input.pop_front() {
            send_tendrils.push_back(SendTendril::from(str));
        }

        // Send message to parser thread, asking it to start reading from the input.
        // Parser operation messages will be sent to main thread as they are evaluated.
        self.to_parser_thread_sender
            .send(ToParserThreadMsg::Feed {
                input: send_tendrils,
            })
            .unwrap();

        loop {
            debug_assert!(!self.has_ended.get());

            match self
                .from_parser_thread_receiver
                .recv()
                .expect("Unexpected channel panic in main thread.")
            {
                FromParserThreadMsg::ProcessOperation(parse_op) => {
                    self.process_operation(parse_op, cx);

                    // The parser might have been aborted during the execution
                    // of `parse_op`.
                    if self.has_ended.get() {
                        return TokenizerResult::Done;
                    }
                },
                FromParserThreadMsg::TokenizerResultDone { updated_input } => {
                    let buffer_queue = create_buffer_queue(updated_input);
                    input.replace_with(buffer_queue);
                    return TokenizerResult::Done;
                },
                FromParserThreadMsg::TokenizerResultScript {
                    script,
                    updated_input,
                } => {
                    let buffer_queue = create_buffer_queue(updated_input);
                    input.replace_with(buffer_queue);
                    let script = self.get_node(&script.id);
                    return TokenizerResult::Script(DomRoot::from_ref(script.downcast().unwrap()));
                },
                FromParserThreadMsg::EncodingIndicator(_) => continue,
                _ => unreachable!(),
            };
        }
    }

    pub(crate) fn end(&self, cx: &mut js::context::JSContext) {
        if self.has_ended.replace(true) {
            return;
        }

        self.to_parser_thread_sender
            .send(ToParserThreadMsg::End)
            .unwrap();

        loop {
            match self
                .from_parser_thread_receiver
                .recv()
                .expect("Unexpected channel panic in main thread.")
            {
                FromParserThreadMsg::ProcessOperation(parse_op) => {
                    self.process_operation(parse_op, cx);
                },
                FromParserThreadMsg::TokenizerResultDone { updated_input: _ } |
                FromParserThreadMsg::TokenizerResultScript {
                    script: _,
                    updated_input: _,
                } |
                FromParserThreadMsg::EncodingIndicator(_) => continue,
                FromParserThreadMsg::End => return,
            };
        }
    }

    pub(crate) fn url(&self) -> &ServoUrl {
        &self.url
    }

    pub(crate) fn set_plaintext_state(&self) {
        self.to_parser_thread_sender
            .send(ToParserThreadMsg::SetPlainTextState)
            .unwrap();
    }

    pub(crate) fn get_current_line(&self) -> u32 {
        self.current_line.get() as u32
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

        super::insert(
            parent,
            Some(sibling),
            node,
            self.parsing_algorithm,
            &self.custom_element_reaction_stack,
            can_gc,
        );
    }

    fn append(&self, parent: ParseNodeId, node: NodeOrText, can_gc: CanGc) {
        let node = match node {
            NodeOrText::Node(n) => {
                HtmlNodeOrText::AppendNode(Dom::from_ref(&**self.get_node(&n.id)))
            },
            NodeOrText::Text(text) => HtmlNodeOrText::AppendText(Tendril::from(text)),
        };

        let parent = &**self.get_node(&parent);
        super::insert(
            parent,
            None,
            node,
            self.parsing_algorithm,
            &self.custom_element_reaction_stack,
            can_gc,
        );
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

    fn process_operation(&self, op: ParseOperation, cx: &mut js::context::JSContext) {
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
                self.insert_node(
                    contents,
                    Dom::from_ref(template.Content(CanGc::from_cx(cx)).upcast()),
                );
            },
            ParseOperation::CreateElement {
                node,
                name,
                attrs,
                current_line,
            } => {
                self.current_line.set(current_line);
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
                    &self.custom_element_reaction_stack,
                    cx,
                );
                self.insert_node(node, Dom::from_ref(element.upcast()));
            },
            ParseOperation::CreateComment { text, node } => {
                let comment =
                    Comment::new(DOMString::from(text), document, None, CanGc::from_cx(cx));
                self.insert_node(node, Dom::from_ref(comment.upcast()));
            },
            ParseOperation::AppendBeforeSibling { sibling, node } => {
                self.append_before_sibling(sibling, node, CanGc::from_cx(cx));
            },
            ParseOperation::Append { parent, node } => {
                self.append(parent, node, CanGc::from_cx(cx));
            },
            ParseOperation::AppendBasedOnParentNode {
                element,
                prev_element,
                node,
            } => {
                if self.has_parent_node(element) {
                    self.append_before_sibling(element, node, CanGc::from_cx(cx));
                } else {
                    self.append(prev_element, node, CanGc::from_cx(cx));
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
                    CanGc::from_cx(cx),
                );

                document
                    .upcast::<Node>()
                    .AppendChild(doctype.upcast(), CanGc::from_cx(cx))
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
                        CanGc::from_cx(cx),
                    );
                }
            },
            ParseOperation::RemoveFromParent { target } => {
                if let Some(ref parent) = self.get_node(&target).GetParentNode() {
                    parent
                        .RemoveChild(&self.get_node(&target), CanGc::from_cx(cx))
                        .unwrap();
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
                    new_parent.AppendChild(&child, CanGc::from_cx(cx)).unwrap();
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
                    control.set_form_owner_from_parser(&form, CanGc::from_cx(cx));
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
                    CanGc::from_cx(cx),
                );
                self.insert_node(node, Dom::from_ref(pi.upcast()));
            },
            ParseOperation::SetQuirksMode { mode } => {
                document.set_quirks_mode(mode);
            },
            ParseOperation::AttachDeclarativeShadowRoot {
                location,
                template,
                attributes,
                sender,
            } => {
                let location = self.get_node(&location);
                let template = self.get_node(&template);
                let attributes: Vec<_> = attributes
                    .into_iter()
                    .map(|attribute| HtmlAttribute {
                        name: attribute.name,
                        value: StrTendril::from(attribute.value),
                    })
                    .collect();

                let did_succeed =
                    attach_declarative_shadow_inner(&location, &template, &attributes);
                sender.send(did_succeed).unwrap();
            },
        }
    }
}

/// Run the parser.
///
/// The `fragment_context` argument is `Some` in the fragment case and describes the context
/// node as well as whether scripting is enabled for the context node. Note that whether or not
/// scripting is enabled for the context node does not affect whether scripting is enabled for the
/// parser, that is determined by the `scripting_enabled` argument.
fn run(
    sink: Sink,
    fragment_context: Option<(ParseNode, bool)>,
    form_parse_node: Option<ParseNode>,
    sender: Sender<FromParserThreadMsg>,
    receiver: Receiver<ToParserThreadMsg>,
    scripting_enabled: bool,
) {
    let options = TreeBuilderOpts {
        scripting_enabled,
        ..Default::default()
    };

    let html_tokenizer = if let Some((context_node, context_scripting_enabled)) = fragment_context {
        let tree_builder =
            TreeBuilder::new_for_fragment(sink, context_node, form_parse_node, options);

        let tok_options = TokenizerOpts {
            initial_state: Some(
                tree_builder.tokenizer_state_for_context_elem(context_scripting_enabled),
            ),
            ..Default::default()
        };

        HtmlTokenizer::new(tree_builder, tok_options)
    } else {
        HtmlTokenizer::new(TreeBuilder::new(sink, options), Default::default())
    };

    loop {
        match receiver
            .recv()
            .expect("Unexpected channel panic in html parser thread")
        {
            ToParserThreadMsg::Feed { input } => {
                let input = create_buffer_queue(input);
                let res = html_tokenizer.feed(&input);

                // Gather changes to 'input' and place them in 'updated_input',
                // which will be sent to the main thread to update feed method's 'input'
                let mut updated_input = VecDeque::new();
                while let Some(st) = input.pop_front() {
                    updated_input.push_back(SendTendril::from(st));
                }

                let res = match res {
                    TokenizerResult::Done => {
                        FromParserThreadMsg::TokenizerResultDone { updated_input }
                    },
                    TokenizerResult::Script(script) => FromParserThreadMsg::TokenizerResultScript {
                        script,
                        updated_input,
                    },
                    TokenizerResult::EncodingIndicator(encoding) => {
                        FromParserThreadMsg::EncodingIndicator(SendTendril::from(encoding))
                    },
                };
                sender.send(res).unwrap();
            },
            ToParserThreadMsg::End => {
                html_tokenizer.end();
                sender.send(FromParserThreadMsg::End).unwrap();
                break;
            },
            ToParserThreadMsg::SetPlainTextState => html_tokenizer.set_plaintext_state(),
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
    parse_node_data: RefCell<FxHashMap<ParseNodeId, ParseNodeData>>,
    next_parse_node_id: Cell<ParseNodeId>,
    document_node: ParseNode,
    sender: Sender<FromParserThreadMsg>,
    allow_declarative_shadow_roots: bool,
}

impl Sink {
    fn new(sender: Sender<FromParserThreadMsg>, allow_declarative_shadow_roots: bool) -> Sink {
        let sink = Sink {
            current_line: Cell::new(1),
            parse_node_data: RefCell::new(FxHashMap::default()),
            next_parse_node_id: Cell::new(1),
            document_node: ParseNode {
                id: 0,
                qual_name: None,
            },
            sender,
            allow_declarative_shadow_roots,
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
            .send(FromParserThreadMsg::ProcessOperation(op))
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

    fn allow_declarative_shadow_roots(&self, _intended_parent: &Self::Handle) -> bool {
        self.allow_declarative_shadow_roots
    }

    fn attach_declarative_shadow(
        &self,
        location: &Self::Handle,
        template: &Self::Handle,
        attributes: &[HtmlAttribute],
    ) -> bool {
        let attributes = attributes
            .iter()
            .map(|attribute| Attribute {
                name: attribute.name.clone(),
                value: String::from(attribute.value.clone()),
            })
            .collect();

        // Unfortunately the parser can only proceed after it knows whether attaching the shadow root
        // succeeded or failed. Attaching a shadow root can fail for many different reasons,
        // and so we need to block until the script thread has processed this operation.
        let (sender, receiver) = unbounded();
        self.send_op(ParseOperation::AttachDeclarativeShadowRoot {
            location: location.id,
            template: template.id,
            attributes,
            sender,
        });

        receiver.recv().unwrap()
    }
}
