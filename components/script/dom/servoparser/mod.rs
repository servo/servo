/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use document_loader::{DocumentLoader, LoadType};
use dom::bindings::cell::DomRefCell;
use dom::bindings::codegen::Bindings::DocumentBinding::{DocumentMethods, DocumentReadyState};
use dom::bindings::codegen::Bindings::HTMLImageElementBinding::HTMLImageElementMethods;
use dom::bindings::codegen::Bindings::HTMLTemplateElementBinding::HTMLTemplateElementMethods;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::codegen::Bindings::ServoParserBinding;
use dom::bindings::inheritance::Castable;
use dom::bindings::refcounted::Trusted;
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::root::{Dom, DomRoot, MutNullableDom, RootedReference};
use dom::bindings::settings_stack::is_execution_stack_empty;
use dom::bindings::str::DOMString;
use dom::characterdata::CharacterData;
use dom::comment::Comment;
use dom::document::{Document, DocumentSource, HasBrowsingContext, IsHTMLDocument};
use dom::documenttype::DocumentType;
use dom::element::{Element, ElementCreator, CustomElementCreationMode};
use dom::globalscope::GlobalScope;
use dom::htmlformelement::{FormControlElementHelpers, HTMLFormElement};
use dom::htmlimageelement::HTMLImageElement;
use dom::htmlscriptelement::{HTMLScriptElement, ScriptResult};
use dom::htmltemplateelement::HTMLTemplateElement;
use dom::node::Node;
use dom::processinginstruction::ProcessingInstruction;
use dom::text::Text;
use dom::virtualmethods::vtable_for;
use dom_struct::dom_struct;
use embedder_traits::resources::{self, Resource};
use html5ever::{Attribute, ExpandedName, LocalName, QualName};
use html5ever::buffer_queue::BufferQueue;
use html5ever::tendril::{StrTendril, ByteTendril, IncompleteUtf8};
use html5ever::tree_builder::{NodeOrText, TreeSink, NextParserState, QuirksMode, ElementFlags};
use html5ever::serialize::TraversalScope::IncludeNode;
use html5ever::serialize::{Serializer, Serialize, TraversalScope};
use hyper::header::ContentType;
use hyper::mime::{Mime, SubLevel, TopLevel};
use hyper_serde::Serde;
use msg::constellation_msg::PipelineId;
use net_traits::{FetchMetadata, FetchResponseListener, Metadata, NetworkError};
use network_listener::PreInvoke;
use profile_traits::time::{TimerMetadata, TimerMetadataFrameType};
use profile_traits::time::{TimerMetadataReflowType, ProfilerCategory, profile};
use script_thread::ScriptThread;
use script_traits::DocumentActivity;
use servo_config::prefs::PREFS;
use servo_url::ServoUrl;
use std::borrow::Cow;
use std::cell::Cell;
use std::mem;
use style::context::QuirksMode as ServoQuirksMode;
use dom::bindings::reflector::DomObject;
use dom::bindings::reflector::MutDomObject;
use dom::bindings::root::StableTraceObject;
use malloc_size_of::MallocSizeOf;
use dom::bindings::trace::JSTraceable;
use typeholder::TypeHolderTrait;
use std::marker::Sized;
use dom::bindings::conversions::IDLInterface;
use std::io;
use dom::bindings::inheritance::{CharacterDataTypeId, NodeTypeId};
use html5ever::serialize::AttrRef;
use xml5ever::tree_builder::{Tracer as XmlTracer, XmlTreeBuilder};
use html5ever::tree_builder::{Tracer as HtmlTracer, TreeBuilder, TreeBuilderOpts};
use html5ever::tokenizer::Tokenizer as HtmlTokenizer;
use std::marker::PhantomData;
use js::jsapi::JSTracer;
use xml5ever::tokenizer::XmlTokenizer;

pub trait ServoParser<TH: TypeHolderTrait>: DomObject<TypeHolder=TH> + MutDomObject + MallocSizeOf + JSTraceable + IDLInterface + 'static {

    fn new_inherited(document: &Document<TH>, tokenizer: Tokenizer<TH>, last_chunk_state: LastChunkState, kind: ParserKind) -> Self where Self: Sized;

    fn tokenize<F>(&self, mut feed: F)
        where F: FnMut(&mut Tokenizer<TH>) -> Result<(), DomRoot<HTMLScriptElement<TH>>>;

    fn new(document: &Document<TH>,
            tokenizer: Tokenizer<TH>,
            last_chunk_state: LastChunkState,
            kind: ParserKind) -> DomRoot<Self> where Self: Sized;

    fn push_bytes_input_chunk(&self, chunk: Vec<u8>);

    fn do_parse_sync(&self);

    fn parse_string_chunk(&self, input: String);

    fn finish(&self);

    fn parser_is_not_active(&self) -> bool;

    fn script_nesting_level(&self) -> usize;

    fn is_script_created(&self) -> bool;

    /// Corresponds to the latter part of the "Otherwise" branch of the 'An end
    /// tag whose tag name is "script"' of
    /// <https://html.spec.whatwg.org/multipage/#parsing-main-incdata>
    ///
    /// This first moves everything from the script input to the beginning of
    /// the network input, effectively resetting the insertion point to just
    /// before the next character to be consumed.
    ///
    ///
    /// ```text
    ///     | ... script input ... network input ...
    ///     ^
    ///     insertion point
    /// ```
    fn resume_with_pending_parsing_blocking_script(&self, script: &HTMLScriptElement<TH>, result: ScriptResult);

    fn can_write(&self) -> bool;

    /// Steps 6-8 of https://html.spec.whatwg.org/multipage/#document.write()
    fn write(&self, text: Vec<DOMString>);

    // Steps 4-6 of https://html.spec.whatwg.org/multipage/#dom-document-close
    fn close(&self);

    // https://html.spec.whatwg.org/multipage/#abort-a-parser
    fn abort(&self);

    // https://html.spec.whatwg.org/multipage/#active-parser
    fn is_active(&self) -> bool;

    fn parse_html_document(document: &Document<TH>, input: DOMString, url: ServoUrl) where Self: Sized;

    fn parse_html_fragment(context: &Element<TH>, input: DOMString) -> Box<Iterator<Item=DomRoot<Node<TH>>>> where Self: Sized;

    fn parse_html_script_input(document: &Document<TH>, url: ServoUrl, type_: &str) where Self: Sized;

    fn parse_xml_document(document: &Document<TH>, input: DOMString, url: ServoUrl) where Self: Sized;

    fn push_string_input_chunk(&self, chunk: String);

    fn parse_sync(&self);

    fn get_aborted(&self) -> Cell<bool>;

    fn get_document(&self) -> &Dom<Document<TH>>;

    fn get_tokenizer(&self) -> &DomRefCell<Tokenizer<TH>>;

    fn get_last_chunk_received(&self) -> Cell<bool>;

    fn get_suspended(&self) -> Cell<bool>;

    fn parse_bytes_chunk(&self, input: Vec<u8>);
}

#[derive(PartialEq)]
pub enum LastChunkState {
    Received,
    NotReceived,
}

pub struct ElementAttribute {
    name: QualName,
    value: DOMString
}

#[derive(Clone, Copy, JSTraceable, MallocSizeOf, PartialEq)]
pub enum ParsingAlgorithm {
    Normal,
    Fragment,
}

impl ElementAttribute {
    pub fn new(name: QualName, value: DOMString) -> ElementAttribute {
        ElementAttribute {
            name: name,
            value: value
        }
    }
}

pub struct FragmentParsingResult<I, TH: TypeHolderTrait>
    where I: Iterator<Item=DomRoot<Node<TH>>>
{
    pub inner: I,
}

impl<I, TH: TypeHolderTrait> Iterator for FragmentParsingResult<I, TH>
    where I: Iterator<Item=DomRoot<Node<TH>>>
{
    type Item = DomRoot<Node<TH>>;

    fn next(&mut self) -> Option<DomRoot<Node<TH>>> {
        let next = self.inner.next()?;
        next.remove_self();
        Some(next)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

#[derive(JSTraceable, MallocSizeOf, PartialEq)]
pub enum ParserKind {
    Normal,
    ScriptCreated,
}

pub trait TokenizerTrait<TH: TypeHolderTrait>: MallocSizeOf + JSTraceable + 'static {
    fn new(document: &Document<TH>,
            url: ServoUrl,
            fragment_context: Option<FragmentContext<TH>>,
            parsing_algorithm: ParsingAlgorithm) -> Self;

    fn feed(&mut self, input: &mut BufferQueue) -> Result<(), DomRoot<HTMLScriptElement<TH>>>;

    fn end(&mut self);

    fn url(&self) -> &ServoUrl;

    fn set_plaintext_state(&mut self);
}

#[derive(JSTraceable, MallocSizeOf)]
#[must_root]
pub enum Tokenizer<TH: TypeHolderTrait> {
    Html(TH::HtmlTokenizer),
    AsyncHtml(TH::AsyncHtmlTokenizer),
    Xml(TH::XmlTokenizer),
}

enum SerializationCommand<TH: TypeHolderTrait> {
    OpenElement(DomRoot<Element<TH>>),
    CloseElement(DomRoot<Element<TH>>),
    SerializeNonelement(DomRoot<Node<TH>>),
}

struct SerializationIterator<TH: TypeHolderTrait> {
    stack: Vec<SerializationCommand<TH>>,
}

fn rev_children_iter<TH: TypeHolderTrait>(n: &Node<TH>) -> impl Iterator<Item=DomRoot<Node<TH>>>{
    match n.downcast::<HTMLTemplateElement<TH>>() {
        Some(t) => t.Content().upcast::<Node<TH>>().rev_children(),
        None => n.rev_children(),
    }
}

impl<TH: TypeHolderTrait> SerializationIterator<TH> {
    fn new(node: &Node<TH>, skip_first: bool) -> SerializationIterator<TH> {
        let mut ret = SerializationIterator {
            stack: vec![],
        };
        if skip_first {
            for c in rev_children_iter(node) {
                ret.push_node(&*c);
            }
        } else {
            ret.push_node(node);
        }
        ret
    }

    fn push_node(&mut self, n: &Node<TH>) {
        match n.downcast::<Element<TH>>() {
            Some(e) => self.stack.push(SerializationCommand::OpenElement(DomRoot::from_ref(e))),
            None => self.stack.push(SerializationCommand::SerializeNonelement(DomRoot::from_ref(n))),
        }
    }
}

impl<TH: TypeHolderTrait> Iterator for SerializationIterator<TH> {
    type Item = SerializationCommand<TH>;

    fn next(&mut self) -> Option<SerializationCommand<TH>> {
        let res = self.stack.pop();

        if let Some(SerializationCommand::OpenElement(ref e)) = res {
            self.stack.push(SerializationCommand::CloseElement(e.clone()));
            for c in rev_children_iter(&*e.upcast::<Node<TH>>()) {
                self.push_node(&c);
            }
        }

        res
    }
}

fn start_element<S: Serializer, TH: TypeHolderTrait>(node: &Element<TH>, serializer: &mut S) -> io::Result<()> {
    let name = QualName::new(None, node.namespace().clone(),
                             node.local_name().clone());
    let attrs = node.attrs().iter().map(|attr| {
        let qname = QualName::new(None, attr.namespace().clone(),
                                  attr.local_name().clone());
        let value = attr.value().clone();
        (qname, value)
    }).collect::<Vec<_>>();
    let attr_refs = attrs.iter().map(|&(ref qname, ref value)| {
        let ar: AttrRef = (&qname, &**value);
        ar
    });
    serializer.start_elem(name, attr_refs)?;
    Ok(())
}

fn end_element<S: Serializer, TH: TypeHolderTrait>(node: &Element<TH>, serializer: &mut S) -> io::Result<()> {
    let name = QualName::new(None, node.namespace().clone(),
                             node.local_name().clone());
    serializer.end_elem(name)
}


impl<'a, TH: TypeHolderTrait> Serialize for &'a Node<TH> {
    fn serialize<S: Serializer>(&self, serializer: &mut S,
                                traversal_scope: TraversalScope) -> io::Result<()> {
        let node = *self;


        let iter = SerializationIterator::new(node, traversal_scope != IncludeNode);

        for cmd in iter {
            match cmd {
                SerializationCommand::OpenElement(n) => {
                    start_element(&n, serializer)?;
                }

                SerializationCommand::CloseElement(n) => {
                    end_element(&&n, serializer)?;
                }

                SerializationCommand::SerializeNonelement(n) => {
                    match n.type_id() {
                        NodeTypeId::DocumentType => {
                            let doctype = n.downcast::<DocumentType<TH>>().unwrap();
                            serializer.write_doctype(&doctype.name())?;
                        },

                        NodeTypeId::CharacterData(CharacterDataTypeId::Text) => {
                            let cdata = n.downcast::<CharacterData<TH>>().unwrap();
                            serializer.write_text(&cdata.data())?;
                        },

                        NodeTypeId::CharacterData(CharacterDataTypeId::Comment) => {
                            let cdata = n.downcast::<CharacterData<TH>>().unwrap();
                            serializer.write_comment(&cdata.data())?;
                        },

                        NodeTypeId::CharacterData(CharacterDataTypeId::ProcessingInstruction) => {
                            let pi = n.downcast::<ProcessingInstruction<TH>>().unwrap();
                            let data = pi.upcast::<CharacterData<TH>>().data();
                            serializer.write_processing_instruction(&pi.target(), &data)?;
                        },

                        NodeTypeId::DocumentFragment => {}

                        NodeTypeId::Document(_) => panic!("Can't serialize Document node itself"),
                        NodeTypeId::Element(_) => panic!("Element shouldn't appear here"),
                    }
                }
            }
        }

        Ok(())
    }
}

impl<TH: TypeHolderTrait> Tokenizer<TH> {
    pub fn feed(&mut self, input: &mut BufferQueue) -> Result<(), DomRoot<HTMLScriptElement<TH>>> {
        match *self {
            Tokenizer::Html(ref mut tokenizer) => tokenizer.feed(input),
            Tokenizer::AsyncHtml(ref mut tokenizer) => tokenizer.feed(input),
            Tokenizer::Xml(ref mut tokenizer) => tokenizer.feed(input),
        }
    }

    pub fn end(&mut self) {
        match *self {
            Tokenizer::Html(ref mut tokenizer) => tokenizer.end(),
            Tokenizer::AsyncHtml(ref mut tokenizer) => tokenizer.end(),
            Tokenizer::Xml(ref mut tokenizer) => tokenizer.end(),
        }
    }

    pub fn url(&self) -> &ServoUrl {
        match *self {
            Tokenizer::Html(ref tokenizer) => tokenizer.url(),
            Tokenizer::AsyncHtml(ref tokenizer) => tokenizer.url(),
            Tokenizer::Xml(ref tokenizer) => tokenizer.url(),
        }
    }

    pub fn set_plaintext_state(&mut self) {
        match *self {
            Tokenizer::Html(ref mut tokenizer) => tokenizer.set_plaintext_state(),
            Tokenizer::AsyncHtml(ref mut tokenizer) => tokenizer.set_plaintext_state(),
            Tokenizer::Xml(_) => unimplemented!(),
        }
    }

    pub fn profiler_category(&self) -> ProfilerCategory {
        match *self {
            Tokenizer::Html(_) => ProfilerCategory::ScriptParseHTML,
            Tokenizer::AsyncHtml(_) => ProfilerCategory::ScriptParseHTML,
            Tokenizer::Xml(_) => ProfilerCategory::ScriptParseXML,
        }
    }
}

/// The context required for asynchronously fetching a document
/// and parsing it progressively.
#[derive(JSTraceable)]
pub struct ParserContext<TH: TypeHolderTrait> {
    /// The parser that initiated the request.
    parser: Option<Trusted<TH::ServoParser>>,
    /// Is this a synthesized document
    is_synthesized_document: bool,
    /// The pipeline associated with this document.
    id: PipelineId,
    /// The URL for this document.
    url: ServoUrl,
}

impl<TH: TypeHolderTrait> ParserContext<TH> {
    pub fn new(id: PipelineId, url: ServoUrl) -> ParserContext<TH> {
        ParserContext {
            parser: None,
            is_synthesized_document: false,
            id: id,
            url: url,
        }
    }
}

impl<TH: TypeHolderTrait> FetchResponseListener for ParserContext<TH> {
    fn process_request_body(&mut self) {}

    fn process_request_eof(&mut self) {}

    fn process_response(&mut self, meta_result: Result<FetchMetadata, NetworkError>) {
        let mut ssl_error = None;
        let mut network_error = None;
        let metadata = match meta_result {
            Ok(meta) => {
                Some(match meta {
                    FetchMetadata::Unfiltered(m) => m,
                    FetchMetadata::Filtered { unsafe_, .. } => unsafe_,
                })
            },
            Err(NetworkError::SslValidation(url, reason)) => {
                ssl_error = Some(reason);
                let mut meta = Metadata::default(url);
                let mime: Option<Mime> = "text/html".parse().ok();
                meta.set_content_type(mime.as_ref());
                Some(meta)
            },
            Err(NetworkError::Internal(reason)) => {
                network_error = Some(reason);
                let mut meta = Metadata::default(self.url.clone());
                let mime: Option<Mime> = "text/html".parse().ok();
                meta.set_content_type(mime.as_ref());
                Some(meta)
            },
            Err(_) => None,
        };
        let content_type = metadata.clone().and_then(|meta| meta.content_type).map(Serde::into_inner);
        let parser: DomRoot<TH::ServoParser> = match ScriptThread::<TH>::page_headers_available(&self.id, metadata) {
            Some(parser) => parser,
            None => return,
        };
        if parser.get_aborted().get() {
            return;
        }

        self.parser = Some(Trusted::new(&*parser));

        match content_type {
            Some(ContentType(Mime(TopLevel::Image, _, _))) => {
                self.is_synthesized_document = true;
                let page = "<html><body></body></html>".into();
                parser.push_string_input_chunk(page);
                parser.parse_sync();

                let doc = parser.get_document();
                let doc_body = DomRoot::upcast::<Node<TH>>(doc.GetBody().unwrap());
                let img = HTMLImageElement::new(local_name!("img"), None, doc);
                img.SetSrc(DOMString::from(self.url.to_string()));
                doc_body.AppendChild(&DomRoot::upcast::<Node<TH>>(img)).expect("Appending failed");

            },
            Some(ContentType(Mime(TopLevel::Text, SubLevel::Plain, _))) => {
                // https://html.spec.whatwg.org/multipage/#read-text
                let page = "<pre>\n".into();
                parser.push_string_input_chunk(page);
                parser.parse_sync();
                let token = parser.get_tokenizer();
                token.borrow_mut().set_plaintext_state();
            },
            Some(ContentType(Mime(TopLevel::Text, SubLevel::Html, _))) => {
                // Handle text/html
                if let Some(reason) = ssl_error {
                    self.is_synthesized_document = true;
                    let page = resources::read_string(Resource::BadCertHTML);
                    let page = page.replace("${reason}", &reason);
                    parser.push_string_input_chunk(page);
                    parser.parse_sync();
                }
                if let Some(reason) = network_error {
                    self.is_synthesized_document = true;
                    let page = resources::read_string(Resource::NetErrorHTML);
                    let page = page.replace("${reason}", &reason);
                    parser.push_string_input_chunk(page);
                    parser.parse_sync();
                }
            },
            Some(ContentType(Mime(TopLevel::Text, SubLevel::Xml, _))) | // Handle text/xml, application/xml
            Some(ContentType(Mime(TopLevel::Application, SubLevel::Xml, _))) => {},
            Some(ContentType(Mime(TopLevel::Application, SubLevel::Ext(ref sub), _)))
                if sub.as_str() == "xhtml+xml".to_owned() => {}, // Handle xhtml (application/xhtml+xml)
            Some(ContentType(Mime(toplevel, sublevel, _))) => {
                // Show warning page for unknown mime types.
                let page = format!("<html><body><p>Unknown content type ({}/{}).</p></body></html>",
                                   toplevel.as_str(),
                                   sublevel.as_str());
                self.is_synthesized_document = true;
                parser.push_string_input_chunk(page);
                parser.parse_sync();
            },
            None => {
                // No content-type header.
                // Merge with #4212 when fixed.
            },
        }
    }

    fn process_response_chunk(&mut self, payload: Vec<u8>) {
        if self.is_synthesized_document {
            return;
        }
        let parser = match self.parser.as_ref() {
            Some(parser) => parser.root(),
            None => return,
        };
        if parser.get_aborted().get() {
            return;
        }
        parser.parse_bytes_chunk(payload);
    }

    fn process_response_eof(&mut self, status: Result<(), NetworkError>) {
        let parser = match self.parser.as_ref() {
            Some(parser) => parser.root(),
            None => return,
        };
        if parser.get_aborted().get() {
            return;
        }

        if let Err(err) = status {
            // TODO(Savago): we should send a notification to callers #5463.
            debug!("Failed to load page URL {}, error: {:?}", self.url, err);
        }

        parser.get_last_chunk_received().set(true);
        if !parser.get_suspended().get() {
            parser.parse_sync();
        }
    }
}

impl<TH: TypeHolderTrait> PreInvoke for ParserContext<TH> {}

pub struct FragmentContext<'a, TH: TypeHolderTrait> {
    pub context_elem: &'a Node<TH>,
    pub form_elem: Option<&'a Node<TH>>,
}

#[allow(unrooted_must_root)]
pub fn insert<TH: TypeHolderTrait>(parent: &Node<TH>, reference_child: Option<&Node<TH>>, child: NodeOrText<Dom<Node<TH>>>) {
    match child {
        NodeOrText::AppendNode(n) => {
            parent.InsertBefore(&n, reference_child).unwrap();
        },
        NodeOrText::AppendText(t) => {
            let text = reference_child
                .and_then(Node::<TH>::GetPreviousSibling)
                .or_else(|| parent.GetLastChild())
                .and_then(DomRoot::downcast::<Text<TH>>);

            if let Some(text) = text {
                text.upcast::<CharacterData<TH>>().append_data(&t);
            } else {
                let text = Text::new(String::from(t).into(), &parent.owner_doc());
                parent.InsertBefore(text.upcast(), reference_child).unwrap();
            }
        },
    }
}

#[derive(JSTraceable, MallocSizeOf)]
#[must_root]
pub struct Sink<TH: TypeHolderTrait> {
    pub base_url: ServoUrl,
    pub document: Dom<Document<TH>>,
    pub current_line: u64,
    pub script: MutNullableDom<HTMLScriptElement<TH>>,
    pub parsing_algorithm: ParsingAlgorithm,
}

impl<TH: TypeHolderTrait> Sink<TH> {
    fn same_tree(&self, x: &Dom<Node<TH>>, y: &Dom<Node<TH>>) -> bool {
        let x = x.downcast::<Element<TH>>().expect("Element node expected");
        let y = y.downcast::<Element<TH>>().expect("Element node expected");

        x.is_in_same_home_subtree(y)
    }

    fn has_parent_node(&self, node: &Dom<Node<TH>>) -> bool {
         node.GetParentNode().is_some()
    }
}

#[allow(unrooted_must_root)]  // FIXME: really?
impl<TH: TypeHolderTrait> TreeSink for Sink<TH> {
    type Output = Self;
    fn finish(self) -> Self { self }

    type Handle = Dom<Node<TH>>;

    fn get_document(&mut self) -> Dom<Node<TH>> {
        Dom::from_ref(self.document.upcast())
    }

    fn get_template_contents(&mut self, target: &Dom<Node<TH>>) -> Dom<Node<TH>> {
        let template = target.downcast::<HTMLTemplateElement<TH>>()
            .expect("tried to get template contents of non-HTMLTemplateElement in HTML parsing");
        Dom::from_ref(template.Content().upcast())
    }

    fn same_node(&self, x: &Dom<Node<TH>>, y: &Dom<Node<TH>>) -> bool {
        x == y
    }

    fn elem_name<'a>(&self, target: &'a Dom<Node<TH>>) -> ExpandedName<'a> {
        let elem = target.downcast::<Element<TH>>()
            .expect("tried to get name of non-Element in HTML parsing");
        ExpandedName {
            ns: elem.namespace(),
            local: elem.local_name(),
        }
    }

    fn create_element(&mut self, name: QualName, attrs: Vec<Attribute>, _flags: ElementFlags)
            -> Dom<Node<TH>> {
        let attrs = attrs
            .into_iter()
            .map(|attr| ElementAttribute::new(attr.name, DOMString::from(String::from(attr.value))))
            .collect();
        let element = create_element_for_token(
            name,
            attrs,
            &*self.document,
            ElementCreator::ParserCreated(self.current_line),
            self.parsing_algorithm,
        );
        Dom::from_ref(element.upcast())
    }

    fn create_comment(&mut self, text: StrTendril) -> Dom<Node<TH>> {
        let comment = Comment::new(DOMString::from(String::from(text)), &*self.document);
        Dom::from_ref(comment.upcast())
    }

    fn create_pi(&mut self, target: StrTendril, data: StrTendril) -> Dom<Node<TH>> {
        let doc = &*self.document;
        let pi = ProcessingInstruction::new(
            DOMString::from(String::from(target)), DOMString::from(String::from(data)),
            doc);
        Dom::from_ref(pi.upcast())
    }

    fn associate_with_form(&mut self, target: &Dom<Node<TH>>, form: &Dom<Node<TH>>, nodes: (&Dom<Node<TH>>, Option<&Dom<Node<TH>>>)) {
        let (element, prev_element) = nodes;
        let tree_node = prev_element.map_or(element, |prev| {
            if self.has_parent_node(element) { element } else { prev }
        });
        if !self.same_tree(tree_node, form) {
            return;
        }

        let node = target;
        let form = DomRoot::downcast::<HTMLFormElement<TH>>(DomRoot::from_ref(&**form))
            .expect("Owner must be a form element");

        let elem = node.downcast::<Element<TH>>();
        let control = elem.and_then(|e| e.as_maybe_form_control());

        if let Some(control) = control {
            control.set_form_owner_from_parser(&form);
        } else {
            // TODO remove this code when keygen is implemented.
            assert_eq!(node.NodeName(), "KEYGEN", "Unknown form-associatable element");
        }
    }

    fn append_before_sibling(&mut self,
            sibling: &Dom<Node<TH>>,
            new_node: NodeOrText<Dom<Node<TH>>>) {
        let parent = sibling.GetParentNode()
            .expect("append_before_sibling called on node without parent");

        insert(&parent, Some(&*sibling), new_node);
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

    fn append(&mut self, parent: &Dom<Node<TH>>, child: NodeOrText<Dom<Node<TH>>>) {
        insert(&parent, None, child);
    }

    fn append_based_on_parent_node(
        &mut self,
        elem: &Dom<Node<TH>>,
        prev_elem: &Dom<Node<TH>>,
        child: NodeOrText<Dom<Node<TH>>>,
    ) {
        if self.has_parent_node(elem) {
            self.append_before_sibling(elem, child);
        } else {
            self.append(prev_elem, child);
        }
    }

    fn append_doctype_to_document(&mut self, name: StrTendril, public_id: StrTendril,
                                  system_id: StrTendril) {
        let doc = &*self.document;
        let doctype = DocumentType::new(
            DOMString::from(String::from(name)), Some(DOMString::from(String::from(public_id))),
            Some(DOMString::from(String::from(system_id))), doc);
        doc.upcast::<Node<TH>>().AppendChild(doctype.upcast()).expect("Appending failed");
    }

    fn add_attrs_if_missing(&mut self, target: &Dom<Node<TH>>, attrs: Vec<Attribute>) {
        let elem = target.downcast::<Element<TH>>()
            .expect("tried to set attrs on non-Element in HTML parsing");
        for attr in attrs {
            elem.set_attribute_from_parser(attr.name, DOMString::from(String::from(attr.value)), None);
        }
    }

    fn remove_from_parent(&mut self, target: &Dom<Node<TH>>) {
        if let Some(ref parent) = target.GetParentNode() {
            parent.RemoveChild(&*target).unwrap();
        }
    }

    fn mark_script_already_started(&mut self, node: &Dom<Node<TH>>) {
        let script = node.downcast::<HTMLScriptElement<TH>>();
        script.map(|script| script.set_already_started(true));
    }

    fn complete_script(&mut self, node: &Dom<Node<TH>>) -> NextParserState {
        if let Some(script) = node.downcast() {
            self.script.set(Some(script));
            NextParserState::Suspend
        } else {
            NextParserState::Continue
        }
    }

    fn reparent_children(&mut self, node: &Dom<Node<TH>>, new_parent: &Dom<Node<TH>>) {
        while let Some(ref child) = node.GetFirstChild() {
            new_parent.AppendChild(&child).unwrap();
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#html-integration-point>
    /// Specifically, the <annotation-xml> cases.
    fn is_mathml_annotation_xml_integration_point(&self, handle: &Dom<Node<TH>>) -> bool {
        let elem = handle.downcast::<Element<TH>>().unwrap();
        elem.get_attribute(&ns!(), &local_name!("encoding")).map_or(false, |attr| {
            attr.value().eq_ignore_ascii_case("text/html")
                || attr.value().eq_ignore_ascii_case("application/xhtml+xml")
        })
    }

    fn set_current_line(&mut self, line_number: u64) {
        self.current_line = line_number;
    }

    fn pop(&mut self, node: &Dom<Node<TH>>) {
        let node = DomRoot::from_ref(&**node);
        vtable_for(&node).pop();
    }
}

/// https://html.spec.whatwg.org/multipage/#create-an-element-for-the-token
pub fn create_element_for_token<TH: TypeHolderTrait>(
    name: QualName,
    attrs: Vec<ElementAttribute>,
    document: &Document<TH>,
    creator: ElementCreator,
    parsing_algorithm: ParsingAlgorithm,
) -> DomRoot<Element<TH>> {
    // Step 3.
    let is = attrs.iter()
        .find(|attr| attr.name.local.eq_str_ignore_ascii_case("is"))
        .map(|attr| LocalName::from(&*attr.value));

    // Step 4.
    let definition = document.lookup_custom_element_definition(&name.ns, &name.local, is.as_ref());

    // Step 5.
    let will_execute_script = definition.is_some() && parsing_algorithm != ParsingAlgorithm::Fragment;

    // Step 6.
    if will_execute_script {
        // Step 6.1.
        document.increment_throw_on_dynamic_markup_insertion_counter();
        // Step 6.2
        if is_execution_stack_empty() {
            document.window().upcast::<GlobalScope<TH>>().perform_a_microtask_checkpoint();
        }
        // Step 6.3
        ScriptThread::<TH>::push_new_element_queue()
    }

    // Step 7.
    let creation_mode = if will_execute_script {
        CustomElementCreationMode::Synchronous
    } else {
        CustomElementCreationMode::Asynchronous
    };
    let element = Element::create(name, is, document, creator, creation_mode);

    // Step 8.
    for attr in attrs {
        element.set_attribute_from_parser(attr.name, attr.value, None);
    }

    // Step 9.
    if will_execute_script {
        // Steps 9.1 - 9.2.
        ScriptThread::<TH>::pop_current_element_queue();
        // Step 9.3.
        document.decrement_throw_on_dynamic_markup_insertion_counter();
    }

    // TODO: Step 10.
    // TODO: Step 11.

    // Step 12 is handled in `associate_with_form`.

    // Step 13.
    element
}

#[allow(unsafe_code)]
unsafe impl<TH: TypeHolderTrait> JSTraceable for HtmlTokenizer<TreeBuilder<Dom<Node<TH>>, Sink<TH>>> {
    unsafe fn trace(&self, trc: *mut JSTracer) {
        struct Tracer<THH: TypeHolderTrait + 'static>(*mut JSTracer, PhantomData<THH>);
        let tracer = Tracer(trc, Default::default());

        impl<THH: TypeHolderTrait> HtmlTracer for Tracer<THH> {
            type Handle = Dom<Node<THH>>;
            #[allow(unrooted_must_root)]
            fn trace_handle(&self, node: &Dom<Node<THH>>) {
                unsafe { node.trace(self.0); }
            }
        }

        let tree_builder = &self.sink;
        tree_builder.trace_handles(&tracer);
        tree_builder.sink.trace(trc);
    }
}



#[allow(unsafe_code)]
unsafe impl<TH: TypeHolderTrait> JSTraceable for XmlTokenizer<XmlTreeBuilder<Dom<Node<TH>>, Sink<TH>>> {
    unsafe fn trace(&self, trc: *mut JSTracer) {
        struct Tracer<THH: TypeHolderTrait + 'static>(*mut JSTracer, PhantomData<THH>);
        let tracer = Tracer(trc, Default::default());

        impl<THH: TypeHolderTrait> XmlTracer for Tracer<THH> {
            type Handle = Dom<Node<THH>>;
            #[allow(unrooted_must_root)]
            fn trace_handle(&self, node: &Dom<Node<THH>>) {
                unsafe { node.trace(self.0); }
            }
        }

        let tree_builder = &self.sink;
        tree_builder.trace_handles(&tracer);
        tree_builder.sink.trace(trc);
    }
}
