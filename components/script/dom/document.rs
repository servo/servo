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
use dom::bindings::codegen::Bindings::EventHandlerBinding::OnErrorEventHandlerNonNull;
use dom::bindings::codegen::Bindings::EventTargetBinding::EventTargetMethods;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::codegen::Bindings::NodeFilterBinding::NodeFilter;
use dom::bindings::codegen::Bindings::PerformanceBinding::PerformanceMethods;
use dom::bindings::codegen::Bindings::TouchBinding::TouchMethods;
use dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use dom::bindings::codegen::UnionTypes::NodeOrString;
use dom::bindings::error::{Error, ErrorResult, Fallible};
use dom::bindings::global::GlobalRef;
use dom::bindings::inheritance::{Castable, ElementTypeId, HTMLElementTypeId, NodeTypeId};
use dom::bindings::js::RootedReference;
use dom::bindings::js::{JS, LayoutJS, MutNullableHeap, Root};
use dom::bindings::num::Finite;
use dom::bindings::refcounted::Trusted;
use dom::bindings::reflector::{Reflectable, reflect_dom_object};
use dom::bindings::trace::RootedVec;
use dom::bindings::xmlname::XMLName::InvalidXMLName;
use dom::bindings::xmlname::{validate_and_extract, namespace_from_domstring, xml_name_type};
use dom::comment::Comment;
use dom::customevent::CustomEvent;
use dom::documentfragment::DocumentFragment;
use dom::documenttype::DocumentType;
use dom::domimplementation::DOMImplementation;
use dom::element::{Element, ElementCreator};
use dom::event::{Event, EventBubbles, EventCancelable};
use dom::eventtarget::EventTarget;
use dom::htmlanchorelement::HTMLAnchorElement;
use dom::htmlappletelement::HTMLAppletElement;
use dom::htmlareaelement::HTMLAreaElement;
use dom::htmlbaseelement::HTMLBaseElement;
use dom::htmlbodyelement::HTMLBodyElement;
use dom::htmlcollection::{CollectionFilter, HTMLCollection};
use dom::htmlelement::HTMLElement;
use dom::htmlembedelement::HTMLEmbedElement;
use dom::htmlformelement::HTMLFormElement;
use dom::htmlheadelement::HTMLHeadElement;
use dom::htmlhtmlelement::HTMLHtmlElement;
use dom::htmliframeelement::{self, HTMLIFrameElement};
use dom::htmlimageelement::HTMLImageElement;
use dom::htmllinkelement::HTMLLinkElement;
use dom::htmlmetaelement::HTMLMetaElement;
use dom::htmlscriptelement::HTMLScriptElement;
use dom::htmlstyleelement::HTMLStyleElement;
use dom::htmltitleelement::HTMLTitleElement;
use dom::keyboardevent::KeyboardEvent;
use dom::location::Location;
use dom::messageevent::MessageEvent;
use dom::mouseevent::MouseEvent;
use dom::node::{self, CloneChildrenFlag, Node, NodeDamage, window_from_node};
use dom::nodeiterator::NodeIterator;
use dom::nodelist::NodeList;
use dom::processinginstruction::ProcessingInstruction;
use dom::range::Range;
use dom::servohtmlparser::ServoHTMLParser;
use dom::text::Text;
use dom::touch::Touch;
use dom::touchevent::TouchEvent;
use dom::touchlist::TouchList;
use dom::treewalker::TreeWalker;
use dom::uievent::UIEvent;
use dom::window::{ReflowReason, Window};
use euclid::point::Point2D;
use html5ever::tree_builder::{LimitedQuirks, NoQuirks, Quirks, QuirksMode};
use ipc_channel::ipc::{self, IpcSender};
use js::jsapi::{JSContext, JSObject, JSRuntime};
use layout_interface::{HitTestResponse, MouseOverResponse};
use layout_interface::{LayoutChan, Msg};
use layout_interface::{ReflowGoal, ReflowQueryType};
use msg::compositor_msg::ScriptToCompositorMsg;
use msg::constellation_msg::AnimationState;
use msg::constellation_msg::ScriptMsg as ConstellationMsg;
use msg::constellation_msg::{ALT, CONTROL, SHIFT, SUPER};
use msg::constellation_msg::{ConstellationChan, FocusType, Key, KeyModifiers, KeyState, MozBrowserEvent, SubpageId};
use net_traits::ControlMsg::{GetCookiesForUrl, SetCookiesForUrl};
use net_traits::CookieSource::NonHTTP;
use net_traits::{AsyncResponseTarget, PendingAsyncLoad};
use num::ToPrimitive;
use script_task::{MainThreadScriptMsg, Runnable};
use script_traits::{MouseButton, TouchEventType, TouchId, UntrustedNodeAddress};
use std::ascii::AsciiExt;
use std::borrow::ToOwned;
use std::boxed::FnBox;
use std::cell::{Cell, Ref, RefMut};
use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::default::Default;
use std::iter::FromIterator;
use std::ptr;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::mpsc::channel;
use string_cache::{Atom, QualName};
use style::restyle_hints::ElementSnapshot;
use style::stylesheets::Stylesheet;
use time;
use url::Url;
use util::str::{DOMString, split_html_space_chars, str_join};

#[derive(JSTraceable, PartialEq, HeapSizeOf)]
pub enum IsHTMLDocument {
    HTMLDocument,
    NonHTMLDocument,
}

#[derive(PartialEq)]
enum ParserBlockedByScript {
    Blocked,
    Unblocked,
}

// https://dom.spec.whatwg.org/#document
#[dom_struct]
pub struct Document {
    node: Node,
    window: JS<Window>,
    implementation: MutNullableHeap<JS<DOMImplementation>>,
    location: MutNullableHeap<JS<Location>>,
    content_type: DOMString,
    last_modified: Option<String>,
    encoding_name: DOMRefCell<DOMString>,
    is_html_document: bool,
    url: Url,
    quirks_mode: Cell<QuirksMode>,
    /// Caches for the getElement methods
    id_map: DOMRefCell<HashMap<Atom, Vec<JS<Element>>>>,
    tag_map: DOMRefCell<HashMap<Atom, JS<HTMLCollection>>>,
    tagns_map: DOMRefCell<HashMap<QualName, JS<HTMLCollection>>>,
    classes_map: DOMRefCell<HashMap<Vec<Atom>, JS<HTMLCollection>>>,
    images: MutNullableHeap<JS<HTMLCollection>>,
    embeds: MutNullableHeap<JS<HTMLCollection>>,
    links: MutNullableHeap<JS<HTMLCollection>>,
    forms: MutNullableHeap<JS<HTMLCollection>>,
    scripts: MutNullableHeap<JS<HTMLCollection>>,
    anchors: MutNullableHeap<JS<HTMLCollection>>,
    applets: MutNullableHeap<JS<HTMLCollection>>,
    /// List of stylesheets associated with nodes in this document. |None| if the list needs to be refreshed.
    stylesheets: DOMRefCell<Option<Vec<Arc<Stylesheet>>>>,
    /// Whether the list of stylesheets has changed since the last reflow was triggered.
    stylesheets_changed_since_reflow: Cell<bool>,
    ready_state: Cell<DocumentReadyState>,
    /// Whether the DOMContentLoaded event has already been dispatched.
    domcontentloaded_dispatched: Cell<bool>,
    /// The element that has most recently requested focus for itself.
    possibly_focused: MutNullableHeap<JS<Element>>,
    /// The element that currently has the document focus context.
    focused: MutNullableHeap<JS<Element>>,
    /// The script element that is currently executing.
    current_script: MutNullableHeap<JS<HTMLScriptElement>>,
    /// https://html.spec.whatwg.org/multipage/#pending-parsing-blocking-script
    pending_parsing_blocking_script: MutNullableHeap<JS<HTMLScriptElement>>,
    /// Number of stylesheets that block executing the next parser-inserted script
    script_blocking_stylesheets_count: Cell<u32>,
    /// https://html.spec.whatwg.org/multipage/#list-of-scripts-that-will-execute-when-the-document-has-finished-parsing
    deferred_scripts: DOMRefCell<Vec<JS<HTMLScriptElement>>>,
    /// https://html.spec.whatwg.org/multipage/#list-of-scripts-that-will-execute-in-order-as-soon-as-possible
    asap_in_order_scripts_list: DOMRefCell<Vec<JS<HTMLScriptElement>>>,
    /// https://html.spec.whatwg.org/multipage/#set-of-scripts-that-will-execute-as-soon-as-possible
    asap_scripts_set: DOMRefCell<Vec<JS<HTMLScriptElement>>>,
    /// https://html.spec.whatwg.org/multipage/#concept-n-noscript
    /// True if scripting is enabled for all scripts in this document
    scripting_enabled: Cell<bool>,
    /// https://html.spec.whatwg.org/multipage/#animation-frame-callback-identifier
    /// Current identifier of animation frame callback
    animation_frame_ident: Cell<u32>,
    /// https://html.spec.whatwg.org/multipage/#list-of-animation-frame-callbacks
    /// List of animation frame callbacks
    #[ignore_heap_size_of = "closures are hard"]
    animation_frame_list: DOMRefCell<HashMap<u32, Box<FnBox(f64)>>>,
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
    /// For each element that has had a state or attribute change since the last restyle,
    /// track the original condition of the element.
    modified_elements: DOMRefCell<HashMap<JS<Element>, ElementSnapshot>>,
    /// http://w3c.github.io/touch-events/#dfn-active-touch-point
    active_touch_points: DOMRefCell<Vec<JS<Touch>>>,
    /// DOM-Related Navigation Timing properties:
    /// http://w3c.github.io/navigation-timing/#widl-PerformanceTiming-domLoading
    dom_loading: Cell<u64>,
    dom_interactive: Cell<u64>,
    dom_content_loaded_event_start: Cell<u64>,
    dom_content_loaded_event_end: Cell<u64>,
    dom_complete: Cell<u64>,
}

impl PartialEq for Document {
    fn eq(&self, other: &Document) -> bool {
        self as *const Document == &*other
    }
}

#[derive(JSTraceable, HeapSizeOf)]
struct ImagesFilter;
impl CollectionFilter for ImagesFilter {
    fn filter(&self, elem: &Element, _root: &Node) -> bool {
        elem.is::<HTMLImageElement>()
    }
}

#[derive(JSTraceable, HeapSizeOf)]
struct EmbedsFilter;
impl CollectionFilter for EmbedsFilter {
    fn filter(&self, elem: &Element, _root: &Node) -> bool {
        elem.is::<HTMLEmbedElement>()
    }
}

#[derive(JSTraceable, HeapSizeOf)]
struct LinksFilter;
impl CollectionFilter for LinksFilter {
    fn filter(&self, elem: &Element, _root: &Node) -> bool {
        (elem.is::<HTMLAnchorElement>() || elem.is::<HTMLAreaElement>()) &&
        elem.has_attribute(&atom!("href"))
    }
}

#[derive(JSTraceable, HeapSizeOf)]
struct FormsFilter;
impl CollectionFilter for FormsFilter {
    fn filter(&self, elem: &Element, _root: &Node) -> bool {
        elem.is::<HTMLFormElement>()
    }
}

#[derive(JSTraceable, HeapSizeOf)]
struct ScriptsFilter;
impl CollectionFilter for ScriptsFilter {
    fn filter(&self, elem: &Element, _root: &Node) -> bool {
        elem.is::<HTMLScriptElement>()
    }
}

#[derive(JSTraceable, HeapSizeOf)]
struct AnchorsFilter;
impl CollectionFilter for AnchorsFilter {
    fn filter(&self, elem: &Element, _root: &Node) -> bool {
        elem.is::<HTMLAnchorElement>() && elem.has_attribute(&atom!("href"))
    }
}

#[derive(JSTraceable, HeapSizeOf)]
struct AppletsFilter;
impl CollectionFilter for AppletsFilter {
    fn filter(&self, elem: &Element, _root: &Node) -> bool {
        elem.is::<HTMLAppletElement>()
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
    pub fn window(&self) -> &Window {
        &*self.window
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
        let browsing_context = self.window.browsing_context();
        let browsing_context = browsing_context.as_ref().unwrap();
        let active_document = browsing_context.active_document();

        if self != active_document {
            return false;
        }
        // FIXME: It should also check whether the browser context is top-level or not
        true
    }

    // https://dom.spec.whatwg.org/#concept-document-url
    pub fn url(&self) -> &Url {
        &self.url
    }

    // https://html.spec.whatwg.org/multipage/#fallback-base-url
    pub fn fallback_base_url(&self) -> Url {
        // Step 1: iframe srcdoc (#4767).
        // Step 2: about:blank with a creator browsing context.
        // Step 3.
        self.url().clone()
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

    pub fn needs_reflow(&self) -> bool {
        self.GetDocumentElement().is_some() &&
        (self.upcast::<Node>().get_has_dirty_descendants() ||
         !self.modified_elements.borrow().is_empty())
    }

    /// Returns the first `base` element in the DOM that has an `href` attribute.
    pub fn base_element(&self) -> Option<Root<HTMLBaseElement>> {
        self.base_element.get()
    }

    /// Refresh the cached first base element in the DOM.
    /// https://github.com/w3c/web-platform-tests/issues/2122
    pub fn refresh_base_element(&self) {
        let base = self.upcast::<Node>()
                       .traverse_preorder()
                       .filter_map(Root::downcast::<HTMLBaseElement>)
                       .find(|element| element.upcast::<Element>().has_attribute(&atom!("href")));
        self.base_element.set(base.r());
    }

    pub fn quirks_mode(&self) -> QuirksMode {
        self.quirks_mode.get()
    }

    pub fn set_quirks_mode(&self, mode: QuirksMode) {
        self.quirks_mode.set(mode);

        if mode == Quirks {
            let LayoutChan(ref layout_chan) = self.window.layout_chan();
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
    }

    /// Reflows and disarms the timer if the reflow timer has expired.
    pub fn reflow_if_reflow_timer_expired(&self) {
        if let Some(reflow_timeout) = self.reflow_timeout.get() {
            if time::precise_time_ns() < reflow_timeout {
                return;
            }

            self.reflow_timeout.set(None);
            self.window.reflow(ReflowGoal::ForDisplay,
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
                return;
            }
        }
        self.reflow_timeout.set(Some(timeout))
    }

    /// Disables any pending reflow timeouts.
    pub fn disarm_reflow_timeout(&self) {
        self.reflow_timeout.set(None)
    }

    /// Remove any existing association between the provided id and any elements in this document.
    pub fn unregister_named_element(&self, to_unregister: &Element, id: Atom) {
        debug!("Removing named element from document {:p}: {:p} id={}",
               self,
               to_unregister,
               id);
        let mut id_map = self.id_map.borrow_mut();
        let is_empty = match id_map.get_mut(&id) {
            None => false,
            Some(elements) => {
                let position = elements.iter()
                                       .position(|element| &**element == to_unregister)
                                       .expect("This element should be in registered.");
                elements.remove(position);
                elements.is_empty()
            }
        };
        if is_empty {
            id_map.remove(&id);
        }
    }

    /// Associate an element present in this document with the provided id.
    pub fn register_named_element(&self, element: &Element, id: Atom) {
        debug!("Adding named element to document {:p}: {:p} id={}",
               self,
               element,
               id);
        assert!(element.upcast::<Node>().is_in_doc());
        assert!(!id.is_empty());

        let mut id_map = self.id_map.borrow_mut();

        let root = self.GetDocumentElement()
                       .expect("The element is in the document, so there must be a document \
                                element.");

        match id_map.entry(id) {
            Vacant(entry) => {
                entry.insert(vec![JS::from_ref(element)]);
            }
            Occupied(entry) => {
                let elements = entry.into_mut();

                let new_node = element.upcast::<Node>();
                let mut head: usize = 0;
                let root = root.upcast::<Node>();
                for node in root.traverse_preorder() {
                    if let Some(elem) = node.downcast() {
                        if &*(*elements)[head] == elem {
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
        self.get_element_by_id(&Atom::from_slice(fragid)).or_else(|| {
            let check_anchor = |node: &HTMLAnchorElement| {
                let elem = node.upcast::<Element>();
                elem.get_attribute(&ns!(""), &atom!("name"))
                    .map_or(false, |attr| &**attr.value() == fragid)
            };
            let doc_node = self.upcast::<Node>();
            doc_node.traverse_preorder()
                    .filter_map(Root::downcast)
                    .find(|node| check_anchor(&node))
                    .map(Root::upcast)
        })
    }

    pub fn hit_test(&self, point: &Point2D<f32>) -> Option<UntrustedNodeAddress> {
        assert!(self.GetDocumentElement().is_some());
        match self.window.layout().hit_test(*point) {
            Ok(HitTestResponse(node_address)) => Some(node_address),
            Err(()) => {
                debug!("layout query error");
                None
            }
        }
    }

    pub fn get_nodes_under_mouse(&self, point: &Point2D<f32>) -> Vec<UntrustedNodeAddress> {
        assert!(self.GetDocumentElement().is_some());
        match self.window.layout().mouse_over(*point) {
            Ok(MouseOverResponse(node_address)) => node_address,
            Err(()) => vec![],
        }
    }

    // https://html.spec.whatwg.org/multipage/#current-document-readiness
    pub fn set_ready_state(&self, state: DocumentReadyState) {
        match state {
            DocumentReadyState::Loading => update_with_current_time(&self.dom_loading),
            DocumentReadyState::Interactive => update_with_current_time(&self.dom_interactive),
            DocumentReadyState::Complete => update_with_current_time(&self.dom_complete),
        };

        self.ready_state.set(state);

        let event = Event::new(GlobalRef::Window(&self.window),
                               DOMString::from("readystatechange"),
                               EventBubbles::DoesNotBubble,
                               EventCancelable::NotCancelable);
        let target = self.upcast::<EventTarget>();
        let _ = event.fire(target);
    }

    /// Return whether scripting is enabled or not
    pub fn is_scripting_enabled(&self) -> bool {
        self.scripting_enabled.get()
    }

    /// Return the element that currently has focus.
    // https://dvcs.w3.org/hg/dom3events/raw-file/tip/html/DOM3-Events.html#events-focusevent-doc-focus
    pub fn get_focused_element(&self) -> Option<Root<Element>> {
        self.focused.get()
    }

    /// Initiate a new round of checking for elements requesting focus. The last element to call
    /// `request_focus` before `commit_focus_transaction` is called will receive focus.
    pub fn begin_focus_transaction(&self) {
        self.possibly_focused.set(None);
    }

    /// Request that the given element receive focus once the current transaction is complete.
    pub fn request_focus(&self, elem: &Element) {
        if elem.is_focusable_area() {
            self.possibly_focused.set(Some(elem))
        }
    }

    /// Reassign the focus context to the element that last requested focus during this
    /// transaction, or none if no elements requested it.
    pub fn commit_focus_transaction(&self, focus_type: FocusType) {
        // TODO: dispatch blur, focus, focusout, and focusin events

        if let Some(ref elem) = self.focused.get() {
            elem.set_focus_state(false);
        }

        self.focused.set(self.possibly_focused.get().r());

        if let Some(ref elem) = self.focused.get() {
            elem.set_focus_state(true);

            // Update the focus state for all elements in the focus chain.
            // https://html.spec.whatwg.org/multipage/#focus-chain
            if focus_type == FocusType::Element {
                let ConstellationChan(ref chan) = self.window.constellation_chan();
                let event = ConstellationMsg::Focus(self.window.pipeline());
                chan.send(event).unwrap();
            }
        }
    }

    /// Handles any updates when the document's title has changed.
    pub fn title_changed(&self) {
        // https://developer.mozilla.org/en-US/docs/Web/Events/mozbrowsertitlechange
        self.trigger_mozbrowser_event(MozBrowserEvent::TitleChange(String::from(self.Title())));

        self.send_title_to_compositor();
    }

    /// Sends this document's title to the compositor.
    pub fn send_title_to_compositor(&self) {
        let window = self.window();
        let compositor = window.compositor();
        compositor.send(ScriptToCompositorMsg::SetTitle(window.pipeline(),
                                                        Some(String::from(self.Title()))))
                  .unwrap();
    }

    pub fn dirty_all_nodes(&self) {
        let root = self.upcast::<Node>();
        for node in root.traverse_preorder() {
            node.dirty(NodeDamage::OtherNodeDamage)
        }
    }

    pub fn handle_mouse_event(&self,
                              js_runtime: *mut JSRuntime,
                              _button: MouseButton,
                              point: Point2D<f32>,
                              mouse_event_type: MouseEventType) {
        let mouse_event_type_string = match mouse_event_type {
            MouseEventType::Click => "click".to_owned(),
            MouseEventType::MouseUp => "mouseup".to_owned(),
            MouseEventType::MouseDown => "mousedown".to_owned(),
        };
        debug!("{}: at {:?}", mouse_event_type_string, point);
        let node = match self.hit_test(&point) {
            Some(node_address) => {
                debug!("node address is {:?}", node_address);
                node::from_untrusted_node_address(js_runtime, node_address)
            },
            None => return,
        };

        let el = match node.downcast::<Element>() {
            Some(el) => Root::from_ref(el),
            None => {
                let parent = node.GetParentNode();
                match parent.and_then(Root::downcast::<Element>) {
                    Some(parent) => parent,
                    None => return,
                }
            },
        };

        let node = el.upcast::<Node>();
        debug!("{} on {:?}", mouse_event_type_string, node.debug_str());
        // Prevent click event if form control element is disabled.
        if let MouseEventType::Click = mouse_event_type {
            if el.click_event_filter_by_disabled_state() {
                return;
            }

            self.begin_focus_transaction();
        }

        // https://dvcs.w3.org/hg/dom3events/raw-file/tip/html/DOM3-Events.html#event-type-click
        let x = point.x as i32;
        let y = point.y as i32;
        let clickCount = 1;
        let event = MouseEvent::new(&self.window,
                                    DOMString::from(mouse_event_type_string),
                                    EventBubbles::Bubbles,
                                    EventCancelable::Cancelable,
                                    Some(&self.window),
                                    clickCount,
                                    x,
                                    y,
                                    x,
                                    y, // TODO: Get real screen coordinates?
                                    false,
                                    false,
                                    false,
                                    false,
                                    0i16,
                                    None);
        let event = event.upcast::<Event>();

        // https://dvcs.w3.org/hg/dom3events/raw-file/tip/html/DOM3-Events.html#trusted-events
        event.set_trusted(true);
        // https://html.spec.whatwg.org/multipage/#run-authentic-click-activation-steps
        match mouse_event_type {
            MouseEventType::Click => el.authentic_click_activation(event),
            _ => {
                let target = node.upcast();
                event.fire(target);
            },
        }

        if let MouseEventType::Click = mouse_event_type {
            self.commit_focus_transaction(FocusType::Element);
        }
        self.window.reflow(ReflowGoal::ForDisplay,
                           ReflowQueryType::NoQuery,
                           ReflowReason::MouseEvent);
    }

    pub fn fire_mouse_event(&self, point: Point2D<f32>, target: &EventTarget, event_name: String) {
        let x = point.x.to_i32().unwrap_or(0);
        let y = point.y.to_i32().unwrap_or(0);

        let mouse_event = MouseEvent::new(&self.window,
                                          DOMString::from(event_name),
                                          EventBubbles::Bubbles,
                                          EventCancelable::Cancelable,
                                          Some(&self.window),
                                          0i32,
                                          x,
                                          y,
                                          x,
                                          y,
                                          false,
                                          false,
                                          false,
                                          false,
                                          0i16,
                                          None);
        let event = mouse_event.upcast::<Event>();
        event.fire(target);
    }

    pub fn handle_mouse_move_event(&self,
                                   js_runtime: *mut JSRuntime,
                                   point: Option<Point2D<f32>>,
                                   prev_mouse_over_targets: &mut RootedVec<JS<Element>>) {
        // Build a list of elements that are currently under the mouse.
        let mouse_over_addresses = point.as_ref()
                                        .map(|point| self.get_nodes_under_mouse(point))
                                        .unwrap_or(vec![]);
        let mut mouse_over_targets = mouse_over_addresses.iter().map(|node_address| {
            node::from_untrusted_node_address(js_runtime, *node_address)
                .inclusive_ancestors()
                .filter_map(Root::downcast::<Element>)
                .next()
                .unwrap()
        }).collect::<RootedVec<JS<Element>>>();

        // Remove hover from any elements in the previous list that are no longer
        // under the mouse.
        for target in prev_mouse_over_targets.iter() {
            if !mouse_over_targets.contains(target) {
                let target_ref = &**target;
                if target_ref.get_hover_state() {
                    target_ref.set_hover_state(false);

                    let target = target_ref.upcast();

                    // FIXME: we should be dispatching this event but we lack an actual
                    //        point to pass to it.
                    if let Some(point) = point {
                        self.fire_mouse_event(point, &target, "mouseout".to_owned());
                    }
                }
            }
        }

        // Set hover state for any elements in the current mouse over list.
        // Check if any of them changed state to determine whether to
        // force a reflow below.
        for target in mouse_over_targets.r() {
            if !target.get_hover_state() {
                target.set_hover_state(true);

                let target = target.upcast();

                if let Some(point) = point {
                    self.fire_mouse_event(point, target, "mouseover".to_owned());
                }
            }
        }

        // Send mousemove event to topmost target
        if mouse_over_addresses.len() > 0 {
            let top_most_node = node::from_untrusted_node_address(js_runtime,
                                                                  mouse_over_addresses[0]);

            let target = top_most_node.upcast();
            if let Some(point) = point {
                self.fire_mouse_event(point, target, "mousemove".to_owned());
            }
        }

        // Store the current mouse over targets for next frame
        prev_mouse_over_targets.clear();
        prev_mouse_over_targets.append(&mut *mouse_over_targets);

        self.window.reflow(ReflowGoal::ForDisplay,
                           ReflowQueryType::NoQuery,
                           ReflowReason::MouseEvent);
    }

    pub fn handle_touch_event(&self,
                              js_runtime: *mut JSRuntime,
                              event_type: TouchEventType,
                              TouchId(identifier): TouchId,
                              point: Point2D<f32>)
                              -> bool {
        let event_name = match event_type {
            TouchEventType::Down => "touchstart",
            TouchEventType::Move => "touchmove",
            TouchEventType::Up => "touchend",
            TouchEventType::Cancel => "touchcancel",
        };

        let node = match self.hit_test(&point) {
            Some(node_address) => node::from_untrusted_node_address(js_runtime, node_address),
            None => return false,
        };
        let el = match node.downcast::<Element>() {
            Some(el) => Root::from_ref(el),
            None => {
                let parent = node.GetParentNode();
                match parent.and_then(Root::downcast::<Element>) {
                    Some(parent) => parent,
                    None => return false,
                }
            },
        };
        let target = Root::upcast::<EventTarget>(el);
        let window = &*self.window;

        let client_x = Finite::wrap(point.x as f64);
        let client_y = Finite::wrap(point.y as f64);
        let page_x = Finite::wrap(point.x as f64 + window.PageXOffset() as f64);
        let page_y = Finite::wrap(point.y as f64 + window.PageYOffset() as f64);

        let touch = Touch::new(window,
                               identifier,
                               target.r(),
                               client_x,
                               client_y, // TODO: Get real screen coordinates?
                               client_x,
                               client_y,
                               page_x,
                               page_y);

        match event_type {
            TouchEventType::Down => {
                // Add a new touch point
                self.active_touch_points.borrow_mut().push(JS::from_rooted(&touch));
            }
            TouchEventType::Move => {
                // Replace an existing touch point
                let mut active_touch_points = self.active_touch_points.borrow_mut();
                match active_touch_points.iter_mut().find(|t| t.Identifier() == identifier) {
                    Some(t) => *t = JS::from_rooted(&touch),
                    None => warn!("Got a touchmove event for a non-active touch point"),
                }
            }
            TouchEventType::Up |
            TouchEventType::Cancel => {
                // Remove an existing touch point
                let mut active_touch_points = self.active_touch_points.borrow_mut();
                match active_touch_points.iter().position(|t| t.Identifier() == identifier) {
                    Some(i) => {
                        active_touch_points.swap_remove(i);
                    }
                    None => warn!("Got a touchend event for a non-active touch point"),
                }
            }
        }

        let mut touches = RootedVec::new();
        touches.extend(self.active_touch_points.borrow().iter().cloned());

        let mut changed_touches = RootedVec::new();
        changed_touches.push(JS::from_rooted(&touch));

        let mut target_touches = RootedVec::new();
        target_touches.extend(self.active_touch_points
                                  .borrow()
                                  .iter()
                                  .filter(|t| t.Target() == target)
                                  .cloned());

        let event = TouchEvent::new(window,
                                    DOMString::from(event_name),
                                    EventBubbles::Bubbles,
                                    EventCancelable::Cancelable,
                                    Some(window),
                                    0i32,
                                    &TouchList::new(window, touches.r()),
                                    &TouchList::new(window, changed_touches.r()),
                                    &TouchList::new(window, target_touches.r()),
                                    // FIXME: modifier keys
                                    false,
                                    false,
                                    false,
                                    false);
        let event = event.upcast::<Event>();
        let result = event.fire(target.r());

        window.reflow(ReflowGoal::ForDisplay,
                      ReflowQueryType::NoQuery,
                      ReflowReason::MouseEvent);
        result
    }

    /// The entry point for all key processing for web content
    pub fn dispatch_key_event(&self,
                              key: Key,
                              state: KeyState,
                              modifiers: KeyModifiers,
                              compositor: &mut IpcSender<ScriptToCompositorMsg>) {
        let focused = self.get_focused_element();
        let body = self.GetBody();

        let target = match (&focused, &body) {
            (&Some(ref focused), _) => focused.upcast(),
            (&None, &Some(ref body)) => body.upcast(),
            (&None, &None) => self.window.upcast(),
        };

        let ctrl = modifiers.contains(CONTROL);
        let alt = modifiers.contains(ALT);
        let shift = modifiers.contains(SHIFT);
        let meta = modifiers.contains(SUPER);

        let is_composing = false;
        let is_repeating = state == KeyState::Repeated;
        let ev_type = DOMString::from(match state {
                                          KeyState::Pressed | KeyState::Repeated => "keydown",
                                          KeyState::Released => "keyup",
                                      }
                                      .to_owned());

        let props = KeyboardEvent::key_properties(key, modifiers);

        let keyevent = KeyboardEvent::new(&self.window,
                                          ev_type,
                                          true,
                                          true,
                                          Some(&self.window),
                                          0,
                                          Some(key),
                                          DOMString::from(props.key_string),
                                          DOMString::from(props.code),
                                          props.location,
                                          is_repeating,
                                          is_composing,
                                          ctrl,
                                          alt,
                                          shift,
                                          meta,
                                          None,
                                          props.key_code);
        let event = keyevent.upcast::<Event>();
        event.fire(target);
        let mut prevented = event.DefaultPrevented();

        // https://dvcs.w3.org/hg/dom3events/raw-file/tip/html/DOM3-Events.html#keys-cancelable-keys
        if state != KeyState::Released && props.is_printable() && !prevented {
            // https://dvcs.w3.org/hg/dom3events/raw-file/tip/html/DOM3-Events.html#keypress-event-order
            let event = KeyboardEvent::new(&self.window,
                                           DOMString::from("keypress"),
                                           true,
                                           true,
                                           Some(&self.window),
                                           0,
                                           Some(key),
                                           DOMString::from(props.key_string),
                                           DOMString::from(props.code),
                                           props.location,
                                           is_repeating,
                                           is_composing,
                                           ctrl,
                                           alt,
                                           shift,
                                           meta,
                                           props.char_code,
                                           0);
            let ev = event.upcast::<Event>();
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
                let maybe_elem = target.downcast::<Element>();
                if let Some(el) = maybe_elem {
                    if let Some(a) = el.as_maybe_activatable() {
                        a.synthetic_click_activation(ctrl, alt, shift, meta);
                    }
                }
            }
            Key::Enter if !prevented && state == KeyState::Released => {
                let maybe_elem = target.downcast::<Element>();
                if let Some(el) = maybe_elem {
                    if let Some(a) = el.as_maybe_activatable() {
                        a.implicit_submission(ctrl, alt, shift, meta);
                    }
                }
            }
            _ => (),
        }

        self.window.reflow(ReflowGoal::ForDisplay,
                           ReflowQueryType::NoQuery,
                           ReflowReason::KeyEvent);
    }

    // https://dom.spec.whatwg.org/#converting-nodes-into-a-node
    pub fn node_from_nodes_and_strings(&self,
                                       mut nodes: Vec<NodeOrString>)
                                       -> Fallible<Root<Node>> {
        if nodes.len() == 1 {
            Ok(match nodes.pop().unwrap() {
                NodeOrString::eNode(node) => node,
                NodeOrString::eString(string) => Root::upcast(self.CreateTextNode(string)),
            })
        } else {
            let fragment = Root::upcast::<Node>(self.CreateDocumentFragment());
            for node in nodes {
                match node {
                    NodeOrString::eNode(node) => {
                        try!(fragment.AppendChild(node.r()));
                    },
                    NodeOrString::eString(string) => {
                        let node = Root::upcast::<Node>(self.CreateTextNode(string));
                        // No try!() here because appending a text node
                        // should not fail.
                        fragment.AppendChild(node.r()).unwrap();
                    }
                }
            }
            Ok(fragment)
        }
    }

    pub fn get_body_attribute(&self, local_name: &Atom) -> DOMString {
        match self.GetBody().and_then(Root::downcast::<HTMLBodyElement>) {
            Some(ref body) => {
                body.upcast::<Element>().get_string_attribute(local_name)
            },
            None => DOMString::new(),
        }
    }

    pub fn set_body_attribute(&self, local_name: &Atom, value: DOMString) {
        if let Some(ref body) = self.GetBody().and_then(Root::downcast::<HTMLBodyElement>) {
            let body = body.upcast::<Element>();
            let value = body.parse_attribute(&ns!(""), &local_name, value);
            body.set_attribute(local_name, value);
        }
    }

    pub fn set_current_script(&self, script: Option<&HTMLScriptElement>) {
        self.current_script.set(script);
    }

    pub fn get_script_blocking_stylesheets_count(&self) -> u32 {
        self.script_blocking_stylesheets_count.get()
    }

    pub fn increment_script_blocking_stylesheet_count(&self) {
        let count_cell = &self.script_blocking_stylesheets_count;
        count_cell.set(count_cell.get() + 1);
    }

    pub fn decrement_script_blocking_stylesheet_count(&self) {
        let count_cell = &self.script_blocking_stylesheets_count;
        assert!(count_cell.get() > 0);
        count_cell.set(count_cell.get() - 1);
    }

    pub fn invalidate_stylesheets(&self) {
        self.stylesheets_changed_since_reflow.set(true);
        *self.stylesheets.borrow_mut() = None;
        // Mark the document element dirty so a reflow will be performed.
        self.get_html_element().map(|root| {
            root.upcast::<Node>().dirty(NodeDamage::NodeStyleDamaged);
        });
    }

    pub fn get_and_reset_stylesheets_changed_since_reflow(&self) -> bool {
        let changed = self.stylesheets_changed_since_reflow.get();
        self.stylesheets_changed_since_reflow.set(false);
        changed
    }

    pub fn set_pending_parsing_blocking_script(&self, script: Option<&HTMLScriptElement>) {
        assert!(self.get_pending_parsing_blocking_script().is_none() || script.is_none());
        self.pending_parsing_blocking_script.set(script);
    }

    pub fn get_pending_parsing_blocking_script(&self) -> Option<Root<HTMLScriptElement>> {
        self.pending_parsing_blocking_script.get()
    }

    pub fn add_deferred_script(&self, script: &HTMLScriptElement) {
        self.deferred_scripts.borrow_mut().push(JS::from_ref(script));
    }

    pub fn add_asap_script(&self, script: &HTMLScriptElement) {
        self.asap_scripts_set.borrow_mut().push(JS::from_ref(script));
    }

    pub fn push_asap_in_order_script(&self, script: &HTMLScriptElement) {
        self.asap_in_order_scripts_list.borrow_mut().push(JS::from_ref(script));
    }

    pub fn trigger_mozbrowser_event(&self, event: MozBrowserEvent) {
        if htmliframeelement::mozbrowser_enabled() {
            if let Some((containing_pipeline_id, subpage_id)) = self.window.parent_info() {
                let ConstellationChan(ref chan) = self.window.constellation_chan();
                let event = ConstellationMsg::MozBrowserEvent(containing_pipeline_id,
                                                              subpage_id,
                                                              event);
                chan.send(event).unwrap();
            }
        }
    }

    /// https://html.spec.whatwg.org/multipage/#dom-window-requestanimationframe
    pub fn request_animation_frame(&self, callback: Box<FnBox(f64)>) -> u32 {
        let ident = self.animation_frame_ident.get() + 1;

        self.animation_frame_ident.set(ident);
        self.animation_frame_list.borrow_mut().insert(ident, callback);

        // TODO: Should tick animation only when document is visible
        let ConstellationChan(ref chan) = self.window.constellation_chan();
        let event = ConstellationMsg::ChangeRunningAnimationsState(self.window.pipeline(),
                                                                   AnimationState::AnimationCallbacksPresent);
        chan.send(event).unwrap();

        ident
    }

    /// https://html.spec.whatwg.org/multipage/#dom-window-cancelanimationframe
    pub fn cancel_animation_frame(&self, ident: u32) {
        self.animation_frame_list.borrow_mut().remove(&ident);
        if self.animation_frame_list.borrow().is_empty() {
            let ConstellationChan(ref chan) = self.window.constellation_chan();
            let event = ConstellationMsg::ChangeRunningAnimationsState(self.window.pipeline(),
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

            let ConstellationChan(ref chan) = self.window.constellation_chan();
            let event = ConstellationMsg::ChangeRunningAnimationsState(self.window.pipeline(),
                                                                       AnimationState::NoAnimationCallbacksPresent);
            chan.send(event).unwrap();
        }
        let performance = self.window.Performance();
        let performance = performance.r();
        let timing = performance.Now();

        for (_, callback) in animation_frame_list {
            callback(*timing);
        }

        self.window.reflow(ReflowGoal::ForDisplay,
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

    pub fn finish_load(&self, load: LoadType) {
        // The parser might need the loader, so restrict the lifetime of the borrow.
        {
            let mut loader = self.loader.borrow_mut();
            loader.finish_load(load.clone());
        }

        if let LoadType::Script(_) = load {
            self.process_deferred_scripts();
            self.process_asap_scripts();
        }

        if self.maybe_execute_parser_blocking_script() == ParserBlockedByScript::Blocked {
            return;
        }

        // A finished resource load can potentially unblock parsing. In that case, resume the
        // parser so its loop can find out.
        if let Some(parser) = self.current_parser.get() {
            if parser.is_suspended() {
                parser.resume();
            }
        } else if self.reflow_timeout.get().is_none() {
            // If we don't have a parser, and the reflow timer has been reset, explicitly
            // trigger a reflow.
            if let LoadType::Stylesheet(_) = load {
                self.window().reflow(ReflowGoal::ForDisplay,
                                     ReflowQueryType::NoQuery,
                                     ReflowReason::StylesheetLoaded);
            }
        }

        let loader = self.loader.borrow();
        if !loader.is_blocked() && !loader.events_inhibited() {
            let win = self.window();
            let msg = MainThreadScriptMsg::DocumentLoadsComplete(win.pipeline());
            win.main_thread_script_chan().send(msg).unwrap();
        }
    }

    /// If document parsing is blocked on a script, and that script is ready to run,
    /// execute it.
    /// https://html.spec.whatwg.org/multipage/#ready-to-be-parser-executed
    fn maybe_execute_parser_blocking_script(&self) -> ParserBlockedByScript {
        let script = match self.pending_parsing_blocking_script.get() {
            None => return ParserBlockedByScript::Unblocked,
            Some(script) => script,
        };

        if self.script_blocking_stylesheets_count.get() == 0 && script.is_ready_to_be_executed() {
            script.execute();
            self.pending_parsing_blocking_script.set(None);
            return ParserBlockedByScript::Unblocked;
        }
        ParserBlockedByScript::Blocked
    }

    /// https://html.spec.whatwg.org/multipage/#the-end step 3
    pub fn process_deferred_scripts(&self) {
        if self.ready_state.get() != DocumentReadyState::Interactive {
            return;
        }
        // Part of substep 1.
        if self.script_blocking_stylesheets_count.get() > 0 {
            return;
        }
        let mut deferred_scripts = self.deferred_scripts.borrow_mut();
        while !deferred_scripts.is_empty() {
            {
                let script = &*deferred_scripts[0];
                // Part of substep 1.
                if !script.is_ready_to_be_executed() {
                    return;
                }
                // Substep 2.
                script.execute();
            }
            // Substep 3.
            deferred_scripts.remove(0);
            // Substep 4 (implicit).
        }
        // https://html.spec.whatwg.org/multipage/#the-end step 4.
        self.maybe_dispatch_dom_content_loaded();
    }

    /// https://html.spec.whatwg.org/multipage/#the-end step 5 and the latter parts of
    /// https://html.spec.whatwg.org/multipage/#prepare-a-script 15.d and 15.e.
    pub fn process_asap_scripts(&self) {
        // Execute the first in-order asap-executed script if it's ready, repeat as required.
        // Re-borrowing the list for each step because it can also be borrowed under execute.
        while self.asap_in_order_scripts_list.borrow().len() > 0 {
            let script = Root::from_ref(&*self.asap_in_order_scripts_list.borrow()[0]);
            if !script.is_ready_to_be_executed() {
                break;
            }
            script.execute();
            self.asap_in_order_scripts_list.borrow_mut().remove(0);
        }

        let mut idx = 0;
        // Re-borrowing the set for each step because it can also be borrowed under execute.
        while idx < self.asap_scripts_set.borrow().len() {
            let script = Root::from_ref(&*self.asap_scripts_set.borrow()[idx]);
            if !script.is_ready_to_be_executed() {
                idx += 1;
                continue;
            }
            script.execute();
            self.asap_scripts_set.borrow_mut().swap_remove(idx);
        }
    }

    pub fn maybe_dispatch_dom_content_loaded(&self) {
        if self.domcontentloaded_dispatched.get() {
            return;
        }
        self.domcontentloaded_dispatched.set(true);

        update_with_current_time(&self.dom_content_loaded_event_start);

        let event = Event::new(GlobalRef::Window(self.window()),
                               DOMString::from("DOMContentLoaded"),
                               EventBubbles::DoesNotBubble,
                               EventCancelable::NotCancelable);
        let doctarget = self.upcast::<EventTarget>();
        let _ = doctarget.DispatchEvent(event.r());
        self.window().reflow(ReflowGoal::ForDisplay,
                             ReflowQueryType::NoQuery,
                             ReflowReason::DOMContentLoaded);

        update_with_current_time(&self.dom_content_loaded_event_end);
    }

    pub fn notify_constellation_load(&self) {
        let pipeline_id = self.window.pipeline();
        let ConstellationChan(ref chan) = self.window.constellation_chan();
        let event = ConstellationMsg::DOMLoad(pipeline_id);
        chan.send(event).unwrap();

    }

    pub fn set_current_parser(&self, script: Option<&ServoHTMLParser>) {
        self.current_parser.set(script);
    }

    pub fn get_current_parser(&self) -> Option<Root<ServoHTMLParser>> {
        self.current_parser.get()
    }

    /// Find an iframe element in the document.
    pub fn find_iframe(&self, subpage_id: SubpageId) -> Option<Root<HTMLIFrameElement>> {
        self.upcast::<Node>()
            .traverse_preorder()
            .filter_map(Root::downcast::<HTMLIFrameElement>)
            .find(|node| node.subpage_id() == Some(subpage_id))
    }

    pub fn get_dom_loading(&self) -> u64 {
        self.dom_loading.get()
    }

    pub fn get_dom_interactive(&self) -> u64 {
        self.dom_interactive.get()
    }

    pub fn get_dom_content_loaded_event_start(&self) -> u64 {
        self.dom_content_loaded_event_start.get()
    }

    pub fn get_dom_content_loaded_event_end(&self) -> u64 {
        self.dom_content_loaded_event_end.get()
    }

    pub fn get_dom_complete(&self) -> u64 {
        self.dom_complete.get()
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

#[allow(unsafe_code)]
pub trait LayoutDocumentHelpers {
    unsafe fn is_html_document_for_layout(&self) -> bool;
    unsafe fn drain_modified_elements(&self) -> Vec<(LayoutJS<Element>, ElementSnapshot)>;
}

#[allow(unsafe_code)]
impl LayoutDocumentHelpers for LayoutJS<Document> {
    #[inline]
    unsafe fn is_html_document_for_layout(&self) -> bool {
        (*self.unsafe_get()).is_html_document
    }

    #[inline]
    #[allow(unrooted_must_root)]
    unsafe fn drain_modified_elements(&self) -> Vec<(LayoutJS<Element>, ElementSnapshot)> {
        let mut elements = (*self.unsafe_get()).modified_elements.borrow_mut_for_layout();
        let drain = elements.drain();
        let layout_drain = drain.map(|(k, v)| (k.to_layout(), v));
        Vec::from_iter(layout_drain)
    }
}

impl Document {
    fn new_inherited(window: &Window,
                     url: Option<Url>,
                     is_html_document: IsHTMLDocument,
                     content_type: Option<DOMString>,
                     last_modified: Option<String>,
                     source: DocumentSource,
                     doc_loader: DocumentLoader)
                     -> Document {
        let url = url.unwrap_or_else(|| Url::parse("about:blank").unwrap());

        let (ready_state, domcontentloaded_dispatched) = if source == DocumentSource::FromParser {
            (DocumentReadyState::Loading, false)
        } else {
            (DocumentReadyState::Complete, true)
        };

        Document {
            node: Node::new_document_node(),
            window: JS::from_ref(window),
            implementation: Default::default(),
            location: Default::default(),
            content_type: match content_type {
                Some(string) => string,
                None => DOMString::from(match is_html_document {
                    // https://dom.spec.whatwg.org/#dom-domimplementation-createhtmldocument
                    IsHTMLDocument::HTMLDocument => "text/html",
                    // https://dom.spec.whatwg.org/#concept-document-content-type
                    IsHTMLDocument::NonHTMLDocument => "application/xml",
                }),
            },
            last_modified: last_modified,
            url: url,
            // https://dom.spec.whatwg.org/#concept-document-quirks
            quirks_mode: Cell::new(NoQuirks),
            // https://dom.spec.whatwg.org/#concept-document-encoding
            encoding_name: DOMRefCell::new(DOMString::from("UTF-8")),
            is_html_document: is_html_document == IsHTMLDocument::HTMLDocument,
            id_map: DOMRefCell::new(HashMap::new()),
            tag_map: DOMRefCell::new(HashMap::new()),
            tagns_map: DOMRefCell::new(HashMap::new()),
            classes_map: DOMRefCell::new(HashMap::new()),
            images: Default::default(),
            embeds: Default::default(),
            links: Default::default(),
            forms: Default::default(),
            scripts: Default::default(),
            anchors: Default::default(),
            applets: Default::default(),
            stylesheets: DOMRefCell::new(None),
            stylesheets_changed_since_reflow: Cell::new(false),
            ready_state: Cell::new(ready_state),
            domcontentloaded_dispatched: Cell::new(domcontentloaded_dispatched),
            possibly_focused: Default::default(),
            focused: Default::default(),
            current_script: Default::default(),
            pending_parsing_blocking_script: Default::default(),
            script_blocking_stylesheets_count: Cell::new(0u32),
            deferred_scripts: DOMRefCell::new(vec![]),
            asap_in_order_scripts_list: DOMRefCell::new(vec![]),
            asap_scripts_set: DOMRefCell::new(vec![]),
            scripting_enabled: Cell::new(true),
            animation_frame_ident: Cell::new(0),
            animation_frame_list: DOMRefCell::new(HashMap::new()),
            loader: DOMRefCell::new(doc_loader),
            current_parser: Default::default(),
            reflow_timeout: Cell::new(None),
            base_element: Default::default(),
            appropriate_template_contents_owner_document: Default::default(),
            modified_elements: DOMRefCell::new(HashMap::new()),
            active_touch_points: DOMRefCell::new(Vec::new()),
            dom_loading: Cell::new(Default::default()),
            dom_interactive: Cell::new(Default::default()),
            dom_content_loaded_event_start: Cell::new(Default::default()),
            dom_content_loaded_event_end: Cell::new(Default::default()),
            dom_complete: Cell::new(Default::default()),
        }
    }

    // https://dom.spec.whatwg.org/#dom-document
    pub fn Constructor(global: GlobalRef) -> Fallible<Root<Document>> {
        let win = global.as_window();
        let doc = win.Document();
        let doc = doc.r();
        let docloader = DocumentLoader::new(&*doc.loader());
        Ok(Document::new(win,
                         None,
                         IsHTMLDocument::NonHTMLDocument,
                         None,
                         None,
                         DocumentSource::NotFromParser,
                         docloader))
    }

    pub fn new(window: &Window,
               url: Option<Url>,
               doctype: IsHTMLDocument,
               content_type: Option<DOMString>,
               last_modified: Option<String>,
               source: DocumentSource,
               doc_loader: DocumentLoader)
               -> Root<Document> {
        let document = reflect_dom_object(box Document::new_inherited(window,
                                                                      url,
                                                                      doctype,
                                                                      content_type,
                                                                      last_modified,
                                                                      source,
                                                                      doc_loader),
                                          GlobalRef::Window(window),
                                          DocumentBinding::Wrap);
        {
            let node = document.upcast::<Node>();
            node.set_owner_doc(document.r());
        }
        document
    }

    fn create_node_list<F: Fn(&Node) -> bool>(&self, callback: F) -> Root<NodeList> {
        let doc = self.GetDocumentElement();
        let maybe_node = doc.r().map(Castable::upcast::<Node>);
        let iter = maybe_node.iter()
                             .flat_map(|node| node.traverse_preorder())
                             .filter(|node| callback(node.r()));
        NodeList::new_simple_list(&self.window, iter)
    }

    fn get_html_element(&self) -> Option<Root<HTMLHtmlElement>> {
        self.GetDocumentElement().and_then(Root::downcast)
    }

    /// Returns the list of stylesheets associated with nodes in the document.
    pub fn stylesheets(&self) -> Ref<Vec<Arc<Stylesheet>>> {
        {
            let mut stylesheets = self.stylesheets.borrow_mut();
            if stylesheets.is_none() {
                let new_stylesheets: Vec<Arc<Stylesheet>> = self.upcast::<Node>()
                    .traverse_preorder()
                    .filter_map(|node| {
                        if let Some(node) = node.downcast::<HTMLStyleElement>() {
                            node.get_stylesheet()
                        } else if let Some(node) = node.downcast::<HTMLLinkElement>() {
                            node.get_stylesheet()
                        } else if let Some(node) = node.downcast::<HTMLMetaElement>() {
                            node.get_stylesheet()
                        } else {
                            None
                        }
                    })
                    .collect();
                *stylesheets = Some(new_stylesheets);
            };
        }
        Ref::map(self.stylesheets.borrow(), |t| t.as_ref().unwrap())
    }

    /// https://html.spec.whatwg.org/multipage/#appropriate-template-contents-owner-document
    pub fn appropriate_template_contents_owner_document(&self) -> Root<Document> {
        self.appropriate_template_contents_owner_document.or_init(|| {
            let doctype = if self.is_html_document {
                IsHTMLDocument::HTMLDocument
            } else {
                IsHTMLDocument::NonHTMLDocument
            };
            let new_doc = Document::new(self.window(),
                                        None,
                                        doctype,
                                        None,
                                        None,
                                        DocumentSource::NotFromParser,
                                        DocumentLoader::new(&self.loader()));
            new_doc.appropriate_template_contents_owner_document.set(Some(&new_doc));
            new_doc
        })
    }

    pub fn get_element_by_id(&self, id: &Atom) -> Option<Root<Element>> {
        self.id_map.borrow().get(&id).map(|ref elements| Root::from_ref(&*(*elements)[0]))
    }

    pub fn element_state_will_change(&self, el: &Element) {
        let mut map = self.modified_elements.borrow_mut();
        let snapshot = map.entry(JS::from_ref(el)).or_insert(ElementSnapshot::new());
        if snapshot.state.is_none() {
            snapshot.state = Some(el.get_state());
        }
    }

    pub fn element_attr_will_change(&self, el: &Element) {
        let mut map = self.modified_elements.borrow_mut();
        let mut snapshot = map.entry(JS::from_ref(el)).or_insert(ElementSnapshot::new());
        if snapshot.attrs.is_none() {
            let attrs = el.attrs()
                          .iter()
                          .map(|attr| (attr.identifier().clone(), attr.value().clone()))
                          .collect();
            snapshot.attrs = Some(attrs);
        }
    }
}


impl Element {
    fn click_event_filter_by_disabled_state(&self) -> bool {
        let node = self.upcast::<Node>();
        match node.type_id() {
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLButtonElement)) |
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLInputElement)) |
            // NodeTypeId::Element(ElementTypeId::HTMLKeygenElement) |
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLOptionElement)) |
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLSelectElement)) |
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTextAreaElement))
                if self.get_disabled_state() => true,
            _ => false,
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
        DOMString::from(self.url().serialize())
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-activeelement
    fn GetActiveElement(&self) -> Option<Root<Element>> {
        // TODO: Step 2.

        match self.get_focused_element() {
            Some(element) => Some(element),     // Step 3. and 4.
            None => match self.GetBody() {      // Step 5.
                Some(body) => Some(Root::upcast(body)),
                None => self.GetDocumentElement(),
            },
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-hasfocus
    fn HasFocus(&self) -> bool {
        let target = self;                                                        // Step 1.
        let browsing_context = self.window.browsing_context();
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
            None => false,
        }
    }

    // https://dom.spec.whatwg.org/#dom-document-documenturi
    fn DocumentURI(&self) -> DOMString {
        self.URL()
    }

    // https://dom.spec.whatwg.org/#dom-document-compatmode
    fn CompatMode(&self) -> DOMString {
        DOMString::from(match self.quirks_mode.get() {
            LimitedQuirks | NoQuirks => "CSS1Compat",
            Quirks => "BackCompat",
        })
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
        self.upcast::<Node>().children().filter_map(Root::downcast).next()
    }

    // https://dom.spec.whatwg.org/#dom-document-documentelement
    fn GetDocumentElement(&self) -> Option<Root<Element>> {
        self.upcast::<Node>().child_elements().next()
    }

    // https://dom.spec.whatwg.org/#dom-document-getelementsbytagname
    fn GetElementsByTagName(&self, tag_name: DOMString) -> Root<HTMLCollection> {
        let tag_atom = Atom::from_slice(&tag_name);
        match self.tag_map.borrow_mut().entry(tag_atom.clone()) {
            Occupied(entry) => Root::from_ref(entry.get()),
            Vacant(entry) => {
                let mut tag_copy = tag_name;
                tag_copy.make_ascii_lowercase();
                let ascii_lower_tag = Atom::from_slice(&tag_copy);
                let result = HTMLCollection::by_atomic_tag_name(&self.window,
                                                                self.upcast(),
                                                                tag_atom,
                                                                ascii_lower_tag);
                entry.insert(JS::from_rooted(&result));
                result
            }
        }
    }

    // https://dom.spec.whatwg.org/#dom-document-getelementsbytagnamens
    fn GetElementsByTagNameNS(&self,
                              maybe_ns: Option<DOMString>,
                              tag_name: DOMString)
                              -> Root<HTMLCollection> {
        let ns = namespace_from_domstring(maybe_ns);
        let local = Atom::from_slice(&tag_name);
        let qname = QualName::new(ns, local);
        match self.tagns_map.borrow_mut().entry(qname.clone()) {
            Occupied(entry) => Root::from_ref(entry.get()),
            Vacant(entry) => {
                let result = HTMLCollection::by_qual_tag_name(&self.window, self.upcast(), qname);
                entry.insert(JS::from_rooted(&result));
                result
            }
        }
    }

    // https://dom.spec.whatwg.org/#dom-document-getelementsbyclassname
    fn GetElementsByClassName(&self, classes: DOMString) -> Root<HTMLCollection> {
        let class_atoms: Vec<Atom> = split_html_space_chars(&classes)
                                         .map(Atom::from_slice)
                                         .collect();
        match self.classes_map.borrow_mut().entry(class_atoms.clone()) {
            Occupied(entry) => Root::from_ref(entry.get()),
            Vacant(entry) => {
                let result = HTMLCollection::by_atomic_class_name(&self.window,
                                                                  self.upcast(),
                                                                  class_atoms);
                entry.insert(JS::from_rooted(&result));
                result
            }
        }
    }

    // https://dom.spec.whatwg.org/#dom-nonelementparentnode-getelementbyid
    fn GetElementById(&self, id: DOMString) -> Option<Root<Element>> {
        self.get_element_by_id(&Atom::from_slice(&id))
    }

    // https://dom.spec.whatwg.org/#dom-document-createelement
    fn CreateElement(&self, mut local_name: DOMString) -> Fallible<Root<Element>> {
        if xml_name_type(&local_name) == InvalidXMLName {
            debug!("Not a valid element name");
            return Err(Error::InvalidCharacter);
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
                       qualified_name: DOMString)
                       -> Fallible<Root<Element>> {
        let (namespace, prefix, local_name) = try!(validate_and_extract(namespace,
                                                                        &qualified_name));
        let name = QualName::new(namespace, local_name);
        Ok(Element::create(name, prefix, self, ElementCreator::ScriptCreated))
    }

    // https://dom.spec.whatwg.org/#dom-document-createattribute
    fn CreateAttribute(&self, local_name: DOMString) -> Fallible<Root<Attr>> {
        if xml_name_type(&local_name) == InvalidXMLName {
            debug!("Not a valid element name");
            return Err(Error::InvalidCharacter);
        }

        let name = Atom::from_slice(&local_name);
        // repetition used because string_cache::atom::Atom is non-copyable
        let l_name = Atom::from_slice(&local_name);
        let value = AttrValue::String(DOMString::new());

        Ok(Attr::new(&self.window, name, value, l_name, ns!(""), None, None))
    }

    // https://dom.spec.whatwg.org/#dom-document-createattributens
    fn CreateAttributeNS(&self,
                         namespace: Option<DOMString>,
                         qualified_name: DOMString)
                         -> Fallible<Root<Attr>> {
        let (namespace, prefix, local_name) = try!(validate_and_extract(namespace,
                                                                        &qualified_name));
        let value = AttrValue::String(DOMString::new());
        let qualified_name = Atom::from_slice(&qualified_name);
        Ok(Attr::new(&self.window,
                     local_name,
                     value,
                     qualified_name,
                     namespace,
                     prefix,
                     None))
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
    fn CreateProcessingInstruction(&self,
                                   target: DOMString,
                                   data: DOMString)
                                   -> Fallible<Root<ProcessingInstruction>> {
        // Step 1.
        if xml_name_type(&target) == InvalidXMLName {
            return Err(Error::InvalidCharacter);
        }

        // Step 2.
        if data.contains("?>") {
            return Err(Error::InvalidCharacter);
        }

        // Step 3.
        Ok(ProcessingInstruction::new(target, data, self))
    }

    // https://dom.spec.whatwg.org/#dom-document-importnode
    fn ImportNode(&self, node: &Node, deep: bool) -> Fallible<Root<Node>> {
        // Step 1.
        if node.is::<Document>() {
            return Err(Error::NotSupported);
        }

        // Step 2.
        let clone_children = match deep {
            true => CloneChildrenFlag::CloneChildren,
            false => CloneChildrenFlag::DoNotCloneChildren,
        };

        Ok(Node::clone(node, Some(self), clone_children))
    }

    // https://dom.spec.whatwg.org/#dom-document-adoptnode
    fn AdoptNode(&self, node: &Node) -> Fallible<Root<Node>> {
        // Step 1.
        if node.is::<Document>() {
            return Err(Error::NotSupported);
        }

        // Step 2.
        Node::adopt(node, self);

        // Step 3.
        Ok(Root::from_ref(node))
    }

    // https://dom.spec.whatwg.org/#dom-document-createevent
    fn CreateEvent(&self, mut interface: DOMString) -> Fallible<Root<Event>> {
        interface.make_ascii_lowercase();
        match &*interface {
            "uievents" | "uievent" =>
                Ok(Root::upcast(UIEvent::new_uninitialized(&self.window))),
            "mouseevents" | "mouseevent" =>
                Ok(Root::upcast(MouseEvent::new_uninitialized(&self.window))),
            "customevent" =>
                Ok(Root::upcast(CustomEvent::new_uninitialized(GlobalRef::Window(&self.window)))),
            "htmlevents" | "events" | "event" =>
                Ok(Event::new_uninitialized(GlobalRef::Window(&self.window))),
            "keyboardevent" | "keyevents" =>
                Ok(Root::upcast(KeyboardEvent::new_uninitialized(&self.window))),
            "messageevent" =>
                Ok(Root::upcast(MessageEvent::new_uninitialized(GlobalRef::Window(&self.window)))),
            "touchevent" =>
                Ok(Root::upcast(
                    TouchEvent::new_uninitialized(&self.window,
                        &TouchList::new(&self.window, &[]),
                        &TouchList::new(&self.window, &[]),
                        &TouchList::new(&self.window, &[]),
                    )
                )),
            _ =>
                Err(Error::NotSupported),
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-lastmodified
    fn LastModified(&self) -> DOMString {
        match self.last_modified {
            Some(ref t) => DOMString::from(t.clone()),
            None => DOMString::from(time::now().strftime("%m/%d/%Y %H:%M:%S").unwrap().to_string()),
        }
    }

    // https://dom.spec.whatwg.org/#dom-document-createrange
    fn CreateRange(&self) -> Root<Range> {
        Range::new_with_doc(self)
    }

    // https://dom.spec.whatwg.org/#dom-document-createnodeiteratorroot-whattoshow-filter
    fn CreateNodeIterator(&self,
                          root: &Node,
                          whatToShow: u32,
                          filter: Option<Rc<NodeFilter>>)
                          -> Root<NodeIterator> {
        NodeIterator::new(self, root, whatToShow, filter)
    }

    // https://w3c.github.io/touch-events/#idl-def-Document
    fn CreateTouch(&self,
                   window: &Window,
                   target: &EventTarget,
                   identifier: i32,
                   pageX: Finite<f64>,
                   pageY: Finite<f64>,
                   screenX: Finite<f64>,
                   screenY: Finite<f64>)
                   -> Root<Touch> {
        let clientX = Finite::wrap(*pageX - window.PageXOffset() as f64);
        let clientY = Finite::wrap(*pageY - window.PageYOffset() as f64);
        Touch::new(window,
                   identifier,
                   target,
                   screenX,
                   screenY,
                   clientX,
                   clientY,
                   pageX,
                   pageY)
    }

    // https://dom.spec.whatwg.org/#dom-document-createtreewalker
    fn CreateTreeWalker(&self,
                        root: &Node,
                        whatToShow: u32,
                        filter: Option<Rc<NodeFilter>>)
                        -> Root<TreeWalker> {
        TreeWalker::new(self, root, whatToShow, filter)
    }

    // https://html.spec.whatwg.org/multipage/#document.title
    fn Title(&self) -> DOMString {
        let title = self.GetDocumentElement().and_then(|root| {
            if root.namespace() == &ns!(SVG) && root.local_name() == &atom!("svg") {
                // Step 1.
                root.upcast::<Node>()
                    .child_elements()
                    .find(|node| {
                        node.namespace() == &ns!(SVG) && node.local_name() == &atom!("title")
                    })
                    .map(Root::upcast::<Node>)
            } else {
                // Step 2.
                root.upcast::<Node>()
                    .traverse_preorder()
                    .find(|node| node.is::<HTMLTitleElement>())
            }
        });

        match title {
            None => DOMString::new(),
            Some(ref title) => {
                // Steps 3-4.
                let value = Node::collect_text_contents(title.children());
                DOMString::from(str_join(split_html_space_chars(&value), " "))
            },
        }
    }

    // https://html.spec.whatwg.org/multipage/#document.title
    fn SetTitle(&self, title: DOMString) {
        let root = match self.GetDocumentElement() {
            Some(root) => root,
            None => return,
        };

        let elem = if root.namespace() == &ns!(SVG) && root.local_name() == &atom!("svg") {
            let elem = root.upcast::<Node>().child_elements().find(|node| {
                node.namespace() == &ns!(SVG) && node.local_name() == &atom!("title")
            });
            match elem {
                Some(elem) => Root::upcast::<Node>(elem),
                None => {
                    let name = QualName::new(ns!(SVG), atom!("title"));
                    let elem = Element::create(name, None, self, ElementCreator::ScriptCreated);
                    let parent = root.upcast::<Node>();
                    let child = elem.upcast::<Node>();
                    parent.InsertBefore(child, parent.GetFirstChild().r())
                          .unwrap()
                }
            }
        } else if root.namespace() == &ns!(HTML) {
            let elem = root.upcast::<Node>()
                           .traverse_preorder()
                           .find(|node| node.is::<HTMLTitleElement>());
            match elem {
                Some(elem) => elem,
                None => {
                    match self.GetHead() {
                        Some(head) => {
                            let name = QualName::new(ns!(HTML), atom!("title"));
                            let elem = Element::create(name,
                                                       None,
                                                       self,
                                                       ElementCreator::ScriptCreated);
                            head.upcast::<Node>()
                                .AppendChild(elem.upcast())
                                .unwrap()
                        },
                        None => return,
                    }
                }
            }
        } else {
            return;
        };

        elem.SetTextContent(Some(title));
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-head
    fn GetHead(&self) -> Option<Root<HTMLHeadElement>> {
        self.get_html_element()
            .and_then(|root| root.upcast::<Node>().children().filter_map(Root::downcast).next())
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-currentscript
    fn GetCurrentScript(&self) -> Option<Root<HTMLScriptElement>> {
        self.current_script.get()
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-body
    fn GetBody(&self) -> Option<Root<HTMLElement>> {
        self.get_html_element().and_then(|root| {
            let node = root.upcast::<Node>();
            node.children().find(|child| {
                match child.type_id() {
                    NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLBodyElement)) |
                    NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLFrameSetElement)) => true,
                    _ => false
                }
            }).map(|node| Root::downcast(node).unwrap())
        })
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-body
    fn SetBody(&self, new_body: Option<&HTMLElement>) -> ErrorResult {
        // Step 1.
        let new_body = match new_body {
            Some(new_body) => new_body,
            None => return Err(Error::HierarchyRequest),
        };

        let node = new_body.upcast::<Node>();
        match node.type_id() {
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLBodyElement)) |
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLFrameSetElement)) => {}
            _ => return Err(Error::HierarchyRequest),
        }

        // Step 2.
        let old_body = self.GetBody();
        if old_body.as_ref().map(|body| body.r()) == Some(new_body) {
            return Ok(());
        }

        match (self.get_html_element(), &old_body) {
            // Step 3.
            (Some(ref root), &Some(ref child)) => {
                let root = root.upcast::<Node>();
                root.ReplaceChild(new_body.upcast(), child.upcast()).unwrap();
            },

            // Step 4.
            (None, _) => return Err(Error::HierarchyRequest),

            // Step 5.
            (Some(ref root), &None) => {
                let root = root.upcast::<Node>();
                root.AppendChild(new_body.upcast()).unwrap();
            }
        }
        Ok(())
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-getelementsbyname
    fn GetElementsByName(&self, name: DOMString) -> Root<NodeList> {
        self.create_node_list(|node| {
            let element = match node.downcast::<Element>() {
                Some(element) => element,
                None => return false,
            };
            if element.namespace() != &ns!(HTML) {
                return false;
            }
            element.get_attribute(&ns!(""), &atom!("name"))
                   .map_or(false, |attr| &**attr.value() == &*name)
        })
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-images
    fn Images(&self) -> Root<HTMLCollection> {
        self.images.or_init(|| {
            let filter = box ImagesFilter;
            HTMLCollection::create(&self.window, self.upcast(), filter)
        })
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-embeds
    fn Embeds(&self) -> Root<HTMLCollection> {
        self.embeds.or_init(|| {
            let filter = box EmbedsFilter;
            HTMLCollection::create(&self.window, self.upcast(), filter)
        })
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-plugins
    fn Plugins(&self) -> Root<HTMLCollection> {
        self.Embeds()
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-links
    fn Links(&self) -> Root<HTMLCollection> {
        self.links.or_init(|| {
            let filter = box LinksFilter;
            HTMLCollection::create(&self.window, self.upcast(), filter)
        })
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-forms
    fn Forms(&self) -> Root<HTMLCollection> {
        self.forms.or_init(|| {
            let filter = box FormsFilter;
            HTMLCollection::create(&self.window, self.upcast(), filter)
        })
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-scripts
    fn Scripts(&self) -> Root<HTMLCollection> {
        self.scripts.or_init(|| {
            let filter = box ScriptsFilter;
            HTMLCollection::create(&self.window, self.upcast(), filter)
        })
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-anchors
    fn Anchors(&self) -> Root<HTMLCollection> {
        self.anchors.or_init(|| {
            let filter = box AnchorsFilter;
            HTMLCollection::create(&self.window, self.upcast(), filter)
        })
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-applets
    fn Applets(&self) -> Root<HTMLCollection> {
        // FIXME: This should be return OBJECT elements containing applets.
        self.applets.or_init(|| {
            let filter = box AppletsFilter;
            HTMLCollection::create(&self.window, self.upcast(), filter)
        })
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-location
    fn Location(&self) -> Root<Location> {
        self.location.or_init(|| Location::new(&self.window))
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-children
    fn Children(&self) -> Root<HTMLCollection> {
        HTMLCollection::children(&self.window, self.upcast())
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-firstelementchild
    fn GetFirstElementChild(&self) -> Option<Root<Element>> {
        self.upcast::<Node>().child_elements().next()
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-lastelementchild
    fn GetLastElementChild(&self) -> Option<Root<Element>> {
        self.upcast::<Node>().rev_children().filter_map(Root::downcast).next()
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-childelementcount
    fn ChildElementCount(&self) -> u32 {
        self.upcast::<Node>().child_elements().count() as u32
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-prepend
    fn Prepend(&self, nodes: Vec<NodeOrString>) -> ErrorResult {
        self.upcast::<Node>().prepend(nodes)
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-append
    fn Append(&self, nodes: Vec<NodeOrString>) -> ErrorResult {
        self.upcast::<Node>().append(nodes)
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-queryselector
    fn QuerySelector(&self, selectors: DOMString) -> Fallible<Option<Root<Element>>> {
        let root = self.upcast::<Node>();
        root.query_selector(selectors)
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-queryselectorall
    fn QuerySelectorAll(&self, selectors: DOMString) -> Fallible<Root<NodeList>> {
        let root = self.upcast::<Node>();
        root.query_selector_all(selectors)
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-readystate
    fn ReadyState(&self) -> DocumentReadyState {
        self.ready_state.get()
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-defaultview
    fn DefaultView(&self) -> Root<Window> {
        Root::from_ref(&*self.window)
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-cookie
    fn GetCookie(&self) -> Fallible<DOMString> {
        // TODO: return empty string for cookie-averse Document
        let url = self.url();
        if !is_scheme_host_port_tuple(&url) {
            return Err(Error::Security);
        }
        let (tx, rx) = ipc::channel().unwrap();
        let _ = self.window.resource_task().send(GetCookiesForUrl((*url).clone(), tx, NonHTTP));
        let cookies = rx.recv().unwrap();
        Ok(cookies.map(DOMString::from).unwrap_or(DOMString::from("")))
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-cookie
    fn SetCookie(&self, cookie: DOMString) -> ErrorResult {
        // TODO: ignore for cookie-averse Document
        let url = self.url();
        if !is_scheme_host_port_tuple(url) {
            return Err(Error::Security);
        }
        let _ = self.window
                    .resource_task()
                    .send(SetCookiesForUrl((*url).clone(), String::from(cookie), NonHTTP));
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
    fn NamedGetter(&self, _cx: *mut JSContext, name: DOMString, found: &mut bool) -> *mut JSObject {
        #[derive(JSTraceable, HeapSizeOf)]
        struct NamedElementFilter {
            name: Atom,
        }
        impl CollectionFilter for NamedElementFilter {
            fn filter(&self, elem: &Element, _root: &Node) -> bool {
                filter_by_name(&self.name, elem.upcast())
            }
        }
        // https://html.spec.whatwg.org/multipage/#dom-document-nameditem-filter
        fn filter_by_name(name: &Atom, node: &Node) -> bool {
            let html_elem_type = match node.type_id() {
                NodeTypeId::Element(ElementTypeId::HTMLElement(type_)) => type_,
                _ => return false,
            };
            let elem = match node.downcast::<Element>() {
                Some(elem) => elem,
                None => return false,
            };
            match html_elem_type {
                HTMLElementTypeId::HTMLAppletElement => {
                    match elem.get_attribute(&ns!(""), &atom!("name")) {
                        Some(ref attr) if attr.value().as_atom() == name => true,
                        _ => {
                            match elem.get_attribute(&ns!(""), &atom!("id")) {
                                Some(ref attr) => attr.value().as_atom() == name,
                                None => false,
                            }
                        },
                    }
                },
                HTMLElementTypeId::HTMLFormElement => {
                    match elem.get_attribute(&ns!(""), &atom!("name")) {
                        Some(ref attr) => attr.value().as_atom() == name,
                        None => false,
                    }
                },
                HTMLElementTypeId::HTMLImageElement => {
                    match elem.get_attribute(&ns!(""), &atom!("name")) {
                        Some(ref attr) => {
                            if attr.value().as_atom() == name {
                                true
                            } else {
                                match elem.get_attribute(&ns!(""), &atom!("id")) {
                                    Some(ref attr) => attr.value().as_atom() == name,
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
        let root = self.upcast::<Node>();
        {
            // Step 1.
            let mut elements = root.traverse_preorder()
                                   .filter(|node| filter_by_name(&name, node.r()))
                                   .peekable();
            if let Some(first) = elements.next() {
                if elements.is_empty() {
                    *found = true;
                    // TODO: Step 2.
                    // Step 3.
                    return first.reflector().get_jsobject().get();
                }
            } else {
                *found = false;
                return ptr::null_mut();
            }
        }
        // Step 4.
        *found = true;
        let filter = NamedElementFilter {
            name: name,
        };
        let collection = HTMLCollection::create(self.window(), root, box filter);
        collection.reflector().get_jsobject().get()
    }

    // https://html.spec.whatwg.org/multipage/#dom-tree-accessors:supported-property-names
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

    // https://html.spec.whatwg.org/multipage/#dom-document-releaseevents
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

fn update_with_current_time(marker: &Cell<u64>) {
    if marker.get() == Default::default() {
        let current_time_ms = time::get_time().sec * 1000;
        marker.set(current_time_ms as u64);
    }
}

pub struct DocumentProgressHandler {
    addr: Trusted<Document>,
}

impl DocumentProgressHandler {
    pub fn new(addr: Trusted<Document>) -> DocumentProgressHandler {
        DocumentProgressHandler {
            addr: addr,
        }
    }

    fn set_ready_state_complete(&self) {
        let document = self.addr.root();
        document.set_ready_state(DocumentReadyState::Complete);
    }

    fn dispatch_load(&self) {
        let document = self.addr.root();
        let window = document.window();
        let event = Event::new(GlobalRef::Window(window),
                               DOMString::from("load"),
                               EventBubbles::DoesNotBubble,
                               EventCancelable::NotCancelable);
        let wintarget = window.upcast::<EventTarget>();
        event.set_trusted(true);
        let _ = wintarget.dispatch_event_with_target(document.upcast(), &event);

        let browsing_context = window.browsing_context();
        let browsing_context = browsing_context.as_ref().unwrap();

        if let Some(frame_element) = browsing_context.frame_element() {
            let frame_window = window_from_node(frame_element);
            let event = Event::new(GlobalRef::Window(frame_window.r()),
                                   DOMString::from("load"),
                                   EventBubbles::DoesNotBubble,
                                   EventCancelable::NotCancelable);
            event.fire(frame_element.upcast());
        };

        document.notify_constellation_load();

        // https://developer.mozilla.org/en-US/docs/Web/Events/mozbrowserloadend
        document.trigger_mozbrowser_event(MozBrowserEvent::LoadEnd);

        window.reflow(ReflowGoal::ForDisplay,
                      ReflowQueryType::NoQuery,
                      ReflowReason::DocumentLoaded);
    }
}

impl Runnable for DocumentProgressHandler {
    fn handler(self: Box<DocumentProgressHandler>) {
        let document = self.addr.root();
        let window = document.window();
        if window.is_alive() {
            self.set_ready_state_complete();
            self.dispatch_load();
        }
    }
}
