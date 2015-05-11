/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use document_loader::{DocumentLoader, LoadType};
use dom::attr::{Attr, AttrHelpers, AttrValue};
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
use dom::bindings::codegen::InheritTypes::{DocumentDerived, EventCast, HTMLBodyElementCast};
use dom::bindings::codegen::InheritTypes::{HTMLElementCast, HTMLHeadElementCast, ElementCast};
use dom::bindings::codegen::InheritTypes::{DocumentTypeCast, HTMLHtmlElementCast, NodeCast};
use dom::bindings::codegen::InheritTypes::{EventTargetCast, HTMLAnchorElementCast};
use dom::bindings::codegen::InheritTypes::{HTMLAnchorElementDerived, HTMLAppletElementDerived};
use dom::bindings::codegen::InheritTypes::{HTMLAreaElementDerived, HTMLEmbedElementDerived};
use dom::bindings::codegen::InheritTypes::{HTMLFormElementDerived, HTMLImageElementDerived};
use dom::bindings::codegen::InheritTypes::{HTMLScriptElementDerived, HTMLTitleElementDerived};
use dom::bindings::codegen::UnionTypes::NodeOrString;
use dom::bindings::error::{ErrorResult, Fallible};
use dom::bindings::error::Error::{NotSupported, InvalidCharacter, Security};
use dom::bindings::error::Error::HierarchyRequest;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, JSRef, LayoutJS, MutNullableHeap};
use dom::bindings::js::{OptionalRootable, Rootable, RootedReference};
use dom::bindings::js::Temporary;
use dom::bindings::js::TemporaryPushable;
use dom::bindings::refcounted::Trusted;
use dom::bindings::trace::RootedVec;
use dom::bindings::utils::reflect_dom_object;
use dom::bindings::utils::{xml_name_type, validate_and_extract};
use dom::bindings::utils::XMLName::InvalidXMLName;
use dom::comment::Comment;
use dom::customevent::CustomEvent;
use dom::documentfragment::DocumentFragment;
use dom::documenttype::DocumentType;
use dom::domimplementation::DOMImplementation;
use dom::element::{Element, ElementCreator, ElementHelpers, AttributeHandlers};
use dom::element::{ElementTypeId, ActivationElementHelpers, FocusElementHelpers};
use dom::event::{Event, EventBubbles, EventCancelable, EventHelpers};
use dom::eventtarget::{EventTarget, EventTargetTypeId, EventTargetHelpers};
use dom::htmlanchorelement::HTMLAnchorElement;
use dom::htmlcollection::{HTMLCollection, CollectionFilter};
use dom::htmlelement::{HTMLElement, HTMLElementTypeId};
use dom::htmlheadelement::HTMLHeadElement;
use dom::htmlhtmlelement::HTMLHtmlElement;
use dom::htmlscriptelement::HTMLScriptElement;
use dom::location::Location;
use dom::mouseevent::MouseEvent;
use dom::keyboardevent::KeyboardEvent;
use dom::messageevent::MessageEvent;
use dom::node::{self, Node, NodeHelpers, NodeTypeId, CloneChildrenFlag, NodeDamage, window_from_node};
use dom::nodelist::NodeList;
use dom::text::Text;
use dom::processinginstruction::ProcessingInstruction;
use dom::range::Range;
use dom::treewalker::TreeWalker;
use dom::uievent::UIEvent;
use dom::window::{Window, WindowHelpers, ReflowReason};

use layout_interface::{HitTestResponse, MouseOverResponse};
use msg::compositor_msg::ScriptListener;
use msg::constellation_msg::AnimationState;
use msg::constellation_msg::Msg as ConstellationMsg;
use msg::constellation_msg::{ConstellationChan, FocusType, Key, KeyState, KeyModifiers, MozBrowserEvent};
use msg::constellation_msg::{SUPER, ALT, SHIFT, CONTROL};
use net_traits::CookieSource::NonHTTP;
use net_traits::ControlMsg::{SetCookiesForUrl, GetCookiesForUrl};
use net_traits::{Metadata, LoadResponse, PendingAsyncLoad};
use script_task::Runnable;
use script_traits::{MouseButton, UntrustedNodeAddress};
use util::opts;
use util::str::{DOMString, split_html_space_chars};
use layout_interface::{ReflowGoal, ReflowQueryType};

use geom::point::Point2D;
use html5ever::tree_builder::{QuirksMode, NoQuirks, LimitedQuirks, Quirks};
use layout_interface::{LayoutChan, Msg};
use string_cache::{Atom, QualName};
use url::Url;
use js::jsapi::JSRuntime;

use num::ToPrimitive;
use std::iter::FromIterator;
use std::borrow::ToOwned;
use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::ascii::AsciiExt;
use std::cell::{Cell, Ref, RefMut, RefCell};
use std::default::Default;
use std::sync::mpsc::{Receiver, channel};
use time;

#[derive(PartialEq)]
#[jstraceable]
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
    animation_frame_ident: Cell<i32>,
    /// https://html.spec.whatwg.org/multipage/#list-of-animation-frame-callbacks
    /// List of animation frame callbacks
    animation_frame_list: RefCell<HashMap<i32, Box<Fn(f64)>>>,
    /// Tracks all outstanding loads related to this document.
    loader: DOMRefCell<DocumentLoader>,
}

impl DocumentDerived for EventTarget {
    fn is_document(&self) -> bool {
        *self.type_id() == EventTargetTypeId::Node(NodeTypeId::Document)
    }
}

#[jstraceable]
struct ImagesFilter;
impl CollectionFilter for ImagesFilter {
    fn filter(&self, elem: JSRef<Element>, _root: JSRef<Node>) -> bool {
        elem.is_htmlimageelement()
    }
}

#[jstraceable]
struct EmbedsFilter;
impl CollectionFilter for EmbedsFilter {
    fn filter(&self, elem: JSRef<Element>, _root: JSRef<Node>) -> bool {
        elem.is_htmlembedelement()
    }
}

#[jstraceable]
struct LinksFilter;
impl CollectionFilter for LinksFilter {
    fn filter(&self, elem: JSRef<Element>, _root: JSRef<Node>) -> bool {
        (elem.is_htmlanchorelement() || elem.is_htmlareaelement()) &&
            elem.has_attribute(&atom!("href"))
    }
}

#[jstraceable]
struct FormsFilter;
impl CollectionFilter for FormsFilter {
    fn filter(&self, elem: JSRef<Element>, _root: JSRef<Node>) -> bool {
        elem.is_htmlformelement()
    }
}

#[jstraceable]
struct ScriptsFilter;
impl CollectionFilter for ScriptsFilter {
    fn filter(&self, elem: JSRef<Element>, _root: JSRef<Node>) -> bool {
        elem.is_htmlscriptelement()
    }
}

#[jstraceable]
struct AnchorsFilter;
impl CollectionFilter for AnchorsFilter {
    fn filter(&self, elem: JSRef<Element>, _root: JSRef<Node>) -> bool {
        elem.is_htmlanchorelement() && elem.has_attribute(&atom!("href"))
    }
}

#[jstraceable]
struct AppletsFilter;
impl CollectionFilter for AppletsFilter {
    fn filter(&self, elem: JSRef<Element>, _root: JSRef<Node>) -> bool {
        elem.is_htmlappletelement()
    }
}

pub trait DocumentHelpers<'a> {
    fn loader(&self) -> Ref<DocumentLoader>;
    fn mut_loader(&self) -> RefMut<DocumentLoader>;
    fn window(self) -> Temporary<Window>;
    fn encoding_name(self) -> Ref<'a, DOMString>;
    fn is_html_document(self) -> bool;
    fn is_fully_active(self) -> bool;
    fn url(self) -> Url;
    fn quirks_mode(self) -> QuirksMode;
    fn set_quirks_mode(self, mode: QuirksMode);
    fn set_encoding_name(self, name: DOMString);
    fn content_changed(self, node: JSRef<Node>, damage: NodeDamage);
    fn content_and_heritage_changed(self, node: JSRef<Node>, damage: NodeDamage);
    fn unregister_named_element(self, to_unregister: JSRef<Element>, id: Atom);
    fn register_named_element(self, element: JSRef<Element>, id: Atom);
    fn load_anchor_href(self, href: DOMString);
    fn find_fragment_node(self, fragid: DOMString) -> Option<Temporary<Element>>;
    fn hit_test(self, point: &Point2D<f32>) -> Option<UntrustedNodeAddress>;
    fn get_nodes_under_mouse(self, point: &Point2D<f32>) -> Vec<UntrustedNodeAddress>;
    fn set_ready_state(self, state: DocumentReadyState);
    fn get_focused_element(self) -> Option<Temporary<Element>>;
    fn is_scripting_enabled(self) -> bool;
    fn begin_focus_transaction(self);
    fn request_focus(self, elem: JSRef<Element>);
    fn commit_focus_transaction(self, focus_type: FocusType);
    fn title_changed(self);
    fn send_title_to_compositor(self);
    fn dirty_all_nodes(self);
    fn dispatch_key_event(self, key: Key, state: KeyState,
        modifiers: KeyModifiers, compositor: &mut Box<ScriptListener+'static>);
    fn node_from_nodes_and_strings(self, nodes: Vec<NodeOrString>)
                                   -> Fallible<Temporary<Node>>;
    fn get_body_attribute(self, local_name: &Atom) -> DOMString;
    fn set_body_attribute(self, local_name: &Atom, value: DOMString);

    fn handle_mouse_event(self, js_runtime: *mut JSRuntime,
                          button: MouseButton, point: Point2D<f32>,
                          mouse_event_type: MouseEventType);
    /// Handles a mouse-move event coming from the compositor.
    fn handle_mouse_move_event(self,
                               js_runtime: *mut JSRuntime,
                               point: Point2D<f32>,
                               prev_mouse_over_targets: &mut RootedVec<JS<Node>>);

    fn set_current_script(self, script: Option<JSRef<HTMLScriptElement>>);
    fn trigger_mozbrowser_event(self, event: MozBrowserEvent);
    /// http://w3c.github.io/animation-timing/#dom-windowanimationtiming-requestanimationframe
    fn request_animation_frame(self, callback: Box<Fn(f64, )>) -> i32;
    /// http://w3c.github.io/animation-timing/#dom-windowanimationtiming-cancelanimationframe
    fn cancel_animation_frame(self, ident: i32);
    /// http://w3c.github.io/animation-timing/#dfn-invoke-callbacks-algorithm
    fn invoke_animation_callbacks(self);
    fn prepare_async_load(self, load: LoadType) -> PendingAsyncLoad;
    fn load_async(self, load: LoadType) -> Receiver<LoadResponse>;
    fn load_sync(self, load: LoadType) -> Result<(Metadata, Vec<u8>), String>;
    fn finish_load(self, load: LoadType);
}

impl<'a> DocumentHelpers<'a> for JSRef<'a, Document> {
    #[inline]
    fn loader(&self) -> Ref<DocumentLoader> {
        self.loader.borrow()
    }

    #[inline]
    fn mut_loader(&self) -> RefMut<DocumentLoader> {
        self.loader.borrow_mut()
    }

    #[inline]
    fn window(self) -> Temporary<Window> {
        Temporary::from_rooted(self.window)
    }

    #[inline]
    fn encoding_name(self) -> Ref<'a, DOMString> {
        self.extended_deref().encoding_name.borrow()
    }

    #[inline]
    fn is_html_document(self) -> bool {
        self.is_html_document
    }

    // https://html.spec.whatwg.org/multipage/#fully-active
    fn is_fully_active(self) -> bool {
        let window = self.window.root();
        let window = window.r();
        let browser_context = window.browser_context();
        let browser_context = browser_context.as_ref().unwrap();
        let active_document = browser_context.active_document().root();

        if self != active_document.r() {
            return false;
        }
        // FIXME: It should also check whether the browser context is top-level or not
        true
    }

    // https://dom.spec.whatwg.org/#dom-document-url
    fn url(self) -> Url {
        self.url.clone()
    }

    fn quirks_mode(self) -> QuirksMode {
        self.quirks_mode.get()
    }

    fn set_quirks_mode(self, mode: QuirksMode) {
        self.quirks_mode.set(mode);

        if mode == Quirks {
            let window = self.window.root();
            let window = window.r();
            let LayoutChan(ref layout_chan) = window.layout_chan();
            layout_chan.send(Msg::SetQuirksMode).unwrap();
        }
    }

    fn set_encoding_name(self, name: DOMString) {
        *self.encoding_name.borrow_mut() = name;
    }

    fn content_changed(self, node: JSRef<Node>, damage: NodeDamage) {
        node.dirty(damage);
    }

    fn content_and_heritage_changed(self, node: JSRef<Node>, damage: NodeDamage) {
        debug!("content_and_heritage_changed on {}", node.debug_str());
        node.force_dirty_ancestors(damage);
        node.dirty(damage);
    }

    /// Remove any existing association between the provided id and any elements in this document.
    fn unregister_named_element(self,
                                to_unregister: JSRef<Element>,
                                id: Atom) {
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
    fn register_named_element(self,
                              element: JSRef<Element>,
                              id: Atom) {
        assert!({
            let node: JSRef<Node> = NodeCast::from_ref(element);
            node.is_in_doc()
        });
        assert!(!id.is_empty());

        let mut idmap = self.idmap.borrow_mut();

        let root = self.GetDocumentElement().expect("The element is in the document, so there must be a document element.").root();

        match idmap.entry(id) {
            Vacant(entry) => {
                entry.insert(vec![JS::from_rooted(element)]);
            }
            Occupied(entry) => {
                let elements = entry.into_mut();

                let new_node: JSRef<Node> = NodeCast::from_ref(element);
                let mut head: usize = 0;
                let root: JSRef<Node> = NodeCast::from_ref(root.r());
                for node in root.traverse_preorder() {
                    let node = node.root();
                    if let Some(elem) = ElementCast::to_ref(node.r()) {
                        if (*elements)[head].root().r() == elem {
                            head += 1;
                        }
                        if new_node == node.r() || head == elements.len() {
                            break;
                        }
                    }
                }

                elements.insert_unrooted(head, &element);
            }
        }
    }

    fn load_anchor_href(self, href: DOMString) {
        let window = self.window.root();
        window.r().load_url(href);
    }

    /// Attempt to find a named element in this page's document.
    /// https://html.spec.whatwg.org/multipage/#the-indicated-part-of-the-document
    fn find_fragment_node(self, fragid: DOMString) -> Option<Temporary<Element>> {
        self.GetElementById(fragid.clone()).or_else(|| {
            let check_anchor = |&node: &JSRef<HTMLAnchorElement>| {
                let elem: JSRef<Element> = ElementCast::from_ref(node);
                elem.get_attribute(&ns!(""), &atom!("name")).root().map_or(false, |attr| {
                    // FIXME(https://github.com/rust-lang/rust/issues/23338)
                    let attr = attr.r();
                    let value = attr.value();
                    &**value == &*fragid
                })
            };
            let doc_node: JSRef<Node> = NodeCast::from_ref(self);
            doc_node.traverse_preorder()
                    .filter_map(HTMLAnchorElementCast::to_temporary)
                    .find(|node| check_anchor(&node.root().r()))
                    .map(ElementCast::from_temporary)
        })
    }

    fn hit_test(self, point: &Point2D<f32>) -> Option<UntrustedNodeAddress> {
        let root = self.GetDocumentElement().root();
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

    fn get_nodes_under_mouse(self, point: &Point2D<f32>) -> Vec<UntrustedNodeAddress> {
        let root = self.GetDocumentElement().root();
        let root = match root.r() {
            Some(root) => root,
            None => return vec!(),
        };
        let root: JSRef<Node> = NodeCast::from_ref(root);
        let win = self.window.root();
        match win.r().layout().mouse_over(root.to_trusted_node_address(), *point) {
            Ok(MouseOverResponse(node_address)) => node_address,
            Err(()) => vec!(),
        }
    }

    // https://html.spec.whatwg.org/multipage/#current-document-readiness
    fn set_ready_state(self, state: DocumentReadyState) {
        self.ready_state.set(state);

        let window = self.window.root();
        let event = Event::new(GlobalRef::Window(window.r()), "readystatechange".to_owned(),
                               EventBubbles::DoesNotBubble,
                               EventCancelable::NotCancelable).root();
        let target: JSRef<EventTarget> = EventTargetCast::from_ref(self);
        let _ = event.r().fire(target);
    }

    /// Return whether scripting is enabled or not
    fn is_scripting_enabled(self) -> bool {
        self.scripting_enabled.get()
    }

    /// Return the element that currently has focus.
    // https://dvcs.w3.org/hg/dom3events/raw-file/tip/html/DOM3-Events.html#events-focusevent-doc-focus
    fn get_focused_element(self) -> Option<Temporary<Element>> {
        self.focused.get().map(Temporary::from_rooted)
    }

    /// Initiate a new round of checking for elements requesting focus. The last element to call
    /// `request_focus` before `commit_focus_transaction` is called will receive focus.
    fn begin_focus_transaction(self) {
        self.possibly_focused.set(None);
    }

    /// Request that the given element receive focus once the current transaction is complete.
    fn request_focus(self, elem: JSRef<Element>) {
        if elem.is_focusable_area() {
            self.possibly_focused.set(Some(JS::from_rooted(elem)))
        }
    }

    /// Reassign the focus context to the element that last requested focus during this
    /// transaction, or none if no elements requested it.
    fn commit_focus_transaction(self, focus_type: FocusType) {
        //TODO: dispatch blur, focus, focusout, and focusin events

        if let Some(ref elem) = self.focused.get().root() {
            let node: JSRef<Node> = NodeCast::from_ref(elem.r());
            node.set_focus_state(false);
        }

        self.focused.set(self.possibly_focused.get());

        if let Some(ref elem) = self.focused.get().root() {
            let node: JSRef<Node> = NodeCast::from_ref(elem.r());
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
    fn title_changed(self) {
        // https://developer.mozilla.org/en-US/docs/Web/Events/mozbrowsertitlechange
        self.trigger_mozbrowser_event(MozBrowserEvent::TitleChange(self.Title()));

        self.send_title_to_compositor();
    }

    /// Sends this document's title to the compositor.
    fn send_title_to_compositor(self) {
        let window = self.window().root();
        // FIXME(https://github.com/rust-lang/rust/issues/23338)
        let window = window.r();
        let mut compositor = window.compositor();
        compositor.set_title(window.pipeline(), Some(self.Title()));
    }

    fn dirty_all_nodes(self) {
        let root: JSRef<Node> = NodeCast::from_ref(self);
        for node in root.traverse_preorder() {
            node.root().r().dirty(NodeDamage::OtherNodeDamage)
        }
    }

    fn handle_mouse_event(self, js_runtime: *mut JSRuntime,
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
        }.root();

        let el = match ElementCast::to_ref(node.r()) {
            Some(el) => Temporary::from_rooted(el),
            None => {
                let parent = node.r().GetParentNode();
                match parent.and_then(ElementCast::to_temporary) {
                    Some(parent) => parent,
                    None => return,
                }
            },
        }.root();

        let node: JSRef<Node> = NodeCast::from_ref(el.r());
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
        let event = MouseEvent::new(window.r(),
                                    mouse_event_type_string,
                                    EventBubbles::Bubbles,
                                    EventCancelable::Cancelable,
                                    Some(window.r()),
                                    0i32,
                                    x, y, x, y,
                                    false, false, false, false,
                                    0i16,
                                    None).root();
        let event: JSRef<Event> = EventCast::from_ref(event.r());

        // https://dvcs.w3.org/hg/dom3events/raw-file/tip/html/DOM3-Events.html#trusted-events
        event.set_trusted(true);
        // https://html.spec.whatwg.org/multipage/#run-authentic-click-activation-steps
        match mouse_event_type {
            MouseEventType::Click => el.r().authentic_click_activation(event),
            _ =>  {
                let target: JSRef<EventTarget> = EventTargetCast::from_ref(node);
                event.fire(target);
            },
        }

        if let MouseEventType::Click = mouse_event_type {
            self.commit_focus_transaction(FocusType::Element);
        }
        window.r().reflow(ReflowGoal::ForDisplay, ReflowQueryType::NoQuery, ReflowReason::MouseEvent);
    }

    fn handle_mouse_move_event(self,
                               js_runtime: *mut JSRuntime,
                               point: Point2D<f32>,
                               prev_mouse_over_targets: &mut RootedVec<JS<Node>>) {
        // Build a list of elements that are currently under the mouse.
        let mouse_over_addresses = self.get_nodes_under_mouse(&point);
        let mut mouse_over_targets: RootedVec<JS<Node>> = RootedVec::new();
        for node_address in mouse_over_addresses.iter() {
            let node = node::from_untrusted_node_address(js_runtime, *node_address);
            mouse_over_targets.push(node.root().r().inclusive_ancestors()
                                                      .map(|node| node.root())
                                                      .find(|node| node.r()
                                                      .is_element())
                                                      .map(|node| JS::from_rooted(node.r())).unwrap());
        };

        // Remove hover from any elements in the previous list that are no longer
        // under the mouse.
        for target in prev_mouse_over_targets.iter() {
            if !mouse_over_targets.contains(target) {
                target.root().r().set_hover_state(false);
            }
        }

        // Set hover state for any elements in the current mouse over list.
        // Check if any of them changed state to determine whether to
        // force a reflow below.
        for target in mouse_over_targets.iter() {
            let target = target.root();
            let target_ref = target.r();
            if !target_ref.get_hover_state() {
                target_ref.set_hover_state(true);
            }
        }

        // Send mousemove event to topmost target
        if mouse_over_addresses.len() > 0 {
            let top_most_node =
                node::from_untrusted_node_address(js_runtime, mouse_over_addresses[0]).root();

            let x = point.x.to_i32().unwrap_or(0);
            let y = point.y.to_i32().unwrap_or(0);

            let window = self.window.root();
            let mouse_event = MouseEvent::new(window.r(),
                                              "mousemove".to_owned(),
                                              EventBubbles::Bubbles,
                                              EventCancelable::Cancelable,
                                              Some(window.r()),
                                              0i32,
                                              x, y, x, y,
                                              false, false, false, false,
                                              0i16,
                                              None).root();

            let event: JSRef<Event> = EventCast::from_ref(mouse_event.r());
            let target: JSRef<EventTarget> = EventTargetCast::from_ref(top_most_node.r());
            event.fire(target);
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
    fn dispatch_key_event(self, key: Key,
                          state: KeyState,
                          modifiers: KeyModifiers,
                          compositor: &mut Box<ScriptListener+'static>) {
        let window = self.window.root();
        let focused = self.get_focused_element().root();
        let body = self.GetBody().root();

        let target: JSRef<EventTarget> = match (&focused, &body) {
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
                                          None, props.key_code).root();
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
                                           props.char_code, 0).root();
            let ev = EventCast::from_ref(event.r());
            ev.fire(target);
            prevented = ev.DefaultPrevented();
            // TODO: if keypress event is canceled, prevent firing input events
        }

        if !prevented {
            compositor.send_key_event(key, state, modifiers);
        }

        // This behavior is unspecced
        // We are supposed to dispatch synthetic click activation for Space and/or Return,
        // however *when* we do it is up to us
        // I'm dispatching it after the key event so the script has a chance to cancel it
        // https://www.w3.org/Bugs/Public/show_bug.cgi?id=27337
        match key {
            Key::Space if !prevented && state == KeyState::Released => {
                let maybe_elem: Option<JSRef<Element>> = ElementCast::to_ref(target);
                maybe_elem.map(|el| el.as_maybe_activatable().map(|a| a.synthetic_click_activation(ctrl, alt, shift, meta)));
            }
            Key::Enter if !prevented && state == KeyState::Released => {
                let maybe_elem: Option<JSRef<Element>> = ElementCast::to_ref(target);
                maybe_elem.map(|el| el.as_maybe_activatable().map(|a| a.implicit_submission(ctrl, alt, shift, meta)));
            }
            _ => ()
        }

        window.r().reflow(ReflowGoal::ForDisplay, ReflowQueryType::NoQuery, ReflowReason::KeyEvent);
    }

    fn node_from_nodes_and_strings(self, nodes: Vec<NodeOrString>)
                                   -> Fallible<Temporary<Node>> {
        if nodes.len() == 1 {
            match nodes.into_iter().next().unwrap() {
                NodeOrString::eNode(node) => Ok(Temporary::from_unrooted(node)),
                NodeOrString::eString(string) => {
                    Ok(NodeCast::from_temporary(self.CreateTextNode(string)))
                },
            }
        } else {
            let fragment = NodeCast::from_temporary(self.CreateDocumentFragment()).root();
            for node in nodes.into_iter() {
                match node {
                    NodeOrString::eNode(node) => {
                        try!(fragment.r().AppendChild(node.root().r()));
                    },
                    NodeOrString::eString(string) => {
                        let node = NodeCast::from_temporary(self.CreateTextNode(string)).root();
                        // No try!() here because appending a text node
                        // should not fail.
                        fragment.r().AppendChild(node.r()).unwrap();
                    }
                }
            }
            Ok(Temporary::from_rooted(fragment.r()))
        }
    }

    fn get_body_attribute(self, local_name: &Atom) -> DOMString {
        match self.GetBody().and_then(HTMLBodyElementCast::to_temporary).root() {
            Some(ref body) => {
                ElementCast::from_ref(body.r()).get_string_attribute(local_name)
            },
            None => "".to_owned()
        }
    }

    fn set_body_attribute(self, local_name: &Atom, value: DOMString) {
        if let Some(ref body) = self.GetBody().and_then(HTMLBodyElementCast::to_temporary).root() {
            ElementCast::from_ref(body.r()).set_string_attribute(local_name, value);
        }
    }

    fn set_current_script(self, script: Option<JSRef<HTMLScriptElement>>) {
        self.current_script.set(script.map(JS::from_rooted));
    }

    fn trigger_mozbrowser_event(self, event: MozBrowserEvent) {
        if opts::experimental_enabled() {
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

    /// http://w3c.github.io/animation-timing/#dom-windowanimationtiming-requestanimationframe
    fn request_animation_frame(self, callback: Box<Fn(f64, )>) -> i32 {
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

    /// http://w3c.github.io/animation-timing/#dom-windowanimationtiming-cancelanimationframe
    fn cancel_animation_frame(self, ident: i32) {
        self.animation_frame_list.borrow_mut().remove(&ident);
        if self.animation_frame_list.borrow().len() == 0 {
            let window = self.window.root();
            let window = window.r();
            let ConstellationChan(ref chan) = window.constellation_chan();
            let event = ConstellationMsg::ChangeRunningAnimationsState(window.pipeline(),
                                                                       AnimationState::NoAnimationCallbacksPresent);
            chan.send(event).unwrap();
        }
    }

    /// http://w3c.github.io/animation-timing/#dfn-invoke-callbacks-algorithm
    fn invoke_animation_callbacks(self) {
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
        let performance = window.Performance().root();
        let performance = performance.r();

        for (_, callback) in animation_frame_list {
            callback(*performance.Now());
        }
    }

    fn prepare_async_load(self, load: LoadType) -> PendingAsyncLoad {
        let mut loader = self.loader.borrow_mut();
        loader.prepare_async_load(load)
    }

    fn load_async(self, load: LoadType) -> Receiver<LoadResponse> {
        let mut loader = self.loader.borrow_mut();
        loader.load_async(load)
    }

    fn load_sync(self, load: LoadType) -> Result<(Metadata, Vec<u8>), String> {
        let mut loader = self.loader.borrow_mut();
        loader.load_sync(load)
    }

    fn finish_load(self, load: LoadType) {
        let mut loader = self.loader.borrow_mut();
        loader.finish_load(load);
    }
}

pub enum MouseEventType {
    Click,
    MouseDown,
    MouseUp,
}

#[derive(PartialEq)]
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
    fn new_inherited(window: JSRef<Window>,
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
            window: JS::from_rooted(window),
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
        }
    }

    // https://dom.spec.whatwg.org/#dom-document
    pub fn Constructor(global: GlobalRef) -> Fallible<Temporary<Document>> {
        let win = global.as_window();
        let doc = win.Document().root();
        let doc = doc.r();
        let docloader = DocumentLoader::new(&*doc.loader());
        Ok(Document::new(win, None,
                         IsHTMLDocument::NonHTMLDocument, None,
                         None, DocumentSource::NotFromParser, docloader))
    }

    pub fn new(window: JSRef<Window>,
               url: Option<Url>,
               doctype: IsHTMLDocument,
               content_type: Option<DOMString>,
               last_modified: Option<DOMString>,
               source: DocumentSource,
               doc_loader: DocumentLoader) -> Temporary<Document> {
        let document = reflect_dom_object(box Document::new_inherited(window, url, doctype,
                                                                      content_type, last_modified,
                                                                      source, doc_loader),
                                          GlobalRef::Window(window),
                                          DocumentBinding::Wrap).root();

        let node: JSRef<Node> = NodeCast::from_ref(document.r());
        node.set_owner_doc(document.r());
        Temporary::from_rooted(document.r())
    }
}

trait PrivateDocumentHelpers {
    fn create_node_list<F: Fn(JSRef<Node>) -> bool>(self, callback: F) -> Temporary<NodeList>;
    fn get_html_element(self) -> Option<Temporary<HTMLHtmlElement>>;
}

impl<'a> PrivateDocumentHelpers for JSRef<'a, Document> {
    fn create_node_list<F: Fn(JSRef<Node>) -> bool>(self, callback: F) -> Temporary<NodeList> {
        let window = self.window.root();
        let document_element = self.GetDocumentElement().root();
        let mut nodes = RootedVec::new();
        if let Some(ref root) = document_element {
            for node in NodeCast::from_ref(root.r()).traverse_preorder() {
                let node = node.root();
                if callback(node.r()) {
                    nodes.push(JS::from_rooted(node.r()));
                }
            }
        };
        NodeList::new_simple_list(window.r(), &nodes)
    }

    fn get_html_element(self) -> Option<Temporary<HTMLHtmlElement>> {
        self.GetDocumentElement()
            .root()
            .r()
            .and_then(HTMLHtmlElementCast::to_ref)
            .map(Temporary::from_rooted)
    }
}

trait PrivateClickEventHelpers {
    fn click_event_filter_by_disabled_state(&self) -> bool;
}

impl<'a> PrivateClickEventHelpers for JSRef<'a, Node> {
    fn click_event_filter_by_disabled_state(&self) -> bool {
        match self.type_id() {
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLButtonElement)) |
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLInputElement)) |
            // NodeTypeId::Element(ElementTypeId::HTMLKeygenElement) |
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLOptionElement)) |
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLSelectElement)) |
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTextAreaElement)) if self.get_disabled_state() => true,
            _ => false
        }
    }
}

impl<'a> DocumentMethods for JSRef<'a, Document> {
    // https://dom.spec.whatwg.org/#dom-document-implementation
    fn Implementation(self) -> Temporary<DOMImplementation> {
        self.implementation.or_init(|| DOMImplementation::new(self))
    }

    // https://dom.spec.whatwg.org/#dom-document-url
    fn URL(self) -> DOMString {
        self.url().serialize()
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-activeelement
    fn GetActiveElement(self) -> Option<Temporary<Element>> {
        // TODO: Step 2.

        match self.get_focused_element() {
            Some(element) => Some(element),     // Step 3. and 4.
            None => match self.GetBody() {      // Step 5.
                Some(body) => Some(ElementCast::from_temporary(body)),
                None => self.GetDocumentElement(),
            }
        }
    }

    // https://dom.spec.whatwg.org/#dom-document-documenturi
    fn DocumentURI(self) -> DOMString {
        self.URL()
    }

    // https://dom.spec.whatwg.org/#dom-document-compatmode
    fn CompatMode(self) -> DOMString {
        match self.quirks_mode.get() {
            LimitedQuirks | NoQuirks => "CSS1Compat".to_owned(),
            Quirks => "BackCompat".to_owned()
        }
    }

    // https://dom.spec.whatwg.org/#dom-document-characterset
    fn CharacterSet(self) -> DOMString {
        // FIXME(https://github.com/rust-lang/rust/issues/23338)
        let encoding_name = self.encoding_name.borrow();
        encoding_name.clone()
    }

    // https://dom.spec.whatwg.org/#dom-document-inputencoding
    fn InputEncoding(self) -> DOMString {
        // FIXME(https://github.com/rust-lang/rust/issues/23338)
        let encoding_name = self.encoding_name.borrow();
        encoding_name.clone()
    }

    // https://dom.spec.whatwg.org/#dom-document-content_type
    fn ContentType(self) -> DOMString {
        self.content_type.clone()
    }

    // https://dom.spec.whatwg.org/#dom-document-doctype
    fn GetDoctype(self) -> Option<Temporary<DocumentType>> {
        let node: JSRef<Node> = NodeCast::from_ref(self);
        node.children()
            .map(|c| c.root())
            .filter_map(|c| DocumentTypeCast::to_ref(c.r()).map(Temporary::from_rooted))
            .next()
    }

    // https://dom.spec.whatwg.org/#dom-document-documentelement
    fn GetDocumentElement(self) -> Option<Temporary<Element>> {
        let node: JSRef<Node> = NodeCast::from_ref(self);
        node.child_elements().next()
    }

    // https://dom.spec.whatwg.org/#dom-document-getelementsbytagname
    fn GetElementsByTagName(self, tag_name: DOMString) -> Temporary<HTMLCollection> {
        let window = self.window.root();
        HTMLCollection::by_tag_name(window.r(), NodeCast::from_ref(self), tag_name)
    }

    // https://dom.spec.whatwg.org/#dom-document-getelementsbytagnamens
    fn GetElementsByTagNameNS(self, maybe_ns: Option<DOMString>, tag_name: DOMString)
                              -> Temporary<HTMLCollection> {
        let window = self.window.root();
        HTMLCollection::by_tag_name_ns(window.r(), NodeCast::from_ref(self), tag_name, maybe_ns)
    }

    // https://dom.spec.whatwg.org/#dom-document-getelementsbyclassname
    fn GetElementsByClassName(self, classes: DOMString) -> Temporary<HTMLCollection> {
        let window = self.window.root();

        HTMLCollection::by_class_name(window.r(), NodeCast::from_ref(self), classes)
    }

    // https://dom.spec.whatwg.org/#dom-nonelementparentnode-getelementbyid
    fn GetElementById(self, id: DOMString) -> Option<Temporary<Element>> {
        let id = Atom::from_slice(&id);
        // FIXME(https://github.com/rust-lang/rust/issues/23338)
        let idmap = self.idmap.borrow();
        idmap.get(&id).map(|ref elements| Temporary::from_rooted((*elements)[0].clone()))
    }

    // https://dom.spec.whatwg.org/#dom-document-createelement
    fn CreateElement(self, mut local_name: DOMString) -> Fallible<Temporary<Element>> {
        if xml_name_type(&local_name) == InvalidXMLName {
            debug!("Not a valid element name");
            return Err(InvalidCharacter);
        }
        if self.is_html_document {
            local_name = local_name.to_ascii_lowercase()
        }
        let name = QualName::new(ns!(HTML), Atom::from_slice(&local_name));
        Ok(Element::create(name, None, self, ElementCreator::ScriptCreated))
    }

    // https://dom.spec.whatwg.org/#dom-document-createelementns
    fn CreateElementNS(self,
                       namespace: Option<DOMString>,
                       qualified_name: DOMString) -> Fallible<Temporary<Element>> {
        let (namespace, prefix, local_name) =
            try!(validate_and_extract(namespace, &qualified_name));
        let name = QualName::new(namespace, local_name);
        Ok(Element::create(name, prefix, self, ElementCreator::ScriptCreated))
    }

    // https://dom.spec.whatwg.org/#dom-document-createattribute
    fn CreateAttribute(self, local_name: DOMString) -> Fallible<Temporary<Attr>> {
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
    fn CreateAttributeNS(self, namespace: Option<DOMString>, qualified_name: DOMString)
                         -> Fallible<Temporary<Attr>> {
        let (namespace, prefix, local_name) =
            try!(validate_and_extract(namespace, &qualified_name));
        let window = self.window.root();
        let value = AttrValue::String("".to_owned());
        let qualified_name = Atom::from_slice(&qualified_name);
        Ok(Attr::new(window.r(), local_name, value, qualified_name,
                     namespace, prefix, None))
    }

    // https://dom.spec.whatwg.org/#dom-document-createdocumentfragment
    fn CreateDocumentFragment(self) -> Temporary<DocumentFragment> {
        DocumentFragment::new(self)
    }

    // https://dom.spec.whatwg.org/#dom-document-createtextnode
    fn CreateTextNode(self, data: DOMString) -> Temporary<Text> {
        Text::new(data, self)
    }

    // https://dom.spec.whatwg.org/#dom-document-createcomment
    fn CreateComment(self, data: DOMString) -> Temporary<Comment> {
        Comment::new(data, self)
    }

    // https://dom.spec.whatwg.org/#dom-document-createprocessinginstruction
    fn CreateProcessingInstruction(self, target: DOMString, data: DOMString) ->
            Fallible<Temporary<ProcessingInstruction>> {
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
    fn ImportNode(self, node: JSRef<Node>, deep: bool) -> Fallible<Temporary<Node>> {
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
    fn AdoptNode(self, node: JSRef<Node>) -> Fallible<Temporary<Node>> {
        // Step 1.
        if node.is_document() {
            return Err(NotSupported);
        }

        // Step 2.
        Node::adopt(node, self);

        // Step 3.
        Ok(Temporary::from_rooted(node))
    }

    // https://dom.spec.whatwg.org/#dom-document-createevent
    fn CreateEvent(self, interface: DOMString) -> Fallible<Temporary<Event>> {
        let window = self.window.root();

        match &*interface.to_ascii_lowercase() {
            "uievents" | "uievent" => Ok(EventCast::from_temporary(
                UIEvent::new_uninitialized(window.r()))),
            "mouseevents" | "mouseevent" => Ok(EventCast::from_temporary(
                MouseEvent::new_uninitialized(window.r()))),
            "customevent" => Ok(EventCast::from_temporary(
                CustomEvent::new_uninitialized(GlobalRef::Window(window.r())))),
            "htmlevents" | "events" | "event" => Ok(Event::new_uninitialized(
                GlobalRef::Window(window.r()))),
            "keyboardevent" | "keyevents" => Ok(EventCast::from_temporary(
                KeyboardEvent::new_uninitialized(window.r()))),
            "messageevent" => Ok(EventCast::from_temporary(
                MessageEvent::new_uninitialized(GlobalRef::Window(window.r())))),
            _ => Err(NotSupported)
        }
    }

    // https://html.spec.whatwg.org/#dom-document-lastmodified
    fn LastModified(self) -> DOMString {
        match self.last_modified {
            Some(ref t) => t.clone(),
            None => time::now().strftime("%m/%d/%Y %H:%M:%S").unwrap().to_string(),
        }
    }

    // https://dom.spec.whatwg.org/#dom-document-createrange
    fn CreateRange(self) -> Temporary<Range> {
        Range::new_with_doc(self)
    }

    // https://dom.spec.whatwg.org/#dom-document-createtreewalker
    fn CreateTreeWalker(self, root: JSRef<Node>, whatToShow: u32, filter: Option<NodeFilter>)
                        -> Temporary<TreeWalker> {
        TreeWalker::new(self, root, whatToShow, filter)
    }

    // https://html.spec.whatwg.org/#document.title
    fn Title(self) -> DOMString {
        let title = self.GetDocumentElement().root().and_then(|root| {
            if root.r().namespace() == &ns!(SVG) && root.r().local_name() == &atom!("svg") {
                // Step 1.
                NodeCast::from_ref(root.r()).child_elements().find(|node| {
                    let node = node.root();
                    node.r().namespace() == &ns!(SVG) &&
                    node.r().local_name() == &atom!("title")
                }).map(NodeCast::from_temporary)
            } else {
                // Step 2.
                NodeCast::from_ref(root.r())
                         .traverse_preorder()
                         .find(|node| node.root().r().is_htmltitleelement())
            }
        }).root();

        match title {
            None => DOMString::new(),
            Some(ref title) => {
                // Steps 3-4.
                let value = Node::collect_text_contents(title.r().children());
                split_html_space_chars(&value).collect::<Vec<_>>().connect(" ")
            },
        }
    }

    // https://html.spec.whatwg.org/#document.title
    fn SetTitle(self, title: DOMString) {
        let root = match self.GetDocumentElement() {
            Some(root) => root,
            None => return,
        }.root();

        let elem = if root.r().namespace() == &ns!(SVG) &&
                       root.r().local_name() == &atom!("svg") {
            let elem = NodeCast::from_ref(root.r()).child_elements().find(|node| {
                let node = node.root();
                node.r().namespace() == &ns!(SVG) &&
                node.r().local_name() == &atom!("title")
            });
            match elem {
                Some(elem) => NodeCast::from_temporary(elem),
                None => {
                    let name = QualName::new(ns!(SVG), atom!("title"));
                    let elem = Element::create(name, None, self,
                                               ElementCreator::ScriptCreated);
                    NodeCast::from_ref(root.r())
                             .AppendChild(NodeCast::from_ref(elem.root().r()))
                             .unwrap()
                }
            }
        } else if root.r().namespace() == &ns!(HTML) {
            let elem = NodeCast::from_ref(root.r())
                                .traverse_preorder()
                                .find(|node| node.root().r().is_htmltitleelement());
            match elem {
                Some(elem) => elem,
                None => {
                    match self.GetHead() {
                        Some(head) => {
                            let name = QualName::new(ns!(HTML), atom!("title"));
                            let elem = Element::create(name, None, self,
                                                       ElementCreator::ScriptCreated);
                            NodeCast::from_ref(head.root().r())
                                     .AppendChild(NodeCast::from_ref(elem.root().r()))
                                     .unwrap()
                        },
                        None => return,
                    }
                }
            }
        } else {
            return
        };

        elem.root().r().SetTextContent(Some(title));
    }

    // https://html.spec.whatwg.org/#dom-document-head
    fn GetHead(self) -> Option<Temporary<HTMLHeadElement>> {
        self.get_html_element().and_then(|root| {
            let root = root.root();
            let node: JSRef<Node> = NodeCast::from_ref(root.r());
            node.children()
                .map(|c| c.root())
                .filter_map(|c| HTMLHeadElementCast::to_ref(c.r()).map(Temporary::from_rooted))
                .next()
        })
    }

    // https://html.spec.whatwg.org/#dom-document-currentscript
    fn GetCurrentScript(self) -> Option<Temporary<HTMLScriptElement>> {
        self.current_script.get().map(Temporary::from_rooted)
    }

    // https://html.spec.whatwg.org/#dom-document-body
    fn GetBody(self) -> Option<Temporary<HTMLElement>> {
        self.get_html_element().and_then(|root| {
            let root = root.root();
            let node: JSRef<Node> = NodeCast::from_ref(root.r());
            node.children().map(|c| c.root()).find(|child| {
                match child.r().type_id() {
                    NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLBodyElement)) |
                    NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLFrameSetElement)) => true,
                    _ => false
                }
            }).map(|node| {
                Temporary::from_rooted(HTMLElementCast::to_ref(node.r()).unwrap())
            })
        })
    }

    // https://html.spec.whatwg.org/#dom-document-body
    fn SetBody(self, new_body: Option<JSRef<HTMLElement>>) -> ErrorResult {
        // Step 1.
        let new_body = match new_body {
            Some(new_body) => new_body,
            None => return Err(HierarchyRequest),
        };

        let node: JSRef<Node> = NodeCast::from_ref(new_body);
        match node.type_id() {
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLBodyElement)) |
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLFrameSetElement)) => {}
            _ => return Err(HierarchyRequest)
        }

        // Step 2.
        let old_body = self.GetBody().root();
        if old_body.as_ref().map(|body| body.r()) == Some(new_body) {
            return Ok(());
        }

        match (self.get_html_element().root(), &old_body) {
            // Step 3.
            (Some(ref root), &Some(ref child)) => {
                let root: JSRef<Node> = NodeCast::from_ref(root.r());
                let child: JSRef<Node> = NodeCast::from_ref(child.r());
                let new_body: JSRef<Node> = NodeCast::from_ref(new_body);
                assert!(root.ReplaceChild(new_body, child).is_ok())
            },

            // Step 4.
            (None, _) => return Err(HierarchyRequest),

            // Step 5.
            (Some(ref root), &None) => {
                let root: JSRef<Node> = NodeCast::from_ref(root.r());
                let new_body: JSRef<Node> = NodeCast::from_ref(new_body);
                assert!(root.AppendChild(new_body).is_ok());
            }
        }
        Ok(())
    }

    // https://html.spec.whatwg.org/#dom-document-getelementsbyname
    fn GetElementsByName(self, name: DOMString) -> Temporary<NodeList> {
        self.create_node_list(|node| {
            let element: JSRef<Element> = match ElementCast::to_ref(node) {
                Some(element) => element,
                None => return false,
            };
            element.get_attribute(&ns!(""), &atom!("name")).root().map_or(false, |attr| {
                // FIXME(https://github.com/rust-lang/rust/issues/23338)
                let attr = attr.r();
                let value = attr.value();
                &**value == &*name
            })
        })
    }

    // https://html.spec.whatwg.org/#dom-document-images
    fn Images(self) -> Temporary<HTMLCollection> {
        self.images.or_init(|| {
            let window = self.window.root();
            let root = NodeCast::from_ref(self);
            let filter = box ImagesFilter;
            HTMLCollection::create(window.r(), root, filter)
        })
    }

    // https://html.spec.whatwg.org/#dom-document-embeds
    fn Embeds(self) -> Temporary<HTMLCollection> {
        self.embeds.or_init(|| {
            let window = self.window.root();
            let root = NodeCast::from_ref(self);
            let filter = box EmbedsFilter;
            HTMLCollection::create(window.r(), root, filter)
        })
    }

    // https://html.spec.whatwg.org/#dom-document-plugins
    fn Plugins(self) -> Temporary<HTMLCollection> {
        self.Embeds()
    }

    // https://html.spec.whatwg.org/#dom-document-links
    fn Links(self) -> Temporary<HTMLCollection> {
        self.links.or_init(|| {
            let window = self.window.root();
            let root = NodeCast::from_ref(self);
            let filter = box LinksFilter;
            HTMLCollection::create(window.r(), root, filter)
        })
    }

    // https://html.spec.whatwg.org/#dom-document-forms
    fn Forms(self) -> Temporary<HTMLCollection> {
        self.forms.or_init(|| {
            let window = self.window.root();
            let root = NodeCast::from_ref(self);
            let filter = box FormsFilter;
            HTMLCollection::create(window.r(), root, filter)
        })
    }

    // https://html.spec.whatwg.org/#dom-document-scripts
    fn Scripts(self) -> Temporary<HTMLCollection> {
        self.scripts.or_init(|| {
            let window = self.window.root();
            let root = NodeCast::from_ref(self);
            let filter = box ScriptsFilter;
            HTMLCollection::create(window.r(), root, filter)
        })
    }

    // https://html.spec.whatwg.org/#dom-document-anchors
    fn Anchors(self) -> Temporary<HTMLCollection> {
        self.anchors.or_init(|| {
            let window = self.window.root();
            let root = NodeCast::from_ref(self);
            let filter = box AnchorsFilter;
            HTMLCollection::create(window.r(), root, filter)
        })
    }

    // https://html.spec.whatwg.org/#dom-document-applets
    fn Applets(self) -> Temporary<HTMLCollection> {
        // FIXME: This should be return OBJECT elements containing applets.
        self.applets.or_init(|| {
            let window = self.window.root();
            let root = NodeCast::from_ref(self);
            let filter = box AppletsFilter;
            HTMLCollection::create(window.r(), root, filter)
        })
    }

    // https://html.spec.whatwg.org/#dom-document-location
    fn Location(self) -> Temporary<Location> {
        let window = self.window.root();
        let window = window.r();
        self.location.or_init(|| Location::new(window))
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-children
    fn Children(self) -> Temporary<HTMLCollection> {
        let window = self.window.root();
        HTMLCollection::children(window.r(), NodeCast::from_ref(self))
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-firstelementchild
    fn GetFirstElementChild(self) -> Option<Temporary<Element>> {
        NodeCast::from_ref(self).child_elements().next()
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-lastelementchild
    fn GetLastElementChild(self) -> Option<Temporary<Element>> {
        NodeCast::from_ref(self).rev_children().filter_map(ElementCast::to_temporary).next()
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-childelementcount
    fn ChildElementCount(self) -> u32 {
        NodeCast::from_ref(self).child_elements().count() as u32
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-prepend
    fn Prepend(self, nodes: Vec<NodeOrString>) -> ErrorResult {
        NodeCast::from_ref(self).prepend(nodes)
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-append
    fn Append(self, nodes: Vec<NodeOrString>) -> ErrorResult {
        NodeCast::from_ref(self).append(nodes)
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-queryselector
    fn QuerySelector(self, selectors: DOMString) -> Fallible<Option<Temporary<Element>>> {
        let root: JSRef<Node> = NodeCast::from_ref(self);
        root.query_selector(selectors)
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-queryselectorall
    fn QuerySelectorAll(self, selectors: DOMString) -> Fallible<Temporary<NodeList>> {
        let root: JSRef<Node> = NodeCast::from_ref(self);
        root.query_selector_all(selectors)
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-readystate
    fn ReadyState(self) -> DocumentReadyState {
        self.ready_state.get()
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-defaultview
    fn DefaultView(self) -> Temporary<Window> {
        Temporary::from_rooted(self.window)
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-cookie
    fn GetCookie(self) -> Fallible<DOMString> {
        //TODO: return empty string for cookie-averse Document
        let url = self.url();
        if !is_scheme_host_port_tuple(&url) {
            return Err(Security);
        }
        let window = self.window.root();
        let (tx, rx) = channel();
        let _ = window.r().resource_task().send(GetCookiesForUrl(url, tx, NonHTTP));
        let cookies = rx.recv().unwrap();
        Ok(cookies.unwrap_or("".to_owned()))
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-cookie
    fn SetCookie(self, cookie: DOMString) -> ErrorResult {
        //TODO: ignore for cookie-averse Document
        let url = self.url();
        if !is_scheme_host_port_tuple(&url) {
            return Err(Security);
        }
        let window = self.window.root();
        let _ = window.r().resource_task().send(SetCookiesForUrl(url, cookie, NonHTTP));
        Ok(())
    }

    fn BgColor(self) -> DOMString {
        self.get_body_attribute(&atom!("bgcolor"))
    }

    fn SetBgColor(self, value: DOMString) {
        self.set_body_attribute(&atom!("bgcolor"), value)
    }

    global_event_handlers!();
    event_handler!(readystatechange, GetOnreadystatechange, SetOnreadystatechange);
}

fn is_scheme_host_port_tuple(url: &Url) -> bool {
    url.host().is_some() && url.port_or_default().is_some()
}

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
        let document = self.addr.to_temporary().root();
        let window = document.r().window().root();
        let event = Event::new(GlobalRef::Window(window.r()), "DOMContentLoaded".to_owned(),
                               EventBubbles::DoesNotBubble,
                               EventCancelable::NotCancelable).root();
        let doctarget: JSRef<EventTarget> = EventTargetCast::from_ref(document.r());
        let _ = doctarget.DispatchEvent(event.r());

        window.r().reflow(ReflowGoal::ForDisplay, ReflowQueryType::NoQuery, ReflowReason::DOMContentLoaded);
    }

    fn set_ready_state_complete(&self) {
        let document = self.addr.to_temporary().root();
        document.r().set_ready_state(DocumentReadyState::Complete);
    }

    fn dispatch_load(&self) {
        let document = self.addr.to_temporary().root();
        let window = document.r().window().root();
        let event = Event::new(GlobalRef::Window(window.r()), "load".to_owned(),
                               EventBubbles::DoesNotBubble,
                               EventCancelable::NotCancelable).root();
        let wintarget: JSRef<EventTarget> = EventTargetCast::from_ref(window.r());
        let doctarget: JSRef<EventTarget> = EventTargetCast::from_ref(document.r());
        event.r().set_trusted(true);
        let _ = wintarget.dispatch_event_with_target(doctarget, event.r());

        let window_ref = window.r();
        let browser_context = window_ref.browser_context();
        let browser_context = browser_context.as_ref().unwrap();

        browser_context.frame_element().map(|frame_element| {
            let frame_element = frame_element.root();
            let frame_window = window_from_node(frame_element.r()).root();
            let event = Event::new(GlobalRef::Window(frame_window.r()), "load".to_owned(),
                                   EventBubbles::DoesNotBubble,
                                   EventCancelable::NotCancelable).root();
            let target: JSRef<EventTarget> = EventTargetCast::from_ref(frame_element.r());
            event.r().fire(target);
        });

        // https://developer.mozilla.org/en-US/docs/Web/Events/mozbrowserloadend
        document.r().trigger_mozbrowser_event(MozBrowserEvent::LoadEnd);

        window_ref.reflow(ReflowGoal::ForDisplay,
                          ReflowQueryType::NoQuery,
                          ReflowReason::DocumentLoaded);
    }
}

impl Runnable for DocumentProgressHandler {
    fn handler(self: Box<DocumentProgressHandler>) {
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
