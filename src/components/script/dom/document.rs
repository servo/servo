/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::InheritTypes::{DocumentDerived, EventCast, HTMLElementCast};
use dom::bindings::codegen::InheritTypes::{HTMLHeadElementCast, TextCast, ElementCast};
use dom::bindings::codegen::InheritTypes::{DocumentTypeCast, HTMLHtmlElementCast, NodeCast};
use dom::bindings::codegen::BindingDeclarations::DocumentBinding;
use dom::bindings::js::{JS, JSRef, Temporary, OptionalSettable, TemporaryPushable};
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
use dom::window::{Window, WindowMethods, WindowHelpers};
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
        self.type_id == NodeTargetTypeId(DocumentNodeTypeId)
    }
}

pub trait DocumentHelpers {
    fn url<'a>(&'a self) -> &'a Url;
    fn quirks_mode(&self) -> QuirksMode;
    fn set_quirks_mode(&mut self, mode: QuirksMode);
    fn set_encoding_name(&mut self, name: DOMString);
    fn content_changed(&self);
    fn damage_and_reflow(&self, damage: DocumentDamageLevel);
    fn wait_until_safe_to_modify_dom(&self);
    fn unregister_named_element(&mut self, to_unregister: &JSRef<Element>, id: DOMString);
    fn register_named_element(&mut self, element: &JSRef<Element>, id: DOMString);
}

impl<'a> DocumentHelpers for JSRef<'a, Document> {
    fn url<'a>(&'a self) -> &'a Url {
        &*self.url
    }

    fn quirks_mode(&self) -> QuirksMode {
        *self.quirks_mode
    }

    fn set_quirks_mode(&mut self, mode: QuirksMode) {
        *self.quirks_mode = mode;
    }

    fn set_encoding_name(&mut self, name: DOMString) {
        self.encoding_name = name;
    }

    fn content_changed(&self) {
        self.damage_and_reflow(ContentChangedDocumentDamage);
    }

    fn damage_and_reflow(&self, damage: DocumentDamageLevel) {
        self.window.root().damage_and_reflow(damage);
    }

    fn wait_until_safe_to_modify_dom(&self) {
        self.window.root().wait_until_safe_to_modify_dom();
    }


    /// Remove any existing association between the provided id and any elements in this document.
    fn unregister_named_element(&mut self,
                                to_unregister: &JSRef<Element>,
                                id: DOMString) {
        let mut is_empty = false;
        match self.idmap.find_mut(&id) {
            None => {},
            Some(elements) => {
                let position = elements.iter()
                                       .map(|elem| elem.root())
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
    fn register_named_element(&mut self,
                              element: &JSRef<Element>,
                              id: DOMString) {
        assert!({
            let node: &JSRef<Node> = NodeCast::from_ref(element);
            node.is_in_doc()
        });

        // FIXME https://github.com/mozilla/rust/issues/13195
        //       Use mangle() when it exists again.
        let root = self.GetDocumentElement().expect("The element is in the document, so there must be a document element.").root();
        match self.idmap.find_mut(&id) {
            Some(elements) => {
                let new_node: &JSRef<Node> = NodeCast::from_ref(element);
                let mut head : uint = 0u;
                let root: &JSRef<Node> = NodeCast::from_ref(&*root);
                for node in root.traverse_preorder() {
                    let elem: Option<&JSRef<Element>> = ElementCast::to_ref(&node);
                    match elem {
                        Some(elem) => {
                            if &*elements.get(head).root() == elem {
                                head = head + 1;
                            }
                            if new_node == &node || head == elements.len() {
                                break;
                            }
                        }
                        None => {}
                    }
                }
                elements.insert_unrooted(head, element);
                return;
            },
            None => (),
        }
        let mut elements = vec!();
        elements.push_unrooted(element);
        self.idmap.insert(id, elements);
    }
}

impl Document {
    pub fn reflect_document(document: ~Document,
                            window: &JSRef<Window>,
                            wrap_fn: extern "Rust" fn(*JSContext, &JSRef<Window>, ~Document) -> JS<Document>)
             -> Temporary<Document> {
        assert!(document.reflector().get_jsobject().is_null());
        let mut raw_doc = reflect_dom_object(document, window, wrap_fn).root();
        assert!(raw_doc.reflector().get_jsobject().is_not_null());

        let mut doc_alias = raw_doc.clone();
        let node: &mut JSRef<Node> = NodeCast::from_mut_ref(&mut doc_alias);
        node.set_owner_doc(&*raw_doc);
        Temporary::from_rooted(&*raw_doc)
    }

    pub fn new_inherited(window: &JSRef<Window>,
                         url: Option<Url>,
                         is_html_document: IsHTMLDocument,
                         content_type: Option<DOMString>) -> Document {
        let url = url.unwrap_or_else(|| from_str("about:blank").unwrap());

        Document {
            node: Node::new_without_doc(DocumentNodeTypeId),
            reflector_: Reflector::new(),
            window: window.unrooted(),
            idmap: HashMap::new(),
            implementation: None,
            content_type: match content_type {
                Some(string) => string.clone(),
                None => match is_html_document {
                    // http://dom.spec.whatwg.org/#dom-domimplementation-createhtmldocument
                    HTMLDocument => "text/html".to_owned(),
                    // http://dom.spec.whatwg.org/#concept-document-content-type
                    NonHTMLDocument => "application/xml".to_owned()
                }
            },
            url: Untraceable::new(url),
            // http://dom.spec.whatwg.org/#concept-document-quirks
            quirks_mode: Untraceable::new(NoQuirks),
            // http://dom.spec.whatwg.org/#concept-document-encoding
            encoding_name: "utf-8".to_owned(),
            is_html_document: is_html_document == HTMLDocument,
        }
    }

    // http://dom.spec.whatwg.org/#dom-document
    pub fn Constructor(owner: &JSRef<Window>) -> Fallible<Temporary<Document>> {
        Ok(Document::new(owner, None, NonHTMLDocument, None))
    }

    pub fn new(window: &JSRef<Window>, url: Option<Url>, doctype: IsHTMLDocument, content_type: Option<DOMString>) -> Temporary<Document> {
        let document = Document::new_inherited(window, url, doctype, content_type);
        Document::reflect_document(~document, window, DocumentBinding::Wrap)
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

trait PrivateDocumentHelpers {
    fn createNodeList(&self, callback: |node: &JSRef<Node>| -> bool) -> Temporary<NodeList>;
    fn get_html_element(&self) -> Option<Temporary<HTMLHtmlElement>>;
}

impl<'a> PrivateDocumentHelpers for JSRef<'a, Document> {
    fn createNodeList(&self, callback: |node: &JSRef<Node>| -> bool) -> Temporary<NodeList> {
        let window = self.window.root();

        match self.GetDocumentElement().root() {
            None => {
                NodeList::new_simple_list(&*window, vec!())
            },
            Some(root) => {
                let mut nodes = vec!();
                let root: &JSRef<Node> = NodeCast::from_ref(&*root);
                for child in root.traverse_preorder() {
                    if callback(&child) {
                        nodes.push(child);
                    }
                }
                NodeList::new_simple_list(&*window, nodes)
            }
        }

    }

    fn get_html_element(&self) -> Option<Temporary<HTMLHtmlElement>> {
        self.GetDocumentElement().root().filtered(|root| {
            root.node.type_id == ElementNodeTypeId(HTMLHtmlElementTypeId)
        }).map(|elem| {
            Temporary::from_rooted(HTMLHtmlElementCast::to_ref(&*elem).unwrap())
        })
    }
}

pub trait DocumentMethods {
    fn Implementation(&mut self) -> Temporary<DOMImplementation>;
    fn URL(&self) -> DOMString;
    fn DocumentURI(&self) -> DOMString;
    fn CompatMode(&self) -> DOMString;
    fn CharacterSet(&self) -> DOMString;
    fn ContentType(&self) -> DOMString;
    fn GetDoctype(&self) -> Option<Temporary<DocumentType>>;
    fn GetDocumentElement(&self) -> Option<Temporary<Element>>;
    fn GetElementsByTagName(&self, tag_name: DOMString) -> Temporary<HTMLCollection>;
    fn GetElementsByTagNameNS(&self, maybe_ns: Option<DOMString>, tag_name: DOMString) -> Temporary<HTMLCollection>;
    fn GetElementsByClassName(&self, classes: DOMString) -> Temporary<HTMLCollection>;
    fn GetElementById(&self, id: DOMString) -> Option<Temporary<Element>>;
    fn CreateElement(&self, local_name: DOMString) -> Fallible<Temporary<Element>>;
    fn CreateElementNS(&self, namespace: Option<DOMString>, qualified_name: DOMString) -> Fallible<Temporary<Element>>;
    fn CreateDocumentFragment(&self) -> Temporary<DocumentFragment>;
    fn CreateTextNode(&self, data: DOMString) -> Temporary<Text>;
    fn CreateComment(&self, data: DOMString) -> Temporary<Comment>;
    fn CreateProcessingInstruction(&self, target: DOMString, data: DOMString) -> Fallible<Temporary<ProcessingInstruction>>;
    fn ImportNode(&self, node: &JSRef<Node>, deep: bool) -> Fallible<Temporary<Node>>;
    fn AdoptNode(&self, node: &mut JSRef<Node>) -> Fallible<Temporary<Node>>;
    fn CreateEvent(&self, interface: DOMString) -> Fallible<Temporary<Event>>;
    fn Title(&self) -> DOMString;
    fn SetTitle(&self, title: DOMString) -> ErrorResult;
    fn GetHead(&self) -> Option<Temporary<HTMLHeadElement>>;
    fn GetBody(&self) -> Option<Temporary<HTMLElement>>;
    fn SetBody(&self, new_body: Option<JSRef<HTMLElement>>) -> ErrorResult;
    fn GetElementsByName(&self, name: DOMString) -> Temporary<NodeList>;
    fn Images(&self) -> Temporary<HTMLCollection>;
    fn Embeds(&self) -> Temporary<HTMLCollection>;
    fn Plugins(&self) -> Temporary<HTMLCollection>;
    fn Links(&self) -> Temporary<HTMLCollection>;
    fn Forms(&self) -> Temporary<HTMLCollection>;
    fn Scripts(&self) -> Temporary<HTMLCollection>;
    fn Anchors(&self) -> Temporary<HTMLCollection>;
    fn Applets(&self) -> Temporary<HTMLCollection>;
    fn Location(&mut self) -> Temporary<Location>;
    fn Children(&self) -> Temporary<HTMLCollection>;
}

impl<'a> DocumentMethods for JSRef<'a, Document> {
    // http://dom.spec.whatwg.org/#dom-document-implementation
    fn Implementation(&mut self) -> Temporary<DOMImplementation> {
        if self.implementation.is_none() {
            let window = self.window.root();
            self.implementation.assign(Some(DOMImplementation::new(&*window)));
        }
        Temporary::new(self.implementation.get_ref().clone())
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
            NoQuirks => "CSS1Compat".to_owned(),
            LimitedQuirks | FullQuirks => "BackCompat".to_owned()
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
    fn GetDoctype(&self) -> Option<Temporary<DocumentType>> {
        let node: &JSRef<Node> = NodeCast::from_ref(self);
        node.children().find(|child| {
            child.is_doctype()
        }).map(|node| {
            let doctype: &JSRef<DocumentType> = DocumentTypeCast::to_ref(&node).unwrap();
            Temporary::from_rooted(doctype)
        })
    }

    // http://dom.spec.whatwg.org/#dom-document-documentelement
    fn GetDocumentElement(&self) -> Option<Temporary<Element>> {
        let node: &JSRef<Node> = NodeCast::from_ref(self);
        node.child_elements().next().map(|elem| Temporary::from_rooted(&elem))
    }

    // http://dom.spec.whatwg.org/#dom-document-getelementsbytagname
    fn GetElementsByTagName(&self, tag_name: DOMString) -> Temporary<HTMLCollection> {
        let window = self.window.root();
        HTMLCollection::by_tag_name(&*window, NodeCast::from_ref(self), tag_name)
    }

    // http://dom.spec.whatwg.org/#dom-document-getelementsbytagnamens
    fn GetElementsByTagNameNS(&self, maybe_ns: Option<DOMString>, tag_name: DOMString) -> Temporary<HTMLCollection> {
        let window = self.window.root();

        let namespace = match maybe_ns {
            Some(namespace) => Namespace::from_str(namespace),
            None => Null
        };
        HTMLCollection::by_tag_name_ns(&*window, NodeCast::from_ref(self), tag_name, namespace)
    }

    // http://dom.spec.whatwg.org/#dom-document-getelementsbyclassname
    fn GetElementsByClassName(&self, classes: DOMString) -> Temporary<HTMLCollection> {
        let window = self.window.root();

        HTMLCollection::by_class_name(&*window, NodeCast::from_ref(self), classes)
    }

    // http://dom.spec.whatwg.org/#dom-nonelementparentnode-getelementbyid
    fn GetElementById(&self, id: DOMString) -> Option<Temporary<Element>> {
        match self.idmap.find_equiv(&id) {
            None => None,
            Some(ref elements) => Some(Temporary::new(elements.get(0).clone())),
        }
    }

    // http://dom.spec.whatwg.org/#dom-document-createelement
    fn CreateElement(&self, local_name: DOMString) -> Fallible<Temporary<Element>> {
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
                       qualified_name: DOMString) -> Fallible<Temporary<Element>> {
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
    fn CreateDocumentFragment(&self) -> Temporary<DocumentFragment> {
        DocumentFragment::new(self)
    }

    // http://dom.spec.whatwg.org/#dom-document-createtextnode
    fn CreateTextNode(&self, data: DOMString)
                          -> Temporary<Text> {
        Text::new(data, self)
    }

    // http://dom.spec.whatwg.org/#dom-document-createcomment
    fn CreateComment(&self, data: DOMString) -> Temporary<Comment> {
        Comment::new(data, self)
    }

    // http://dom.spec.whatwg.org/#dom-document-createprocessinginstruction
    fn CreateProcessingInstruction(&self, target: DOMString,
                                       data: DOMString) -> Fallible<Temporary<ProcessingInstruction>> {
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
    fn ImportNode(&self, node: &JSRef<Node>, deep: bool) -> Fallible<Temporary<Node>> {
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
    fn AdoptNode(&self, node: &mut JSRef<Node>) -> Fallible<Temporary<Node>> {
        // Step 1.
        if node.is_document() {
            return Err(NotSupported);
        }

        // Step 2.
        Node::adopt(node, self);

        // Step 3.
        Ok(Temporary::from_rooted(node))
    }

    // http://dom.spec.whatwg.org/#dom-document-createevent
    fn CreateEvent(&self, interface: DOMString) -> Fallible<Temporary<Event>> {
        let window = self.window.root();

        match interface.to_ascii_lower().as_slice() {
            // FIXME: Implement CustomEvent (http://dom.spec.whatwg.org/#customevent)
            "uievents" | "uievent" => Ok(EventCast::from_temporary(UIEvent::new_uninitialized(&*window))),
            "mouseevents" | "mouseevent" => Ok(EventCast::from_temporary(MouseEvent::new_uninitialized(&*window))),
            "htmlevents" | "events" | "event" => Ok(Event::new(&*window)),
            _ => Err(NotSupported)
        }
    }

    // http://www.whatwg.org/specs/web-apps/current-work/#document.title
    fn Title(&self) -> DOMString {
        let mut title = "".to_owned();
        self.GetDocumentElement().root().map(|root| {
            let root: &JSRef<Node> = NodeCast::from_ref(&*root);
            root.traverse_preorder()
                .find(|node| node.type_id() == ElementNodeTypeId(HTMLTitleElementTypeId))
                .map(|title_elem| {
                    for child in title_elem.children() {
                        if child.is_text() {
                            let text: &JSRef<Text> = TextCast::to_ref(&child).unwrap();
                            title.push_str(text.deref().characterdata.data.as_slice());
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
        self.GetDocumentElement().root().map(|root| {
            let root: &JSRef<Node> = NodeCast::from_ref(&*root);
            let mut head_node = root.traverse_preorder().find(|child| {
                child.type_id() == ElementNodeTypeId(HTMLHeadElementTypeId)
            });
            head_node.as_mut().map(|head| {
                let mut title_node = head.children().find(|child| {
                    child.type_id() == ElementNodeTypeId(HTMLTitleElementTypeId)
                });

                match title_node {
                    Some(ref mut title_node) => {
                        for mut title_child in title_node.children() {
                            assert!(title_node.RemoveChild(&mut title_child).is_ok());
                        }
                        let mut new_text = self.CreateTextNode(title.clone()).root();

                        assert!(title_node.AppendChild(NodeCast::from_mut_ref(&mut *new_text)).is_ok());
                    },
                    None => {
                        let mut new_title = HTMLTitleElement::new("title".to_owned(), self).root();
                        let new_title: &mut JSRef<Node> = NodeCast::from_mut_ref(&mut *new_title);

                        let mut new_text = self.CreateTextNode(title.clone()).root();

                        assert!(new_title.AppendChild(NodeCast::from_mut_ref(&mut *new_text)).is_ok());
                        assert!(head.AppendChild(&mut *new_title).is_ok());
                    },
                }
            });
        });
        Ok(())
    }

    // http://www.whatwg.org/specs/web-apps/current-work/#dom-document-head
    fn GetHead(&self) -> Option<Temporary<HTMLHeadElement>> {
        self.get_html_element().and_then(|root| {
            let root = root.root();
            let node: &JSRef<Node> = NodeCast::from_ref(&*root);
            node.children().find(|child| {
                child.type_id() == ElementNodeTypeId(HTMLHeadElementTypeId)
            }).map(|node| {
                Temporary::from_rooted(HTMLHeadElementCast::to_ref(&node).unwrap())
            })
        })
    }

    // http://www.whatwg.org/specs/web-apps/current-work/#dom-document-body
    fn GetBody(&self) -> Option<Temporary<HTMLElement>> {
        self.get_html_element().and_then(|root| {
            let root = root.root();
            let node: &JSRef<Node> = NodeCast::from_ref(&*root);
            node.children().find(|child| {
                match child.type_id() {
                    ElementNodeTypeId(HTMLBodyElementTypeId) |
                    ElementNodeTypeId(HTMLFrameSetElementTypeId) => true,
                    _ => false
                }
            }).map(|node| {
                Temporary::from_rooted(HTMLElementCast::to_ref(&node).unwrap())
            })
        })
    }

    // http://www.whatwg.org/specs/web-apps/current-work/#dom-document-body
    fn SetBody(&self, new_body: Option<JSRef<HTMLElement>>) -> ErrorResult {
        // Step 1.
        match new_body {
            Some(ref htmlelem) => {
                let node: &JSRef<Node> = NodeCast::from_ref(htmlelem);
                match node.type_id() {
                    ElementNodeTypeId(HTMLBodyElementTypeId) | ElementNodeTypeId(HTMLFrameSetElementTypeId) => {}
                    _ => return Err(HierarchyRequest)
                }
            }
            None => return Err(HierarchyRequest)
        }

        // Step 2.
        let mut old_body = self.GetBody().root();
        //FIXME: covariant lifetime workaround. do not judge.
        if old_body.as_ref().map(|body| body.deref()) == new_body.as_ref().map(|a| &*a) {
            return Ok(());
        }

        // Step 3.
        match self.get_html_element().root() {
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
    fn GetElementsByName(&self, name: DOMString) -> Temporary<NodeList> {
        self.createNodeList(|node| {
            if !node.is_element() {
                return false;
            }

            let element: &JSRef<Element> = ElementCast::to_ref(node).unwrap();
            element.get_attribute(Null, "name").root().map_or(false, |mut attr| {
                attr.value_ref() == name
            })
        })
    }

    fn Images(&self) -> Temporary<HTMLCollection> {
        let window = self.window.root();

        // FIXME: https://github.com/mozilla/servo/issues/1847
        struct ImagesFilter;
        impl CollectionFilter for ImagesFilter {
            fn filter(&self, elem: &JSRef<Element>, _root: &JSRef<Node>) -> bool {
                "img" == elem.deref().local_name
            }
        }
        let filter = ~ImagesFilter;
        HTMLCollection::create(&*window, NodeCast::from_ref(self), filter)
    }

    fn Embeds(&self) -> Temporary<HTMLCollection> {
        let window = self.window.root();

        // FIXME: https://github.com/mozilla/servo/issues/1847
        struct EmbedsFilter;
        impl CollectionFilter for EmbedsFilter {
            fn filter(&self, elem: &JSRef<Element>, _root: &JSRef<Node>) -> bool {
                "embed" == elem.deref().local_name
            }
        }
        let filter = ~EmbedsFilter;
        HTMLCollection::create(&*window, NodeCast::from_ref(self), filter)
    }

    fn Plugins(&self) -> Temporary<HTMLCollection> {
        // FIXME: https://github.com/mozilla/servo/issues/1847
        self.Embeds()
    }

    fn Links(&self) -> Temporary<HTMLCollection> {
        let window = self.window.root();

        // FIXME: https://github.com/mozilla/servo/issues/1847
        struct LinksFilter;
        impl CollectionFilter for LinksFilter {
            fn filter(&self, elem: &JSRef<Element>, _root: &JSRef<Node>) -> bool {
                ("a" == elem.deref().local_name || "area" == elem.deref().local_name) &&
                elem.get_attribute(Null, "href").is_some()
            }
        }
        let filter = ~LinksFilter;
        HTMLCollection::create(&*window, NodeCast::from_ref(self), filter)
    }

    fn Forms(&self) -> Temporary<HTMLCollection> {
        let window = self.window.root();

        // FIXME: https://github.com/mozilla/servo/issues/1847
        struct FormsFilter;
        impl CollectionFilter for FormsFilter {
            fn filter(&self, elem: &JSRef<Element>, _root: &JSRef<Node>) -> bool {
                "form" == elem.deref().local_name
            }
        }
        let filter = ~FormsFilter;
        HTMLCollection::create(&*window, NodeCast::from_ref(self), filter)
    }

    fn Scripts(&self) -> Temporary<HTMLCollection> {
        let window = self.window.root();

        // FIXME: https://github.com/mozilla/servo/issues/1847
        struct ScriptsFilter;
        impl CollectionFilter for ScriptsFilter {
            fn filter(&self, elem: &JSRef<Element>, _root: &JSRef<Node>) -> bool {
                "script" == elem.deref().local_name
            }
        }
        let filter = ~ScriptsFilter;
        HTMLCollection::create(&*window, NodeCast::from_ref(self), filter)
    }

    fn Anchors(&self) -> Temporary<HTMLCollection> {
        let window = self.window.root();

        // FIXME: https://github.com/mozilla/servo/issues/1847
        struct AnchorsFilter;
        impl CollectionFilter for AnchorsFilter {
            fn filter(&self, elem: &JSRef<Element>, _root: &JSRef<Node>) -> bool {
                "a" == elem.deref().local_name && elem.get_attribute(Null, "name").is_some()
            }
        }
        let filter = ~AnchorsFilter;
        HTMLCollection::create(&*window, NodeCast::from_ref(self), filter)
    }

    fn Applets(&self) -> Temporary<HTMLCollection> {
        let window = self.window.root();

        // FIXME: This should be return OBJECT elements containing applets.
        struct AppletsFilter;
        impl CollectionFilter for AppletsFilter {
            fn filter(&self, elem: &JSRef<Element>, _root: &JSRef<Node>) -> bool {
                "applet" == elem.deref().local_name
            }
        }
        let filter = ~AppletsFilter;
        HTMLCollection::create(&*window, NodeCast::from_ref(self), filter)
    }

    fn Location(&mut self) -> Temporary<Location> {
        let mut window = self.window.root();
        window.Location()
    }

    fn Children(&self) -> Temporary<HTMLCollection> {
        let window = self.window.root();
        HTMLCollection::children(&*window, NodeCast::from_ref(self))
    }
}
