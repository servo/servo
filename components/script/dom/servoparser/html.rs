/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![cfg_attr(crown, allow(crown::unrooted_must_root))]

use std::cell::Cell;
use std::io;

use html5ever::buffer_queue::BufferQueue;
use html5ever::serialize::TraversalScope::IncludeNode;
use html5ever::serialize::{AttrRef, Serialize, Serializer, TraversalScope};
use html5ever::tokenizer::{Tokenizer as HtmlTokenizer, TokenizerOpts};
use html5ever::tree_builder::{QuirksMode as HTML5EverQuirksMode, TreeBuilder, TreeBuilderOpts};
use html5ever::{QualName, local_name, ns};
use markup5ever::TokenizerResult;
use script_bindings::trace::CustomTraceable;
use servo_url::ServoUrl;
use style::attr::AttrValue;
use style::context::QuirksMode as StyleContextQuirksMode;
use xml5ever::LocalName;

use crate::dom::bindings::codegen::Bindings::HTMLTemplateElementBinding::HTMLTemplateElementMethods;
use crate::dom::bindings::codegen::Bindings::ShadowRootBinding::ShadowRootMode;
use crate::dom::bindings::codegen::GenericBindings::ShadowRootBinding::ShadowRoot_Binding::ShadowRootMethods;
use crate::dom::bindings::inheritance::{Castable, CharacterDataTypeId, NodeTypeId};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::characterdata::CharacterData;
use crate::dom::document::Document;
use crate::dom::documentfragment::DocumentFragment;
use crate::dom::documenttype::DocumentType;
use crate::dom::element::Element;
use crate::dom::htmlscriptelement::HTMLScriptElement;
use crate::dom::htmltemplateelement::HTMLTemplateElement;
use crate::dom::node::Node;
use crate::dom::processinginstruction::ProcessingInstruction;
use crate::dom::servoparser::{ParsingAlgorithm, Sink};
use crate::dom::shadowroot::ShadowRoot;
use crate::script_runtime::CanGc;

#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) struct Tokenizer {
    #[ignore_malloc_size_of = "Defined in html5ever"]
    inner: HtmlTokenizer<TreeBuilder<Dom<Node>, Sink>>,
}

impl Tokenizer {
    pub(crate) fn new(
        document: &Document,
        url: ServoUrl,
        fragment_context: Option<super::FragmentContext>,
        parsing_algorithm: ParsingAlgorithm,
    ) -> Self {
        let sink = Sink {
            base_url: url,
            document: Dom::from_ref(document),
            current_line: Cell::new(1),
            script: Default::default(),
            parsing_algorithm,
        };

        let quirks_mode = match document.quirks_mode() {
            StyleContextQuirksMode::Quirks => HTML5EverQuirksMode::Quirks,
            StyleContextQuirksMode::LimitedQuirks => HTML5EverQuirksMode::LimitedQuirks,
            StyleContextQuirksMode::NoQuirks => HTML5EverQuirksMode::NoQuirks,
        };

        let options = TreeBuilderOpts {
            scripting_enabled: document.scripting_enabled(),
            iframe_srcdoc: document.url().as_str() == "about:srcdoc",
            quirks_mode,
            ..Default::default()
        };

        let inner = if let Some(fragment_context) = fragment_context {
            let tree_builder = TreeBuilder::new_for_fragment(
                sink,
                Dom::from_ref(fragment_context.context_elem),
                fragment_context.form_elem.map(Dom::from_ref),
                options,
            );

            let tokenizer_options = TokenizerOpts {
                initial_state: Some(tree_builder.tokenizer_state_for_context_elem(
                    fragment_context.context_element_allows_scripting,
                )),
                ..Default::default()
            };

            HtmlTokenizer::new(tree_builder, tokenizer_options)
        } else {
            HtmlTokenizer::new(TreeBuilder::new(sink, options), Default::default())
        };

        Tokenizer { inner }
    }

    pub(crate) fn feed(&self, input: &BufferQueue) -> TokenizerResult<DomRoot<HTMLScriptElement>> {
        match self.inner.feed(input) {
            TokenizerResult::Done => TokenizerResult::Done,
            TokenizerResult::Script(script) => {
                TokenizerResult::Script(DomRoot::from_ref(script.downcast().unwrap()))
            },
        }
    }

    pub(crate) fn end(&self) {
        self.inner.end();
    }

    pub(crate) fn url(&self) -> &ServoUrl {
        &self.inner.sink.sink.base_url
    }

    pub(crate) fn set_plaintext_state(&self) {
        self.inner.set_plaintext_state();
    }
}

/// <https://html.spec.whatwg.org/multipage/#html-fragment-serialisation-algorithm>
fn start_element<S: Serializer>(element: &Element, serializer: &mut S) -> io::Result<()> {
    let name = QualName::new(
        None,
        element.namespace().clone(),
        element.local_name().clone(),
    );

    let mut attributes = vec![];

    // The "is" value of an element is treated as if it was an attribute and it is serialized before all
    // other attributes. If the element already has an "is" attribute then the "is" value is ignored.
    if !element.has_attribute(&LocalName::from("is")) {
        if let Some(is_value) = element.get_is() {
            let qualified_name = QualName::new(None, ns!(), LocalName::from("is"));

            attributes.push((qualified_name, AttrValue::String(is_value.to_string())));
        }
    }

    // Collect all the "normal" attributes
    attributes.extend(element.attrs().iter().map(|attr| {
        let qname = QualName::new(None, attr.namespace().clone(), attr.local_name().clone());
        let value = attr.value().clone();
        (qname, value)
    }));

    let attr_refs = attributes.iter().map(|(qname, value)| {
        let ar: AttrRef = (qname, &**value);
        ar
    });
    serializer.start_elem(name, attr_refs)?;
    Ok(())
}

enum SerializationCommand {
    OpenElement(DomRoot<Element>),
    CloseElement(QualName),
    SerializeNonelement(DomRoot<Node>),
    SerializeShadowRoot(DomRoot<ShadowRoot>),
}

struct SerializationIterator {
    stack: Vec<SerializationCommand>,

    /// Whether or not shadow roots should be serialized
    serialize_shadow_roots: bool,

    /// List of shadow root objects that should be serialized
    shadow_roots: Vec<DomRoot<ShadowRoot>>,
}

enum SerializationChildrenIterator<C, S> {
    None,
    Children(C),
    ShadowContents(S),
}

impl SerializationIterator {
    fn new(
        node: &Node,
        skip_first: bool,
        serialize_shadow_roots: bool,
        shadow_roots: Vec<DomRoot<ShadowRoot>>,
        can_gc: CanGc,
    ) -> SerializationIterator {
        let mut ret = SerializationIterator {
            stack: vec![],
            serialize_shadow_roots,
            shadow_roots,
        };
        if skip_first || node.is::<DocumentFragment>() || node.is::<Document>() {
            ret.handle_node_contents(node, can_gc);
        } else {
            ret.push_node(node);
        }
        ret
    }

    fn handle_node_contents(&mut self, node: &Node, can_gc: CanGc) {
        if node.downcast::<Element>().is_some_and(Element::is_void) {
            return;
        }

        if let Some(template_element) = node.downcast::<HTMLTemplateElement>() {
            for child in template_element
                .Content(can_gc)
                .upcast::<Node>()
                .rev_children()
            {
                self.push_node(&child);
            }
        } else {
            for child in node.rev_children() {
                self.push_node(&child);
            }
        }

        if let Some(shadow_root) = node.downcast::<Element>().and_then(Element::shadow_root) {
            let should_be_serialized = (self.serialize_shadow_roots && shadow_root.Serializable()) ||
                self.shadow_roots.contains(&shadow_root);
            if !shadow_root.is_user_agent_widget() && should_be_serialized {
                self.stack
                    .push(SerializationCommand::SerializeShadowRoot(shadow_root));
            }
        }
    }

    fn push_node(&mut self, node: &Node) {
        let Some(element) = node.downcast::<Element>() else {
            self.stack.push(SerializationCommand::SerializeNonelement(
                DomRoot::from_ref(node),
            ));
            return;
        };

        self.stack
            .push(SerializationCommand::OpenElement(DomRoot::from_ref(
                element,
            )));
    }
}

impl Iterator for SerializationIterator {
    type Item = SerializationCommand;

    fn next(&mut self) -> Option<SerializationCommand> {
        let res = self.stack.pop()?;

        match &res {
            SerializationCommand::OpenElement(element) => {
                let name = QualName::new(
                    None,
                    element.namespace().clone(),
                    element.local_name().clone(),
                );
                self.stack.push(SerializationCommand::CloseElement(name));
                self.handle_node_contents(element.upcast(), CanGc::note());
            },
            SerializationCommand::SerializeShadowRoot(shadow_root) => {
                self.stack
                    .push(SerializationCommand::CloseElement(QualName::new(
                        None,
                        ns!(),
                        local_name!("template"),
                    )));
                self.handle_node_contents(shadow_root.upcast(), CanGc::note());
            },
            _ => {},
        }

        Some(res)
    }
}

/// <https://html.spec.whatwg.org/multipage/#html-fragment-serialisation-algorithm>
pub(crate) fn serialize_html_fragment<S: Serializer>(
    node: &Node,
    serializer: &mut S,
    traversal_scope: TraversalScope,
    serialize_shadow_roots: bool,
    shadow_roots: Vec<DomRoot<ShadowRoot>>,
    can_gc: CanGc,
) -> io::Result<()> {
    let iter = SerializationIterator::new(
        node,
        traversal_scope != IncludeNode,
        serialize_shadow_roots,
        shadow_roots,
        can_gc,
    );

    for cmd in iter {
        match cmd {
            SerializationCommand::OpenElement(n) => {
                start_element(&n, serializer)?;
            },
            SerializationCommand::CloseElement(name) => {
                serializer.end_elem(name)?;
            },
            SerializationCommand::SerializeNonelement(n) => match n.type_id() {
                NodeTypeId::DocumentType => {
                    let doctype = n.downcast::<DocumentType>().unwrap();
                    serializer.write_doctype(doctype.name())?;
                },

                NodeTypeId::CharacterData(CharacterDataTypeId::Text(_)) => {
                    let cdata = n.downcast::<CharacterData>().unwrap();
                    serializer.write_text(&cdata.data())?;
                },

                NodeTypeId::CharacterData(CharacterDataTypeId::Comment) => {
                    let cdata = n.downcast::<CharacterData>().unwrap();
                    serializer.write_comment(&cdata.data())?;
                },

                NodeTypeId::CharacterData(CharacterDataTypeId::ProcessingInstruction) => {
                    let pi = n.downcast::<ProcessingInstruction>().unwrap();
                    let data = pi.upcast::<CharacterData>().data();
                    serializer.write_processing_instruction(pi.target(), &data)?;
                },

                NodeTypeId::DocumentFragment(_) | NodeTypeId::Attr => {},

                NodeTypeId::Document(_) => panic!("Can't serialize Document node itself"),
                NodeTypeId::Element(_) => panic!("Element shouldn't appear here"),
            },
            SerializationCommand::SerializeShadowRoot(shadow_root) => {
                // Shadow roots are serialized as template elements with a fixed set of
                // attributes. Because these template elements don't actually exist in the DOM
                // we have to make up a vector of attributes ourselves.
                let mut attributes = vec![];
                let mut push_attribute = |name, value| {
                    let qualified_name = QualName::new(None, ns!(), LocalName::from(name));
                    attributes.push((qualified_name, value))
                };

                let mode = if shadow_root.Mode() == ShadowRootMode::Open {
                    "open"
                } else {
                    "closed"
                };
                push_attribute("shadowrootmode", mode);

                if shadow_root.DelegatesFocus() {
                    push_attribute("shadowrootdelegatesfocus", "");
                }

                if shadow_root.Serializable() {
                    push_attribute("shadowrootserializable", "");
                }

                if shadow_root.Clonable() {
                    push_attribute("shadowrootclonable", "");
                }

                let name = QualName::new(None, ns!(), local_name!("template"));
                serializer.start_elem(name, attributes.iter().map(|(a, b)| (a, *b)))?;
            },
        }
    }

    Ok(())
}

// TODO: This trait confuses the concepts of XML serialization and HTML serialization and
// the impl should go away eventually
impl Serialize for &Node {
    fn serialize<S>(&self, serializer: &mut S, traversal_scope: TraversalScope) -> io::Result<()>
    where
        S: Serializer,
    {
        serialize_html_fragment(
            self,
            serializer,
            traversal_scope,
            false,
            vec![],
            CanGc::note(),
        )
    }
}
