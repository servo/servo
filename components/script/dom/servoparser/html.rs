/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![allow(unrooted_must_root)]

use document_loader::DocumentLoader;
use dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use dom::bindings::codegen::Bindings::HTMLTemplateElementBinding::HTMLTemplateElementMethods;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::inheritance::{Castable, CharacterDataTypeId, NodeTypeId};
use dom::bindings::js::{JS, RootedReference};
use dom::bindings::str::DOMString;
use dom::characterdata::CharacterData;
use dom::comment::Comment;
use dom::document::{DocumentSource, IsHTMLDocument};
use dom::document::Document;
use dom::documenttype::DocumentType;
use dom::element::{Element, ElementCreator};
use dom::htmlformelement::HTMLFormElement;
use dom::htmlscriptelement::HTMLScriptElement;
use dom::htmltemplateelement::HTMLTemplateElement;
use dom::node::{document_from_node, window_from_node};
use dom::node::Node;
use dom::processinginstruction::ProcessingInstruction;
use dom::text::Text;
use html5ever::Attribute;
use html5ever::serialize::{AttrRef, Serializable, Serializer};
use html5ever::serialize::TraversalScope;
use html5ever::serialize::TraversalScope::{ChildrenOnly, IncludeNode};
use html5ever::tendril::StrTendril;
use html5ever::tokenizer::{Tokenizer as H5ETokenizer, TokenizerOpts};
use html5ever::tree_builder::{NodeOrText, QuirksMode};
use html5ever::tree_builder::{TreeBuilder, TreeBuilderOpts, TreeSink};
use html5ever_atoms::QualName;
use msg::constellation_msg::PipelineId;
use std::borrow::Cow;
use std::io::{self, Write};
use super::{HtmlTokenizer, LastChunkState, ServoParser, Sink, Tokenizer};
use url::Url;

fn insert(parent: &Node, reference_child: Option<&Node>, child: NodeOrText<JS<Node>>) {
    match child {
        NodeOrText::AppendNode(n) => {
            assert!(parent.InsertBefore(&n, reference_child).is_ok());
        },
        NodeOrText::AppendText(t) => {
            // FIXME(ajeffrey): convert directly from tendrils to DOMStrings
            let s: String = t.into();
            let text = Text::new(DOMString::from(s), &parent.owner_doc());
            assert!(parent.InsertBefore(text.upcast(), reference_child).is_ok());
        }
    }
}

impl<'a> TreeSink for Sink {
    type Output = Self;
    fn finish(self) -> Self { self }

    type Handle = JS<Node>;

    fn get_document(&mut self) -> JS<Node> {
        JS::from_ref(self.document.upcast())
    }

    fn get_template_contents(&mut self, target: JS<Node>) -> JS<Node> {
        let template = target.downcast::<HTMLTemplateElement>()
            .expect("tried to get template contents of non-HTMLTemplateElement in HTML parsing");
        JS::from_ref(template.Content().upcast())
    }

    fn same_node(&self, x: JS<Node>, y: JS<Node>) -> bool {
        x == y
    }

    fn elem_name(&self, target: JS<Node>) -> QualName {
        let elem = target.downcast::<Element>()
            .expect("tried to get name of non-Element in HTML parsing");
        QualName {
            ns: elem.namespace().clone(),
            local: elem.local_name().clone(),
        }
    }

    fn create_element(&mut self, name: QualName, attrs: Vec<Attribute>)
            -> JS<Node> {
        let elem = Element::create(name, None, &*self.document,
                                   ElementCreator::ParserCreated);

        for attr in attrs {
            elem.set_attribute_from_parser(attr.name, DOMString::from(String::from(attr.value)), None);
        }

        JS::from_ref(elem.upcast())
    }

    fn create_comment(&mut self, text: StrTendril) -> JS<Node> {
        let comment = Comment::new(DOMString::from(String::from(text)), &*self.document);
        JS::from_ref(comment.upcast())
    }

    fn append_before_sibling(&mut self,
            sibling: JS<Node>,
            new_node: NodeOrText<JS<Node>>) -> Result<(), NodeOrText<JS<Node>>> {
        // If there is no parent, return the node to the parser.
        let parent = match sibling.GetParentNode() {
            Some(p) => p,
            None => return Err(new_node),
        };

        insert(&parent, Some(&*sibling), new_node);
        Ok(())
    }

    fn parse_error(&mut self, msg: Cow<'static, str>) {
        debug!("Parse error: {}", msg);
    }

    fn set_quirks_mode(&mut self, mode: QuirksMode) {
        self.document.set_quirks_mode(mode);
    }

    fn append(&mut self, parent: JS<Node>, child: NodeOrText<JS<Node>>) {
        // FIXME(#3701): Use a simpler algorithm and merge adjacent text nodes
        insert(&parent, None, child);
    }

    fn append_doctype_to_document(&mut self, name: StrTendril, public_id: StrTendril,
                                  system_id: StrTendril) {
        let doc = &*self.document;
        let doctype = DocumentType::new(
            DOMString::from(String::from(name)), Some(DOMString::from(String::from(public_id))),
            Some(DOMString::from(String::from(system_id))), doc);
        doc.upcast::<Node>().AppendChild(doctype.upcast()).expect("Appending failed");
    }

    fn add_attrs_if_missing(&mut self, target: JS<Node>, attrs: Vec<Attribute>) {
        let elem = target.downcast::<Element>()
            .expect("tried to set attrs on non-Element in HTML parsing");
        for attr in attrs {
            elem.set_attribute_from_parser(attr.name, DOMString::from(String::from(attr.value)), None);
        }
    }

    fn remove_from_parent(&mut self, target: JS<Node>) {
        if let Some(ref parent) = target.GetParentNode() {
            parent.RemoveChild(&*target).unwrap();
        }
    }

    fn mark_script_already_started(&mut self, node: JS<Node>) {
        let script = node.downcast::<HTMLScriptElement>();
        script.map(|script| script.set_already_started(true));
    }

    fn reparent_children(&mut self, node: JS<Node>, new_parent: JS<Node>) {
        while let Some(ref child) = node.GetFirstChild() {
            new_parent.AppendChild(&child).unwrap();
        }

    }
}

impl<'a> Serializable for &'a Node {
    fn serialize<'wr, Wr: Write>(&self, serializer: &mut Serializer<'wr, Wr>,
                                 traversal_scope: TraversalScope) -> io::Result<()> {
        let node = *self;
        match (traversal_scope, node.type_id()) {
            (_, NodeTypeId::Element(..)) => {
                let elem = node.downcast::<Element>().unwrap();
                let name = QualName::new(elem.namespace().clone(),
                                         elem.local_name().clone());
                if traversal_scope == IncludeNode {
                    let attrs = elem.attrs().iter().map(|attr| {
                        let qname = QualName::new(attr.namespace().clone(),
                                                  attr.local_name().clone());
                        let value = attr.value().clone();
                        (qname, value)
                    }).collect::<Vec<_>>();
                    let attr_refs = attrs.iter().map(|&(ref qname, ref value)| {
                        let ar: AttrRef = (&qname, &**value);
                        ar
                    });
                    try!(serializer.start_elem(name.clone(), attr_refs));
                }

                let children = if let Some(tpl) = node.downcast::<HTMLTemplateElement>() {
                    // https://github.com/w3c/DOM-Parsing/issues/1
                    tpl.Content().upcast::<Node>().children()
                } else {
                    node.children()
                };

                for handle in children {
                    try!((&*handle).serialize(serializer, IncludeNode));
                }

                if traversal_scope == IncludeNode {
                    try!(serializer.end_elem(name.clone()));
                }
                Ok(())
            },

            (ChildrenOnly, NodeTypeId::Document(_)) => {
                for handle in node.children() {
                    try!((&*handle).serialize(serializer, IncludeNode));
                }
                Ok(())
            },

            (ChildrenOnly, _) => Ok(()),

            (IncludeNode, NodeTypeId::DocumentType) => {
                let doctype = node.downcast::<DocumentType>().unwrap();
                serializer.write_doctype(&doctype.name())
            },

            (IncludeNode, NodeTypeId::CharacterData(CharacterDataTypeId::Text)) => {
                let cdata = node.downcast::<CharacterData>().unwrap();
                serializer.write_text(&cdata.data())
            },

            (IncludeNode, NodeTypeId::CharacterData(CharacterDataTypeId::Comment)) => {
                let cdata = node.downcast::<CharacterData>().unwrap();
                serializer.write_comment(&cdata.data())
            },

            (IncludeNode, NodeTypeId::CharacterData(CharacterDataTypeId::ProcessingInstruction)) => {
                let pi = node.downcast::<ProcessingInstruction>().unwrap();
                let data = pi.upcast::<CharacterData>().data();
                serializer.write_processing_instruction(&pi.target(), &data)
            },

            (IncludeNode, NodeTypeId::DocumentFragment) => Ok(()),

            (IncludeNode, NodeTypeId::Document(_)) => panic!("Can't serialize Document node itself"),
        }
    }
}

/// FragmentContext is used only to pass this group of related values
/// into functions.
#[derive(Copy, Clone)]
pub struct FragmentContext<'a> {
    pub context_elem: &'a Node,
    pub form_elem: Option<&'a Node>,
}

pub enum ParseContext<'a> {
    Fragment(FragmentContext<'a>),
    Owner(Option<PipelineId>),
}

pub fn parse_html(document: &Document,
                  input: DOMString,
                  url: Url,
                  context: ParseContext) {
    let sink = Sink {
        base_url: url,
        document: JS::from_ref(document),
    };

    let options = TreeBuilderOpts {
        ignore_missing_rules: true,
        .. Default::default()
    };

    let parser = match context {
        ParseContext::Owner(owner) => {
            let tb = TreeBuilder::new(sink, options);
            let tok = H5ETokenizer::new(tb, Default::default());

            ServoParser::new(
                document,
                owner,
                Tokenizer::HTML(HtmlTokenizer::new(tok)),
                LastChunkState::NotReceived)
        },
        ParseContext::Fragment(fc) => {
            let tb = TreeBuilder::new_for_fragment(
                sink,
                JS::from_ref(fc.context_elem),
                fc.form_elem.map(|n| JS::from_ref(n)),
                options);

            let tok_options = TokenizerOpts {
                initial_state: Some(tb.tokenizer_state_for_context_elem()),
                .. Default::default()
            };
            let tok = H5ETokenizer::new(tb, tok_options);

            ServoParser::new(
                document,
                None,
                Tokenizer::HTML(HtmlTokenizer::new(tok)),
                LastChunkState::Received)
        }
    };
    parser.parse_chunk(String::from(input));
}

// https://html.spec.whatwg.org/multipage/#parsing-html-fragments
pub fn parse_html_fragment(context_node: &Node,
                           input: DOMString,
                           output: &Node) {
    let window = window_from_node(context_node);
    let context_document = document_from_node(context_node);
    let url = context_document.url();

    // Step 1.
    let loader = DocumentLoader::new(&*context_document.loader());
    let document = Document::new(&window, None, Some(url.clone()),
                                 IsHTMLDocument::HTMLDocument,
                                 None, None,
                                 DocumentSource::FromParser,
                                 loader,
                                 None, None);

    // Step 2.
    document.set_quirks_mode(context_document.quirks_mode());

    // Step 11.
    let form = context_node.inclusive_ancestors()
                           .find(|element| element.is::<HTMLFormElement>());
    let fragment_context = FragmentContext {
        context_elem: context_node,
        form_elem: form.r(),
    };
    parse_html(&document, input, url.clone(), ParseContext::Fragment(fragment_context));

    // Step 14.
    let root_element = document.GetDocumentElement().expect("no document element");
    for child in root_element.upcast::<Node>().children() {
        output.AppendChild(&child).unwrap();
    }
}
