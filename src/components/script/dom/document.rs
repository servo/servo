/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::InheritTypes::{DocumentDerived, EventCast, HTMLElementCast};
use dom::bindings::codegen::InheritTypes::{HTMLHeadElementCast, TextCast, ElementCast};
use dom::bindings::codegen::InheritTypes::{DocumentTypeCast, HTMLHtmlElementCast, NodeCast};
use dom::bindings::codegen::BindingDeclarations::DocumentBinding;
use dom::bindings::js::{JS, JSRef, RootCollection, Unrooted, OptionalAssignable};
use dom::bindings::js::OptionalRootable;
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
use dom::htmlelement::HTMLElement;
use dom::htmlheadelement::HTMLHeadElement;
use dom::htmlhtmlelement::HTMLHtmlElement;
use dom::htmltitleelement::HTMLTitleElement;
use dom::mouseevent::MouseEvent;
use dom::node::{Node, ElementNodeTypeId, DocumentNodeTypeId, NodeHelpers, NodeMethods};
use dom::node::{CloneChildren, DoNotCloneChildren};
use dom::nodelist::NodeList;
use dom::text::Text;
use dom::processinginstruction::ProcessingInstruction;
use dom::uievent::UIEvent;
use dom::window::{Window, WindowMethods};
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
    pub fn reflect_document(document: ~Document,
                            window:   &JSRef<Window>,
                            wrap_fn:  extern "Rust" fn(*JSContext, &JSRef<Window>, ~Document) -> JS<Document>)
             -> Unrooted<Document> {
        let roots = RootCollection::new();
        assert!(document.reflector().get_jsobject().is_null());
        let mut raw_doc = reflect_dom_object(document, window, wrap_fn).root(&roots);
        assert!(raw_doc.reflector().get_jsobject().is_not_null());

        let mut doc_alias = raw_doc.clone();
        let node: &mut JSRef<Node> = NodeCast::from_mut_ref(&mut doc_alias);
        node.get_mut().set_owner_doc(&*raw_doc);
        Unrooted::new_rooted(&*raw_doc)
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

    // http://dom.spec.whatwg.org/#dom-document
    pub fn Constructor(owner: &JSRef<Window>) -> Fallible<Unrooted<Document>> {
        Ok(Document::new(owner, None, NonHTMLDocument, None))
    }

    pub fn new(window: &JSRef<Window>, url: Option<Url>, doctype: IsHTMLDocument, content_type: Option<DOMString>) -> Unrooted<Document> {
        let document = Document::new_inherited(window.unrooted(), url, doctype, content_type);
        Document::reflect_document(~document, window, DocumentBinding::Wrap)
    }

    pub fn url<'a>(&'a self) -> &'a Url {
        &*self.url
    }

    pub fn quirks_mode(&self) -> QuirksMode {
        *self.quirks_mode
    }

    pub fn set_quirks_mode(&mut self, mode: QuirksMode) {
        *self.quirks_mode = mode;
    }

    pub fn set_encoding_name(&mut self, name: DOMString) {
        self.encoding_name = name;
    }

    pub fn content_changed(&self) {
        self.damage_and_reflow(ContentChangedDocumentDamage);
    }

    pub fn damage_and_reflow(&self, damage: DocumentDamageLevel) {
        let roots = RootCollection::new();
        self.window.root(&roots).damage_and_reflow(damage);
    }

    pub fn wait_until_safe_to_modify_dom(&self) {
        let roots = RootCollection::new();
        self.window.root(&roots).wait_until_safe_to_modify_dom();
    }


    /// Remove any existing association between the provided id and any elements in this document.
    pub fn unregister_named_element(&mut self,
                                    to_unregister: &JSRef<Element>,
                                    id: DOMString) {
        let roots = RootCollection::new();
        let mut is_empty = false;
        match self.idmap.find_mut(&id) {
            None => {},
            Some(elements) => {
                let position = elements.iter()
                                       .map(|elem| elem.root(&roots))
                                       .position(|element| &*element == to_unregister)
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
                                  abstract_self: &JSRef<Document>,
                                  element: &JSRef<Element>,
                                  id: DOMString) {
        let roots = RootCollection::new();
        assert!({
            let node: &JSRef<Node> = NodeCast::from_ref(element);
            node.is_in_doc()
        });

        // FIXME https://github.com/mozilla/rust/issues/13195
        //       Use mangle() when it exists again.
        let root = abstract_self.GetDocumentElement().expect("The element is in the document, so there must be a document element.").root(&roots);
        match self.idmap.find_mut(&id) {
            Some(elements) => {
                let new_node: &JSRef<Node> = NodeCast::from_ref(element);
                let mut head : uint = 0u;
                let root: &JSRef<Node> = NodeCast::from_ref(&*root);
                for node in root.traverse_preorder(&roots) {
                    let elem: Option<&JSRef<Element>> = ElementCast::to_ref(&node);
                    match elem {
                        Some(elem) => {
                            if elements.get(head) == &elem.unrooted() {
                                head = head + 1;
                            }
                            if new_node == &node || head == elements.len() {
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

impl Reflectable for Document {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.node.reflector()
    }

    fn mut_reflector<'a>(&'a mut self) -> &'a mut Reflector {
        self.node.mut_reflector()
    }
}

trait DocumentHelpers {
    fn createNodeList(&self, callback: |node: &JSRef<Node>| -> bool) -> Unrooted<NodeList>;
    fn get_html_element(&self) -> Option<Unrooted<HTMLHtmlElement>>;
}

impl<'a> DocumentHelpers for JSRef<'a, Document> {
    fn createNodeList(&self, callback: |node: &JSRef<Node>| -> bool) -> Unrooted<NodeList> {
        let roots = RootCollection::new();
        let window = self.window.root(&roots);

        let mut nodes = vec!();
        match self.GetDocumentElement().root(&roots) {
            None => {},
            Some(root) => {
                let root: &JSRef<Node> = NodeCast::from_ref(&*root);
                for child in root.traverse_preorder(&roots) {
                    if callback(&child) {
                        nodes.push(child);
                    }
                }
            }
        }

        NodeList::new_simple_list(&*window, nodes)
    }

    fn get_html_element(&self) -> Option<Unrooted<HTMLHtmlElement>> {
        let roots = RootCollection::new();
        self.GetDocumentElement().root(&roots).filtered(|root| {
            root.node.type_id == ElementNodeTypeId(HTMLHtmlElementTypeId)
        }).map(|elem| {
            Unrooted::new_rooted(HTMLHtmlElementCast::to_ref(&*elem).unwrap())
        })
    }
}

pub trait DocumentMethods {
    fn Implementation(&mut self) -> Unrooted<DOMImplementation>;
    fn URL(&self) -> DOMString;
    fn DocumentURI(&self) -> DOMString;
    fn CompatMode(&self) -> DOMString;
    fn CharacterSet(&self) -> DOMString;
    fn ContentType(&self) -> DOMString;
    fn GetDoctype(&self) -> Option<Unrooted<DocumentType>>;
    fn GetDocumentElement(&self) -> Option<Unrooted<Element>>;
    fn GetElementsByTagName(&self, tag_name: DOMString) -> Unrooted<HTMLCollection>;
    fn GetElementsByTagNameNS(&self, maybe_ns: Option<DOMString>, tag_name: DOMString) -> Unrooted<HTMLCollection>;
    fn GetElementsByClassName(&self, classes: DOMString) -> Unrooted<HTMLCollection>;
    fn GetElementById(&self, id: DOMString) -> Option<Unrooted<Element>>;
    fn CreateElement(&self, local_name: DOMString) -> Fallible<Unrooted<Element>>;
    fn CreateElementNS(&self, namespace: Option<DOMString>, qualified_name: DOMString) -> Fallible<Unrooted<Element>>;
    fn CreateDocumentFragment(&self) -> Unrooted<DocumentFragment>;
    fn CreateTextNode(&self, data: DOMString) -> Unrooted<Text>;
    fn CreateComment(&self, data: DOMString) -> Unrooted<Comment>;
    fn CreateProcessingInstruction(&self, target: DOMString, data: DOMString) -> Fallible<Unrooted<ProcessingInstruction>>;
    fn ImportNode(&self, node: &JSRef<Node>, deep: bool) -> Fallible<Unrooted<Node>>;
    fn AdoptNode(&self, node: &mut JSRef<Node>) -> Fallible<Unrooted<Node>>;
    fn CreateEvent(&self, interface: DOMString) -> Fallible<Unrooted<Event>>;
    fn Title(&self) -> DOMString;
    fn SetTitle(&self, title: DOMString) -> ErrorResult;
    fn GetHead(&self) -> Option<Unrooted<HTMLHeadElement>>;
    fn GetBody(&self) -> Option<Unrooted<HTMLElement>>;
    fn SetBody(&self, new_body: Option<JSRef<HTMLElement>>) -> ErrorResult;
    fn GetElementsByName(&self, name: DOMString) -> Unrooted<NodeList>;
    fn Images(&self) -> Unrooted<HTMLCollection>;
    fn Embeds(&self) -> Unrooted<HTMLCollection>;
    fn Plugins(&self) -> Unrooted<HTMLCollection>;
    fn Links(&self) -> Unrooted<HTMLCollection>;
    fn Forms(&self) -> Unrooted<HTMLCollection>;
    fn Scripts(&self) -> Unrooted<HTMLCollection>;
    fn Anchors(&self) -> Unrooted<HTMLCollection>;
    fn Applets(&self) -> Unrooted<HTMLCollection>;
    fn Location(&mut self) -> Unrooted<Location>;
    fn Children(&self) -> Unrooted<HTMLCollection>;
}

impl<'a> DocumentMethods for JSRef<'a, Document> {
    // http://dom.spec.whatwg.org/#dom-document-implementation
    fn Implementation(&mut self) -> Unrooted<DOMImplementation> {
        if self.implementation.is_none() {
            let roots = RootCollection::new();
            let window = self.window.root(&roots);
            self.implementation.assign(Some(DOMImplementation::new(&*window)));
        }
        Unrooted::new(self.implementation.get_ref().clone())
    }

    // http://dom.spec.whatwg.org/#dom-document-url
    fn URL(&self) -> DOMString {
        self.url().to_str()
    }

    // http://dom.spec.whatwg.org/#dom-document-documenturi
    fn DocumentURI(&self) -> DOMString {
        self.URL()
    }

    // http://dom.spec.whatwg.org/#dom-document-compatmode
    fn CompatMode(&self) -> DOMString {
        match *self.quirks_mode {
            NoQuirks => ~"CSS1Compat",
            LimitedQuirks | FullQuirks => ~"BackCompat"
        }
    }

    // http://dom.spec.whatwg.org/#dom-document-characterset
    fn CharacterSet(&self) -> DOMString {
        self.encoding_name.to_ascii_lower()
    }

    // http://dom.spec.whatwg.org/#dom-document-content_type
    fn ContentType(&self) -> DOMString {
        self.content_type.clone()
    }

    // http://dom.spec.whatwg.org/#dom-document-doctype
    fn GetDoctype(&self) -> Option<Unrooted<DocumentType>> {
        self.node.children().find(|child| {
            child.is_doctype()
        }).map(|node| {
            let doctype: &JSRef<DocumentType> = DocumentTypeCast::to_ref(&node).unwrap();
            Unrooted::new(doctype.unrooted())
        })
    }

    // http://dom.spec.whatwg.org/#dom-document-documentelement
    fn GetDocumentElement(&self) -> Option<Unrooted<Element>> {
        self.node.child_elements().next().map(|elem| Unrooted::new_rooted(&elem))
    }

    // http://dom.spec.whatwg.org/#dom-document-getelementsbytagname
    fn GetElementsByTagName(&self, tag_name: DOMString) -> Unrooted<HTMLCollection> {
        let roots = RootCollection::new();
        let window = self.window.root(&roots);
        HTMLCollection::by_tag_name(&*window, NodeCast::from_ref(self), tag_name)
    }

    // http://dom.spec.whatwg.org/#dom-document-getelementsbytagnamens
    fn GetElementsByTagNameNS(&self, maybe_ns: Option<DOMString>, tag_name: DOMString) -> Unrooted<HTMLCollection> {
        let roots = RootCollection::new();
        let window = self.window.root(&roots);

        let namespace = match maybe_ns {
            Some(namespace) => Namespace::from_str(namespace),
            None => Null
        };
        HTMLCollection::by_tag_name_ns(&*window, NodeCast::from_ref(self), tag_name, namespace)
    }

    // http://dom.spec.whatwg.org/#dom-document-getelementsbyclassname
    fn GetElementsByClassName(&self, classes: DOMString) -> Unrooted<HTMLCollection> {
        let roots = RootCollection::new();
        let window = self.window.root(&roots);

        HTMLCollection::by_class_name(&*window, NodeCast::from_ref(self), classes)
    }

    // http://dom.spec.whatwg.org/#dom-nonelementparentnode-getelementbyid
    fn GetElementById(&self, id: DOMString) -> Option<Unrooted<Element>> {
        match self.idmap.find_equiv(&id) {
            None => None,
            Some(ref elements) => Some(Unrooted::new(elements.get(0).clone())),
        }
    }

    // http://dom.spec.whatwg.org/#dom-document-createelement
    fn CreateElement(&self, local_name: DOMString) -> Fallible<Unrooted<Element>> {
        if xml_name_type(local_name) == InvalidXMLName {
            debug!("Not a valid element name");
            return Err(InvalidCharacter);
        }
        let local_name = local_name.to_ascii_lower();
        Ok(build_element_from_tag(local_name, self))
    }

    // http://dom.spec.whatwg.org/#dom-document-createelementns
    fn CreateElementNS(&self,
                       namespace: Option<DOMString>,
                       qualified_name: DOMString) -> Fallible<Unrooted<Element>> {
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
            Ok(build_element_from_tag(local_name_from_qname, self))
        } else {
            Ok(Element::new(local_name_from_qname, ns, prefix_from_qname, self))
        }
    }

    // http://dom.spec.whatwg.org/#dom-document-createdocumentfragment
    fn CreateDocumentFragment(&self) -> Unrooted<DocumentFragment> {
        DocumentFragment::new(self)
    }

    // http://dom.spec.whatwg.org/#dom-document-createtextnode
    fn CreateTextNode(&self, data: DOMString)
                          -> Unrooted<Text> {
        Text::new(data, self)
    }

    // http://dom.spec.whatwg.org/#dom-document-createcomment
    fn CreateComment(&self, data: DOMString) -> Unrooted<Comment> {
        Comment::new(data, self)
    }

    // http://dom.spec.whatwg.org/#dom-document-createprocessinginstruction
    fn CreateProcessingInstruction(&self, target: DOMString,
                                       data: DOMString) -> Fallible<Unrooted<ProcessingInstruction>> {
        // Step 1.
        if xml_name_type(target) == InvalidXMLName {
            return Err(InvalidCharacter);
        }

        // Step 2.
        if data.contains("?>") {
            return Err(InvalidCharacter);
        }

        // Step 3.
        Ok(ProcessingInstruction::new(target, data, self))
    }

    // http://dom.spec.whatwg.org/#dom-document-importnode
    fn ImportNode(&self, node: &JSRef<Node>, deep: bool) -> Fallible<Unrooted<Node>> {
        // Step 1.
        if node.is_document() {
            return Err(NotSupported);
        }

        // Step 2.
        let clone_children = match deep {
            true => CloneChildren,
            false => DoNotCloneChildren
        };

        Ok(Node::clone(node, Some(self), clone_children))
    }

    // http://dom.spec.whatwg.org/#dom-document-adoptnode
    fn AdoptNode(&self, node: &mut JSRef<Node>) -> Fallible<Unrooted<Node>> {
        // Step 1.
        if node.is_document() {
            return Err(NotSupported);
        }

        // Step 2.
        Node::adopt(node, self);

        // Step 3.
        Ok(Unrooted::new_rooted(node))
    }

    // http://dom.spec.whatwg.org/#dom-document-createevent
    fn CreateEvent(&self, interface: DOMString) -> Fallible<Unrooted<Event>> {
        let roots = RootCollection::new();
        let window = self.window.root(&roots);

        match interface.to_ascii_lower().as_slice() {
            // FIXME: Implement CustomEvent (http://dom.spec.whatwg.org/#customevent)
            "uievents" | "uievent" => Ok(EventCast::from_unrooted(UIEvent::new(&*window))),
            "mouseevents" | "mouseevent" => Ok(EventCast::from_unrooted(MouseEvent::new(&*window))),
            "htmlevents" | "events" | "event" => Ok(Event::new(&*window)),
            _ => Err(NotSupported)
        }
    }

    // http://www.whatwg.org/specs/web-apps/current-work/#document.title
    fn Title(&self) -> DOMString {
        let mut title = ~"";
        let roots = RootCollection::new();
        self.GetDocumentElement().root(&roots).map(|root| {
            let root: &JSRef<Node> = NodeCast::from_ref(&*root);
            root.traverse_preorder(&roots)
                .find(|node| node.type_id() == ElementNodeTypeId(HTMLTitleElementTypeId))
                .map(|title_elem| {
                    for child in title_elem.deref().children() {
                        if child.is_text() {
                            let text: &JSRef<Text> = TextCast::to_ref(&child).unwrap();
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
    fn SetTitle(&self, title: DOMString) -> ErrorResult {
        let roots = RootCollection::new();

        self.GetDocumentElement().root(&roots).map(|root| {
            let root: &JSRef<Node> = NodeCast::from_ref(&*root);
            let mut head_node = root.traverse_preorder(&roots).find(|child| {
                child.get().type_id == ElementNodeTypeId(HTMLHeadElementTypeId)
            });
            head_node.as_mut().map(|head| {
                let mut title_node = head.children().find(|child| {
                    child.get().type_id == ElementNodeTypeId(HTMLTitleElementTypeId)
                });

                match title_node {
                    Some(ref mut title_node) => {
                        for mut title_child in title_node.children() {
                            assert!(title_node.RemoveChild(&mut title_child).is_ok());
                        }
                        let mut new_text = self.CreateTextNode(title.clone()).root(&roots);

                        assert!(title_node.AppendChild(NodeCast::from_mut_ref(&mut *new_text)).is_ok());
                    },
                    None => {
                        let mut new_title = HTMLTitleElement::new(~"title", self).root(&roots);
                        let new_title: &mut JSRef<Node> = NodeCast::from_mut_ref(&mut *new_title);

                        let mut new_text = self.CreateTextNode(title.clone()).root(&roots);

                        assert!(new_title.AppendChild(NodeCast::from_mut_ref(&mut *new_text)).is_ok());
                        assert!(head.AppendChild(&mut *new_title).is_ok());
                    },
                }
            });
        });
        Ok(())
    }

    // http://www.whatwg.org/specs/web-apps/current-work/#dom-document-head
    fn GetHead(&self) -> Option<Unrooted<HTMLHeadElement>> {
        let roots = RootCollection::new();
        self.get_html_element().and_then(|root| {
            let root = root.root(&roots);
            let node: &JSRef<Node> = NodeCast::from_ref(&*root);
            node.children().find(|child| {
                child.type_id() == ElementNodeTypeId(HTMLHeadElementTypeId)
            }).map(|node| {
                Unrooted::new_rooted(HTMLHeadElementCast::to_ref(&node).unwrap())
            })
        })
    }

    // http://www.whatwg.org/specs/web-apps/current-work/#dom-document-body
    fn GetBody(&self) -> Option<Unrooted<HTMLElement>> {
        let roots = RootCollection::new();
        self.get_html_element().and_then(|root| {
            let root = root.root(&roots);
            let node: &JSRef<Node> = NodeCast::from_ref(&*root);
            node.children().find(|child| {
                match child.type_id() {
                    ElementNodeTypeId(HTMLBodyElementTypeId) |
                    ElementNodeTypeId(HTMLFrameSetElementTypeId) => true,
                    _ => false
                }
            }).map(|node| {
                Unrooted::new_rooted(HTMLElementCast::to_ref(&node).unwrap())
            })
        })
    }

    // http://www.whatwg.org/specs/web-apps/current-work/#dom-document-body
    fn SetBody(&self, new_body: Option<JSRef<HTMLElement>>) -> ErrorResult {
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
        let mut old_body = self.GetBody().root(&roots);
        //FIXME: covariant lifetime workaround. do not judge.
        if old_body.as_ref().map(|body| body.deref()) == new_body.as_ref().map(|a| &*a) {
            return Ok(());
        }

        // Step 3.
        match self.get_html_element().root(&roots) {
            // Step 4.
            None => return Err(HierarchyRequest),
            Some(ref mut root) => {
                let mut new_body_unwrapped = new_body.unwrap();
                let new_body: &mut JSRef<Node> = NodeCast::from_mut_ref(&mut new_body_unwrapped);

                let root: &mut JSRef<Node> = NodeCast::from_mut_ref(&mut **root);
                match old_body {
                    Some(ref mut child) => {
                        let child: &mut JSRef<Node> = NodeCast::from_mut_ref(&mut **child);

                        assert!(root.ReplaceChild(new_body, child).is_ok())
                    }
                    None => assert!(root.AppendChild(new_body).is_ok())
                };
            }
        }
        Ok(())
    }

    // http://www.whatwg.org/specs/web-apps/current-work/#dom-document-getelementsbyname
    fn GetElementsByName(&self, name: DOMString) -> Unrooted<NodeList> {
        let roots = RootCollection::new();

        self.createNodeList(|node| {
            if !node.get().is_element() {
                return false;
            }

            let element: &JSRef<Element> = ElementCast::to_ref(node).unwrap();
            element.get_attribute(Null, "name").root(&roots).map_or(false, |mut attr| {
                attr.value_ref() == name
            })
        })
    }

    fn Images(&self) -> Unrooted<HTMLCollection> {
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
        HTMLCollection::create(&*window, NodeCast::from_ref(self), filter)
    }

    fn Embeds(&self) -> Unrooted<HTMLCollection> {
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
        HTMLCollection::create(&*window, NodeCast::from_ref(self), filter)
    }

    fn Plugins(&self) -> Unrooted<HTMLCollection> {
        // FIXME: https://github.com/mozilla/servo/issues/1847
        self.Embeds()
    }

    fn Links(&self) -> Unrooted<HTMLCollection> {
        let roots = RootCollection::new();
        let window = self.window.root(&roots);

        // FIXME: https://github.com/mozilla/servo/issues/1847
        struct LinksFilter;
        impl CollectionFilter for LinksFilter {
            fn filter(&self, elem: &JSRef<Element>, _root: &JSRef<Node>) -> bool {
                (elem.get().local_name == ~"a" || elem.get().local_name == ~"area") &&
                elem.get_attribute(Null, "href").is_some()
            }
        }
        let filter = ~LinksFilter;
        HTMLCollection::create(&*window, NodeCast::from_ref(self), filter)
    }

    fn Forms(&self) -> Unrooted<HTMLCollection> {
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
        HTMLCollection::create(&*window, NodeCast::from_ref(self), filter)
    }

    fn Scripts(&self) -> Unrooted<HTMLCollection> {
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
        HTMLCollection::create(&*window, NodeCast::from_ref(self), filter)
    }

    fn Anchors(&self) -> Unrooted<HTMLCollection> {
        let roots = RootCollection::new();
        let window = self.window.root(&roots);

        // FIXME: https://github.com/mozilla/servo/issues/1847
        struct AnchorsFilter;
        impl CollectionFilter for AnchorsFilter {
            fn filter(&self, elem: &JSRef<Element>, _root: &JSRef<Node>) -> bool {
                elem.get().local_name == ~"a" && elem.get_attribute(Null, "name").is_some()
            }
        }
        let filter = ~AnchorsFilter;
        HTMLCollection::create(&*window, NodeCast::from_ref(self), filter)
    }

    fn Applets(&self) -> Unrooted<HTMLCollection> {
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
        HTMLCollection::create(&*window, NodeCast::from_ref(self), filter)
    }

    fn Location(&mut self) -> Unrooted<Location> {
        let roots = RootCollection::new();
        let mut window = self.window.root(&roots);
        window.Location()
    }

    fn Children(&self) -> Unrooted<HTMLCollection> {
        let roots = RootCollection::new();
        let window = self.window.root(&roots);
        HTMLCollection::children(&*window, NodeCast::from_ref(self))
    }
}
