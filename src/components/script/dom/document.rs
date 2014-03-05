/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::InheritTypes::{DocumentDerived, EventCast, HTMLElementCast};
use dom::bindings::codegen::InheritTypes::{DocumentBase, NodeCast, DocumentCast};
use dom::bindings::codegen::InheritTypes::{HTMLHeadElementCast, TextCast, ElementCast};
use dom::bindings::codegen::InheritTypes::{DocumentTypeCast, HTMLHtmlElementCast};
use dom::bindings::codegen::DocumentBinding;
use dom::bindings::js::JS;
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::bindings::error::{ErrorResult, Fallible, NotSupported, InvalidCharacter, HierarchyRequest};
use dom::bindings::utils::{xml_name_type, InvalidXMLName};
use dom::comment::Comment;
use dom::documentfragment::DocumentFragment;
use dom::documenttype::DocumentType;
use dom::domimplementation::DOMImplementation;
use dom::element::{Element};
use dom::element::{HTMLHtmlElementTypeId, HTMLHeadElementTypeId, HTMLTitleElementTypeId};
use dom::element::{HTMLBodyElementTypeId, HTMLFrameSetElementTypeId};
use dom::event::Event;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlcollection::HTMLCollection;
use dom::nodelist::NodeList;
use dom::htmlelement::HTMLElement;
use dom::htmlheadelement::HTMLHeadElement;
use dom::htmlhtmlelement::HTMLHtmlElement;
use dom::htmltitleelement::HTMLTitleElement;
use dom::mouseevent::MouseEvent;
use dom::node::{Node, ElementNodeTypeId, DocumentNodeTypeId, NodeHelpers, INode};
use dom::text::Text;
use dom::processinginstruction::ProcessingInstruction;
use dom::uievent::UIEvent;
use dom::window::Window;
use html::hubbub_html_parser::build_element_from_tag;
use hubbub::hubbub::{QuirksMode, NoQuirks, LimitedQuirks, FullQuirks};
use layout_interface::{DocumentDamageLevel, ContentChangedDocumentDamage};
use servo_util::namespace::Null;
use servo_util::str::DOMString;

use extra::url::{Url, from_str};
use js::jsapi::{JSObject, JSContext};
use std::ascii::StrAsciiExt;
use std::hashmap::HashMap;

use extra::serialize::{Encoder, Encodable};

#[deriving(Eq,Encodable)]
pub enum IsHTMLDocument {
    HTMLDocument,
    NonHTMLDocument,
}

#[deriving(Encodable)]
pub struct Document {
    node: Node,
    reflector_: Reflector,
    window: JS<Window>,
    idmap: HashMap<DOMString, JS<Element>>,
    implementation: Option<JS<DOMImplementation>>,
    content_type: DOMString,
    encoding_name: DOMString,
    is_html_document: bool,
    extra: Untraceable,
}

struct Untraceable {
    url: Url,
    quirks_mode: QuirksMode,
}

impl<S: Encoder> Encodable<S> for Untraceable {
    fn encode(&self, _: &mut S) {
    }
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
             window:    &JS<Window>,
             wrap_fn:   extern "Rust" fn(*JSContext, &JS<Window>, ~D) -> *JSObject)
             -> JS<D> {
        assert!(document.reflector().get_jsobject().is_null());
        let raw_doc = reflect_dom_object(document, window, wrap_fn);
        assert!(raw_doc.reflector().get_jsobject().is_not_null());

        let document = DocumentCast::from(&raw_doc);
        let mut node: JS<Node> = NodeCast::from(&document);
        node.get_mut().set_owner_doc(&document);
        raw_doc
    }

    pub fn new_inherited(window: JS<Window>,
                         url: Option<Url>,
                         is_html_document: IsHTMLDocument,
                         content_type: Option<DOMString>) -> Document {
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
            extra: Untraceable {
                url: match url {
                    None => from_str("about:blank").unwrap(),
                    Some(_url) => _url
                },
                // http://dom.spec.whatwg.org/#concept-document-quirks
                quirks_mode: NoQuirks,
            },
            // http://dom.spec.whatwg.org/#concept-document-encoding
            encoding_name: ~"utf-8",
            is_html_document: is_html_document == HTMLDocument,
        }
    }

    pub fn new(window: &JS<Window>, url: Option<Url>, doctype: IsHTMLDocument, content_type: Option<DOMString>) -> JS<Document> {
        let document = Document::new_inherited(window.clone(), url, doctype, content_type);
        Document::reflect_document(~document, window, DocumentBinding::Wrap)
    }
}

impl Document {
    // http://dom.spec.whatwg.org/#dom-document
    pub fn Constructor(owner: &JS<Window>) -> Fallible<JS<Document>> {
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
            self.implementation = Some(DOMImplementation::new(&self.window));
        }
        self.implementation.get_ref().clone()
    }

    // http://dom.spec.whatwg.org/#dom-document-url
    pub fn URL(&self) -> DOMString {
        self.extra.url.to_str()
    }

    // http://dom.spec.whatwg.org/#dom-document-documenturi
    pub fn DocumentURI(&self) -> DOMString {
        self.URL()
    }

    // http://dom.spec.whatwg.org/#dom-document-compatmode
    pub fn CompatMode(&self) -> DOMString {
        match self.extra.quirks_mode {
            NoQuirks => ~"CSS1Compat",
            LimitedQuirks | FullQuirks => ~"BackCompat"
        }
    }

    pub fn set_quirks_mode(&mut self, mode: QuirksMode) {
        self.extra.quirks_mode = mode;
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
                            .map(|node| DocumentTypeCast::to(&node))
    }

    // http://dom.spec.whatwg.org/#dom-document-documentelement
    pub fn GetDocumentElement(&self) -> Option<JS<Element>> {
        self.node.child_elements().next()
    }

    // http://dom.spec.whatwg.org/#dom-document-getelementsbytagname
    pub fn GetElementsByTagName(&self, tag: DOMString) -> JS<HTMLCollection> {
        self.createHTMLCollection(|elem| elem.tag_name == tag)
    }

    // http://dom.spec.whatwg.org/#dom-nonelementparentnode-getelementbyid
    pub fn GetElementById(&self, id: DOMString) -> Option<JS<Element>> {
        // TODO: "in tree order, within the context object's tree"
        // http://dom.spec.whatwg.org/#dom-document-getelementbyid.
        match self.idmap.find_equiv(&id) {
            None => None,
            Some(node) => Some(node.clone()),
        }
    }

    // http://dom.spec.whatwg.org/#dom-document-createelement
    pub fn CreateElement(&self, abstract_self: &JS<Document>, local_name: DOMString)
                         -> Fallible<JS<Element>> {
        if xml_name_type(local_name) == InvalidXMLName {
            debug!("Not a valid element name");
            return Err(InvalidCharacter);
        }
        let local_name = local_name.to_ascii_lower();
        Ok(build_element_from_tag(local_name, abstract_self))
    }

    // http://dom.spec.whatwg.org/#dom-document-createdocumentfragment
    pub fn CreateDocumentFragment(&self, abstract_self: &JS<Document>) -> JS<DocumentFragment> {
        DocumentFragment::new(abstract_self)
    }

    // http://dom.spec.whatwg.org/#dom-document-createtextnode
    pub fn CreateTextNode(&self, abstract_self: &JS<Document>, data: DOMString)
                          -> JS<Text> {
        Text::new(data, abstract_self)
    }

    // http://dom.spec.whatwg.org/#dom-document-createcomment
    pub fn CreateComment(&self, abstract_self: &JS<Document>, data: DOMString) -> JS<Comment> {
        Comment::new(data, abstract_self)
    }

    // http://dom.spec.whatwg.org/#dom-document-createprocessinginstruction
    pub fn CreateProcessingInstruction(&self, abstract_self: &JS<Document>, target: DOMString,
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

    // http://dom.spec.whatwg.org/#dom-document-createevent
    pub fn CreateEvent(&self, interface: DOMString) -> Fallible<JS<Event>> {
        match interface.as_slice() {
            "UIEvents" => Ok(EventCast::from(&UIEvent::new(&self.window))),
            "MouseEvents" => Ok(EventCast::from(&MouseEvent::new(&self.window))),
            "HTMLEvents" => Ok(Event::new(&self.window)),
            _ => Err(NotSupported)
        }
    }

    // http://www.whatwg.org/specs/web-apps/current-work/#document.title
    pub fn Title(&self, _: &JS<Document>) -> DOMString {
        let mut title = ~"";
        self.GetDocumentElement().map(|root| {
            let root: JS<Node> = NodeCast::from(&root);
            root.traverse_preorder()
                .find(|node| node.type_id() == ElementNodeTypeId(HTMLTitleElementTypeId))
                .map(|title_elem| {
                    for child in title_elem.children() {
                        if child.is_text() {
                            let text: JS<Text> = TextCast::to(&child);
                            title.push_str(text.get().characterdata.data.as_slice());
                        }
                    }
                });
        });
        let v: ~[&str] = title.words().collect();
        title = v.connect(" ");
        title = title.trim().to_owned();
        title
    }

    // http://www.whatwg.org/specs/web-apps/current-work/#document.title
    pub fn SetTitle(&self, abstract_self: &JS<Document>, title: DOMString) -> ErrorResult {
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
                        for mut title_child in title_node.children() {
                            title_node.RemoveChild(&mut title_child);
                        }
                        let new_text = self.CreateTextNode(abstract_self, title.clone());
                        title_node.AppendChild(&mut NodeCast::from(&new_text));
                    },
                    None => {
                        let mut new_title: JS<Node> =
                            NodeCast::from(&HTMLTitleElement::new(~"title", abstract_self));
                        let new_text = self.CreateTextNode(abstract_self, title.clone());
                        new_title.AppendChild(&mut NodeCast::from(&new_text));
                        head.AppendChild(&mut new_title);
                    },
                }
            });
        });
        Ok(())
    }

    fn get_html_element(&self) -> Option<JS<HTMLHtmlElement>> {
        self.GetDocumentElement().filtered(|root| {
            root.get().node.type_id == ElementNodeTypeId(HTMLHtmlElementTypeId)
        }).map(|elem| HTMLHtmlElementCast::to(&elem))
    }

    // http://www.whatwg.org/specs/web-apps/current-work/#dom-document-head
    pub fn GetHead(&self) -> Option<JS<HTMLHeadElement>> {
        self.get_html_element().and_then(|root| {
            let node: JS<Node> = NodeCast::from(&root);
            node.children().find(|child| {
                child.type_id() == ElementNodeTypeId(HTMLHeadElementTypeId)
            }).map(|node| HTMLHeadElementCast::to(&node))
        })
    }

    // http://www.whatwg.org/specs/web-apps/current-work/#dom-document-body
    pub fn GetBody(&self, _: &JS<Document>) -> Option<JS<HTMLElement>> {
        self.get_html_element().and_then(|root| {
            let node: JS<Node> = NodeCast::from(&root);
            node.children().find(|child| {
                match child.type_id() {
                    ElementNodeTypeId(HTMLBodyElementTypeId) |
                    ElementNodeTypeId(HTMLFrameSetElementTypeId) => true,
                    _ => false
                }
            }).map(|node| HTMLElementCast::to(&node))
        })
    }

    // http://www.whatwg.org/specs/web-apps/current-work/#dom-document-body
    pub fn SetBody(&self, abstract_self: &JS<Document>, new_body: Option<JS<HTMLElement>>) -> ErrorResult {
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
        if old_body == new_body {
            return Ok(());
        }

        // Step 3.
        match self.get_html_element() {
            // Step 4.
            None => return Err(HierarchyRequest),
            Some(root) => {
                let mut new_body: JS<Node> = NodeCast::from(&new_body.unwrap());
                let mut root: JS<Node> = NodeCast::from(&root);
                match old_body {
                    Some(child) => {
                        let mut child: JS<Node> = NodeCast::from(&child);
                        root.ReplaceChild(&mut new_body, &mut child)
                    }
                    None => root.AppendChild(&mut new_body)
                };
            }
        }
        Ok(())
    }

    // http://www.whatwg.org/specs/web-apps/current-work/#dom-document-getelementsbyname
    pub fn GetElementsByName(&self, name: DOMString) -> JS<NodeList> {
        self.createNodeList(|elem| {
            elem.get_attribute(Null, "name").map_default(false, |attr| {
                attr.get().value_ref() == name
            })
        })
    }

    pub fn Images(&self) -> JS<HTMLCollection> {
        self.createHTMLCollection(|elem| "img" == elem.tag_name)
    }

    pub fn Embeds(&self) -> JS<HTMLCollection> {
        self.createHTMLCollection(|elem| "embed" == elem.tag_name)
    }

    pub fn Plugins(&self) -> JS<HTMLCollection> {
        self.Embeds()
    }

    pub fn Links(&self) -> JS<HTMLCollection> {
        self.createHTMLCollection(|elem| {
            ("a" == elem.tag_name || "area" == elem.tag_name) &&
            elem.get_attribute(Null, "href").is_some()
        })
    }

    pub fn Forms(&self) -> JS<HTMLCollection> {
        self.createHTMLCollection(|elem| "form" == elem.tag_name)
    }

    pub fn Scripts(&self) -> JS<HTMLCollection> {
        self.createHTMLCollection(|elem| "script" == elem.tag_name)
    }

    pub fn Anchors(&self) -> JS<HTMLCollection> {
        self.createHTMLCollection(|elem| {
            "a" == elem.tag_name && elem.get_attribute(Null, "name").is_some()
        })
    }

    pub fn Applets(&self) -> JS<HTMLCollection> {
        // FIXME: This should be return OBJECT elements containing applets.
        self.createHTMLCollection(|elem| "applet" == elem.tag_name)
    }

    pub fn create_collection(&self, callback: |elem: &Element| -> bool) -> ~[JS<Element>] {
        let mut elements = ~[];
        match self.GetDocumentElement() {
            None => {},
            Some(root) => {
                let root: JS<Node> = NodeCast::from(&root);
                for child in root.traverse_preorder() {
                    if child.is_element() {
                        let elem: JS<Element> = ElementCast::to(&child);
                        if callback(elem.get()) {
                            elements.push(elem);
                        }
                    }
                }
            }
        }
        elements
    }

    pub fn createHTMLCollection(&self, callback: |elem: &Element| -> bool) -> JS<HTMLCollection> {
        HTMLCollection::new(&self.window, self.create_collection(callback))
    }

    pub fn createNodeList(&self, callback: |elem: &Element| -> bool) -> JS<NodeList> {
        let elements = self.create_collection(callback);
        let nodes = elements.map(|element| {
            let node: JS<Node> = NodeCast::from(element);
            node
        });
        NodeList::new_simple_list(&self.window, nodes)
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
                                    id: DOMString) {
        self.idmap.remove(&id);
    }

    /// Associate an element present in this document with the provided id.
    pub fn register_named_element(&mut self,
                                  element: &JS<Element>,
                                  id: DOMString) {
        assert!({
            let node: JS<Node> = NodeCast::from(element);
            node.is_in_doc()
        });

        // TODO: support the case if multiple elements
        // which haves same id are in the same document.
        self.idmap.mangle(id, element,
                          |_, new_element: &JS<Element>| -> JS<Element> {
                              new_element.clone()
                          },
                          |_, old_element: &mut JS<Element>, new_element: &JS<Element>| {
                              *old_element = new_element.clone();
                          });
    }
}
