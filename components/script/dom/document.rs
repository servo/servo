/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use document_loader::{DocumentLoader, LoadType};
use dom::attr::{Attr, AttrValue};
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::DocumentBinding;
use dom::bindings::codegen::Bindings::DocumentBinding::{DocumentMethods, DocumentReadyState};
use dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use dom::bindings::codegen::Bindings::EventHandlerBinding::EventHandlerNonNull;
use dom::bindings::codegen::Bindings::EventTargetBinding::EventTargetMethods;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::codegen::Bindings::NodeFilterBinding::NodeFilter;
use dom::bindings::codegen::Bindings::PerformanceBinding::PerformanceMethods;
use dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use dom::bindings::codegen::InheritTypes::ElementDerived;
use dom::bindings::codegen::InheritTypes::HTMLBaseElementCast;
use dom::bindings::codegen::InheritTypes::{DocumentDerived, EventCast, HTMLBodyElementCast};
use dom::bindings::codegen::InheritTypes::{DocumentTypeCast, HTMLHtmlElementCast, NodeCast};
use dom::bindings::codegen::InheritTypes::{EventTargetCast, HTMLAnchorElementCast};
use dom::bindings::codegen::InheritTypes::{HTMLAnchorElementDerived, HTMLAppletElementDerived};
use dom::bindings::codegen::InheritTypes::{HTMLAreaElementDerived, HTMLEmbedElementDerived};
use dom::bindings::codegen::InheritTypes::{HTMLElementCast, HTMLHeadElementCast, ElementCast, HTMLIFrameElementCast};
use dom::bindings::codegen::InheritTypes::{HTMLFormElementDerived, HTMLImageElementDerived};
use dom::bindings::codegen::InheritTypes::{HTMLScriptElementDerived, HTMLTitleElementDerived};
use dom::bindings::codegen::UnionTypes::NodeOrString;
use dom::bindings::error::Error::HierarchyRequest;
use dom::bindings::error::Error::{NotSupported, InvalidCharacter, Security};
use dom::bindings::error::{ErrorResult, Fallible};
use dom::bindings::global::GlobalRef;
use dom::bindings::js::RootedReference;
use dom::bindings::js::{JS, Root, LayoutJS, MutNullableHeap};
use dom::bindings::refcounted::Trusted;
use dom::bindings::trace::RootedVec;
use dom::bindings::utils::XMLName::InvalidXMLName;
use dom::bindings::utils::{reflect_dom_object, Reflectable};
use dom::bindings::utils::{xml_name_type, validate_and_extract};
use dom::comment::Comment;
use dom::customevent::CustomEvent;
use dom::documentfragment::DocumentFragment;
use dom::documenttype::DocumentType;
use dom::domimplementation::DOMImplementation;
use dom::element::{Element, ElementCreator, ElementTypeId};
use dom::event::{Event, EventBubbles, EventCancelable};
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::htmlanchorelement::HTMLAnchorElement;
use dom::htmlbaseelement::HTMLBaseElement;
use dom::htmlcollection::{HTMLCollection, CollectionFilter};
use dom::htmlelement::{HTMLElement, HTMLElementTypeId};
use dom::htmlheadelement::HTMLHeadElement;
use dom::htmlhtmlelement::HTMLHtmlElement;
use dom::htmliframeelement::{self, HTMLIFrameElement};
use dom::htmlscriptelement::HTMLScriptElement;
use dom::keyboardevent::KeyboardEvent;
use dom::location::Location;
use dom::messageevent::MessageEvent;
use dom::mouseevent::MouseEvent;
use dom::node::{self, Node, NodeTypeId, CloneChildrenFlag, NodeDamage, window_from_node};
use dom::nodeiterator::NodeIterator;
use dom::nodelist::NodeList;
use dom::processinginstruction::ProcessingInstruction;
use dom::range::Range;
use dom::servohtmlparser::ServoHTMLParser;
use dom::text::Text;
use dom::treewalker::TreeWalker;
use dom::uievent::UIEvent;
use dom::window::{Window, ReflowReason};

use layout_interface::{HitTestResponse, MouseOverResponse};
use layout_interface::{ReflowGoal, ReflowQueryType};
use msg::compositor_msg::ScriptToCompositorMsg;
use msg::constellation_msg::AnimationState;
use msg::constellation_msg::Msg as ConstellationMsg;
use msg::constellation_msg::{ConstellationChan, FocusType, Key, KeyState, KeyModifiers, MozBrowserEvent, SubpageId};
use msg::constellation_msg::{SUPER, ALT, SHIFT, CONTROL};
use net_traits::ControlMsg::{SetCookiesForUrl, GetCookiesForUrl};
use net_traits::CookieSource::NonHTTP;
use net_traits::{Metadata, PendingAsyncLoad, AsyncResponseTarget};
use script_task::Runnable;
use script_traits::{MouseButton, UntrustedNodeAddress};
use util::str::{DOMString, split_html_space_chars};

use euclid::point::Point2D;
use html5ever::tree_builder::{QuirksMode, NoQuirks, LimitedQuirks, Quirks};
use ipc_channel::ipc::{self, IpcSender};
use js::jsapi::{JSContext, JSObject, JSRuntime};
use layout_interface::{LayoutChan, Msg};
use string_cache::{Atom, QualName};
use url::Url;

use num::ToPrimitive;
use std::ascii::AsciiExt;
use std::borrow::ToOwned;
use std::boxed::FnBox;
use std::cell::{Cell, Ref, RefMut, RefCell};
use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::default::Default;
use std::iter::FromIterator;
use std::ptr;
use std::rc::Rc;
use std::sync::mpsc::channel;
use time;

#[derive(JSTraceable, PartialEq, HeapSizeOf)]
pub enum IsHTMLDocument {
    HTMLDocument,
    NonHTMLDocument,
}

// https://dom.spec.whatwg.org/#document
#[dom_struct]
pub struct Document {
    node: Node,
    window: JS<Window>,
    idmap: DOMRefCell<HashMap<Atom, Vec<JS<Element>>>>,
    implementation: MutNullableHeap<JS<DOMImplementation>>,
    location: MutNullableHeap<JS<Location>>,
    content_type: DOMString,
    last_modified: Option<DOMString>,
    encoding_name: DOMRefCell<DOMString>,
    is_html_document: bool,
    url: Url,
    quirks_mode: Cell<QuirksMode>,
    images: MutNullableHeap<JS<HTMLCollection>>,
    embeds: MutNullableHeap<JS<HTMLCollection>>,
    links: MutNullableHeap<JS<HTMLCollection>>,
    forms: MutNullableHeap<JS<HTMLCollection>>,
    scripts: MutNullableHeap<JS<HTMLCollection>>,
    anchors: MutNullableHeap<JS<HTMLCollection>>,
    applets: MutNullableHeap<JS<HTMLCollection>>,
    ready_state: Cell<DocumentReadyState>,
    /// The element that has most recently requested focus for itself.
    possibly_focused: MutNullableHeap<JS<Element>>,
    /// The element that currently has the document focus context.
    focused: MutNullableHeap<JS<Element>>,
    /// The script element that is currently executing.
    current_script: MutNullableHeap<JS<HTMLScriptElement>>,
    /// https://html.spec.whatwg.org/multipage/#concept-n-noscript
    /// True if scripting is enabled for all scripts in this document
    scripting_enabled: Cell<bool>,
    /// https://html.spec.whatwg.org/multipage/#animation-frame-callback-identifier
    /// Current identifier of animation frame callback
    animation_frame_ident: Cell<u32>,
    /// https://html.spec.whatwg.org/multipage/#list-of-animation-frame-callbacks
    /// List of animation frame callbacks
    #[ignore_heap_size_of = "closures are hard"]
    animation_frame_list: RefCell<HashMap<u32, Box<FnBox(f64)>>>,
    /// Tracks all outstanding loads related to this document.
    loader: DOMRefCell<DocumentLoader>,
    /// The current active HTML parser, to allow resuming after interruptions.
    current_parser: MutNullableHeap<JS<ServoHTMLParser>>,
    /// When we should kick off a reflow. This happens during parsing.
    reflow_timeout: Cell<Option<u64>>,
    /// The cached first `base` element with an `href` attribute.
    base_element: MutNullableHeap<JS<HTMLBaseElement>>,
    /// This field is set to the document itself for inert documents.
    /// https://html.spec.whatwg.org/multipage/#appropriate-template-contents-owner-document
    appropriate_template_contents_owner_document: MutNullableHeap<JS<Document>>,
}

impl PartialEq for Document {
    fn eq(&self, other: &Document) -> bool {
        self as *const Document == &*other
    }
}

impl DocumentDerived for EventTarget {
    fn is_document(&self) -> bool {
        *self.type_id() == EventTargetTypeId::Node(NodeTypeId::Document)
    }
}

#[derive(JSTraceable, HeapSizeOf)]
struct ImagesFilter;
impl CollectionFilter for ImagesFilter {
    fn filter(&self, elem: &Element, _root: &Node) -> bool {
        elem.is_htmlimageelement()
    }
}

#[derive(JSTraceable, HeapSizeOf)]
struct EmbedsFilter;
impl CollectionFilter for EmbedsFilter {
    fn filter(&self, elem: &Element, _root: &Node) -> bool {
        elem.is_htmlembedelement()
    }
}

#[derive(JSTraceable, HeapSizeOf)]
struct LinksFilter;
impl CollectionFilter for LinksFilter {
    fn filter(&self, elem: &Element, _root: &Node) -> bool {
        (elem.is_htmlanchorelement() || elem.is_htmlareaelement()) &&
            elem.has_attribute(&atom!("href"))
    }
}

#[derive(JSTraceable, HeapSizeOf)]
struct FormsFilter;
impl CollectionFilter for FormsFilter {
    fn filter(&self, elem: &Element, _root: &Node) -> bool {
        elem.is_htmlformelement()
    }
}

#[derive(JSTraceable, HeapSizeOf)]
struct ScriptsFilter;
impl CollectionFilter for ScriptsFilter {
    fn filter(&self, elem: &Element, _root: &Node) -> bool {
        elem.is_htmlscriptelement()
    }
}

#[derive(JSTraceable, HeapSizeOf)]
struct AnchorsFilter;
impl CollectionFilter for AnchorsFilter {
    fn filter(&self, elem: &Element, _root: &Node) -> bool {
        elem.is_htmlanchorelement() && elem.has_attribute(&atom!("href"))
    }
}

#[derive(JSTraceable, HeapSizeOf)]
struct AppletsFilter;
impl CollectionFilter for AppletsFilter {
    fn filter(&self, elem: &Element, _root: &Node) -> bool {
        elem.is_htmlappletelement()
    }
}


impl Document {
    #[inline]
    pub fn loader(&self) -> Ref<DocumentLoader> {
        self.loader.borrow()
    }

    #[inline]
    pub fn mut_loader(&self) -> RefMut<DocumentLoader> {
        self.loader.borrow_mut()
    }

    #[inline]
    pub fn window(&self) -> Root<Window> {
        self.window.root()
    }

    #[inline]
    pub fn encoding_name(&self) -> Ref<DOMString> {
        self.encoding_name.borrow()
    }

    #[inline]
    pub fn is_html_document(&self) -> bool {
        self.is_html_document
    }

    // https://html.spec.whatwg.org/multipage/#fully-active
    pub fn is_fully_active(&self) -> bool {
        let window = self.window.root();
        let window = window.r();
        let browsing_context = window.browsing_context();
        let browsing_context = browsing_context.as_ref().unwrap();
        let active_document = browsing_context.active_document();

        if self != active_document.r() {
            return false;
        }
        // FIXME: It should also check whether the browser context is top-level or not
        true
    }

    // https://dom.spec.whatwg.org/#concept-document-url
    pub fn url(&self) -> Url {
        self.url.clone()
    }

    // https://html.spec.whatwg.org/multipage/#fallback-base-url
    pub fn fallback_base_url(&self) -> Url {
        // Step 1: iframe srcdoc (#4767).
        // Step 2: about:blank with a creator browsing context.
        // Step 3.
        self.url()
    }

    // https://html.spec.whatwg.org/multipage/#document-base-url
    pub fn base_url(&self) -> Url {
        match self.base_element() {
            // Step 1.
            None => self.fallback_base_url(),
            // Step 2.
            Some(base) => base.frozen_base_url(),
        }
    }

    /// Returns the first `base` element in the DOM that has an `href` attribute.
    pub fn base_element(&self) -> Option<Root<HTMLBaseElement>> {
        self.base_element.get().map(Root::from_rooted)
    }

    /// Refresh the cached first base element in the DOM.
    /// https://github.com/w3c/web-platform-tests/issues/2122
    pub fn refresh_base_element(&self) {
        let base = NodeCast::from_ref(self)
            .traverse_preorder()
            .filter_map(HTMLBaseElementCast::to_root)
            .filter(|element| ElementCast::from_ref(&**element).has_attribute(&atom!("href")))
            .next();
        self.base_element.set(base.map(|element| JS::from_ref(&*element)));
    }

    pub fn quirks_mode(&self) -> QuirksMode {
        self.quirks_mode.get()
    }

    pub fn set_quirks_mode(&self, mode: QuirksMode) {
        self.quirks_mode.set(mode);

        if mode == Quirks {
            let window = self.window.root();
            let window = window.r();
            let LayoutChan(ref layout_chan) = window.layout_chan();
            layout_chan.send(Msg::SetQuirksMode).unwrap();
        }
    }

    pub fn set_encoding_name(&self, name: DOMString) {
        *self.encoding_name.borrow_mut() = name;
    }

    pub fn content_changed(&self, node: &Node, damage: NodeDamage) {
        node.dirty(damage);
    }

    pub fn content_and_heritage_changed(&self, node: &Node, damage: NodeDamage) {
        node.force_dirty_ancestors(damage);
        node.dirty(damage);
    }

    /// Reflows and disarms the timer if the reflow timer has expired.
    pub fn reflow_if_reflow_timer_expired(&self) {
        if let Some(reflow_timeout) = self.reflow_timeout.get() {
            if time::precise_time_ns() < reflow_timeout {
                return
            }

            self.reflow_timeout.set(None);
            let window = self.window.root();
            window.r().reflow(ReflowGoal::ForDisplay,
                              ReflowQueryType::NoQuery,
                              ReflowReason::RefreshTick);
        }
    }

    /// Schedules a reflow to be kicked off at the given `timeout` (in `time::precise_time_ns()`
    /// units). This reflow happens even if the event loop is busy. This is used to display initial
    /// page content during parsing.
    pub fn set_reflow_timeout(&self, timeout: u64) {
        if let Some(existing_timeout) = self.reflow_timeout.get() {
            if existing_timeout < timeout {
                return
            }
        }
        self.reflow_timeout.set(Some(timeout))
    }

    /// Disables any pending reflow timeouts.
    pub fn disarm_reflow_timeout(&self) {
        self.reflow_timeout.set(None)
    }

    /// Remove any existing association between the provided id and any elements in this document.
    pub fn unregister_named_element(&self,
                                to_unregister: &Element,
                                id: Atom) {
        debug!("Removing named element from document {:p}: {:p} id={}", self, to_unregister, id);
        let mut idmap = self.idmap.borrow_mut();
        let is_empty = match idmap.get_mut(&id) {
            None => false,
            Some(elements) => {
                let position = elements.iter()
                                       .map(|elem| elem.root())
                                       .position(|element| element.r() == to_unregister)
                                       .expect("This element should be in registered.");
                elements.remove(position);
                elements.is_empty()
            }
        };
        if is_empty {
            idmap.remove(&id);
        }
    }

    /// Associate an element present in this document with the provided id.
    pub fn register_named_element(&self,
                              element: &Element,
                              id: Atom) {
        debug!("Adding named element to document {:p}: {:p} id={}", self, element, id);
        assert!({
            let node = NodeCast::from_ref(element);
            node.is_in_doc()
        });
        assert!(!id.is_empty());

        let mut idmap = self.idmap.borrow_mut();

        let root = self.GetDocumentElement().expect(
            "The element is in the document, so there must be a document element.");

        match idmap.entry(id) {
            Vacant(entry) => {
                entry.insert(vec![JS::from_ref(element)]);
            }
            Occupied(entry) => {
                let elements = entry.into_mut();

                let new_node = NodeCast::from_ref(element);
                let mut head: usize = 0;
                let root = NodeCast::from_ref(root.r());
                for node in root.traverse_preorder() {
                    if let Some(elem) = ElementCast::to_ref(node.r()) {
                        if (*elements)[head].root().r() == elem {
                            head += 1;
                        }
                        if new_node == node.r() || head == elements.len() {
                            break;
                        }
                    }
                }

                elements.insert(head, JS::from_ref(element));
            }
        }
    }

    /// Attempt to find a named element in this page's document.
    /// https://html.spec.whatwg.org/multipage/#the-indicated-part-of-the-document
    pub fn find_fragment_node(&self, fragid: &str) -> Option<Root<Element>> {
        self.GetElementById(fragid.to_owned()).or_else(|| {
            let check_anchor = |&node: &&HTMLAnchorElement| {
                let elem = ElementCast::from_ref(node);
                elem.get_attribute(&ns!(""), &atom!("name")).map_or(false, |attr| {
                    &**attr.r().value() == fragid
                })
            };
            let doc_node = NodeCast::from_ref(self);
            doc_node.traverse_preorder()
                    .filter_map(HTMLAnchorElementCast::to_root)
                    .find(|node| check_anchor(&node.r()))
                    .map(ElementCast::from_root)
        })
    }

    pub fn hit_test(&self, point: &Point2D<f32>) -> Option<UntrustedNodeAddress> {
        let root = self.GetDocumentElement();
        let root = match root.r() {
            Some(root) => root,
            None => return None,
        };
        let root = NodeCast::from_ref(root);
        let win = self.window.root();
        let address = match win.r().layout().hit_test(root.to_trusted_node_address(), *point) {
            Ok(HitTestResponse(node_address)) => Some(node_address),
            Err(()) => {
                debug!("layout query error");
                None
            }
        };
        address
    }

    pub fn get_nodes_under_mouse(&self, point: &Point2D<f32>) -> Vec<UntrustedNodeAddress> {
        let root = self.GetDocumentElement();
        let root = match root.r() {
            Some(root) => root,
            None => return vec!(),
        };
        let root = NodeCast::from_ref(root);
        let win = self.window.root();
        match win.r().layout().mouse_over(root.to_trusted_node_address(), *point) {
            Ok(MouseOverResponse(node_address)) => node_address,
            Err(()) => vec!(),
        }
    }

    // https://html.spec.whatwg.org/multipage/#current-document-readiness
    pub fn set_ready_state(&self, state: DocumentReadyState) {
        self.ready_state.set(state);

        let window = self.window.root();
        let event = Event::new(GlobalRef::Window(window.r()), "readystatechange".to_owned(),
                               EventBubbles::DoesNotBubble,
                               EventCancelable::NotCancelable);
        let target = EventTargetCast::from_ref(self);
        let _ = event.r().fire(target);
    }

    /// Return whether scripting is enabled or not
    pub fn is_scripting_enabled(&self) -> bool {
        self.scripting_enabled.get()
    }

    /// Return the element that currently has focus.
    // https://dvcs.w3.org/hg/dom3events/raw-file/tip/html/DOM3-Events.html#events-focusevent-doc-focus
    pub fn get_focused_element(&self) -> Option<Root<Element>> {
        self.focused.get().map(Root::from_rooted)
    }

    /// Initiate a new round of checking for elements requesting focus. The last element to call
    /// `request_focus` before `commit_focus_transaction` is called will receive focus.
    pub fn begin_focus_transaction(&self) {
        self.possibly_focused.set(None);
    }

    /// Request that the given element receive focus once the current transaction is complete.
    pub fn request_focus(&self, elem: &Element) {
        if elem.is_focusable_area() {
            self.possibly_focused.set(Some(JS::from_ref(elem)))
        }
    }

    /// Reassign the focus context to the element that last requested focus during this
    /// transaction, or none if no elements requested it.
    pub fn commit_focus_transaction(&self, focus_type: FocusType) {
        //TODO: dispatch blur, focus, focusout, and focusin events

        if let Some(ref elem) = self.focused.get().map(|t| t.root()) {
            let node = NodeCast::from_ref(elem.r());
            node.set_focus_state(false);
        }

        self.focused.set(self.possibly_focused.get());

        if let Some(ref elem) = self.focused.get().map(|t| t.root()) {
            let node = NodeCast::from_ref(elem.r());
            node.set_focus_state(true);

            // Update the focus state for all elements in the focus chain.
            // https://html.spec.whatwg.org/multipage/#focus-chain
            if focus_type == FocusType::Element {
                let window = self.window.root();
                let ConstellationChan(ref chan) = window.r().constellation_chan();
                let event = ConstellationMsg::Focus(window.r().pipeline());
                chan.send(event).unwrap();
            }
        }
    }

    /// Handles any updates when the document's title has changed.
    pub fn title_changed(&self) {
        // https://developer.mozilla.org/en-US/docs/Web/Events/mozbrowsertitlechange
        self.trigger_mozbrowser_event(MozBrowserEvent::TitleChange(self.Title()));

        self.send_title_to_compositor();
    }

    /// Sends this document's title to the compositor.
    pub fn send_title_to_compositor(&self) {
        let window = self.window();
        // FIXME(https://github.com/rust-lang/rust/issues/23338)
        let window = window.r();
        let compositor = window.compositor();
        compositor.send(ScriptToCompositorMsg::SetTitle(window.pipeline(), Some(self.Title()))).unwrap();
    }

    pub fn dirty_all_nodes(&self) {
        let root = NodeCast::from_ref(self);
        for node in root.traverse_preorder() {
            node.r().dirty(NodeDamage::OtherNodeDamage)
        }
    }

    pub fn handle_mouse_event(&self, js_runtime: *mut JSRuntime,
                          _button: MouseButton, point: Point2D<f32>,
                          mouse_event_type: MouseEventType) {
        let mouse_event_type_string = match mouse_event_type {
            MouseEventType::Click => "click".to_owned(),
            MouseEventType::MouseUp => "mouseup".to_owned(),
            MouseEventType::MouseDown => "mousedown".to_owned(),
        };
        debug!("{}: at {:?}", mouse_event_type_string, point);
        let node = match self.hit_test(&point) {
            Some(node_address) => {
                debug!("node address is {:?}", node_address.0);
                node::from_untrusted_node_address(js_runtime, node_address)
            },
            None => return,
        };

        let el = match ElementCast::to_ref(node.r()) {
            Some(el) => Root::from_ref(el),
            None => {
                let parent = node.r().GetParentNode();
                match parent.and_then(ElementCast::to_root) {
                    Some(parent) => parent,
                    None => return,
                }
            },
        };

        let node = NodeCast::from_ref(el.r());
        debug!("{} on {:?}", mouse_event_type_string, node.debug_str());
        // Prevent click event if form control element is disabled.
        if let  MouseEventType::Click = mouse_event_type {
            if node.click_event_filter_by_disabled_state() {
                return;
            }

            self.begin_focus_transaction();
        }

        let window = self.window.root();

        // https://dvcs.w3.org/hg/dom3events/raw-file/tip/html/DOM3-Events.html#event-type-click
        let x = point.x as i32;
        let y = point.y as i32;
        let clickCount = 1;
        let event = MouseEvent::new(window.r(),
                                    mouse_event_type_string,
                                    EventBubbles::Bubbles,
                                    EventCancelable::Cancelable,
                                    Some(window.r()),
                                    clickCount,
                                    x, y, x, y,
                                    false, false, false, false,
                                    0i16,
                                    None);
        let event = EventCast::from_ref(event.r());

        // https://dvcs.w3.org/hg/dom3events/raw-file/tip/html/DOM3-Events.html#trusted-events
        event.set_trusted(true);
        // https://html.spec.whatwg.org/multipage/#run-authentic-click-activation-steps
        match mouse_event_type {
            MouseEventType::Click => el.authentic_click_activation(event),
            _ =>  {
                let target = EventTargetCast::from_ref(node);
                event.fire(target);
            },
        }

        if let MouseEventType::Click = mouse_event_type {
            self.commit_focus_transaction(FocusType::Element);
        }
        window.r().reflow(ReflowGoal::ForDisplay, ReflowQueryType::NoQuery, ReflowReason::MouseEvent);
    }

    pub fn fire_mouse_event(&self,
                        point: Point2D<f32>,
                        target: &EventTarget,
                        event_name: String) {
        let x = point.x.to_i32().unwrap_or(0);
        let y = point.y.to_i32().unwrap_or(0);

        let window = self.window.root();

        let mouse_event = MouseEvent::new(window.r(),
                                          event_name,
                                          EventBubbles::Bubbles,
                                          EventCancelable::Cancelable,
                                          Some(window.r()),
                                          0i32,
                                          x, y, x, y,
                                          false, false, false, false,
                                          0i16,
                                          None);
        let event = EventCast::from_ref(mouse_event.r());
        event.fire(target);
    }

    pub fn handle_mouse_move_event(&self,
                               js_runtime: *mut JSRuntime,
                               point: Point2D<f32>,
                               prev_mouse_over_targets: &mut RootedVec<JS<Node>>) {
        // Build a list of elements that are currently under the mouse.
        let mouse_over_addresses = self.get_nodes_under_mouse(&point);
        let mut mouse_over_targets: RootedVec<JS<Node>> = RootedVec::new();
        for node_address in &mouse_over_addresses {
            let node = node::from_untrusted_node_address(js_runtime, *node_address);
            mouse_over_targets.push(node.r().inclusive_ancestors()
                                            .find(|node| node.r().is_element())
                                            .map(|node| JS::from_rooted(&node)).unwrap());
        };

        // Remove hover from any elements in the previous list that are no longer
        // under the mouse.
        for target in prev_mouse_over_targets.iter() {
            if !mouse_over_targets.contains(target) {
                let target = target.root();
                let target_ref = target.r();
                if target_ref.get_hover_state() {
                    target_ref.set_hover_state(false);

                    let target = EventTargetCast::from_ref(target_ref);

                    self.fire_mouse_event(point, &target, "mouseout".to_owned());
                }
            }
        }

        // Set hover state for any elements in the current mouse over list.
        // Check if any of them changed state to determine whether to
        // force a reflow below.
        for target in mouse_over_targets.r() {
            if !target.get_hover_state() {
                target.set_hover_state(true);

                let target = EventTargetCast::from_ref(*target);

                self.fire_mouse_event(point, target, "mouseover".to_owned());

            }
        }

        // Send mousemove event to topmost target
        if mouse_over_addresses.len() > 0 {
            let top_most_node =
                node::from_untrusted_node_address(js_runtime, mouse_over_addresses[0]);

            let target = EventTargetCast::from_ref(top_most_node.r());
            self.fire_mouse_event(point, target, "mousemove".to_owned());
        }

        // Store the current mouse over targets for next frame
        prev_mouse_over_targets.clear();
        prev_mouse_over_targets.append(&mut *mouse_over_targets);

        let window = self.window.root();
        window.r().reflow(ReflowGoal::ForDisplay,
                          ReflowQueryType::NoQuery,
                          ReflowReason::MouseEvent);
    }

    /// The entry point for all key processing for web content
    pub fn dispatch_key_event(&self,
                          key: Key,
                          state: KeyState,
                          modifiers: KeyModifiers,
                          compositor: &mut IpcSender<ScriptToCompositorMsg>) {
        let window = self.window.root();
        let focused = self.get_focused_element();
        let body = self.GetBody();

        let target = match (&focused, &body) {
            (&Some(ref focused), _) => EventTargetCast::from_ref(focused.r()),
            (&None, &Some(ref body)) => EventTargetCast::from_ref(body.r()),
            (&None, &None) => EventTargetCast::from_ref(window.r()),
        };

        let ctrl = modifiers.contains(CONTROL);
        let alt = modifiers.contains(ALT);
        let shift = modifiers.contains(SHIFT);
        let meta = modifiers.contains(SUPER);

        let is_composing = false;
        let is_repeating = state == KeyState::Repeated;
        let ev_type = match state {
            KeyState::Pressed | KeyState::Repeated => "keydown",
            KeyState::Released => "keyup",
        }.to_owned();

        let props = KeyboardEvent::key_properties(key, modifiers);

        let keyevent = KeyboardEvent::new(window.r(), ev_type, true, true,
                                          Some(window.r()), 0, Some(key),
                                          props.key_string.to_owned(), props.code.to_owned(),
                                          props.location, is_repeating, is_composing,
                                          ctrl, alt, shift, meta,
                                          None, props.key_code);
        let event = EventCast::from_ref(keyevent.r());
        event.fire(target);
        let mut prevented = event.DefaultPrevented();

        // https://dvcs.w3.org/hg/dom3events/raw-file/tip/html/DOM3-Events.html#keys-cancelable-keys
        if state != KeyState::Released && props.is_printable() && !prevented {
            // https://dvcs.w3.org/hg/dom3events/raw-file/tip/html/DOM3-Events.html#keypress-event-order
            let event = KeyboardEvent::new(window.r(), "keypress".to_owned(),
                                           true, true, Some(window.r()), 0, Some(key),
                                            props.key_string.to_owned(), props.code.to_owned(),
                                           props.location, is_repeating, is_composing,
                                           ctrl, alt, shift, meta,
                                           props.char_code, 0);
            let ev = EventCast::from_ref(event.r());
            ev.fire(target);
            prevented = ev.DefaultPrevented();
            // TODO: if keypress event is canceled, prevent firing input events
        }

        if !prevented {
            compositor.send(ScriptToCompositorMsg::SendKeyEvent(key, state, modifiers)).unwrap();
        }

        // This behavior is unspecced
        // We are supposed to dispatch synthetic click activation for Space and/or Return,
        // however *when* we do it is up to us
        // I'm dispatching it after the key event so the script has a chance to cancel it
        // https://www.w3.org/Bugs/Public/show_bug.cgi?id=27337
        match key {
            Key::Space if !prevented && state == KeyState::Released => {
                let maybe_elem: Option<&Element> = ElementCast::to_ref(target);
                if let Some(el) = maybe_elem {
                    if let Some(a) = el.as_maybe_activatable() {
                        a.synthetic_click_activation(ctrl, alt, shift, meta);
                    }
                }
            }
            Key::Enter if !prevented && state == KeyState::Released => {
                let maybe_elem: Option<&Element> = ElementCast::to_ref(target);
                if let Some(el) = maybe_elem {
                    if let Some(a) = el.as_maybe_activatable() {
                        a.implicit_submission(ctrl, alt, shift, meta);
                    }
                }
            }
            _ => ()
        }

        window.r().reflow(ReflowGoal::ForDisplay, ReflowQueryType::NoQuery, ReflowReason::KeyEvent);
    }

    pub fn node_from_nodes_and_strings(&self, nodes: Vec<NodeOrString>)
                                   -> Fallible<Root<Node>> {
        if nodes.len() == 1 {
            match nodes.into_iter().next().unwrap() {
                NodeOrString::eNode(node) => Ok(node),
                NodeOrString::eString(string) => {
                    Ok(NodeCast::from_root(self.CreateTextNode(string)))
                },
            }
        } else {
            let fragment = NodeCast::from_root(self.CreateDocumentFragment());
            for node in nodes {
                match node {
                    NodeOrString::eNode(node) => {
                        try!(fragment.r().AppendChild(node.r()));
                    },
                    NodeOrString::eString(string) => {
                        let node = NodeCast::from_root(self.CreateTextNode(string));
                        // No try!() here because appending a text node
                        // should not fail.
                        fragment.r().AppendChild(node.r()).unwrap();
                    }
                }
            }
            Ok(fragment)
        }
    }

    pub fn get_body_attribute(&self, local_name: &Atom) -> DOMString {
        match self.GetBody().and_then(HTMLBodyElementCast::to_root) {
            Some(ref body) => {
                ElementCast::from_ref(body.r()).get_string_attribute(local_name)
            },
            None => "".to_owned()
        }
    }

    pub fn set_body_attribute(&self, local_name: &Atom, value: DOMString) {
        if let Some(ref body) = self.GetBody().and_then(HTMLBodyElementCast::to_root) {
            ElementCast::from_ref(body.r()).set_string_attribute(local_name, value);
        }
    }

    pub fn set_current_script(&self, script: Option<&HTMLScriptElement>) {
        self.current_script.set(script.map(JS::from_ref));
    }

    pub fn trigger_mozbrowser_event(&self, event: MozBrowserEvent) {
        if htmliframeelement::mozbrowser_enabled() {
            let window = self.window.root();

            if let Some((containing_pipeline_id, subpage_id)) = window.r().parent_info() {
                let ConstellationChan(ref chan) = window.r().constellation_chan();
                let event = ConstellationMsg::MozBrowserEvent(containing_pipeline_id,
                                                              subpage_id,
                                                              event);
                chan.send(event).unwrap();
            }
        }
    }

    /// https://html.spec.whatwg.org/multipage/#dom-window-requestanimationframe
    pub fn request_animation_frame(&self, callback: Box<FnBox(f64)>) -> u32 {
        let window = self.window.root();
        let window = window.r();
        let ident = self.animation_frame_ident.get() + 1;

        self.animation_frame_ident.set(ident);
        self.animation_frame_list.borrow_mut().insert(ident, callback);

        // TODO: Should tick animation only when document is visible
        let ConstellationChan(ref chan) = window.constellation_chan();
        let event = ConstellationMsg::ChangeRunningAnimationsState(window.pipeline(),
                                                                   AnimationState::AnimationCallbacksPresent);
        chan.send(event).unwrap();

        ident
    }

    /// https://html.spec.whatwg.org/multipage/#dom-window-cancelanimationframe
    pub fn cancel_animation_frame(&self, ident: u32) {
        self.animation_frame_list.borrow_mut().remove(&ident);
        if self.animation_frame_list.borrow().is_empty() {
            let window = self.window.root();
            let window = window.r();
            let ConstellationChan(ref chan) = window.constellation_chan();
            let event = ConstellationMsg::ChangeRunningAnimationsState(window.pipeline(),
                                                                       AnimationState::NoAnimationCallbacksPresent);
            chan.send(event).unwrap();
        }
    }

    /// https://html.spec.whatwg.org/multipage/#run-the-animation-frame-callbacks
    pub fn run_the_animation_frame_callbacks(&self) {
        let animation_frame_list;
        {
            let mut list = self.animation_frame_list.borrow_mut();
            animation_frame_list = Vec::from_iter(list.drain());

            let window = self.window.root();
            let window = window.r();
            let ConstellationChan(ref chan) = window.constellation_chan();
            let event = ConstellationMsg::ChangeRunningAnimationsState(window.pipeline(),
                                                                       AnimationState::NoAnimationCallbacksPresent);
            chan.send(event).unwrap();
        }
        let window = self.window.root();
        let window = window.r();
        let performance = window.Performance();
        let performance = performance.r();
        let timing = performance.Now();

        for (_, callback) in animation_frame_list {
            callback(*timing);
        }

        window.reflow(ReflowGoal::ForDisplay,
                      ReflowQueryType::NoQuery,
                      ReflowReason::RequestAnimationFrame);
    }

    pub fn prepare_async_load(&self, load: LoadType) -> PendingAsyncLoad {
        let mut loader = self.loader.borrow_mut();
        loader.prepare_async_load(load)
    }

    pub fn load_async(&self, load: LoadType, listener: AsyncResponseTarget) {
        let mut loader = self.loader.borrow_mut();
        loader.load_async(load, listener)
    }

    pub fn load_sync(&self, load: LoadType) -> Result<(Metadata, Vec<u8>), String> {
        let mut loader = self.loader.borrow_mut();
        loader.load_sync(load)
    }

    pub fn finish_load(&self, load: LoadType) {
        let mut loader = self.loader.borrow_mut();
        loader.finish_load(load);
    }

    pub fn notify_constellation_load(&self) {
        let window = self.window.root();
        let pipeline_id = window.r().pipeline();
        let ConstellationChan(ref chan) = window.r().constellation_chan();
        let event = ConstellationMsg::DOMLoad(pipeline_id);
        chan.send(event).unwrap();

    }

    pub fn set_current_parser(&self, script: Option<&ServoHTMLParser>) {
        self.current_parser.set(script.map(JS::from_ref));
    }

    pub fn get_current_parser(&self) -> Option<Root<ServoHTMLParser>> {
        self.current_parser.get().map(Root::from_rooted)
    }

    /// Find an iframe element in the document.
    pub fn find_iframe(&self, subpage_id: SubpageId) -> Option<Root<HTMLIFrameElement>> {
        NodeCast::from_ref(self).traverse_preorder()
            .filter_map(HTMLIFrameElementCast::to_root)
            .find(|node| node.r().subpage_id() == Some(subpage_id))
    }
}

#[derive(HeapSizeOf)]
pub enum MouseEventType {
    Click,
    MouseDown,
    MouseUp,
}


#[derive(PartialEq, HeapSizeOf)]
pub enum DocumentSource {
    FromParser,
    NotFromParser,
}

pub trait LayoutDocumentHelpers {
    #[allow(unsafe_code)]
    unsafe fn is_html_document_for_layout(&self) -> bool;
}

impl LayoutDocumentHelpers for LayoutJS<Document> {
    #[allow(unrooted_must_root)]
    #[inline]
    #[allow(unsafe_code)]
    unsafe fn is_html_document_for_layout(&self) -> bool {
        (*self.unsafe_get()).is_html_document
    }
}

impl Document {
    fn new_inherited(window: &Window,
                     url: Option<Url>,
                     is_html_document: IsHTMLDocument,
                     content_type: Option<DOMString>,
                     last_modified: Option<DOMString>,
                     source: DocumentSource,
                     doc_loader: DocumentLoader) -> Document {
        let url = url.unwrap_or_else(|| Url::parse("about:blank").unwrap());

        let ready_state = if source == DocumentSource::FromParser {
            DocumentReadyState::Loading
        } else {
            DocumentReadyState::Complete
        };

        Document {
            node: Node::new_without_doc(NodeTypeId::Document),
            window: JS::from_ref(window),
            idmap: DOMRefCell::new(HashMap::new()),
            implementation: Default::default(),
            location: Default::default(),
            content_type: match content_type {
                Some(string) => string,
                None => match is_html_document {
                    // https://dom.spec.whatwg.org/#dom-domimplementation-createhtmldocument
                    IsHTMLDocument::HTMLDocument => "text/html".to_owned(),
                    // https://dom.spec.whatwg.org/#concept-document-content-type
                    IsHTMLDocument::NonHTMLDocument => "application/xml".to_owned()
                }
            },
            last_modified: last_modified,
            url: url,
            // https://dom.spec.whatwg.org/#concept-document-quirks
            quirks_mode: Cell::new(NoQuirks),
            // https://dom.spec.whatwg.org/#concept-document-encoding
            encoding_name: DOMRefCell::new("UTF-8".to_owned()),
            is_html_document: is_html_document == IsHTMLDocument::HTMLDocument,
            images: Default::default(),
            embeds: Default::default(),
            links: Default::default(),
            forms: Default::default(),
            scripts: Default::default(),
            anchors: Default::default(),
            applets: Default::default(),
            ready_state: Cell::new(ready_state),
            possibly_focused: Default::default(),
            focused: Default::default(),
            current_script: Default::default(),
            scripting_enabled: Cell::new(true),
            animation_frame_ident: Cell::new(0),
            animation_frame_list: RefCell::new(HashMap::new()),
            loader: DOMRefCell::new(doc_loader),
            current_parser: Default::default(),
            reflow_timeout: Cell::new(None),
            base_element: Default::default(),
            appropriate_template_contents_owner_document: Default::default(),
        }
    }

    // https://dom.spec.whatwg.org/#dom-document
    pub fn Constructor(global: GlobalRef) -> Fallible<Root<Document>> {
        let win = global.as_window();
        let doc = win.Document();
        let doc = doc.r();
        let docloader = DocumentLoader::new(&*doc.loader());
        Ok(Document::new(win, None,
                         IsHTMLDocument::NonHTMLDocument, None,
                         None, DocumentSource::NotFromParser, docloader))
    }

    pub fn new(window: &Window,
               url: Option<Url>,
               doctype: IsHTMLDocument,
               content_type: Option<DOMString>,
               last_modified: Option<DOMString>,
               source: DocumentSource,
               doc_loader: DocumentLoader) -> Root<Document> {
        let document = reflect_dom_object(box Document::new_inherited(window, url, doctype,
                                                                      content_type, last_modified,
                                                                      source, doc_loader),
                                          GlobalRef::Window(window),
                                          DocumentBinding::Wrap);
        {
            let node = NodeCast::from_ref(document.r());
            node.set_owner_doc(document.r());
        }
        document
    }

    fn create_node_list<F: Fn(&Node) -> bool>(&self, callback: F) -> Root<NodeList> {
        let window = self.window.root();
        let doc = self.GetDocumentElement();
        let maybe_node = doc.r().map(NodeCast::from_ref);
        let iter = maybe_node.iter().flat_map(|node| node.traverse_preorder())
                             .filter(|node| callback(node.r()));
        NodeList::new_simple_list(window.r(), iter)
    }

    fn get_html_element(&self) -> Option<Root<HTMLHtmlElement>> {
        self.GetDocumentElement()
            .r()
            .and_then(HTMLHtmlElementCast::to_ref)
            .map(Root::from_ref)
    }

    /// https://html.spec.whatwg.org/multipage/#appropriate-template-contents-owner-document
    pub fn appropriate_template_contents_owner_document(&self) -> Root<Document> {
        self.appropriate_template_contents_owner_document.or_init(|| {
            let doctype = if self.is_html_document {
                IsHTMLDocument::HTMLDocument
            } else {
                IsHTMLDocument::NonHTMLDocument
            };
            let new_doc = Document::new(
                &*self.window(), None, doctype, None, None,
                DocumentSource::NotFromParser, DocumentLoader::new(&self.loader()));
            new_doc.appropriate_template_contents_owner_document.set(
                Some(JS::from_ref(&*new_doc)));
            new_doc
        })
    }
}


impl Node {
    fn click_event_filter_by_disabled_state(&self) -> bool {
        match self.type_id() {
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLButtonElement)) |
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLInputElement)) |
            // NodeTypeId::Element(ElementTypeId::HTMLKeygenElement) |
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLOptionElement)) |
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLSelectElement)) |
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTextAreaElement))
                if self.get_disabled_state() => true,
            _ => false
        }
    }
}

impl DocumentMethods for Document {
    // https://dom.spec.whatwg.org/#dom-document-implementation
    fn Implementation(&self) -> Root<DOMImplementation> {
        self.implementation.or_init(|| DOMImplementation::new(self))
    }

    // https://dom.spec.whatwg.org/#dom-document-url
    fn URL(&self) -> DOMString {
        self.url().serialize()
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-activeelement
    fn GetActiveElement(&self) -> Option<Root<Element>> {
        // TODO: Step 2.

        match self.get_focused_element() {
            Some(element) => Some(element),     // Step 3. and 4.
            None => match self.GetBody() {      // Step 5.
                Some(body) => Some(ElementCast::from_root(body)),
                None => self.GetDocumentElement(),
            }
        }
    }

    // https://html.spec.whatwg.org/#dom-document-hasfocus
    fn HasFocus(&self) -> bool {
        let target = self;                                                        // Step 1.
        let window = self.window.root();
        let window = window.r();
        let browsing_context = window.browsing_context();
        let browsing_context = browsing_context.as_ref();

        match browsing_context {
            Some(browsing_context) => {
                let condidate = browsing_context.active_document();                        // Step 2.
                if condidate.node.get_unique_id() == target.node.get_unique_id() {           // Step 3.
                    true
                } else {
                    false //TODO  Step 4.
                }
            }
            None => false
        }
    }

    // https://dom.spec.whatwg.org/#dom-document-documenturi
    fn DocumentURI(&self) -> DOMString {
        self.URL()
    }

    // https://dom.spec.whatwg.org/#dom-document-compatmode
    fn CompatMode(&self) -> DOMString {
        match self.quirks_mode.get() {
            LimitedQuirks | NoQuirks => "CSS1Compat".to_owned(),
            Quirks => "BackCompat".to_owned()
        }
    }

    // https://dom.spec.whatwg.org/#dom-document-characterset
    fn CharacterSet(&self) -> DOMString {
        self.encoding_name.borrow().clone()
    }

    // https://dom.spec.whatwg.org/#dom-document-inputencoding
    fn InputEncoding(&self) -> DOMString {
        self.encoding_name.borrow().clone()
    }

    // https://dom.spec.whatwg.org/#dom-document-content_type
    fn ContentType(&self) -> DOMString {
        self.content_type.clone()
    }

    // https://dom.spec.whatwg.org/#dom-document-doctype
    fn GetDoctype(&self) -> Option<Root<DocumentType>> {
        let node = NodeCast::from_ref(self);
        node.children()
            .filter_map(|c| DocumentTypeCast::to_ref(c.r()).map(Root::from_ref))
            .next()
    }

    // https://dom.spec.whatwg.org/#dom-document-documentelement
    fn GetDocumentElement(&self) -> Option<Root<Element>> {
        let node = NodeCast::from_ref(self);
        node.child_elements().next()
    }

    // https://dom.spec.whatwg.org/#dom-document-getelementsbytagname
    fn GetElementsByTagName(&self, tag_name: DOMString) -> Root<HTMLCollection> {
        let window = self.window.root();
        HTMLCollection::by_tag_name(window.r(), NodeCast::from_ref(self), tag_name)
    }

    // https://dom.spec.whatwg.org/#dom-document-getelementsbytagnamens
    fn GetElementsByTagNameNS(&self, maybe_ns: Option<DOMString>, tag_name: DOMString)
                              -> Root<HTMLCollection> {
        let window = self.window.root();
        HTMLCollection::by_tag_name_ns(window.r(), NodeCast::from_ref(self), tag_name, maybe_ns)
    }

    // https://dom.spec.whatwg.org/#dom-document-getelementsbyclassname
    fn GetElementsByClassName(&self, classes: DOMString) -> Root<HTMLCollection> {
        let window = self.window.root();

        HTMLCollection::by_class_name(window.r(), NodeCast::from_ref(self), classes)
    }

    // https://dom.spec.whatwg.org/#dom-nonelementparentnode-getelementbyid
    fn GetElementById(&self, id: DOMString) -> Option<Root<Element>> {
        let id = Atom::from_slice(&id);
        self.idmap.borrow().get(&id).map(|ref elements| (*elements)[0].root())
    }

    // https://dom.spec.whatwg.org/#dom-document-createelement
    fn CreateElement(&self, mut local_name: DOMString) -> Fallible<Root<Element>> {
        if xml_name_type(&local_name) == InvalidXMLName {
            debug!("Not a valid element name");
            return Err(InvalidCharacter);
        }
        if self.is_html_document {
            local_name.make_ascii_lowercase();
        }
        let name = QualName::new(ns!(HTML), Atom::from_slice(&local_name));
        Ok(Element::create(name, None, self, ElementCreator::ScriptCreated))
    }

    // https://dom.spec.whatwg.org/#dom-document-createelementns
    fn CreateElementNS(&self,
                       namespace: Option<DOMString>,
                       qualified_name: DOMString) -> Fallible<Root<Element>> {
        let (namespace, prefix, local_name) =
            try!(validate_and_extract(namespace, &qualified_name));
        let name = QualName::new(namespace, local_name);
        Ok(Element::create(name, prefix, self, ElementCreator::ScriptCreated))
    }

    // https://dom.spec.whatwg.org/#dom-document-createattribute
    fn CreateAttribute(&self, local_name: DOMString) -> Fallible<Root<Attr>> {
        if xml_name_type(&local_name) == InvalidXMLName {
            debug!("Not a valid element name");
            return Err(InvalidCharacter);
        }

        let window = self.window.root();
        let name = Atom::from_slice(&local_name);
        // repetition used because string_cache::atom::Atom is non-copyable
        let l_name = Atom::from_slice(&local_name);
        let value = AttrValue::String("".to_owned());

        Ok(Attr::new(window.r(), name, value, l_name, ns!(""), None, None))
    }

    // https://dom.spec.whatwg.org/#dom-document-createattributens
    fn CreateAttributeNS(&self, namespace: Option<DOMString>, qualified_name: DOMString)
                         -> Fallible<Root<Attr>> {
        let (namespace, prefix, local_name) =
            try!(validate_and_extract(namespace, &qualified_name));
        let window = self.window.root();
        let value = AttrValue::String("".to_owned());
        let qualified_name = Atom::from_slice(&qualified_name);
        Ok(Attr::new(window.r(), local_name, value, qualified_name,
                     namespace, prefix, None))
    }

    // https://dom.spec.whatwg.org/#dom-document-createdocumentfragment
    fn CreateDocumentFragment(&self) -> Root<DocumentFragment> {
        DocumentFragment::new(self)
    }

    // https://dom.spec.whatwg.org/#dom-document-createtextnode
    fn CreateTextNode(&self, data: DOMString) -> Root<Text> {
        Text::new(data, self)
    }

    // https://dom.spec.whatwg.org/#dom-document-createcomment
    fn CreateComment(&self, data: DOMString) -> Root<Comment> {
        Comment::new(data, self)
    }

    // https://dom.spec.whatwg.org/#dom-document-createprocessinginstruction
    fn CreateProcessingInstruction(&self, target: DOMString, data: DOMString) ->
            Fallible<Root<ProcessingInstruction>> {
        // Step 1.
        if xml_name_type(&target) == InvalidXMLName {
            return Err(InvalidCharacter);
        }

        // Step 2.
        if data.contains("?>") {
            return Err(InvalidCharacter);
        }

        // Step 3.
        Ok(ProcessingInstruction::new(target, data, self))
    }

    // https://dom.spec.whatwg.org/#dom-document-importnode
    fn ImportNode(&self, node: &Node, deep: bool) -> Fallible<Root<Node>> {
        // Step 1.
        if node.is_document() {
            return Err(NotSupported);
        }

        // Step 2.
        let clone_children = match deep {
            true => CloneChildrenFlag::CloneChildren,
            false => CloneChildrenFlag::DoNotCloneChildren
        };

        Ok(Node::clone(node, Some(self), clone_children))
    }

    // https://dom.spec.whatwg.org/#dom-document-adoptnode
    fn AdoptNode(&self, node: &Node) -> Fallible<Root<Node>> {
        // Step 1.
        if node.is_document() {
            return Err(NotSupported);
        }

        // Step 2.
        Node::adopt(node, self);

        // Step 3.
        Ok(Root::from_ref(node))
    }

    // https://dom.spec.whatwg.org/#dom-document-createevent
    fn CreateEvent(&self, mut interface: DOMString) -> Fallible<Root<Event>> {
        let window = self.window.root();

        interface.make_ascii_lowercase();
        match &*interface {
            "uievents" | "uievent" => Ok(EventCast::from_root(
                UIEvent::new_uninitialized(window.r()))),
            "mouseevents" | "mouseevent" => Ok(EventCast::from_root(
                MouseEvent::new_uninitialized(window.r()))),
            "customevent" => Ok(EventCast::from_root(
                CustomEvent::new_uninitialized(GlobalRef::Window(window.r())))),
            "htmlevents" | "events" | "event" => Ok(Event::new_uninitialized(
                GlobalRef::Window(window.r()))),
            "keyboardevent" | "keyevents" => Ok(EventCast::from_root(
                KeyboardEvent::new_uninitialized(window.r()))),
            "messageevent" => Ok(EventCast::from_root(
                MessageEvent::new_uninitialized(GlobalRef::Window(window.r())))),
            _ => Err(NotSupported)
        }
    }

    // https://html.spec.whatwg.org/#dom-document-lastmodified
    fn LastModified(&self) -> DOMString {
        match self.last_modified {
            Some(ref t) => t.clone(),
            None => time::now().strftime("%m/%d/%Y %H:%M:%S").unwrap().to_string(),
        }
    }

    // https://dom.spec.whatwg.org/#dom-document-createrange
    fn CreateRange(&self) -> Root<Range> {
        Range::new_with_doc(self)
    }

    // https://dom.spec.whatwg.org/#dom-document-createnodeiteratorroot-whattoshow-filter
    fn CreateNodeIterator(&self, root: &Node, whatToShow: u32, filter: Option<Rc<NodeFilter>>)
                        -> Root<NodeIterator> {
        NodeIterator::new(self, root, whatToShow, filter)
    }

    // https://dom.spec.whatwg.org/#dom-document-createtreewalker
    fn CreateTreeWalker(&self, root: &Node, whatToShow: u32, filter: Option<Rc<NodeFilter>>)
                        -> Root<TreeWalker> {
        TreeWalker::new(self, root, whatToShow, filter)
    }

    // https://html.spec.whatwg.org/#document.title
    fn Title(&self) -> DOMString {
        let title = self.GetDocumentElement().and_then(|root| {
            if root.r().namespace() == &ns!(SVG) && root.r().local_name() == &atom!("svg") {
                // Step 1.
                NodeCast::from_ref(root.r()).child_elements().find(|node| {
                    node.r().namespace() == &ns!(SVG) &&
                    node.r().local_name() == &atom!("title")
                }).map(NodeCast::from_root)
            } else {
                // Step 2.
                NodeCast::from_ref(root.r())
                         .traverse_preorder()
                         .find(|node| node.r().is_htmltitleelement())
            }
        });

        match title {
            None => DOMString::new(),
            Some(ref title) => {
                // Steps 3-4.
                let value = Node::collect_text_contents(title.r().children());
                split_html_space_chars(&value).collect::<Vec<_>>().join(" ")
            },
        }
    }

    // https://html.spec.whatwg.org/#document.title
    fn SetTitle(&self, title: DOMString) {
        let root = match self.GetDocumentElement() {
            Some(root) => root,
            None => return,
        };

        let elem = if root.r().namespace() == &ns!(SVG) &&
                       root.r().local_name() == &atom!("svg") {
            let elem = NodeCast::from_ref(root.r()).child_elements().find(|node| {
                node.r().namespace() == &ns!(SVG) &&
                node.r().local_name() == &atom!("title")
            });
            match elem {
                Some(elem) => NodeCast::from_root(elem),
                None => {
                    let name = QualName::new(ns!(SVG), atom!("title"));
                    let elem = Element::create(name, None, self,
                                               ElementCreator::ScriptCreated);
                    NodeCast::from_ref(root.r())
                             .AppendChild(NodeCast::from_ref(elem.r()))
                             .unwrap()
                }
            }
        } else if root.r().namespace() == &ns!(HTML) {
            let elem = NodeCast::from_ref(root.r())
                                .traverse_preorder()
                                .find(|node| node.r().is_htmltitleelement());
            match elem {
                Some(elem) => elem,
                None => {
                    match self.GetHead() {
                        Some(head) => {
                            let name = QualName::new(ns!(HTML), atom!("title"));
                            let elem = Element::create(name, None, self,
                                                       ElementCreator::ScriptCreated);
                            NodeCast::from_ref(head.r())
                                     .AppendChild(NodeCast::from_ref(elem.r()))
                                     .unwrap()
                        },
                        None => return,
                    }
                }
            }
        } else {
            return
        };

        elem.r().SetTextContent(Some(title));
    }

    // https://html.spec.whatwg.org/#dom-document-head
    fn GetHead(&self) -> Option<Root<HTMLHeadElement>> {
        self.get_html_element().and_then(|root| {
            let node = NodeCast::from_ref(root.r());
            node.children()
                .filter_map(|c| HTMLHeadElementCast::to_ref(c.r()).map(Root::from_ref))
                .next()
        })
    }

    // https://html.spec.whatwg.org/#dom-document-currentscript
    fn GetCurrentScript(&self) -> Option<Root<HTMLScriptElement>> {
        self.current_script.get().map(Root::from_rooted)
    }

    // https://html.spec.whatwg.org/#dom-document-body
    fn GetBody(&self) -> Option<Root<HTMLElement>> {
        self.get_html_element().and_then(|root| {
            let node = NodeCast::from_ref(root.r());
            node.children().find(|child| {
                match child.r().type_id() {
                    NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLBodyElement)) |
                    NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLFrameSetElement)) => true,
                    _ => false
                }
            }).map(|node| {
                Root::from_ref(HTMLElementCast::to_ref(node.r()).unwrap())
            })
        })
    }

    // https://html.spec.whatwg.org/#dom-document-body
    fn SetBody(&self, new_body: Option<&HTMLElement>) -> ErrorResult {
        // Step 1.
        let new_body = match new_body {
            Some(new_body) => new_body,
            None => return Err(HierarchyRequest),
        };

        let node = NodeCast::from_ref(new_body);
        match node.type_id() {
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLBodyElement)) |
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLFrameSetElement)) => {}
            _ => return Err(HierarchyRequest)
        }

        // Step 2.
        let old_body = self.GetBody();
        if old_body.as_ref().map(|body| body.r()) == Some(new_body) {
            return Ok(());
        }

        match (self.get_html_element(), &old_body) {
            // Step 3.
            (Some(ref root), &Some(ref child)) => {
                let root = NodeCast::from_ref(root.r());
                let child = NodeCast::from_ref(child.r());
                let new_body = NodeCast::from_ref(new_body);
                assert!(root.ReplaceChild(new_body, child).is_ok())
            },

            // Step 4.
            (None, _) => return Err(HierarchyRequest),

            // Step 5.
            (Some(ref root), &None) => {
                let root = NodeCast::from_ref(root.r());
                let new_body = NodeCast::from_ref(new_body);
                assert!(root.AppendChild(new_body).is_ok());
            }
        }
        Ok(())
    }

    // https://html.spec.whatwg.org/#dom-document-getelementsbyname
    fn GetElementsByName(&self, name: DOMString) -> Root<NodeList> {
        self.create_node_list(|node| {
            let element = match ElementCast::to_ref(node) {
                Some(element) => element,
                None => return false,
            };
            if element.namespace() != &ns!(HTML) {
                return false;
            }
            element.get_attribute(&ns!(""), &atom!("name")).map_or(false, |attr| {
                &**attr.r().value() == &*name
            })
        })
    }

    // https://html.spec.whatwg.org/#dom-document-images
    fn Images(&self) -> Root<HTMLCollection> {
        self.images.or_init(|| {
            let window = self.window.root();
            let root = NodeCast::from_ref(self);
            let filter = box ImagesFilter;
            HTMLCollection::create(window.r(), root, filter)
        })
    }

    // https://html.spec.whatwg.org/#dom-document-embeds
    fn Embeds(&self) -> Root<HTMLCollection> {
        self.embeds.or_init(|| {
            let window = self.window.root();
            let root = NodeCast::from_ref(self);
            let filter = box EmbedsFilter;
            HTMLCollection::create(window.r(), root, filter)
        })
    }

    // https://html.spec.whatwg.org/#dom-document-plugins
    fn Plugins(&self) -> Root<HTMLCollection> {
        self.Embeds()
    }

    // https://html.spec.whatwg.org/#dom-document-links
    fn Links(&self) -> Root<HTMLCollection> {
        self.links.or_init(|| {
            let window = self.window.root();
            let root = NodeCast::from_ref(self);
            let filter = box LinksFilter;
            HTMLCollection::create(window.r(), root, filter)
        })
    }

    // https://html.spec.whatwg.org/#dom-document-forms
    fn Forms(&self) -> Root<HTMLCollection> {
        self.forms.or_init(|| {
            let window = self.window.root();
            let root = NodeCast::from_ref(self);
            let filter = box FormsFilter;
            HTMLCollection::create(window.r(), root, filter)
        })
    }

    // https://html.spec.whatwg.org/#dom-document-scripts
    fn Scripts(&self) -> Root<HTMLCollection> {
        self.scripts.or_init(|| {
            let window = self.window.root();
            let root = NodeCast::from_ref(self);
            let filter = box ScriptsFilter;
            HTMLCollection::create(window.r(), root, filter)
        })
    }

    // https://html.spec.whatwg.org/#dom-document-anchors
    fn Anchors(&self) -> Root<HTMLCollection> {
        self.anchors.or_init(|| {
            let window = self.window.root();
            let root = NodeCast::from_ref(self);
            let filter = box AnchorsFilter;
            HTMLCollection::create(window.r(), root, filter)
        })
    }

    // https://html.spec.whatwg.org/#dom-document-applets
    fn Applets(&self) -> Root<HTMLCollection> {
        // FIXME: This should be return OBJECT elements containing applets.
        self.applets.or_init(|| {
            let window = self.window.root();
            let root = NodeCast::from_ref(self);
            let filter = box AppletsFilter;
            HTMLCollection::create(window.r(), root, filter)
        })
    }

    // https://html.spec.whatwg.org/#dom-document-location
    fn Location(&self) -> Root<Location> {
        let window = self.window.root();
        let window = window.r();
        self.location.or_init(|| Location::new(window))
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-children
    fn Children(&self) -> Root<HTMLCollection> {
        let window = self.window.root();
        HTMLCollection::children(window.r(), NodeCast::from_ref(self))
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-firstelementchild
    fn GetFirstElementChild(&self) -> Option<Root<Element>> {
        NodeCast::from_ref(self).child_elements().next()
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-lastelementchild
    fn GetLastElementChild(&self) -> Option<Root<Element>> {
        NodeCast::from_ref(self).rev_children().filter_map(ElementCast::to_root).next()
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-childelementcount
    fn ChildElementCount(&self) -> u32 {
        NodeCast::from_ref(self).child_elements().count() as u32
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-prepend
    fn Prepend(&self, nodes: Vec<NodeOrString>) -> ErrorResult {
        NodeCast::from_ref(self).prepend(nodes)
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-append
    fn Append(&self, nodes: Vec<NodeOrString>) -> ErrorResult {
        NodeCast::from_ref(self).append(nodes)
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-queryselector
    fn QuerySelector(&self, selectors: DOMString) -> Fallible<Option<Root<Element>>> {
        let root = NodeCast::from_ref(self);
        root.query_selector(selectors)
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-queryselectorall
    fn QuerySelectorAll(&self, selectors: DOMString) -> Fallible<Root<NodeList>> {
        let root = NodeCast::from_ref(self);
        root.query_selector_all(selectors)
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-readystate
    fn ReadyState(&self) -> DocumentReadyState {
        self.ready_state.get()
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-defaultview
    fn DefaultView(&self) -> Root<Window> {
        self.window.root()
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-cookie
    fn GetCookie(&self) -> Fallible<DOMString> {
        //TODO: return empty string for cookie-averse Document
        let url = self.url();
        if !is_scheme_host_port_tuple(&url) {
            return Err(Security);
        }
        let window = self.window.root();
        let (tx, rx) = ipc::channel().unwrap();
        let _ = window.r().resource_task().send(GetCookiesForUrl(url, tx, NonHTTP));
        let cookies = rx.recv().unwrap();
        Ok(cookies.unwrap_or("".to_owned()))
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-cookie
    fn SetCookie(&self, cookie: DOMString) -> ErrorResult {
        //TODO: ignore for cookie-averse Document
        let url = self.url();
        if !is_scheme_host_port_tuple(&url) {
            return Err(Security);
        }
        let window = self.window.root();
        let _ = window.r().resource_task().send(SetCookiesForUrl(url, cookie, NonHTTP));
        Ok(())
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-bgcolor
    fn BgColor(&self) -> DOMString {
        self.get_body_attribute(&atom!("bgcolor"))
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-bgcolor
    fn SetBgColor(&self, value: DOMString) {
        self.set_body_attribute(&atom!("bgcolor"), value)
    }

    // https://html.spec.whatwg.org/multipage/#dom-tree-accessors:dom-document-nameditem-filter
    fn NamedGetter(&self, _cx: *mut JSContext, name: DOMString, found: &mut bool)
                   -> *mut JSObject {
        #[derive(JSTraceable, HeapSizeOf)]
        struct NamedElementFilter {
            name: Atom,
        }
        impl CollectionFilter for NamedElementFilter {
            fn filter(&self, elem: &Element, _root: &Node) -> bool {
                filter_by_name(&self.name, NodeCast::from_ref(elem))
            }
        }
        // https://html.spec.whatwg.org/#dom-document-nameditem-filter
        fn filter_by_name(name: &Atom, node: &Node) -> bool {
            let html_elem_type = match node.type_id() {
                NodeTypeId::Element(ElementTypeId::HTMLElement(type_)) => type_,
                _ => return false,
            };
            let elem = match ElementCast::to_ref(node) {
                Some(elem) => elem,
                None => return false,
            };
            match html_elem_type {
                HTMLElementTypeId::HTMLAppletElement => {
                    match elem.get_attribute(&ns!(""), &atom!("name")) {
                        Some(ref attr) if attr.r().value().as_atom() == name => true,
                        _ => {
                            match elem.get_attribute(&ns!(""), &atom!("id")) {
                                Some(ref attr) => attr.r().value().as_atom() == name,
                                None => false,
                            }
                        },
                    }
                },
                HTMLElementTypeId::HTMLFormElement => {
                    match elem.get_attribute(&ns!(""), &atom!("name")) {
                        Some(ref attr) => attr.r().value().as_atom() == name,
                        None => false,
                    }
                },
                HTMLElementTypeId::HTMLImageElement => {
                    match elem.get_attribute(&ns!(""), &atom!("name")) {
                        Some(ref attr) => {
                            if attr.r().value().as_atom() == name {
                                true
                            } else {
                                match elem.get_attribute(&ns!(""), &atom!("id")) {
                                    Some(ref attr) => attr.r().value().as_atom() == name,
                                    None => false,
                                }
                            }
                        },
                        None => false,
                    }
                },
                // TODO: Handle <embed>, <iframe> and <object>.
                _ => false,
            }
        }
        let name = Atom::from_slice(&name);
        let root = NodeCast::from_ref(self);
        {
            // Step 1.
            let mut elements = root.traverse_preorder().filter(|node| {
                filter_by_name(&name, node.r())
            }).peekable();
            if let Some(first) = elements.next() {
                if elements.is_empty() {
                    *found = true;
                    // TODO: Step 2.
                    // Step 3.
                    return first.r().reflector().get_jsobject().get()
                }
            } else {
                *found = false;
                return ptr::null_mut();
            }
        }
        // Step 4.
        *found = true;
        let window = self.window();
        let filter = NamedElementFilter { name: name };
        let collection = HTMLCollection::create(window.r(), root, box filter);
        collection.r().reflector().get_jsobject().get()
    }

    // https://html.spec.whatwg.org/multipage/#document
    fn SupportedPropertyNames(&self) -> Vec<DOMString> {
        // FIXME: unimplemented (https://github.com/servo/servo/issues/7273)
        vec![]
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-clear
    fn Clear(&self) {
        // This method intentionally does nothing
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-captureevents
    fn CaptureEvents(&self) {
        // This method intentionally does nothing
    }

    // https://html.spec.whatwg.org/#dom-document-releaseevents
    fn ReleaseEvents(&self) {
        // This method intentionally does nothing
    }

    // https://html.spec.whatwg.org/multipage/#globaleventhandlers
    global_event_handlers!();

    // https://html.spec.whatwg.org/multipage/#handler-onreadystatechange
    event_handler!(readystatechange, GetOnreadystatechange, SetOnreadystatechange);
}

fn is_scheme_host_port_tuple(url: &Url) -> bool {
    url.host().is_some() && url.port_or_default().is_some()
}

#[derive(HeapSizeOf)]
pub enum DocumentProgressTask {
    DOMContentLoaded,
    Load,
}

pub struct DocumentProgressHandler {
    addr: Trusted<Document>,
    task: DocumentProgressTask,
}

impl DocumentProgressHandler {
    pub fn new(addr: Trusted<Document>, task: DocumentProgressTask) -> DocumentProgressHandler {
        DocumentProgressHandler {
            addr: addr,
            task: task,
        }
    }

    fn dispatch_dom_content_loaded(&self) {
        let document = self.addr.root();
        let window = document.r().window();
        let event = Event::new(GlobalRef::Window(window.r()), "DOMContentLoaded".to_owned(),
                               EventBubbles::DoesNotBubble,
                               EventCancelable::NotCancelable);
        let doctarget = EventTargetCast::from_ref(document.r());
        let _ = doctarget.DispatchEvent(event.r());

        window.r().reflow(ReflowGoal::ForDisplay, ReflowQueryType::NoQuery, ReflowReason::DOMContentLoaded);
    }

    fn set_ready_state_complete(&self) {
        let document = self.addr.root();
        document.r().set_ready_state(DocumentReadyState::Complete);
    }

    fn dispatch_load(&self) {
        let document = self.addr.root();
        let window = document.r().window();
        let event = Event::new(GlobalRef::Window(window.r()), "load".to_owned(),
                               EventBubbles::DoesNotBubble,
                               EventCancelable::NotCancelable);
        let wintarget = EventTargetCast::from_ref(window.r());
        let doctarget = EventTargetCast::from_ref(document.r());
        event.r().set_trusted(true);
        let _ = wintarget.dispatch_event_with_target(doctarget, event.r());

        let window_ref = window.r();
        let browsing_context = window_ref.browsing_context();
        let browsing_context = browsing_context.as_ref().unwrap();

        browsing_context.frame_element().map(|frame_element| {
            let frame_window = window_from_node(frame_element.r());
            let event = Event::new(GlobalRef::Window(frame_window.r()), "load".to_owned(),
                                   EventBubbles::DoesNotBubble,
                                   EventCancelable::NotCancelable);
            let target = EventTargetCast::from_ref(frame_element.r());
            event.r().fire(target);
        });

        document.r().notify_constellation_load();

        // https://developer.mozilla.org/en-US/docs/Web/Events/mozbrowserloadend
        document.r().trigger_mozbrowser_event(MozBrowserEvent::LoadEnd);

        window_ref.reflow(ReflowGoal::ForDisplay,
                          ReflowQueryType::NoQuery,
                          ReflowReason::DocumentLoaded);
    }
}

impl Runnable for DocumentProgressHandler {
    fn handler(self: Box<DocumentProgressHandler>) {
        let document = self.addr.root();
        let window = document.r().window();
        if window.r().is_alive() {
            match self.task {
                DocumentProgressTask::DOMContentLoaded => {
                    self.dispatch_dom_content_loaded();
                }
                DocumentProgressTask::Load => {
                    self.set_ready_state_complete();
                    self.dispatch_load();
                }
            }
        }
    }
}
