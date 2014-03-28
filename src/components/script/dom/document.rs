/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::InheritTypes::{DocumentDerived, EventCast, HTMLElementCast};
use dom::bindings::codegen::InheritTypes::{DocumentBase, NodeCast, DocumentCast};
use dom::bindings::codegen::InheritTypes::{HTMLHeadElementCast, TextCast, ElementCast};
use dom::bindings::codegen::InheritTypes::{DocumentTypeCast, HTMLHtmlElementCast};
use dom::bindings::codegen::BindingDeclarations::DocumentBinding;
use dom::bindings::js::{JS, JSRef, RootCollection};
use dom::bindings::trace::Untraceable;
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::bindings::error::{ErrorResult, Fallible, NotSupported, InvalidCharacter, HierarchyRequest, NamespaceError};
use dom::bindings::utils::{xml_name_type, InvalidXMLName, Name, QName};
use dom::comment::Comment;
use dom::documentfragment::DocumentFragment;
use dom::documenttype::DocumentType;
use dom::domimplementation::DOMImplementation;
use dom::element::{Element, AttributeHandlers, get_attribute_parts};
use dom::element::{HTMLHtmlElementTypeId, HTMLHeadElementTypeId, HTMLTitleElementTypeId};
use dom::element::{HTMLBodyElementTypeId, HTMLFrameSetElementTypeId};
use dom::event::Event;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlcollection::{HTMLCollection, CollectionFilter};
use dom::nodelist::NodeList;
use dom::htmlelement::HTMLElement;
use dom::htmlheadelement::HTMLHeadElement;
use dom::htmlhtmlelement::HTMLHtmlElement;
use dom::htmltitleelement::HTMLTitleElement;
use dom::mouseevent::MouseEvent;
use dom::node::{Node, ElementNodeTypeId, DocumentNodeTypeId, NodeHelpers, INode};
use dom::node::{CloneChildren, DoNotCloneChildren};
use dom::text::Text;
use dom::processinginstruction::ProcessingInstruction;
use dom::uievent::UIEvent;
use dom::window::Window;
use dom::location::Location;
use html::hubbub_html_parser::build_element_from_tag;
use hubbub::hubbub::{QuirksMode, NoQuirks, LimitedQuirks, FullQuirks};
use layout_interface::{DocumentDamageLevel, ContentChangedDocumentDamage};
use servo_util::namespace;
use servo_util::namespace::{Namespace, Null};
use servo_util::str::{DOMString, null_str_as_empty_ref};

use collections::hashmap::HashMap;
use js::jsapi::JSContext;
use std::ascii::StrAsciiExt;
use url::{Url, from_str};

#[deriving(Eq,Encodable)]
pub enum IsHTMLDocument {
    HTMLDocument,
    NonHTMLDocument,
}

#[deriving(Encodable)]
pub struct Document {
    pub node: Node,
    pub reflector_: Reflector,
    pub window: JS<Window>,
    pub idmap: HashMap<DOMString, Vec<JS<Element>>>,
    pub implementation: Option<JS<DOMImplementation>>,
    pub content_type: DOMString,
    pub encoding_name: DOMString,
    pub is_html_document: bool,
    pub url: Untraceable<Url>,
    pub quirks_mode: Untraceable<QuirksMode>,
}

impl DocumentDerived for EventTarget {
    fn is_document(&self) -> bool {
        match self.type_id {
            NodeTargetTypeId(DocumentNodeTypeId) => true,
            _ => false
        }
    }
}

impl Document {
    pub fn reflect_document<D: Reflectable+DocumentBase>
            (document:  ~D,
             window:    &JSRef<Window>,
             wrap_fn:   extern "Rust" fn(*JSContext, &JSRef<Window>, ~D) -> JS<D>)
             -> JS<D> {
        let roots = RootCollection::new();
        assert!(document.reflector().get_jsobject().is_null());
        let raw_doc = reflect_dom_object(document, window, wrap_fn);
        assert!(raw_doc.reflector().get_jsobject().is_not_null());

        let document = DocumentCast::from(&raw_doc);
        let document_root = document.root(&roots);
        let mut node: JS<Node> = NodeCast::from(&document);
        node.get_mut().set_owner_doc(&document_root.root_ref());
        raw_doc
    }

    pub fn new_inherited(window: JS<Window>,
                         url: Option<Url>,
                         is_html_document: IsHTMLDocument,
                         content_type: Option<DOMString>) -> Document {
        let url = url.unwrap_or_else(|| from_str("about:blank").unwrap());

        Document {
            node: Node::new_without_doc(DocumentNodeTypeId),
            reflector_: Reflector::new(),
            window: window,
            idmap: HashMap::new(),
            implementation: None,
            content_type: match content_type {
                Some(string) => string.clone(),
                None => match is_html_document {
                    // http://dom.spec.whatwg.org/#dom-domimplementation-createhtmldocument
                    HTMLDocument => ~"text/html",
                    // http://dom.spec.whatwg.org/#concept-document-content-type
                    NonHTMLDocument => ~"application/xml"
                }
            },
            url: Untraceable::new(url),
            // http://dom.spec.whatwg.org/#concept-document-quirks
            quirks_mode: Untraceable::new(NoQuirks),
            // http://dom.spec.whatwg.org/#concept-document-encoding
            encoding_name: ~"utf-8",
            is_html_document: is_html_document == HTMLDocument,
        }
    }

    pub fn new(window: &JSRef<Window>, url: Option<Url>, doctype: IsHTMLDocument, content_type: Option<DOMString>) -> JS<Document> {
        let document = Document::new_inherited(window.unrooted(), url, doctype, content_type);
        Document::reflect_document(~document, window, DocumentBinding::Wrap)
    }
}

impl Document {
    pub fn url<'a>(&'a self) -> &'a Url {
        &*self.url
    }
}

impl Document {
    // http://dom.spec.whatwg.org/#dom-document
    pub fn Constructor(owner: &JSRef<Window>) -> Fallible<JS<Document>> {
        Ok(Document::new(owner, None, NonHTMLDocument, None))
    }
}

impl Reflectable for Document {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.node.reflector()
    }

    fn mut_reflector<'a>(&'a mut self) -> &'a mut Reflector {
        self.node.mut_reflector()
    }
}

impl Document {
    // http://dom.spec.whatwg.org/#dom-document-implementation
    pub fn Implementation(&mut self) -> JS<DOMImplementation> {
        if self.implementation.is_none() {
            let roots = RootCollection::new();
            let window = self.window.root(&roots);
            self.implementation = Some(DOMImplementation::new(&window.root_ref()));
        }
        self.implementation.get_ref().clone()
    }

    // http://dom.spec.whatwg.org/#dom-document-url
    pub fn URL(&self) -> DOMString {
        self.url().to_str()
    }

    // http://dom.spec.whatwg.org/#dom-document-documenturi
    pub fn DocumentURI(&self) -> DOMString {
        self.URL()
    }

    // http://dom.spec.whatwg.org/#dom-document-compatmode
    pub fn CompatMode(&self) -> DOMString {
        match *self.quirks_mode {
            NoQuirks => ~"CSS1Compat",
            LimitedQuirks | FullQuirks => ~"BackCompat"
        }
    }

    pub fn quirks_mode(&self) -> QuirksMode {
        *self.quirks_mode
    }

    pub fn set_quirks_mode(&mut self, mode: QuirksMode) {
        *self.quirks_mode = mode;
    }

    // http://dom.spec.whatwg.org/#dom-document-characterset
    pub fn CharacterSet(&self) -> DOMString {
        self.encoding_name.to_ascii_lower()
    }

    pub fn set_encoding_name(&mut self, name: DOMString) {
        self.encoding_name = name;
    }

    // http://dom.spec.whatwg.org/#dom-document-content_type
    pub fn ContentType(&self) -> DOMString {
        self.content_type.clone()
    }

    // http://dom.spec.whatwg.org/#dom-document-doctype
    pub fn GetDoctype(&self) -> Option<JS<DocumentType>> {
        self.node.children().find(|child| child.is_doctype())
                            .map(|node| DocumentTypeCast::to(&node).unwrap())
    }

    // http://dom.spec.whatwg.org/#dom-document-documentelement
    pub fn GetDocumentElement(&self) -> Option<JS<Element>> {
        self.node.child_elements().next()
    }

    // http://dom.spec.whatwg.org/#dom-document-getelementsbytagname
    pub fn GetElementsByTagName(&self, abstract_self: &JSRef<Document>, tag_name: DOMString) -> JS<HTMLCollection> {
        let roots = RootCollection::new();
        let window = self.window.root(&roots);
        HTMLCollection::by_tag_name(&window.root_ref(), NodeCast::from_ref(abstract_self), tag_name)
    }

    // http://dom.spec.whatwg.org/#dom-document-getelementsbytagnamens
    pub fn GetElementsByTagNameNS(&self, abstract_self: &JSRef<Document>, maybe_ns: Option<DOMString>, tag_name: DOMString) -> JS<HTMLCollection> {
        let roots = RootCollection::new();
        let window = self.window.root(&roots);

        let namespace = match maybe_ns {
            Some(namespace) => Namespace::from_str(namespace),
            None => Null
        };
        HTMLCollection::by_tag_name_ns(&window.root_ref(), NodeCast::from_ref(abstract_self), tag_name, namespace)
    }

    // http://dom.spec.whatwg.org/#dom-document-getelementsbyclassname
    pub fn GetElementsByClassName(&self, abstract_self: &JSRef<Document>, classes: DOMString) -> JS<HTMLCollection> {
        let roots = RootCollection::new();
        let window = self.window.root(&roots);

        HTMLCollection::by_class_name(&window.root_ref(), NodeCast::from_ref(abstract_self), classes)
    }

    // http://dom.spec.whatwg.org/#dom-nonelementparentnode-getelementbyid
    pub fn GetElementById(&self, id: DOMString) -> Option<JS<Element>> {
        match self.idmap.find_equiv(&id) {
            None => None,
            Some(ref elements) => Some(elements.get(0).clone()),
        }
    }

    // http://dom.spec.whatwg.org/#dom-document-createelement
    pub fn CreateElement(&self, abstract_self: &JSRef<Document>, local_name: DOMString)
                         -> Fallible<JS<Element>> {
        if xml_name_type(local_name) == InvalidXMLName {
            debug!("Not a valid element name");
            return Err(InvalidCharacter);
        }
        let local_name = local_name.to_ascii_lower();
        Ok(build_element_from_tag(local_name, abstract_self))
    }

    // http://dom.spec.whatwg.org/#dom-document-createelementns
    pub fn CreateElementNS(&self, abstract_self: &JSRef<Document>,
                           namespace: Option<DOMString>,
                           qualified_name: DOMString) -> Fallible<JS<Element>> {
        let ns = Namespace::from_str(null_str_as_empty_ref(&namespace));
        match xml_name_type(qualified_name) {
            InvalidXMLName => {
                debug!("Not a valid element name");
                return Err(InvalidCharacter);
            },
            Name => {
                debug!("Not a valid qualified element name");
                return Err(NamespaceError);
            },
            QName => {}
        }

        let (prefix_from_qname, local_name_from_qname) = get_attribute_parts(qualified_name);
        match (&ns, prefix_from_qname.clone(), local_name_from_qname.as_slice()) {
            // throw if prefix is not null and namespace is null
            (&namespace::Null, Some(_), _) => {
                debug!("Namespace can't be null with a non-null prefix");
                return Err(NamespaceError);
            },
            // throw if prefix is "xml" and namespace is not the XML namespace
            (_, Some(ref prefix), _) if "xml" == *prefix && ns != namespace::XML => {
                debug!("Namespace must be the xml namespace if the prefix is 'xml'");
                return Err(NamespaceError);
            },
            // throw if namespace is the XMLNS namespace and neither qualifiedName nor prefix is "xmlns"
            (&namespace::XMLNS, Some(ref prefix), _) if "xmlns" == *prefix => {},
            (&namespace::XMLNS, _, "xmlns") => {},
            (&namespace::XMLNS, _, _) => {
                debug!("The prefix or the qualified name must be 'xmlns' if namespace is the XMLNS namespace ");
                return Err(NamespaceError);
            },
            _ => {}
        }

        if ns == namespace::HTML {
            Ok(build_element_from_tag(local_name_from_qname, abstract_self))
        } else {
            Ok(Element::new(local_name_from_qname, ns, prefix_from_qname, abstract_self))
        }
    }

    // http://dom.spec.whatwg.org/#dom-document-createdocumentfragment
    pub fn CreateDocumentFragment(&self, abstract_self: &JSRef<Document>) -> JS<DocumentFragment> {
        DocumentFragment::new(abstract_self)
    }

    // http://dom.spec.whatwg.org/#dom-document-createtextnode
    pub fn CreateTextNode(&self, abstract_self: &JSRef<Document>, data: DOMString)
                          -> JS<Text> {
        Text::new(data, abstract_self)
    }

    // http://dom.spec.whatwg.org/#dom-document-createcomment
    pub fn CreateComment(&self, abstract_self: &JSRef<Document>, data: DOMString) -> JS<Comment> {
        Comment::new(data, abstract_self)
    }

    // http://dom.spec.whatwg.org/#dom-document-createprocessinginstruction
    pub fn CreateProcessingInstruction(&self, abstract_self: &JSRef<Document>, target: DOMString,
                                       data: DOMString) -> Fallible<JS<ProcessingInstruction>> {
        // Step 1.
        if xml_name_type(target) == InvalidXMLName {
            return Err(InvalidCharacter);
        }

        // Step 2.
        if data.contains("?>") {
            return Err(InvalidCharacter);
        }

        // Step 3.
        Ok(ProcessingInstruction::new(target, data, abstract_self))
    }

    // http://dom.spec.whatwg.org/#dom-document-importnode
    pub fn ImportNode(&self, abstract_self: &JSRef<Document>, node: &JSRef<Node>, deep: bool) -> Fallible<JS<Node>> {
        // Step 1.
        if node.unrooted().is_document() {
            return Err(NotSupported);
        }

        // Step 2.
        let clone_children = match deep {
            true => CloneChildren,
            false => DoNotCloneChildren
        };

        Ok(Node::clone(node, Some(abstract_self), clone_children))
    }

    // http://dom.spec.whatwg.org/#dom-document-adoptnode
    pub fn AdoptNode(&self, abstract_self: &JSRef<Document>, node: &JSRef<Node>) -> Fallible<JS<Node>> {
        // Step 1.
        if node.unrooted().is_document() {
            return Err(NotSupported);
        }

        // Step 2.
        let mut adoptee = node.clone();
        Node::adopt(&mut adoptee, abstract_self);

        // Step 3.
        Ok(adoptee.unrooted())
    }

    // http://dom.spec.whatwg.org/#dom-document-createevent
    pub fn CreateEvent(&self, interface: DOMString) -> Fallible<JS<Event>> {
        let roots = RootCollection::new();
        let window = self.window.root(&roots);

        match interface.to_ascii_lower().as_slice() {
            // FIXME: Implement CustomEvent (http://dom.spec.whatwg.org/#customevent)
            "uievents" | "uievent" => Ok(EventCast::from(&UIEvent::new(&window.root_ref()))),
            "mouseevents" | "mouseevent" => Ok(EventCast::from(&MouseEvent::new(&window.root_ref()))),
            "htmlevents" | "events" | "event" => Ok(Event::new(&window.root_ref())),
            _ => Err(NotSupported)
        }
    }

    // http://www.whatwg.org/specs/web-apps/current-work/#document.title
    pub fn Title(&self, _: &JSRef<Document>) -> DOMString {
        let mut title = ~"";
        self.GetDocumentElement().map(|root| {
            let root: JS<Node> = NodeCast::from(&root);
            root.traverse_preorder()
                .find(|node| node.type_id() == ElementNodeTypeId(HTMLTitleElementTypeId))
                .map(|title_elem| {
                    for child in title_elem.children() {
                        if child.is_text() {
                            let text: JS<Text> = TextCast::to(&child).unwrap();
                            title.push_str(text.get().characterdata.data.as_slice());
                        }
                    }
                });
        });
        let v: Vec<&str> = title.words().collect();
        let title = v.connect(" ");
        title.trim().to_owned()
    }

    // http://www.whatwg.org/specs/web-apps/current-work/#document.title
    pub fn SetTitle(&self, abstract_self: &JSRef<Document>, title: DOMString) -> ErrorResult {
        let roots = RootCollection::new();

        self.GetDocumentElement().map(|root| {
            let root: JS<Node> = NodeCast::from(&root);
            let mut head_node = root.traverse_preorder().find(|child| {
                child.get().type_id == ElementNodeTypeId(HTMLHeadElementTypeId)
            });
            head_node.as_mut().map(|head| {
                let mut title_node = head.children().find(|child| {
                    child.get().type_id == ElementNodeTypeId(HTMLTitleElementTypeId)
                });

                match title_node {
                    Some(ref mut title_node) => {
                        for title_child in title_node.children() {
                            let title_child = title_child.root(&roots);
                            assert!(title_node.RemoveChild(&mut title_child.root_ref()).is_ok());
                        }
                        let new_text = self.CreateTextNode(abstract_self, title.clone());
                        let new_text = new_text.root(&roots);

                        assert!(title_node.AppendChild(NodeCast::from_mut_ref(&mut new_text.root_ref())).is_ok());
                    },
                    None => {
                        let mut new_title: JS<Node> =
                            NodeCast::from(&HTMLTitleElement::new(~"title", abstract_self));
                        let new_title_root = new_title.root(&roots);

                        let new_text = self.CreateTextNode(abstract_self, title.clone());
                        let new_text = new_text.root(&roots);

                        assert!(new_title.AppendChild(NodeCast::from_mut_ref(&mut new_text.root_ref())).is_ok());
                        assert!(head.AppendChild(&mut new_title_root.root_ref()).is_ok());
                    },
                }
            });
        });
        Ok(())
    }

    fn get_html_element(&self) -> Option<JS<HTMLHtmlElement>> {
        self.GetDocumentElement().filtered(|root| {
            root.get().node.type_id == ElementNodeTypeId(HTMLHtmlElementTypeId)
        }).map(|elem| HTMLHtmlElementCast::to(&elem).unwrap())
    }

    // http://www.whatwg.org/specs/web-apps/current-work/#dom-document-head
    pub fn GetHead(&self) -> Option<JS<HTMLHeadElement>> {
        self.get_html_element().and_then(|root| {
            let node: JS<Node> = NodeCast::from(&root);
            node.children().find(|child| {
                child.type_id() == ElementNodeTypeId(HTMLHeadElementTypeId)
            }).map(|node| HTMLHeadElementCast::to(&node).unwrap())
        })
    }

    // http://www.whatwg.org/specs/web-apps/current-work/#dom-document-body
    pub fn GetBody(&self, _: &JSRef<Document>) -> Option<JS<HTMLElement>> {
        self.get_html_element().and_then(|root| {
            let node: JS<Node> = NodeCast::from(&root);
            node.children().find(|child| {
                match child.type_id() {
                    ElementNodeTypeId(HTMLBodyElementTypeId) |
                    ElementNodeTypeId(HTMLFrameSetElementTypeId) => true,
                    _ => false
                }
            }).map(|node| HTMLElementCast::to(&node).unwrap())
        })
    }

    // http://www.whatwg.org/specs/web-apps/current-work/#dom-document-body
    pub fn SetBody(&self, abstract_self: &JSRef<Document>, new_body: Option<JSRef<HTMLElement>>) -> ErrorResult {
        let roots = RootCollection::new();

        // Step 1.
        match new_body {
            Some(ref node) => {
                match node.get().element.node.type_id {
                    ElementNodeTypeId(HTMLBodyElementTypeId) | ElementNodeTypeId(HTMLFrameSetElementTypeId) => {}
                    _ => return Err(HierarchyRequest)
                }
            }
            None => return Err(HierarchyRequest)
        }

        // Step 2.
        let old_body: Option<JS<HTMLElement>> = self.GetBody(abstract_self);
        if old_body == new_body.as_ref().map(|new_body| new_body.unrooted()) {
            return Ok(());
        }

        // Step 3.
        match self.get_html_element() {
            // Step 4.
            None => return Err(HierarchyRequest),
            Some(root) => {
                let mut new_body_unwrapped = new_body.unwrap();
                let new_body: &mut JSRef<Node> = NodeCast::from_mut_ref(&mut new_body_unwrapped);

                let mut root: JS<Node> = NodeCast::from(&root);
                match old_body {
                    Some(child) => {
                        let child: JS<Node> = NodeCast::from(&child);
                        let child = child.root(&roots);

                        assert!(root.ReplaceChild(new_body, &mut child.root_ref()).is_ok())
                    }
                    None => assert!(root.AppendChild(new_body).is_ok())
                };
            }
        }
        Ok(())
    }

    // http://www.whatwg.org/specs/web-apps/current-work/#dom-document-getelementsbyname
    pub fn GetElementsByName(&self, name: DOMString) -> JS<NodeList> {
        self.createNodeList(|node| {
            if !node.get().is_element() {
                return false;
            }

            let element: JS<Element> = ElementCast::to(&node.unrooted()).unwrap();
            element.get_attribute(Null, "name").map_or(false, |attr| {
                attr.get().value_ref() == name
            })
        })
    }

    pub fn Images(&self, abstract_self: &JSRef<Document>) -> JS<HTMLCollection> {
        let roots = RootCollection::new();
        let window = self.window.root(&roots);

        // FIXME: https://github.com/mozilla/servo/issues/1847
        struct ImagesFilter;
        impl CollectionFilter for ImagesFilter {
            fn filter(&self, elem: &JSRef<Element>, _root: &JSRef<Node>) -> bool {
                elem.get().local_name == ~"img"
            }
        }
        let filter = ~ImagesFilter;
        HTMLCollection::create(&window.root_ref(), NodeCast::from_ref(abstract_self), filter)
    }

    pub fn Embeds(&self, abstract_self: &JSRef<Document>) -> JS<HTMLCollection> {
        let roots = RootCollection::new();
        let window = self.window.root(&roots);

        // FIXME: https://github.com/mozilla/servo/issues/1847
        struct EmbedsFilter;
        impl CollectionFilter for EmbedsFilter {
            fn filter(&self, elem: &JSRef<Element>, _root: &JSRef<Node>) -> bool {
                elem.get().local_name == ~"embed"
            }
        }
        let filter = ~EmbedsFilter;
        HTMLCollection::create(&window.root_ref(), NodeCast::from_ref(abstract_self), filter)
    }

    pub fn Plugins(&self, abstract_self: &JSRef<Document>) -> JS<HTMLCollection> {
        // FIXME: https://github.com/mozilla/servo/issues/1847
        self.Embeds(abstract_self)
    }

    pub fn Links(&self, abstract_self: &JSRef<Document>) -> JS<HTMLCollection> {
        let roots = RootCollection::new();
        let window = self.window.root(&roots);

        // FIXME: https://github.com/mozilla/servo/issues/1847
        struct LinksFilter;
        impl CollectionFilter for LinksFilter {
            fn filter(&self, elem: &JSRef<Element>, _root: &JSRef<Node>) -> bool {
                (elem.get().local_name == ~"a" || elem.get().local_name == ~"area") &&
                elem.unrooted().get_attribute(Null, "href").is_some()
            }
        }
        let filter = ~LinksFilter;
        HTMLCollection::create(&window.root_ref(), NodeCast::from_ref(abstract_self), filter)
    }

    pub fn Forms(&self, abstract_self: &JSRef<Document>) -> JS<HTMLCollection> {
        let roots = RootCollection::new();
        let window = self.window.root(&roots);

        // FIXME: https://github.com/mozilla/servo/issues/1847
        struct FormsFilter;
        impl CollectionFilter for FormsFilter {
            fn filter(&self, elem: &JSRef<Element>, _root: &JSRef<Node>) -> bool {
                elem.get().local_name == ~"form"
            }
        }
        let filter = ~FormsFilter;
        HTMLCollection::create(&window.root_ref(), NodeCast::from_ref(abstract_self), filter)
    }

    pub fn Scripts(&self, abstract_self: &JSRef<Document>) -> JS<HTMLCollection> {
        let roots = RootCollection::new();
        let window = self.window.root(&roots);

        // FIXME: https://github.com/mozilla/servo/issues/1847
        struct ScriptsFilter;
        impl CollectionFilter for ScriptsFilter {
            fn filter(&self, elem: &JSRef<Element>, _root: &JSRef<Node>) -> bool {
                elem.get().local_name == ~"script"
            }
        }
        let filter = ~ScriptsFilter;
        HTMLCollection::create(&window.root_ref(), NodeCast::from_ref(abstract_self), filter)
    }

    pub fn Anchors(&self, abstract_self: &JSRef<Document>) -> JS<HTMLCollection> {
        let roots = RootCollection::new();
        let window = self.window.root(&roots);

        // FIXME: https://github.com/mozilla/servo/issues/1847
        struct AnchorsFilter;
        impl CollectionFilter for AnchorsFilter {
            fn filter(&self, elem: &JSRef<Element>, _root: &JSRef<Node>) -> bool {
                elem.get().local_name == ~"a" && elem.unrooted().get_attribute(Null, "name").is_some()
            }
        }
        let filter = ~AnchorsFilter;
        HTMLCollection::create(&window.root_ref(), NodeCast::from_ref(abstract_self), filter)
    }

    pub fn Applets(&self, abstract_self: &JSRef<Document>) -> JS<HTMLCollection> {
        let roots = RootCollection::new();
        let window = self.window.root(&roots);

        // FIXME: This should be return OBJECT elements containing applets.
        struct AppletsFilter;
        impl CollectionFilter for AppletsFilter {
            fn filter(&self, elem: &JSRef<Element>, _root: &JSRef<Node>) -> bool {
                elem.get().local_name == ~"applet"
            }
        }
        let filter = ~AppletsFilter;
        HTMLCollection::create(&window.root_ref(), NodeCast::from_ref(abstract_self), filter)
    }

    pub fn Location(&mut self, _abstract_self: &JSRef<Document>) -> JS<Location> {
        let roots = RootCollection::new();
        let window = self.window.root(&roots);
        self.window.get_mut().Location(&window.root_ref())
    }

    pub fn Children(&self, abstract_self: &JSRef<Document>) -> JS<HTMLCollection> {
        let roots = RootCollection::new();
        let window = self.window.root(&roots);
        HTMLCollection::children(&window.root_ref(), NodeCast::from_ref(abstract_self))
    }

    pub fn createNodeList(&self, callback: |node: &JSRef<Node>| -> bool) -> JS<NodeList> {
        let roots = RootCollection::new();
        let window = self.window.root(&roots);

        let mut nodes = vec!();
        match self.GetDocumentElement() {
            None => {},
            Some(root) => {
                let root: JS<Node> = NodeCast::from(&root);
                for child in root.traverse_preorder() {
                    let child = child.root(&roots);
                    if callback(&child.root_ref()) {
                        nodes.push(child.root_ref().unrooted());
                    }
                }
            }
        }

        NodeList::new_simple_list(&window.root_ref(), nodes)
    }

    pub fn content_changed(&self) {
        self.damage_and_reflow(ContentChangedDocumentDamage);
    }

    pub fn damage_and_reflow(&self, damage: DocumentDamageLevel) {
        self.window.get().damage_and_reflow(damage);
    }

    pub fn wait_until_safe_to_modify_dom(&self) {
        self.window.get().wait_until_safe_to_modify_dom();
    }


    /// Remove any existing association between the provided id and any elements in this document.
    pub fn unregister_named_element(&mut self,
                                    abstract_self: &JSRef<Element>,
                                    id: DOMString) {
        let mut is_empty = false;
        match self.idmap.find_mut(&id) {
            None => {},
            Some(elements) => {
                let position = elements.iter()
                                       .position(|element| element == &abstract_self.unrooted())
                                       .expect("This element should be in registered.");
                elements.remove(position);
                is_empty = elements.is_empty();
            }
        }
        if is_empty {
            self.idmap.remove(&id);
        }
    }

    /// Associate an element present in this document with the provided id.
    pub fn register_named_element(&mut self,
                                  element: &JSRef<Element>,
                                  id: DOMString) {
        assert!({
            let node: JS<Node> = NodeCast::from(&element.unrooted());
            node.is_in_doc()
        });

        // FIXME https://github.com/mozilla/rust/issues/13195
        //       Use mangle() when it exists again.
        let root = self.GetDocumentElement().expect("The element is in the document, so there must be a document element.");
        match self.idmap.find_mut(&id) {
            Some(elements) => {
                let new_node = NodeCast::from_ref(element);
                let mut head : uint = 0u;
                let root: JS<Node> = NodeCast::from(&root);
                for node in root.traverse_preorder() {
                    match ElementCast::to(&node) {
                        Some(elem) => {
                            if elements.get(head) == &elem {
                                head = head + 1;
                            }
                            if new_node.unrooted() == node || head == elements.len() {
                                break;
                            }
                        }
                        None => {}
                    }
                }
                elements.insert(head, element.unrooted());
                return;
            },
            None => (),
        }
        self.idmap.insert(id, vec!(element.unrooted()));
    }
}
