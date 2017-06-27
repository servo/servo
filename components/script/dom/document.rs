/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cookie_rs;
use core::nonzero::NonZero;
use devtools_traits::ScriptToDevtoolsControlMsg;
use document_loader::{DocumentLoader, LoadType};
use dom::activation::{ActivationSource, synthetic_click_activation};
use dom::attr::Attr;
use dom::beforeunloadevent::BeforeUnloadEvent;
use dom::bindings::callback::ExceptionHandling;
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::DOMRectBinding::DOMRectMethods;
use dom::bindings::codegen::Bindings::DocumentBinding;
use dom::bindings::codegen::Bindings::DocumentBinding::{DocumentMethods, DocumentReadyState, ElementCreationOptions};
use dom::bindings::codegen::Bindings::ElementBinding::ElementMethods;
use dom::bindings::codegen::Bindings::EventHandlerBinding::EventHandlerNonNull;
use dom::bindings::codegen::Bindings::EventHandlerBinding::OnErrorEventHandlerNonNull;
use dom::bindings::codegen::Bindings::HTMLIFrameElementBinding::HTMLIFrameElementBinding::HTMLIFrameElementMethods;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::codegen::Bindings::NodeFilterBinding::NodeFilter;
use dom::bindings::codegen::Bindings::PerformanceBinding::PerformanceMethods;
use dom::bindings::codegen::Bindings::TouchBinding::TouchMethods;
use dom::bindings::codegen::Bindings::WindowBinding::{FrameRequestCallback, ScrollBehavior, WindowMethods};
use dom::bindings::codegen::UnionTypes::NodeOrString;
use dom::bindings::error::{Error, ErrorResult, Fallible};
use dom::bindings::inheritance::{Castable, ElementTypeId, HTMLElementTypeId, NodeTypeId};
use dom::bindings::js::{JS, LayoutJS, MutNullableJS, Root};
use dom::bindings::js::RootedReference;
use dom::bindings::num::Finite;
use dom::bindings::refcounted::{Trusted, TrustedPromise};
use dom::bindings::reflector::{DomObject, reflect_dom_object};
use dom::bindings::str::{DOMString, USVString};
use dom::bindings::xmlname::{namespace_from_domstring, validate_and_extract, xml_name_type};
use dom::bindings::xmlname::XMLName::InvalidXMLName;
use dom::closeevent::CloseEvent;
use dom::comment::Comment;
use dom::customelementregistry::CustomElementDefinition;
use dom::customevent::CustomEvent;
use dom::documentfragment::DocumentFragment;
use dom::documenttype::DocumentType;
use dom::domimplementation::DOMImplementation;
use dom::element::{Element, ElementCreator, ElementPerformFullscreenEnter, ElementPerformFullscreenExit};
use dom::element::CustomElementCreationMode;
use dom::errorevent::ErrorEvent;
use dom::event::{Event, EventBubbles, EventCancelable, EventDefault, EventStatus};
use dom::eventtarget::EventTarget;
use dom::focusevent::FocusEvent;
use dom::forcetouchevent::ForceTouchEvent;
use dom::globalscope::GlobalScope;
use dom::hashchangeevent::HashChangeEvent;
use dom::htmlanchorelement::HTMLAnchorElement;
use dom::htmlappletelement::HTMLAppletElement;
use dom::htmlareaelement::HTMLAreaElement;
use dom::htmlbaseelement::HTMLBaseElement;
use dom::htmlbodyelement::HTMLBodyElement;
use dom::htmlcollection::{CollectionFilter, HTMLCollection};
use dom::htmlelement::HTMLElement;
use dom::htmlembedelement::HTMLEmbedElement;
use dom::htmlformelement::{FormControl, FormControlElementHelpers, HTMLFormElement};
use dom::htmlheadelement::HTMLHeadElement;
use dom::htmlhtmlelement::HTMLHtmlElement;
use dom::htmliframeelement::HTMLIFrameElement;
use dom::htmlimageelement::HTMLImageElement;
use dom::htmlscriptelement::{HTMLScriptElement, ScriptResult};
use dom::htmltitleelement::HTMLTitleElement;
use dom::keyboardevent::KeyboardEvent;
use dom::location::Location;
use dom::messageevent::MessageEvent;
use dom::mouseevent::MouseEvent;
use dom::node::{self, CloneChildrenFlag, Node, NodeDamage, window_from_node, IS_IN_DOC, LayoutNodeHelpers};
use dom::node::VecPreOrderInsertionHelper;
use dom::nodeiterator::NodeIterator;
use dom::nodelist::NodeList;
use dom::pagetransitionevent::PageTransitionEvent;
use dom::popstateevent::PopStateEvent;
use dom::processinginstruction::ProcessingInstruction;
use dom::progressevent::ProgressEvent;
use dom::promise::Promise;
use dom::range::Range;
use dom::servoparser::ServoParser;
use dom::storageevent::StorageEvent;
use dom::stylesheetlist::StyleSheetList;
use dom::text::Text;
use dom::touch::Touch;
use dom::touchevent::TouchEvent;
use dom::touchlist::TouchList;
use dom::treewalker::TreeWalker;
use dom::uievent::UIEvent;
use dom::webglcontextevent::WebGLContextEvent;
use dom::window::{ReflowReason, Window};
use dom::windowproxy::WindowProxy;
use dom_struct::dom_struct;
use encoding::EncodingRef;
use encoding::all::UTF_8;
use euclid::{Point2D, Vector2D};
use html5ever::{LocalName, QualName};
use hyper::header::{Header, SetCookie};
use hyper_serde::Serde;
use ipc_channel::ipc::{self, IpcSender};
use js::jsapi::{JSContext, JSObject, JSRuntime};
use js::jsapi::JS_GetRuntime;
use msg::constellation_msg::{ALT, CONTROL, SHIFT, SUPER};
use msg::constellation_msg::{BrowsingContextId, Key, KeyModifiers, KeyState, TopLevelBrowsingContextId};
use net_traits::{FetchResponseMsg, IpcSend, ReferrerPolicy};
use net_traits::CookieSource::NonHTTP;
use net_traits::CoreResourceMsg::{GetCookiesForUrl, SetCookiesForUrl};
use net_traits::pub_domains::is_pub_domain;
use net_traits::request::RequestInit;
use net_traits::response::HttpsState;
use num_traits::ToPrimitive;
use script_layout_interface::message::{Msg, ReflowQueryType};
use script_runtime::{CommonScriptMsg, ScriptThreadEventCategory};
use script_thread::{MainThreadScriptMsg, Runnable, ScriptThread};
use script_traits::{AnimationState, CompositorEvent, DocumentActivity};
use script_traits::{MouseButton, MouseEventType, MozBrowserEvent};
use script_traits::{MsDuration, ScriptMsg as ConstellationMsg, TouchpadPressurePhase};
use script_traits::{TouchEventType, TouchId};
use script_traits::UntrustedNodeAddress;
use servo_atoms::Atom;
use servo_config::prefs::PREFS;
use servo_url::{ImmutableOrigin, MutableOrigin, ServoUrl};
use std::ascii::AsciiExt;
use std::borrow::ToOwned;
use std::cell::{Cell, Ref, RefMut};
use std::collections::{HashMap, HashSet, VecDeque};
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::default::Default;
use std::iter::once;
use std::mem;
use std::rc::Rc;
use std::time::{Duration, Instant};
use style::attr::AttrValue;
use style::context::{QuirksMode, ReflowGoal};
use style::invalidation::element::restyle_hints::{RestyleHint, RESTYLE_SELF, RESTYLE_STYLE_ATTRIBUTE};
use style::selector_parser::{RestyleDamage, Snapshot};
use style::shared_lock::SharedRwLock as StyleSharedRwLock;
use style::str::{HTML_SPACE_CHARACTERS, split_html_space_chars, str_join};
use style::stylearc::Arc;
use style::stylesheets::Stylesheet;
use task_source::TaskSource;
use time;
use timers::OneshotTimerCallback;
use url::Host;
use url::percent_encoding::percent_decode;
use webrender_traits::ClipId;

/// The number of times we are allowed to see spurious `requestAnimationFrame()` calls before
/// falling back to fake ones.
///
/// A spurious `requestAnimationFrame()` call is defined as one that does not change the DOM.
const SPURIOUS_ANIMATION_FRAME_THRESHOLD: u8 = 5;

/// The amount of time between fake `requestAnimationFrame()`s.
const FAKE_REQUEST_ANIMATION_FRAME_DELAY: u64 = 16;

pub enum TouchEventResult {
    Processed(bool),
    Forwarded,
}

#[derive(Clone, Copy, Debug, HeapSizeOf, JSTraceable, PartialEq)]
pub enum IsHTMLDocument {
    HTMLDocument,
    NonHTMLDocument,
}

#[derive(JSTraceable, HeapSizeOf)]
#[must_root]
pub struct StylesheetInDocument {
    pub node: JS<Node>,
    #[ignore_heap_size_of = "Arc"]
    pub stylesheet: Arc<Stylesheet>,
}

#[derive(Debug, HeapSizeOf)]
pub struct PendingRestyle {
    /// If this element had a state or attribute change since the last restyle, track
    /// the original condition of the element.
    pub snapshot: Option<Snapshot>,

    /// Any explicit restyles hints that have been accumulated for this element.
    pub hint: RestyleHint,

    /// Any explicit restyles damage that have been accumulated for this element.
    pub damage: RestyleDamage,
}

impl PendingRestyle {
    pub fn new() -> Self {
        PendingRestyle {
            snapshot: None,
            hint: RestyleHint::empty(),
            damage: RestyleDamage::empty(),
        }
    }
}

/// https://dom.spec.whatwg.org/#document
#[dom_struct]
pub struct Document {
    node: Node,
    window: JS<Window>,
    implementation: MutNullableJS<DOMImplementation>,
    location: MutNullableJS<Location>,
    content_type: DOMString,
    last_modified: Option<String>,
    encoding: Cell<EncodingRef>,
    has_browsing_context: bool,
    is_html_document: bool,
    activity: Cell<DocumentActivity>,
    url: DOMRefCell<ServoUrl>,
    #[ignore_heap_size_of = "defined in selectors"]
    quirks_mode: Cell<QuirksMode>,
    /// Caches for the getElement methods
    id_map: DOMRefCell<HashMap<Atom, Vec<JS<Element>>>>,
    tag_map: DOMRefCell<HashMap<LocalName, JS<HTMLCollection>>>,
    tagns_map: DOMRefCell<HashMap<QualName, JS<HTMLCollection>>>,
    classes_map: DOMRefCell<HashMap<Vec<Atom>, JS<HTMLCollection>>>,
    images: MutNullableJS<HTMLCollection>,
    embeds: MutNullableJS<HTMLCollection>,
    links: MutNullableJS<HTMLCollection>,
    forms: MutNullableJS<HTMLCollection>,
    scripts: MutNullableJS<HTMLCollection>,
    anchors: MutNullableJS<HTMLCollection>,
    applets: MutNullableJS<HTMLCollection>,
    /// Lock use for style attributes and author-origin stylesheet objects in this document.
    /// Can be acquired once for accessing many objects.
    style_shared_lock: StyleSharedRwLock,
    /// List of stylesheets associated with nodes in this document. |None| if the list needs to be refreshed.
    stylesheets: DOMRefCell<Option<Vec<StylesheetInDocument>>>,
    /// Whether the list of stylesheets has changed since the last reflow was triggered.
    stylesheets_changed_since_reflow: Cell<bool>,
    stylesheet_list: MutNullableJS<StyleSheetList>,
    ready_state: Cell<DocumentReadyState>,
    /// Whether the DOMContentLoaded event has already been dispatched.
    domcontentloaded_dispatched: Cell<bool>,
    /// The element that has most recently requested focus for itself.
    possibly_focused: MutNullableJS<Element>,
    /// The element that currently has the document focus context.
    focused: MutNullableJS<Element>,
    /// The script element that is currently executing.
    current_script: MutNullableJS<HTMLScriptElement>,
    /// https://html.spec.whatwg.org/multipage/#pending-parsing-blocking-script
    pending_parsing_blocking_script: DOMRefCell<Option<PendingScript>>,
    /// Number of stylesheets that block executing the next parser-inserted script
    script_blocking_stylesheets_count: Cell<u32>,
    /// https://html.spec.whatwg.org/multipage/#list-of-scripts-that-will-execute-when-the-document-has-finished-parsing
    deferred_scripts: PendingInOrderScriptVec,
    /// https://html.spec.whatwg.org/multipage/#list-of-scripts-that-will-execute-in-order-as-soon-as-possible
    asap_in_order_scripts_list: PendingInOrderScriptVec,
    /// https://html.spec.whatwg.org/multipage/#set-of-scripts-that-will-execute-as-soon-as-possible
    asap_scripts_set: DOMRefCell<Vec<JS<HTMLScriptElement>>>,
    /// https://html.spec.whatwg.org/multipage/#concept-n-noscript
    /// True if scripting is enabled for all scripts in this document
    scripting_enabled: bool,
    /// https://html.spec.whatwg.org/multipage/#animation-frame-callback-identifier
    /// Current identifier of animation frame callback
    animation_frame_ident: Cell<u32>,
    /// https://html.spec.whatwg.org/multipage/#list-of-animation-frame-callbacks
    /// List of animation frame callbacks
    animation_frame_list: DOMRefCell<Vec<(u32, Option<AnimationFrameCallback>)>>,
    /// Whether we're in the process of running animation callbacks.
    ///
    /// Tracking this is not necessary for correctness. Instead, it is an optimization to avoid
    /// sending needless `ChangeRunningAnimationsState` messages to the compositor.
    running_animation_callbacks: Cell<bool>,
    /// Tracks all outstanding loads related to this document.
    loader: DOMRefCell<DocumentLoader>,
    /// The current active HTML parser, to allow resuming after interruptions.
    current_parser: MutNullableJS<ServoParser>,
    /// When we should kick off a reflow. This happens during parsing.
    reflow_timeout: Cell<Option<u64>>,
    /// The cached first `base` element with an `href` attribute.
    base_element: MutNullableJS<HTMLBaseElement>,
    /// This field is set to the document itself for inert documents.
    /// https://html.spec.whatwg.org/multipage/#appropriate-template-contents-owner-document
    appropriate_template_contents_owner_document: MutNullableJS<Document>,
    /// Information on elements needing restyle to ship over to the layout thread when the
    /// time comes.
    pending_restyles: DOMRefCell<HashMap<JS<Element>, PendingRestyle>>,
    /// This flag will be true if layout suppressed a reflow attempt that was
    /// needed in order for the page to be painted.
    needs_paint: Cell<bool>,
    /// http://w3c.github.io/touch-events/#dfn-active-touch-point
    active_touch_points: DOMRefCell<Vec<JS<Touch>>>,
    /// Navigation Timing properties:
    /// https://w3c.github.io/navigation-timing/#sec-PerformanceNavigationTiming
    dom_loading: Cell<u64>,
    dom_interactive: Cell<u64>,
    dom_content_loaded_event_start: Cell<u64>,
    dom_content_loaded_event_end: Cell<u64>,
    dom_complete: Cell<u64>,
    load_event_start: Cell<u64>,
    load_event_end: Cell<u64>,
    /// https://html.spec.whatwg.org/multipage/#concept-document-https-state
    https_state: Cell<HttpsState>,
    touchpad_pressure_phase: Cell<TouchpadPressurePhase>,
    /// The document's origin.
    origin: MutableOrigin,
    ///  https://w3c.github.io/webappsec-referrer-policy/#referrer-policy-states
    referrer_policy: Cell<Option<ReferrerPolicy>>,
    /// https://html.spec.whatwg.org/multipage/#dom-document-referrer
    referrer: Option<String>,
    /// https://html.spec.whatwg.org/multipage/#target-element
    target_element: MutNullableJS<Element>,
    /// https://w3c.github.io/uievents/#event-type-dblclick
    #[ignore_heap_size_of = "Defined in std"]
    last_click_info: DOMRefCell<Option<(Instant, Point2D<f32>)>>,
    /// https://html.spec.whatwg.org/multipage/#ignore-destructive-writes-counter
    ignore_destructive_writes_counter: Cell<u32>,
    /// The number of spurious `requestAnimationFrame()` requests we've received.
    ///
    /// A rAF request is considered spurious if nothing was actually reflowed.
    spurious_animation_frames: Cell<u8>,

    /// Track the total number of elements in this DOM's tree.
    /// This is sent to the layout thread every time a reflow is done;
    /// layout uses this to determine if the gains from parallel layout will be worth the overhead.
    ///
    /// See also: https://github.com/servo/servo/issues/10110
    dom_count: Cell<u32>,
    /// Entry node for fullscreen.
    fullscreen_element: MutNullableJS<Element>,
    /// Map from ID to set of form control elements that have that ID as
    /// their 'form' content attribute. Used to reset form controls
    /// whenever any element with the same ID as the form attribute
    /// is inserted or removed from the document.
    /// See https://html.spec.whatwg.org/multipage/#form-owner
    form_id_listener_map: DOMRefCell<HashMap<Atom, HashSet<JS<Element>>>>,
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
        elem.has_attribute(&local_name!("href"))
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
        elem.is::<HTMLAnchorElement>() && elem.has_attribute(&local_name!("href"))
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
    pub fn has_browsing_context(&self) -> bool { self.has_browsing_context }

    /// https://html.spec.whatwg.org/multipage/#concept-document-bc
    #[inline]
    pub fn browsing_context(&self) -> Option<Root<WindowProxy>> {
        if self.has_browsing_context {
            self.window.undiscarded_window_proxy()
        } else {
            None
        }
    }

    #[inline]
    pub fn window(&self) -> &Window {
        &*self.window
    }

    #[inline]
    pub fn is_html_document(&self) -> bool {
        self.is_html_document
    }

    pub fn set_https_state(&self, https_state: HttpsState) {
        self.https_state.set(https_state);
        self.trigger_mozbrowser_event(MozBrowserEvent::SecurityChange(https_state));
    }

    pub fn is_fully_active(&self) -> bool {
        self.activity.get() == DocumentActivity::FullyActive
    }

    pub fn is_active(&self) -> bool {
        self.activity.get() != DocumentActivity::Inactive
    }

    pub fn set_activity(&self, activity: DocumentActivity) {
        // This function should only be called on documents with a browsing context
        assert!(self.has_browsing_context);
        // Set the document's activity level, reflow if necessary, and suspend or resume timers.
        if activity != self.activity.get() {
            self.activity.set(activity);
            if activity == DocumentActivity::FullyActive {
                self.title_changed();
                self.dirty_all_nodes();
                self.window().reflow(
                    ReflowGoal::ForDisplay,
                    ReflowQueryType::NoQuery,
                    ReflowReason::CachedPageNeededReflow
                );
                self.window().resume();
            } else {
                self.window().suspend();
            }
        }
    }

    pub fn origin(&self) -> &MutableOrigin {
        &self.origin
    }

    // https://dom.spec.whatwg.org/#concept-document-url
    pub fn url(&self) -> ServoUrl {
        self.url.borrow().clone()
    }

    pub fn set_url(&self, url: ServoUrl) {
        *self.url.borrow_mut() = url;
    }

    // https://html.spec.whatwg.org/multipage/#fallback-base-url
    pub fn fallback_base_url(&self) -> ServoUrl {
        // Step 1: iframe srcdoc (#4767).
        // Step 2: about:blank with a creator browsing context.
        // Step 3.
        self.url()
    }

    // https://html.spec.whatwg.org/multipage/#document-base-url
    pub fn base_url(&self) -> ServoUrl {
        match self.base_element() {
            // Step 1.
            None => self.fallback_base_url(),
            // Step 2.
            Some(base) => base.frozen_base_url(),
        }
    }

    pub fn needs_paint(&self) -> bool {
        self.needs_paint.get()
    }

    pub fn needs_reflow(&self) -> bool {
        // FIXME: This should check the dirty bit on the document,
        // not the document element. Needs some layout changes to make
        // that workable.
        self.stylesheets_changed_since_reflow.get() ||
        match self.GetDocumentElement() {
            Some(root) => {
                root.upcast::<Node>().has_dirty_descendants() ||
                !self.pending_restyles.borrow().is_empty() ||
                self.needs_paint()
            }
            None => false,
        }
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
                       .find(|element| element.upcast::<Element>().has_attribute(&local_name!("href")));
        self.base_element.set(base.r());
    }

    pub fn dom_count(&self) -> u32 {
        self.dom_count.get()
    }

    /// This is called by `bind_to_tree` when a node is added to the DOM.
    /// The internal count is used by layout to determine whether to be sequential or parallel.
    /// (it's sequential for small DOMs)
    pub fn increment_dom_count(&self) {
        self.dom_count.set(self.dom_count.get() + 1);
    }

    /// This is called by `unbind_from_tree` when a node is removed from the DOM.
    pub fn decrement_dom_count(&self) {
        self.dom_count.set(self.dom_count.get() - 1);
    }

    pub fn quirks_mode(&self) -> QuirksMode {
        self.quirks_mode.get()
    }

    pub fn set_quirks_mode(&self, mode: QuirksMode) {
        self.quirks_mode.set(mode);

        if mode == QuirksMode::Quirks {
            self.window.layout_chan().send(Msg::SetQuirksMode(mode)).unwrap();
        }
    }

    pub fn encoding(&self) -> EncodingRef {
        self.encoding.get()
    }

    pub fn set_encoding(&self, encoding: EncodingRef) {
        self.encoding.set(encoding);
    }

    pub fn content_and_heritage_changed(&self, node: &Node, damage: NodeDamage) {
        node.dirty(damage);
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

    /// Remove any existing association between the provided id and any elements in this document.
    pub fn unregister_named_element(&self, to_unregister: &Element, id: Atom) {
        debug!("Removing named element from document {:p}: {:p} id={}",
               self,
               to_unregister,
               id);
        // Limit the scope of the borrow because id_map might be borrowed again by
        // GetElementById through the following sequence of calls
        // reset_form_owner_for_listeners -> reset_form_owner -> GetElementById
        {
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
        self.reset_form_owner_for_listeners(&id);
    }

    /// Associate an element present in this document with the provided id.
    pub fn register_named_element(&self, element: &Element, id: Atom) {
        debug!("Adding named element to document {:p}: {:p} id={}",
               self,
               element,
               id);
        assert!(element.upcast::<Node>().is_in_doc());
        assert!(!id.is_empty());

        let root = self.GetDocumentElement()
                       .expect("The element is in the document, so there must be a document \
                                element.");

        // Limit the scope of the borrow because id_map might be borrowed again by
        // GetElementById through the following sequence of calls
        // reset_form_owner_for_listeners -> reset_form_owner -> GetElementById
        {
            let mut id_map = self.id_map.borrow_mut();
            let mut elements = id_map.entry(id.clone()).or_insert(Vec::new());
            elements.insert_pre_order(element, root.r().upcast::<Node>());
        }
        self.reset_form_owner_for_listeners(&id);
    }

    pub fn register_form_id_listener<T: ?Sized + FormControl>(&self, id: DOMString, listener: &T) {
        let mut map = self.form_id_listener_map.borrow_mut();
        let listener = listener.to_element();
        let mut set = map.entry(Atom::from(id)).or_insert(HashSet::new());
        set.insert(JS::from_ref(listener));
    }

    pub fn unregister_form_id_listener<T: ?Sized + FormControl>(&self, id: DOMString, listener: &T) {
        let mut map = self.form_id_listener_map.borrow_mut();
        if let Occupied(mut entry) = map.entry(Atom::from(id)) {
            entry.get_mut().remove(&JS::from_ref(listener.to_element()));
            if entry.get().is_empty() {
                entry.remove();
            }
        }
    }

    /// Attempt to find a named element in this page's document.
    /// https://html.spec.whatwg.org/multipage/#the-indicated-part-of-the-document
    pub fn find_fragment_node(&self, fragid: &str) -> Option<Root<Element>> {
        // Step 1 is not handled here; the fragid is already obtained by the calling function
        // Step 2: Simply use None to indicate the top of the document.
        // Step 3 & 4
        percent_decode(fragid.as_bytes()).decode_utf8().ok()
        // Step 5
            .and_then(|decoded_fragid| self.get_element_by_id(&Atom::from(decoded_fragid)))
        // Step 6
            .or_else(|| self.get_anchor_by_name(fragid))
        // Step 7 & 8
    }

    /// Scroll to the target element, and when we do not find a target
    /// and the fragment is empty or "top", scroll to the top.
    /// https://html.spec.whatwg.org/multipage/#scroll-to-the-fragment-identifier
    pub fn check_and_scroll_fragment(&self, fragment: &str) {
        let target = self.find_fragment_node(fragment);

        // Step 1
        self.set_target_element(target.r());

        let point = target.r().map(|element| {
            // FIXME(#8275, pcwalton): This is pretty bogus when multiple layers are involved.
            // Really what needs to happen is that this needs to go through layout to ask which
            // layer the element belongs to, and have it send the scroll message to the
            // compositor.
            let rect = element.upcast::<Node>().bounding_content_box_or_zero();

            // In order to align with element edges, we snap to unscaled pixel boundaries, since
            // the paint thread currently does the same for drawing elements. This is important
            // for pages that require pixel perfect scroll positioning for proper display
            // (like Acid2). Since we don't have the device pixel ratio here, this might not be
            // accurate, but should work as long as the ratio is a whole number. Once #8275 is
            // fixed this should actually take into account the real device pixel ratio.
            (rect.origin.x.to_nearest_px() as f32, rect.origin.y.to_nearest_px() as f32)
        }).or_else(|| if fragment.is_empty() || fragment.eq_ignore_ascii_case("top") {
            // FIXME(stshine): this should be the origin of the stacking context space,
            // which may differ under the influence of writing mode.
            Some((0.0, 0.0))
        } else {
            None
        });

        if let Some((x, y)) = point {
            // Step 3
            let global_scope = self.window.upcast::<GlobalScope>();
            let webrender_pipeline_id = global_scope.pipeline_id().to_webrender();
            self.window.update_viewport_for_scroll(x, y);
            self.window.perform_a_scroll(x,
                                         y,
                                         ClipId::root_scroll_node(webrender_pipeline_id),
                                         ScrollBehavior::Instant,
                                         target.r());
        }
    }

    fn get_anchor_by_name(&self, name: &str) -> Option<Root<Element>> {
        let check_anchor = |node: &HTMLAnchorElement| {
            let elem = node.upcast::<Element>();
            elem.get_attribute(&ns!(), &local_name!("name"))
                .map_or(false, |attr| &**attr.value() == name)
        };
        let doc_node = self.upcast::<Node>();
        doc_node.traverse_preorder()
                .filter_map(Root::downcast)
                .find(|node| check_anchor(&node))
                .map(Root::upcast)
    }

    // https://html.spec.whatwg.org/multipage/#current-document-readiness
    pub fn set_ready_state(&self, state: DocumentReadyState) {
        match state {
            DocumentReadyState::Loading => {
                // https://developer.mozilla.org/en-US/docs/Web/Events/mozbrowserconnected
                self.trigger_mozbrowser_event(MozBrowserEvent::Connected);
                update_with_current_time_ms(&self.dom_loading);
            },
            DocumentReadyState::Complete => {
                // https://developer.mozilla.org/en-US/docs/Web/Events/mozbrowserloadend
                self.trigger_mozbrowser_event(MozBrowserEvent::LoadEnd);
                update_with_current_time_ms(&self.dom_complete);
            },
            DocumentReadyState::Interactive => update_with_current_time_ms(&self.dom_interactive),
        };

        self.ready_state.set(state);

        self.upcast::<EventTarget>().fire_event(atom!("readystatechange"));
    }

    /// Return whether scripting is enabled or not
    pub fn is_scripting_enabled(&self) -> bool {
        self.scripting_enabled
    }

    /// Return the element that currently has focus.
    // https://w3c.github.io/uievents/#events-focusevent-doc-focus
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
        if self.focused == self.possibly_focused.get().r() {
            return
        }
        if let Some(ref elem) = self.focused.get() {
            let node = elem.upcast::<Node>();
            elem.set_focus_state(false);
            // FIXME: pass appropriate relatedTarget
            self.fire_focus_event(FocusEventType::Blur, node, None);
        }

        self.focused.set(self.possibly_focused.get().r());

        if let Some(ref elem) = self.focused.get() {
            elem.set_focus_state(true);
            let node = elem.upcast::<Node>();
            // FIXME: pass appropriate relatedTarget
            self.fire_focus_event(FocusEventType::Focus, node, None);
            // Update the focus state for all elements in the focus chain.
            // https://html.spec.whatwg.org/multipage/#focus-chain
            if focus_type == FocusType::Element {
                let global_scope = self.window.upcast::<GlobalScope>();
                let event = ConstellationMsg::Focus(global_scope.pipeline_id());
                global_scope.constellation_chan().send(event).unwrap();
            }
        }
    }

    /// Handles any updates when the document's title has changed.
    pub fn title_changed(&self) {
        if self.browsing_context().is_some() {
            // https://developer.mozilla.org/en-US/docs/Web/Events/mozbrowsertitlechange
            self.trigger_mozbrowser_event(MozBrowserEvent::TitleChange(String::from(self.Title())));

            self.send_title_to_compositor();
        }
    }

    /// Sends this document's title to the compositor.
    pub fn send_title_to_compositor(&self) {
        let window = self.window();
        let global_scope = window.upcast::<GlobalScope>();
        global_scope
              .constellation_chan()
              .send(ConstellationMsg::SetTitle(global_scope.pipeline_id(),
                                               Some(String::from(self.Title()))))
              .unwrap();
    }

    pub fn dirty_all_nodes(&self) {
        let root = self.upcast::<Node>();
        for node in root.traverse_preorder() {
            node.dirty(NodeDamage::OtherNodeDamage)
        }
    }

    #[allow(unsafe_code)]
    pub fn handle_mouse_event(&self,
                              js_runtime: *mut JSRuntime,
                              button: MouseButton,
                              client_point: Point2D<f32>,
                              mouse_event_type: MouseEventType) {
        let mouse_event_type_string = match mouse_event_type {
            MouseEventType::Click => "click".to_owned(),
            MouseEventType::MouseUp => "mouseup".to_owned(),
            MouseEventType::MouseDown => "mousedown".to_owned(),
        };
        debug!("{}: at {:?}", mouse_event_type_string, client_point);

        let node = match self.window.hit_test_query(client_point, false) {
            Some(node_address) => {
                debug!("node address is {:?}", node_address);
                unsafe {
                    node::from_untrusted_node_address(js_runtime, node_address)
                }
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

        // If the target is an iframe, forward the event to the child document.
        if let Some(iframe) = el.downcast::<HTMLIFrameElement>() {
            if let Some(pipeline_id) = iframe.pipeline_id() {
                let rect = iframe.upcast::<Element>().GetBoundingClientRect();
                let child_origin = Vector2D::new(rect.X() as f32, rect.Y() as f32);
                let child_point = client_point - child_origin;

                let event = CompositorEvent::MouseButtonEvent(mouse_event_type, button, child_point);
                let event = ConstellationMsg::ForwardEvent(pipeline_id, event);
                self.window.upcast::<GlobalScope>().constellation_chan().send(event).unwrap();
            }
            return;
        }

        let node = el.upcast::<Node>();
        debug!("{} on {:?}", mouse_event_type_string, node.debug_str());
        // Prevent click event if form control element is disabled.
        if let MouseEventType::Click = mouse_event_type {
            if el.click_event_filter_by_disabled_state() {
                return;
            }

            self.begin_focus_transaction();
        }

        // https://w3c.github.io/uievents/#event-type-click
        let client_x = client_point.x as i32;
        let client_y = client_point.y as i32;
        let click_count = 1;
        let event = MouseEvent::new(&self.window,
                                    DOMString::from(mouse_event_type_string),
                                    EventBubbles::Bubbles,
                                    EventCancelable::Cancelable,
                                    Some(&self.window),
                                    click_count,
                                    client_x,
                                    client_y,
                                    client_x,
                                    client_y, // TODO: Get real screen coordinates?
                                    false,
                                    false,
                                    false,
                                    false,
                                    0i16,
                                    None);
        let event = event.upcast::<Event>();

        // https://w3c.github.io/uievents/#trusted-events
        event.set_trusted(true);
        // https://html.spec.whatwg.org/multipage/#run-authentic-click-activation-steps
        let activatable = el.as_maybe_activatable();
        match mouse_event_type {
            MouseEventType::Click => el.authentic_click_activation(event),
            MouseEventType::MouseDown => {
                if let Some(a) = activatable {
                    a.enter_formal_activation_state();
                }

                let target = node.upcast();
                event.fire(target);
            },
            MouseEventType::MouseUp => {
                if let Some(a) = activatable {
                    a.exit_formal_activation_state();
                }

                let target = node.upcast();
                event.fire(target);
            },
        }

        if let MouseEventType::Click = mouse_event_type {
            self.commit_focus_transaction(FocusType::Element);
            self.maybe_fire_dblclick(client_point, node);
        }

        self.window.reflow(ReflowGoal::ForDisplay,
                           ReflowQueryType::NoQuery,
                           ReflowReason::MouseEvent);
    }

    fn maybe_fire_dblclick(&self, click_pos: Point2D<f32>, target: &Node) {
        // https://w3c.github.io/uievents/#event-type-dblclick
        let now = Instant::now();

        let opt = self.last_click_info.borrow_mut().take();

        if let Some((last_time, last_pos)) = opt {
            let DBL_CLICK_TIMEOUT = Duration::from_millis(PREFS.get("dom.document.dblclick_timeout").as_u64()
                                                                                                    .unwrap_or(300));
            let DBL_CLICK_DIST_THRESHOLD = PREFS.get("dom.document.dblclick_dist").as_u64().unwrap_or(1);

            // Calculate distance between this click and the previous click.
            let line = click_pos - last_pos;
            let dist = (line.dot(line) as f64).sqrt();

            if  now.duration_since(last_time) < DBL_CLICK_TIMEOUT &&
                dist < DBL_CLICK_DIST_THRESHOLD as f64 {
                // A double click has occurred if this click is within a certain time and dist. of previous click.
                let click_count = 2;
                let client_x = click_pos.x as i32;
                let client_y = click_pos.y as i32;

                let event = MouseEvent::new(&self.window,
                                            DOMString::from("dblclick"),
                                            EventBubbles::Bubbles,
                                            EventCancelable::Cancelable,
                                            Some(&self.window),
                                            click_count,
                                            client_x,
                                            client_y,
                                            client_x,
                                            client_y,
                                            false,
                                            false,
                                            false,
                                            false,
                                            0i16,
                                            None);
                event.upcast::<Event>().fire(target.upcast());

                // When a double click occurs, self.last_click_info is left as None so that a
                // third sequential click will not cause another double click.
                return;
            }
        }

        // Update last_click_info with the time and position of the click.
        *self.last_click_info.borrow_mut() = Some((now, click_pos));
    }

    #[allow(unsafe_code)]
    pub fn handle_touchpad_pressure_event(&self,
                                          js_runtime: *mut JSRuntime,
                                          client_point: Point2D<f32>,
                                          pressure: f32,
                                          phase_now: TouchpadPressurePhase) {
        let node = match self.window.hit_test_query(client_point, false) {
            Some(node_address) => unsafe {
                node::from_untrusted_node_address(js_runtime, node_address)
            },
            None => return
        };

        let el = match node.downcast::<Element>() {
            Some(el) => Root::from_ref(el),
            None => {
                let parent = node.GetParentNode();
                match parent.and_then(Root::downcast::<Element>) {
                    Some(parent) => parent,
                    None => return
                }
            },
        };

        // If the target is an iframe, forward the event to the child document.
        if let Some(iframe) = el.downcast::<HTMLIFrameElement>() {
            if let Some(pipeline_id) = iframe.pipeline_id() {
                let rect = iframe.upcast::<Element>().GetBoundingClientRect();
                let child_origin = Vector2D::new(rect.X() as f32, rect.Y() as f32);
                let child_point = client_point - child_origin;

                let event = CompositorEvent::TouchpadPressureEvent(child_point,
                                                                   pressure,
                                                                   phase_now);
                let event = ConstellationMsg::ForwardEvent(pipeline_id, event);
                self.window.upcast::<GlobalScope>().constellation_chan().send(event).unwrap();
            }
            return;
        }

        let phase_before = self.touchpad_pressure_phase.get();
        self.touchpad_pressure_phase.set(phase_now);

        if phase_before == TouchpadPressurePhase::BeforeClick &&
           phase_now == TouchpadPressurePhase::BeforeClick {
            return;
        }

        let node = el.upcast::<Node>();
        let target = node.upcast();

        let force = match phase_now {
            TouchpadPressurePhase::BeforeClick => pressure,
            TouchpadPressurePhase::AfterFirstClick => 1. + pressure,
            TouchpadPressurePhase::AfterSecondClick => 2. + pressure,
        };

        if phase_now != TouchpadPressurePhase::BeforeClick {
            self.fire_forcetouch_event("servomouseforcechanged".to_owned(), target, force);
        }

        if phase_before != TouchpadPressurePhase::AfterSecondClick &&
           phase_now == TouchpadPressurePhase::AfterSecondClick {
            self.fire_forcetouch_event("servomouseforcedown".to_owned(), target, force);
        }

        if phase_before == TouchpadPressurePhase::AfterSecondClick &&
           phase_now != TouchpadPressurePhase::AfterSecondClick {
            self.fire_forcetouch_event("servomouseforceup".to_owned(), target, force);
        }
    }

    fn fire_forcetouch_event(&self, event_name: String, target: &EventTarget, force: f32) {
        let force_event = ForceTouchEvent::new(&self.window,
                                               DOMString::from(event_name),
                                               force);
        let event = force_event.upcast::<Event>();
        event.fire(target);
    }

    pub fn fire_mouse_event(&self, client_point: Point2D<f32>, target: &EventTarget, event_name: String) {
        let client_x = client_point.x.to_i32().unwrap_or(0);
        let client_y = client_point.y.to_i32().unwrap_or(0);

        let mouse_event = MouseEvent::new(&self.window,
                                          DOMString::from(event_name),
                                          EventBubbles::Bubbles,
                                          EventCancelable::Cancelable,
                                          Some(&self.window),
                                          0i32,
                                          client_x,
                                          client_y,
                                          client_x,
                                          client_y,
                                          false,
                                          false,
                                          false,
                                          false,
                                          0i16,
                                          None);
        let event = mouse_event.upcast::<Event>();
        event.fire(target);
    }

    #[allow(unsafe_code)]
    pub fn handle_mouse_move_event(&self,
                                   js_runtime: *mut JSRuntime,
                                   client_point: Option<Point2D<f32>>,
                                   prev_mouse_over_target: &MutNullableJS<Element>) {
        let client_point = match client_point {
            None => {
                // If there's no point, there's no target under the mouse
                // FIXME: dispatch mouseout here. We have no point.
                prev_mouse_over_target.set(None);
                return;
            }
            Some(client_point) => client_point,
        };

        let maybe_new_target = self.window.hit_test_query(client_point, true).and_then(|address| {
            let node = unsafe { node::from_untrusted_node_address(js_runtime, address) };
            node.inclusive_ancestors()
                .filter_map(Root::downcast::<Element>)
                .next()
        });

        // Send mousemove event to topmost target, and forward it if it's an iframe
        if let Some(ref new_target) = maybe_new_target {
            // If the target is an iframe, forward the event to the child document.
            if let Some(iframe) = new_target.downcast::<HTMLIFrameElement>() {
                if let Some(pipeline_id) = iframe.pipeline_id() {
                    let rect = iframe.upcast::<Element>().GetBoundingClientRect();
                    let child_origin = Vector2D::new(rect.X() as f32, rect.Y() as f32);
                    let child_point = client_point - child_origin;

                    let event = CompositorEvent::MouseMoveEvent(Some(child_point));
                    let event = ConstellationMsg::ForwardEvent(pipeline_id, event);
                    self.window.upcast::<GlobalScope>().constellation_chan().send(event).unwrap();
                }
                return;
            }

            self.fire_mouse_event(client_point, new_target.upcast(), "mousemove".to_owned());
        }

        // Nothing more to do here, mousemove is sent,
        // and the element under the mouse hasn't changed.
        if maybe_new_target == prev_mouse_over_target.get() {
            return;
        }

        let old_target_is_ancestor_of_new_target = match (prev_mouse_over_target.get(), maybe_new_target.as_ref()) {
            (Some(old_target), Some(new_target))
                => old_target.upcast::<Node>().is_ancestor_of(new_target.upcast::<Node>()),
            _   => false,
        };

        // Here we know the target has changed, so we must update the state,
        // dispatch mouseout to the previous one, mouseover to the new one,
        if let Some(old_target) = prev_mouse_over_target.get() {
            // If the old target is an ancestor of the new target, this can be skipped
            // completely, since the node's hover state will be reseted below.
            if !old_target_is_ancestor_of_new_target {
                for element in old_target.upcast::<Node>()
                                         .inclusive_ancestors()
                                         .filter_map(Root::downcast::<Element>) {
                    element.set_hover_state(false);
                    element.set_active_state(false);
                }
            }

            // Remove hover state to old target and its parents
            self.fire_mouse_event(client_point, old_target.upcast(), "mouseout".to_owned());

            // TODO: Fire mouseleave here only if the old target is
            // not an ancestor of the new target.
        }

        if let Some(ref new_target) = maybe_new_target {
            for element in new_target.upcast::<Node>()
                                     .inclusive_ancestors()
                                     .filter_map(Root::downcast::<Element>) {
                if element.hover_state() {
                    break;
                }

                element.set_hover_state(true);
            }

            self.fire_mouse_event(client_point, &new_target.upcast(), "mouseover".to_owned());

            // TODO: Fire mouseenter here.
        }

        // Store the current mouse over target for next frame.
        prev_mouse_over_target.set(maybe_new_target.r());

        self.window.reflow(ReflowGoal::ForDisplay,
                           ReflowQueryType::NoQuery,
                           ReflowReason::MouseEvent);
    }

    #[allow(unsafe_code)]
    pub fn handle_touch_event(&self,
                              js_runtime: *mut JSRuntime,
                              event_type: TouchEventType,
                              touch_id: TouchId,
                              point: Point2D<f32>)
                              -> TouchEventResult {
        let TouchId(identifier) = touch_id;

        let event_name = match event_type {
            TouchEventType::Down => "touchstart",
            TouchEventType::Move => "touchmove",
            TouchEventType::Up => "touchend",
            TouchEventType::Cancel => "touchcancel",
        };

        let node = match self.window.hit_test_query(point, false) {
            Some(node_address) => unsafe {
                node::from_untrusted_node_address(js_runtime, node_address)
            },
            None => return TouchEventResult::Processed(false),
        };
        let el = match node.downcast::<Element>() {
            Some(el) => Root::from_ref(el),
            None => {
                let parent = node.GetParentNode();
                match parent.and_then(Root::downcast::<Element>) {
                    Some(parent) => parent,
                    None => return TouchEventResult::Processed(false),
                }
            },
        };

        // If the target is an iframe, forward the event to the child document.
        if let Some(iframe) = el.downcast::<HTMLIFrameElement>() {
            if let Some(pipeline_id) = iframe.pipeline_id() {
                let rect = iframe.upcast::<Element>().GetBoundingClientRect();
                let child_origin = Vector2D::new(rect.X() as f32, rect.Y() as f32);
                let child_point = point - child_origin;

                let event = CompositorEvent::TouchEvent(event_type, touch_id, child_point);
                let event = ConstellationMsg::ForwardEvent(pipeline_id, event);
                self.window.upcast::<GlobalScope>().constellation_chan().send(event).unwrap();
            }
            return TouchEventResult::Forwarded;
        }

        let target = Root::upcast::<EventTarget>(el);
        let window = &*self.window;

        let client_x = Finite::wrap(point.x as f64);
        let client_y = Finite::wrap(point.y as f64);
        let page_x = Finite::wrap(point.x as f64 + window.PageXOffset() as f64);
        let page_y = Finite::wrap(point.y as f64 + window.PageYOffset() as f64);

        let touch = Touch::new(window,
                               identifier,
                               &target,
                               client_x,
                               client_y, // TODO: Get real screen coordinates?
                               client_x,
                               client_y,
                               page_x,
                               page_y);

        match event_type {
            TouchEventType::Down => {
                // Add a new touch point
                self.active_touch_points.borrow_mut().push(JS::from_ref(&*touch));
            }
            TouchEventType::Move => {
                // Replace an existing touch point
                let mut active_touch_points = self.active_touch_points.borrow_mut();
                match active_touch_points.iter_mut().find(|t| t.Identifier() == identifier) {
                    Some(t) => *t = JS::from_ref(&*touch),
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

        rooted_vec!(let mut touches);
        touches.extend(self.active_touch_points.borrow().iter().cloned());
        rooted_vec!(let mut target_touches);
        target_touches.extend(self.active_touch_points
                                  .borrow()
                                  .iter()
                                  .filter(|t| t.Target() == target)
                                  .cloned());
        rooted_vec!(let changed_touches <- once(touch));

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
        let result = event.fire(&target);

        window.reflow(ReflowGoal::ForDisplay,
                      ReflowQueryType::NoQuery,
                      ReflowReason::MouseEvent);

        match result {
            EventStatus::Canceled => TouchEventResult::Processed(false),
            EventStatus::NotCanceled => TouchEventResult::Processed(true),
        }
    }

    /// The entry point for all key processing for web content
    pub fn dispatch_key_event(&self,
                              ch: Option<char>,
                              key: Key,
                              state: KeyState,
                              modifiers: KeyModifiers,
                              constellation: &IpcSender<ConstellationMsg>) {
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

        let props = KeyboardEvent::key_properties(ch, key, modifiers);

        let keyevent = KeyboardEvent::new(&self.window,
                                          ev_type,
                                          true,
                                          true,
                                          Some(&self.window),
                                          0,
                                          ch,
                                          Some(key),
                                          DOMString::from(props.key_string.clone()),
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
        let mut cancel_state = event.get_cancel_state();

        // https://w3c.github.io/uievents/#keys-cancelable-keys
        if state != KeyState::Released && props.is_printable() && cancel_state != EventDefault::Prevented {
            // https://w3c.github.io/uievents/#keypress-event-order
            let event = KeyboardEvent::new(&self.window,
                                           DOMString::from("keypress"),
                                           true,
                                           true,
                                           Some(&self.window),
                                           0,
                                           ch,
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
            cancel_state = ev.get_cancel_state();
        }

        if cancel_state == EventDefault::Allowed {
            constellation.send(ConstellationMsg::SendKeyEvent(ch, key, state, modifiers)).unwrap();

            // This behavior is unspecced
            // We are supposed to dispatch synthetic click activation for Space and/or Return,
            // however *when* we do it is up to us.
            // Here, we're dispatching it after the key event so the script has a chance to cancel it
            // https://www.w3.org/Bugs/Public/show_bug.cgi?id=27337
            match key {
                Key::Space if state == KeyState::Released => {
                    let maybe_elem = target.downcast::<Element>();
                    if let Some(el) = maybe_elem {
                        synthetic_click_activation(el,
                                                   false,
                                                   false,
                                                   false,
                                                   false,
                                                   ActivationSource::NotFromClick)
                    }
                }
                Key::Enter if state == KeyState::Released => {
                    let maybe_elem = target.downcast::<Element>();
                    if let Some(el) = maybe_elem {
                        if let Some(a) = el.as_maybe_activatable() {
                            a.implicit_submission(ctrl, alt, shift, meta);
                        }
                    }
                }
                _ => (),
            }
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
                NodeOrString::Node(node) => node,
                NodeOrString::String(string) => Root::upcast(self.CreateTextNode(string)),
            })
        } else {
            let fragment = Root::upcast::<Node>(self.CreateDocumentFragment());
            for node in nodes {
                match node {
                    NodeOrString::Node(node) => {
                        fragment.AppendChild(&node)?;
                    },
                    NodeOrString::String(string) => {
                        let node = Root::upcast::<Node>(self.CreateTextNode(string));
                        // No try!() here because appending a text node
                        // should not fail.
                        fragment.AppendChild(&node).unwrap();
                    }
                }
            }
            Ok(fragment)
        }
    }

    pub fn get_body_attribute(&self, local_name: &LocalName) -> DOMString {
        match self.GetBody().and_then(Root::downcast::<HTMLBodyElement>) {
            Some(ref body) => {
                body.upcast::<Element>().get_string_attribute(local_name)
            },
            None => DOMString::new(),
        }
    }

    pub fn set_body_attribute(&self, local_name: &LocalName, value: DOMString) {
        if let Some(ref body) = self.GetBody().and_then(Root::downcast::<HTMLBodyElement>) {
            let body = body.upcast::<Element>();
            let value = body.parse_attribute(&ns!(), &local_name, value);
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
        if let Some(element) = self.GetDocumentElement() {
            element.upcast::<Node>().dirty(NodeDamage::NodeStyleDamaged);
        }
    }

    pub fn get_and_reset_stylesheets_changed_since_reflow(&self) -> bool {
        let changed = self.stylesheets_changed_since_reflow.get();
        self.stylesheets_changed_since_reflow.set(false);
        changed
    }

    pub fn trigger_mozbrowser_event(&self, event: MozBrowserEvent) {
        if PREFS.is_mozbrowser_enabled() {
            if let Some((parent_pipeline_id, _)) = self.window.parent_info() {
                let top_level_browsing_context_id = self.window.window_proxy().top_level_browsing_context_id();
                let event = ConstellationMsg::MozBrowserEvent(parent_pipeline_id,
                                                              top_level_browsing_context_id,
                                                              event);
                self.window.upcast::<GlobalScope>().constellation_chan().send(event).unwrap();
            }
        }
    }

    /// https://html.spec.whatwg.org/multipage/#dom-window-requestanimationframe
    pub fn request_animation_frame(&self, callback: AnimationFrameCallback) -> u32 {
        let ident = self.animation_frame_ident.get() + 1;

        self.animation_frame_ident.set(ident);
        self.animation_frame_list.borrow_mut().push((ident, Some(callback)));

        // TODO: Should tick animation only when document is visible

        // If we are running 'fake' animation frames, we unconditionally
        // set up a one-shot timer for script to execute the rAF callbacks.
        if self.is_faking_animation_frames() {
            let callback = FakeRequestAnimationFrameCallback {
                document: Trusted::new(self),
            };
            self.global()
                .schedule_callback(OneshotTimerCallback::FakeRequestAnimationFrame(callback),
                                   MsDuration::new(FAKE_REQUEST_ANIMATION_FRAME_DELAY));
        } else if !self.running_animation_callbacks.get() {
            // No need to send a `ChangeRunningAnimationsState` if we're running animation callbacks:
            // we're guaranteed to already be in the "animation callbacks present" state.
            //
            // This reduces CPU usage by avoiding needless thread wakeups in the common case of
            // repeated rAF.

            let global_scope = self.window.upcast::<GlobalScope>();
            let event = ConstellationMsg::ChangeRunningAnimationsState(
                global_scope.pipeline_id(),
                AnimationState::AnimationCallbacksPresent);
            global_scope.constellation_chan().send(event).unwrap();
        }

        ident
    }

    /// https://html.spec.whatwg.org/multipage/#dom-window-cancelanimationframe
    pub fn cancel_animation_frame(&self, ident: u32) {
        let mut list = self.animation_frame_list.borrow_mut();
        if let Some(mut pair) = list.iter_mut().find(|pair| pair.0 == ident) {
            pair.1 = None;
        }
    }

    /// https://html.spec.whatwg.org/multipage/#run-the-animation-frame-callbacks
    pub fn run_the_animation_frame_callbacks(&self) {
        rooted_vec!(let mut animation_frame_list);
        mem::swap(
            &mut *animation_frame_list,
            &mut *self.animation_frame_list.borrow_mut());

        self.running_animation_callbacks.set(true);
        let was_faking_animation_frames = self.is_faking_animation_frames();
        let timing = self.window.Performance().Now();

        for (_, callback) in animation_frame_list.drain(..) {
            if let Some(callback) = callback {
                callback.call(self, *timing);
            }
        }

        self.running_animation_callbacks.set(false);

        let spurious = !self.window.reflow(ReflowGoal::ForDisplay,
                                           ReflowQueryType::NoQuery,
                                           ReflowReason::RequestAnimationFrame);

        if spurious && !was_faking_animation_frames {
            // If the rAF callbacks did not mutate the DOM, then the
            // reflow call above means that layout will not be invoked,
            // and therefore no new frame will be sent to the compositor.
            // If this happens, the compositor will not tick the animation
            // and the next rAF will never be called! When this happens
            // for several frames, then the spurious rAF detection below
            // will kick in and use a timer to tick the callbacks. However,
            // for the interim frames where we are deciding whether this rAF
            // is considered spurious, we need to ensure that the layout
            // and compositor *do* tick the animation.
            self.window.force_reflow(ReflowGoal::ForDisplay,
                                     ReflowQueryType::NoQuery,
                                     ReflowReason::RequestAnimationFrame);
        }

        // Only send the animation change state message after running any callbacks.
        // This means that if the animation callback adds a new callback for
        // the next frame (which is the common case), we won't send a NoAnimationCallbacksPresent
        // message quickly followed by an AnimationCallbacksPresent message.
        //
        // If this frame was spurious and we've seen too many spurious frames in a row, tell the
        // constellation to stop giving us video refresh callbacks, to save energy. (A spurious
        // animation frame is one in which the callback did not mutate the DOM—that is, an
        // animation frame that wasn't actually used for animation.)
        if self.animation_frame_list.borrow().is_empty() ||
                (!was_faking_animation_frames && self.is_faking_animation_frames()) {
            mem::swap(&mut *self.animation_frame_list.borrow_mut(),
                      &mut *animation_frame_list);
            let global_scope = self.window.upcast::<GlobalScope>();
            let event = ConstellationMsg::ChangeRunningAnimationsState(
                global_scope.pipeline_id(),
                AnimationState::NoAnimationCallbacksPresent);
            global_scope.constellation_chan().send(event).unwrap();
        }

        // Update the counter of spurious animation frames.
        if spurious {
            if self.spurious_animation_frames.get() < SPURIOUS_ANIMATION_FRAME_THRESHOLD {
                self.spurious_animation_frames.set(self.spurious_animation_frames.get() + 1)
            }
        } else {
            self.spurious_animation_frames.set(0)
        }
    }

    pub fn fetch_async(&self, load: LoadType,
                       request: RequestInit,
                       fetch_target: IpcSender<FetchResponseMsg>) {
        let mut loader = self.loader.borrow_mut();
        loader.fetch_async(load, request, fetch_target);
    }

    // https://html.spec.whatwg.org/multipage/#the-end
    // https://html.spec.whatwg.org/multipage/#delay-the-load-event
    pub fn finish_load(&self, load: LoadType) {
        // This does not delay the load event anymore.
        debug!("Document got finish_load: {:?}", load);
        self.loader.borrow_mut().finish_load(&load);

        match load {
            LoadType::Stylesheet(_) => {
                // A stylesheet finishing to load may unblock any pending
                // parsing-blocking script or deferred script.
                self.process_pending_parsing_blocking_script();

                // Step 3.
                self.process_deferred_scripts();
            },
            LoadType::PageSource(_) => {
                if self.has_browsing_context {
                    // Disarm the reflow timer and trigger the initial reflow.
                    self.reflow_timeout.set(None);
                    self.upcast::<Node>().dirty(NodeDamage::OtherNodeDamage);
                    self.window.reflow(ReflowGoal::ForDisplay,
                                       ReflowQueryType::NoQuery,
                                       ReflowReason::FirstLoad);
                }

                // Deferred scripts have to wait for page to finish loading,
                // this is the first opportunity to process them.

                // Step 3.
                self.process_deferred_scripts();
            },
            _ => {},
        }

        // Step 4 is in another castle, namely at the end of
        // process_deferred_scripts.

        // Step 5 can be found in asap_script_loaded and
        // asap_in_order_script_loaded.

        let loader = self.loader.borrow();
        if loader.is_blocked() || loader.events_inhibited() {
            // Step 6.
            return;
        }

        ScriptThread::mark_document_with_no_blocked_loads(self);
    }

    // https://html.spec.whatwg.org/multipage/#the-end
    pub fn maybe_queue_document_completion(&self) {
        if self.loader.borrow().is_blocked() {
            // Step 6.
            return;
        }

        assert!(!self.loader.borrow().events_inhibited());
        self.loader.borrow_mut().inhibit_events();

        // The rest will ever run only once per document.
        // Step 7.
        debug!("Document loads are complete.");
        let handler = box DocumentProgressHandler::new(Trusted::new(self));
        self.window.dom_manipulation_task_source().queue(handler, self.window.upcast()).unwrap();

        // Step 8.
        // TODO: pageshow event.

        // Step 9.
        // TODO: pending application cache download process tasks.

        // Step 10.
        // TODO: printing steps.

        // Step 11.
        // TODO: ready for post-load tasks.

        // Step 12.
        // TODO: completely loaded.
    }

    // https://html.spec.whatwg.org/multipage/#pending-parsing-blocking-script
    pub fn set_pending_parsing_blocking_script(&self,
                                               script: &HTMLScriptElement,
                                               load: Option<ScriptResult>) {
        assert!(!self.has_pending_parsing_blocking_script());
        *self.pending_parsing_blocking_script.borrow_mut() = Some(PendingScript::new_with_load(script, load));
    }

    // https://html.spec.whatwg.org/multipage/#pending-parsing-blocking-script
    pub fn has_pending_parsing_blocking_script(&self) -> bool {
        self.pending_parsing_blocking_script.borrow().is_some()
    }

    /// https://html.spec.whatwg.org/multipage/#prepare-a-script step 22.d.
    pub fn pending_parsing_blocking_script_loaded(&self, element: &HTMLScriptElement, result: ScriptResult) {
        {
            let mut blocking_script = self.pending_parsing_blocking_script.borrow_mut();
            let entry = blocking_script.as_mut().unwrap();
            assert!(&*entry.element == element);
            entry.loaded(result);
        }
        self.process_pending_parsing_blocking_script();
    }

    fn process_pending_parsing_blocking_script(&self) {
        if self.script_blocking_stylesheets_count.get() > 0 {
            return;
        }
        let pair = self.pending_parsing_blocking_script
            .borrow_mut()
            .as_mut()
            .and_then(PendingScript::take_result);
        if let Some((element, result)) = pair {
            *self.pending_parsing_blocking_script.borrow_mut() = None;
            self.get_current_parser().unwrap().resume_with_pending_parsing_blocking_script(&element, result);
        }
    }

    // https://html.spec.whatwg.org/multipage/#set-of-scripts-that-will-execute-as-soon-as-possible
    pub fn add_asap_script(&self, script: &HTMLScriptElement) {
        self.asap_scripts_set.borrow_mut().push(JS::from_ref(script));
    }

    /// https://html.spec.whatwg.org/multipage/#the-end step 5.
    /// https://html.spec.whatwg.org/multipage/#prepare-a-script step 22.d.
    pub fn asap_script_loaded(&self, element: &HTMLScriptElement, result: ScriptResult) {
        {
            let mut scripts = self.asap_scripts_set.borrow_mut();
            let idx = scripts.iter().position(|entry| &**entry == element).unwrap();
            scripts.swap_remove(idx);
        }
        element.execute(result);
    }

    // https://html.spec.whatwg.org/multipage/#list-of-scripts-that-will-execute-in-order-as-soon-as-possible
    pub fn push_asap_in_order_script(&self, script: &HTMLScriptElement) {
        self.asap_in_order_scripts_list.push(script);
    }

    /// https://html.spec.whatwg.org/multipage/#the-end step 5.
    /// https://html.spec.whatwg.org/multipage/#prepare-a-script step 22.c.
    pub fn asap_in_order_script_loaded(&self,
                                       element: &HTMLScriptElement,
                                       result: ScriptResult) {
        self.asap_in_order_scripts_list.loaded(element, result);
        while let Some((element, result)) = self.asap_in_order_scripts_list.take_next_ready_to_be_executed() {
            element.execute(result);
        }
    }

    // https://html.spec.whatwg.org/multipage/#list-of-scripts-that-will-execute-when-the-document-has-finished-parsing
    pub fn add_deferred_script(&self, script: &HTMLScriptElement) {
        self.deferred_scripts.push(script);
    }

    /// https://html.spec.whatwg.org/multipage/#the-end step 3.
    /// https://html.spec.whatwg.org/multipage/#prepare-a-script step 22.d.
    pub fn deferred_script_loaded(&self, element: &HTMLScriptElement, result: ScriptResult) {
        self.deferred_scripts.loaded(element, result);
        self.process_deferred_scripts();
    }

    /// https://html.spec.whatwg.org/multipage/#the-end step 3.
    fn process_deferred_scripts(&self) {
        if self.ready_state.get() != DocumentReadyState::Interactive {
            return;
        }
        // Part of substep 1.
        loop {
            if self.script_blocking_stylesheets_count.get() > 0 {
                return;
            }
            if let Some((element, result)) = self.deferred_scripts.take_next_ready_to_be_executed() {
                element.execute(result);
            } else {
                break;
            }
        }
        if self.deferred_scripts.is_empty() {
            // https://html.spec.whatwg.org/multipage/#the-end step 4.
            self.maybe_dispatch_dom_content_loaded();
        }
    }

    // https://html.spec.whatwg.org/multipage/#the-end step 4.
    pub fn maybe_dispatch_dom_content_loaded(&self) {
        if self.domcontentloaded_dispatched.get() {
            return;
        }
        self.domcontentloaded_dispatched.set(true);
        assert!(self.ReadyState() != DocumentReadyState::Complete,
                "Complete before DOMContentLoaded?");

        update_with_current_time_ms(&self.dom_content_loaded_event_start);

        // Step 4.1.
        let window = self.window();
        window.dom_manipulation_task_source().queue_event(self.upcast(), atom!("DOMContentLoaded"),
            EventBubbles::Bubbles, EventCancelable::NotCancelable, window);

        window.reflow(ReflowGoal::ForDisplay,
                      ReflowQueryType::NoQuery,
                      ReflowReason::DOMContentLoaded);
        update_with_current_time_ms(&self.dom_content_loaded_event_end);

        // Step 4.2.
        // TODO: client message queue.
    }

    // https://html.spec.whatwg.org/multipage/#abort-a-document
    fn abort(&self) {
        // We need to inhibit the loader before anything else.
        self.loader.borrow_mut().inhibit_events();

        // Step 1.
        for iframe in self.iter_iframes() {
            if let Some(document) = iframe.GetContentDocument() {
                // TODO: abort the active documents of every child browsing context.
                document.abort();
                // TODO: salvageable flag.
            }
        }

        // Step 2.
        self.script_blocking_stylesheets_count.set(0);
        *self.pending_parsing_blocking_script.borrow_mut() = None;
        *self.asap_scripts_set.borrow_mut() = vec![];
        self.asap_in_order_scripts_list.clear();
        self.deferred_scripts.clear();

        // TODO: https://github.com/servo/servo/issues/15236
        self.window.cancel_all_tasks();

        // Step 3.
        if let Some(parser) = self.get_current_parser() {
            parser.abort();
            // TODO: salvageable flag.
        }
    }

    pub fn notify_constellation_load(&self) {
        let global_scope = self.window.upcast::<GlobalScope>();
        let pipeline_id = global_scope.pipeline_id();
        let load_event = ConstellationMsg::LoadComplete(pipeline_id);
        global_scope.constellation_chan().send(load_event).unwrap();
    }

    pub fn set_current_parser(&self, script: Option<&ServoParser>) {
        self.current_parser.set(script);
    }

    pub fn get_current_parser(&self) -> Option<Root<ServoParser>> {
        self.current_parser.get()
    }

    /// Iterate over all iframes in the document.
    pub fn iter_iframes(&self) -> impl Iterator<Item=Root<HTMLIFrameElement>> {
        self.upcast::<Node>()
            .traverse_preorder()
            .filter_map(Root::downcast::<HTMLIFrameElement>)
    }

    /// Find an iframe element in the document.
    pub fn find_iframe(&self, browsing_context_id: BrowsingContextId) -> Option<Root<HTMLIFrameElement>> {
        self.iter_iframes()
            .find(|node| node.browsing_context_id() == Some(browsing_context_id))
    }

    /// Find a mozbrowser iframe element in the document.
    pub fn find_mozbrowser_iframe(&self,
                                  top_level_browsing_context_id: TopLevelBrowsingContextId)
                                  -> Option<Root<HTMLIFrameElement>>
    {
        match self.find_iframe(BrowsingContextId::from(top_level_browsing_context_id)) {
            None => None,
            Some(iframe) => {
                assert!(iframe.Mozbrowser());
                Some(iframe)
            },
        }
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

    pub fn get_load_event_start(&self) -> u64 {
        self.load_event_start.get()
    }

    pub fn get_load_event_end(&self) -> u64 {
        self.load_event_end.get()
    }

    // https://html.spec.whatwg.org/multipage/#fire-a-focus-event
    fn fire_focus_event(&self, focus_event_type: FocusEventType, node: &Node, related_target: Option<&EventTarget>) {
        let (event_name, does_bubble) = match focus_event_type {
            FocusEventType::Focus => (DOMString::from("focus"), EventBubbles::DoesNotBubble),
            FocusEventType::Blur => (DOMString::from("blur"), EventBubbles::DoesNotBubble),
        };
        let event = FocusEvent::new(&self.window,
                                    event_name,
                                    does_bubble,
                                    EventCancelable::NotCancelable,
                                    Some(&self.window),
                                    0i32,
                                    related_target);
        let event = event.upcast::<Event>();
        event.set_trusted(true);
        let target = node.upcast();
        event.fire(target);
    }

    /// https://html.spec.whatwg.org/multipage/#cookie-averse-document-object
    pub fn is_cookie_averse(&self) -> bool {
        !self.has_browsing_context || !url_has_network_scheme(&self.url())
    }

    pub fn nodes_from_point(&self, client_point: &Point2D<f32>) -> Vec<UntrustedNodeAddress> {
        if !self.window.reflow(ReflowGoal::ForScriptQuery,
                               ReflowQueryType::NodesFromPoint(*client_point),
                               ReflowReason::Query) {
            return vec!();
        };

        self.window.layout().nodes_from_point_response()
    }

    /// https://html.spec.whatwg.org/multipage/#look-up-a-custom-element-definition
    pub fn lookup_custom_element_definition(&self,
                                            local_name: LocalName,
                                            is: Option<LocalName>)
                                            -> Option<Rc<CustomElementDefinition>> {
        if !PREFS.get("dom.customelements.enabled").as_boolean().unwrap_or(false) {
            return None;
        }

        // Step 2
        if !self.has_browsing_context {
            return None;
        }

        // Step 3
        let registry = self.window.CustomElements();

        registry.lookup_definition(local_name, is)
    }
}

#[derive(PartialEq, HeapSizeOf)]
pub enum DocumentSource {
    FromParser,
    NotFromParser,
}

#[allow(unsafe_code)]
pub trait LayoutDocumentHelpers {
    unsafe fn is_html_document_for_layout(&self) -> bool;
    unsafe fn drain_pending_restyles(&self) -> Vec<(LayoutJS<Element>, PendingRestyle)>;
    unsafe fn needs_paint_from_layout(&self);
    unsafe fn will_paint(&self);
    unsafe fn quirks_mode(&self) -> QuirksMode;
    unsafe fn style_shared_lock(&self) -> &StyleSharedRwLock;
}

#[allow(unsafe_code)]
impl LayoutDocumentHelpers for LayoutJS<Document> {
    #[inline]
    unsafe fn is_html_document_for_layout(&self) -> bool {
        (*self.unsafe_get()).is_html_document
    }

    #[inline]
    #[allow(unrooted_must_root)]
    unsafe fn drain_pending_restyles(&self) -> Vec<(LayoutJS<Element>, PendingRestyle)> {
        let mut elements = (*self.unsafe_get()).pending_restyles.borrow_mut_for_layout();
        // Elements were in a document when they were adding to this list, but that
        // may no longer be true when the next layout occurs.
        let result = elements.drain()
            .map(|(k, v)| (k.to_layout(), v))
            .filter(|&(ref k, _)| k.upcast::<Node>().get_flag(IS_IN_DOC))
            .collect();
        result
    }

    #[inline]
    unsafe fn needs_paint_from_layout(&self) {
        (*self.unsafe_get()).needs_paint.set(true)
    }

    #[inline]
    unsafe fn will_paint(&self) {
        (*self.unsafe_get()).needs_paint.set(false)
    }

    #[inline]
    unsafe fn quirks_mode(&self) -> QuirksMode {
        (*self.unsafe_get()).quirks_mode()
    }

    #[inline]
    unsafe fn style_shared_lock(&self) -> &StyleSharedRwLock {
        (*self.unsafe_get()).style_shared_lock()
    }
}

// https://html.spec.whatwg.org/multipage/#is-a-registrable-domain-suffix-of-or-is-equal-to
// The spec says to return a bool, we actually return an Option<Host> containing
// the parsed host in the successful case, to avoid having to re-parse the host.
fn get_registrable_domain_suffix_of_or_is_equal_to(host_suffix_string: &str, original_host: Host) -> Option<Host> {
    // Step 1
    if host_suffix_string.is_empty() {
        return None;
    }

    // Step 2-3.
    let host = match Host::parse(host_suffix_string) {
        Ok(host) => host,
        Err(_) => return None,
    };

    // Step 4.
    if host != original_host {
        // Step 4.1
        let host = match host {
            Host::Domain(ref host) => host,
            _ => return None,
        };
        let original_host = match original_host {
            Host::Domain(ref original_host) => original_host,
            _ => return None,
        };

        // Step 4.2
        let (prefix, suffix) = match original_host.len().checked_sub(host.len()) {
            Some(index) => original_host.split_at(index),
            None => return None,
        };
        if !prefix.ends_with(".") {
            return None;
        }
        if suffix != host {
            return None;
        }

        // Step 4.3
        if is_pub_domain(host) {
            return None;
        }
    }

    // Step 5
    Some(host)
}

/// https://url.spec.whatwg.org/#network-scheme
fn url_has_network_scheme(url: &ServoUrl) -> bool {
    match url.scheme() {
        "ftp" | "http" | "https" => true,
        _ => false,
    }
}

#[derive(Copy, Clone, HeapSizeOf, JSTraceable, PartialEq, Eq)]
pub enum HasBrowsingContext {
    No,
    Yes,
}

impl Document {
    pub fn new_inherited(window: &Window,
                         has_browsing_context: HasBrowsingContext,
                         url: Option<ServoUrl>,
                         origin: MutableOrigin,
                         is_html_document: IsHTMLDocument,
                         content_type: Option<DOMString>,
                         last_modified: Option<String>,
                         activity: DocumentActivity,
                         source: DocumentSource,
                         doc_loader: DocumentLoader,
                         referrer: Option<String>,
                         referrer_policy: Option<ReferrerPolicy>)
                         -> Document {
        let url = url.unwrap_or_else(|| ServoUrl::parse("about:blank").unwrap());

        let (ready_state, domcontentloaded_dispatched) = if source == DocumentSource::FromParser {
            (DocumentReadyState::Loading, false)
        } else {
            (DocumentReadyState::Complete, true)
        };

        Document {
            node: Node::new_document_node(),
            window: JS::from_ref(window),
            has_browsing_context: has_browsing_context == HasBrowsingContext::Yes,
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
            url: DOMRefCell::new(url),
            // https://dom.spec.whatwg.org/#concept-document-quirks
            quirks_mode: Cell::new(QuirksMode::NoQuirks),
            // https://dom.spec.whatwg.org/#concept-document-encoding
            encoding: Cell::new(UTF_8),
            is_html_document: is_html_document == IsHTMLDocument::HTMLDocument,
            activity: Cell::new(activity),
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
            style_shared_lock: {
                lazy_static! {
                    /// Per-process shared lock for author-origin stylesheets
                    ///
                    /// FIXME: make it per-document or per-pipeline instead:
                    /// https://github.com/servo/servo/issues/16027
                    /// (Need to figure out what to do with the style attribute
                    /// of elements adopted into another document.)
                    static ref PER_PROCESS_AUTHOR_SHARED_LOCK: StyleSharedRwLock = {
                        StyleSharedRwLock::new()
                    };
                }
                PER_PROCESS_AUTHOR_SHARED_LOCK.clone()
                //StyleSharedRwLock::new()
            },
            stylesheets: DOMRefCell::new(None),
            stylesheets_changed_since_reflow: Cell::new(false),
            stylesheet_list: MutNullableJS::new(None),
            ready_state: Cell::new(ready_state),
            domcontentloaded_dispatched: Cell::new(domcontentloaded_dispatched),
            possibly_focused: Default::default(),
            focused: Default::default(),
            current_script: Default::default(),
            pending_parsing_blocking_script: Default::default(),
            script_blocking_stylesheets_count: Cell::new(0u32),
            deferred_scripts: Default::default(),
            asap_in_order_scripts_list: Default::default(),
            asap_scripts_set: Default::default(),
            scripting_enabled: has_browsing_context == HasBrowsingContext::Yes,
            animation_frame_ident: Cell::new(0),
            animation_frame_list: DOMRefCell::new(vec![]),
            running_animation_callbacks: Cell::new(false),
            loader: DOMRefCell::new(doc_loader),
            current_parser: Default::default(),
            reflow_timeout: Cell::new(None),
            base_element: Default::default(),
            appropriate_template_contents_owner_document: Default::default(),
            pending_restyles: DOMRefCell::new(HashMap::new()),
            needs_paint: Cell::new(false),
            active_touch_points: DOMRefCell::new(Vec::new()),
            dom_loading: Cell::new(Default::default()),
            dom_interactive: Cell::new(Default::default()),
            dom_content_loaded_event_start: Cell::new(Default::default()),
            dom_content_loaded_event_end: Cell::new(Default::default()),
            dom_complete: Cell::new(Default::default()),
            load_event_start: Cell::new(Default::default()),
            load_event_end: Cell::new(Default::default()),
            https_state: Cell::new(HttpsState::None),
            touchpad_pressure_phase: Cell::new(TouchpadPressurePhase::BeforeClick),
            origin: origin,
            referrer: referrer,
            referrer_policy: Cell::new(referrer_policy),
            target_element: MutNullableJS::new(None),
            last_click_info: DOMRefCell::new(None),
            ignore_destructive_writes_counter: Default::default(),
            spurious_animation_frames: Cell::new(0),
            dom_count: Cell::new(1),
            fullscreen_element: MutNullableJS::new(None),
            form_id_listener_map: Default::default(),
        }
    }

    // https://dom.spec.whatwg.org/#dom-document-document
    pub fn Constructor(window: &Window) -> Fallible<Root<Document>> {
        let doc = window.Document();
        let docloader = DocumentLoader::new(&*doc.loader());
        Ok(Document::new(window,
                         HasBrowsingContext::No,
                         None,
                         doc.origin().clone(),
                         IsHTMLDocument::NonHTMLDocument,
                         None,
                         None,
                         DocumentActivity::Inactive,
                         DocumentSource::NotFromParser,
                         docloader,
                         None,
                         None))
    }

    pub fn new(window: &Window,
               has_browsing_context: HasBrowsingContext,
               url: Option<ServoUrl>,
               origin: MutableOrigin,
               doctype: IsHTMLDocument,
               content_type: Option<DOMString>,
               last_modified: Option<String>,
               activity: DocumentActivity,
               source: DocumentSource,
               doc_loader: DocumentLoader,
               referrer: Option<String>,
               referrer_policy: Option<ReferrerPolicy>)
               -> Root<Document> {
        let document = reflect_dom_object(box Document::new_inherited(window,
                                                                      has_browsing_context,
                                                                      url,
                                                                      origin,
                                                                      doctype,
                                                                      content_type,
                                                                      last_modified,
                                                                      activity,
                                                                      source,
                                                                      doc_loader,
                                                                      referrer,
                                                                      referrer_policy),
                                          window,
                                          DocumentBinding::Wrap);
        {
            let node = document.upcast::<Node>();
            node.set_owner_doc(&document);
        }
        document
    }

    fn create_node_list<F: Fn(&Node) -> bool>(&self, callback: F) -> Root<NodeList> {
        let doc = self.GetDocumentElement();
        let maybe_node = doc.r().map(Castable::upcast::<Node>);
        let iter = maybe_node.iter()
                             .flat_map(|node| node.traverse_preorder())
                             .filter(|node| callback(&node));
        NodeList::new_simple_list(&self.window, iter)
    }

    fn get_html_element(&self) -> Option<Root<HTMLHtmlElement>> {
        self.GetDocumentElement().and_then(Root::downcast)
    }

    // Ensure that the stylesheets vector is populated
    fn ensure_stylesheets(&self) {
        let mut stylesheets = self.stylesheets.borrow_mut();
        if stylesheets.is_none() {
            *stylesheets = Some(self.upcast::<Node>()
                .traverse_preorder()
                .filter_map(|node| {
                    node.get_stylesheet()
                        .map(|stylesheet| StylesheetInDocument {
                        node: JS::from_ref(&*node),
                        stylesheet: stylesheet,
                    })
                })
                .collect());
        };
    }

    /// Return a reference to the per-document shared lock used in stylesheets.
    pub fn style_shared_lock(&self) -> &StyleSharedRwLock {
        &self.style_shared_lock
    }

    /// Returns the list of stylesheets associated with nodes in the document.
    pub fn stylesheets(&self) -> Vec<Arc<Stylesheet>> {
        self.ensure_stylesheets();
        self.stylesheets.borrow().as_ref().unwrap().iter()
                        .map(|s| s.stylesheet.clone())
                        .collect()
    }

    pub fn with_style_sheets_in_document<F, T>(&self, mut f: F) -> T
            where F: FnMut(&[StylesheetInDocument]) -> T {
        self.ensure_stylesheets();
        f(&self.stylesheets.borrow().as_ref().unwrap())
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
                                        HasBrowsingContext::No,
                                        None,
                                        // https://github.com/whatwg/html/issues/2109
                                        MutableOrigin::new(ImmutableOrigin::new_opaque()),
                                        doctype,
                                        None,
                                        None,
                                        DocumentActivity::Inactive,
                                        DocumentSource::NotFromParser,
                                        DocumentLoader::new(&self.loader()),
                                        None,
                                        None);
            new_doc.appropriate_template_contents_owner_document.set(Some(&new_doc));
            new_doc
        })
    }

    pub fn get_element_by_id(&self, id: &Atom) -> Option<Root<Element>> {
        self.id_map.borrow().get(&id).map(|ref elements| Root::from_ref(&*(*elements)[0]))
    }

    pub fn ensure_pending_restyle(&self, el: &Element) -> RefMut<PendingRestyle> {
        let map = self.pending_restyles.borrow_mut();
        RefMut::map(map, |m| m.entry(JS::from_ref(el)).or_insert_with(PendingRestyle::new))
    }

    pub fn element_state_will_change(&self, el: &Element) {
        let mut entry = self.ensure_pending_restyle(el);
        if entry.snapshot.is_none() {
            entry.snapshot = Some(Snapshot::new(el.html_element_in_html_document()));
        }
        let mut snapshot = entry.snapshot.as_mut().unwrap();
        if snapshot.state.is_none() {
            snapshot.state = Some(el.state());
        }
    }

    pub fn element_attr_will_change(&self, el: &Element, attr: &Attr) {
        // FIXME(emilio): Kind of a shame we have to duplicate this.
        //
        // I'm getting rid of the whole hashtable soon anyway, since all it does
        // right now is populate the element restyle data in layout, and we
        // could in theory do it in the DOM I think.
        let mut entry = self.ensure_pending_restyle(el);
        if entry.snapshot.is_none() {
            entry.snapshot = Some(Snapshot::new(el.html_element_in_html_document()));
        }
        if attr.local_name() == &local_name!("style") {
            entry.hint.insert(RESTYLE_STYLE_ATTRIBUTE);
        }

        // FIXME(emilio): This should become something like
        // element.is_attribute_mapped(attr.local_name()).
        if attr.local_name() == &local_name!("width") ||
           attr.local_name() == &local_name!("height") {
            entry.hint.insert(RESTYLE_SELF);
        }

        let mut snapshot = entry.snapshot.as_mut().unwrap();
        if attr.local_name() == &local_name!("id") {
            snapshot.id_changed = true;
        } else if attr.local_name() == &local_name!("class") {
            snapshot.class_changed = true;
        } else {
            snapshot.other_attributes_changed = true;
        }
        if snapshot.attrs.is_none() {
            let attrs = el.attrs()
                          .iter()
                          .map(|attr| (attr.identifier().clone(), attr.value().clone()))
                          .collect();
            snapshot.attrs = Some(attrs);
        }
    }

    pub fn set_referrer_policy(&self, policy: Option<ReferrerPolicy>) {
        self.referrer_policy.set(policy);
    }

    //TODO - default still at no-referrer
    pub fn get_referrer_policy(&self) -> Option<ReferrerPolicy> {
        return self.referrer_policy.get();
    }

    pub fn set_target_element(&self, node: Option<&Element>) {
        if let Some(ref element) = self.target_element.get() {
            element.set_target_state(false);
        }

        self.target_element.set(node);

        if let Some(ref element) = self.target_element.get() {
            element.set_target_state(true);
        }

        self.window.reflow(ReflowGoal::ForDisplay,
                           ReflowQueryType::NoQuery,
                           ReflowReason::ElementStateChanged);
    }

    pub fn incr_ignore_destructive_writes_counter(&self) {
        self.ignore_destructive_writes_counter.set(
            self.ignore_destructive_writes_counter.get() + 1);
    }

    pub fn decr_ignore_destructive_writes_counter(&self) {
        self.ignore_destructive_writes_counter.set(
            self.ignore_destructive_writes_counter.get() - 1);
    }

    /// Whether we've seen so many spurious animation frames (i.e. animation frames that didn't
    /// mutate the DOM) that we've decided to fall back to fake ones.
    fn is_faking_animation_frames(&self) -> bool {
        self.spurious_animation_frames.get() >= SPURIOUS_ANIMATION_FRAME_THRESHOLD
    }

    // https://fullscreen.spec.whatwg.org/#dom-element-requestfullscreen
    #[allow(unrooted_must_root)]
    pub fn enter_fullscreen(&self, pending: &Element) -> Rc<Promise> {
        // Step 1
        let promise = Promise::new(self.global().r());
        let mut error = false;

        // Step 4
        // check namespace
        match *pending.namespace() {
            ns!(mathml) => {
                if pending.local_name().as_ref() != "math" {
                    error = true;
                }
            }
            ns!(svg) => {
                if pending.local_name().as_ref() != "svg" {
                    error = true;
                }
            }
            ns!(html) => (),
            _ => error = true,
        }
        // fullscreen element ready check
        if !pending.fullscreen_element_ready_check() {
            error = true;
        }
        // TODO fullscreen is supported
        // TODO This algorithm is allowed to request fullscreen.

        // Step 5 Parallel start

        let window = self.window();
        // Step 6
        if !error {
            let event = ConstellationMsg::SetFullscreenState(true);
            window.upcast::<GlobalScope>().constellation_chan().send(event).unwrap();
        }

        // Step 7
        let trusted_pending = Trusted::new(pending);
        let trusted_promise = TrustedPromise::new(promise.clone());
        let handler = ElementPerformFullscreenEnter::new(trusted_pending, trusted_promise, error);
        let script_msg = CommonScriptMsg::RunnableMsg(ScriptThreadEventCategory::EnterFullscreen, handler);
        let msg = MainThreadScriptMsg::Common(script_msg);
        window.main_thread_script_chan().send(msg).unwrap();

        promise
    }

    // https://fullscreen.spec.whatwg.org/#exit-fullscreen
    #[allow(unrooted_must_root)]
    pub fn exit_fullscreen(&self) -> Rc<Promise> {
        let global = self.global();
        // Step 1
        let promise = Promise::new(global.r());
        // Step 2
        if self.fullscreen_element.get().is_none() {
            promise.reject_error(global.get_cx(), Error::Type(String::from("fullscreen is null")));
            return promise
        }
        // TODO Step 3-6
        let element = self.fullscreen_element.get().unwrap();

        // Step 7 Parallel start

        let window = self.window();
        // Step 8
        let event = ConstellationMsg::SetFullscreenState(false);
        window.upcast::<GlobalScope>().constellation_chan().send(event).unwrap();

        // Step 9
        let trusted_element = Trusted::new(element.r());
        let trusted_promise = TrustedPromise::new(promise.clone());
        let handler = ElementPerformFullscreenExit::new(trusted_element, trusted_promise);
        let script_msg = CommonScriptMsg::RunnableMsg(ScriptThreadEventCategory::ExitFullscreen, handler);
        let msg = MainThreadScriptMsg::Common(script_msg);
        window.main_thread_script_chan().send(msg).unwrap();

        promise
    }

    pub fn set_fullscreen_element(&self, element: Option<&Element>) {
        self.fullscreen_element.set(element);
    }

    pub fn get_allow_fullscreen(&self) -> bool {
        // https://html.spec.whatwg.org/multipage/#allowed-to-use
        match self.browsing_context() {
            // Step 1
            None => false,
            Some(_) => {
                // Step 2
                let window = self.window();
                if window.is_top_level() {
                    true
                } else {
                    // Step 3
                    window.GetFrameElement().map_or(false, |el| el.has_attribute(&local_name!("allowfullscreen")))
                }
            }
        }
    }

    fn reset_form_owner_for_listeners(&self, id: &Atom) {
        let map = self.form_id_listener_map.borrow();
        if let Some(listeners) = map.get(id) {
            for listener in listeners {
                listener.r().as_maybe_form_control()
                        .expect("Element must be a form control")
                        .reset_form_owner();
            }
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
                if self.disabled_state() => true,
            _ => false,
        }
    }
}

impl DocumentMethods for Document {
    // https://drafts.csswg.org/cssom/#dom-document-stylesheets
    fn StyleSheets(&self) -> Root<StyleSheetList> {
        self.stylesheet_list.or_init(|| StyleSheetList::new(&self.window, JS::from_ref(&self)))
    }

    // https://dom.spec.whatwg.org/#dom-document-implementation
    fn Implementation(&self) -> Root<DOMImplementation> {
        self.implementation.or_init(|| DOMImplementation::new(self))
    }

    // https://dom.spec.whatwg.org/#dom-document-url
    fn URL(&self) -> USVString {
        USVString(String::from(self.url().as_str()))
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
        // Step 1-2.
        if self.window().parent_info().is_none() && self.is_fully_active() {
            return true;
        }
        // TODO Step 3.
        false
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-domain
    fn Domain(&self) -> DOMString {
        // Step 1.
        if !self.has_browsing_context {
            return DOMString::new();
        }

        // Step 2.
        match self.origin.effective_domain() {
            // Step 3.
            None => DOMString::new(),
            // Step 4.
            Some(Host::Domain(domain)) => DOMString::from(domain),
            Some(host) => DOMString::from(host.to_string()),
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-domain
    fn SetDomain(&self, value: DOMString) -> ErrorResult {
        // Step 1.
        if !self.has_browsing_context {
            return Err(Error::Security);
        }

        // TODO: Step 2. "If this Document object's active sandboxing
        // flag set has its sandboxed document.domain browsing context
        // flag set, then throw a "SecurityError" DOMException."

        // Steps 3-4.
        let effective_domain = match self.origin.effective_domain() {
            Some(effective_domain) => effective_domain,
            None => return Err(Error::Security),
        };

        // Step 5
        let host = match get_registrable_domain_suffix_of_or_is_equal_to(&*value, effective_domain) {
            None => return Err(Error::Security),
            Some(host) => host,
        };

        // Step 6
        self.origin.set_domain(host);

        Ok(())
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-referrer
    fn Referrer(&self) -> DOMString {
        match self.referrer {
            Some(ref referrer) => DOMString::from(referrer.to_string()),
            None => DOMString::new()
        }
    }

    // https://dom.spec.whatwg.org/#dom-document-documenturi
    fn DocumentURI(&self) -> USVString {
        self.URL()
    }

    // https://dom.spec.whatwg.org/#dom-document-compatmode
    fn CompatMode(&self) -> DOMString {
        DOMString::from(match self.quirks_mode.get() {
            QuirksMode::LimitedQuirks | QuirksMode::NoQuirks => "CSS1Compat",
            QuirksMode::Quirks => "BackCompat",
        })
    }

    // https://dom.spec.whatwg.org/#dom-document-characterset
    fn CharacterSet(&self) -> DOMString {
        DOMString::from(match self.encoding.get().name() {
            "utf-8"         => "UTF-8",
            "ibm866"        => "IBM866",
            "iso-8859-2"    => "ISO-8859-2",
            "iso-8859-3"    => "ISO-8859-3",
            "iso-8859-4"    => "ISO-8859-4",
            "iso-8859-5"    => "ISO-8859-5",
            "iso-8859-6"    => "ISO-8859-6",
            "iso-8859-7"    => "ISO-8859-7",
            "iso-8859-8"    => "ISO-8859-8",
            "iso-8859-8-i"  => "ISO-8859-8-I",
            "iso-8859-10"   => "ISO-8859-10",
            "iso-8859-13"   => "ISO-8859-13",
            "iso-8859-14"   => "ISO-8859-14",
            "iso-8859-15"   => "ISO-8859-15",
            "iso-8859-16"   => "ISO-8859-16",
            "koi8-r"        => "KOI8-R",
            "koi8-u"        => "KOI8-U",
            "gbk"           => "GBK",
            "big5"          => "Big5",
            "euc-jp"        => "EUC-JP",
            "iso-2022-jp"   => "ISO-2022-JP",
            "shift_jis"     => "Shift_JIS",
            "euc-kr"        => "EUC-KR",
            "utf-16be"      => "UTF-16BE",
            "utf-16le"      => "UTF-16LE",
            name            => name
        })
    }

    // https://dom.spec.whatwg.org/#dom-document-charset
    fn Charset(&self) -> DOMString {
        self.CharacterSet()
    }

    // https://dom.spec.whatwg.org/#dom-document-inputencoding
    fn InputEncoding(&self) -> DOMString {
        self.CharacterSet()
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
    fn GetElementsByTagName(&self, qualified_name: DOMString) -> Root<HTMLCollection> {
        let qualified_name = LocalName::from(&*qualified_name);
        match self.tag_map.borrow_mut().entry(qualified_name.clone()) {
            Occupied(entry) => Root::from_ref(entry.get()),
            Vacant(entry) => {
                let result = HTMLCollection::by_qualified_name(
                    &self.window, self.upcast(), qualified_name);
                entry.insert(JS::from_ref(&*result));
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
        let local = LocalName::from(tag_name);
        let qname = QualName::new(None, ns, local);
        match self.tagns_map.borrow_mut().entry(qname.clone()) {
            Occupied(entry) => Root::from_ref(entry.get()),
            Vacant(entry) => {
                let result = HTMLCollection::by_qual_tag_name(&self.window, self.upcast(), qname);
                entry.insert(JS::from_ref(&*result));
                result
            }
        }
    }

    // https://dom.spec.whatwg.org/#dom-document-getelementsbyclassname
    fn GetElementsByClassName(&self, classes: DOMString) -> Root<HTMLCollection> {
        let class_atoms: Vec<Atom> = split_html_space_chars(&classes)
                                         .map(Atom::from)
                                         .collect();
        match self.classes_map.borrow_mut().entry(class_atoms.clone()) {
            Occupied(entry) => Root::from_ref(entry.get()),
            Vacant(entry) => {
                let result = HTMLCollection::by_atomic_class_name(&self.window,
                                                                  self.upcast(),
                                                                  class_atoms);
                entry.insert(JS::from_ref(&*result));
                result
            }
        }
    }

    // https://dom.spec.whatwg.org/#dom-nonelementparentnode-getelementbyid
    fn GetElementById(&self, id: DOMString) -> Option<Root<Element>> {
        self.get_element_by_id(&Atom::from(id))
    }

    // https://dom.spec.whatwg.org/#dom-document-createelement
    fn CreateElement(&self,
                     mut local_name: DOMString,
                     options: &ElementCreationOptions)
                     -> Fallible<Root<Element>> {
        if xml_name_type(&local_name) == InvalidXMLName {
            debug!("Not a valid element name");
            return Err(Error::InvalidCharacter);
        }
        if self.is_html_document {
            local_name.make_ascii_lowercase();
        }

        let ns = if self.is_html_document || self.content_type == "application/xhtml+xml" {
            ns!(html)
        } else {
            ns!()
        };

        let name = QualName::new(None, ns, LocalName::from(local_name));
        let is = options.is.as_ref().map(|is| LocalName::from(&**is));
        Ok(Element::create(name, is, self, ElementCreator::ScriptCreated, CustomElementCreationMode::Synchronous))
    }

    // https://dom.spec.whatwg.org/#dom-document-createelementns
    fn CreateElementNS(&self,
                       namespace: Option<DOMString>,
                       qualified_name: DOMString,
                       options: &ElementCreationOptions)
                       -> Fallible<Root<Element>> {
        let (namespace, prefix, local_name) = validate_and_extract(namespace,
                                                                        &qualified_name)?;
        let name = QualName::new(prefix, namespace, local_name);
        let is = options.is.as_ref().map(|is| LocalName::from(&**is));
        Ok(Element::create(name, is, self, ElementCreator::ScriptCreated, CustomElementCreationMode::Synchronous))
    }

    // https://dom.spec.whatwg.org/#dom-document-createattribute
    fn CreateAttribute(&self, mut local_name: DOMString) -> Fallible<Root<Attr>> {
        if xml_name_type(&local_name) == InvalidXMLName {
            debug!("Not a valid element name");
            return Err(Error::InvalidCharacter);
        }
        if self.is_html_document {
            local_name.make_ascii_lowercase();
        }
        let name = LocalName::from(local_name);
        let value = AttrValue::String("".to_owned());

        Ok(Attr::new(&self.window, name.clone(), value, name, ns!(), None, None))
    }

    // https://dom.spec.whatwg.org/#dom-document-createattributens
    fn CreateAttributeNS(&self,
                         namespace: Option<DOMString>,
                         qualified_name: DOMString)
                         -> Fallible<Root<Attr>> {
        let (namespace, prefix, local_name) = validate_and_extract(namespace,
                                                                        &qualified_name)?;
        let value = AttrValue::String("".to_owned());
        let qualified_name = LocalName::from(qualified_name);
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
        let clone_children = if deep {
            CloneChildrenFlag::CloneChildren
        } else {
            CloneChildrenFlag::DoNotCloneChildren
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
            "beforeunloadevent" =>
                Ok(Root::upcast(BeforeUnloadEvent::new_uninitialized(&self.window))),
            "closeevent" =>
                Ok(Root::upcast(CloseEvent::new_uninitialized(self.window.upcast()))),
            "customevent" =>
                Ok(Root::upcast(CustomEvent::new_uninitialized(self.window.upcast()))),
            "errorevent" =>
                Ok(Root::upcast(ErrorEvent::new_uninitialized(self.window.upcast()))),
            "events" | "event" | "htmlevents" | "svgevents" =>
                Ok(Event::new_uninitialized(&self.window.upcast())),
            "focusevent" =>
                Ok(Root::upcast(FocusEvent::new_uninitialized(&self.window))),
            "hashchangeevent" =>
                Ok(Root::upcast(HashChangeEvent::new_uninitialized(&self.window))),
            "keyboardevent" =>
                Ok(Root::upcast(KeyboardEvent::new_uninitialized(&self.window))),
            "messageevent" =>
                Ok(Root::upcast(MessageEvent::new_uninitialized(self.window.upcast()))),
            "mouseevent" | "mouseevents" =>
                Ok(Root::upcast(MouseEvent::new_uninitialized(&self.window))),
            "pagetransitionevent" =>
                Ok(Root::upcast(PageTransitionEvent::new_uninitialized(&self.window))),
            "popstateevent" =>
                Ok(Root::upcast(PopStateEvent::new_uninitialized(&self.window))),
            "progressevent" =>
                Ok(Root::upcast(ProgressEvent::new_uninitialized(self.window.upcast()))),
            "storageevent" => {
                Ok(Root::upcast(StorageEvent::new_uninitialized(&self.window, "".into())))
            },
            "touchevent" =>
                Ok(Root::upcast(
                    TouchEvent::new_uninitialized(&self.window,
                        &TouchList::new(&self.window, &[]),
                        &TouchList::new(&self.window, &[]),
                        &TouchList::new(&self.window, &[]),
                    )
                )),
            "uievent" | "uievents" =>
                Ok(Root::upcast(UIEvent::new_uninitialized(&self.window))),
            "webglcontextevent" =>
                Ok(Root::upcast(WebGLContextEvent::new_uninitialized(&self.window))),
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
                          what_to_show: u32,
                          filter: Option<Rc<NodeFilter>>)
                          -> Root<NodeIterator> {
        NodeIterator::new(self, root, what_to_show, filter)
    }

    // https://w3c.github.io/touch-events/#idl-def-Document
    fn CreateTouch(&self,
                   window: &Window,
                   target: &EventTarget,
                   identifier: i32,
                   page_x: Finite<f64>,
                   page_y: Finite<f64>,
                   screen_x: Finite<f64>,
                   screen_y: Finite<f64>)
                   -> Root<Touch> {
        let client_x = Finite::wrap(*page_x - window.PageXOffset() as f64);
        let client_y = Finite::wrap(*page_y - window.PageYOffset() as f64);
        Touch::new(window,
                   identifier,
                   target,
                   screen_x,
                   screen_y,
                   client_x,
                   client_y,
                   page_x,
                   page_y)
    }

    // https://w3c.github.io/touch-events/#idl-def-document-createtouchlist(touch...)
    fn CreateTouchList(&self, touches: &[&Touch]) -> Root<TouchList> {
        TouchList::new(&self.window, &touches)
    }

    // https://dom.spec.whatwg.org/#dom-document-createtreewalker
    fn CreateTreeWalker(&self,
                        root: &Node,
                        what_to_show: u32,
                        filter: Option<Rc<NodeFilter>>)
                        -> Root<TreeWalker> {
        TreeWalker::new(self, root, what_to_show, filter)
    }

    // https://html.spec.whatwg.org/multipage/#document.title
    fn Title(&self) -> DOMString {
        let title = self.GetDocumentElement().and_then(|root| {
            if root.namespace() == &ns!(svg) && root.local_name() == &local_name!("svg") {
                // Step 1.
                root.upcast::<Node>()
                    .child_elements()
                    .find(|node| {
                        node.namespace() == &ns!(svg) && node.local_name() == &local_name!("title")
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
                let value = title.child_text_content();
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

        let elem = if root.namespace() == &ns!(svg) && root.local_name() == &local_name!("svg") {
            let elem = root.upcast::<Node>().child_elements().find(|node| {
                node.namespace() == &ns!(svg) && node.local_name() == &local_name!("title")
            });
            match elem {
                Some(elem) => Root::upcast::<Node>(elem),
                None => {
                    let name = QualName::new(None, ns!(svg), local_name!("title"));
                    let elem = Element::create(name,
                                               None,
                                               self,
                                               ElementCreator::ScriptCreated,
                                               CustomElementCreationMode::Synchronous);
                    let parent = root.upcast::<Node>();
                    let child = elem.upcast::<Node>();
                    parent.InsertBefore(child, parent.GetFirstChild().r())
                          .unwrap()
                }
            }
        } else if root.namespace() == &ns!(html) {
            let elem = root.upcast::<Node>()
                           .traverse_preorder()
                           .find(|node| node.is::<HTMLTitleElement>());
            match elem {
                Some(elem) => elem,
                None => {
                    match self.GetHead() {
                        Some(head) => {
                            let name = QualName::new(None, ns!(html), local_name!("title"));
                            let elem = Element::create(name,
                                                       None,
                                                       self,
                                                       ElementCreator::ScriptCreated,
                                                       CustomElementCreationMode::Synchronous);
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
        if old_body.r() == Some(new_body) {
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
            if element.namespace() != &ns!(html) {
                return false;
            }
            element.get_attribute(&ns!(), &local_name!("name"))
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
    fn GetLocation(&self) -> Option<Root<Location>> {
        self.browsing_context().map(|_| self.location.or_init(|| Location::new(&self.window)))
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
    fn GetDefaultView(&self) -> Option<Root<Window>> {
        if self.has_browsing_context {
            Some(Root::from_ref(&*self.window))
        } else {
            None
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-cookie
    fn GetCookie(&self) -> Fallible<DOMString> {
        if self.is_cookie_averse() {
            return Ok(DOMString::new());
        }

        if !self.origin.is_tuple() {
            return Err(Error::Security);
        }

        let url = self.url();
        let (tx, rx) = ipc::channel().unwrap();
        let _ = self.window
            .upcast::<GlobalScope>()
            .resource_threads()
            .send(GetCookiesForUrl(url, tx, NonHTTP));
        let cookies = rx.recv().unwrap();
        Ok(cookies.map_or(DOMString::new(), DOMString::from))
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-cookie
    fn SetCookie(&self, cookie: DOMString) -> ErrorResult {
        if self.is_cookie_averse() {
            return Ok(());
        }

        if !self.origin.is_tuple() {
            return Err(Error::Security);
        }

        if let Ok(cookie_header) = SetCookie::parse_header(&vec![cookie.to_string().into_bytes()]) {
            let cookies = cookie_header.0.into_iter().filter_map(|cookie| {
                cookie_rs::Cookie::parse(cookie).ok().map(Serde)
            }).collect();
            let _ = self.window
                    .upcast::<GlobalScope>()
                    .resource_threads()
                    .send(SetCookiesForUrl(self.url(), cookies, NonHTTP));
        }
        Ok(())
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-bgcolor
    fn BgColor(&self) -> DOMString {
        self.get_body_attribute(&local_name!("bgcolor"))
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-bgcolor
    fn SetBgColor(&self, value: DOMString) {
        self.set_body_attribute(&local_name!("bgcolor"), value)
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-fgcolor
    fn FgColor(&self) -> DOMString {
        self.get_body_attribute(&local_name!("text"))
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-fgcolor
    fn SetFgColor(&self, value: DOMString) {
        self.set_body_attribute(&local_name!("text"), value)
    }

    #[allow(unsafe_code)]
    // https://html.spec.whatwg.org/multipage/#dom-tree-accessors:dom-document-nameditem-filter
    unsafe fn NamedGetter(&self, _cx: *mut JSContext, name: DOMString) -> Option<NonZero<*mut JSObject>> {
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
                    match elem.get_attribute(&ns!(), &local_name!("name")) {
                        Some(ref attr) if attr.value().as_atom() == name => true,
                        _ => {
                            match elem.get_attribute(&ns!(), &local_name!("id")) {
                                Some(ref attr) => attr.value().as_atom() == name,
                                None => false,
                            }
                        },
                    }
                },
                HTMLElementTypeId::HTMLFormElement => {
                    match elem.get_attribute(&ns!(), &local_name!("name")) {
                        Some(ref attr) => attr.value().as_atom() == name,
                        None => false,
                    }
                },
                HTMLElementTypeId::HTMLImageElement => {
                    match elem.get_attribute(&ns!(), &local_name!("name")) {
                        Some(ref attr) => {
                            if attr.value().as_atom() == name {
                                true
                            } else {
                                match elem.get_attribute(&ns!(), &local_name!("id")) {
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
        let name = Atom::from(name);
        let root = self.upcast::<Node>();
        {
            // Step 1.
            let mut elements = root.traverse_preorder()
                                   .filter(|node| filter_by_name(&name, &node))
                                   .peekable();
            if let Some(first) = elements.next() {
                if elements.peek().is_none() {
                    // TODO: Step 2.
                    // Step 3.
                    return Some(NonZero::new(first.reflector().get_jsobject().get()));
                }
            } else {
                return None;
            }
        }
        // Step 4.
        let filter = NamedElementFilter {
            name: name,
        };
        let collection = HTMLCollection::create(self.window(), root, box filter);
        Some(NonZero::new(collection.reflector().get_jsobject().get()))
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

    #[allow(unsafe_code)]
    // https://drafts.csswg.org/cssom-view/#dom-document-elementfrompoint
    fn ElementFromPoint(&self, x: Finite<f64>, y: Finite<f64>) -> Option<Root<Element>> {
        let x = *x as f32;
        let y = *y as f32;
        let point = &Point2D::new(x, y);
        let window = window_from_node(self);
        let viewport = window.window_size().unwrap().initial_viewport;

        if self.browsing_context().is_none() {
            return None;
        }

        if x < 0.0 || y < 0.0 || x > viewport.width || y > viewport.height {
            return None;
        }

        match self.window.hit_test_query(*point, false) {
            Some(untrusted_node_address) => {
                let js_runtime = unsafe { JS_GetRuntime(window.get_cx()) };

                let node = unsafe {
                    node::from_untrusted_node_address(js_runtime, untrusted_node_address)
                };
                let parent_node = node.GetParentNode().unwrap();
                let element_ref = node.downcast::<Element>().unwrap_or_else(|| {
                    parent_node.downcast::<Element>().unwrap()
                });

                Some(Root::from_ref(element_ref))
            },
            None => self.GetDocumentElement()
        }
    }

    #[allow(unsafe_code)]
    // https://drafts.csswg.org/cssom-view/#dom-document-elementsfrompoint
    fn ElementsFromPoint(&self, x: Finite<f64>, y: Finite<f64>) -> Vec<Root<Element>> {
        let x = *x as f32;
        let y = *y as f32;
        let point = &Point2D::new(x, y);
        let window = window_from_node(self);
        let viewport = window.window_size().unwrap().initial_viewport;

        if self.browsing_context().is_none() {
            return vec!();
        }

        // Step 2
        if x < 0.0 || y < 0.0 || x > viewport.width || y > viewport.height {
            return vec!();
        }

        let js_runtime = unsafe { JS_GetRuntime(window.get_cx()) };

        // Step 1 and Step 3
        let mut elements: Vec<Root<Element>> = self.nodes_from_point(point).iter()
            .flat_map(|&untrusted_node_address| {
                let node = unsafe {
                    node::from_untrusted_node_address(js_runtime, untrusted_node_address)
                };
                Root::downcast::<Element>(node)
        }).collect();

        // Step 4
        if let Some(root_element) = self.GetDocumentElement() {
            if elements.last() != Some(&root_element) {
                elements.push(root_element);
            }
        }

        // Step 5
        elements
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-open
    fn Open(&self, type_: DOMString, replace: DOMString) -> Fallible<Root<Document>> {
        if !self.is_html_document() {
            // Step 1.
            return Err(Error::InvalidState);
        }

        // Step 2.
        // TODO: handle throw-on-dynamic-markup-insertion counter.

        if !self.is_active() {
            // Step 3.
            return Ok(Root::from_ref(self));
        }

        let entry_responsible_document = GlobalScope::entry().as_window().Document();

        // This check is same-origin not same-origin-domain.
        // https://github.com/whatwg/html/issues/2282
        // https://github.com/whatwg/html/pull/2288
        if !self.origin.same_origin(&entry_responsible_document.origin) {
            // Step 4.
            return Err(Error::Security);
        }

        if self.get_current_parser().map_or(false, |parser| parser.script_nesting_level() > 0) {
            // Step 5.
            return Ok(Root::from_ref(self));
        }

        // Step 6.
        // TODO: ignore-opens-during-unload counter check.

        // Step 7: first argument already bound to `type_`.

        // Step 8.
        // TODO: check session history's state.
        let replace = replace.eq_ignore_ascii_case("replace");

        // Step 9.
        // TODO: salvageable flag.

        // Step 10.
        // TODO: prompt to unload.

        // Step 11.
        // TODO: unload.

        // Step 12.
        self.abort();

        // Step 13.
        for node in self.upcast::<Node>().traverse_preorder() {
            node.upcast::<EventTarget>().remove_all_listeners();
        }

        // Step 14.
        // TODO: remove any tasks associated with the Document in any task source.

        // Step 15.
        Node::replace_all(None, self.upcast::<Node>());

        // Steps 16-18.
        // Let's not?
        // TODO: https://github.com/whatwg/html/issues/1698

        // Step 19.
        self.implementation.set(None);
        self.location.set(None);
        self.images.set(None);
        self.embeds.set(None);
        self.links.set(None);
        self.forms.set(None);
        self.scripts.set(None);
        self.anchors.set(None);
        self.applets.set(None);
        *self.stylesheets.borrow_mut() = None;
        self.stylesheets_changed_since_reflow.set(true);
        self.animation_frame_ident.set(0);
        self.animation_frame_list.borrow_mut().clear();
        self.pending_restyles.borrow_mut().clear();
        self.target_element.set(None);
        *self.last_click_info.borrow_mut() = None;

        // Step 20.
        self.set_encoding(UTF_8);

        // Step 21.
        // TODO: reload override buffer.

        // Step 22.
        // TODO: salvageable flag.

        let url = entry_responsible_document.url();

        // Step 23.
        self.set_url(url.clone());

        // Step 24.
        // TODO: mute iframe load.

        // Step 27.
        let type_ = if type_.eq_ignore_ascii_case("replace") {
            "text/html"
        } else if let Some(position) = type_.find(';') {
            &type_[0..position]
        } else {
            &*type_
        };
        let type_ = type_.trim_matches(HTML_SPACE_CHARACTERS);

        // Step 25.
        let resource_threads =
            self.window.upcast::<GlobalScope>().resource_threads().clone();
        *self.loader.borrow_mut() =
            DocumentLoader::new_with_threads(resource_threads, Some(url.clone()));
        ServoParser::parse_html_script_input(self, url, type_);

        // Step 26.
        self.ready_state.set(DocumentReadyState::Interactive);

        // Step 28 is handled when creating the parser in step 25.

        // Step 29.
        // TODO: truncate session history.

        // Step 30.
        // TODO: remove history traversal tasks.

        // Step 31.
        // TODO: remove earlier entries.

        if !replace {
            // Step 32.
            // TODO: add history entry.
        }

        // Step 33.
        // TODO: clear fired unload flag.

        // Step 34 is handled when creating the parser in step 25.

        // Step 35.
        Ok(Root::from_ref(self))
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-write
    fn Write(&self, text: Vec<DOMString>) -> ErrorResult {
        if !self.is_html_document() {
            // Step 1.
            return Err(Error::InvalidState);
        }

        // Step 2.
        // TODO: handle throw-on-dynamic-markup-insertion counter.
        if !self.is_active() {
            // Step 3.
            return Ok(());
        }

        let parser = match self.get_current_parser() {
            Some(ref parser) if parser.can_write() => Root::from_ref(&**parser),
            _ => {
                // Either there is no parser, which means the parsing ended;
                // or script nesting level is 0, which means the method was
                // called from outside a parser-executed script.
                if self.ignore_destructive_writes_counter.get() > 0 {
                    // Step 4.
                    // TODO: handle ignore-opens-during-unload counter.
                    return Ok(());
                }
                // Step 5.
                self.Open("text/html".into(), "".into())?;
                self.get_current_parser().unwrap()
            }
        };

        // Step 7.
        // TODO: handle reload override buffer.

        // Steps 6-8.
        parser.write(text);

        // Step 9.
        Ok(())
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-writeln
    fn Writeln(&self, mut text: Vec<DOMString>) -> ErrorResult {
        text.push("\n".into());
        self.Write(text)
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-close
    fn Close(&self) -> ErrorResult {
        if !self.is_html_document() {
            // Step 1.
            return Err(Error::InvalidState);
        }

        // Step 2.
        // TODO: handle throw-on-dynamic-markup-insertion counter.

        let parser = match self.get_current_parser() {
            Some(ref parser) if parser.is_script_created() => Root::from_ref(&**parser),
            _ => {
                // Step 3.
                return Ok(());
            }
        };

        // Step 4-6.
        parser.close();

        Ok(())
    }

    // https://html.spec.whatwg.org/multipage/#documentandelementeventhandlers
    document_and_element_event_handlers!();

    // https://fullscreen.spec.whatwg.org/#handler-document-onfullscreenerror
    event_handler!(fullscreenerror, GetOnfullscreenerror, SetOnfullscreenerror);

    // https://fullscreen.spec.whatwg.org/#handler-document-onfullscreenchange
    event_handler!(fullscreenchange, GetOnfullscreenchange, SetOnfullscreenchange);

    // https://fullscreen.spec.whatwg.org/#dom-document-fullscreenenabled
    fn FullscreenEnabled(&self) -> bool {
        self.get_allow_fullscreen()
    }

    // https://fullscreen.spec.whatwg.org/#dom-document-fullscreen
    fn Fullscreen(&self) -> bool {
        self.fullscreen_element.get().is_some()
    }

    // https://fullscreen.spec.whatwg.org/#dom-document-fullscreenelement
    fn GetFullscreenElement(&self) -> Option<Root<Element>> {
        // TODO ShadowRoot
        self.fullscreen_element.get()
    }

    #[allow(unrooted_must_root)]
    // https://fullscreen.spec.whatwg.org/#dom-document-exitfullscreen
    fn ExitFullscreen(&self) -> Rc<Promise> {
        self.exit_fullscreen()
    }
}

fn update_with_current_time_ms(marker: &Cell<u64>) {
    if marker.get() == Default::default() {
        let time = time::get_time();
        let current_time_ms = time.sec * 1000 + time.nsec as i64 / 1000000;
        marker.set(current_time_ms as u64);
    }
}

/// https://w3c.github.io/webappsec-referrer-policy/#determine-policy-for-token
pub fn determine_policy_for_token(token: &str) -> Option<ReferrerPolicy> {
    let lower = token.to_lowercase();
    return match lower.as_ref() {
        "never" | "no-referrer" => Some(ReferrerPolicy::NoReferrer),
        "default" | "no-referrer-when-downgrade" => Some(ReferrerPolicy::NoReferrerWhenDowngrade),
        "origin" => Some(ReferrerPolicy::Origin),
        "same-origin" => Some(ReferrerPolicy::SameOrigin),
        "strict-origin" => Some(ReferrerPolicy::StrictOrigin),
        "strict-origin-when-cross-origin" => Some(ReferrerPolicy::StrictOriginWhenCrossOrigin),
        "origin-when-cross-origin" => Some(ReferrerPolicy::OriginWhenCrossOrigin),
        "always" | "unsafe-url" => Some(ReferrerPolicy::UnsafeUrl),
        "" => Some(ReferrerPolicy::NoReferrer),
        _ => None,
    }
}

pub struct DocumentProgressHandler {
    addr: Trusted<Document>
}

impl DocumentProgressHandler {
     pub fn new(addr: Trusted<Document>) -> DocumentProgressHandler {
        DocumentProgressHandler {
            addr: addr
        }
    }

    fn set_ready_state_complete(&self) {
        let document = self.addr.root();
        document.set_ready_state(DocumentReadyState::Complete);
    }

    fn dispatch_load(&self) {
        let document = self.addr.root();
        if document.browsing_context().is_none() {
            return;
        }
        let window = document.window();
        let event = Event::new(window.upcast(),
                               atom!("load"),
                               EventBubbles::DoesNotBubble,
                               EventCancelable::NotCancelable);
        let wintarget = window.upcast::<EventTarget>();
        event.set_trusted(true);

        // http://w3c.github.io/navigation-timing/#widl-PerformanceNavigationTiming-loadEventStart
        update_with_current_time_ms(&document.load_event_start);

        debug!("About to dispatch load for {:?}", document.url());
        let _ = wintarget.dispatch_event_with_target(document.upcast(), &event);

        // http://w3c.github.io/navigation-timing/#widl-PerformanceNavigationTiming-loadEventEnd
        update_with_current_time_ms(&document.load_event_end);

        window.reflow(ReflowGoal::ForDisplay,
                      ReflowQueryType::NoQuery,
                      ReflowReason::DocumentLoaded);

        document.notify_constellation_load();
    }
}

impl Runnable for DocumentProgressHandler {
    fn name(&self) -> &'static str { "DocumentProgressHandler" }

    fn handler(self: Box<DocumentProgressHandler>) {
        let document = self.addr.root();
        let window = document.window();
        if window.is_alive() {
            self.set_ready_state_complete();
            self.dispatch_load();
            if let Some(fragment) = document.url().fragment() {
                document.check_and_scroll_fragment(fragment);
            }
        }
    }
}

/// Specifies the type of focus event that is sent to a pipeline
#[derive(Copy, Clone, PartialEq)]
pub enum FocusType {
    Element,    // The first focus message - focus the element itself
    Parent,     // Focusing a parent element (an iframe)
}

/// Focus events
pub enum FocusEventType {
    Focus,      // Element gained focus. Doesn't bubble.
    Blur,       // Element lost focus. Doesn't bubble.
}

/// A fake `requestAnimationFrame()` callback—"fake" because it is not triggered by the video
/// refresh but rather a simple timer.
///
/// If the page is observed to be using `requestAnimationFrame()` for non-animation purposes (i.e.
/// without mutating the DOM), then we fall back to simple timeouts to save energy over video
/// refresh.
#[derive(JSTraceable, HeapSizeOf)]
pub struct FakeRequestAnimationFrameCallback {
    /// The document.
    #[ignore_heap_size_of = "non-owning"]
    document: Trusted<Document>,
}

impl FakeRequestAnimationFrameCallback {
    pub fn invoke(self) {
        let document = self.document.root();
        document.run_the_animation_frame_callbacks();
    }
}

#[derive(HeapSizeOf, JSTraceable)]
pub enum AnimationFrameCallback {
    DevtoolsFramerateTick { actor_name: String },
    FrameRequestCallback {
        #[ignore_heap_size_of = "Rc is hard"]
        callback: Rc<FrameRequestCallback>
    },
}

impl AnimationFrameCallback {
    fn call(&self, document: &Document, now: f64) {
        match *self {
            AnimationFrameCallback::DevtoolsFramerateTick { ref actor_name } => {
                let msg = ScriptToDevtoolsControlMsg::FramerateTick(actor_name.clone(), now);
                let devtools_sender = document.window().upcast::<GlobalScope>().devtools_chan().unwrap();
                devtools_sender.send(msg).unwrap();
            }
            AnimationFrameCallback::FrameRequestCallback { ref callback } => {
                // TODO(jdm): The spec says that any exceptions should be suppressed:
                // https://github.com/servo/servo/issues/6928
                let _ = callback.Call__(Finite::wrap(now), ExceptionHandling::Report);
            }
        }
    }
}

#[derive(Default, HeapSizeOf, JSTraceable)]
#[must_root]
struct PendingInOrderScriptVec {
    scripts: DOMRefCell<VecDeque<PendingScript>>,
}

impl PendingInOrderScriptVec {
    fn is_empty(&self) -> bool {
        self.scripts.borrow().is_empty()
    }

    fn push(&self, element: &HTMLScriptElement) {
        self.scripts.borrow_mut().push_back(PendingScript::new(element));
    }

    fn loaded(&self, element: &HTMLScriptElement, result: ScriptResult) {
        let mut scripts = self.scripts.borrow_mut();
        let mut entry = scripts.iter_mut().find(|entry| &*entry.element == element).unwrap();
        entry.loaded(result);
    }

    fn take_next_ready_to_be_executed(&self) -> Option<(Root<HTMLScriptElement>, ScriptResult)> {
        let mut scripts = self.scripts.borrow_mut();
        let pair = scripts.front_mut().and_then(PendingScript::take_result);
        if pair.is_none() {
            return None;
        }
        scripts.pop_front();
        pair
    }

    fn clear(&self) {
        *self.scripts.borrow_mut() = Default::default();
    }
}

#[derive(HeapSizeOf, JSTraceable)]
#[must_root]
struct PendingScript {
    element: JS<HTMLScriptElement>,
    load: Option<ScriptResult>,
}

impl PendingScript {
    fn new(element: &HTMLScriptElement) -> Self {
        Self { element: JS::from_ref(element), load: None }
    }

    fn new_with_load(element: &HTMLScriptElement, load: Option<ScriptResult>) -> Self {
        Self { element: JS::from_ref(element), load }
    }

    fn loaded(&mut self, result: ScriptResult) {
        assert!(self.load.is_none());
        self.load = Some(result);
    }

    fn take_result(&mut self) -> Option<(Root<HTMLScriptElement>, ScriptResult)> {
        self.load.take().map(|result| (Root::from_ref(&*self.element), result))
    }
}
