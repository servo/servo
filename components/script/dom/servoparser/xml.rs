/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![cfg_attr(crown, expect(crown::unrooted_must_root))]

use std::cell::Cell;
use std::io::{self, Write};

use markup5ever::serialize::AttrRef;
use markup5ever::{Namespace, QualName, TokenizerResult, local_name, ns};
use script_bindings::codegen::GenericBindings::NodeBinding::NodeMethods;
use script_bindings::script_runtime::temp_cx;
use script_bindings::trace::CustomTraceable;
use servo_url::ServoUrl;
use xml5ever::buffer_queue::BufferQueue;
use xml5ever::serialize::TraversalScope::IncludeNode;
use xml5ever::serialize::{NamespacePrefixMap, TraversalScope, XmlSerializer};
use xml5ever::tokenizer::XmlTokenizer;
use xml5ever::tree_builder::XmlTreeBuilder;

use crate::dom::attr::Attr;
use crate::dom::bindings::codegen::Bindings::HTMLTemplateElementBinding::HTMLTemplateElementMethods;
use crate::dom::bindings::inheritance::{Castable, CharacterDataTypeId, NodeTypeId};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::characterdata::CharacterData;
use crate::dom::document::Document;
use crate::dom::documentfragment::DocumentFragment;
use crate::dom::documenttype::DocumentType;
use crate::dom::element::Element;
use crate::dom::html::htmlscriptelement::HTMLScriptElement;
use crate::dom::html::htmltemplateelement::HTMLTemplateElement;
use crate::dom::node::Node;
use crate::dom::processinginstruction::ProcessingInstruction;
use crate::dom::servoparser::{ParsingAlgorithm, Sink};

#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) struct Tokenizer {
    #[ignore_malloc_size_of = "Defined in xml5ever"]
    inner: XmlTokenizer<XmlTreeBuilder<Dom<Node>, Sink>>,
}

impl Tokenizer {
    pub(crate) fn new(document: &Document, url: ServoUrl) -> Self {
        let sink = Sink {
            base_url: url,
            document: Dom::from_ref(document),
            current_line: Cell::new(1),
            script: Default::default(),
            parsing_algorithm: ParsingAlgorithm::Normal,
            custom_element_reaction_stack: document.custom_element_reaction_stack(),
        };

        let tb = XmlTreeBuilder::new(sink, Default::default());
        let tok = XmlTokenizer::new(tb, Default::default());

        Tokenizer { inner: tok }
    }

    pub(crate) fn feed(&self, input: &BufferQueue) -> TokenizerResult<DomRoot<HTMLScriptElement>> {
        loop {
            match self.inner.run(input) {
                TokenizerResult::Done => return TokenizerResult::Done,
                TokenizerResult::Script(handle) => {
                    // Apparently the parser can sometimes create <script> elements without a namespace, resulting
                    // in them not being HTMLScriptElements.
                    if let Some(script) = handle.downcast::<HTMLScriptElement>() {
                        return TokenizerResult::Script(DomRoot::from_ref(script));
                    }
                },
                TokenizerResult::EncodingIndicator(encoding) => {
                    return TokenizerResult::EncodingIndicator(encoding);
                },
            }
        }
    }

    pub(crate) fn end(&self) {
        self.inner.end()
    }

    pub(crate) fn url(&self) -> &ServoUrl {
        &self.inner.sink.sink.base_url
    }

    pub(crate) fn get_current_line(&self) -> u32 {
        self.inner.sink.sink.current_line.get() as u32
    }
}

#[derive(Debug)]
enum SerializationCommand {
    SerializeNode {
        node: DomRoot<Node>,
        namespace: Namespace,
        prefix_map: NamespacePrefixMap,
    },
    CloseElement(String),
}

fn serialize_xml_fragment<Wr: Write>(
    cx: &mut js::context::JSContext,
    node: &Node,
    serializer: &mut XmlSerializer<Wr>,
    traversal_scope: TraversalScope,
) -> io::Result<()> {
    debug_assert!(!node.is::<Attr>(), "Should have handled Attr in caller");
    let mut stack = Vec::new();
    fn push_node(
        stack: &mut Vec<SerializationCommand>,
        node: &Node,
        namespace: Namespace,
        prefix_map: NamespacePrefixMap,
    ) {
        stack.push(SerializationCommand::SerializeNode {
            node: DomRoot::from_ref(node),
            namespace,
            prefix_map,
        });
    }
    fn push_children(
        stack: &mut Vec<SerializationCommand>,
        cx: &mut js::context::JSContext,
        node: &Node,
        namespace: Namespace,
        prefix_map: NamespacePrefixMap,
    ) {
        if let Some(template_element) = node.downcast::<HTMLTemplateElement>() {
            for child in template_element.Content(cx).upcast::<Node>().rev_children() {
                push_node(stack, &child, namespace.clone(), prefix_map.clone());
            }
        } else {
            for child in node.rev_children() {
                push_node(stack, &child, namespace.clone(), prefix_map.clone());
            }
        }
    }

    let namespace = ns!();
    let prefix_map = NamespacePrefixMap::default();
    if traversal_scope != IncludeNode || node.is::<DocumentFragment>() || node.is::<Document>() {
        push_children(&mut stack, cx, &node, namespace, prefix_map);
    } else {
        push_node(&mut stack, &node, namespace, prefix_map);
    }

    while let Some(command) = stack.pop() {
        match command {
            SerializationCommand::SerializeNode {
                node: n,
                namespace,
                prefix_map,
            } => {
                match n.type_id() {
                    NodeTypeId::Element(_) => {
                        let element = n.downcast::<Element>().unwrap();
                        let has_children = n.HasChildNodes() ||
                            (element.is_html_element() &&
                                *element.local_name() == local_name!("template"));
                        // TODO: would be nice to have a getter on Element for this
                        let name = QualName::new(
                            element.prefix().clone(),
                            element.namespace().clone(),
                            element.local_name().clone(),
                        );

                        let attributes: Vec<_> = element
                            .attrs()
                            .borrow()
                            .iter()
                            .map(|attr| {
                                let qname = QualName::new(
                                    attr.prefix().cloned(),
                                    attr.namespace().clone(),
                                    attr.local_name().clone(),
                                );
                                let value = attr.value().clone();
                                (qname, value)
                            })
                            .collect();
                        let attr_refs = attributes.iter().map(|(qname, value)| {
                            let ar: AttrRef = (qname, &**value);
                            ar
                        });
                        if has_children {
                            let (qualified_name, inherit_ns, inherit_prefix_map) =
                                serializer.start_elem(&name, attr_refs, namespace, &prefix_map)?;
                            stack.push(SerializationCommand::CloseElement(qualified_name.clone()));
                            push_children(&mut stack, cx, &n, inherit_ns, inherit_prefix_map);
                        } else {
                            serializer.write_empty_elem(
                                &name,
                                attr_refs,
                                namespace,
                                &prefix_map,
                            )?;
                        }
                    },

                    NodeTypeId::DocumentType => {
                        let doctype = n.downcast::<DocumentType>().unwrap();
                        serializer.write_doctype(&doctype.name().str())?;
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
                        serializer.write_processing_instruction(&pi.target().str(), &data)?;
                    },

                    NodeTypeId::Attr => panic!("Should not encounter Attr while serializing"),
                    NodeTypeId::DocumentFragment(_) => {
                        panic!("Should not encounter DocumentFragment while serializing")
                    },
                    NodeTypeId::Document(_) => {
                        panic!("Should not encounter Document while serializing")
                    },
                }
            },
            SerializationCommand::CloseElement(qualified_name) => {
                serializer.end_elem(qualified_name)?;
            },
        }
    }

    Ok(())
}

#[expect(unsafe_code)]
pub fn serialize_xml(root: &Node, traversal_scope: TraversalScope) -> io::Result<DOMString> {
    if root.is::<Attr>() {
        return Ok(DOMString::new());
    }
    let mut writer = vec![];
    let mut ser = XmlSerializer::new(&mut writer);
    {
        // TODO: https://github.com/servo/servo/issues/42839
        let mut cx = unsafe { temp_cx() };
        let cx = &mut cx;
        serialize_xml_fragment(cx, root, &mut ser, traversal_scope)?;
    }
    // FIXME(ajeffrey): Directly convert UTF8 to DOMString
    Ok(DOMString::from(String::from_utf8(writer).unwrap()))
}
