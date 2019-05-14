/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::compartments::{AlreadyInCompartment, InCompartment};
use crate::document_loader::{DocumentLoader, LoadType};
use crate::dom::activation::{synthetic_click_activation, ActivationSource};
use crate::dom::attr::Attr;
use crate::dom::beforeunloadevent::BeforeUnloadEvent;
use crate::dom::bindings::callback::ExceptionHandling;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::BeforeUnloadEventBinding::BeforeUnloadEventBinding::BeforeUnloadEventMethods;
use crate::dom::bindings::codegen::Bindings::DocumentBinding;
use crate::dom::bindings::codegen::Bindings::DocumentBinding::ElementCreationOptions;
use crate::dom::bindings::codegen::Bindings::DocumentBinding::{
    DocumentMethods, DocumentReadyState,
};
use crate::dom::bindings::codegen::Bindings::EventBinding::EventBinding::EventMethods;
use crate::dom::bindings::codegen::Bindings::HTMLIFrameElementBinding::HTMLIFrameElementBinding::HTMLIFrameElementMethods;
use crate::dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use crate::dom::bindings::codegen::Bindings::NodeFilterBinding::NodeFilter;
use crate::dom::bindings::codegen::Bindings::PerformanceBinding::PerformanceMethods;
use crate::dom::bindings::codegen::Bindings::ShadowRootBinding::ShadowRootMethods;
use crate::dom::bindings::codegen::Bindings::TouchBinding::TouchMethods;
use crate::dom::bindings::codegen::Bindings::WindowBinding::{
    FrameRequestCallback, ScrollBehavior, WindowMethods,
};
use crate::dom::bindings::codegen::UnionTypes::NodeOrString;
use crate::dom::bindings::error::{Error, ErrorResult, Fallible};
use crate::dom::bindings::inheritance::{Castable, ElementTypeId, HTMLElementTypeId, NodeTypeId};
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::refcounted::{Trusted, TrustedPromise};
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject};
use crate::dom::bindings::root::{Dom, DomRoot, DomSlice, LayoutDom, MutNullableDom};
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::bindings::xmlname::XMLName::InvalidXMLName;
use crate::dom::bindings::xmlname::{
    namespace_from_domstring, validate_and_extract, xml_name_type,
};
use crate::dom::cdatasection::CDATASection;
use crate::dom::closeevent::CloseEvent;
use crate::dom::comment::Comment;
use crate::dom::compositionevent::CompositionEvent;
use crate::dom::cssstylesheet::CSSStyleSheet;
use crate::dom::customelementregistry::CustomElementDefinition;
use crate::dom::customevent::CustomEvent;
use crate::dom::documentfragment::DocumentFragment;
use crate::dom::documentorshadowroot::{DocumentOrShadowRoot, StyleSheetInDocument};
use crate::dom::documenttype::DocumentType;
use crate::dom::domimplementation::DOMImplementation;
use crate::dom::element::CustomElementCreationMode;
use crate::dom::element::{
    Element, ElementCreator, ElementPerformFullscreenEnter, ElementPerformFullscreenExit,
};
use crate::dom::errorevent::ErrorEvent;
use crate::dom::event::{Event, EventBubbles, EventCancelable, EventDefault, EventStatus};
use crate::dom::eventtarget::EventTarget;
use crate::dom::focusevent::FocusEvent;
use crate::dom::globalscope::GlobalScope;
use crate::dom::hashchangeevent::HashChangeEvent;
use crate::dom::htmlanchorelement::HTMLAnchorElement;
use crate::dom::htmlareaelement::HTMLAreaElement;
use crate::dom::htmlbaseelement::HTMLBaseElement;
use crate::dom::htmlbodyelement::HTMLBodyElement;
use crate::dom::htmlcollection::{CollectionFilter, HTMLCollection};
use crate::dom::htmlelement::HTMLElement;
use crate::dom::htmlembedelement::HTMLEmbedElement;
use crate::dom::htmlformelement::{FormControl, FormControlElementHelpers, HTMLFormElement};
use crate::dom::htmlheadelement::HTMLHeadElement;
use crate::dom::htmlhtmlelement::HTMLHtmlElement;
use crate::dom::htmliframeelement::HTMLIFrameElement;
use crate::dom::htmlimageelement::HTMLImageElement;
use crate::dom::htmlscriptelement::{HTMLScriptElement, ScriptResult};
use crate::dom::htmltitleelement::HTMLTitleElement;
use crate::dom::keyboardevent::KeyboardEvent;
use crate::dom::location::Location;
use crate::dom::messageevent::MessageEvent;
use crate::dom::mouseevent::MouseEvent;
use crate::dom::node::{self, document_from_node, window_from_node, CloneChildrenFlag};
use crate::dom::node::{LayoutNodeHelpers, Node, NodeDamage, NodeFlags, ShadowIncluding};
use crate::dom::nodeiterator::NodeIterator;
use crate::dom::nodelist::NodeList;
use crate::dom::pagetransitionevent::PageTransitionEvent;
use crate::dom::popstateevent::PopStateEvent;
use crate::dom::processinginstruction::ProcessingInstruction;
use crate::dom::progressevent::ProgressEvent;
use crate::dom::promise::Promise;
use crate::dom::range::Range;
use crate::dom::servoparser::ServoParser;
use crate::dom::shadowroot::ShadowRoot;
use crate::dom::storageevent::StorageEvent;
use crate::dom::stylesheetlist::{StyleSheetList, StyleSheetListOwner};
use crate::dom::text::Text;
use crate::dom::touch::Touch;
use crate::dom::touchevent::TouchEvent;
use crate::dom::touchlist::TouchList;
use crate::dom::treewalker::TreeWalker;
use crate::dom::uievent::UIEvent;
use crate::dom::virtualmethods::vtable_for;
use crate::dom::webglcontextevent::WebGLContextEvent;
use crate::dom::window::{ReflowReason, Window};
use crate::dom::windowproxy::WindowProxy;
use crate::fetch::FetchCanceller;
use crate::script_runtime::{CommonScriptMsg, ScriptThreadEventCategory};
use crate::script_thread::{MainThreadScriptMsg, ScriptThread};
use crate::stylesheet_set::StylesheetSetRef;
use crate::task::TaskBox;
use crate::task_source::{TaskSource, TaskSourceName};
use crate::timers::OneshotTimerCallback;
use cookie::Cookie;
use devtools_traits::ScriptToDevtoolsControlMsg;
use dom_struct::dom_struct;
use embedder_traits::EmbedderMsg;
use encoding_rs::{Encoding, UTF_8};
use euclid::Point2D;
use html5ever::{LocalName, Namespace, QualName};
use hyper_serde::Serde;
use ipc_channel::ipc::{self, IpcSender};
use js::jsapi::{JSContext, JSObject, JSRuntime};
use keyboard_types::{Key, KeyState, Modifiers};
use metrics::{
    InteractiveFlag, InteractiveMetrics, InteractiveWindow, ProfilerMetadataFactory,
    ProgressiveWebMetric,
};
use mime::{self, Mime};
use msg::constellation_msg::BrowsingContextId;
use net_traits::pub_domains::is_pub_domain;
use net_traits::request::RequestBuilder;
use net_traits::response::HttpsState;
use net_traits::CookieSource::NonHTTP;
use net_traits::CoreResourceMsg::{GetCookiesForUrl, SetCookiesForUrl};
use net_traits::{FetchResponseMsg, IpcSend, ReferrerPolicy};
use num_traits::ToPrimitive;
use profile_traits::ipc as profile_ipc;
use profile_traits::time::{TimerMetadata, TimerMetadataFrameType, TimerMetadataReflowType};
use ref_slice::ref_slice;
use script_layout_interface::message::{Msg, ReflowGoal};
use script_traits::{AnimationState, DocumentActivity, MouseButton, MouseEventType};
use script_traits::{MsDuration, ScriptMsg, TouchEventType, TouchId, UntrustedNodeAddress};
use servo_arc::Arc;
use servo_atoms::Atom;
use servo_config::pref;
use servo_url::{ImmutableOrigin, MutableOrigin, ServoUrl};
use std::borrow::ToOwned;
use std::cell::{Cell, Ref, RefMut};
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::{HashMap, HashSet, VecDeque};
use std::default::Default;
use std::mem;
use std::ptr::NonNull;
use std::rc::Rc;
use std::time::{Duration, Instant};
use style::attr::AttrValue;
use style::context::QuirksMode;
use style::invalidation::element::restyle_hints::RestyleHint;
use style::media_queries::{Device, MediaType};
use style::selector_parser::{RestyleDamage, Snapshot};
use style::shared_lock::SharedRwLock as StyleSharedRwLock;
use style::str::{split_html_space_chars, str_join};
use style::stylesheet_set::DocumentStylesheetSet;
use style::stylesheets::{Origin, OriginSet, Stylesheet};
use url::percent_encoding::percent_decode;
use url::Host;
use uuid::Uuid;

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

pub enum FireMouseEventType {
    Move,
    Over,
    Out,
}

impl FireMouseEventType {
    pub fn as_str(&self) -> &str {
        match self {
            &FireMouseEventType::Move => "mousemove",
            &FireMouseEventType::Over => "mouseover",
            &FireMouseEventType::Out => "mouseout",
        }
    }
}

#[derive(Clone, Copy, Debug, JSTraceable, MallocSizeOf, PartialEq)]
pub enum IsHTMLDocument {
    HTMLDocument,
    NonHTMLDocument,
}

#[derive(Debug, MallocSizeOf)]
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

/// <https://dom.spec.whatwg.org/#document>
#[dom_struct]
pub struct Document {
    node: Node,
    document_or_shadow_root: DocumentOrShadowRoot,
    window: Dom<Window>,
    implementation: MutNullableDom<DOMImplementation>,
    #[ignore_malloc_size_of = "type from external crate"]
    content_type: Mime,
    last_modified: Option<String>,
    encoding: Cell<&'static Encoding>,
    has_browsing_context: bool,
    is_html_document: bool,
    activity: Cell<DocumentActivity>,
    url: DomRefCell<ServoUrl>,
    #[ignore_malloc_size_of = "defined in selectors"]
    quirks_mode: Cell<QuirksMode>,
    /// Caches for the getElement methods
    id_map: DomRefCell<HashMap<Atom, Vec<Dom<Element>>>>,
    tag_map: DomRefCell<HashMap<LocalName, Dom<HTMLCollection>>>,
    tagns_map: DomRefCell<HashMap<QualName, Dom<HTMLCollection>>>,
    classes_map: DomRefCell<HashMap<Vec<Atom>, Dom<HTMLCollection>>>,
    images: MutNullableDom<HTMLCollection>,
    embeds: MutNullableDom<HTMLCollection>,
    links: MutNullableDom<HTMLCollection>,
    forms: MutNullableDom<HTMLCollection>,
    scripts: MutNullableDom<HTMLCollection>,
    anchors: MutNullableDom<HTMLCollection>,
    applets: MutNullableDom<HTMLCollection>,
    /// Lock use for style attributes and author-origin stylesheet objects in this document.
    /// Can be acquired once for accessing many objects.
    style_shared_lock: StyleSharedRwLock,
    /// List of stylesheets associated with nodes in this document. |None| if the list needs to be refreshed.
    stylesheets: DomRefCell<DocumentStylesheetSet<StyleSheetInDocument>>,
    stylesheet_list: MutNullableDom<StyleSheetList>,
    ready_state: Cell<DocumentReadyState>,
    /// Whether the DOMContentLoaded event has already been dispatched.
    domcontentloaded_dispatched: Cell<bool>,
    /// The element that has most recently requested focus for itself.
    possibly_focused: MutNullableDom<Element>,
    /// The element that currently has the document focus context.
    focused: MutNullableDom<Element>,
    /// The script element that is currently executing.
    current_script: MutNullableDom<HTMLScriptElement>,
    /// <https://html.spec.whatwg.org/multipage/#pending-parsing-blocking-script>
    pending_parsing_blocking_script: DomRefCell<Option<PendingScript>>,
    /// Number of stylesheets that block executing the next parser-inserted script
    script_blocking_stylesheets_count: Cell<u32>,
    /// https://html.spec.whatwg.org/multipage/#list-of-scripts-that-will-execute-when-the-document-has-finished-parsing
    deferred_scripts: PendingInOrderScriptVec,
    /// <https://html.spec.whatwg.org/multipage/#list-of-scripts-that-will-execute-in-order-as-soon-as-possible>
    asap_in_order_scripts_list: PendingInOrderScriptVec,
    /// <https://html.spec.whatwg.org/multipage/#set-of-scripts-that-will-execute-as-soon-as-possible>
    asap_scripts_set: DomRefCell<Vec<Dom<HTMLScriptElement>>>,
    /// <https://html.spec.whatwg.org/multipage/#concept-n-noscript>
    /// True if scripting is enabled for all scripts in this document
    scripting_enabled: bool,
    /// <https://html.spec.whatwg.org/multipage/#animation-frame-callback-identifier>
    /// Current identifier of animation frame callback
    animation_frame_ident: Cell<u32>,
    /// <https://html.spec.whatwg.org/multipage/#list-of-animation-frame-callbacks>
    /// List of animation frame callbacks
    animation_frame_list: DomRefCell<Vec<(u32, Option<AnimationFrameCallback>)>>,
    /// Whether we're in the process of running animation callbacks.
    ///
    /// Tracking this is not necessary for correctness. Instead, it is an optimization to avoid
    /// sending needless `ChangeRunningAnimationsState` messages to the compositor.
    running_animation_callbacks: Cell<bool>,
    /// Tracks all outstanding loads related to this document.
    loader: DomRefCell<DocumentLoader>,
    /// The current active HTML parser, to allow resuming after interruptions.
    current_parser: MutNullableDom<ServoParser>,
    /// When we should kick off a reflow. This happens during parsing.
    reflow_timeout: Cell<Option<u64>>,
    /// The cached first `base` element with an `href` attribute.
    base_element: MutNullableDom<HTMLBaseElement>,
    /// This field is set to the document itself for inert documents.
    /// <https://html.spec.whatwg.org/multipage/#appropriate-template-contents-owner-document>
    appropriate_template_contents_owner_document: MutNullableDom<Document>,
    /// Information on elements needing restyle to ship over to the layout thread when the
    /// time comes.
    pending_restyles: DomRefCell<HashMap<Dom<Element>, PendingRestyle>>,
    /// This flag will be true if layout suppressed a reflow attempt that was
    /// needed in order for the page to be painted.
    needs_paint: Cell<bool>,
    /// <http://w3c.github.io/touch-events/#dfn-active-touch-point>
    active_touch_points: DomRefCell<Vec<Dom<Touch>>>,
    /// Navigation Timing properties:
    /// <https://w3c.github.io/navigation-timing/#sec-PerformanceNavigationTiming>
    dom_loading: Cell<u64>,
    dom_interactive: Cell<u64>,
    dom_content_loaded_event_start: Cell<u64>,
    dom_content_loaded_event_end: Cell<u64>,
    dom_complete: Cell<u64>,
    top_level_dom_complete: Cell<u64>,
    load_event_start: Cell<u64>,
    load_event_end: Cell<u64>,
    unload_event_start: Cell<u64>,
    unload_event_end: Cell<u64>,
    /// <https://html.spec.whatwg.org/multipage/#concept-document-https-state>
    https_state: Cell<HttpsState>,
    /// The document's origin.
    origin: MutableOrigin,
    ///  https://w3c.github.io/webappsec-referrer-policy/#referrer-policy-states
    referrer_policy: Cell<Option<ReferrerPolicy>>,
    /// <https://html.spec.whatwg.org/multipage/#dom-document-referrer>
    referrer: Option<String>,
    /// <https://html.spec.whatwg.org/multipage/#target-element>
    target_element: MutNullableDom<Element>,
    /// <https://w3c.github.io/uievents/#event-type-dblclick>
    #[ignore_malloc_size_of = "Defined in std"]
    last_click_info: DomRefCell<Option<(Instant, Point2D<f32>)>>,
    /// <https://html.spec.whatwg.org/multipage/#ignore-destructive-writes-counter>
    ignore_destructive_writes_counter: Cell<u32>,
    /// <https://html.spec.whatwg.org/multipage/#ignore-opens-during-unload-counter>
    ignore_opens_during_unload_counter: Cell<u32>,
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
    fullscreen_element: MutNullableDom<Element>,
    /// Map from ID to set of form control elements that have that ID as
    /// their 'form' content attribute. Used to reset form controls
    /// whenever any element with the same ID as the form attribute
    /// is inserted or removed from the document.
    /// See https://html.spec.whatwg.org/multipage/#form-owner
    form_id_listener_map: DomRefCell<HashMap<Atom, HashSet<Dom<Element>>>>,
    interactive_time: DomRefCell<InteractiveMetrics>,
    tti_window: DomRefCell<InteractiveWindow>,
    /// RAII canceller for Fetch
    canceller: FetchCanceller,
    /// https://html.spec.whatwg.org/multipage/#throw-on-dynamic-markup-insertion-counter
    throw_on_dynamic_markup_insertion_counter: Cell<u64>,
    /// https://html.spec.whatwg.org/multipage/#page-showing
    page_showing: Cell<bool>,
    /// Whether the document is salvageable.
    salvageable: Cell<bool>,
    /// Whether the unload event has already been fired.
    fired_unload: Cell<bool>,
    /// List of responsive images
    responsive_images: DomRefCell<Vec<Dom<HTMLImageElement>>>,
    /// Number of redirects for the document load
    redirect_count: Cell<u16>,
    /// Number of outstanding requests to prevent JS or layout from running.
    script_and_layout_blockers: Cell<u32>,
    /// List of tasks to execute as soon as last script/layout blocker is removed.
    #[ignore_malloc_size_of = "Measuring trait objects is hard"]
    delayed_tasks: DomRefCell<Vec<Box<dyn TaskBox>>>,
    /// https://html.spec.whatwg.org/multipage/#completely-loaded
    completely_loaded: Cell<bool>,
    /// Set of shadow roots connected to the document tree.
    shadow_roots: DomRefCell<HashSet<Dom<ShadowRoot>>>,
    /// Whether any of the shadow roots need the stylesheets flushed.
    shadow_roots_styles_changed: Cell<bool>,
    /// List of registered media controls.
    /// We need to keep this list to allow the media controls to
    /// access the "privileged" document.servoGetMediaControls(id) API,
    /// where `id` needs to match any of the registered ShadowRoots
    /// hosting the media controls UI.
    media_controls: DomRefCell<HashMap<String, Dom<ShadowRoot>>>,
}

#[derive(JSTraceable, MallocSizeOf)]
struct ImagesFilter;
impl CollectionFilter for ImagesFilter {
    fn filter(&self, elem: &Element, _root: &Node) -> bool {
        elem.is::<HTMLImageElement>()
    }
}

#[derive(JSTraceable, MallocSizeOf)]
struct EmbedsFilter;
impl CollectionFilter for EmbedsFilter {
    fn filter(&self, elem: &Element, _root: &Node) -> bool {
        elem.is::<HTMLEmbedElement>()
    }
}

#[derive(JSTraceable, MallocSizeOf)]
struct LinksFilter;
impl CollectionFilter for LinksFilter {
    fn filter(&self, elem: &Element, _root: &Node) -> bool {
        (elem.is::<HTMLAnchorElement>() || elem.is::<HTMLAreaElement>()) &&
            elem.has_attribute(&local_name!("href"))
    }
}

#[derive(JSTraceable, MallocSizeOf)]
struct FormsFilter;
impl CollectionFilter for FormsFilter {
    fn filter(&self, elem: &Element, _root: &Node) -> bool {
        elem.is::<HTMLFormElement>()
    }
}

#[derive(JSTraceable, MallocSizeOf)]
struct ScriptsFilter;
impl CollectionFilter for ScriptsFilter {
    fn filter(&self, elem: &Element, _root: &Node) -> bool {
        elem.is::<HTMLScriptElement>()
    }
}

#[derive(JSTraceable, MallocSizeOf)]
struct AnchorsFilter;
impl CollectionFilter for AnchorsFilter {
    fn filter(&self, elem: &Element, _root: &Node) -> bool {
        elem.is::<HTMLAnchorElement>() && elem.has_attribute(&local_name!("href"))
    }
}

impl Document {
    #[inline]
    pub fn loader(&self) -> Ref<DocumentLoader> {
        self.loader.borrow()
    }

    #[inline]
    pub fn loader_mut(&self) -> RefMut<DocumentLoader> {
        self.loader.borrow_mut()
    }

    #[inline]
    pub fn has_browsing_context(&self) -> bool {
        self.has_browsing_context
    }

    /// <https://html.spec.whatwg.org/multipage/#concept-document-bc>
    #[inline]
    pub fn browsing_context(&self) -> Option<DomRoot<WindowProxy>> {
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
    }

    pub fn is_completely_loaded(&self) -> bool {
        self.completely_loaded.get()
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
                self.window()
                    .reflow(ReflowGoal::Full, ReflowReason::CachedPageNeededReflow);
                self.window().resume();
                // html.spec.whatwg.org/multipage/#history-traversal
                // Step 4.6
                if self.ready_state.get() == DocumentReadyState::Complete {
                    let document = Trusted::new(self);
                    self.window
                        .task_manager()
                        .dom_manipulation_task_source()
                        .queue(
                            task!(fire_pageshow_event: move || {
                                let document = document.root();
                                let window = document.window();
                                // Step 4.6.1
                                if document.page_showing.get() {
                                    return;
                                }
                                // Step 4.6.2
                                document.page_showing.set(true);
                                // Step 4.6.4
                                let event = PageTransitionEvent::new(
                                    window,
                                    atom!("pageshow"),
                                    false, // bubbles
                                    false, // cancelable
                                    true, // persisted
                                );
                                let event = event.upcast::<Event>();
                                event.set_trusted(true);
                                // FIXME(nox): Why are errors silenced here?
                                let _ = window.upcast::<EventTarget>().dispatch_event_with_target(
                                    document.upcast(),
                                    &event,
                                );
                            }),
                            self.window.upcast(),
                        )
                        .unwrap();
                }
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
        self.stylesheets.borrow().has_changed() ||
            self.GetDocumentElement().map_or(false, |root| {
                root.upcast::<Node>().has_dirty_descendants() ||
                    !self.pending_restyles.borrow().is_empty() ||
                    self.needs_paint()
            })
    }

    /// Returns the first `base` element in the DOM that has an `href` attribute.
    pub fn base_element(&self) -> Option<DomRoot<HTMLBaseElement>> {
        self.base_element.get()
    }

    /// Refresh the cached first base element in the DOM.
    /// <https://github.com/w3c/web-platform-tests/issues/2122>
    pub fn refresh_base_element(&self) {
        let base = self
            .upcast::<Node>()
            .traverse_preorder(ShadowIncluding::No)
            .filter_map(DomRoot::downcast::<HTMLBaseElement>)
            .find(|element| {
                element
                    .upcast::<Element>()
                    .has_attribute(&local_name!("href"))
            });
        self.base_element.set(base.deref());
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
            self.window
                .layout_chan()
                .send(Msg::SetQuirksMode(mode))
                .unwrap();
        }
    }

    pub fn encoding(&self) -> &'static Encoding {
        self.encoding.get()
    }

    pub fn set_encoding(&self, encoding: &'static Encoding) {
        self.encoding.set(encoding);
    }

    pub fn content_and_heritage_changed(&self, node: &Node) {
        if node.is_connected() {
            node.note_dirty_descendants();
        }

        // FIXME(emilio): This is very inefficient, ideally the flag above would
        // be enough and incremental layout could figure out from there.
        node.dirty(NodeDamage::OtherNodeDamage);
    }

    /// Reflows and disarms the timer if the reflow timer has expired.
    pub fn reflow_if_reflow_timer_expired(&self) {
        if let Some(reflow_timeout) = self.reflow_timeout.get() {
            if time::precise_time_ns() < reflow_timeout {
                return;
            }

            self.reflow_timeout.set(None);
            self.window
                .reflow(ReflowGoal::Full, ReflowReason::RefreshTick);
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
        self.document_or_shadow_root
            .unregister_named_element(&self.id_map, to_unregister, &id);
        self.reset_form_owner_for_listeners(&id);
    }

    /// Associate an element present in this document with the provided id.
    pub fn register_named_element(&self, element: &Element, id: Atom) {
        let root = self.GetDocumentElement().expect(
            "The element is in the document, so there must be a document \
             element.",
        );
        self.document_or_shadow_root.register_named_element(
            &self.id_map,
            element,
            &id,
            DomRoot::from_ref(root.upcast::<Node>()),
        );
        self.reset_form_owner_for_listeners(&id);
    }

    pub fn register_form_id_listener<T: ?Sized + FormControl>(&self, id: DOMString, listener: &T) {
        let mut map = self.form_id_listener_map.borrow_mut();
        let listener = listener.to_element();
        let set = map.entry(Atom::from(id)).or_insert(HashSet::new());
        set.insert(Dom::from_ref(listener));
    }

    pub fn unregister_form_id_listener<T: ?Sized + FormControl>(
        &self,
        id: DOMString,
        listener: &T,
    ) {
        let mut map = self.form_id_listener_map.borrow_mut();
        if let Occupied(mut entry) = map.entry(Atom::from(id)) {
            entry
                .get_mut()
                .remove(&Dom::from_ref(listener.to_element()));
            if entry.get().is_empty() {
                entry.remove();
            }
        }
    }

    /// Attempt to find a named element in this page's document.
    /// <https://html.spec.whatwg.org/multipage/#the-indicated-part-of-the-document>
    pub fn find_fragment_node(&self, fragid: &str) -> Option<DomRoot<Element>> {
        // Step 1 is not handled here; the fragid is already obtained by the calling function
        // Step 2: Simply use None to indicate the top of the document.
        // Step 3 & 4
        percent_decode(fragid.as_bytes())
            .decode_utf8()
            .ok()
            // Step 5
            .and_then(|decoded_fragid| self.get_element_by_id(&Atom::from(decoded_fragid)))
            // Step 6
            .or_else(|| self.get_anchor_by_name(fragid))
        // Step 7 & 8
    }

    /// Scroll to the target element, and when we do not find a target
    /// and the fragment is empty or "top", scroll to the top.
    /// <https://html.spec.whatwg.org/multipage/#scroll-to-the-fragment-identifier>
    pub fn check_and_scroll_fragment(&self, fragment: &str) {
        let target = self.find_fragment_node(fragment);

        // Step 1
        self.set_target_element(target.deref());

        let point = target
            .as_ref()
            .map(|element| {
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
                (
                    rect.origin.x.to_nearest_px() as f32,
                    rect.origin.y.to_nearest_px() as f32,
                )
            })
            .or_else(|| {
                if fragment.is_empty() || fragment.eq_ignore_ascii_case("top") {
                    // FIXME(stshine): this should be the origin of the stacking context space,
                    // which may differ under the influence of writing mode.
                    Some((0.0, 0.0))
                } else {
                    None
                }
            });

        if let Some((x, y)) = point {
            // Step 3
            let global_scope = self.window.upcast::<GlobalScope>();
            self.window.update_viewport_for_scroll(x, y);
            self.window.perform_a_scroll(
                x,
                y,
                global_scope.pipeline_id().root_scroll_id(),
                ScrollBehavior::Instant,
                target.deref(),
            );
        }
    }

    fn get_anchor_by_name(&self, name: &str) -> Option<DomRoot<Element>> {
        let check_anchor = |node: &HTMLAnchorElement| {
            let elem = node.upcast::<Element>();
            elem.get_attribute(&ns!(), &local_name!("name"))
                .map_or(false, |attr| &**attr.value() == name)
        };
        let doc_node = self.upcast::<Node>();
        doc_node
            .traverse_preorder(ShadowIncluding::No)
            .filter_map(DomRoot::downcast)
            .find(|node| check_anchor(&node))
            .map(DomRoot::upcast)
    }

    // https://html.spec.whatwg.org/multipage/#current-document-readiness
    pub fn set_ready_state(&self, state: DocumentReadyState) {
        match state {
            DocumentReadyState::Loading => {
                update_with_current_time_ms(&self.dom_loading);
            },
            DocumentReadyState::Complete => {
                update_with_current_time_ms(&self.dom_complete);
            },
            DocumentReadyState::Interactive => update_with_current_time_ms(&self.dom_interactive),
        };

        self.ready_state.set(state);

        self.upcast::<EventTarget>()
            .fire_event(atom!("readystatechange"));
    }

    /// Return whether scripting is enabled or not
    pub fn is_scripting_enabled(&self) -> bool {
        self.scripting_enabled
    }

    /// Return the element that currently has focus.
    // https://w3c.github.io/uievents/#events-focusevent-doc-focus
    pub fn get_focused_element(&self) -> Option<DomRoot<Element>> {
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
        if self.focused == self.possibly_focused.get().deref() {
            return;
        }
        if let Some(ref elem) = self.focused.get() {
            let node = elem.upcast::<Node>();
            elem.set_focus_state(false);
            // FIXME: pass appropriate relatedTarget
            self.fire_focus_event(FocusEventType::Blur, node, None);

            // Notify the embedder to hide the input method.
            if elem.input_method_type().is_some() {
                self.send_to_embedder(EmbedderMsg::HideIME);
            }
        }

        self.focused.set(self.possibly_focused.get().deref());

        if let Some(ref elem) = self.focused.get() {
            elem.set_focus_state(true);
            let node = elem.upcast::<Node>();
            // FIXME: pass appropriate relatedTarget
            self.fire_focus_event(FocusEventType::Focus, node, None);
            // Update the focus state for all elements in the focus chain.
            // https://html.spec.whatwg.org/multipage/#focus-chain
            if focus_type == FocusType::Element {
                self.window().send_to_constellation(ScriptMsg::Focus);
            }

            // Notify the embedder to display an input method.
            if let Some(kind) = elem.input_method_type() {
                self.send_to_embedder(EmbedderMsg::ShowIME(kind));
            }
        }
    }

    /// Handles any updates when the document's title has changed.
    pub fn title_changed(&self) {
        if self.browsing_context().is_some() {
            self.send_title_to_embedder();
        }
    }

    /// Sends this document's title to the constellation.
    pub fn send_title_to_embedder(&self) {
        let window = self.window();
        if window.is_top_level() {
            let title = Some(String::from(self.Title()));
            self.send_to_embedder(EmbedderMsg::ChangePageTitle(title));
        }
    }

    fn send_to_embedder(&self, msg: EmbedderMsg) {
        let window = self.window();
        window.send_to_embedder(msg);
    }

    pub fn dirty_all_nodes(&self) {
        let root = self.upcast::<Node>();
        for node in root.traverse_preorder(ShadowIncluding::Yes) {
            node.dirty(NodeDamage::OtherNodeDamage)
        }
    }

    #[allow(unsafe_code)]
    pub fn handle_mouse_event(
        &self,
        js_runtime: *mut JSRuntime,
        _button: MouseButton,
        client_point: Point2D<f32>,
        mouse_event_type: MouseEventType,
        node_address: Option<UntrustedNodeAddress>,
        point_in_node: Option<Point2D<f32>>,
    ) {
        let mouse_event_type_string = match mouse_event_type {
            MouseEventType::Click => "click".to_owned(),
            MouseEventType::MouseUp => "mouseup".to_owned(),
            MouseEventType::MouseDown => "mousedown".to_owned(),
        };
        debug!("{}: at {:?}", mouse_event_type_string, client_point);

        let el = node_address.and_then(|address| {
            let node = unsafe { node::from_untrusted_node_address(js_runtime, address) };
            node.inclusive_ancestors(ShadowIncluding::No)
                .filter_map(DomRoot::downcast::<Element>)
                .next()
        });
        let el = match el {
            Some(el) => el,
            None => return,
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

        // https://w3c.github.io/uievents/#event-type-click
        let client_x = client_point.x as i32;
        let client_y = client_point.y as i32;
        let click_count = 1;
        let event = MouseEvent::new(
            &self.window,
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
            None,
            point_in_node,
        );
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

        self.window
            .reflow(ReflowGoal::Full, ReflowReason::MouseEvent);
    }

    fn maybe_fire_dblclick(&self, click_pos: Point2D<f32>, target: &Node) {
        // https://w3c.github.io/uievents/#event-type-dblclick
        let now = Instant::now();

        let opt = self.last_click_info.borrow_mut().take();

        if let Some((last_time, last_pos)) = opt {
            let DBL_CLICK_TIMEOUT =
                Duration::from_millis(pref!(dom.document.dblclick_timeout) as u64);
            let DBL_CLICK_DIST_THRESHOLD = pref!(dom.document.dblclick_dist) as u64;

            // Calculate distance between this click and the previous click.
            let line = click_pos - last_pos;
            let dist = (line.dot(line) as f64).sqrt();

            if now.duration_since(last_time) < DBL_CLICK_TIMEOUT &&
                dist < DBL_CLICK_DIST_THRESHOLD as f64
            {
                // A double click has occurred if this click is within a certain time and dist. of previous click.
                let click_count = 2;
                let client_x = click_pos.x as i32;
                let client_y = click_pos.y as i32;

                let event = MouseEvent::new(
                    &self.window,
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
                    None,
                    None,
                );
                event.upcast::<Event>().fire(target.upcast());

                // When a double click occurs, self.last_click_info is left as None so that a
                // third sequential click will not cause another double click.
                return;
            }
        }

        // Update last_click_info with the time and position of the click.
        *self.last_click_info.borrow_mut() = Some((now, click_pos));
    }

    pub fn fire_mouse_event(
        &self,
        client_point: Point2D<f32>,
        target: &EventTarget,
        event_name: FireMouseEventType,
    ) {
        let client_x = client_point.x.to_i32().unwrap_or(0);
        let client_y = client_point.y.to_i32().unwrap_or(0);

        let mouse_event = MouseEvent::new(
            &self.window,
            DOMString::from(event_name.as_str()),
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
            None,
            None,
        );
        let event = mouse_event.upcast::<Event>();
        event.fire(target);
    }

    #[allow(unsafe_code)]
    pub fn handle_mouse_move_event(
        &self,
        js_runtime: *mut JSRuntime,
        client_point: Option<Point2D<f32>>,
        prev_mouse_over_target: &MutNullableDom<Element>,
        node_address: Option<UntrustedNodeAddress>,
    ) {
        let client_point = match client_point {
            None => {
                // If there's no point, there's no target under the mouse
                // FIXME: dispatch mouseout here. We have no point.
                prev_mouse_over_target.set(None);
                return;
            },
            Some(client_point) => client_point,
        };

        let maybe_new_target = node_address.and_then(|address| {
            let node = unsafe { node::from_untrusted_node_address(js_runtime, address) };
            node.inclusive_ancestors(ShadowIncluding::No)
                .filter_map(DomRoot::downcast::<Element>)
                .next()
        });

        // Send mousemove event to topmost target, unless it's an iframe, in which case the
        // compositor should have also sent an event to the inner document.
        let new_target = match maybe_new_target {
            Some(ref target) => target,
            None => return,
        };

        self.fire_mouse_event(client_point, new_target.upcast(), FireMouseEventType::Move);

        // Nothing more to do here, mousemove is sent,
        // and the element under the mouse hasn't changed.
        if maybe_new_target == prev_mouse_over_target.get() {
            return;
        }

        let old_target_is_ancestor_of_new_target =
            match (prev_mouse_over_target.get(), maybe_new_target.as_ref()) {
                (Some(old_target), Some(new_target)) => old_target
                    .upcast::<Node>()
                    .is_ancestor_of(new_target.upcast::<Node>()),
                _ => false,
            };

        // Here we know the target has changed, so we must update the state,
        // dispatch mouseout to the previous one, mouseover to the new one,
        if let Some(old_target) = prev_mouse_over_target.get() {
            // If the old target is an ancestor of the new target, this can be skipped
            // completely, since the node's hover state will be reseted below.
            if !old_target_is_ancestor_of_new_target {
                for element in old_target
                    .upcast::<Node>()
                    .inclusive_ancestors(ShadowIncluding::No)
                    .filter_map(DomRoot::downcast::<Element>)
                {
                    element.set_hover_state(false);
                    element.set_active_state(false);
                }
            }

            // Remove hover state to old target and its parents
            self.fire_mouse_event(client_point, old_target.upcast(), FireMouseEventType::Out);

            // TODO: Fire mouseleave here only if the old target is
            // not an ancestor of the new target.
        }

        if let Some(ref new_target) = maybe_new_target {
            for element in new_target
                .upcast::<Node>()
                .inclusive_ancestors(ShadowIncluding::No)
                .filter_map(DomRoot::downcast::<Element>)
            {
                if element.hover_state() {
                    break;
                }

                element.set_hover_state(true);
            }

            self.fire_mouse_event(client_point, &new_target.upcast(), FireMouseEventType::Over);

            // TODO: Fire mouseenter here.
        }

        // Store the current mouse over target for next frame.
        prev_mouse_over_target.set(maybe_new_target.deref());

        self.window
            .reflow(ReflowGoal::Full, ReflowReason::MouseEvent);
    }

    #[allow(unsafe_code)]
    pub fn handle_touch_event(
        &self,
        js_runtime: *mut JSRuntime,
        event_type: TouchEventType,
        touch_id: TouchId,
        point: Point2D<f32>,
        node_address: Option<UntrustedNodeAddress>,
    ) -> TouchEventResult {
        let TouchId(identifier) = touch_id;

        let event_name = match event_type {
            TouchEventType::Down => "touchstart",
            TouchEventType::Move => "touchmove",
            TouchEventType::Up => "touchend",
            TouchEventType::Cancel => "touchcancel",
        };

        let el = node_address.and_then(|address| {
            let node = unsafe { node::from_untrusted_node_address(js_runtime, address) };
            node.inclusive_ancestors(ShadowIncluding::No)
                .filter_map(DomRoot::downcast::<Element>)
                .next()
        });
        let el = match el {
            Some(el) => el,
            None => return TouchEventResult::Forwarded,
        };

        let target = DomRoot::upcast::<EventTarget>(el);
        let window = &*self.window;

        let client_x = Finite::wrap(point.x as f64);
        let client_y = Finite::wrap(point.y as f64);
        let page_x = Finite::wrap(point.x as f64 + window.PageXOffset() as f64);
        let page_y = Finite::wrap(point.y as f64 + window.PageYOffset() as f64);

        let touch = Touch::new(
            window, identifier, &target, client_x,
            client_y, // TODO: Get real screen coordinates?
            client_x, client_y, page_x, page_y,
        );

        match event_type {
            TouchEventType::Down => {
                // Add a new touch point
                self.active_touch_points
                    .borrow_mut()
                    .push(Dom::from_ref(&*touch));
            },
            TouchEventType::Move => {
                // Replace an existing touch point
                let mut active_touch_points = self.active_touch_points.borrow_mut();
                match active_touch_points
                    .iter_mut()
                    .find(|t| t.Identifier() == identifier)
                {
                    Some(t) => *t = Dom::from_ref(&*touch),
                    None => warn!("Got a touchmove event for a non-active touch point"),
                }
            },
            TouchEventType::Up | TouchEventType::Cancel => {
                // Remove an existing touch point
                let mut active_touch_points = self.active_touch_points.borrow_mut();
                match active_touch_points
                    .iter()
                    .position(|t| t.Identifier() == identifier)
                {
                    Some(i) => {
                        active_touch_points.swap_remove(i);
                    },
                    None => warn!("Got a touchend event for a non-active touch point"),
                }
            },
        }

        rooted_vec!(let mut target_touches);
        let touches = {
            let touches = self.active_touch_points.borrow();
            target_touches.extend(touches.iter().filter(|t| t.Target() == target).cloned());
            TouchList::new(window, touches.r())
        };

        let event = TouchEvent::new(
            window,
            DOMString::from(event_name),
            EventBubbles::Bubbles,
            EventCancelable::Cancelable,
            Some(window),
            0i32,
            &touches,
            &TouchList::new(window, ref_slice(&&*touch)),
            &TouchList::new(window, target_touches.r()),
            // FIXME: modifier keys
            false,
            false,
            false,
            false,
        );
        let event = event.upcast::<Event>();
        let result = event.fire(&target);

        window.reflow(ReflowGoal::Full, ReflowReason::MouseEvent);

        match result {
            EventStatus::Canceled => TouchEventResult::Processed(false),
            EventStatus::NotCanceled => TouchEventResult::Processed(true),
        }
    }

    /// The entry point for all key processing for web content
    pub fn dispatch_key_event(&self, keyboard_event: ::keyboard_types::KeyboardEvent) {
        let focused = self.get_focused_element();
        let body = self.GetBody();

        let target = match (&focused, &body) {
            (&Some(ref focused), _) => focused.upcast(),
            (&None, &Some(ref body)) => body.upcast(),
            (&None, &None) => self.window.upcast(),
        };

        let keyevent = KeyboardEvent::new(
            &self.window,
            DOMString::from(keyboard_event.state.to_string()),
            true,
            true,
            Some(&self.window),
            0,
            keyboard_event.key.clone(),
            DOMString::from(keyboard_event.code.to_string()),
            keyboard_event.location as u32,
            keyboard_event.repeat,
            keyboard_event.is_composing,
            keyboard_event.modifiers,
            0,
            keyboard_event.key.legacy_keycode(),
        );
        let event = keyevent.upcast::<Event>();
        event.fire(target);
        let mut cancel_state = event.get_cancel_state();

        // https://w3c.github.io/uievents/#keys-cancelable-keys
        if keyboard_event.state == KeyState::Down &&
            keyboard_event.key.legacy_charcode() != 0 &&
            cancel_state != EventDefault::Prevented
        {
            // https://w3c.github.io/uievents/#keypress-event-order
            let event = KeyboardEvent::new(
                &self.window,
                DOMString::from("keypress"),
                true,
                true,
                Some(&self.window),
                0,
                keyboard_event.key.clone(),
                DOMString::from(keyboard_event.code.to_string()),
                keyboard_event.location as u32,
                keyboard_event.repeat,
                keyboard_event.is_composing,
                keyboard_event.modifiers,
                keyboard_event.key.legacy_charcode(),
                0,
            );
            let ev = event.upcast::<Event>();
            ev.fire(target);
            cancel_state = ev.get_cancel_state();
        }

        if cancel_state == EventDefault::Allowed {
            let msg = EmbedderMsg::Keyboard(keyboard_event.clone());
            self.send_to_embedder(msg);

            // This behavior is unspecced
            // We are supposed to dispatch synthetic click activation for Space and/or Return,
            // however *when* we do it is up to us.
            // Here, we're dispatching it after the key event so the script has a chance to cancel it
            // https://www.w3.org/Bugs/Public/show_bug.cgi?id=27337
            match keyboard_event.key {
                Key::Character(ref letter)
                    if letter == " " && keyboard_event.state == KeyState::Up =>
                {
                    let maybe_elem = target.downcast::<Element>();
                    if let Some(el) = maybe_elem {
                        synthetic_click_activation(
                            el,
                            false,
                            false,
                            false,
                            false,
                            ActivationSource::NotFromClick,
                        )
                    }
                }
                Key::Enter if keyboard_event.state == KeyState::Up => {
                    let maybe_elem = target.downcast::<Element>();
                    if let Some(el) = maybe_elem {
                        if let Some(a) = el.as_maybe_activatable() {
                            let ctrl = keyboard_event.modifiers.contains(Modifiers::CONTROL);
                            let alt = keyboard_event.modifiers.contains(Modifiers::ALT);
                            let shift = keyboard_event.modifiers.contains(Modifiers::SHIFT);
                            let meta = keyboard_event.modifiers.contains(Modifiers::META);
                            a.implicit_submission(ctrl, alt, shift, meta);
                        }
                    }
                },
                _ => (),
            }
        }

        self.window.reflow(ReflowGoal::Full, ReflowReason::KeyEvent);
    }

    pub fn dispatch_composition_event(
        &self,
        composition_event: ::keyboard_types::CompositionEvent,
    ) {
        // spec: https://w3c.github.io/uievents/#compositionstart
        // spec: https://w3c.github.io/uievents/#compositionupdate
        // spec: https://w3c.github.io/uievents/#compositionend
        // > Event.target : focused element processing the composition
        let focused = self.get_focused_element();
        let target = if let Some(elem) = &focused {
            elem.upcast()
        } else {
            // Event is only dispatched if there is a focused element.
            return;
        };

        let cancelable = composition_event.state == keyboard_types::CompositionState::Start;

        let compositionevent = CompositionEvent::new(
            &self.window,
            DOMString::from(composition_event.state.to_string()),
            true,
            cancelable,
            Some(&self.window),
            0,
            DOMString::from(composition_event.data),
        );
        let event = compositionevent.upcast::<Event>();
        event.fire(target);
    }

    // https://dom.spec.whatwg.org/#converting-nodes-into-a-node
    pub fn node_from_nodes_and_strings(
        &self,
        mut nodes: Vec<NodeOrString>,
    ) -> Fallible<DomRoot<Node>> {
        if nodes.len() == 1 {
            Ok(match nodes.pop().unwrap() {
                NodeOrString::Node(node) => node,
                NodeOrString::String(string) => DomRoot::upcast(self.CreateTextNode(string)),
            })
        } else {
            let fragment = DomRoot::upcast::<Node>(self.CreateDocumentFragment());
            for node in nodes {
                match node {
                    NodeOrString::Node(node) => {
                        fragment.AppendChild(&node)?;
                    },
                    NodeOrString::String(string) => {
                        let node = DomRoot::upcast::<Node>(self.CreateTextNode(string));
                        // No try!() here because appending a text node
                        // should not fail.
                        fragment.AppendChild(&node).unwrap();
                    },
                }
            }
            Ok(fragment)
        }
    }

    pub fn get_body_attribute(&self, local_name: &LocalName) -> DOMString {
        match self
            .GetBody()
            .and_then(DomRoot::downcast::<HTMLBodyElement>)
        {
            Some(ref body) => body.upcast::<Element>().get_string_attribute(local_name),
            None => DOMString::new(),
        }
    }

    pub fn set_body_attribute(&self, local_name: &LocalName, value: DOMString) {
        if let Some(ref body) = self
            .GetBody()
            .and_then(DomRoot::downcast::<HTMLBodyElement>)
        {
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
        self.stylesheets.borrow_mut().force_dirty(OriginSet::all());

        // Mark the document element dirty so a reflow will be performed.
        //
        // FIXME(emilio): Use the DocumentStylesheetSet invalidation stuff.
        if let Some(element) = self.GetDocumentElement() {
            element.upcast::<Node>().dirty(NodeDamage::NodeStyleDamaged);
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-window-requestanimationframe>
    pub fn request_animation_frame(&self, callback: AnimationFrameCallback) -> u32 {
        let ident = self.animation_frame_ident.get() + 1;

        self.animation_frame_ident.set(ident);
        self.animation_frame_list
            .borrow_mut()
            .push((ident, Some(callback)));

        // TODO: Should tick animation only when document is visible

        // If we are running 'fake' animation frames, we unconditionally
        // set up a one-shot timer for script to execute the rAF callbacks.
        if self.is_faking_animation_frames() {
            let callback = FakeRequestAnimationFrameCallback {
                document: Trusted::new(self),
            };
            self.global().schedule_callback(
                OneshotTimerCallback::FakeRequestAnimationFrame(callback),
                MsDuration::new(FAKE_REQUEST_ANIMATION_FRAME_DELAY),
            );
        } else if !self.running_animation_callbacks.get() {
            // No need to send a `ChangeRunningAnimationsState` if we're running animation callbacks:
            // we're guaranteed to already be in the "animation callbacks present" state.
            //
            // This reduces CPU usage by avoiding needless thread wakeups in the common case of
            // repeated rAF.

            let event =
                ScriptMsg::ChangeRunningAnimationsState(AnimationState::AnimationCallbacksPresent);
            self.window().send_to_constellation(event);
        }

        ident
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-window-cancelanimationframe>
    pub fn cancel_animation_frame(&self, ident: u32) {
        let mut list = self.animation_frame_list.borrow_mut();
        if let Some(pair) = list.iter_mut().find(|pair| pair.0 == ident) {
            pair.1 = None;
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#run-the-animation-frame-callbacks>
    pub fn run_the_animation_frame_callbacks(&self) {
        rooted_vec!(let mut animation_frame_list);
        mem::swap(
            &mut *animation_frame_list,
            &mut *self.animation_frame_list.borrow_mut(),
        );

        self.running_animation_callbacks.set(true);
        let was_faking_animation_frames = self.is_faking_animation_frames();
        let timing = self.global().performance().Now();

        for (_, callback) in animation_frame_list.drain(..) {
            if let Some(callback) = callback {
                callback.call(self, *timing);
            }
        }

        self.running_animation_callbacks.set(false);

        let spurious = !self
            .window
            .reflow(ReflowGoal::Full, ReflowReason::RequestAnimationFrame);

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
            self.window
                .force_reflow(ReflowGoal::Full, ReflowReason::RequestAnimationFrame);
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
        let is_empty = self.animation_frame_list.borrow().is_empty();
        if is_empty || (!was_faking_animation_frames && self.is_faking_animation_frames()) {
            if is_empty {
                // If the current animation frame list in the DOM instance is empty,
                // we can reuse the original `Vec<T>` that we put on the stack to
                // avoid allocating a new one next time an animation callback
                // is queued.
                mem::swap(
                    &mut *self.animation_frame_list.borrow_mut(),
                    &mut *animation_frame_list,
                );
            }
            let event = ScriptMsg::ChangeRunningAnimationsState(
                AnimationState::NoAnimationCallbacksPresent,
            );
            self.window().send_to_constellation(event);
        }

        // Update the counter of spurious animation frames.
        if spurious {
            if self.spurious_animation_frames.get() < SPURIOUS_ANIMATION_FRAME_THRESHOLD {
                self.spurious_animation_frames
                    .set(self.spurious_animation_frames.get() + 1)
            }
        } else {
            self.spurious_animation_frames.set(0)
        }
    }

    pub fn fetch_async(
        &self,
        load: LoadType,
        request: RequestBuilder,
        fetch_target: IpcSender<FetchResponseMsg>,
    ) {
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
                if self.has_browsing_context && self.is_fully_active() {
                    // Note: if the document is not fully active, the layout thread will have exited already.
                    // The underlying problem might actually be that layout exits while it should be kept alive.
                    // See https://github.com/servo/servo/issues/22507

                    // Disarm the reflow timer and trigger the initial reflow.
                    self.reflow_timeout.set(None);
                    self.upcast::<Node>().dirty(NodeDamage::OtherNodeDamage);
                    self.window
                        .reflow(ReflowGoal::Full, ReflowReason::FirstLoad);
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

        // Servo measures when the top-level content (not iframes) is loaded.
        if (self.top_level_dom_complete.get() == 0) && loader.is_only_blocked_by_iframes() {
            update_with_current_time_ms(&self.top_level_dom_complete);
        }

        if loader.is_blocked() || loader.events_inhibited() {
            // Step 6.
            return;
        }

        ScriptThread::mark_document_with_no_blocked_loads(self);
    }

    // https://html.spec.whatwg.org/multipage/#prompt-to-unload-a-document
    pub fn prompt_to_unload(&self, recursive_flag: bool) -> bool {
        // TODO: Step 1, increase the event loop's termination nesting level by 1.
        // Step 2
        self.incr_ignore_opens_during_unload_counter();
        //Step 3-5.
        let document = Trusted::new(self);
        let beforeunload_event = BeforeUnloadEvent::new(
            &self.window,
            atom!("beforeunload"),
            EventBubbles::Bubbles,
            EventCancelable::Cancelable,
        );
        let event = beforeunload_event.upcast::<Event>();
        event.set_trusted(true);
        let event_target = self.window.upcast::<EventTarget>();
        let has_listeners = event.has_listeners_for(&event_target, &atom!("beforeunload"));
        event_target.dispatch_event_with_target(document.root().upcast(), &event);
        // TODO: Step 6, decrease the event loop's termination nesting level by 1.
        // Step 7
        if has_listeners {
            self.salvageable.set(false);
        }
        let mut can_unload = true;
        // TODO: Step 8, also check sandboxing modals flag.
        let default_prevented = event.DefaultPrevented();
        let return_value_not_empty = !event
            .downcast::<BeforeUnloadEvent>()
            .unwrap()
            .ReturnValue()
            .is_empty();
        if default_prevented || return_value_not_empty {
            let (chan, port) = ipc::channel().expect("Failed to create IPC channel!");
            let msg = EmbedderMsg::AllowUnload(chan);
            self.send_to_embedder(msg);
            can_unload = port.recv().unwrap();
        }
        // Step 9
        if !recursive_flag {
            for iframe in self.iter_iframes() {
                // TODO: handle the case of cross origin iframes.
                let document = document_from_node(&*iframe);
                can_unload = document.prompt_to_unload(true);
                if !document.salvageable() {
                    self.salvageable.set(false);
                }
                if !can_unload {
                    break;
                }
            }
        }
        // Step 10
        self.decr_ignore_opens_during_unload_counter();
        can_unload
    }

    // https://html.spec.whatwg.org/multipage/#unload-a-document
    pub fn unload(&self, recursive_flag: bool) {
        // TODO: Step 1, increase the event loop's termination nesting level by 1.
        // Step 2
        self.incr_ignore_opens_during_unload_counter();
        let document = Trusted::new(self);
        // Step 3-6
        if self.page_showing.get() {
            self.page_showing.set(false);
            let event = PageTransitionEvent::new(
                &self.window,
                atom!("pagehide"),
                false,                  // bubbles
                false,                  // cancelable
                self.salvageable.get(), // persisted
            );
            let event = event.upcast::<Event>();
            event.set_trusted(true);
            let _ = self
                .window
                .upcast::<EventTarget>()
                .dispatch_event_with_target(document.root().upcast(), &event);
            // TODO Step 6, document visibility steps.
        }
        // Step 7
        if !self.fired_unload.get() {
            let event = Event::new(
                &self.window.upcast(),
                atom!("unload"),
                EventBubbles::Bubbles,
                EventCancelable::Cancelable,
            );
            event.set_trusted(true);
            let event_target = self.window.upcast::<EventTarget>();
            let has_listeners = event.has_listeners_for(&event_target, &atom!("unload"));
            let _ = event_target.dispatch_event_with_target(document.root().upcast(), &event);
            self.fired_unload.set(true);
            // Step 9
            if has_listeners {
                self.salvageable.set(false);
            }
        }
        // TODO: Step 8, decrease the event loop's termination nesting level by 1.

        // Step 13
        if !recursive_flag {
            for iframe in self.iter_iframes() {
                // TODO: handle the case of cross origin iframes.
                let document = document_from_node(&*iframe);
                document.unload(true);
                if !document.salvageable() {
                    self.salvageable.set(false);
                }
            }
        }
        // Step 10, 14
        if !self.salvageable.get() {
            // https://html.spec.whatwg.org/multipage/#unloading-document-cleanup-steps
            let global_scope = self.window.upcast::<GlobalScope>();
            // Step 1 of clean-up steps.
            global_scope.close_event_sources();
            let msg = ScriptMsg::DiscardDocument;
            let _ = global_scope.script_to_constellation_chan().send(msg);
        }
        // Step 15, End
        self.decr_ignore_opens_during_unload_counter();
    }

    // https://html.spec.whatwg.org/multipage/#the-end
    pub fn maybe_queue_document_completion(&self) {
        // https://html.spec.whatwg.org/multipage/#delaying-load-events-mode
        let is_in_delaying_load_events_mode = match self.window.undiscarded_window_proxy() {
            Some(window_proxy) => window_proxy.is_delaying_load_events_mode(),
            None => false,
        };

        // Note: if the document is not fully active, the layout thread will have exited already,
        // and this method will panic.
        // The underlying problem might actually be that layout exits while it should be kept alive.
        // See https://github.com/servo/servo/issues/22507
        let not_ready_for_load = self.loader.borrow().is_blocked() ||
            !self.is_fully_active() ||
            is_in_delaying_load_events_mode;

        if not_ready_for_load {
            // Step 6.
            return;
        }

        assert!(!self.loader.borrow().events_inhibited());
        self.loader.borrow_mut().inhibit_events();

        // The rest will ever run only once per document.
        // Step 7.
        debug!("Document loads are complete.");
        let document = Trusted::new(self);
        self.window
            .task_manager()
            .dom_manipulation_task_source()
            .queue(
                task!(fire_load_event: move || {
                    let document = document.root();
                    let window = document.window();
                    if !window.is_alive() {
                        return;
                    }

                    // Step 7.1.
                    document.set_ready_state(DocumentReadyState::Complete);

                    // Step 7.2.
                    if document.browsing_context().is_none() {
                        return;
                    }
                    let event = Event::new(
                        window.upcast(),
                        atom!("load"),
                        EventBubbles::DoesNotBubble,
                        EventCancelable::NotCancelable,
                    );
                    event.set_trusted(true);

                    // http://w3c.github.io/navigation-timing/#widl-PerformanceNavigationTiming-loadEventStart
                    update_with_current_time_ms(&document.load_event_start);

                    debug!("About to dispatch load for {:?}", document.url());
                    // FIXME(nox): Why are errors silenced here?
                    let _ = window.upcast::<EventTarget>().dispatch_event_with_target(
                        document.upcast(),
                        &event,
                    );

                    // http://w3c.github.io/navigation-timing/#widl-PerformanceNavigationTiming-loadEventEnd
                    update_with_current_time_ms(&document.load_event_end);

                    window.reflow(ReflowGoal::Full, ReflowReason::DocumentLoaded);

                    if let Some(fragment) = document.url().fragment() {
                        document.check_and_scroll_fragment(fragment);
                    }
                }),
                self.window.upcast(),
            )
            .unwrap();

        // Step 8.
        let document = Trusted::new(self);
        if document.root().browsing_context().is_some() {
            self.window
                .task_manager()
                .dom_manipulation_task_source()
                .queue(
                    task!(fire_pageshow_event: move || {
                        let document = document.root();
                        let window = document.window();
                        if document.page_showing.get() || !window.is_alive() {
                            return;
                        }

                        document.page_showing.set(true);

                        let event = PageTransitionEvent::new(
                            window,
                            atom!("pageshow"),
                            false, // bubbles
                            false, // cancelable
                            false, // persisted
                        );
                        let event = event.upcast::<Event>();
                        event.set_trusted(true);

                        // FIXME(nox): Why are errors silenced here?
                        let _ = window.upcast::<EventTarget>().dispatch_event_with_target(
                            document.upcast(),
                            &event,
                        );
                    }),
                    self.window.upcast(),
                )
                .unwrap();
        }

        // Step 9.
        // TODO: pending application cache download process tasks.

        // Step 10.
        // TODO: printing steps.

        // Step 11.
        // TODO: ready for post-load tasks.

        // Step 12: completely loaded.
        // https://html.spec.whatwg.org/multipage/#completely-loaded
        // TODO: fully implement "completely loaded".
        let document = Trusted::new(self);
        if document.root().browsing_context().is_some() {
            self.window
                .task_manager()
                .dom_manipulation_task_source()
                .queue(
                    task!(completely_loaded: move || {
                        let document = document.root();
                        document.completely_loaded.set(true);
                        // Note: this will, among others, result in the "iframe-load-event-steps" being run.
                        // https://html.spec.whatwg.org/multipage/#iframe-load-event-steps
                        document.notify_constellation_load();
                    }),
                    self.window.upcast(),
                )
                .unwrap();
        }
    }

    // https://html.spec.whatwg.org/multipage/#pending-parsing-blocking-script
    pub fn set_pending_parsing_blocking_script(
        &self,
        script: &HTMLScriptElement,
        load: Option<ScriptResult>,
    ) {
        assert!(!self.has_pending_parsing_blocking_script());
        *self.pending_parsing_blocking_script.borrow_mut() =
            Some(PendingScript::new_with_load(script, load));
    }

    // https://html.spec.whatwg.org/multipage/#pending-parsing-blocking-script
    pub fn has_pending_parsing_blocking_script(&self) -> bool {
        self.pending_parsing_blocking_script.borrow().is_some()
    }

    /// https://html.spec.whatwg.org/multipage/#prepare-a-script step 22.d.
    pub fn pending_parsing_blocking_script_loaded(
        &self,
        element: &HTMLScriptElement,
        result: ScriptResult,
    ) {
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
        let pair = self
            .pending_parsing_blocking_script
            .borrow_mut()
            .as_mut()
            .and_then(PendingScript::take_result);
        if let Some((element, result)) = pair {
            *self.pending_parsing_blocking_script.borrow_mut() = None;
            self.get_current_parser()
                .unwrap()
                .resume_with_pending_parsing_blocking_script(&element, result);
        }
    }

    // https://html.spec.whatwg.org/multipage/#set-of-scripts-that-will-execute-as-soon-as-possible
    pub fn add_asap_script(&self, script: &HTMLScriptElement) {
        self.asap_scripts_set
            .borrow_mut()
            .push(Dom::from_ref(script));
    }

    /// https://html.spec.whatwg.org/multipage/#the-end step 5.
    /// https://html.spec.whatwg.org/multipage/#prepare-a-script step 22.d.
    pub fn asap_script_loaded(&self, element: &HTMLScriptElement, result: ScriptResult) {
        {
            let mut scripts = self.asap_scripts_set.borrow_mut();
            let idx = scripts
                .iter()
                .position(|entry| &**entry == element)
                .unwrap();
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
    pub fn asap_in_order_script_loaded(&self, element: &HTMLScriptElement, result: ScriptResult) {
        self.asap_in_order_scripts_list.loaded(element, result);
        while let Some((element, result)) = self
            .asap_in_order_scripts_list
            .take_next_ready_to_be_executed()
        {
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
            if let Some((element, result)) = self.deferred_scripts.take_next_ready_to_be_executed()
            {
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
        assert_ne!(
            self.ReadyState(),
            DocumentReadyState::Complete,
            "Complete before DOMContentLoaded?"
        );

        update_with_current_time_ms(&self.dom_content_loaded_event_start);

        // Step 4.1.
        let window = self.window();
        let document = Trusted::new(self);
        window
            .task_manager()
            .dom_manipulation_task_source()
            .queue(
                task!(fire_dom_content_loaded_event: move || {
                let document = document.root();
                document.upcast::<EventTarget>().fire_bubbling_event(atom!("DOMContentLoaded"));
                update_with_current_time_ms(&document.dom_content_loaded_event_end);
                }),
                window.upcast(),
            )
            .unwrap();

        // html parsing has finished - set dom content loaded
        self.interactive_time
            .borrow()
            .maybe_set_tti(self, InteractiveFlag::DOMContentLoaded);

        // Step 4.2.
        // TODO: client message queue.
    }

    // https://html.spec.whatwg.org/multipage/#abort-a-document
    pub fn abort(&self) {
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
        let global_scope = self.window.upcast::<GlobalScope>();
        let loads_cancelled = self.loader.borrow_mut().cancel_all_loads();
        let event_sources_canceled = global_scope.close_event_sources();
        if loads_cancelled || event_sources_canceled {
            // If any loads were canceled.
            self.salvageable.set(false);
        };

        // Also Step 2.
        // Note: the spec says to discard any tasks queued for fetch.
        // This cancels all tasks on the networking task source, which might be too broad.
        // See https://github.com/whatwg/html/issues/3837
        self.window
            .cancel_all_tasks_from_source(TaskSourceName::Networking);

        // Step 3.
        if let Some(parser) = self.get_current_parser() {
            parser.abort();
            self.salvageable.set(false);
        }
    }

    pub fn notify_constellation_load(&self) {
        self.window().send_to_constellation(ScriptMsg::LoadComplete);
    }

    pub fn set_current_parser(&self, script: Option<&ServoParser>) {
        self.current_parser.set(script);
    }

    pub fn get_current_parser(&self) -> Option<DomRoot<ServoParser>> {
        self.current_parser.get()
    }

    pub fn can_invoke_script(&self) -> bool {
        match self.get_current_parser() {
            Some(parser) => {
                // It is safe to run script if the parser is not actively parsing,
                // or if it is impossible to interact with the token stream.
                parser.parser_is_not_active() ||
                    self.throw_on_dynamic_markup_insertion_counter.get() > 0
            },
            None => true,
        }
    }

    /// Iterate over all iframes in the document.
    pub fn iter_iframes(&self) -> impl Iterator<Item = DomRoot<HTMLIFrameElement>> {
        self.upcast::<Node>()
            .traverse_preorder(ShadowIncluding::Yes)
            .filter_map(DomRoot::downcast::<HTMLIFrameElement>)
    }

    /// Find an iframe element in the document.
    pub fn find_iframe(
        &self,
        browsing_context_id: BrowsingContextId,
    ) -> Option<DomRoot<HTMLIFrameElement>> {
        self.iter_iframes()
            .find(|node| node.browsing_context_id() == Some(browsing_context_id))
    }

    pub fn get_dom_loading(&self) -> u64 {
        self.dom_loading.get()
    }

    pub fn get_dom_interactive(&self) -> u64 {
        self.dom_interactive.get()
    }

    pub fn set_navigation_start(&self, navigation_start: u64) {
        self.interactive_time
            .borrow_mut()
            .set_navigation_start(navigation_start);
    }

    pub fn get_interactive_metrics(&self) -> Ref<InteractiveMetrics> {
        self.interactive_time.borrow()
    }

    pub fn has_recorded_tti_metric(&self) -> bool {
        self.get_interactive_metrics().get_tti().is_some()
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

    pub fn get_top_level_dom_complete(&self) -> u64 {
        self.top_level_dom_complete.get()
    }

    pub fn get_load_event_start(&self) -> u64 {
        self.load_event_start.get()
    }

    pub fn get_load_event_end(&self) -> u64 {
        self.load_event_end.get()
    }

    pub fn get_unload_event_start(&self) -> u64 {
        self.unload_event_start.get()
    }

    pub fn get_unload_event_end(&self) -> u64 {
        self.unload_event_end.get()
    }

    pub fn start_tti(&self) {
        if self.get_interactive_metrics().needs_tti() {
            self.tti_window.borrow_mut().start_window();
        }
    }

    /// check tti for this document
    /// if it's been 10s since this doc encountered a task over 50ms, then we consider the
    /// main thread available and try to set tti
    pub fn record_tti_if_necessary(&self) {
        if self.has_recorded_tti_metric() {
            return;
        }
        if self.tti_window.borrow().needs_check() {
            self.get_interactive_metrics().maybe_set_tti(
                self,
                InteractiveFlag::TimeToInteractive(self.tti_window.borrow().get_start()),
            );
        }
    }

    // https://html.spec.whatwg.org/multipage/#fire-a-focus-event
    fn fire_focus_event(
        &self,
        focus_event_type: FocusEventType,
        node: &Node,
        related_target: Option<&EventTarget>,
    ) {
        let (event_name, does_bubble) = match focus_event_type {
            FocusEventType::Focus => (DOMString::from("focus"), EventBubbles::DoesNotBubble),
            FocusEventType::Blur => (DOMString::from("blur"), EventBubbles::DoesNotBubble),
        };
        let event = FocusEvent::new(
            &self.window,
            event_name,
            does_bubble,
            EventCancelable::NotCancelable,
            Some(&self.window),
            0i32,
            related_target,
        );
        let event = event.upcast::<Event>();
        event.set_trusted(true);
        let target = node.upcast();
        event.fire(target);
    }

    /// <https://html.spec.whatwg.org/multipage/#cookie-averse-document-object>
    pub fn is_cookie_averse(&self) -> bool {
        !self.has_browsing_context || !url_has_network_scheme(&self.url())
    }

    /// <https://html.spec.whatwg.org/multipage/#look-up-a-custom-element-definition>
    pub fn lookup_custom_element_definition(
        &self,
        namespace: &Namespace,
        local_name: &LocalName,
        is: Option<&LocalName>,
    ) -> Option<Rc<CustomElementDefinition>> {
        if !pref!(dom.custom_elements.enabled) {
            return None;
        }

        // Step 1
        if *namespace != ns!(html) {
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

    pub fn increment_throw_on_dynamic_markup_insertion_counter(&self) {
        let counter = self.throw_on_dynamic_markup_insertion_counter.get();
        self.throw_on_dynamic_markup_insertion_counter
            .set(counter + 1);
    }

    pub fn decrement_throw_on_dynamic_markup_insertion_counter(&self) {
        let counter = self.throw_on_dynamic_markup_insertion_counter.get();
        self.throw_on_dynamic_markup_insertion_counter
            .set(counter - 1);
    }

    pub fn react_to_environment_changes(&self) {
        for image in self.responsive_images.borrow().iter() {
            image.react_to_environment_changes();
        }
    }

    pub fn register_responsive_image(&self, img: &HTMLImageElement) {
        self.responsive_images.borrow_mut().push(Dom::from_ref(img));
    }

    pub fn unregister_responsive_image(&self, img: &HTMLImageElement) {
        let index = self
            .responsive_images
            .borrow()
            .iter()
            .position(|x| **x == *img);
        if let Some(i) = index {
            self.responsive_images.borrow_mut().remove(i);
        }
    }

    pub fn register_media_controls(&self, controls: &ShadowRoot) -> String {
        let id = Uuid::new_v4().to_string();
        self.media_controls
            .borrow_mut()
            .insert(id.clone(), Dom::from_ref(controls));
        id
    }

    pub fn unregister_media_controls(&self, id: &str) {
        if let Some(media_controls) = self.media_controls.borrow().get(id) {
            media_controls.Host().detach_shadow();
            media_controls
                .upcast::<Node>()
                .dirty(NodeDamage::OtherNodeDamage);
        } else {
            debug_assert!(false, "Trying to unregister unknown media controls");
        }
        self.media_controls.borrow_mut().remove(id);
    }
}

#[derive(MallocSizeOf, PartialEq)]
pub enum DocumentSource {
    FromParser,
    NotFromParser,
}

#[allow(unsafe_code)]
pub trait LayoutDocumentHelpers {
    unsafe fn is_html_document_for_layout(&self) -> bool;
    unsafe fn drain_pending_restyles(&self) -> Vec<(LayoutDom<Element>, PendingRestyle)>;
    unsafe fn needs_paint_from_layout(&self);
    unsafe fn will_paint(&self);
    unsafe fn quirks_mode(&self) -> QuirksMode;
    unsafe fn style_shared_lock(&self) -> &StyleSharedRwLock;
    unsafe fn shadow_roots(&self) -> Vec<LayoutDom<ShadowRoot>>;
    unsafe fn shadow_roots_styles_changed(&self) -> bool;
    unsafe fn flush_shadow_roots_stylesheets(&self);
}

#[allow(unsafe_code)]
impl LayoutDocumentHelpers for LayoutDom<Document> {
    #[inline]
    unsafe fn is_html_document_for_layout(&self) -> bool {
        (*self.unsafe_get()).is_html_document
    }

    #[inline]
    #[allow(unrooted_must_root)]
    unsafe fn drain_pending_restyles(&self) -> Vec<(LayoutDom<Element>, PendingRestyle)> {
        let mut elements = (*self.unsafe_get())
            .pending_restyles
            .borrow_mut_for_layout();
        // Elements were in a document when they were added to this list, but that
        // may no longer be true when the next layout occurs.
        let result = elements
            .drain()
            .map(|(k, v)| (k.to_layout(), v))
            .filter(|&(ref k, _)| k.upcast::<Node>().get_flag(NodeFlags::IS_CONNECTED))
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

    #[inline]
    unsafe fn shadow_roots(&self) -> Vec<LayoutDom<ShadowRoot>> {
        (*self.unsafe_get())
            .shadow_roots
            .borrow_for_layout()
            .iter()
            .map(|sr| sr.to_layout())
            .collect()
    }

    #[inline]
    unsafe fn shadow_roots_styles_changed(&self) -> bool {
        (*self.unsafe_get()).shadow_roots_styles_changed()
    }

    #[inline]
    unsafe fn flush_shadow_roots_stylesheets(&self) {
        (*self.unsafe_get()).flush_shadow_roots_stylesheets()
    }
}

// https://html.spec.whatwg.org/multipage/#is-a-registrable-domain-suffix-of-or-is-equal-to
// The spec says to return a bool, we actually return an Option<Host> containing
// the parsed host in the successful case, to avoid having to re-parse the host.
fn get_registrable_domain_suffix_of_or_is_equal_to(
    host_suffix_string: &str,
    original_host: Host,
) -> Option<Host> {
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
        let index = original_host.len().checked_sub(host.len())?;
        let (prefix, suffix) = original_host.split_at(index);

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

/// <https://url.spec.whatwg.org/#network-scheme>
fn url_has_network_scheme(url: &ServoUrl) -> bool {
    match url.scheme() {
        "ftp" | "http" | "https" => true,
        _ => false,
    }
}

#[derive(Clone, Copy, Eq, JSTraceable, MallocSizeOf, PartialEq)]
pub enum HasBrowsingContext {
    No,
    Yes,
}

impl Document {
    pub fn new_inherited(
        window: &Window,
        has_browsing_context: HasBrowsingContext,
        url: Option<ServoUrl>,
        origin: MutableOrigin,
        is_html_document: IsHTMLDocument,
        content_type: Option<Mime>,
        last_modified: Option<String>,
        activity: DocumentActivity,
        source: DocumentSource,
        doc_loader: DocumentLoader,
        referrer: Option<String>,
        referrer_policy: Option<ReferrerPolicy>,
        canceller: FetchCanceller,
    ) -> Document {
        let url = url.unwrap_or_else(|| ServoUrl::parse("about:blank").unwrap());

        let (ready_state, domcontentloaded_dispatched) = if source == DocumentSource::FromParser {
            (DocumentReadyState::Loading, false)
        } else {
            (DocumentReadyState::Complete, true)
        };

        let interactive_time =
            InteractiveMetrics::new(window.time_profiler_chan().clone(), url.clone());

        let content_type = content_type.unwrap_or_else(|| {
            match is_html_document {
                // https://dom.spec.whatwg.org/#dom-domimplementation-createhtmldocument
                IsHTMLDocument::HTMLDocument => mime::TEXT_HTML,
                // https://dom.spec.whatwg.org/#concept-document-content-type
                IsHTMLDocument::NonHTMLDocument => "application/xml".parse().unwrap(),
            }
        });

        let encoding = content_type
            .get_param(mime::CHARSET)
            .and_then(|charset| Encoding::for_label(charset.as_str().as_bytes()))
            .unwrap_or(UTF_8);

        let has_browsing_context = has_browsing_context == HasBrowsingContext::Yes;
        Document {
            node: Node::new_document_node(),
            document_or_shadow_root: DocumentOrShadowRoot::new(window),
            window: Dom::from_ref(window),
            has_browsing_context,
            implementation: Default::default(),
            content_type,
            last_modified: last_modified,
            url: DomRefCell::new(url),
            // https://dom.spec.whatwg.org/#concept-document-quirks
            quirks_mode: Cell::new(QuirksMode::NoQuirks),
            id_map: DomRefCell::new(HashMap::new()),
            // https://dom.spec.whatwg.org/#concept-document-encoding
            encoding: Cell::new(encoding),
            is_html_document: is_html_document == IsHTMLDocument::HTMLDocument,
            activity: Cell::new(activity),
            tag_map: DomRefCell::new(HashMap::new()),
            tagns_map: DomRefCell::new(HashMap::new()),
            classes_map: DomRefCell::new(HashMap::new()),
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
                    /// <https://github.com/servo/servo/issues/16027>
                    /// (Need to figure out what to do with the style attribute
                    /// of elements adopted into another document.)
                    static ref PER_PROCESS_AUTHOR_SHARED_LOCK: StyleSharedRwLock = {
                        StyleSharedRwLock::new()
                    };
                }
                PER_PROCESS_AUTHOR_SHARED_LOCK.clone()
                //StyleSharedRwLock::new()
            },
            stylesheets: DomRefCell::new(DocumentStylesheetSet::new()),
            stylesheet_list: MutNullableDom::new(None),
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
            scripting_enabled: has_browsing_context,
            animation_frame_ident: Cell::new(0),
            animation_frame_list: DomRefCell::new(vec![]),
            running_animation_callbacks: Cell::new(false),
            loader: DomRefCell::new(doc_loader),
            current_parser: Default::default(),
            reflow_timeout: Cell::new(None),
            base_element: Default::default(),
            appropriate_template_contents_owner_document: Default::default(),
            pending_restyles: DomRefCell::new(HashMap::new()),
            needs_paint: Cell::new(false),
            active_touch_points: DomRefCell::new(Vec::new()),
            dom_loading: Cell::new(Default::default()),
            dom_interactive: Cell::new(Default::default()),
            dom_content_loaded_event_start: Cell::new(Default::default()),
            dom_content_loaded_event_end: Cell::new(Default::default()),
            dom_complete: Cell::new(Default::default()),
            top_level_dom_complete: Cell::new(Default::default()),
            load_event_start: Cell::new(Default::default()),
            load_event_end: Cell::new(Default::default()),
            unload_event_start: Cell::new(Default::default()),
            unload_event_end: Cell::new(Default::default()),
            https_state: Cell::new(HttpsState::None),
            origin: origin,
            referrer: referrer,
            referrer_policy: Cell::new(referrer_policy),
            target_element: MutNullableDom::new(None),
            last_click_info: DomRefCell::new(None),
            ignore_destructive_writes_counter: Default::default(),
            ignore_opens_during_unload_counter: Default::default(),
            spurious_animation_frames: Cell::new(0),
            dom_count: Cell::new(1),
            fullscreen_element: MutNullableDom::new(None),
            form_id_listener_map: Default::default(),
            interactive_time: DomRefCell::new(interactive_time),
            tti_window: DomRefCell::new(InteractiveWindow::new()),
            canceller: canceller,
            throw_on_dynamic_markup_insertion_counter: Cell::new(0),
            page_showing: Cell::new(false),
            salvageable: Cell::new(true),
            fired_unload: Cell::new(false),
            responsive_images: Default::default(),
            redirect_count: Cell::new(0),
            completely_loaded: Cell::new(false),
            script_and_layout_blockers: Cell::new(0),
            delayed_tasks: Default::default(),
            shadow_roots: DomRefCell::new(HashSet::new()),
            shadow_roots_styles_changed: Cell::new(false),
            media_controls: DomRefCell::new(HashMap::new()),
        }
    }

    /// Prevent any JS or layout from running until the corresponding call to
    /// `remove_script_and_layout_blocker`. Used to isolate periods in which
    /// the DOM is in an unstable state and should not be exposed to arbitrary
    /// web content. Any attempts to invoke content JS or query layout during
    /// that time will trigger a panic. `add_delayed_task` will cause the
    /// provided task to be executed as soon as the last blocker is removed.
    pub fn add_script_and_layout_blocker(&self) {
        self.script_and_layout_blockers
            .set(self.script_and_layout_blockers.get() + 1);
    }

    /// Terminate the period in which JS or layout is disallowed from running.
    /// If no further blockers remain, any delayed tasks in the queue will
    /// be executed in queue order until the queue is empty.
    pub fn remove_script_and_layout_blocker(&self) {
        assert!(self.script_and_layout_blockers.get() > 0);
        self.script_and_layout_blockers
            .set(self.script_and_layout_blockers.get() - 1);
        while self.script_and_layout_blockers.get() == 0 && !self.delayed_tasks.borrow().is_empty()
        {
            let task = self.delayed_tasks.borrow_mut().remove(0);
            task.run_box();
        }
    }

    /// Enqueue a task to run as soon as any JS and layout blockers are removed.
    pub fn add_delayed_task<T: 'static + TaskBox>(&self, task: T) {
        self.delayed_tasks.borrow_mut().push(Box::new(task));
    }

    /// Assert that the DOM is in a state that will allow running content JS or
    /// performing a layout operation.
    pub fn ensure_safe_to_run_script_or_layout(&self) {
        assert_eq!(
            self.script_and_layout_blockers.get(),
            0,
            "Attempt to use script or layout while DOM not in a stable state"
        );
    }

    // https://dom.spec.whatwg.org/#dom-document-document
    pub fn Constructor(window: &Window) -> Fallible<DomRoot<Document>> {
        let doc = window.Document();
        let docloader = DocumentLoader::new(&*doc.loader());
        Ok(Document::new(
            window,
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
            None,
            Default::default(),
        ))
    }

    pub fn new(
        window: &Window,
        has_browsing_context: HasBrowsingContext,
        url: Option<ServoUrl>,
        origin: MutableOrigin,
        doctype: IsHTMLDocument,
        content_type: Option<Mime>,
        last_modified: Option<String>,
        activity: DocumentActivity,
        source: DocumentSource,
        doc_loader: DocumentLoader,
        referrer: Option<String>,
        referrer_policy: Option<ReferrerPolicy>,
        canceller: FetchCanceller,
    ) -> DomRoot<Document> {
        let document = reflect_dom_object(
            Box::new(Document::new_inherited(
                window,
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
                referrer_policy,
                canceller,
            )),
            window,
            DocumentBinding::Wrap,
        );
        {
            let node = document.upcast::<Node>();
            node.set_owner_doc(&document);
        }
        document
    }

    pub fn get_redirect_count(&self) -> u16 {
        self.redirect_count.get()
    }

    pub fn set_redirect_count(&self, count: u16) {
        self.redirect_count.set(count)
    }

    fn create_node_list<F: Fn(&Node) -> bool>(&self, callback: F) -> DomRoot<NodeList> {
        let doc = self.GetDocumentElement();
        let maybe_node = doc.deref().map(Castable::upcast::<Node>);
        let iter = maybe_node
            .iter()
            .flat_map(|node| node.traverse_preorder(ShadowIncluding::No))
            .filter(|node| callback(&node));
        NodeList::new_simple_list(&self.window, iter)
    }

    fn get_html_element(&self) -> Option<DomRoot<HTMLHtmlElement>> {
        self.GetDocumentElement().and_then(DomRoot::downcast)
    }

    /// Return a reference to the per-document shared lock used in stylesheets.
    pub fn style_shared_lock(&self) -> &StyleSharedRwLock {
        &self.style_shared_lock
    }

    /// Flushes the stylesheet list, and returns whether any stylesheet changed.
    pub fn flush_stylesheets_for_reflow(&self) -> bool {
        // NOTE(emilio): The invalidation machinery is used on the replicated
        // list on the layout thread.
        //
        // FIXME(emilio): This really should differentiate between CSSOM changes
        // and normal stylesheets additions / removals, because in the last case
        // the layout thread already has that information and we could avoid
        // dirtying the whole thing.
        let mut stylesheets = self.stylesheets.borrow_mut();
        let have_changed = stylesheets.has_changed();
        stylesheets.flush_without_invalidation();
        have_changed
    }

    /// Returns a `Device` suitable for media query evaluation.
    ///
    /// FIXME(emilio): This really needs to be somehow more in sync with layout.
    /// Feels like a hack.
    pub fn device(&self) -> Device {
        let window_size = self.window().window_size();
        let viewport_size = window_size.initial_viewport;
        let device_pixel_ratio = window_size.device_pixel_ratio;
        Device::new(MediaType::screen(), viewport_size, device_pixel_ratio)
    }

    pub fn salvageable(&self) -> bool {
        self.salvageable.get()
    }

    /// <https://html.spec.whatwg.org/multipage/#appropriate-template-contents-owner-document>
    pub fn appropriate_template_contents_owner_document(&self) -> DomRoot<Document> {
        self.appropriate_template_contents_owner_document
            .or_init(|| {
                let doctype = if self.is_html_document {
                    IsHTMLDocument::HTMLDocument
                } else {
                    IsHTMLDocument::NonHTMLDocument
                };
                let new_doc = Document::new(
                    self.window(),
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
                    None,
                    Default::default(),
                );
                new_doc
                    .appropriate_template_contents_owner_document
                    .set(Some(&new_doc));
                new_doc
            })
    }

    pub fn get_element_by_id(&self, id: &Atom) -> Option<DomRoot<Element>> {
        self.id_map
            .borrow()
            .get(&id)
            .map(|ref elements| DomRoot::from_ref(&*(*elements)[0]))
    }

    pub fn ensure_pending_restyle(&self, el: &Element) -> RefMut<PendingRestyle> {
        let map = self.pending_restyles.borrow_mut();
        RefMut::map(map, |m| {
            m.entry(Dom::from_ref(el))
                .or_insert_with(PendingRestyle::new)
        })
    }

    pub fn element_state_will_change(&self, el: &Element) {
        let mut entry = self.ensure_pending_restyle(el);
        if entry.snapshot.is_none() {
            entry.snapshot = Some(Snapshot::new(el.html_element_in_html_document()));
        }
        let snapshot = entry.snapshot.as_mut().unwrap();
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
            entry.hint.insert(RestyleHint::RESTYLE_STYLE_ATTRIBUTE);
        }

        if vtable_for(el.upcast()).attribute_affects_presentational_hints(attr) {
            entry.hint.insert(RestyleHint::RESTYLE_SELF);
        }

        let snapshot = entry.snapshot.as_mut().unwrap();
        if attr.local_name() == &local_name!("id") {
            snapshot.id_changed = true;
        } else if attr.local_name() == &local_name!("class") {
            snapshot.class_changed = true;
        } else {
            snapshot.other_attributes_changed = true;
        }
        if snapshot.attrs.is_none() {
            let attrs = el
                .attrs()
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

        self.window
            .reflow(ReflowGoal::Full, ReflowReason::ElementStateChanged);
    }

    pub fn incr_ignore_destructive_writes_counter(&self) {
        self.ignore_destructive_writes_counter
            .set(self.ignore_destructive_writes_counter.get() + 1);
    }

    pub fn decr_ignore_destructive_writes_counter(&self) {
        self.ignore_destructive_writes_counter
            .set(self.ignore_destructive_writes_counter.get() - 1);
    }

    pub fn is_prompting_or_unloading(&self) -> bool {
        self.ignore_opens_during_unload_counter.get() > 0
    }

    fn incr_ignore_opens_during_unload_counter(&self) {
        self.ignore_opens_during_unload_counter
            .set(self.ignore_opens_during_unload_counter.get() + 1);
    }

    fn decr_ignore_opens_during_unload_counter(&self) {
        self.ignore_opens_during_unload_counter
            .set(self.ignore_opens_during_unload_counter.get() - 1);
    }

    /// Whether we've seen so many spurious animation frames (i.e. animation frames that didn't
    /// mutate the DOM) that we've decided to fall back to fake ones.
    fn is_faking_animation_frames(&self) -> bool {
        self.spurious_animation_frames.get() >= SPURIOUS_ANIMATION_FRAME_THRESHOLD
    }

    // https://fullscreen.spec.whatwg.org/#dom-element-requestfullscreen
    pub fn enter_fullscreen(&self, pending: &Element) -> Rc<Promise> {
        // Step 1
        let in_compartment_proof = AlreadyInCompartment::assert(&self.global());
        let promise = Promise::new_in_current_compartment(
            &self.global(),
            InCompartment::Already(&in_compartment_proof),
        );
        let mut error = false;

        // Step 4
        // check namespace
        match *pending.namespace() {
            ns!(mathml) => {
                if pending.local_name().as_ref() != "math" {
                    error = true;
                }
            },
            ns!(svg) => {
                if pending.local_name().as_ref() != "svg" {
                    error = true;
                }
            },
            ns!(html) => (),
            _ => error = true,
        }
        // fullscreen element ready check
        if !pending.fullscreen_element_ready_check() {
            error = true;
        }

        if pref!(dom.fullscreen.test) {
            // For reftests we just take over the current window,
            // and don't try to really enter fullscreen.
            info!("Tests don't really enter fullscreen.");
        } else {
            // TODO fullscreen is supported
            // TODO This algorithm is allowed to request fullscreen.
            warn!("Fullscreen not supported yet");
        }

        // Step 5 Parallel start

        let window = self.window();
        // Step 6
        if !error {
            let event = EmbedderMsg::SetFullscreenState(true);
            self.send_to_embedder(event);
        }

        let pipeline_id = self.window().pipeline_id();

        // Step 7
        let trusted_pending = Trusted::new(pending);
        let trusted_promise = TrustedPromise::new(promise.clone());
        let handler = ElementPerformFullscreenEnter::new(trusted_pending, trusted_promise, error);
        // NOTE: This steps should be running in parallel
        // https://fullscreen.spec.whatwg.org/#dom-element-requestfullscreen
        let script_msg = CommonScriptMsg::Task(
            ScriptThreadEventCategory::EnterFullscreen,
            handler,
            pipeline_id,
            TaskSourceName::DOMManipulation,
        );
        let msg = MainThreadScriptMsg::Common(script_msg);
        window.main_thread_script_chan().send(msg).unwrap();

        promise
    }

    // https://fullscreen.spec.whatwg.org/#exit-fullscreen
    pub fn exit_fullscreen(&self) -> Rc<Promise> {
        let global = self.global();
        // Step 1
        let in_compartment_proof = AlreadyInCompartment::assert(&global);
        let promise = Promise::new_in_current_compartment(
            &global,
            InCompartment::Already(&in_compartment_proof),
        );
        // Step 2
        if self.fullscreen_element.get().is_none() {
            promise.reject_error(Error::Type(String::from("fullscreen is null")));
            return promise;
        }
        // TODO Step 3-6
        let element = self.fullscreen_element.get().unwrap();

        // Step 7 Parallel start

        let window = self.window();
        // Step 8
        let event = EmbedderMsg::SetFullscreenState(false);
        self.send_to_embedder(event);

        // Step 9
        let trusted_element = Trusted::new(&*element);
        let trusted_promise = TrustedPromise::new(promise.clone());
        let handler = ElementPerformFullscreenExit::new(trusted_element, trusted_promise);
        let pipeline_id = Some(global.pipeline_id());
        // NOTE: This steps should be running in parallel
        // https://fullscreen.spec.whatwg.org/#exit-fullscreen
        let script_msg = CommonScriptMsg::Task(
            ScriptThreadEventCategory::ExitFullscreen,
            handler,
            pipeline_id,
            TaskSourceName::DOMManipulation,
        );
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
                    window.GetFrameElement().map_or(false, |el| {
                        el.has_attribute(&local_name!("allowfullscreen"))
                    })
                }
            },
        }
    }

    fn reset_form_owner_for_listeners(&self, id: &Atom) {
        let map = self.form_id_listener_map.borrow();
        if let Some(listeners) = map.get(id) {
            for listener in listeners {
                listener
                    .as_maybe_form_control()
                    .expect("Element must be a form control")
                    .reset_form_owner();
            }
        }
    }

    pub fn register_shadow_root(&self, shadow_root: &ShadowRoot) {
        self.shadow_roots
            .borrow_mut()
            .insert(Dom::from_ref(shadow_root));
        self.invalidate_shadow_roots_stylesheets();
    }

    pub fn unregister_shadow_root(&self, shadow_root: &ShadowRoot) {
        let mut shadow_roots = self.shadow_roots.borrow_mut();
        shadow_roots.remove(&Dom::from_ref(shadow_root));
    }

    pub fn invalidate_shadow_roots_stylesheets(&self) {
        self.shadow_roots_styles_changed.set(true);
    }

    pub fn shadow_roots_styles_changed(&self) -> bool {
        self.shadow_roots_styles_changed.get()
    }

    pub fn flush_shadow_roots_stylesheets(&self) {
        if !self.shadow_roots_styles_changed.get() {
            return;
        }
        self.shadow_roots_styles_changed.set(false);
    }

    pub fn stylesheet_count(&self) -> usize {
        self.stylesheets.borrow().len()
    }

    pub fn stylesheet_at(&self, index: usize) -> Option<DomRoot<CSSStyleSheet>> {
        let stylesheets = self.stylesheets.borrow();

        stylesheets
            .get(Origin::Author, index)
            .and_then(|s| s.owner.upcast::<Node>().get_cssom_stylesheet())
    }

    /// Add a stylesheet owned by `owner` to the list of document sheets, in the
    /// correct tree position.
    #[allow(unrooted_must_root)] // Owner needs to be rooted already necessarily.
    pub fn add_stylesheet(&self, owner: &Element, sheet: Arc<Stylesheet>) {
        let stylesheets = &mut *self.stylesheets.borrow_mut();
        let insertion_point = stylesheets
            .iter()
            .map(|(sheet, _origin)| sheet)
            .find(|sheet_in_doc| {
                owner
                    .upcast::<Node>()
                    .is_before(sheet_in_doc.owner.upcast())
            })
            .cloned();

        self.window
            .layout_chan()
            .send(Msg::AddStylesheet(
                sheet.clone(),
                insertion_point.as_ref().map(|s| s.sheet.clone()),
            ))
            .unwrap();

        DocumentOrShadowRoot::add_stylesheet(
            owner,
            StylesheetSetRef::Document(stylesheets),
            sheet,
            insertion_point,
            self.style_shared_lock(),
        );
    }

    /// Remove a stylesheet owned by `owner` from the list of document sheets.
    #[allow(unrooted_must_root)] // Owner needs to be rooted already necessarily.
    pub fn remove_stylesheet(&self, owner: &Element, s: &Arc<Stylesheet>) {
        self.window
            .layout_chan()
            .send(Msg::RemoveStylesheet(s.clone()))
            .unwrap();

        DocumentOrShadowRoot::remove_stylesheet(
            owner,
            s,
            StylesheetSetRef::Document(&mut *self.stylesheets.borrow_mut()),
        )
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

impl ProfilerMetadataFactory for Document {
    fn new_metadata(&self) -> Option<TimerMetadata> {
        Some(TimerMetadata {
            url: String::from(self.url().as_str()),
            iframe: TimerMetadataFrameType::RootWindow,
            incremental: TimerMetadataReflowType::Incremental,
        })
    }
}

impl DocumentMethods for Document {
    // https://drafts.csswg.org/cssom/#dom-document-stylesheets
    fn StyleSheets(&self) -> DomRoot<StyleSheetList> {
        self.stylesheet_list.or_init(|| {
            StyleSheetList::new(
                &self.window,
                StyleSheetListOwner::Document(Dom::from_ref(self)),
            )
        })
    }

    // https://dom.spec.whatwg.org/#dom-document-implementation
    fn Implementation(&self) -> DomRoot<DOMImplementation> {
        self.implementation.or_init(|| DOMImplementation::new(self))
    }

    // https://dom.spec.whatwg.org/#dom-document-url
    fn URL(&self) -> USVString {
        USVString(String::from(self.url().as_str()))
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-activeelement
    fn GetActiveElement(&self) -> Option<DomRoot<Element>> {
        self.document_or_shadow_root.get_active_element(
            self.get_focused_element(),
            self.GetBody(),
            self.GetDocumentElement(),
        )
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
        let host = match get_registrable_domain_suffix_of_or_is_equal_to(&*value, effective_domain)
        {
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
            None => DOMString::new(),
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
        DOMString::from(self.encoding.get().name())
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
        DOMString::from(self.content_type.to_string())
    }

    // https://dom.spec.whatwg.org/#dom-document-doctype
    fn GetDoctype(&self) -> Option<DomRoot<DocumentType>> {
        self.upcast::<Node>()
            .children()
            .filter_map(DomRoot::downcast)
            .next()
    }

    // https://dom.spec.whatwg.org/#dom-document-documentelement
    fn GetDocumentElement(&self) -> Option<DomRoot<Element>> {
        self.upcast::<Node>().child_elements().next()
    }

    // https://dom.spec.whatwg.org/#dom-document-getelementsbytagname
    fn GetElementsByTagName(&self, qualified_name: DOMString) -> DomRoot<HTMLCollection> {
        let qualified_name = LocalName::from(&*qualified_name);
        match self.tag_map.borrow_mut().entry(qualified_name.clone()) {
            Occupied(entry) => DomRoot::from_ref(entry.get()),
            Vacant(entry) => {
                let result =
                    HTMLCollection::by_qualified_name(&self.window, self.upcast(), qualified_name);
                entry.insert(Dom::from_ref(&*result));
                result
            },
        }
    }

    // https://dom.spec.whatwg.org/#dom-document-getelementsbytagnamens
    fn GetElementsByTagNameNS(
        &self,
        maybe_ns: Option<DOMString>,
        tag_name: DOMString,
    ) -> DomRoot<HTMLCollection> {
        let ns = namespace_from_domstring(maybe_ns);
        let local = LocalName::from(tag_name);
        let qname = QualName::new(None, ns, local);
        match self.tagns_map.borrow_mut().entry(qname.clone()) {
            Occupied(entry) => DomRoot::from_ref(entry.get()),
            Vacant(entry) => {
                let result = HTMLCollection::by_qual_tag_name(&self.window, self.upcast(), qname);
                entry.insert(Dom::from_ref(&*result));
                result
            },
        }
    }

    // https://dom.spec.whatwg.org/#dom-document-getelementsbyclassname
    fn GetElementsByClassName(&self, classes: DOMString) -> DomRoot<HTMLCollection> {
        let class_atoms: Vec<Atom> = split_html_space_chars(&classes).map(Atom::from).collect();
        match self.classes_map.borrow_mut().entry(class_atoms.clone()) {
            Occupied(entry) => DomRoot::from_ref(entry.get()),
            Vacant(entry) => {
                let result =
                    HTMLCollection::by_atomic_class_name(&self.window, self.upcast(), class_atoms);
                entry.insert(Dom::from_ref(&*result));
                result
            },
        }
    }

    // https://dom.spec.whatwg.org/#dom-nonelementparentnode-getelementbyid
    fn GetElementById(&self, id: DOMString) -> Option<DomRoot<Element>> {
        self.get_element_by_id(&Atom::from(id))
    }

    // https://dom.spec.whatwg.org/#dom-document-createelement
    fn CreateElement(
        &self,
        mut local_name: DOMString,
        options: &ElementCreationOptions,
    ) -> Fallible<DomRoot<Element>> {
        if xml_name_type(&local_name) == InvalidXMLName {
            debug!("Not a valid element name");
            return Err(Error::InvalidCharacter);
        }
        if self.is_html_document {
            local_name.make_ascii_lowercase();
        }

        let is_xhtml = self.content_type.type_() == mime::APPLICATION &&
            self.content_type.subtype().as_str() == "xhtml" &&
            self.content_type.suffix() == Some(mime::XML);

        let ns = if self.is_html_document || is_xhtml {
            ns!(html)
        } else {
            ns!()
        };

        let name = QualName::new(None, ns, LocalName::from(local_name));
        let is = options.is.as_ref().map(|is| LocalName::from(&**is));
        Ok(Element::create(
            name,
            is,
            self,
            ElementCreator::ScriptCreated,
            CustomElementCreationMode::Synchronous,
        ))
    }

    // https://dom.spec.whatwg.org/#dom-document-createelementns
    fn CreateElementNS(
        &self,
        namespace: Option<DOMString>,
        qualified_name: DOMString,
        options: &ElementCreationOptions,
    ) -> Fallible<DomRoot<Element>> {
        let (namespace, prefix, local_name) = validate_and_extract(namespace, &qualified_name)?;
        let name = QualName::new(prefix, namespace, local_name);
        let is = options.is.as_ref().map(|is| LocalName::from(&**is));
        Ok(Element::create(
            name,
            is,
            self,
            ElementCreator::ScriptCreated,
            CustomElementCreationMode::Synchronous,
        ))
    }

    // https://dom.spec.whatwg.org/#dom-document-createattribute
    fn CreateAttribute(&self, mut local_name: DOMString) -> Fallible<DomRoot<Attr>> {
        if xml_name_type(&local_name) == InvalidXMLName {
            debug!("Not a valid element name");
            return Err(Error::InvalidCharacter);
        }
        if self.is_html_document {
            local_name.make_ascii_lowercase();
        }
        let name = LocalName::from(local_name);
        let value = AttrValue::String("".to_owned());

        Ok(Attr::new(
            &self.window,
            name.clone(),
            value,
            name,
            ns!(),
            None,
            None,
        ))
    }

    // https://dom.spec.whatwg.org/#dom-document-createattributens
    fn CreateAttributeNS(
        &self,
        namespace: Option<DOMString>,
        qualified_name: DOMString,
    ) -> Fallible<DomRoot<Attr>> {
        let (namespace, prefix, local_name) = validate_and_extract(namespace, &qualified_name)?;
        let value = AttrValue::String("".to_owned());
        let qualified_name = LocalName::from(qualified_name);
        Ok(Attr::new(
            &self.window,
            local_name,
            value,
            qualified_name,
            namespace,
            prefix,
            None,
        ))
    }

    // https://dom.spec.whatwg.org/#dom-document-createdocumentfragment
    fn CreateDocumentFragment(&self) -> DomRoot<DocumentFragment> {
        DocumentFragment::new(self)
    }

    // https://dom.spec.whatwg.org/#dom-document-createtextnode
    fn CreateTextNode(&self, data: DOMString) -> DomRoot<Text> {
        Text::new(data, self)
    }

    // https://dom.spec.whatwg.org/#dom-document-createcdatasection
    fn CreateCDATASection(&self, data: DOMString) -> Fallible<DomRoot<CDATASection>> {
        // Step 1
        if self.is_html_document {
            return Err(Error::NotSupported);
        }

        // Step 2
        if data.contains("]]>") {
            return Err(Error::InvalidCharacter);
        }

        // Step 3
        Ok(CDATASection::new(data, self))
    }

    // https://dom.spec.whatwg.org/#dom-document-createcomment
    fn CreateComment(&self, data: DOMString) -> DomRoot<Comment> {
        Comment::new(data, self)
    }

    // https://dom.spec.whatwg.org/#dom-document-createprocessinginstruction
    fn CreateProcessingInstruction(
        &self,
        target: DOMString,
        data: DOMString,
    ) -> Fallible<DomRoot<ProcessingInstruction>> {
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
    fn ImportNode(&self, node: &Node, deep: bool) -> Fallible<DomRoot<Node>> {
        // Step 1.
        if node.is::<Document>() || node.is::<ShadowRoot>() {
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
    fn AdoptNode(&self, node: &Node) -> Fallible<DomRoot<Node>> {
        // Step 1.
        if node.is::<Document>() {
            return Err(Error::NotSupported);
        }

        // Step 2.
        if node.is::<ShadowRoot>() {
            return Err(Error::HierarchyRequest);
        }

        // Step 3.
        Node::adopt(node, self);

        // Step 4.
        Ok(DomRoot::from_ref(node))
    }

    // https://dom.spec.whatwg.org/#dom-document-createevent
    fn CreateEvent(&self, mut interface: DOMString) -> Fallible<DomRoot<Event>> {
        interface.make_ascii_lowercase();
        match &*interface {
            "beforeunloadevent" => Ok(DomRoot::upcast(BeforeUnloadEvent::new_uninitialized(
                &self.window,
            ))),
            "closeevent" => Ok(DomRoot::upcast(CloseEvent::new_uninitialized(
                self.window.upcast(),
            ))),
            "customevent" => Ok(DomRoot::upcast(CustomEvent::new_uninitialized(
                self.window.upcast(),
            ))),
            "errorevent" => Ok(DomRoot::upcast(ErrorEvent::new_uninitialized(
                self.window.upcast(),
            ))),
            "events" | "event" | "htmlevents" | "svgevents" => {
                Ok(Event::new_uninitialized(&self.window.upcast()))
            },
            "focusevent" => Ok(DomRoot::upcast(FocusEvent::new_uninitialized(&self.window))),
            "hashchangeevent" => Ok(DomRoot::upcast(HashChangeEvent::new_uninitialized(
                &self.window,
            ))),
            "keyboardevent" => Ok(DomRoot::upcast(KeyboardEvent::new_uninitialized(
                &self.window,
            ))),
            "messageevent" => Ok(DomRoot::upcast(MessageEvent::new_uninitialized(
                self.window.upcast(),
            ))),
            "mouseevent" | "mouseevents" => {
                Ok(DomRoot::upcast(MouseEvent::new_uninitialized(&self.window)))
            },
            "pagetransitionevent" => Ok(DomRoot::upcast(PageTransitionEvent::new_uninitialized(
                &self.window,
            ))),
            "popstateevent" => Ok(DomRoot::upcast(PopStateEvent::new_uninitialized(
                &self.window,
            ))),
            "progressevent" => Ok(DomRoot::upcast(ProgressEvent::new_uninitialized(
                self.window.upcast(),
            ))),
            "storageevent" => Ok(DomRoot::upcast(StorageEvent::new_uninitialized(
                &self.window,
                "".into(),
            ))),
            "touchevent" => Ok(DomRoot::upcast(TouchEvent::new_uninitialized(
                &self.window,
                &TouchList::new(&self.window, &[]),
                &TouchList::new(&self.window, &[]),
                &TouchList::new(&self.window, &[]),
            ))),
            "uievent" | "uievents" => Ok(DomRoot::upcast(UIEvent::new_uninitialized(&self.window))),
            "webglcontextevent" => Ok(DomRoot::upcast(WebGLContextEvent::new_uninitialized(
                &self.window,
            ))),
            _ => Err(Error::NotSupported),
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-lastmodified
    fn LastModified(&self) -> DOMString {
        match self.last_modified {
            Some(ref t) => DOMString::from(t.clone()),
            None => DOMString::from(
                time::now()
                    .strftime("%m/%d/%Y %H:%M:%S")
                    .unwrap()
                    .to_string(),
            ),
        }
    }

    // https://dom.spec.whatwg.org/#dom-document-createrange
    fn CreateRange(&self) -> DomRoot<Range> {
        Range::new_with_doc(self)
    }

    // https://dom.spec.whatwg.org/#dom-document-createnodeiteratorroot-whattoshow-filter
    fn CreateNodeIterator(
        &self,
        root: &Node,
        what_to_show: u32,
        filter: Option<Rc<NodeFilter>>,
    ) -> DomRoot<NodeIterator> {
        NodeIterator::new(self, root, what_to_show, filter)
    }

    // https://w3c.github.io/touch-events/#idl-def-Document
    fn CreateTouch(
        &self,
        window: &Window,
        target: &EventTarget,
        identifier: i32,
        page_x: Finite<f64>,
        page_y: Finite<f64>,
        screen_x: Finite<f64>,
        screen_y: Finite<f64>,
    ) -> DomRoot<Touch> {
        let client_x = Finite::wrap(*page_x - window.PageXOffset() as f64);
        let client_y = Finite::wrap(*page_y - window.PageYOffset() as f64);
        Touch::new(
            window, identifier, target, screen_x, screen_y, client_x, client_y, page_x, page_y,
        )
    }

    // https://w3c.github.io/touch-events/#idl-def-document-createtouchlist(touch...)
    fn CreateTouchList(&self, touches: &[&Touch]) -> DomRoot<TouchList> {
        TouchList::new(&self.window, &touches)
    }

    // https://dom.spec.whatwg.org/#dom-document-createtreewalker
    fn CreateTreeWalker(
        &self,
        root: &Node,
        what_to_show: u32,
        filter: Option<Rc<NodeFilter>>,
    ) -> DomRoot<TreeWalker> {
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
                    .map(DomRoot::upcast::<Node>)
            } else {
                // Step 2.
                root.upcast::<Node>()
                    .traverse_preorder(ShadowIncluding::No)
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
                Some(elem) => DomRoot::upcast::<Node>(elem),
                None => {
                    let name = QualName::new(None, ns!(svg), local_name!("title"));
                    let elem = Element::create(
                        name,
                        None,
                        self,
                        ElementCreator::ScriptCreated,
                        CustomElementCreationMode::Synchronous,
                    );
                    let parent = root.upcast::<Node>();
                    let child = elem.upcast::<Node>();
                    parent
                        .InsertBefore(child, parent.GetFirstChild().deref())
                        .unwrap()
                },
            }
        } else if root.namespace() == &ns!(html) {
            let elem = root
                .upcast::<Node>()
                .traverse_preorder(ShadowIncluding::No)
                .find(|node| node.is::<HTMLTitleElement>());
            match elem {
                Some(elem) => elem,
                None => match self.GetHead() {
                    Some(head) => {
                        let name = QualName::new(None, ns!(html), local_name!("title"));
                        let elem = Element::create(
                            name,
                            None,
                            self,
                            ElementCreator::ScriptCreated,
                            CustomElementCreationMode::Synchronous,
                        );
                        head.upcast::<Node>().AppendChild(elem.upcast()).unwrap()
                    },
                    None => return,
                },
            }
        } else {
            return;
        };

        elem.SetTextContent(Some(title));
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-head
    fn GetHead(&self) -> Option<DomRoot<HTMLHeadElement>> {
        self.get_html_element().and_then(|root| {
            root.upcast::<Node>()
                .children()
                .filter_map(DomRoot::downcast)
                .next()
        })
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-currentscript
    fn GetCurrentScript(&self) -> Option<DomRoot<HTMLScriptElement>> {
        self.current_script.get()
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-body
    fn GetBody(&self) -> Option<DomRoot<HTMLElement>> {
        self.get_html_element().and_then(|root| {
            let node = root.upcast::<Node>();
            node.children()
                .find(|child| match child.type_id() {
                    NodeTypeId::Element(ElementTypeId::HTMLElement(
                        HTMLElementTypeId::HTMLBodyElement,
                    )) |
                    NodeTypeId::Element(ElementTypeId::HTMLElement(
                        HTMLElementTypeId::HTMLFrameSetElement,
                    )) => true,
                    _ => false,
                })
                .map(|node| DomRoot::downcast(node).unwrap())
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
            NodeTypeId::Element(ElementTypeId::HTMLElement(
                HTMLElementTypeId::HTMLFrameSetElement,
            )) => {},
            _ => return Err(Error::HierarchyRequest),
        }

        // Step 2.
        let old_body = self.GetBody();
        if old_body.deref() == Some(new_body) {
            return Ok(());
        }

        match (self.get_html_element(), &old_body) {
            // Step 3.
            (Some(ref root), &Some(ref child)) => {
                let root = root.upcast::<Node>();
                root.ReplaceChild(new_body.upcast(), child.upcast())
                    .unwrap();
            },

            // Step 4.
            (None, _) => return Err(Error::HierarchyRequest),

            // Step 5.
            (Some(ref root), &None) => {
                let root = root.upcast::<Node>();
                root.AppendChild(new_body.upcast()).unwrap();
            },
        }
        Ok(())
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-getelementsbyname
    fn GetElementsByName(&self, name: DOMString) -> DomRoot<NodeList> {
        self.create_node_list(|node| {
            let element = match node.downcast::<Element>() {
                Some(element) => element,
                None => return false,
            };
            if element.namespace() != &ns!(html) {
                return false;
            }
            element
                .get_attribute(&ns!(), &local_name!("name"))
                .map_or(false, |attr| &**attr.value() == &*name)
        })
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-images
    fn Images(&self) -> DomRoot<HTMLCollection> {
        self.images.or_init(|| {
            let filter = Box::new(ImagesFilter);
            HTMLCollection::create(&self.window, self.upcast(), filter)
        })
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-embeds
    fn Embeds(&self) -> DomRoot<HTMLCollection> {
        self.embeds.or_init(|| {
            let filter = Box::new(EmbedsFilter);
            HTMLCollection::create(&self.window, self.upcast(), filter)
        })
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-plugins
    fn Plugins(&self) -> DomRoot<HTMLCollection> {
        self.Embeds()
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-links
    fn Links(&self) -> DomRoot<HTMLCollection> {
        self.links.or_init(|| {
            let filter = Box::new(LinksFilter);
            HTMLCollection::create(&self.window, self.upcast(), filter)
        })
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-forms
    fn Forms(&self) -> DomRoot<HTMLCollection> {
        self.forms.or_init(|| {
            let filter = Box::new(FormsFilter);
            HTMLCollection::create(&self.window, self.upcast(), filter)
        })
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-scripts
    fn Scripts(&self) -> DomRoot<HTMLCollection> {
        self.scripts.or_init(|| {
            let filter = Box::new(ScriptsFilter);
            HTMLCollection::create(&self.window, self.upcast(), filter)
        })
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-anchors
    fn Anchors(&self) -> DomRoot<HTMLCollection> {
        self.anchors.or_init(|| {
            let filter = Box::new(AnchorsFilter);
            HTMLCollection::create(&self.window, self.upcast(), filter)
        })
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-applets
    fn Applets(&self) -> DomRoot<HTMLCollection> {
        self.applets
            .or_init(|| HTMLCollection::always_empty(&self.window, self.upcast()))
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-location
    fn GetLocation(&self) -> Option<DomRoot<Location>> {
        if self.is_fully_active() {
            Some(self.window.Location())
        } else {
            None
        }
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-children
    fn Children(&self) -> DomRoot<HTMLCollection> {
        HTMLCollection::children(&self.window, self.upcast())
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-firstelementchild
    fn GetFirstElementChild(&self) -> Option<DomRoot<Element>> {
        self.upcast::<Node>().child_elements().next()
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-lastelementchild
    fn GetLastElementChild(&self) -> Option<DomRoot<Element>> {
        self.upcast::<Node>()
            .rev_children()
            .filter_map(DomRoot::downcast)
            .next()
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
    fn QuerySelector(&self, selectors: DOMString) -> Fallible<Option<DomRoot<Element>>> {
        let root = self.upcast::<Node>();
        root.query_selector(selectors)
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-queryselectorall
    fn QuerySelectorAll(&self, selectors: DOMString) -> Fallible<DomRoot<NodeList>> {
        let root = self.upcast::<Node>();
        root.query_selector_all(selectors)
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-readystate
    fn ReadyState(&self) -> DocumentReadyState {
        self.ready_state.get()
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-defaultview
    fn GetDefaultView(&self) -> Option<DomRoot<Window>> {
        if self.has_browsing_context {
            Some(DomRoot::from_ref(&*self.window))
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
        let (tx, rx) = profile_ipc::channel(self.global().time_profiler_chan().clone()).unwrap();
        let _ = self
            .window
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

        let cookies = if let Some(cookie) = Cookie::parse(cookie.to_string()).ok().map(Serde) {
            vec![cookie]
        } else {
            vec![]
        };

        let _ = self
            .window
            .upcast::<GlobalScope>()
            .resource_threads()
            .send(SetCookiesForUrl(self.url(), cookies, NonHTTP));
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
    unsafe fn NamedGetter(
        &self,
        _cx: *mut JSContext,
        name: DOMString,
    ) -> Option<NonNull<JSObject>> {
        #[derive(JSTraceable, MallocSizeOf)]
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
            let mut elements = root
                .traverse_preorder(ShadowIncluding::No)
                .filter(|node| filter_by_name(&name, &node))
                .peekable();
            if let Some(first) = elements.next() {
                if elements.peek().is_none() {
                    // TODO: Step 2.
                    // Step 3.
                    return Some(NonNull::new_unchecked(
                        first.reflector().get_jsobject().get(),
                    ));
                }
            } else {
                return None;
            }
        }
        // Step 4.
        let filter = NamedElementFilter { name: name };
        let collection = HTMLCollection::create(self.window(), root, Box::new(filter));
        Some(NonNull::new_unchecked(
            collection.reflector().get_jsobject().get(),
        ))
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
    event_handler!(
        readystatechange,
        GetOnreadystatechange,
        SetOnreadystatechange
    );

    // https://drafts.csswg.org/cssom-view/#dom-document-elementfrompoint
    fn ElementFromPoint(&self, x: Finite<f64>, y: Finite<f64>) -> Option<DomRoot<Element>> {
        self.document_or_shadow_root.element_from_point(
            x,
            y,
            self.GetDocumentElement(),
            self.has_browsing_context,
        )
    }

    // https://drafts.csswg.org/cssom-view/#dom-document-elementsfrompoint
    fn ElementsFromPoint(&self, x: Finite<f64>, y: Finite<f64>) -> Vec<DomRoot<Element>> {
        self.document_or_shadow_root.elements_from_point(
            x,
            y,
            self.GetDocumentElement(),
            self.has_browsing_context,
        )
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-open
    fn Open(
        &self,
        _unused1: Option<DOMString>,
        _unused2: Option<DOMString>,
    ) -> Fallible<DomRoot<Document>> {
        // Step 1
        if !self.is_html_document() {
            return Err(Error::InvalidState);
        }

        // Step 2
        if self.throw_on_dynamic_markup_insertion_counter.get() > 0 {
            return Err(Error::InvalidState);
        }

        // Step 3
        let entry_responsible_document = GlobalScope::entry().as_window().Document();

        // Step 4
        // This check is same-origin not same-origin-domain.
        // https://github.com/whatwg/html/issues/2282
        // https://github.com/whatwg/html/pull/2288
        if !self.origin.same_origin(&entry_responsible_document.origin) {
            return Err(Error::Security);
        }

        // Step 5
        if self
            .get_current_parser()
            .map_or(false, |parser| parser.is_active())
        {
            return Ok(DomRoot::from_ref(self));
        }

        // Step 6
        if self.is_prompting_or_unloading() {
            return Ok(DomRoot::from_ref(self));
        }

        // TODO: prompt to unload.
        // TODO: set unload_event_start and unload_event_end

        window_from_node(self).set_navigation_start();

        // Step 7
        // TODO: https://github.com/servo/servo/issues/21937
        if self.has_browsing_context() {
            self.abort();
        }

        // Step 8
        for node in self
            .upcast::<Node>()
            .traverse_preorder(ShadowIncluding::Yes)
        {
            node.upcast::<EventTarget>().remove_all_listeners();
        }

        // Step 9
        if self.window.Document() == DomRoot::from_ref(self) {
            self.window.upcast::<EventTarget>().remove_all_listeners();
        }

        // Step 10
        // TODO: https://github.com/servo/servo/issues/21936
        Node::replace_all(None, self.upcast::<Node>());

        // Step 11
        if self.is_fully_active() {
            let mut new_url = entry_responsible_document.url();
            if entry_responsible_document != DomRoot::from_ref(self) {
                new_url.set_fragment(None);
            }
            // TODO: https://github.com/servo/servo/issues/21939
            self.set_url(new_url);
        }

        // Step 12
        // TODO: https://github.com/servo/servo/issues/21938

        // Step 13
        self.set_quirks_mode(QuirksMode::NoQuirks);

        // Step 14
        let resource_threads = self
            .window
            .upcast::<GlobalScope>()
            .resource_threads()
            .clone();
        *self.loader.borrow_mut() =
            DocumentLoader::new_with_threads(resource_threads, Some(self.url()));
        ServoParser::parse_html_script_input(self, self.url());

        // Step 15
        self.ready_state.set(DocumentReadyState::Loading);

        // Step 16
        // Handled when creating the parser in step 14

        // Step 17
        Ok(DomRoot::from_ref(self))
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-open-window
    fn Open_(
        &self,
        url: DOMString,
        target: DOMString,
        features: DOMString,
    ) -> Fallible<DomRoot<WindowProxy>> {
        // WhatWG spec states this should always return a WindowProxy, but the spec for WindowProxy.open states
        // it optionally returns a WindowProxy. Assume an error if window.open returns none.
        // See https://github.com/whatwg/html/issues/4091
        let context = self.browsing_context().ok_or(Error::InvalidAccess)?;
        context
            .open(url, target, features)
            .ok_or(Error::InvalidAccess)
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-write
    fn Write(&self, text: Vec<DOMString>) -> ErrorResult {
        if !self.is_html_document() {
            // Step 1.
            return Err(Error::InvalidState);
        }

        // Step 2.
        if self.throw_on_dynamic_markup_insertion_counter.get() > 0 {
            return Err(Error::InvalidState);
        }

        if !self.is_active() {
            // Step 3.
            return Ok(());
        }

        let parser = match self.get_current_parser() {
            Some(ref parser) if parser.can_write() => DomRoot::from_ref(&**parser),
            _ => {
                // Either there is no parser, which means the parsing ended;
                // or script nesting level is 0, which means the method was
                // called from outside a parser-executed script.
                if self.is_prompting_or_unloading() ||
                    self.ignore_destructive_writes_counter.get() > 0
                {
                    // Step 4.
                    return Ok(());
                }
                // Step 5.
                self.Open(None, None)?;
                self.get_current_parser().unwrap()
            },
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
        if self.throw_on_dynamic_markup_insertion_counter.get() > 0 {
            return Err(Error::InvalidState);
        }

        let parser = match self.get_current_parser() {
            Some(ref parser) if parser.is_script_created() => DomRoot::from_ref(&**parser),
            _ => {
                // Step 3.
                return Ok(());
            },
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
    event_handler!(
        fullscreenchange,
        GetOnfullscreenchange,
        SetOnfullscreenchange
    );

    // https://fullscreen.spec.whatwg.org/#dom-document-fullscreenenabled
    fn FullscreenEnabled(&self) -> bool {
        self.get_allow_fullscreen()
    }

    // https://fullscreen.spec.whatwg.org/#dom-document-fullscreen
    fn Fullscreen(&self) -> bool {
        self.fullscreen_element.get().is_some()
    }

    // https://fullscreen.spec.whatwg.org/#dom-document-fullscreenelement
    fn GetFullscreenElement(&self) -> Option<DomRoot<Element>> {
        // TODO ShadowRoot
        self.fullscreen_element.get()
    }

    // https://fullscreen.spec.whatwg.org/#dom-document-exitfullscreen
    fn ExitFullscreen(&self) -> Rc<Promise> {
        self.exit_fullscreen()
    }

    // check-tidy: no specs after this line
    // Servo only API to get an instance of the controls of a specific
    // media element matching the given id.
    fn ServoGetMediaControls(&self, id: DOMString) -> Fallible<DomRoot<ShadowRoot>> {
        match self.media_controls.borrow().get(&*id) {
            Some(m) => Ok(DomRoot::from_ref(&*m)),
            None => Err(Error::InvalidAccess),
        }
    }
}

fn update_with_current_time_ms(marker: &Cell<u64>) {
    if marker.get() == 0 {
        let time = time::get_time();
        let current_time_ms = time.sec * 1000 + time.nsec as i64 / 1000000;
        marker.set(current_time_ms as u64);
    }
}

/// <https://w3c.github.io/webappsec-referrer-policy/#determine-policy-for-token>
pub fn determine_policy_for_token(token: &str) -> Option<ReferrerPolicy> {
    match_ignore_ascii_case! { token,
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

/// Specifies the type of focus event that is sent to a pipeline
#[derive(Clone, Copy, PartialEq)]
pub enum FocusType {
    Element, // The first focus message - focus the element itself
    Parent,  // Focusing a parent element (an iframe)
}

/// Focus events
pub enum FocusEventType {
    Focus, // Element gained focus. Doesn't bubble.
    Blur,  // Element lost focus. Doesn't bubble.
}

/// A fake `requestAnimationFrame()` callback—"fake" because it is not triggered by the video
/// refresh but rather a simple timer.
///
/// If the page is observed to be using `requestAnimationFrame()` for non-animation purposes (i.e.
/// without mutating the DOM), then we fall back to simple timeouts to save energy over video
/// refresh.
#[derive(JSTraceable, MallocSizeOf)]
pub struct FakeRequestAnimationFrameCallback {
    /// The document.
    #[ignore_malloc_size_of = "non-owning"]
    document: Trusted<Document>,
}

impl FakeRequestAnimationFrameCallback {
    pub fn invoke(self) {
        let document = self.document.root();
        document.run_the_animation_frame_callbacks();
    }
}

#[derive(JSTraceable, MallocSizeOf)]
pub enum AnimationFrameCallback {
    DevtoolsFramerateTick {
        actor_name: String,
    },
    FrameRequestCallback {
        #[ignore_malloc_size_of = "Rc is hard"]
        callback: Rc<FrameRequestCallback>,
    },
}

impl AnimationFrameCallback {
    fn call(&self, document: &Document, now: f64) {
        match *self {
            AnimationFrameCallback::DevtoolsFramerateTick { ref actor_name } => {
                let msg = ScriptToDevtoolsControlMsg::FramerateTick(actor_name.clone(), now);
                let devtools_sender = document
                    .window()
                    .upcast::<GlobalScope>()
                    .devtools_chan()
                    .unwrap();
                devtools_sender.send(msg).unwrap();
            },
            AnimationFrameCallback::FrameRequestCallback { ref callback } => {
                // TODO(jdm): The spec says that any exceptions should be suppressed:
                // https://github.com/servo/servo/issues/6928
                let _ = callback.Call__(Finite::wrap(now), ExceptionHandling::Report);
            },
        }
    }
}

#[derive(Default, JSTraceable, MallocSizeOf)]
#[must_root]
struct PendingInOrderScriptVec {
    scripts: DomRefCell<VecDeque<PendingScript>>,
}

impl PendingInOrderScriptVec {
    fn is_empty(&self) -> bool {
        self.scripts.borrow().is_empty()
    }

    fn push(&self, element: &HTMLScriptElement) {
        self.scripts
            .borrow_mut()
            .push_back(PendingScript::new(element));
    }

    fn loaded(&self, element: &HTMLScriptElement, result: ScriptResult) {
        let mut scripts = self.scripts.borrow_mut();
        let entry = scripts
            .iter_mut()
            .find(|entry| &*entry.element == element)
            .unwrap();
        entry.loaded(result);
    }

    fn take_next_ready_to_be_executed(&self) -> Option<(DomRoot<HTMLScriptElement>, ScriptResult)> {
        let mut scripts = self.scripts.borrow_mut();
        let pair = scripts.front_mut()?.take_result()?;
        scripts.pop_front();
        Some(pair)
    }

    fn clear(&self) {
        *self.scripts.borrow_mut() = Default::default();
    }
}

#[derive(JSTraceable, MallocSizeOf)]
#[must_root]
struct PendingScript {
    element: Dom<HTMLScriptElement>,
    load: Option<ScriptResult>,
}

impl PendingScript {
    fn new(element: &HTMLScriptElement) -> Self {
        Self {
            element: Dom::from_ref(element),
            load: None,
        }
    }

    fn new_with_load(element: &HTMLScriptElement, load: Option<ScriptResult>) -> Self {
        Self {
            element: Dom::from_ref(element),
            load,
        }
    }

    fn loaded(&mut self, result: ScriptResult) {
        assert!(self.load.is_none());
        self.load = Some(result);
    }

    fn take_result(&mut self) -> Option<(DomRoot<HTMLScriptElement>, ScriptResult)> {
        self.load
            .take()
            .map(|result| (DomRoot::from_ref(&*self.element), result))
    }
}
