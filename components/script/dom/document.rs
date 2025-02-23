/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::Cow;
use std::cell::{Cell, RefCell};
use std::cmp::Ordering;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::{HashMap, HashSet, VecDeque};
use std::default::Default;
use std::f64::consts::PI;
use std::mem;
use std::rc::Rc;
use std::slice::from_ref;
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};

use base::cross_process_instant::CrossProcessInstant;
use base::id::WebViewId;
use canvas_traits::webgl::{self, WebGLContextId, WebGLMsg};
use chrono::Local;
use content_security_policy::{self as csp, CspList, PolicyDisposition};
use cookie::Cookie;
use cssparser::match_ignore_ascii_case;
use devtools_traits::ScriptToDevtoolsControlMsg;
use dom_struct::dom_struct;
use embedder_traits::{
    AllowOrDeny, ContextMenuResult, EditingActionEvent, EmbedderMsg, ImeEvent, InputEvent,
    LoadStatus, MouseButton, MouseButtonAction, MouseButtonEvent, TouchEvent, TouchEventType,
    TouchId, WheelEvent,
};
use encoding_rs::{Encoding, UTF_8};
use euclid::default::{Point2D, Rect, Size2D};
use html5ever::{local_name, namespace_url, ns, LocalName, Namespace, QualName};
use hyper_serde::Serde;
use ipc_channel::ipc;
use js::rust::{HandleObject, HandleValue};
use keyboard_types::{Code, Key, KeyState};
use metrics::{
    InteractiveFlag, InteractiveMetrics, InteractiveWindow, ProfilerMetadataFactory,
    ProgressiveWebMetric,
};
use mime::{self, Mime};
use net_traits::policy_container::PolicyContainer;
use net_traits::pub_domains::is_pub_domain;
use net_traits::request::{InsecureRequestsPolicy, RequestBuilder};
use net_traits::response::HttpsState;
use net_traits::CookieSource::NonHTTP;
use net_traits::CoreResourceMsg::{GetCookiesForUrl, SetCookiesForUrl};
use net_traits::{FetchResponseListener, IpcSend, ReferrerPolicy};
use num_traits::ToPrimitive;
use percent_encoding::percent_decode;
use profile_traits::ipc as profile_ipc;
use profile_traits::time::{TimerMetadata, TimerMetadataFrameType, TimerMetadataReflowType};
use script_layout_interface::{PendingRestyle, TrustedNodeAddress};
use script_traits::{
    AnimationState, AnimationTickType, ConstellationInputEvent, DocumentActivity, ScriptMsg,
};
use servo_arc::Arc;
use servo_atoms::Atom;
use servo_config::pref;
use servo_media::{ClientContextId, ServoMedia};
use servo_url::{ImmutableOrigin, MutableOrigin, ServoUrl};
use style::attr::AttrValue;
use style::context::QuirksMode;
use style::invalidation::element::restyle_hints::RestyleHint;
use style::selector_parser::Snapshot;
use style::shared_lock::SharedRwLock as StyleSharedRwLock;
use style::str::{split_html_space_chars, str_join};
use style::stylesheet_set::DocumentStylesheetSet;
use style::stylesheets::{Origin, OriginSet, Stylesheet};
use url::Host;
use uuid::Uuid;
#[cfg(feature = "webgpu")]
use webgpu::swapchain::WebGPUContextId;
use webrender_api::units::DeviceIntRect;
use webrender_traits::CompositorHitTestResult;

use super::bindings::codegen::Bindings::XPathEvaluatorBinding::XPathEvaluatorMethods;
use super::clipboardevent::ClipboardEventType;
use crate::animation_timeline::AnimationTimeline;
use crate::animations::Animations;
use crate::canvas_context::CanvasContext as _;
use crate::document_loader::{DocumentLoader, LoadType};
use crate::dom::attr::Attr;
use crate::dom::beforeunloadevent::BeforeUnloadEvent;
use crate::dom::bindings::callback::ExceptionHandling;
use crate::dom::bindings::cell::{DomRefCell, Ref, RefMut};
use crate::dom::bindings::codegen::Bindings::BeforeUnloadEventBinding::BeforeUnloadEvent_Binding::BeforeUnloadEventMethods;
use crate::dom::bindings::codegen::Bindings::DocumentBinding::{
    DocumentMethods, DocumentReadyState, DocumentVisibilityState, NamedPropertyValue,
};
use crate::dom::bindings::codegen::Bindings::EventBinding::Event_Binding::EventMethods;
use crate::dom::bindings::codegen::Bindings::HTMLIFrameElementBinding::HTMLIFrameElement_Binding::HTMLIFrameElementMethods;
use crate::dom::bindings::codegen::Bindings::HTMLInputElementBinding::HTMLInputElementMethods;
use crate::dom::bindings::codegen::Bindings::HTMLTextAreaElementBinding::HTMLTextAreaElementMethods;
use crate::dom::bindings::codegen::Bindings::NavigatorBinding::Navigator_Binding::NavigatorMethods;
use crate::dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use crate::dom::bindings::codegen::Bindings::NodeFilterBinding::NodeFilter;
use crate::dom::bindings::codegen::Bindings::PerformanceBinding::PerformanceMethods;
use crate::dom::bindings::codegen::Bindings::PermissionStatusBinding::PermissionName;
use crate::dom::bindings::codegen::Bindings::ShadowRootBinding::ShadowRootMethods;
use crate::dom::bindings::codegen::Bindings::TouchBinding::TouchMethods;
use crate::dom::bindings::codegen::Bindings::WindowBinding::{
    FrameRequestCallback, ScrollBehavior, WindowMethods,
};
use crate::dom::bindings::codegen::Bindings::XPathNSResolverBinding::XPathNSResolver;
use crate::dom::bindings::codegen::UnionTypes::{NodeOrString, StringOrElementCreationOptions};
use crate::dom::bindings::error::{Error, ErrorInfo, ErrorResult, Fallible};
use crate::dom::bindings::inheritance::{Castable, ElementTypeId, HTMLElementTypeId, NodeTypeId};
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::refcounted::{Trusted, TrustedPromise};
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, DomGlobal};
use crate::dom::bindings::root::{Dom, DomRoot, DomSlice, LayoutDom, MutNullableDom, ToLayout};
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::bindings::trace::{HashMapTracedValues, NoTrace};
#[cfg(feature = "webgpu")]
use crate::dom::bindings::weakref::WeakRef;
use crate::dom::bindings::xmlname::{
    matches_name_production, namespace_from_domstring, validate_and_extract,
};
use crate::dom::cdatasection::CDATASection;
use crate::dom::clipboardevent::ClipboardEvent;
use crate::dom::comment::Comment;
use crate::dom::compositionevent::CompositionEvent;
use crate::dom::cssstylesheet::CSSStyleSheet;
use crate::dom::customelementregistry::CustomElementDefinition;
use crate::dom::customevent::CustomEvent;
use crate::dom::datatransfer::DataTransfer;
use crate::dom::documentfragment::DocumentFragment;
use crate::dom::documentorshadowroot::{DocumentOrShadowRoot, StyleSheetInDocument};
use crate::dom::documenttype::DocumentType;
use crate::dom::domimplementation::DOMImplementation;
use crate::dom::element::{
    CustomElementCreationMode, Element, ElementCreator, ElementPerformFullscreenEnter,
    ElementPerformFullscreenExit,
};
use crate::dom::event::{Event, EventBubbles, EventCancelable, EventDefault, EventStatus};
use crate::dom::eventtarget::EventTarget;
use crate::dom::focusevent::FocusEvent;
use crate::dom::fontfaceset::FontFaceSet;
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
use crate::dom::htmlinputelement::HTMLInputElement;
use crate::dom::htmlmetaelement::RefreshRedirectDue;
use crate::dom::htmlscriptelement::{HTMLScriptElement, ScriptResult};
use crate::dom::htmltextareaelement::HTMLTextAreaElement;
use crate::dom::htmltitleelement::HTMLTitleElement;
use crate::dom::keyboardevent::KeyboardEvent;
use crate::dom::location::Location;
use crate::dom::messageevent::MessageEvent;
use crate::dom::mouseevent::MouseEvent;
use crate::dom::node::{
    self, CloneChildrenFlag, Node, NodeDamage, NodeFlags, NodeTraits, ShadowIncluding,
};
use crate::dom::nodeiterator::NodeIterator;
use crate::dom::nodelist::NodeList;
use crate::dom::pagetransitionevent::PageTransitionEvent;
use crate::dom::performanceentry::PerformanceEntry;
use crate::dom::pointerevent::PointerEvent;
use crate::dom::processinginstruction::ProcessingInstruction;
use crate::dom::promise::Promise;
use crate::dom::range::Range;
use crate::dom::resizeobserver::{ResizeObservationDepth, ResizeObserver};
use crate::dom::selection::Selection;
use crate::dom::servoparser::ServoParser;
use crate::dom::shadowroot::ShadowRoot;
use crate::dom::storageevent::StorageEvent;
use crate::dom::stylesheetlist::{StyleSheetList, StyleSheetListOwner};
use crate::dom::text::Text;
use crate::dom::touch::Touch;
use crate::dom::touchevent::TouchEvent as DomTouchEvent;
use crate::dom::touchlist::TouchList;
use crate::dom::treewalker::TreeWalker;
use crate::dom::types::VisibilityStateEntry;
use crate::dom::uievent::UIEvent;
use crate::dom::virtualmethods::vtable_for;
use crate::dom::webglrenderingcontext::WebGLRenderingContext;
#[cfg(feature = "webgpu")]
use crate::dom::webgpu::gpucanvascontext::GPUCanvasContext;
use crate::dom::wheelevent::WheelEvent as DomWheelEvent;
use crate::dom::window::Window;
use crate::dom::windowproxy::WindowProxy;
use crate::dom::xpathevaluator::XPathEvaluator;
use crate::drag_data_store::{DragDataStore, Kind, Mode};
use crate::fetch::FetchCanceller;
use crate::iframe_collection::IFrameCollection;
use crate::messaging::{CommonScriptMsg, MainThreadScriptMsg};
use crate::network_listener::{NetworkListener, PreInvoke};
use crate::realms::{enter_realm, AlreadyInRealm, InRealm};
use crate::script_runtime::{CanGc, ScriptThreadEventCategory};
use crate::script_thread::{with_script_thread, ScriptThread};
use crate::stylesheet_set::StylesheetSetRef;
use crate::task::TaskBox;
use crate::task_source::TaskSourceName;
use crate::timers::OneshotTimerCallback;
use crate::DomTypes;

/// The number of times we are allowed to see spurious `requestAnimationFrame()` calls before
/// falling back to fake ones.
///
/// A spurious `requestAnimationFrame()` call is defined as one that does not change the DOM.
const SPURIOUS_ANIMATION_FRAME_THRESHOLD: u8 = 5;

/// The amount of time between fake `requestAnimationFrame()`s.
const FAKE_REQUEST_ANIMATION_FRAME_DELAY: u64 = 16;

pub(crate) enum TouchEventResult {
    Processed(bool),
    Forwarded,
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum FireMouseEventType {
    Move,
    Over,
    Out,
    Enter,
    Leave,
}

impl FireMouseEventType {
    pub(crate) fn as_str(&self) -> &str {
        match *self {
            FireMouseEventType::Move => "mousemove",
            FireMouseEventType::Over => "mouseover",
            FireMouseEventType::Out => "mouseout",
            FireMouseEventType::Enter => "mouseenter",
            FireMouseEventType::Leave => "mouseleave",
        }
    }
}

#[derive(Clone, Copy, Debug, JSTraceable, MallocSizeOf, PartialEq)]
pub(crate) enum IsHTMLDocument {
    HTMLDocument,
    NonHTMLDocument,
}

#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
enum FocusTransaction {
    /// No focus operation is in effect.
    NotInTransaction,
    /// A focus operation is in effect.
    /// Contains the element that has most recently requested focus for itself.
    InTransaction(Option<Dom<Element>>),
}

/// Information about a declarative refresh
#[derive(JSTraceable, MallocSizeOf)]
pub(crate) enum DeclarativeRefresh {
    PendingLoad {
        #[no_trace]
        url: ServoUrl,
        time: u64,
    },
    CreatedAfterLoad,
}
#[cfg(feature = "webgpu")]
pub(crate) type WebGPUContextsMap =
    Rc<RefCell<HashMapTracedValues<WebGPUContextId, WeakRef<GPUCanvasContext>>>>;

/// <https://dom.spec.whatwg.org/#document>
#[dom_struct]
pub(crate) struct Document {
    node: Node,
    document_or_shadow_root: DocumentOrShadowRoot,
    window: Dom<Window>,
    implementation: MutNullableDom<DOMImplementation>,
    #[ignore_malloc_size_of = "type from external crate"]
    #[no_trace]
    content_type: Mime,
    last_modified: Option<String>,
    #[no_trace]
    encoding: Cell<&'static Encoding>,
    has_browsing_context: bool,
    is_html_document: bool,
    #[no_trace]
    activity: Cell<DocumentActivity>,
    #[no_trace]
    url: DomRefCell<ServoUrl>,
    #[ignore_malloc_size_of = "defined in selectors"]
    #[no_trace]
    quirks_mode: Cell<QuirksMode>,
    /// Caches for the getElement methods
    id_map: DomRefCell<HashMapTracedValues<Atom, Vec<Dom<Element>>>>,
    name_map: DomRefCell<HashMapTracedValues<Atom, Vec<Dom<Element>>>>,
    tag_map: DomRefCell<HashMapTracedValues<LocalName, Dom<HTMLCollection>>>,
    tagns_map: DomRefCell<HashMapTracedValues<QualName, Dom<HTMLCollection>>>,
    classes_map: DomRefCell<HashMapTracedValues<Vec<Atom>, Dom<HTMLCollection>>>,
    images: MutNullableDom<HTMLCollection>,
    embeds: MutNullableDom<HTMLCollection>,
    links: MutNullableDom<HTMLCollection>,
    forms: MutNullableDom<HTMLCollection>,
    scripts: MutNullableDom<HTMLCollection>,
    anchors: MutNullableDom<HTMLCollection>,
    applets: MutNullableDom<HTMLCollection>,
    /// Information about the `<iframes>` in this [`Document`].
    iframes: RefCell<IFrameCollection>,
    /// Lock use for style attributes and author-origin stylesheet objects in this document.
    /// Can be acquired once for accessing many objects.
    #[no_trace]
    style_shared_lock: StyleSharedRwLock,
    /// List of stylesheets associated with nodes in this document. |None| if the list needs to be refreshed.
    #[custom_trace]
    stylesheets: DomRefCell<DocumentStylesheetSet<StyleSheetInDocument>>,
    stylesheet_list: MutNullableDom<StyleSheetList>,
    ready_state: Cell<DocumentReadyState>,
    /// Whether the DOMContentLoaded event has already been dispatched.
    domcontentloaded_dispatched: Cell<bool>,
    /// The state of this document's focus transaction.
    focus_transaction: DomRefCell<FocusTransaction>,
    /// The element that currently has the document focus context.
    focused: MutNullableDom<Element>,
    /// The script element that is currently executing.
    current_script: MutNullableDom<HTMLScriptElement>,
    /// <https://html.spec.whatwg.org/multipage/#pending-parsing-blocking-script>
    pending_parsing_blocking_script: DomRefCell<Option<PendingScript>>,
    /// Number of stylesheets that block executing the next parser-inserted script
    script_blocking_stylesheets_count: Cell<u32>,
    /// <https://html.spec.whatwg.org/multipage/#list-of-scripts-that-will-execute-when-the-document-has-finished-parsing>
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
    /// The cached first `base` element with an `href` attribute.
    base_element: MutNullableDom<HTMLBaseElement>,
    /// This field is set to the document itself for inert documents.
    /// <https://html.spec.whatwg.org/multipage/#appropriate-template-contents-owner-document>
    appropriate_template_contents_owner_document: MutNullableDom<Document>,
    /// Information on elements needing restyle to ship over to layout when the
    /// time comes.
    pending_restyles: DomRefCell<HashMap<Dom<Element>, NoTrace<PendingRestyle>>>,
    /// This flag will be true if the `Document` needs to be painted again
    /// during the next full layout attempt due to some external change such as
    /// the web view changing size, or because the previous layout was only for
    /// layout queries (which do not trigger display).
    needs_paint: Cell<bool>,
    /// <http://w3c.github.io/touch-events/#dfn-active-touch-point>
    active_touch_points: DomRefCell<Vec<Dom<Touch>>>,
    /// Navigation Timing properties:
    /// <https://w3c.github.io/navigation-timing/#sec-PerformanceNavigationTiming>
    #[no_trace]
    dom_interactive: Cell<Option<CrossProcessInstant>>,
    #[no_trace]
    dom_content_loaded_event_start: Cell<Option<CrossProcessInstant>>,
    #[no_trace]
    dom_content_loaded_event_end: Cell<Option<CrossProcessInstant>>,
    #[no_trace]
    dom_complete: Cell<Option<CrossProcessInstant>>,
    #[no_trace]
    top_level_dom_complete: Cell<Option<CrossProcessInstant>>,
    #[no_trace]
    load_event_start: Cell<Option<CrossProcessInstant>>,
    #[no_trace]
    load_event_end: Cell<Option<CrossProcessInstant>>,
    #[no_trace]
    unload_event_start: Cell<Option<CrossProcessInstant>>,
    #[no_trace]
    unload_event_end: Cell<Option<CrossProcessInstant>>,
    /// <https://html.spec.whatwg.org/multipage/#concept-document-https-state>
    #[no_trace]
    https_state: Cell<HttpsState>,
    /// The document's origin.
    #[no_trace]
    origin: MutableOrigin,
    /// <https://html.spec.whatwg.org/multipage/#dom-document-referrer>
    referrer: Option<String>,
    /// <https://html.spec.whatwg.org/multipage/#target-element>
    target_element: MutNullableDom<Element>,
    /// <https://html.spec.whatwg.org/multipage/#concept-document-policy-container>
    #[no_trace]
    policy_container: DomRefCell<PolicyContainer>,
    /// <https://w3c.github.io/uievents/#event-type-dblclick>
    #[ignore_malloc_size_of = "Defined in std"]
    #[no_trace]
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
    /// This is sent to layout every time a reflow is done;
    /// layout uses this to determine if the gains from parallel layout will be worth the overhead.
    ///
    /// See also: <https://github.com/servo/servo/issues/10110>
    dom_count: Cell<u32>,
    /// Entry node for fullscreen.
    fullscreen_element: MutNullableDom<Element>,
    /// Map from ID to set of form control elements that have that ID as
    /// their 'form' content attribute. Used to reset form controls
    /// whenever any element with the same ID as the form attribute
    /// is inserted or removed from the document.
    /// See <https://html.spec.whatwg.org/multipage/#form-owner>
    form_id_listener_map: DomRefCell<HashMapTracedValues<Atom, HashSet<Dom<Element>>>>,
    #[no_trace]
    interactive_time: DomRefCell<InteractiveMetrics>,
    #[no_trace]
    tti_window: DomRefCell<InteractiveWindow>,
    /// RAII canceller for Fetch
    canceller: FetchCanceller,
    /// <https://html.spec.whatwg.org/multipage/#throw-on-dynamic-markup-insertion-counter>
    throw_on_dynamic_markup_insertion_counter: Cell<u64>,
    /// <https://html.spec.whatwg.org/multipage/#page-showing>
    page_showing: Cell<bool>,
    /// Whether the document is salvageable.
    salvageable: Cell<bool>,
    /// Whether the document was aborted with an active parser
    active_parser_was_aborted: Cell<bool>,
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
    /// <https://html.spec.whatwg.org/multipage/#completely-loaded>
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
    /// List of all WebGL context IDs that need flushing.
    dirty_webgl_contexts:
        DomRefCell<HashMapTracedValues<WebGLContextId, Dom<WebGLRenderingContext>>>,
    /// List of all WebGPU contexts.
    #[cfg(feature = "webgpu")]
    #[ignore_malloc_size_of = "Rc are hard"]
    webgpu_contexts: WebGPUContextsMap,
    /// <https://w3c.github.io/slection-api/#dfn-selection>
    selection: MutNullableDom<Selection>,
    /// A timeline for animations which is used for synchronizing animations.
    /// <https://drafts.csswg.org/web-animations/#timeline>
    animation_timeline: DomRefCell<AnimationTimeline>,
    /// Animations for this Document
    animations: DomRefCell<Animations>,
    /// The nearest inclusive ancestors to all the nodes that require a restyle.
    dirty_root: MutNullableDom<Element>,
    /// <https://html.spec.whatwg.org/multipage/#will-declaratively-refresh>
    declarative_refresh: DomRefCell<Option<DeclarativeRefresh>>,
    /// Pending input events, to be handled at the next rendering opportunity.
    #[no_trace]
    #[ignore_malloc_size_of = "CompositorEvent contains data from outside crates"]
    pending_input_events: DomRefCell<Vec<ConstellationInputEvent>>,
    /// The index of the last mouse move event in the pending compositor events queue.
    mouse_move_event_index: DomRefCell<Option<usize>>,
    /// Pending animation ticks, to be handled at the next rendering opportunity.
    #[no_trace]
    #[ignore_malloc_size_of = "AnimationTickType contains data from an outside crate"]
    pending_animation_ticks: DomRefCell<AnimationTickType>,
    /// <https://drafts.csswg.org/resize-observer/#dom-document-resizeobservers-slot>
    ///
    /// Note: we are storing, but never removing, resize observers.
    /// The lifetime of resize observers is specified at
    /// <https://drafts.csswg.org/resize-observer/#lifetime>.
    /// But implementing it comes with known problems:
    /// - <https://bugzilla.mozilla.org/show_bug.cgi?id=1596992>
    /// - <https://github.com/w3c/csswg-drafts/issues/4518>
    resize_observers: DomRefCell<Vec<Dom<ResizeObserver>>>,
    /// The set of all fonts loaded by this document.
    /// <https://drafts.csswg.org/css-font-loading/#font-face-source>
    fonts: MutNullableDom<FontFaceSet>,
    /// <https://html.spec.whatwg.org/multipage/#visibility-state>
    visibility_state: Cell<DocumentVisibilityState>,
    /// <https://www.iana.org/assignments/http-status-codes/http-status-codes.xhtml>
    status_code: Option<u16>,
    /// <https://html.spec.whatwg.org/multipage/#is-initial-about:blank>
    is_initial_about_blank: Cell<bool>,
    /// <https://w3c.github.io/webappsec-upgrade-insecure-requests/#insecure-requests-policy>
    #[no_trace]
    inherited_insecure_requests_policy: Cell<Option<InsecureRequestsPolicy>>,
    /// <https://w3c.github.io/IntersectionObserver/#document-intersectionobservertaskqueued>
    intersection_observer_task_queued: Cell<bool>,
}

#[allow(non_snake_case)]
impl Document {
    pub(crate) fn note_node_with_dirty_descendants(&self, node: &Node) {
        debug_assert!(*node.owner_doc() == *self);
        if !node.is_connected() {
            return;
        }

        let parent = match node.inclusive_ancestors(ShadowIncluding::Yes).nth(1) {
            Some(parent) => parent,
            None => {
                // There is no parent so this is the Document node, so we
                // behave as if we were called with the document element.
                let document_element = match self.GetDocumentElement() {
                    Some(element) => element,
                    None => return,
                };
                if let Some(dirty_root) = self.dirty_root.get() {
                    // There was an existing dirty root so we mark its
                    // ancestors as dirty until the document element.
                    for ancestor in dirty_root
                        .upcast::<Node>()
                        .inclusive_ancestors(ShadowIncluding::Yes)
                    {
                        if ancestor.is::<Element>() {
                            ancestor.set_flag(NodeFlags::HAS_DIRTY_DESCENDANTS, true);
                        }
                    }
                }
                self.dirty_root.set(Some(&document_element));
                return;
            },
        };

        if parent.is::<Element>() {
            if !parent.is_styled() {
                return;
            }

            if parent.is_display_none() {
                return;
            }
        }

        let element_parent: DomRoot<Element>;
        let element = match node.downcast::<Element>() {
            Some(element) => element,
            None => {
                // Current node is not an element, it's probably a text node,
                // we try to get its element parent.
                match DomRoot::downcast::<Element>(parent) {
                    Some(parent) => {
                        element_parent = parent;
                        &element_parent
                    },
                    None => {
                        // Parent is not an element so it must be a document,
                        // and this is not an element either, so there is
                        // nothing to do.
                        return;
                    },
                }
            },
        };

        let dirty_root = match self.dirty_root.get() {
            None => {
                element
                    .upcast::<Node>()
                    .set_flag(NodeFlags::HAS_DIRTY_DESCENDANTS, true);
                self.dirty_root.set(Some(element));
                return;
            },
            Some(root) => root,
        };

        for ancestor in element
            .upcast::<Node>()
            .inclusive_ancestors(ShadowIncluding::Yes)
        {
            if ancestor.get_flag(NodeFlags::HAS_DIRTY_DESCENDANTS) {
                return;
            }
            if ancestor.is::<Element>() {
                ancestor.set_flag(NodeFlags::HAS_DIRTY_DESCENDANTS, true);
            }
        }

        let new_dirty_root = element
            .upcast::<Node>()
            .common_ancestor(dirty_root.upcast(), ShadowIncluding::Yes)
            .expect("Couldn't find common ancestor");

        let mut has_dirty_descendants = true;
        for ancestor in dirty_root
            .upcast::<Node>()
            .inclusive_ancestors(ShadowIncluding::Yes)
        {
            ancestor.set_flag(NodeFlags::HAS_DIRTY_DESCENDANTS, has_dirty_descendants);
            has_dirty_descendants &= *ancestor != *new_dirty_root;
        }

        let maybe_shadow_host = new_dirty_root
            .downcast::<ShadowRoot>()
            .map(ShadowRootMethods::Host);
        let new_dirty_root_element = new_dirty_root
            .downcast::<Element>()
            .or(maybe_shadow_host.as_deref());

        self.dirty_root.set(new_dirty_root_element);
    }

    pub(crate) fn take_dirty_root(&self) -> Option<DomRoot<Element>> {
        self.dirty_root.take()
    }

    #[inline]
    pub(crate) fn loader(&self) -> Ref<DocumentLoader> {
        self.loader.borrow()
    }

    #[inline]
    pub(crate) fn loader_mut(&self) -> RefMut<DocumentLoader> {
        self.loader.borrow_mut()
    }

    #[inline]
    pub(crate) fn has_browsing_context(&self) -> bool {
        self.has_browsing_context
    }

    /// <https://html.spec.whatwg.org/multipage/#concept-document-bc>
    #[inline]
    pub(crate) fn browsing_context(&self) -> Option<DomRoot<WindowProxy>> {
        if self.has_browsing_context {
            self.window.undiscarded_window_proxy()
        } else {
            None
        }
    }

    pub(crate) fn webview_id(&self) -> WebViewId {
        self.window.webview_id()
    }

    #[inline]
    pub(crate) fn window(&self) -> &Window {
        &self.window
    }

    #[inline]
    pub(crate) fn is_html_document(&self) -> bool {
        self.is_html_document
    }

    pub(crate) fn is_xhtml_document(&self) -> bool {
        self.content_type.type_() == mime::APPLICATION &&
            self.content_type.subtype().as_str() == "xhtml" &&
            self.content_type.suffix() == Some(mime::XML)
    }

    pub(crate) fn set_https_state(&self, https_state: HttpsState) {
        self.https_state.set(https_state);
    }

    pub(crate) fn is_fully_active(&self) -> bool {
        self.activity.get() == DocumentActivity::FullyActive
    }

    pub(crate) fn is_active(&self) -> bool {
        self.activity.get() != DocumentActivity::Inactive
    }

    pub(crate) fn set_activity(&self, activity: DocumentActivity, can_gc: CanGc) {
        // This function should only be called on documents with a browsing context
        assert!(self.has_browsing_context);
        if activity == self.activity.get() {
            return;
        }

        // Set the document's activity level, reflow if necessary, and suspend or resume timers.
        self.activity.set(activity);
        let media = ServoMedia::get();
        let pipeline_id = self.window().pipeline_id();
        let client_context_id =
            ClientContextId::build(pipeline_id.namespace_id.0, pipeline_id.index.0.get());

        if activity != DocumentActivity::FullyActive {
            self.window().suspend(can_gc);
            media.suspend(&client_context_id);
            return;
        }

        self.title_changed();
        self.dirty_all_nodes();
        self.window().resume(can_gc);
        media.resume(&client_context_id);

        if self.ready_state.get() != DocumentReadyState::Complete {
            return;
        }

        // This step used to be Step 4.6 in html.spec.whatwg.org/multipage/#history-traversal
        // But it's now Step 4 in https://html.spec.whatwg.org/multipage/#reactivate-a-document
        // TODO: See #32687 for more information.
        let document = Trusted::new(self);
        self.owner_global()
            .task_manager()
            .dom_manipulation_task_source()
            .queue(task!(fire_pageshow_event: move || {
                let document = document.root();
                let window = document.window();
                // Step 4.6.1
                if document.page_showing.get() {
                    return;
                }
                // Step 4.6.2 Set document's page showing flag to true.
                document.page_showing.set(true);
                // Step 4.6.3 Update the visibility state of document to "visible".
                document.update_visibility_state(DocumentVisibilityState::Visible, CanGc::note());
                // Step 4.6.4 Fire a page transition event named pageshow at document's relevant
                // global object with true.
                let event = PageTransitionEvent::new(
                    window,
                    atom!("pageshow"),
                    false, // bubbles
                    false, // cancelable
                    true, // persisted
                    CanGc::note(),
                );
                let event = event.upcast::<Event>();
                event.set_trusted(true);
                window.dispatch_event_with_target_override(event, CanGc::note());
            }))
    }

    pub(crate) fn origin(&self) -> &MutableOrigin {
        &self.origin
    }

    /// <https://dom.spec.whatwg.org/#concept-document-url>
    pub(crate) fn url(&self) -> ServoUrl {
        self.url.borrow().clone()
    }

    pub(crate) fn set_url(&self, url: ServoUrl) {
        *self.url.borrow_mut() = url;
    }

    /// <https://html.spec.whatwg.org/multipage/#fallback-base-url>
    pub(crate) fn fallback_base_url(&self) -> ServoUrl {
        let document_url = self.url();
        if let Some(browsing_context) = self.browsing_context() {
            // Step 1: If document is an iframe srcdoc document, then return the
            // document base URL of document's browsing context's container document.
            let container_base_url = browsing_context
                .parent()
                .and_then(|parent| parent.document())
                .map(|document| document.base_url());
            if document_url.as_str() == "about:srcdoc" {
                if let Some(base_url) = container_base_url {
                    return base_url;
                }
            }
            // Step 2: If document's URL is about:blank, and document's browsing
            // context's creator base URL is non-null, then return that creator base URL.
            if document_url.as_str() == "about:blank" && browsing_context.has_creator_base_url() {
                return browsing_context.creator_base_url().unwrap();
            }
        }
        // Step 3: Return document's URL.
        document_url
    }

    /// <https://html.spec.whatwg.org/multipage/#document-base-url>
    pub(crate) fn base_url(&self) -> ServoUrl {
        match self.base_element() {
            // Step 1.
            None => self.fallback_base_url(),
            // Step 2.
            Some(base) => base.frozen_base_url(),
        }
    }

    pub(crate) fn set_needs_paint(&self, value: bool) {
        self.needs_paint.set(value)
    }

    pub(crate) fn needs_reflow(&self) -> Option<ReflowTriggerCondition> {
        // FIXME: This should check the dirty bit on the document,
        // not the document element. Needs some layout changes to make
        // that workable.
        if self.stylesheets.borrow().has_changed() {
            return Some(ReflowTriggerCondition::StylesheetsChanged);
        }

        let root = self.GetDocumentElement()?;
        if root.upcast::<Node>().has_dirty_descendants() {
            return Some(ReflowTriggerCondition::DirtyDescendants);
        }

        if !self.pending_restyles.borrow().is_empty() {
            return Some(ReflowTriggerCondition::PendingRestyles);
        }

        if self.needs_paint.get() {
            return Some(ReflowTriggerCondition::PaintPostponed);
        }

        None
    }

    /// Returns the first `base` element in the DOM that has an `href` attribute.
    pub(crate) fn base_element(&self) -> Option<DomRoot<HTMLBaseElement>> {
        self.base_element.get()
    }

    /// Refresh the cached first base element in the DOM.
    /// <https://github.com/w3c/web-platform-tests/issues/2122>
    pub(crate) fn refresh_base_element(&self) {
        let base = self
            .upcast::<Node>()
            .traverse_preorder(ShadowIncluding::No)
            .filter_map(DomRoot::downcast::<HTMLBaseElement>)
            .find(|element| {
                element
                    .upcast::<Element>()
                    .has_attribute(&local_name!("href"))
            });
        self.base_element.set(base.as_deref());
    }

    pub(crate) fn dom_count(&self) -> u32 {
        self.dom_count.get()
    }

    /// This is called by `bind_to_tree` when a node is added to the DOM.
    /// The internal count is used by layout to determine whether to be sequential or parallel.
    /// (it's sequential for small DOMs)
    pub(crate) fn increment_dom_count(&self) {
        self.dom_count.set(self.dom_count.get() + 1);
    }

    /// This is called by `unbind_from_tree` when a node is removed from the DOM.
    pub(crate) fn decrement_dom_count(&self) {
        self.dom_count.set(self.dom_count.get() - 1);
    }

    pub(crate) fn quirks_mode(&self) -> QuirksMode {
        self.quirks_mode.get()
    }

    pub(crate) fn set_quirks_mode(&self, new_mode: QuirksMode) {
        let old_mode = self.quirks_mode.replace(new_mode);

        if old_mode != new_mode {
            self.window.layout_mut().set_quirks_mode(new_mode);
        }
    }

    pub(crate) fn encoding(&self) -> &'static Encoding {
        self.encoding.get()
    }

    pub(crate) fn set_encoding(&self, encoding: &'static Encoding) {
        self.encoding.set(encoding);
    }

    pub(crate) fn content_and_heritage_changed(&self, node: &Node) {
        if node.is_connected() {
            node.note_dirty_descendants();
        }

        // FIXME(emilio): This is very inefficient, ideally the flag above would
        // be enough and incremental layout could figure out from there.
        node.dirty(NodeDamage::OtherNodeDamage);
    }

    /// Remove any existing association between the provided id and any elements in this document.
    pub(crate) fn unregister_element_id(&self, to_unregister: &Element, id: Atom) {
        self.document_or_shadow_root
            .unregister_named_element(&self.id_map, to_unregister, &id);
        self.reset_form_owner_for_listeners(&id);
    }

    /// Associate an element present in this document with the provided id.
    pub(crate) fn register_element_id(&self, element: &Element, id: Atom) {
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

    /// Remove any existing association between the provided name and any elements in this document.
    pub(crate) fn unregister_element_name(&self, to_unregister: &Element, name: Atom) {
        self.document_or_shadow_root
            .unregister_named_element(&self.name_map, to_unregister, &name);
    }

    /// Associate an element present in this document with the provided name.
    pub(crate) fn register_element_name(&self, element: &Element, name: Atom) {
        let root = self.GetDocumentElement().expect(
            "The element is in the document, so there must be a document \
             element.",
        );
        self.document_or_shadow_root.register_named_element(
            &self.name_map,
            element,
            &name,
            DomRoot::from_ref(root.upcast::<Node>()),
        );
    }

    pub(crate) fn register_form_id_listener<T: ?Sized + FormControl>(
        &self,
        id: DOMString,
        listener: &T,
    ) {
        let mut map = self.form_id_listener_map.borrow_mut();
        let listener = listener.to_element();
        let set = map.entry(Atom::from(id)).or_default();
        set.insert(Dom::from_ref(listener));
    }

    pub(crate) fn unregister_form_id_listener<T: ?Sized + FormControl>(
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
    pub(crate) fn find_fragment_node(&self, fragid: &str) -> Option<DomRoot<Element>> {
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
    pub(crate) fn check_and_scroll_fragment(&self, fragment: &str, can_gc: CanGc) {
        let target = self.find_fragment_node(fragment);

        // Step 1
        self.set_target_element(target.as_deref());

        let point = target
            .as_ref()
            .map(|element| {
                // TODO: This strategy is completely wrong if the element we are scrolling to in
                // inside other scrollable containers. Ideally this should use an implementation of
                // `scrollIntoView` when that is available:
                // See https://github.com/servo/servo/issues/24059.
                let rect = element
                    .upcast::<Node>()
                    .bounding_content_box_or_zero(can_gc);

                // In order to align with element edges, we snap to unscaled pixel boundaries, since
                // the paint thread currently does the same for drawing elements. This is important
                // for pages that require pixel perfect scroll positioning for proper display
                // (like Acid2).
                let device_pixel_ratio = self.window.device_pixel_ratio().get();
                (
                    rect.origin.x.to_nearest_pixel(device_pixel_ratio),
                    rect.origin.y.to_nearest_pixel(device_pixel_ratio),
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
            self.window
                .scroll(x as f64, y as f64, ScrollBehavior::Instant, can_gc)
        }
    }

    fn get_anchor_by_name(&self, name: &str) -> Option<DomRoot<Element>> {
        let name = Atom::from(name);
        self.name_map.borrow().get(&name).and_then(|elements| {
            elements
                .iter()
                .find(|e| e.is::<HTMLAnchorElement>())
                .map(|e| DomRoot::from_ref(&**e))
        })
    }

    // https://html.spec.whatwg.org/multipage/#current-document-readiness
    pub(crate) fn set_ready_state(&self, state: DocumentReadyState, can_gc: CanGc) {
        match state {
            DocumentReadyState::Loading => {
                if self.window().is_top_level() {
                    self.send_to_embedder(EmbedderMsg::NotifyLoadStatusChanged(
                        self.webview_id(),
                        LoadStatus::Started,
                    ));
                    self.send_to_embedder(EmbedderMsg::Status(self.webview_id(), None));
                }
            },
            DocumentReadyState::Complete => {
                if self.window().is_top_level() {
                    self.send_to_embedder(EmbedderMsg::NotifyLoadStatusChanged(
                        self.webview_id(),
                        LoadStatus::Complete,
                    ));
                }
                update_with_current_instant(&self.dom_complete);
            },
            DocumentReadyState::Interactive => update_with_current_instant(&self.dom_interactive),
        };

        self.ready_state.set(state);

        self.upcast::<EventTarget>()
            .fire_event(atom!("readystatechange"), can_gc);
    }

    /// Return whether scripting is enabled or not
    pub(crate) fn is_scripting_enabled(&self) -> bool {
        self.scripting_enabled
    }

    /// Return the element that currently has focus.
    // https://w3c.github.io/uievents/#events-focusevent-doc-focus
    pub(crate) fn get_focused_element(&self) -> Option<DomRoot<Element>> {
        self.focused.get()
    }

    /// Initiate a new round of checking for elements requesting focus. The last element to call
    /// `request_focus` before `commit_focus_transaction` is called will receive focus.
    fn begin_focus_transaction(&self) {
        *self.focus_transaction.borrow_mut() = FocusTransaction::InTransaction(Default::default());
    }

    /// <https://html.spec.whatwg.org/multipage/#focus-fixup-rule>
    pub(crate) fn perform_focus_fixup_rule(&self, not_focusable: &Element, can_gc: CanGc) {
        if Some(not_focusable) != self.focused.get().as_deref() {
            return;
        }
        self.request_focus(
            self.GetBody().as_ref().map(|e| e.upcast()),
            FocusType::Element,
            can_gc,
        )
    }

    /// Request that the given element receive focus once the current transaction is complete.
    /// If None is passed, then whatever element is currently focused will no longer be focused
    /// once the transaction is complete.
    pub(crate) fn request_focus(
        &self,
        elem: Option<&Element>,
        focus_type: FocusType,
        can_gc: CanGc,
    ) {
        let implicit_transaction = matches!(
            *self.focus_transaction.borrow(),
            FocusTransaction::NotInTransaction
        );
        if implicit_transaction {
            self.begin_focus_transaction();
        }
        if elem.map_or(true, |e| e.is_focusable_area()) {
            *self.focus_transaction.borrow_mut() =
                FocusTransaction::InTransaction(elem.map(Dom::from_ref));
        }
        if implicit_transaction {
            self.commit_focus_transaction(focus_type, can_gc);
        }
    }

    /// Reassign the focus context to the element that last requested focus during this
    /// transaction, or none if no elements requested it.
    fn commit_focus_transaction(&self, focus_type: FocusType, can_gc: CanGc) {
        let possibly_focused = match *self.focus_transaction.borrow() {
            FocusTransaction::NotInTransaction => unreachable!(),
            FocusTransaction::InTransaction(ref elem) => {
                elem.as_ref().map(|e| DomRoot::from_ref(&**e))
            },
        };
        *self.focus_transaction.borrow_mut() = FocusTransaction::NotInTransaction;
        if self.focused == possibly_focused.as_deref() {
            return;
        }
        if let Some(ref elem) = self.focused.get() {
            let node = elem.upcast::<Node>();
            elem.set_focus_state(false);
            // FIXME: pass appropriate relatedTarget
            self.fire_focus_event(FocusEventType::Blur, node, None, can_gc);

            // Notify the embedder to hide the input method.
            if elem.input_method_type().is_some() {
                self.send_to_embedder(EmbedderMsg::HideIME(self.webview_id()));
            }
        }

        self.focused.set(possibly_focused.as_deref());

        if let Some(ref elem) = self.focused.get() {
            elem.set_focus_state(true);
            let node = elem.upcast::<Node>();
            // FIXME: pass appropriate relatedTarget
            self.fire_focus_event(FocusEventType::Focus, node, None, can_gc);
            // Update the focus state for all elements in the focus chain.
            // https://html.spec.whatwg.org/multipage/#focus-chain
            if focus_type == FocusType::Element {
                self.window().send_to_constellation(ScriptMsg::Focus);
            }

            // Notify the embedder to display an input method.
            if let Some(kind) = elem.input_method_type() {
                let rect = elem.upcast::<Node>().bounding_content_box_or_zero(can_gc);
                let rect = Rect::new(
                    Point2D::new(rect.origin.x.to_px(), rect.origin.y.to_px()),
                    Size2D::new(rect.size.width.to_px(), rect.size.height.to_px()),
                );
                let (text, multiline) = if let Some(input) = elem.downcast::<HTMLInputElement>() {
                    (
                        Some((
                            input.Value().to_string(),
                            input.GetSelectionEnd().unwrap_or(0) as i32,
                        )),
                        false,
                    )
                } else if let Some(textarea) = elem.downcast::<HTMLTextAreaElement>() {
                    (
                        Some((
                            textarea.Value().to_string(),
                            textarea.GetSelectionEnd().unwrap_or(0) as i32,
                        )),
                        true,
                    )
                } else {
                    (None, false)
                };
                self.send_to_embedder(EmbedderMsg::ShowIME(
                    self.webview_id(),
                    kind,
                    text,
                    multiline,
                    DeviceIntRect::from_untyped(&rect.to_box2d()),
                ));
            }
        }
    }

    /// Handles any updates when the document's title has changed.
    pub(crate) fn title_changed(&self) {
        if self.browsing_context().is_some() {
            self.send_title_to_embedder();
            let title = String::from(self.Title());
            self.window.send_to_constellation(ScriptMsg::TitleChanged(
                self.window.pipeline_id(),
                title.clone(),
            ));
            if let Some(chan) = self.window.as_global_scope().devtools_chan() {
                let _ = chan.send(ScriptToDevtoolsControlMsg::TitleChanged(
                    self.window.pipeline_id(),
                    title,
                ));
            }
        }
    }

    /// Determine the title of the [`Document`] according to the specification at:
    /// <https://html.spec.whatwg.org/multipage/#document.title>. The difference
    /// here is that when the title isn't specified `None` is returned.
    fn title(&self) -> Option<DOMString> {
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

        title.map(|title| {
            // Steps 3-4.
            let value = title.child_text_content();
            DOMString::from(str_join(split_html_space_chars(&value), " "))
        })
    }

    /// Sends this document's title to the constellation.
    pub(crate) fn send_title_to_embedder(&self) {
        let window = self.window();
        if window.is_top_level() {
            let title = self.title().map(String::from);
            self.send_to_embedder(EmbedderMsg::ChangePageTitle(self.webview_id(), title));
        }
    }

    fn send_to_embedder(&self, msg: EmbedderMsg) {
        let window = self.window();
        window.send_to_embedder(msg);
    }

    pub(crate) fn dirty_all_nodes(&self) {
        let root = match self.GetDocumentElement() {
            Some(root) => root,
            None => return,
        };
        for node in root
            .upcast::<Node>()
            .traverse_preorder(ShadowIncluding::Yes)
        {
            node.dirty(NodeDamage::OtherNodeDamage)
        }
    }

    #[allow(unsafe_code)]
    pub(crate) fn handle_mouse_button_event(
        &self,
        event: MouseButtonEvent,
        hit_test_result: Option<CompositorHitTestResult>,
        pressed_mouse_buttons: u16,
        can_gc: CanGc,
    ) {
        // Ignore all incoming events without a hit test.
        let Some(hit_test_result) = hit_test_result else {
            return;
        };

        let mouse_event_type_string = match event.action {
            MouseButtonAction::Click => "click".to_owned(),
            MouseButtonAction::Up => "mouseup".to_owned(),
            MouseButtonAction::Down => "mousedown".to_owned(),
        };
        debug!(
            "{}: at {:?}",
            mouse_event_type_string, hit_test_result.point_in_viewport
        );

        let node = unsafe { node::from_untrusted_compositor_node_address(hit_test_result.node) };
        let Some(el) = node
            .inclusive_ancestors(ShadowIncluding::No)
            .filter_map(DomRoot::downcast::<Element>)
            .next()
        else {
            return;
        };

        let node = el.upcast::<Node>();
        debug!("{} on {:?}", mouse_event_type_string, node.debug_str());
        // Prevent click event if form control element is disabled.
        if let MouseButtonAction::Click = event.action {
            // The click event is filtered by the disabled state.
            if el.is_actually_disabled() {
                return;
            }

            self.begin_focus_transaction();
            self.request_focus(Some(&*el), FocusType::Element, can_gc);
        }

        // https://w3c.github.io/uievents/#event-type-click
        let client_x = hit_test_result.point_in_viewport.x as i32;
        let client_y = hit_test_result.point_in_viewport.y as i32;
        let click_count = 1;
        let dom_event = MouseEvent::new(
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
            event.button.into(),
            pressed_mouse_buttons,
            None,
            Some(hit_test_result.point_relative_to_item),
            can_gc,
        );
        let dom_event = dom_event.upcast::<Event>();

        // https://w3c.github.io/uievents/#trusted-events
        dom_event.set_trusted(true);
        // https://html.spec.whatwg.org/multipage/#run-authentic-click-activation-steps
        let activatable = el.as_maybe_activatable();
        match event.action {
            MouseButtonAction::Click => {
                el.set_click_in_progress(true);
                dom_event.fire(node.upcast(), can_gc);
                el.set_click_in_progress(false);
            },
            MouseButtonAction::Down => {
                if let Some(a) = activatable {
                    a.enter_formal_activation_state();
                }

                let target = node.upcast();
                dom_event.fire(target, can_gc);
            },
            MouseButtonAction::Up => {
                if let Some(a) = activatable {
                    a.exit_formal_activation_state();
                }

                let target = node.upcast();
                dom_event.fire(target, can_gc);
            },
        }

        if let MouseButtonAction::Click = event.action {
            self.commit_focus_transaction(FocusType::Element, can_gc);
            self.maybe_fire_dblclick(
                hit_test_result.point_in_viewport,
                node,
                pressed_mouse_buttons,
                can_gc,
            );
        }

        // When the contextmenu event is triggered by right mouse button
        // the contextmenu event MUST be dispatched after the mousedown event.
        if let (MouseButtonAction::Down, MouseButton::Right) = (event.action, event.button) {
            self.maybe_show_context_menu(
                node.upcast(),
                pressed_mouse_buttons,
                hit_test_result.point_in_viewport,
                can_gc,
            );
        }
    }

    /// <https://www.w3.org/TR/uievents/#maybe-show-context-menu>
    fn maybe_show_context_menu(
        &self,
        target: &EventTarget,
        pressed_mouse_buttons: u16,
        client_point: Point2D<f32>,
        can_gc: CanGc,
    ) {
        let client_x = client_point.x.to_i32().unwrap_or(0);
        let client_y = client_point.y.to_i32().unwrap_or(0);

        // <https://w3c.github.io/uievents/#contextmenu>
        let menu_event = PointerEvent::new(
            &self.window,                   // window
            None,                           // proto
            DOMString::from("contextmenu"), // type
            EventBubbles::Bubbles,          // can_bubble
            EventCancelable::Cancelable,    // cancelable
            Some(&self.window),             // view
            0,                              // detail
            client_x,                       // screen_x
            client_y,                       // screen_y
            client_x,                       // client_x
            client_y,                       // client_y
            false,                          // ctrl_key
            false,                          // alt_key
            false,                          // shift_key
            false,                          // meta_key
            2i16,                           // button, right mouse button
            pressed_mouse_buttons,          // buttons
            None,                           // related_target
            None,                           // point_in_target
            // TODO: decide generic pointer id
            // <https://www.w3.org/TR/pointerevents3/#dom-pointerevent-pointerid>
            0,                        // pointer_id
            1,                        // width
            1,                        // height
            0.5,                      // pressure
            0.0,                      // tangential_pressure
            0,                        // tilt_x
            0,                        // tilt_y
            0,                        // twist
            PI / 2.0,                 // altitude_angle
            0.0,                      // azimuth_angle
            DOMString::from("mouse"), // pointer_type
            true,                     // is_primary
            vec![],                   // coalesced_events
            vec![],                   // predicted_events
            can_gc,
        );
        let event = menu_event.upcast::<Event>();
        event.fire(target, can_gc);

        // if the event was not canceled, notify the embedder to show the context menu
        if event.status() == EventStatus::NotCanceled {
            let (sender, receiver) =
                ipc::channel::<ContextMenuResult>().expect("Failed to create IPC channel.");
            self.send_to_embedder(EmbedderMsg::ShowContextMenu(
                self.webview_id(),
                sender,
                None,
                vec![],
            ));
            let _ = receiver.recv().unwrap();
        };
    }

    fn maybe_fire_dblclick(
        &self,
        click_pos: Point2D<f32>,
        target: &Node,
        pressed_mouse_buttons: u16,
        can_gc: CanGc,
    ) {
        // https://w3c.github.io/uievents/#event-type-dblclick
        let now = Instant::now();

        let opt = self.last_click_info.borrow_mut().take();

        if let Some((last_time, last_pos)) = opt {
            let DBL_CLICK_TIMEOUT =
                Duration::from_millis(pref!(dom_document_dblclick_timeout) as u64);
            let DBL_CLICK_DIST_THRESHOLD = pref!(dom_document_dblclick_dist) as u64;

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
                    pressed_mouse_buttons,
                    None,
                    None,
                    can_gc,
                );
                event.upcast::<Event>().fire(target.upcast(), can_gc);

                // When a double click occurs, self.last_click_info is left as None so that a
                // third sequential click will not cause another double click.
                return;
            }
        }

        // Update last_click_info with the time and position of the click.
        *self.last_click_info.borrow_mut() = Some((now, click_pos));
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn fire_mouse_event(
        &self,
        client_point: Point2D<f32>,
        target: &EventTarget,
        event_name: FireMouseEventType,
        can_bubble: EventBubbles,
        cancelable: EventCancelable,
        pressed_mouse_buttons: u16,
        can_gc: CanGc,
    ) {
        let client_x = client_point.x.to_i32().unwrap_or(0);
        let client_y = client_point.y.to_i32().unwrap_or(0);

        MouseEvent::new(
            &self.window,
            DOMString::from(event_name.as_str()),
            can_bubble,
            cancelable,
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
            pressed_mouse_buttons,
            None,
            None,
            can_gc,
        )
        .upcast::<Event>()
        .fire(target, can_gc);
    }

    pub(crate) fn handle_editing_action(&self, action: EditingActionEvent, can_gc: CanGc) -> bool {
        let clipboard_event = match action {
            EditingActionEvent::Copy => ClipboardEventType::Copy,
            EditingActionEvent::Cut => ClipboardEventType::Cut,
            EditingActionEvent::Paste => ClipboardEventType::Paste,
        };
        self.handle_clipboard_action(clipboard_event, can_gc)
    }

    /// <https://www.w3.org/TR/clipboard-apis/#clipboard-actions>
    fn handle_clipboard_action(&self, action: ClipboardEventType, can_gc: CanGc) -> bool {
        // The script_triggered flag is set if the action runs because of a script, e.g. document.execCommand()
        let script_triggered = false;

        // The script_may_access_clipboard flag is set
        // if action is paste and the script thread is allowed to read from clipboard or
        // if action is copy or cut and the script thread is allowed to modify the clipboard
        let script_may_access_clipboard = false;

        // Step 1 If the script-triggered flag is set and the script-may-access-clipboard flag is unset
        if script_triggered && !script_may_access_clipboard {
            return false;
        }

        // Step 2 Fire a clipboard event
        let event = ClipboardEvent::new(
            &self.window,
            None,
            DOMString::from(action.as_str()),
            EventBubbles::Bubbles,
            EventCancelable::Cancelable,
            None,
            can_gc,
        );
        self.fire_clipboard_event(&event, action, can_gc);

        // Step 3 If a script doesn't call preventDefault()
        // the event will be handled inside target's VirtualMethods::handle_event

        let e = event.upcast::<Event>();

        if !e.IsTrusted() {
            return false;
        }

        // Step 4 If the event was canceled, then
        if e.DefaultPrevented() {
            match e.Type().str() {
                "copy" => {
                    // Step 4.1 Call the write content to the clipboard algorithm,
                    // passing on the DataTransferItemList items, a clear-was-called flag and a types-to-clear list.
                    if let Some(clipboard_data) = event.get_clipboard_data() {
                        let drag_data_store =
                            clipboard_data.data_store().expect("This shouldn't fail");
                        self.write_content_to_the_clipboard(&drag_data_store);
                    }
                },
                "cut" => {
                    // Step 4.1 Call the write content to the clipboard algorithm,
                    // passing on the DataTransferItemList items, a clear-was-called flag and a types-to-clear list.
                    if let Some(clipboard_data) = event.get_clipboard_data() {
                        let drag_data_store =
                            clipboard_data.data_store().expect("This shouldn't fail");
                        self.write_content_to_the_clipboard(&drag_data_store);
                    }

                    // Step 4.2 Fire a clipboard event named clipboardchange
                    self.fire_clipboardchange_event(can_gc);
                },
                "paste" => return false,
                _ => (),
            }
        }
        //Step 5
        true
    }

    /// <https://www.w3.org/TR/clipboard-apis/#fire-a-clipboard-event>
    fn fire_clipboard_event(
        &self,
        event: &ClipboardEvent,
        action: ClipboardEventType,
        can_gc: CanGc,
    ) {
        // Step 1 Let clear_was_called be false
        // Step 2 Let types_to_clear an empty list
        let mut drag_data_store = DragDataStore::new();

        // Step 4 let clipboard-entry be the sequence number of clipboard content, null if the OS doesn't support it.

        // Step 5 let trusted be true if the event is generated by the user agent, false otherwise
        let trusted = true;

        // Step 6 if the context is editable:
        let focused = self.get_focused_element();
        let body = self.GetBody();

        let target = match (&focused, &body) {
            (Some(focused), _) => focused.upcast(),
            (&None, Some(body)) => body.upcast(),
            (&None, &None) => self.window.upcast(),
        };
        // Step 6.2 else TODO require Selection see https://github.com/w3c/clipboard-apis/issues/70

        // Step 7
        match action {
            ClipboardEventType::Copy | ClipboardEventType::Cut => {
                // Step 7.2.1
                drag_data_store.set_mode(Mode::ReadWrite);
            },
            ClipboardEventType::Paste => {
                let (sender, receiver) = ipc::channel().unwrap();
                self.window
                    .send_to_constellation(ScriptMsg::ForwardToEmbedder(
                        EmbedderMsg::GetClipboardText(self.window.webview_id(), sender),
                    ));
                let text_contents = receiver
                    .recv()
                    .map(Result::unwrap_or_default)
                    .unwrap_or_default();

                // Step 7.1.1
                drag_data_store.set_mode(Mode::ReadOnly);
                // Step 7.1.2 If trusted or the implementation gives script-generated events access to the clipboard
                if trusted {
                    // Step 7.1.2.1 For each clipboard-part on the OS clipboard:

                    // Step 7.1.2.1.1 If clipboard-part contains plain text, then
                    let data = DOMString::from(text_contents.to_string());
                    let type_ = DOMString::from("text/plain");
                    let _ = drag_data_store.add(Kind::Text { data, type_ });

                    // Step 7.1.2.1.2 TODO If clipboard-part represents file references, then for each file reference
                    // Step 7.1.2.1.3 TODO If clipboard-part contains HTML- or XHTML-formatted text then

                    // Step 7.1.3 Update clipboard-event-data’s files to match clipboard-event-data’s items
                    // Step 7.1.4 Update clipboard-event-data’s types to match clipboard-event-data’s items
                }
            },
            ClipboardEventType::Change => (),
        }

        // Step 3
        let clipboard_event_data = DataTransfer::new(
            &self.window,
            Rc::new(RefCell::new(Some(drag_data_store))),
            can_gc,
        );

        // Step 8
        event.set_clipboard_data(Some(&clipboard_event_data));
        let event = event.upcast::<Event>();
        // Step 9
        event.set_trusted(trusted);
        // Step 10 Set event’s composed to true.
        event.set_composed(true);
        // Step 11
        event.dispatch(target, false, can_gc);
    }

    pub(crate) fn fire_clipboardchange_event(&self, can_gc: CanGc) {
        let clipboardchange_event = ClipboardEvent::new(
            &self.window,
            None,
            DOMString::from("clipboardchange"),
            EventBubbles::Bubbles,
            EventCancelable::Cancelable,
            None,
            can_gc,
        );
        self.fire_clipboard_event(&clipboardchange_event, ClipboardEventType::Change, can_gc);
    }

    /// <https://www.w3.org/TR/clipboard-apis/#write-content-to-the-clipboard>
    fn write_content_to_the_clipboard(&self, drag_data_store: &DragDataStore) {
        // Step 1
        if drag_data_store.list_len() > 0 {
            // Step 1.1 Clear the clipboard.
            self.send_to_embedder(EmbedderMsg::ClearClipboard(self.webview_id()));
            // Step 1.2
            for item in drag_data_store.iter_item_list() {
                match item {
                    Kind::Text { data, .. } => {
                        // Step 1.2.1.1 Ensure encoding is correct per OS and locale conventions
                        // Step 1.2.1.2 Normalize line endings according to platform conventions
                        // Step 1.2.1.3
                        self.send_to_embedder(EmbedderMsg::SetClipboardText(
                            self.webview_id(),
                            data.to_string(),
                        ));
                    },
                    Kind::File { .. } => {
                        // Step 1.2.2 If data is of a type listed in the mandatory data types list, then
                        // Step 1.2.2.1 Place part on clipboard with the appropriate OS clipboard format description
                        // Step 1.2.3 Else this is left to the implementation
                    },
                }
            }
        } else {
            // Step 2.1
            if drag_data_store.clear_was_called {
                // Step 2.1.1 If types-to-clear list is empty, clear the clipboard
                self.send_to_embedder(EmbedderMsg::ClearClipboard(self.webview_id()));
                // Step 2.1.2 Else remove the types in the list from the clipboard
                // As of now this can't be done with Arboard, and it's possible that will be removed from the spec
            }
        }
    }

    #[allow(unsafe_code)]
    pub(crate) unsafe fn handle_mouse_move_event(
        &self,
        hit_test_result: Option<CompositorHitTestResult>,
        pressed_mouse_buttons: u16,
        prev_mouse_over_target: &MutNullableDom<Element>,
        can_gc: CanGc,
    ) {
        // Ignore all incoming events without a hit test.
        let Some(hit_test_result) = hit_test_result else {
            return;
        };

        let node = unsafe { node::from_untrusted_compositor_node_address(hit_test_result.node) };
        let Some(new_target) = node
            .inclusive_ancestors(ShadowIncluding::No)
            .filter_map(DomRoot::downcast::<Element>)
            .next()
        else {
            return;
        };

        let target_has_changed = prev_mouse_over_target
            .get()
            .as_ref()
            .map_or(true, |old_target| old_target != &new_target);

        // Here we know the target has changed, so we must update the state,
        // dispatch mouseout to the previous one, mouseover to the new one.
        if target_has_changed {
            // Dispatch mouseout and mouseleave to previous target.
            if let Some(old_target) = prev_mouse_over_target.get() {
                let old_target_is_ancestor_of_new_target = old_target
                    .upcast::<Node>()
                    .is_ancestor_of(new_target.upcast::<Node>());

                // If the old target is an ancestor of the new target, this can be skipped
                // completely, since the node's hover state will be reset below.
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

                self.fire_mouse_event(
                    hit_test_result.point_in_viewport,
                    old_target.upcast(),
                    FireMouseEventType::Out,
                    EventBubbles::Bubbles,
                    EventCancelable::Cancelable,
                    pressed_mouse_buttons,
                    can_gc,
                );

                if !old_target_is_ancestor_of_new_target {
                    let event_target = DomRoot::from_ref(old_target.upcast::<Node>());
                    let moving_into = Some(DomRoot::from_ref(new_target.upcast::<Node>()));
                    self.handle_mouse_enter_leave_event(
                        hit_test_result.point_in_viewport,
                        FireMouseEventType::Leave,
                        moving_into,
                        event_target,
                        pressed_mouse_buttons,
                        can_gc,
                    );
                }
            }

            // Dispatch mouseover and mouseenter to new target.
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

            self.fire_mouse_event(
                hit_test_result.point_in_viewport,
                new_target.upcast(),
                FireMouseEventType::Over,
                EventBubbles::Bubbles,
                EventCancelable::Cancelable,
                pressed_mouse_buttons,
                can_gc,
            );

            let moving_from = prev_mouse_over_target
                .get()
                .map(|old_target| DomRoot::from_ref(old_target.upcast::<Node>()));
            let event_target = DomRoot::from_ref(new_target.upcast::<Node>());
            self.handle_mouse_enter_leave_event(
                hit_test_result.point_in_viewport,
                FireMouseEventType::Enter,
                moving_from,
                event_target,
                pressed_mouse_buttons,
                can_gc,
            );
        }

        // Send mousemove event to topmost target, unless it's an iframe, in which case the
        // compositor should have also sent an event to the inner document.
        self.fire_mouse_event(
            hit_test_result.point_in_viewport,
            new_target.upcast(),
            FireMouseEventType::Move,
            EventBubbles::Bubbles,
            EventCancelable::Cancelable,
            pressed_mouse_buttons,
            can_gc,
        );

        // If the target has changed then store the current mouse over target for next frame.
        if target_has_changed {
            prev_mouse_over_target.set(Some(&new_target));
        }
    }

    fn handle_mouse_enter_leave_event(
        &self,
        client_point: Point2D<f32>,
        event_type: FireMouseEventType,
        related_target: Option<DomRoot<Node>>,
        event_target: DomRoot<Node>,
        pressed_mouse_buttons: u16,
        can_gc: CanGc,
    ) {
        assert!(matches!(
            event_type,
            FireMouseEventType::Enter | FireMouseEventType::Leave
        ));

        let common_ancestor = match related_target.as_ref() {
            Some(related_target) => event_target
                .common_ancestor(related_target, ShadowIncluding::No)
                .unwrap_or_else(|| DomRoot::from_ref(&*event_target)),
            None => DomRoot::from_ref(&*event_target),
        };

        // We need to create a target chain in case the event target shares
        // its boundaries with its ancestors.
        let mut targets = vec![];
        let mut current = Some(event_target);
        while let Some(node) = current {
            if node == common_ancestor {
                break;
            }
            current = node.GetParentNode();
            targets.push(node);
        }

        // The order for dispatching mouseenter events starts from the topmost
        // common ancestor of the event target and the related target.
        if event_type == FireMouseEventType::Enter {
            targets = targets.into_iter().rev().collect();
        }

        for target in targets {
            self.fire_mouse_event(
                client_point,
                target.upcast(),
                event_type,
                EventBubbles::DoesNotBubble,
                EventCancelable::NotCancelable,
                pressed_mouse_buttons,
                can_gc,
            );
        }
    }

    #[allow(unsafe_code)]
    pub(crate) fn handle_wheel_event(
        &self,
        event: WheelEvent,
        hit_test_result: Option<CompositorHitTestResult>,
        can_gc: CanGc,
    ) {
        // Ignore all incoming events without a hit test.
        let Some(hit_test_result) = hit_test_result else {
            return;
        };

        let node = unsafe { node::from_untrusted_compositor_node_address(hit_test_result.node) };
        let Some(el) = node
            .inclusive_ancestors(ShadowIncluding::No)
            .filter_map(DomRoot::downcast::<Element>)
            .next()
        else {
            return;
        };

        let node = el.upcast::<Node>();
        let wheel_event_type_string = "wheel".to_owned();
        debug!(
            "{}: on {:?} at {:?}",
            wheel_event_type_string,
            node.debug_str(),
            hit_test_result.point_in_viewport
        );

        // https://w3c.github.io/uievents/#event-wheelevents
        let dom_event = DomWheelEvent::new(
            &self.window,
            DOMString::from(wheel_event_type_string),
            EventBubbles::Bubbles,
            EventCancelable::Cancelable,
            Some(&self.window),
            0i32,
            Finite::wrap(event.delta.x),
            Finite::wrap(event.delta.y),
            Finite::wrap(event.delta.z),
            event.delta.mode as u32,
            can_gc,
        );

        let dom_event = dom_event.upcast::<Event>();
        dom_event.set_trusted(true);

        let target = node.upcast();
        dom_event.fire(target, can_gc);
    }

    #[allow(unsafe_code)]
    pub(crate) fn handle_touch_event(
        &self,
        event: TouchEvent,
        hit_test_result: Option<CompositorHitTestResult>,
        can_gc: CanGc,
    ) -> TouchEventResult {
        // Ignore all incoming events without a hit test.
        let Some(hit_test_result) = hit_test_result else {
            return TouchEventResult::Forwarded;
        };

        let TouchId(identifier) = event.id;
        let event_name = match event.event_type {
            TouchEventType::Down => "touchstart",
            TouchEventType::Move => "touchmove",
            TouchEventType::Up => "touchend",
            TouchEventType::Cancel => "touchcancel",
        };

        let node = unsafe { node::from_untrusted_compositor_node_address(hit_test_result.node) };
        let Some(el) = node
            .inclusive_ancestors(ShadowIncluding::No)
            .filter_map(DomRoot::downcast::<Element>)
            .next()
        else {
            return TouchEventResult::Forwarded;
        };

        let target = DomRoot::upcast::<EventTarget>(el);
        let window = &*self.window;

        let client_x = Finite::wrap(hit_test_result.point_in_viewport.x as f64);
        let client_y = Finite::wrap(hit_test_result.point_in_viewport.y as f64);
        let page_x =
            Finite::wrap(hit_test_result.point_in_viewport.x as f64 + window.PageXOffset() as f64);
        let page_y =
            Finite::wrap(hit_test_result.point_in_viewport.y as f64 + window.PageYOffset() as f64);

        let touch = Touch::new(
            window, identifier, &target, client_x,
            client_y, // TODO: Get real screen coordinates?
            client_x, client_y, page_x, page_y, can_gc,
        );

        match event.event_type {
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
            TouchList::new(window, touches.r(), can_gc)
        };

        let event = DomTouchEvent::new(
            window,
            DOMString::from(event_name),
            EventBubbles::Bubbles,
            EventCancelable::Cancelable,
            Some(window),
            0i32,
            &touches,
            &TouchList::new(window, from_ref(&&*touch), can_gc),
            &TouchList::new(window, target_touches.r(), can_gc),
            // FIXME: modifier keys
            false,
            false,
            false,
            false,
        );
        let event = event.upcast::<Event>();
        let result = event.fire(&target, can_gc);

        match result {
            EventStatus::Canceled => TouchEventResult::Processed(false),
            EventStatus::NotCanceled => TouchEventResult::Processed(true),
        }
    }

    /// The entry point for all key processing for web content
    pub(crate) fn dispatch_key_event(
        &self,
        keyboard_event: ::keyboard_types::KeyboardEvent,
        can_gc: CanGc,
    ) {
        let focused = self.get_focused_element();
        let body = self.GetBody();

        let target = match (&focused, &body) {
            (Some(focused), _) => focused.upcast(),
            (&None, Some(body)) => body.upcast(),
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
            can_gc,
        );
        let event = keyevent.upcast::<Event>();
        event.fire(target, can_gc);
        let mut cancel_state = event.get_cancel_state();

        // https://w3c.github.io/uievents/#keys-cancelable-keys
        if keyboard_event.state == KeyState::Down &&
            is_character_value_key(&(keyboard_event.key)) &&
            !keyboard_event.is_composing &&
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
                can_gc,
            );
            let ev = event.upcast::<Event>();
            ev.fire(target, can_gc);
            cancel_state = ev.get_cancel_state();
        }

        if cancel_state == EventDefault::Allowed {
            let msg = EmbedderMsg::Keyboard(self.webview_id(), keyboard_event.clone());
            self.send_to_embedder(msg);

            // This behavior is unspecced
            // We are supposed to dispatch synthetic click activation for Space and/or Return,
            // however *when* we do it is up to us.
            // Here, we're dispatching it after the key event so the script has a chance to cancel it
            // https://www.w3.org/Bugs/Public/show_bug.cgi?id=27337
            if (keyboard_event.key == Key::Enter || keyboard_event.code == Code::Space) &&
                keyboard_event.state == KeyState::Up
            {
                if let Some(elem) = target.downcast::<Element>() {
                    elem.upcast::<Node>()
                        .fire_synthetic_mouse_event_not_trusted(DOMString::from("click"), can_gc);
                }
            }
        }
    }

    pub(crate) fn dispatch_ime_event(&self, event: ImeEvent, can_gc: CanGc) {
        let composition_event = match event {
            ImeEvent::Dismissed => {
                self.request_focus(
                    self.GetBody().as_ref().map(|e| e.upcast()),
                    FocusType::Element,
                    can_gc,
                );
                return;
            },
            ImeEvent::Composition(composition_event) => composition_event,
        };

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
            can_gc,
        );
        let event = compositionevent.upcast::<Event>();
        event.fire(target, can_gc);
    }

    // https://dom.spec.whatwg.org/#converting-nodes-into-a-node
    pub(crate) fn node_from_nodes_and_strings(
        &self,
        mut nodes: Vec<NodeOrString>,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<Node>> {
        if nodes.len() == 1 {
            Ok(match nodes.pop().unwrap() {
                NodeOrString::Node(node) => node,
                NodeOrString::String(string) => {
                    DomRoot::upcast(self.CreateTextNode(string, can_gc))
                },
            })
        } else {
            let fragment = DomRoot::upcast::<Node>(self.CreateDocumentFragment(can_gc));
            for node in nodes {
                match node {
                    NodeOrString::Node(node) => {
                        fragment.AppendChild(&node)?;
                    },
                    NodeOrString::String(string) => {
                        let node = DomRoot::upcast::<Node>(self.CreateTextNode(string, can_gc));
                        // No try!() here because appending a text node
                        // should not fail.
                        fragment.AppendChild(&node).unwrap();
                    },
                }
            }
            Ok(fragment)
        }
    }

    pub(crate) fn get_body_attribute(&self, local_name: &LocalName) -> DOMString {
        match self
            .GetBody()
            .and_then(DomRoot::downcast::<HTMLBodyElement>)
        {
            Some(ref body) => body.upcast::<Element>().get_string_attribute(local_name),
            None => DOMString::new(),
        }
    }

    pub(crate) fn set_body_attribute(
        &self,
        local_name: &LocalName,
        value: DOMString,
        can_gc: CanGc,
    ) {
        if let Some(ref body) = self
            .GetBody()
            .and_then(DomRoot::downcast::<HTMLBodyElement>)
        {
            let body = body.upcast::<Element>();
            let value = body.parse_attribute(&ns!(), local_name, value);
            body.set_attribute(local_name, value, can_gc);
        }
    }

    pub(crate) fn set_current_script(&self, script: Option<&HTMLScriptElement>) {
        self.current_script.set(script);
    }

    pub(crate) fn get_script_blocking_stylesheets_count(&self) -> u32 {
        self.script_blocking_stylesheets_count.get()
    }

    pub(crate) fn increment_script_blocking_stylesheet_count(&self) {
        let count_cell = &self.script_blocking_stylesheets_count;
        count_cell.set(count_cell.get() + 1);
    }

    pub(crate) fn decrement_script_blocking_stylesheet_count(&self) {
        let count_cell = &self.script_blocking_stylesheets_count;
        assert!(count_cell.get() > 0);
        count_cell.set(count_cell.get() - 1);
    }

    pub(crate) fn invalidate_stylesheets(&self) {
        self.stylesheets.borrow_mut().force_dirty(OriginSet::all());

        // Mark the document element dirty so a reflow will be performed.
        //
        // FIXME(emilio): Use the DocumentStylesheetSet invalidation stuff.
        if let Some(element) = self.GetDocumentElement() {
            element.upcast::<Node>().dirty(NodeDamage::NodeStyleDamaged);
        }
    }

    /// Whether or not this `Document` has any active requestAnimationFrame callbacks
    /// registered.
    pub(crate) fn has_active_request_animation_frame_callbacks(&self) -> bool {
        !self.animation_frame_list.borrow().is_empty()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-window-requestanimationframe>
    pub(crate) fn request_animation_frame(&self, callback: AnimationFrameCallback) -> u32 {
        let ident = self.animation_frame_ident.get() + 1;

        self.animation_frame_ident.set(ident);
        self.animation_frame_list
            .borrow_mut()
            .push((ident, Some(callback)));

        // If we are running 'fake' animation frames, we unconditionally
        // set up a one-shot timer for script to execute the rAF callbacks.
        if self.is_faking_animation_frames() && !self.window().throttled() {
            self.schedule_fake_animation_frame();
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
    pub(crate) fn cancel_animation_frame(&self, ident: u32) {
        let mut list = self.animation_frame_list.borrow_mut();
        if let Some(pair) = list.iter_mut().find(|pair| pair.0 == ident) {
            pair.1 = None;
        }
    }

    fn schedule_fake_animation_frame(&self) {
        warn!("Scheduling fake animation frame. Animation frames tick too fast.");
        let callback = FakeRequestAnimationFrameCallback {
            document: Trusted::new(self),
        };
        self.global().schedule_callback(
            OneshotTimerCallback::FakeRequestAnimationFrame(callback),
            Duration::from_millis(FAKE_REQUEST_ANIMATION_FRAME_DELAY),
        );
    }

    /// <https://html.spec.whatwg.org/multipage/#run-the-animation-frame-callbacks>
    pub(crate) fn run_the_animation_frame_callbacks(&self) {
        let _realm = enter_realm(self);
        rooted_vec!(let mut animation_frame_list);
        mem::swap(
            &mut *animation_frame_list,
            &mut *self.animation_frame_list.borrow_mut(),
        );

        self.pending_animation_ticks
            .borrow_mut()
            .remove(AnimationTickType::REQUEST_ANIMATION_FRAME);

        self.running_animation_callbacks.set(true);
        let was_faking_animation_frames = self.is_faking_animation_frames();
        let timing = self.global().performance().Now();

        for (_, callback) in animation_frame_list.drain(..) {
            if let Some(callback) = callback {
                callback.call(self, *timing);
            }
        }

        self.running_animation_callbacks.set(false);
        let callbacks_did_not_trigger_reflow = self.needs_reflow().is_none();
        let is_empty = self.animation_frame_list.borrow().is_empty();

        if !is_empty && callbacks_did_not_trigger_reflow && !was_faking_animation_frames {
            // If the rAF callbacks did not mutate the DOM, then the impending
            // reflow call as part of *update the rendering* will not do anything
            // and therefore no new frame will be sent to the compositor.
            // If this happens, the compositor will not tick the animation
            // and the next rAF will never be called! When this happens
            // for several frames, then the spurious rAF detection below
            // will kick in and use a timer to tick the callbacks. However,
            // for the interim frames where we are deciding whether this rAF
            // is considered spurious, we need to ensure that the layout
            // and compositor *do* tick the animation.
            self.set_needs_paint(true);
        }

        // Update the counter of spurious animation frames.
        let spurious_frames = self.spurious_animation_frames.get();
        if callbacks_did_not_trigger_reflow {
            if spurious_frames < SPURIOUS_ANIMATION_FRAME_THRESHOLD {
                self.spurious_animation_frames.set(spurious_frames + 1);
            }
        } else {
            self.spurious_animation_frames.set(0);
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
        let just_crossed_spurious_animation_threshold =
            !was_faking_animation_frames && self.is_faking_animation_frames();
        if is_empty || just_crossed_spurious_animation_threshold {
            if is_empty {
                // If the current animation frame list in the DOM instance is empty,
                // we can reuse the original `Vec<T>` that we put on the stack to
                // avoid allocating a new one next time an animation callback
                // is queued.
                mem::swap(
                    &mut *self.animation_frame_list.borrow_mut(),
                    &mut *animation_frame_list,
                );
            } else if just_crossed_spurious_animation_threshold {
                // We just realized that we need to stop requesting compositor's animation ticks
                // due to spurious animation frames, but we still have rAF callbacks queued. Since
                // `is_faking_animation_frames` would not have been true at the point where these
                // new callbacks were registered, the one-shot timer will not have been setup in
                // `request_animation_frame()`. Since we stop the compositor ticks below, we need
                // to expliclty trigger a OneshotTimerCallback for these queued callbacks.
                self.schedule_fake_animation_frame();
            }
            let event = ScriptMsg::ChangeRunningAnimationsState(
                AnimationState::NoAnimationCallbacksPresent,
            );
            self.window().send_to_constellation(event);
        }

        // If we were previously faking animation frames, we need to re-enable video refresh
        // callbacks when we stop seeing spurious animation frames.
        if was_faking_animation_frames && !self.is_faking_animation_frames() && !is_empty {
            self.window()
                .send_to_constellation(ScriptMsg::ChangeRunningAnimationsState(
                    AnimationState::AnimationCallbacksPresent,
                ));
        }
    }

    pub(crate) fn policy_container(&self) -> Ref<PolicyContainer> {
        self.policy_container.borrow()
    }

    /// Add the policy container and HTTPS state to a given request.
    ///
    /// TODO: Can this hapen for all requests that go through the document?
    pub(crate) fn prepare_request(&self, request: RequestBuilder) -> RequestBuilder {
        request
            .policy_container(self.policy_container().to_owned())
            .https_state(self.https_state.get())
    }

    pub(crate) fn fetch<Listener: FetchResponseListener + PreInvoke + Send + 'static>(
        &self,
        load: LoadType,
        mut request: RequestBuilder,
        listener: Listener,
    ) {
        request = request.insecure_requests_policy(self.insecure_requests_policy());
        let callback = NetworkListener {
            context: std::sync::Arc::new(Mutex::new(listener)),
            task_source: self
                .owner_global()
                .task_manager()
                .networking_task_source()
                .into(),
        }
        .into_callback();
        self.loader_mut()
            .fetch_async_with_callback(load, request, callback);
    }

    pub(crate) fn fetch_background<Listener: FetchResponseListener + PreInvoke + Send + 'static>(
        &self,
        mut request: RequestBuilder,
        listener: Listener,
    ) {
        request = request.insecure_requests_policy(self.insecure_requests_policy());
        let callback = NetworkListener {
            context: std::sync::Arc::new(Mutex::new(listener)),
            task_source: self
                .owner_global()
                .task_manager()
                .networking_task_source()
                .into(),
        }
        .into_callback();
        self.loader_mut().fetch_async_background(request, callback);
    }

    // https://html.spec.whatwg.org/multipage/#the-end
    // https://html.spec.whatwg.org/multipage/#delay-the-load-event
    pub(crate) fn finish_load(&self, load: LoadType, can_gc: CanGc) {
        // This does not delay the load event anymore.
        debug!("Document got finish_load: {:?}", load);
        self.loader.borrow_mut().finish_load(&load);

        match load {
            LoadType::Stylesheet(_) => {
                // A stylesheet finishing to load may unblock any pending
                // parsing-blocking script or deferred script.
                self.process_pending_parsing_blocking_script(can_gc);

                // Step 3.
                self.process_deferred_scripts();
            },
            LoadType::PageSource(_) => {
                // We finished loading the page, so if the `Window` is still waiting for
                // the first layout, allow it.
                if self.has_browsing_context && self.is_fully_active() {
                    self.window().allow_layout_if_necessary(can_gc);
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
        if self.top_level_dom_complete.get().is_none() && loader.is_only_blocked_by_iframes() {
            update_with_current_instant(&self.top_level_dom_complete);
        }

        if loader.is_blocked() || loader.events_inhibited() {
            // Step 6.
            return;
        }

        ScriptThread::mark_document_with_no_blocked_loads(self);
    }

    // https://html.spec.whatwg.org/multipage/#prompt-to-unload-a-document
    pub(crate) fn prompt_to_unload(&self, recursive_flag: bool, can_gc: CanGc) -> bool {
        // TODO: Step 1, increase the event loop's termination nesting level by 1.
        // Step 2
        self.incr_ignore_opens_during_unload_counter();
        //Step 3-5.
        let beforeunload_event = BeforeUnloadEvent::new(
            &self.window,
            atom!("beforeunload"),
            EventBubbles::Bubbles,
            EventCancelable::Cancelable,
        );
        let event = beforeunload_event.upcast::<Event>();
        event.set_trusted(true);
        let event_target = self.window.upcast::<EventTarget>();
        let has_listeners = event_target.has_listeners_for(&atom!("beforeunload"));
        self.window
            .dispatch_event_with_target_override(event, can_gc);
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
            let msg = EmbedderMsg::AllowUnload(self.webview_id(), chan);
            self.send_to_embedder(msg);
            can_unload = port.recv().unwrap() == AllowOrDeny::Allow;
        }
        // Step 9
        if !recursive_flag {
            // `prompt_to_unload` might cause futher modifications to the DOM so collecting here prevents
            // a double borrow if the `IFrameCollection` needs to be validated again.
            let iframes: Vec<_> = self.iframes().iter().collect();
            for iframe in &iframes {
                // TODO: handle the case of cross origin iframes.
                let document = iframe.owner_document();
                can_unload = document.prompt_to_unload(true, can_gc);
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
    pub(crate) fn unload(&self, recursive_flag: bool, can_gc: CanGc) {
        // TODO: Step 1, increase the event loop's termination nesting level by 1.
        // Step 2
        self.incr_ignore_opens_during_unload_counter();
        // Step 3-6 If oldDocument's page showing is true:
        if self.page_showing.get() {
            // Set oldDocument's page showing to false.
            self.page_showing.set(false);
            // Fire a page transition event named pagehide at oldDocument's relevant global object with oldDocument's
            // salvageable state.
            let event = PageTransitionEvent::new(
                &self.window,
                atom!("pagehide"),
                false,                  // bubbles
                false,                  // cancelable
                self.salvageable.get(), // persisted
                can_gc,
            );
            let event = event.upcast::<Event>();
            event.set_trusted(true);
            let _ = self
                .window
                .dispatch_event_with_target_override(event, can_gc);
            // Step 6 Update the visibility state of oldDocument to "hidden".
            self.update_visibility_state(DocumentVisibilityState::Hidden, can_gc);
        }
        // Step 7
        if !self.fired_unload.get() {
            let event = Event::new(
                self.window.upcast(),
                atom!("unload"),
                EventBubbles::Bubbles,
                EventCancelable::Cancelable,
                can_gc,
            );
            event.set_trusted(true);
            let event_target = self.window.upcast::<EventTarget>();
            let has_listeners = event_target.has_listeners_for(&atom!("unload"));
            let _ = self
                .window
                .dispatch_event_with_target_override(&event, can_gc);
            self.fired_unload.set(true);
            // Step 9
            if has_listeners {
                self.salvageable.set(false);
            }
        }
        // TODO: Step 8, decrease the event loop's termination nesting level by 1.

        // Step 13
        if !recursive_flag {
            // `unload` might cause futher modifications to the DOM so collecting here prevents
            // a double borrow if the `IFrameCollection` needs to be validated again.
            let iframes: Vec<_> = self.iframes().iter().collect();
            for iframe in &iframes {
                // TODO: handle the case of cross origin iframes.
                let document = iframe.owner_document();
                document.unload(true, can_gc);
                if !document.salvageable() {
                    self.salvageable.set(false);
                }
            }
        }

        let global_scope = self.window.as_global_scope();
        // Step 10, 14
        // https://html.spec.whatwg.org/multipage/#unloading-document-cleanup-steps
        if !self.salvageable.get() {
            // Step 1 of clean-up steps.
            global_scope.close_event_sources();
            let msg = ScriptMsg::DiscardDocument;
            let _ = global_scope.script_to_constellation_chan().send(msg);
        }
        // https://w3c.github.io/FileAPI/#lifeTime
        global_scope.clean_up_all_file_resources();

        // Step 15, End
        self.decr_ignore_opens_during_unload_counter();
    }

    // https://html.spec.whatwg.org/multipage/#the-end
    pub(crate) fn maybe_queue_document_completion(&self) {
        // https://html.spec.whatwg.org/multipage/#delaying-load-events-mode
        let is_in_delaying_load_events_mode = match self.window.undiscarded_window_proxy() {
            Some(window_proxy) => window_proxy.is_delaying_load_events_mode(),
            None => false,
        };

        // Note: if the document is not fully active, layout will have exited already,
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
        self.owner_global()
            .task_manager()
            .dom_manipulation_task_source()
            .queue(task!(fire_load_event: move || {
                let document = document.root();
                let window = document.window();
                if !window.is_alive() {
                    return;
                }

                // Step 7.1.
                document.set_ready_state(DocumentReadyState::Complete, CanGc::note());

                // Step 7.2.
                if document.browsing_context().is_none() {
                    return;
                }
                let event = Event::new(
                    window.upcast(),
                    atom!("load"),
                    EventBubbles::DoesNotBubble,
                    EventCancelable::NotCancelable,
                    CanGc::note(),
                );
                event.set_trusted(true);

                // http://w3c.github.io/navigation-timing/#widl-PerformanceNavigationTiming-loadEventStart
                update_with_current_instant(&document.load_event_start);

                debug!("About to dispatch load for {:?}", document.url());
                window.dispatch_event_with_target_override(&event, CanGc::note());

                // http://w3c.github.io/navigation-timing/#widl-PerformanceNavigationTiming-loadEventEnd
                update_with_current_instant(&document.load_event_end);

                if let Some(fragment) = document.url().fragment() {
                    document.check_and_scroll_fragment(fragment, CanGc::note());
                }
            }));

        // Step 8.
        let document = Trusted::new(self);
        if document.root().browsing_context().is_some() {
            self.owner_global()
                .task_manager()
                .dom_manipulation_task_source()
                .queue(task!(fire_pageshow_event: move || {
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
                        CanGc::note(),
                    );
                    let event = event.upcast::<Event>();
                    event.set_trusted(true);

                    window.dispatch_event_with_target_override(event, CanGc::note());
                }));
        }

        // Step 9.
        // TODO: pending application cache download process tasks.

        // Step 10.
        // TODO: printing steps.

        // Step 11.
        // TODO: ready for post-load tasks.

        // The dom.webxr.sessionavailable pref allows webxr
        // content to immediately begin a session without waiting for a user gesture.
        // TODO: should this only happen on the first document loaded?
        // https://immersive-web.github.io/webxr/#user-intention
        // https://github.com/immersive-web/navigation/issues/10
        #[cfg(feature = "webxr")]
        if pref!(dom_webxr_sessionavailable) && self.window.is_top_level() {
            self.window.Navigator().Xr().dispatch_sessionavailable();
        }

        // Step 12: completely loaded.
        // https://html.spec.whatwg.org/multipage/#completely-loaded
        // TODO: fully implement "completely loaded".
        let document = Trusted::new(self);
        if document.root().browsing_context().is_some() {
            self.owner_global()
                .task_manager()
                .dom_manipulation_task_source()
                .queue(task!(completely_loaded: move || {
                    let document = document.root();
                    document.completely_loaded.set(true);
                    if let Some(DeclarativeRefresh::PendingLoad {
                        url,
                        time
                    }) = &*document.declarative_refresh.borrow() {
                        // https://html.spec.whatwg.org/multipage/#shared-declarative-refresh-steps
                        document.window.as_global_scope().schedule_callback(
                            OneshotTimerCallback::RefreshRedirectDue(RefreshRedirectDue {
                                window: DomRoot::from_ref(document.window()),
                                url: url.clone(),
                            }),
                            Duration::from_secs(*time),
                        );
                    }
                    // Note: this will, among others, result in the "iframe-load-event-steps" being run.
                    // https://html.spec.whatwg.org/multipage/#iframe-load-event-steps
                    document.notify_constellation_load();
                }));
        }
    }

    pub(crate) fn completely_loaded(&self) -> bool {
        self.completely_loaded.get()
    }

    // https://html.spec.whatwg.org/multipage/#pending-parsing-blocking-script
    pub(crate) fn set_pending_parsing_blocking_script(
        &self,
        script: &HTMLScriptElement,
        load: Option<ScriptResult>,
    ) {
        assert!(!self.has_pending_parsing_blocking_script());
        *self.pending_parsing_blocking_script.borrow_mut() =
            Some(PendingScript::new_with_load(script, load));
    }

    // https://html.spec.whatwg.org/multipage/#pending-parsing-blocking-script
    pub(crate) fn has_pending_parsing_blocking_script(&self) -> bool {
        self.pending_parsing_blocking_script.borrow().is_some()
    }

    /// <https://html.spec.whatwg.org/multipage/#prepare-a-script> step 22.d.
    pub(crate) fn pending_parsing_blocking_script_loaded(
        &self,
        element: &HTMLScriptElement,
        result: ScriptResult,
        can_gc: CanGc,
    ) {
        {
            let mut blocking_script = self.pending_parsing_blocking_script.borrow_mut();
            let entry = blocking_script.as_mut().unwrap();
            assert!(&*entry.element == element);
            entry.loaded(result);
        }
        self.process_pending_parsing_blocking_script(can_gc);
    }

    fn process_pending_parsing_blocking_script(&self, can_gc: CanGc) {
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
                .resume_with_pending_parsing_blocking_script(&element, result, can_gc);
        }
    }

    // https://html.spec.whatwg.org/multipage/#set-of-scripts-that-will-execute-as-soon-as-possible
    pub(crate) fn add_asap_script(&self, script: &HTMLScriptElement) {
        self.asap_scripts_set
            .borrow_mut()
            .push(Dom::from_ref(script));
    }

    /// <https://html.spec.whatwg.org/multipage/#the-end> step 5.
    /// <https://html.spec.whatwg.org/multipage/#prepare-a-script> step 22.d.
    pub(crate) fn asap_script_loaded(&self, element: &HTMLScriptElement, result: ScriptResult) {
        {
            let mut scripts = self.asap_scripts_set.borrow_mut();
            let idx = scripts
                .iter()
                .position(|entry| &**entry == element)
                .unwrap();
            scripts.swap_remove(idx);
        }
        element.execute(result, CanGc::note());
    }

    // https://html.spec.whatwg.org/multipage/#list-of-scripts-that-will-execute-in-order-as-soon-as-possible
    pub(crate) fn push_asap_in_order_script(&self, script: &HTMLScriptElement) {
        self.asap_in_order_scripts_list.push(script);
    }

    /// <https://html.spec.whatwg.org/multipage/#the-end> step 5.
    /// <https://html.spec.whatwg.org/multipage/#prepare-a-script> step> 22.c.
    pub(crate) fn asap_in_order_script_loaded(
        &self,
        element: &HTMLScriptElement,
        result: ScriptResult,
    ) {
        self.asap_in_order_scripts_list.loaded(element, result);
        while let Some((element, result)) = self
            .asap_in_order_scripts_list
            .take_next_ready_to_be_executed()
        {
            element.execute(result, CanGc::note());
        }
    }

    // https://html.spec.whatwg.org/multipage/#list-of-scripts-that-will-execute-when-the-document-has-finished-parsing
    pub(crate) fn add_deferred_script(&self, script: &HTMLScriptElement) {
        self.deferred_scripts.push(script);
    }

    /// <https://html.spec.whatwg.org/multipage/#the-end> step 3.
    /// <https://html.spec.whatwg.org/multipage/#prepare-a-script> step 22.d.
    pub(crate) fn deferred_script_loaded(&self, element: &HTMLScriptElement, result: ScriptResult) {
        self.deferred_scripts.loaded(element, result);
        self.process_deferred_scripts();
    }

    /// <https://html.spec.whatwg.org/multipage/#the-end> step 3.
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
                element.execute(result, CanGc::note());
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
    pub(crate) fn maybe_dispatch_dom_content_loaded(&self) {
        if self.domcontentloaded_dispatched.get() {
            return;
        }
        self.domcontentloaded_dispatched.set(true);
        assert_ne!(
            self.ReadyState(),
            DocumentReadyState::Complete,
            "Complete before DOMContentLoaded?"
        );

        update_with_current_instant(&self.dom_content_loaded_event_start);

        // Step 4.1.
        let document = Trusted::new(self);
        self.owner_global()
            .task_manager()
            .dom_manipulation_task_source()
            .queue(
                task!(fire_dom_content_loaded_event: move || {
                let document = document.root();
                document.upcast::<EventTarget>().fire_bubbling_event(atom!("DOMContentLoaded"), CanGc::note());
                update_with_current_instant(&document.dom_content_loaded_event_end);
                })
            );

        // html parsing has finished - set dom content loaded
        self.interactive_time
            .borrow()
            .maybe_set_tti(self, InteractiveFlag::DOMContentLoaded);

        // Step 4.2.
        // TODO: client message queue.
    }

    // https://html.spec.whatwg.org/multipage/#abort-a-document
    pub(crate) fn abort(&self, can_gc: CanGc) {
        // We need to inhibit the loader before anything else.
        self.loader.borrow_mut().inhibit_events();

        // Step 1.
        for iframe in self.iframes().iter() {
            if let Some(document) = iframe.GetContentDocument() {
                // TODO: abort the active documents of every child browsing context.
                document.abort(can_gc);
                // TODO: salvageable flag.
            }
        }

        // Step 2.
        self.script_blocking_stylesheets_count.set(0);
        *self.pending_parsing_blocking_script.borrow_mut() = None;
        *self.asap_scripts_set.borrow_mut() = vec![];
        self.asap_in_order_scripts_list.clear();
        self.deferred_scripts.clear();
        let loads_cancelled = self.loader.borrow_mut().cancel_all_loads();
        let event_sources_canceled = self.window.as_global_scope().close_event_sources();
        if loads_cancelled || event_sources_canceled {
            // If any loads were canceled.
            self.salvageable.set(false);
        };

        // Also Step 2.
        // Note: the spec says to discard any tasks queued for fetch.
        // This cancels all tasks on the networking task source, which might be too broad.
        // See https://github.com/whatwg/html/issues/3837
        self.owner_global()
            .task_manager()
            .cancel_pending_tasks_for_source(TaskSourceName::Networking);

        // Step 3.
        if let Some(parser) = self.get_current_parser() {
            self.active_parser_was_aborted.set(true);
            parser.abort(can_gc);
            self.salvageable.set(false);
        }
    }

    pub(crate) fn notify_constellation_load(&self) {
        self.window().send_to_constellation(ScriptMsg::LoadComplete);
    }

    pub(crate) fn set_current_parser(&self, script: Option<&ServoParser>) {
        self.current_parser.set(script);
    }

    pub(crate) fn get_current_parser(&self) -> Option<DomRoot<ServoParser>> {
        self.current_parser.get()
    }

    /// A reference to the [`IFrameCollection`] of this [`Document`], holding information about
    /// `<iframe>`s found within it.
    pub(crate) fn iframes(&self) -> Ref<IFrameCollection> {
        self.iframes.borrow_mut().validate(self);
        self.iframes.borrow()
    }

    /// A mutable reference to the [`IFrameCollection`] of this [`Document`], holding information about
    /// `<iframe>`s found within it.
    pub(crate) fn iframes_mut(&self) -> RefMut<IFrameCollection> {
        self.iframes.borrow_mut().validate(self);
        self.iframes.borrow_mut()
    }

    pub(crate) fn get_dom_interactive(&self) -> Option<CrossProcessInstant> {
        self.dom_interactive.get()
    }

    pub(crate) fn set_navigation_start(&self, navigation_start: CrossProcessInstant) {
        self.interactive_time
            .borrow_mut()
            .set_navigation_start(navigation_start);
    }

    pub(crate) fn get_interactive_metrics(&self) -> Ref<InteractiveMetrics> {
        self.interactive_time.borrow()
    }

    pub(crate) fn has_recorded_tti_metric(&self) -> bool {
        self.get_interactive_metrics().get_tti().is_some()
    }

    pub(crate) fn get_dom_content_loaded_event_start(&self) -> Option<CrossProcessInstant> {
        self.dom_content_loaded_event_start.get()
    }

    pub(crate) fn get_dom_content_loaded_event_end(&self) -> Option<CrossProcessInstant> {
        self.dom_content_loaded_event_end.get()
    }

    pub(crate) fn get_dom_complete(&self) -> Option<CrossProcessInstant> {
        self.dom_complete.get()
    }

    pub(crate) fn get_top_level_dom_complete(&self) -> Option<CrossProcessInstant> {
        self.top_level_dom_complete.get()
    }

    pub(crate) fn get_load_event_start(&self) -> Option<CrossProcessInstant> {
        self.load_event_start.get()
    }

    pub(crate) fn get_load_event_end(&self) -> Option<CrossProcessInstant> {
        self.load_event_end.get()
    }

    pub(crate) fn get_unload_event_start(&self) -> Option<CrossProcessInstant> {
        self.unload_event_start.get()
    }

    pub(crate) fn get_unload_event_end(&self) -> Option<CrossProcessInstant> {
        self.unload_event_end.get()
    }

    pub(crate) fn start_tti(&self) {
        if self.get_interactive_metrics().needs_tti() {
            self.tti_window.borrow_mut().start_window();
        }
    }

    /// check tti for this document
    /// if it's been 10s since this doc encountered a task over 50ms, then we consider the
    /// main thread available and try to set tti
    pub(crate) fn record_tti_if_necessary(&self) {
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
        can_gc: CanGc,
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
            can_gc,
        );
        let event = event.upcast::<Event>();
        event.set_trusted(true);
        let target = node.upcast();
        event.fire(target, can_gc);
    }

    /// <https://html.spec.whatwg.org/multipage/#cookie-averse-document-object>
    pub(crate) fn is_cookie_averse(&self) -> bool {
        !self.has_browsing_context || !url_has_network_scheme(&self.url())
    }

    /// <https://html.spec.whatwg.org/multipage/#look-up-a-custom-element-definition>
    pub(crate) fn lookup_custom_element_definition(
        &self,
        namespace: &Namespace,
        local_name: &LocalName,
        is: Option<&LocalName>,
    ) -> Option<Rc<CustomElementDefinition>> {
        if !pref!(dom_customelements_enabled) {
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

    pub(crate) fn increment_throw_on_dynamic_markup_insertion_counter(&self) {
        let counter = self.throw_on_dynamic_markup_insertion_counter.get();
        self.throw_on_dynamic_markup_insertion_counter
            .set(counter + 1);
    }

    pub(crate) fn decrement_throw_on_dynamic_markup_insertion_counter(&self) {
        let counter = self.throw_on_dynamic_markup_insertion_counter.get();
        self.throw_on_dynamic_markup_insertion_counter
            .set(counter - 1);
    }

    pub(crate) fn react_to_environment_changes(&self) {
        for image in self.responsive_images.borrow().iter() {
            image.react_to_environment_changes();
        }
    }

    pub(crate) fn register_responsive_image(&self, img: &HTMLImageElement) {
        self.responsive_images.borrow_mut().push(Dom::from_ref(img));
    }

    pub(crate) fn unregister_responsive_image(&self, img: &HTMLImageElement) {
        let index = self
            .responsive_images
            .borrow()
            .iter()
            .position(|x| **x == *img);
        if let Some(i) = index {
            self.responsive_images.borrow_mut().remove(i);
        }
    }

    pub(crate) fn register_media_controls(&self, controls: &ShadowRoot) -> String {
        let id = Uuid::new_v4().to_string();
        self.media_controls
            .borrow_mut()
            .insert(id.clone(), Dom::from_ref(controls));
        id
    }

    pub(crate) fn unregister_media_controls(&self, id: &str) {
        if let Some(ref media_controls) = self.media_controls.borrow_mut().remove(id) {
            let media_controls = DomRoot::from_ref(&**media_controls);
            media_controls.Host().detach_shadow();
        } else {
            debug_assert!(false, "Trying to unregister unknown media controls");
        }
    }

    pub(crate) fn add_dirty_webgl_canvas(&self, context: &WebGLRenderingContext) {
        self.dirty_webgl_contexts
            .borrow_mut()
            .entry(context.context_id())
            .or_insert_with(|| Dom::from_ref(context));
    }

    pub(crate) fn flush_dirty_webgl_canvases(&self) {
        let dirty_context_ids: Vec<_> = self
            .dirty_webgl_contexts
            .borrow_mut()
            .drain()
            .filter(|(_, context)| context.onscreen())
            .map(|(id, _)| id)
            .collect();

        if dirty_context_ids.is_empty() {
            return;
        }

        #[allow(unused)]
        let mut time = 0;
        let (sender, receiver) = webgl::webgl_channel().unwrap();
        self.window
            .webgl_chan()
            .expect("Where's the WebGL channel?")
            .send(WebGLMsg::SwapBuffers(dirty_context_ids, sender, time))
            .unwrap();
        receiver.recv().unwrap();
    }

    #[cfg(feature = "webgpu")]
    pub(crate) fn webgpu_contexts(&self) -> WebGPUContextsMap {
        self.webgpu_contexts.clone()
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    #[cfg(feature = "webgpu")]
    pub(crate) fn update_rendering_of_webgpu_canvases(&self) {
        self.webgpu_contexts
            .borrow_mut()
            .iter()
            .filter_map(|(_, context)| context.root())
            .filter(|context| context.onscreen())
            .for_each(|context| context.update_rendering());
    }

    pub(crate) fn id_map(&self) -> Ref<HashMapTracedValues<Atom, Vec<Dom<Element>>>> {
        self.id_map.borrow()
    }

    pub(crate) fn name_map(&self) -> Ref<HashMapTracedValues<Atom, Vec<Dom<Element>>>> {
        self.name_map.borrow()
    }

    /// <https://drafts.csswg.org/resize-observer/#dom-resizeobserver-resizeobserver>
    pub(crate) fn add_resize_observer(&self, resize_observer: &ResizeObserver) {
        self.resize_observers
            .borrow_mut()
            .push(Dom::from_ref(resize_observer));
    }

    /// <https://drafts.csswg.org/resize-observer/#gather-active-observations-h>
    /// <https://drafts.csswg.org/resize-observer/#has-active-resize-observations>
    pub(crate) fn gather_active_resize_observations_at_depth(
        &self,
        depth: &ResizeObservationDepth,
        can_gc: CanGc,
    ) -> bool {
        let mut has_active_resize_observations = false;
        for observer in self.resize_observers.borrow_mut().iter_mut() {
            observer.gather_active_resize_observations_at_depth(
                depth,
                &mut has_active_resize_observations,
                can_gc,
            );
        }
        has_active_resize_observations
    }

    /// <https://drafts.csswg.org/resize-observer/#broadcast-active-resize-observations>
    pub(crate) fn broadcast_active_resize_observations(
        &self,
        can_gc: CanGc,
    ) -> ResizeObservationDepth {
        let mut shallowest = ResizeObservationDepth::max();
        // Breaking potential re-borrow cycle on `resize_observers`:
        // broadcasting resize observations calls into a JS callback,
        // which can add new observers.
        let iterator: Vec<DomRoot<ResizeObserver>> = self
            .resize_observers
            .borrow()
            .iter()
            .cloned()
            .map(|obs| DomRoot::from_ref(&*obs))
            .collect();
        for observer in iterator {
            observer.broadcast_active_resize_observations(&mut shallowest, can_gc);
        }
        shallowest
    }

    /// <https://drafts.csswg.org/resize-observer/#has-skipped-observations-h>
    pub(crate) fn has_skipped_resize_observations(&self) -> bool {
        self.resize_observers
            .borrow()
            .iter()
            .any(|observer| observer.has_skipped_resize_observations())
    }

    /// <https://drafts.csswg.org/resize-observer/#deliver-resize-loop-error-notification>
    pub(crate) fn deliver_resize_loop_error_notification(&self, can_gc: CanGc) {
        let error_info: ErrorInfo = crate::dom::bindings::error::ErrorInfo {
            message: "ResizeObserver loop completed with undelivered notifications.".to_string(),
            ..Default::default()
        };
        self.window
            .as_global_scope()
            .report_an_error(error_info, HandleValue::null(), can_gc);
    }

    pub(crate) fn status_code(&self) -> Option<u16> {
        self.status_code
    }

    /// <https://html.spec.whatwg.org/multipage/#encoding-parsing-a-url>
    pub(crate) fn encoding_parse_a_url(&self, url: &str) -> Result<ServoUrl, url::ParseError> {
        // NOTE: This algorithm is defined for both Document and environment settings objects.
        // This implementation is only for documents.

        // Step 1. Let encoding be UTF-8.
        // Step 2. If environment is a Document object, then set encoding to environment's character encoding.
        let encoding = self.encoding.get();

        // Step 3. Otherwise, if environment's relevant global object is a Window object, set encoding to environment's
        // relevant global object's associated Document's character encoding.

        // Step 4. Let baseURL be environment's base URL, if environment is a Document object;
        // otherwise environment's API base URL.
        let base_url = self.base_url();

        // Step 5. Return the result of applying the URL parser to url, with baseURL and encoding.
        url::Url::options()
            .base_url(Some(base_url.as_url()))
            .encoding_override(Some(&|s| encoding.encode(s).0))
            .parse(url)
            .map(ServoUrl::from)
    }

    /// <https://html.spec.whatwg.org/multipage/#allowed-to-use>
    pub(crate) fn allowed_to_use_feature(&self, _feature: PermissionName) -> bool {
        // Step 1. If document's browsing context is null, then return false.
        if !self.has_browsing_context {
            return false;
        }

        // Step 2. If document is not fully active, then return false.
        if !self.is_fully_active() {
            return false;
        }

        // Step 3. If the result of running is feature enabled in document for origin on
        // feature, document, and document's origin is "Enabled", then return true.
        // Step 4. Return false.
        // TODO: All features are currently enabled for `Document`s because we do not
        // implement the Permissions Policy specification.
        true
    }
}

fn is_character_value_key(key: &Key) -> bool {
    matches!(key, Key::Character(_) | Key::Enter)
}

#[derive(MallocSizeOf, PartialEq)]
pub(crate) enum DocumentSource {
    FromParser,
    NotFromParser,
}

#[allow(unsafe_code)]
pub(crate) trait LayoutDocumentHelpers<'dom> {
    fn is_html_document_for_layout(&self) -> bool;
    fn quirks_mode(self) -> QuirksMode;
    fn style_shared_lock(self) -> &'dom StyleSharedRwLock;
    fn shadow_roots(self) -> Vec<LayoutDom<'dom, ShadowRoot>>;
    fn shadow_roots_styles_changed(self) -> bool;
    fn flush_shadow_roots_stylesheets(self);
}

#[allow(unsafe_code)]
impl<'dom> LayoutDocumentHelpers<'dom> for LayoutDom<'dom, Document> {
    #[inline]
    fn is_html_document_for_layout(&self) -> bool {
        self.unsafe_get().is_html_document
    }

    #[inline]
    fn quirks_mode(self) -> QuirksMode {
        self.unsafe_get().quirks_mode.get()
    }

    #[inline]
    fn style_shared_lock(self) -> &'dom StyleSharedRwLock {
        self.unsafe_get().style_shared_lock()
    }

    #[inline]
    fn shadow_roots(self) -> Vec<LayoutDom<'dom, ShadowRoot>> {
        // FIXME(nox): We should just return a
        // &'dom HashSet<LayoutDom<'dom, ShadowRoot>> here but not until
        // I rework the ToLayout trait as mentioned in
        // LayoutDom::to_layout_slice.
        unsafe {
            self.unsafe_get()
                .shadow_roots
                .borrow_for_layout()
                .iter()
                .map(|sr| sr.to_layout())
                .collect()
        }
    }

    #[inline]
    fn shadow_roots_styles_changed(self) -> bool {
        self.unsafe_get().shadow_roots_styles_changed.get()
    }

    #[inline]
    fn flush_shadow_roots_stylesheets(self) {
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

        if !prefix.ends_with('.') {
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
    matches!(url.scheme(), "ftp" | "http" | "https")
}

#[derive(Clone, Copy, Eq, JSTraceable, MallocSizeOf, PartialEq)]
pub(crate) enum HasBrowsingContext {
    No,
    Yes,
}

impl Document {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new_inherited(
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
        status_code: Option<u16>,
        canceller: FetchCanceller,
        is_initial_about_blank: bool,
        inherited_insecure_requests_policy: Option<InsecureRequestsPolicy>,
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
            last_modified,
            url: DomRefCell::new(url),
            // https://dom.spec.whatwg.org/#concept-document-quirks
            quirks_mode: Cell::new(QuirksMode::NoQuirks),
            id_map: DomRefCell::new(HashMapTracedValues::new()),
            name_map: DomRefCell::new(HashMapTracedValues::new()),
            // https://dom.spec.whatwg.org/#concept-document-encoding
            encoding: Cell::new(encoding),
            is_html_document: is_html_document == IsHTMLDocument::HTMLDocument,
            activity: Cell::new(activity),
            tag_map: DomRefCell::new(HashMapTracedValues::new()),
            tagns_map: DomRefCell::new(HashMapTracedValues::new()),
            classes_map: DomRefCell::new(HashMapTracedValues::new()),
            images: Default::default(),
            embeds: Default::default(),
            links: Default::default(),
            forms: Default::default(),
            scripts: Default::default(),
            anchors: Default::default(),
            applets: Default::default(),
            iframes: Default::default(),
            style_shared_lock: {
                /// Per-process shared lock for author-origin stylesheets
                ///
                /// FIXME: make it per-document or per-pipeline instead:
                /// <https://github.com/servo/servo/issues/16027>
                /// (Need to figure out what to do with the style attribute
                /// of elements adopted into another document.)
                static PER_PROCESS_AUTHOR_SHARED_LOCK: LazyLock<StyleSharedRwLock> =
                    LazyLock::new(StyleSharedRwLock::new);

                PER_PROCESS_AUTHOR_SHARED_LOCK.clone()
                //StyleSharedRwLock::new()
            },
            stylesheets: DomRefCell::new(DocumentStylesheetSet::new()),
            stylesheet_list: MutNullableDom::new(None),
            ready_state: Cell::new(ready_state),
            domcontentloaded_dispatched: Cell::new(domcontentloaded_dispatched),
            focus_transaction: DomRefCell::new(FocusTransaction::NotInTransaction),
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
            base_element: Default::default(),
            appropriate_template_contents_owner_document: Default::default(),
            pending_restyles: DomRefCell::new(HashMap::new()),
            needs_paint: Cell::new(false),
            active_touch_points: DomRefCell::new(Vec::new()),
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
            origin,
            referrer,
            target_element: MutNullableDom::new(None),
            policy_container: DomRefCell::new(PolicyContainer::default()),
            last_click_info: DomRefCell::new(None),
            ignore_destructive_writes_counter: Default::default(),
            ignore_opens_during_unload_counter: Default::default(),
            spurious_animation_frames: Cell::new(0),
            dom_count: Cell::new(1),
            fullscreen_element: MutNullableDom::new(None),
            form_id_listener_map: Default::default(),
            interactive_time: DomRefCell::new(interactive_time),
            tti_window: DomRefCell::new(InteractiveWindow::default()),
            canceller,
            throw_on_dynamic_markup_insertion_counter: Cell::new(0),
            page_showing: Cell::new(false),
            salvageable: Cell::new(true),
            active_parser_was_aborted: Cell::new(false),
            fired_unload: Cell::new(false),
            responsive_images: Default::default(),
            redirect_count: Cell::new(0),
            completely_loaded: Cell::new(false),
            script_and_layout_blockers: Cell::new(0),
            delayed_tasks: Default::default(),
            shadow_roots: DomRefCell::new(HashSet::new()),
            shadow_roots_styles_changed: Cell::new(false),
            media_controls: DomRefCell::new(HashMap::new()),
            dirty_webgl_contexts: DomRefCell::new(HashMapTracedValues::new()),
            #[cfg(feature = "webgpu")]
            webgpu_contexts: Rc::new(RefCell::new(HashMapTracedValues::new())),
            selection: MutNullableDom::new(None),
            animation_timeline: if pref!(layout_animations_test_enabled) {
                DomRefCell::new(AnimationTimeline::new_for_testing())
            } else {
                DomRefCell::new(AnimationTimeline::new())
            },
            animations: DomRefCell::new(Animations::new()),
            dirty_root: Default::default(),
            declarative_refresh: Default::default(),
            pending_animation_ticks: Default::default(),
            pending_input_events: Default::default(),
            mouse_move_event_index: Default::default(),
            resize_observers: Default::default(),
            fonts: Default::default(),
            visibility_state: Cell::new(DocumentVisibilityState::Hidden),
            status_code,
            is_initial_about_blank: Cell::new(is_initial_about_blank),
            inherited_insecure_requests_policy: Cell::new(inherited_insecure_requests_policy),
            intersection_observer_task_queued: Cell::new(false),
        }
    }

    /// Returns a policy value that should be used for fetches initiated by this document.
    pub(crate) fn insecure_requests_policy(&self) -> InsecureRequestsPolicy {
        if let Some(csp_list) = self.get_csp_list() {
            for policy in &csp_list.0 {
                if policy.contains_a_directive_whose_name_is("upgrade-insecure-requests") &&
                    policy.disposition == PolicyDisposition::Enforce
                {
                    return InsecureRequestsPolicy::Upgrade;
                }
            }
        }

        self.inherited_insecure_requests_policy
            .get()
            .unwrap_or(InsecureRequestsPolicy::DoNotUpgrade)
    }

    /// Note a pending compositor event, to be processed at the next `update_the_rendering` task.
    pub(crate) fn note_pending_input_event(&self, event: ConstellationInputEvent) {
        let mut pending_compositor_events = self.pending_input_events.borrow_mut();
        if matches!(event.event, InputEvent::MouseMove(..)) {
            // First try to replace any existing mouse move event.
            if let Some(mouse_move_event) = self
                .mouse_move_event_index
                .borrow()
                .and_then(|index| pending_compositor_events.get_mut(index))
            {
                *mouse_move_event = event;
                return;
            }

            *self.mouse_move_event_index.borrow_mut() = Some(pending_compositor_events.len());
        }

        pending_compositor_events.push(event);
    }

    /// Get pending compositor events, for processing within an `update_the_rendering` task.
    pub(crate) fn take_pending_input_events(&self) -> Vec<ConstellationInputEvent> {
        // Reset the mouse event index.
        *self.mouse_move_event_index.borrow_mut() = None;
        mem::take(&mut *self.pending_input_events.borrow_mut())
    }

    pub(crate) fn set_csp_list(&self, csp_list: Option<CspList>) {
        self.policy_container.borrow_mut().set_csp_list(csp_list);
    }

    pub(crate) fn get_csp_list(&self) -> Option<CspList> {
        self.policy_container.borrow().csp_list.clone()
    }

    /// <https://www.w3.org/TR/CSP/#should-block-inline>
    pub(crate) fn should_elements_inline_type_behavior_be_blocked(
        &self,
        el: &Element,
        type_: csp::InlineCheckType,
        source: &str,
    ) -> csp::CheckResult {
        let element = csp::Element {
            nonce: el
                .get_attribute(&ns!(), &local_name!("nonce"))
                .map(|attr| Cow::Owned(attr.value().to_string())),
        };
        // TODO: Instead of ignoring violations, report them.
        self.get_csp_list()
            .map(|c| {
                c.should_elements_inline_type_behavior_be_blocked(&element, type_, source)
                    .0
            })
            .unwrap_or(csp::CheckResult::Allowed)
    }

    /// Prevent any JS or layout from running until the corresponding call to
    /// `remove_script_and_layout_blocker`. Used to isolate periods in which
    /// the DOM is in an unstable state and should not be exposed to arbitrary
    /// web content. Any attempts to invoke content JS or query layout during
    /// that time will trigger a panic. `add_delayed_task` will cause the
    /// provided task to be executed as soon as the last blocker is removed.
    pub(crate) fn add_script_and_layout_blocker(&self) {
        self.script_and_layout_blockers
            .set(self.script_and_layout_blockers.get() + 1);
    }

    /// Terminate the period in which JS or layout is disallowed from running.
    /// If no further blockers remain, any delayed tasks in the queue will
    /// be executed in queue order until the queue is empty.
    pub(crate) fn remove_script_and_layout_blocker(&self) {
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
    pub(crate) fn add_delayed_task<T: 'static + TaskBox>(&self, task: T) {
        self.delayed_tasks.borrow_mut().push(Box::new(task));
    }

    /// Assert that the DOM is in a state that will allow running content JS or
    /// performing a layout operation.
    pub(crate) fn ensure_safe_to_run_script_or_layout(&self) {
        assert_eq!(
            self.script_and_layout_blockers.get(),
            0,
            "Attempt to use script or layout while DOM not in a stable state"
        );
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
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
        status_code: Option<u16>,
        canceller: FetchCanceller,
        is_initial_about_blank: bool,
        inherited_insecure_requests_policy: Option<InsecureRequestsPolicy>,
        can_gc: CanGc,
    ) -> DomRoot<Document> {
        Self::new_with_proto(
            window,
            None,
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
            status_code,
            canceller,
            is_initial_about_blank,
            inherited_insecure_requests_policy,
            can_gc,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn new_with_proto(
        window: &Window,
        proto: Option<HandleObject>,
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
        status_code: Option<u16>,
        canceller: FetchCanceller,
        is_initial_about_blank: bool,
        inherited_insecure_requests_policy: Option<InsecureRequestsPolicy>,
        can_gc: CanGc,
    ) -> DomRoot<Document> {
        let document = reflect_dom_object_with_proto(
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
                status_code,
                canceller,
                is_initial_about_blank,
                inherited_insecure_requests_policy,
            )),
            window,
            proto,
            can_gc,
        );
        {
            let node = document.upcast::<Node>();
            node.set_owner_doc(&document);
        }
        document
    }

    pub(crate) fn get_redirect_count(&self) -> u16 {
        self.redirect_count.get()
    }

    pub(crate) fn set_redirect_count(&self, count: u16) {
        self.redirect_count.set(count)
    }

    pub(crate) fn elements_by_name_count(&self, name: &DOMString) -> u32 {
        if name.is_empty() {
            return 0;
        }
        self.count_node_list(|n| Document::is_element_in_get_by_name(n, name))
    }

    pub(crate) fn nth_element_by_name(
        &self,
        index: u32,
        name: &DOMString,
    ) -> Option<DomRoot<Node>> {
        if name.is_empty() {
            return None;
        }
        self.nth_in_node_list(index, |n| Document::is_element_in_get_by_name(n, name))
    }

    // Note that document.getByName does not match on the same conditions
    // as the document named getter.
    fn is_element_in_get_by_name(node: &Node, name: &DOMString) -> bool {
        let element = match node.downcast::<Element>() {
            Some(element) => element,
            None => return false,
        };
        if element.namespace() != &ns!(html) {
            return false;
        }
        element.get_name().is_some_and(|n| *n == **name)
    }

    fn count_node_list<F: Fn(&Node) -> bool>(&self, callback: F) -> u32 {
        let doc = self.GetDocumentElement();
        let maybe_node = doc.as_deref().map(Castable::upcast::<Node>);
        maybe_node
            .iter()
            .flat_map(|node| node.traverse_preorder(ShadowIncluding::No))
            .filter(|node| callback(node))
            .count() as u32
    }

    fn nth_in_node_list<F: Fn(&Node) -> bool>(
        &self,
        index: u32,
        callback: F,
    ) -> Option<DomRoot<Node>> {
        let doc = self.GetDocumentElement();
        let maybe_node = doc.as_deref().map(Castable::upcast::<Node>);
        maybe_node
            .iter()
            .flat_map(|node| node.traverse_preorder(ShadowIncluding::No))
            .filter(|node| callback(node))
            .nth(index as usize)
            .map(|n| DomRoot::from_ref(&*n))
    }

    fn get_html_element(&self) -> Option<DomRoot<HTMLHtmlElement>> {
        self.GetDocumentElement().and_then(DomRoot::downcast)
    }

    /// Return a reference to the per-document shared lock used in stylesheets.
    pub(crate) fn style_shared_lock(&self) -> &StyleSharedRwLock {
        &self.style_shared_lock
    }

    /// Flushes the stylesheet list, and returns whether any stylesheet changed.
    pub(crate) fn flush_stylesheets_for_reflow(&self) -> bool {
        // NOTE(emilio): The invalidation machinery is used on the replicated
        // list in layout.
        //
        // FIXME(emilio): This really should differentiate between CSSOM changes
        // and normal stylesheets additions / removals, because in the last case
        // layout already has that information and we could avoid dirtying the whole thing.
        let mut stylesheets = self.stylesheets.borrow_mut();
        let have_changed = stylesheets.has_changed();
        stylesheets.flush_without_invalidation();
        have_changed
    }

    pub(crate) fn salvageable(&self) -> bool {
        self.salvageable.get()
    }

    /// <https://html.spec.whatwg.org/multipage/#appropriate-template-contents-owner-document>
    pub(crate) fn appropriate_template_contents_owner_document(
        &self,
        can_gc: CanGc,
    ) -> DomRoot<Document> {
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
                    false,
                    Some(self.insecure_requests_policy()),
                    can_gc,
                );
                new_doc
                    .appropriate_template_contents_owner_document
                    .set(Some(&new_doc));
                new_doc
            })
    }

    pub(crate) fn get_element_by_id(&self, id: &Atom) -> Option<DomRoot<Element>> {
        self.id_map
            .borrow()
            .get(id)
            .map(|elements| DomRoot::from_ref(&*elements[0]))
    }

    pub(crate) fn ensure_pending_restyle(&self, el: &Element) -> RefMut<PendingRestyle> {
        let map = self.pending_restyles.borrow_mut();
        RefMut::map(map, |m| {
            &mut m
                .entry(Dom::from_ref(el))
                .or_insert_with(|| NoTrace(PendingRestyle::default()))
                .0
        })
    }

    pub(crate) fn element_state_will_change(&self, el: &Element) {
        let mut entry = self.ensure_pending_restyle(el);
        if entry.snapshot.is_none() {
            entry.snapshot = Some(Snapshot::new());
        }
        let snapshot = entry.snapshot.as_mut().unwrap();
        if snapshot.state.is_none() {
            snapshot.state = Some(el.state());
        }
    }

    pub(crate) fn element_attr_will_change(&self, el: &Element, attr: &Attr) {
        // FIXME(emilio): Kind of a shame we have to duplicate this.
        //
        // I'm getting rid of the whole hashtable soon anyway, since all it does
        // right now is populate the element restyle data in layout, and we
        // could in theory do it in the DOM I think.
        let mut entry = self.ensure_pending_restyle(el);
        if entry.snapshot.is_none() {
            entry.snapshot = Some(Snapshot::new());
        }
        if attr.local_name() == &local_name!("style") {
            entry.hint.insert(RestyleHint::RESTYLE_STYLE_ATTRIBUTE);
        }

        if vtable_for(el.upcast()).attribute_affects_presentational_hints(attr) {
            entry.hint.insert(RestyleHint::RESTYLE_SELF);
        }

        let snapshot = entry.snapshot.as_mut().unwrap();
        if attr.local_name() == &local_name!("id") {
            if snapshot.id_changed {
                return;
            }
            snapshot.id_changed = true;
        } else if attr.local_name() == &local_name!("class") {
            if snapshot.class_changed {
                return;
            }
            snapshot.class_changed = true;
        } else {
            snapshot.other_attributes_changed = true;
        }
        let local_name = style::LocalName::cast(attr.local_name());
        if !snapshot.changed_attrs.contains(local_name) {
            snapshot.changed_attrs.push(local_name.clone());
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

    pub(crate) fn set_referrer_policy(&self, policy: ReferrerPolicy) {
        self.policy_container
            .borrow_mut()
            .set_referrer_policy(policy);
    }

    pub(crate) fn get_referrer_policy(&self) -> ReferrerPolicy {
        self.policy_container.borrow().get_referrer_policy()
    }

    pub(crate) fn set_target_element(&self, node: Option<&Element>) {
        if let Some(ref element) = self.target_element.get() {
            element.set_target_state(false);
        }

        self.target_element.set(node);

        if let Some(ref element) = self.target_element.get() {
            element.set_target_state(true);
        }
    }

    pub(crate) fn incr_ignore_destructive_writes_counter(&self) {
        self.ignore_destructive_writes_counter
            .set(self.ignore_destructive_writes_counter.get() + 1);
    }

    pub(crate) fn decr_ignore_destructive_writes_counter(&self) {
        self.ignore_destructive_writes_counter
            .set(self.ignore_destructive_writes_counter.get() - 1);
    }

    pub(crate) fn is_prompting_or_unloading(&self) -> bool {
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
    pub(crate) fn enter_fullscreen(&self, pending: &Element, can_gc: CanGc) -> Rc<Promise> {
        // Step 1
        let in_realm_proof = AlreadyInRealm::assert();
        let promise = Promise::new_in_current_realm(InRealm::Already(&in_realm_proof), can_gc);
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

        if pref!(dom_fullscreen_test) {
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
            let event = EmbedderMsg::NotifyFullscreenStateChanged(self.webview_id(), true);
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
            Some(pipeline_id),
            TaskSourceName::DOMManipulation,
        );
        let msg = MainThreadScriptMsg::Common(script_msg);
        window.main_thread_script_chan().send(msg).unwrap();

        promise
    }

    // https://fullscreen.spec.whatwg.org/#exit-fullscreen
    pub(crate) fn exit_fullscreen(&self, can_gc: CanGc) -> Rc<Promise> {
        let global = self.global();
        // Step 1
        let in_realm_proof = AlreadyInRealm::assert();
        let promise = Promise::new_in_current_realm(InRealm::Already(&in_realm_proof), can_gc);
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
        let event = EmbedderMsg::NotifyFullscreenStateChanged(self.webview_id(), false);
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

    pub(crate) fn set_fullscreen_element(&self, element: Option<&Element>) {
        self.fullscreen_element.set(element);
    }

    pub(crate) fn get_allow_fullscreen(&self) -> bool {
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
                    window
                        .GetFrameElement()
                        .is_some_and(|el| el.has_attribute(&local_name!("allowfullscreen")))
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

    pub(crate) fn register_shadow_root(&self, shadow_root: &ShadowRoot) {
        self.shadow_roots
            .borrow_mut()
            .insert(Dom::from_ref(shadow_root));
        self.invalidate_shadow_roots_stylesheets();
    }

    pub(crate) fn unregister_shadow_root(&self, shadow_root: &ShadowRoot) {
        let mut shadow_roots = self.shadow_roots.borrow_mut();
        shadow_roots.remove(&Dom::from_ref(shadow_root));
    }

    pub(crate) fn invalidate_shadow_roots_stylesheets(&self) {
        self.shadow_roots_styles_changed.set(true);
    }

    pub(crate) fn shadow_roots_styles_changed(&self) -> bool {
        self.shadow_roots_styles_changed.get()
    }

    pub(crate) fn flush_shadow_roots_stylesheets(&self) {
        if !self.shadow_roots_styles_changed.get() {
            return;
        }
        self.shadow_roots_styles_changed.set(false);
    }

    pub(crate) fn stylesheet_count(&self) -> usize {
        self.stylesheets.borrow().len()
    }

    pub(crate) fn stylesheet_at(&self, index: usize) -> Option<DomRoot<CSSStyleSheet>> {
        let stylesheets = self.stylesheets.borrow();

        stylesheets
            .get(Origin::Author, index)
            .and_then(|s| s.owner.upcast::<Node>().get_cssom_stylesheet())
    }

    /// Add a stylesheet owned by `owner` to the list of document sheets, in the
    /// correct tree position.
    #[cfg_attr(crown, allow(crown::unrooted_must_root))] // Owner needs to be rooted already necessarily.
    pub(crate) fn add_stylesheet(&self, owner: &Element, sheet: Arc<Stylesheet>) {
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

        let cloned_stylesheet = sheet.clone();
        let insertion_point2 = insertion_point.clone();
        self.window.layout_mut().add_stylesheet(
            cloned_stylesheet,
            insertion_point2.as_ref().map(|s| s.sheet.clone()),
        );

        DocumentOrShadowRoot::add_stylesheet(
            owner,
            StylesheetSetRef::Document(stylesheets),
            sheet,
            insertion_point,
            self.style_shared_lock(),
        );
    }

    /// Given a stylesheet, load all web fonts from it in Layout.
    pub(crate) fn load_web_fonts_from_stylesheet(&self, stylesheet: Arc<Stylesheet>) {
        self.window
            .layout()
            .load_web_fonts_from_stylesheet(stylesheet);
    }

    /// Remove a stylesheet owned by `owner` from the list of document sheets.
    #[cfg_attr(crown, allow(crown::unrooted_must_root))] // Owner needs to be rooted already necessarily.
    pub(crate) fn remove_stylesheet(&self, owner: &Element, stylesheet: &Arc<Stylesheet>) {
        let cloned_stylesheet = stylesheet.clone();
        self.window
            .layout_mut()
            .remove_stylesheet(cloned_stylesheet);

        DocumentOrShadowRoot::remove_stylesheet(
            owner,
            stylesheet,
            StylesheetSetRef::Document(&mut *self.stylesheets.borrow_mut()),
        )
    }

    pub(crate) fn get_elements_with_id(&self, id: &Atom) -> Ref<[Dom<Element>]> {
        Ref::map(self.id_map.borrow(), |map| {
            map.get(id).map(|vec| &**vec).unwrap_or_default()
        })
    }

    pub(crate) fn get_elements_with_name(&self, name: &Atom) -> Ref<[Dom<Element>]> {
        Ref::map(self.name_map.borrow(), |map| {
            map.get(name).map(|vec| &**vec).unwrap_or_default()
        })
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn drain_pending_restyles(&self) -> Vec<(TrustedNodeAddress, PendingRestyle)> {
        self.pending_restyles
            .borrow_mut()
            .drain()
            .filter_map(|(elem, restyle)| {
                let node = elem.upcast::<Node>();
                if !node.get_flag(NodeFlags::IS_CONNECTED) {
                    return None;
                }
                node.note_dirty_descendants();
                Some((node.to_trusted_node_address(), restyle.0))
            })
            .collect()
    }

    /// Note a pending animation tick, to be processed at the next `update_the_rendering` task.
    pub(crate) fn note_pending_animation_tick(&self, tick_type: AnimationTickType) {
        self.pending_animation_ticks.borrow_mut().extend(tick_type);
    }

    /// Whether this document has received an animation tick for rafs.
    pub(crate) fn has_received_raf_tick(&self) -> bool {
        self.pending_animation_ticks
            .borrow()
            .contains(AnimationTickType::REQUEST_ANIMATION_FRAME)
    }

    pub(crate) fn advance_animation_timeline_for_testing(&self, delta: f64) {
        self.animation_timeline.borrow_mut().advance_specific(delta);
        let current_timeline_value = self.current_animation_timeline_value();
        self.animations
            .borrow()
            .update_for_new_timeline_value(&self.window, current_timeline_value);
    }

    pub(crate) fn maybe_mark_animating_nodes_as_dirty(&self) {
        let current_timeline_value = self.current_animation_timeline_value();
        self.animations
            .borrow()
            .mark_animating_nodes_as_dirty(current_timeline_value);
    }

    pub(crate) fn current_animation_timeline_value(&self) -> f64 {
        self.animation_timeline.borrow().current_value()
    }

    pub(crate) fn animations(&self) -> Ref<Animations> {
        self.animations.borrow()
    }

    pub(crate) fn update_animations_post_reflow(&self) {
        self.animations
            .borrow()
            .do_post_reflow_update(&self.window, self.current_animation_timeline_value());
    }

    pub(crate) fn cancel_animations_for_node(&self, node: &Node) {
        self.animations.borrow().cancel_animations_for_node(node);
    }

    /// An implementation of <https://drafts.csswg.org/web-animations-1/#update-animations-and-send-events>.
    pub(crate) fn update_animations_and_send_events(&self, can_gc: CanGc) {
        // Only update the time if it isn't being managed by a test.
        if !pref!(layout_animations_test_enabled) {
            self.animation_timeline.borrow_mut().update();
        }

        // > 1. Update the current time of all timelines associated with doc passing now
        // > as the timestamp.
        // > 2. Remove replaced animations for doc.
        //
        // We still want to update the animations, because our timeline
        // value might have been advanced previously via the TestBinding.
        let current_timeline_value = self.current_animation_timeline_value();
        self.animations
            .borrow()
            .update_for_new_timeline_value(&self.window, current_timeline_value);
        self.maybe_mark_animating_nodes_as_dirty();

        // > 3. Perform a microtask checkpoint.
        self.window()
            .as_global_scope()
            .perform_a_microtask_checkpoint(can_gc);

        // Steps 4 through 7 occur inside `send_pending_events().`
        let _realm = enter_realm(self);
        self.animations().send_pending_events(self.window(), can_gc);
    }

    pub(crate) fn will_declaratively_refresh(&self) -> bool {
        self.declarative_refresh.borrow().is_some()
    }
    pub(crate) fn set_declarative_refresh(&self, refresh: DeclarativeRefresh) {
        *self.declarative_refresh.borrow_mut() = Some(refresh);
    }

    /// <https://html.spec.whatwg.org/multipage/#visibility-state>
    fn update_visibility_state(&self, visibility_state: DocumentVisibilityState, can_gc: CanGc) {
        // Step 1 If document's visibility state equals visibilityState, then return.
        if self.visibility_state.get() == visibility_state {
            return;
        }
        // Step 2 Set document's visibility state to visibilityState.
        self.visibility_state.set(visibility_state);
        // Step 3 Queue a new VisibilityStateEntry whose visibility state is visibilityState and whose timestamp is
        // the current high resolution time given document's relevant global object.
        let entry = VisibilityStateEntry::new(
            &self.global(),
            visibility_state,
            CrossProcessInstant::now(),
            can_gc,
        );
        self.window
            .Performance()
            .queue_entry(entry.upcast::<PerformanceEntry>(), can_gc);

        // Step 4 Run the screen orientation change steps with document.
        // TODO ScreenOrientation hasn't implemented yet

        // Step 5 Run the view transition page visibility change steps with document.
        // TODO ViewTransition hasn't implemented yet

        // Step 6 Run any page visibility change steps which may be defined in other specifications, with visibility
        // state and document. Any other specs' visibility steps will go here.

        // <https://www.w3.org/TR/gamepad/#handling-visibility-change>
        if visibility_state == DocumentVisibilityState::Hidden {
            self.window
                .Navigator()
                .GetGamepads()
                .iter_mut()
                .for_each(|gamepad| {
                    if let Some(g) = gamepad {
                        g.vibration_actuator().handle_visibility_change();
                    }
                });
        }

        // Step 7 Fire an event named visibilitychange at document, with its bubbles attribute initialized to true.
        self.upcast::<EventTarget>()
            .fire_bubbling_event(atom!("visibilitychange"), can_gc);
    }

    /// <https://html.spec.whatwg.org/multipage/#is-initial-about:blank>
    pub(crate) fn is_initial_about_blank(&self) -> bool {
        self.is_initial_about_blank.get()
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

#[allow(non_snake_case)]
impl DocumentMethods<crate::DomTypeHolder> for Document {
    // https://dom.spec.whatwg.org/#dom-document-document
    fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<Document>> {
        let doc = window.Document();
        let docloader = DocumentLoader::new(&doc.loader());
        Ok(Document::new_with_proto(
            window,
            proto,
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
            false,
            Some(doc.insecure_requests_policy()),
            can_gc,
        ))
    }

    // https://w3c.github.io/editing/ActiveDocuments/execCommand.html#querycommandsupported()
    fn QueryCommandSupported(&self, _command: DOMString) -> bool {
        false
    }

    // https://drafts.csswg.org/cssom/#dom-document-stylesheets
    fn StyleSheets(&self) -> DomRoot<StyleSheetList> {
        self.stylesheet_list.or_init(|| {
            StyleSheetList::new(
                &self.window,
                StyleSheetListOwner::Document(Dom::from_ref(self)),
                CanGc::note(),
            )
        })
    }

    // https://dom.spec.whatwg.org/#dom-document-implementation
    fn Implementation(&self) -> DomRoot<DOMImplementation> {
        self.implementation
            .or_init(|| DOMImplementation::new(self, CanGc::note()))
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
        let host = match get_registrable_domain_suffix_of_or_is_equal_to(&value, effective_domain) {
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
        if let Some(entry) = self.tag_map.borrow_mut().get(&qualified_name) {
            return DomRoot::from_ref(entry);
        }
        let result =
            HTMLCollection::by_qualified_name(&self.window, self.upcast(), qualified_name.clone());
        self.tag_map
            .borrow_mut()
            .insert(qualified_name, Dom::from_ref(&*result));
        result
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
        if let Some(collection) = self.tagns_map.borrow().get(&qname) {
            return DomRoot::from_ref(collection);
        }
        let result = HTMLCollection::by_qual_tag_name(&self.window, self.upcast(), qname.clone());
        self.tagns_map
            .borrow_mut()
            .insert(qname, Dom::from_ref(&*result));
        result
    }

    // https://dom.spec.whatwg.org/#dom-document-getelementsbyclassname
    fn GetElementsByClassName(&self, classes: DOMString) -> DomRoot<HTMLCollection> {
        let class_atoms: Vec<Atom> = split_html_space_chars(&classes).map(Atom::from).collect();
        if let Some(collection) = self.classes_map.borrow().get(&class_atoms) {
            return DomRoot::from_ref(collection);
        }
        let result =
            HTMLCollection::by_atomic_class_name(&self.window, self.upcast(), class_atoms.clone());
        self.classes_map
            .borrow_mut()
            .insert(class_atoms, Dom::from_ref(&*result));
        result
    }

    // https://dom.spec.whatwg.org/#dom-nonelementparentnode-getelementbyid
    fn GetElementById(&self, id: DOMString) -> Option<DomRoot<Element>> {
        self.get_element_by_id(&Atom::from(id))
    }

    /// <https://dom.spec.whatwg.org/#dom-document-createelement>
    fn CreateElement(
        &self,
        mut local_name: DOMString,
        options: StringOrElementCreationOptions,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<Element>> {
        // Step 1. If localName does not match the Name production, then throw an "InvalidCharacterError" DOMException.
        if !matches_name_production(&local_name) {
            debug!("Not a valid element name");
            return Err(Error::InvalidCharacter);
        }

        if self.is_html_document {
            local_name.make_ascii_lowercase();
        }

        let ns = if self.is_html_document || self.is_xhtml_document() {
            ns!(html)
        } else {
            ns!()
        };

        let name = QualName::new(None, ns, LocalName::from(local_name));
        let is = match options {
            StringOrElementCreationOptions::String(_) => None,
            StringOrElementCreationOptions::ElementCreationOptions(options) => {
                options.is.as_ref().map(|is| LocalName::from(&**is))
            },
        };
        Ok(Element::create(
            name,
            is,
            self,
            ElementCreator::ScriptCreated,
            CustomElementCreationMode::Synchronous,
            None,
            can_gc,
        ))
    }

    /// <https://dom.spec.whatwg.org/#dom-document-createelementns>
    fn CreateElementNS(
        &self,
        namespace: Option<DOMString>,
        qualified_name: DOMString,
        options: StringOrElementCreationOptions,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<Element>> {
        // Step 1. Let namespace, prefix, and localName be the result of passing namespace and qualifiedName
        // to validate and extract.
        let (namespace, prefix, local_name) = validate_and_extract(namespace, &qualified_name)?;

        // Step 2. Let is be null.
        // Step 3. If options is a dictionary and options["is"] exists, then set is to it.
        let name = QualName::new(prefix, namespace, local_name);
        let is = match options {
            StringOrElementCreationOptions::String(_) => None,
            StringOrElementCreationOptions::ElementCreationOptions(options) => {
                options.is.as_ref().map(|is| LocalName::from(&**is))
            },
        };

        // Step 4. Return the result of creating an element given document, localName, namespace, prefix, is, and true.
        Ok(Element::create(
            name,
            is,
            self,
            ElementCreator::ScriptCreated,
            CustomElementCreationMode::Synchronous,
            None,
            can_gc,
        ))
    }

    /// <https://dom.spec.whatwg.org/#dom-document-createattribute>
    fn CreateAttribute(&self, mut local_name: DOMString, can_gc: CanGc) -> Fallible<DomRoot<Attr>> {
        // Step 1. If localName does not match the Name production in XML,
        // then throw an "InvalidCharacterError" DOMException.
        if !matches_name_production(&local_name) {
            debug!("Not a valid element name");
            return Err(Error::InvalidCharacter);
        }
        if self.is_html_document {
            local_name.make_ascii_lowercase();
        }
        let name = LocalName::from(local_name);
        let value = AttrValue::String("".to_owned());

        Ok(Attr::new(
            self,
            name.clone(),
            value,
            name,
            ns!(),
            None,
            None,
            can_gc,
        ))
    }

    // https://dom.spec.whatwg.org/#dom-document-createattributens
    fn CreateAttributeNS(
        &self,
        namespace: Option<DOMString>,
        qualified_name: DOMString,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<Attr>> {
        let (namespace, prefix, local_name) = validate_and_extract(namespace, &qualified_name)?;
        let value = AttrValue::String("".to_owned());
        let qualified_name = LocalName::from(qualified_name);
        Ok(Attr::new(
            self,
            local_name,
            value,
            qualified_name,
            namespace,
            prefix,
            None,
            can_gc,
        ))
    }

    // https://dom.spec.whatwg.org/#dom-document-createdocumentfragment
    fn CreateDocumentFragment(&self, can_gc: CanGc) -> DomRoot<DocumentFragment> {
        DocumentFragment::new(self, can_gc)
    }

    // https://dom.spec.whatwg.org/#dom-document-createtextnode
    fn CreateTextNode(&self, data: DOMString, can_gc: CanGc) -> DomRoot<Text> {
        Text::new(data, self, can_gc)
    }

    // https://dom.spec.whatwg.org/#dom-document-createcdatasection
    fn CreateCDATASection(
        &self,
        data: DOMString,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<CDATASection>> {
        // Step 1
        if self.is_html_document {
            return Err(Error::NotSupported);
        }

        // Step 2
        if data.contains("]]>") {
            return Err(Error::InvalidCharacter);
        }

        // Step 3
        Ok(CDATASection::new(data, self, can_gc))
    }

    // https://dom.spec.whatwg.org/#dom-document-createcomment
    fn CreateComment(&self, data: DOMString, can_gc: CanGc) -> DomRoot<Comment> {
        Comment::new(data, self, None, can_gc)
    }

    // https://dom.spec.whatwg.org/#dom-document-createprocessinginstruction
    fn CreateProcessingInstruction(
        &self,
        target: DOMString,
        data: DOMString,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<ProcessingInstruction>> {
        // Step 1. If target does not match the Name production, then throw an "InvalidCharacterError" DOMException.
        if !matches_name_production(&target) {
            return Err(Error::InvalidCharacter);
        }

        // Step 2.
        if data.contains("?>") {
            return Err(Error::InvalidCharacter);
        }

        // Step 3.
        Ok(ProcessingInstruction::new(target, data, self, can_gc))
    }

    // https://dom.spec.whatwg.org/#dom-document-importnode
    fn ImportNode(&self, node: &Node, deep: bool, can_gc: CanGc) -> Fallible<DomRoot<Node>> {
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

        Ok(Node::clone(node, Some(self), clone_children, can_gc))
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
    fn CreateEvent(&self, mut interface: DOMString, can_gc: CanGc) -> Fallible<DomRoot<Event>> {
        interface.make_ascii_lowercase();
        match &*interface {
            "beforeunloadevent" => Ok(DomRoot::upcast(BeforeUnloadEvent::new_uninitialized(
                &self.window,
                can_gc,
            ))),
            "compositionevent" | "textevent" => Ok(DomRoot::upcast(
                CompositionEvent::new_uninitialized(&self.window, can_gc),
            )),
            "customevent" => Ok(DomRoot::upcast(CustomEvent::new_uninitialized(
                self.window.upcast(),
                can_gc,
            ))),
            // FIXME(#25136): devicemotionevent, deviceorientationevent
            // FIXME(#7529): dragevent
            "events" | "event" | "htmlevents" | "svgevents" => {
                Ok(Event::new_uninitialized(self.window.upcast(), can_gc))
            },
            "focusevent" => Ok(DomRoot::upcast(FocusEvent::new_uninitialized(
                &self.window,
                can_gc,
            ))),
            "hashchangeevent" => Ok(DomRoot::upcast(HashChangeEvent::new_uninitialized(
                &self.window,
                can_gc,
            ))),
            "keyboardevent" => Ok(DomRoot::upcast(KeyboardEvent::new_uninitialized(
                &self.window,
                can_gc,
            ))),
            "messageevent" => Ok(DomRoot::upcast(MessageEvent::new_uninitialized(
                self.window.upcast(),
                can_gc,
            ))),
            "mouseevent" | "mouseevents" => Ok(DomRoot::upcast(MouseEvent::new_uninitialized(
                &self.window,
                can_gc,
            ))),
            "storageevent" => Ok(DomRoot::upcast(StorageEvent::new_uninitialized(
                &self.window,
                "".into(),
                can_gc,
            ))),
            "touchevent" => Ok(DomRoot::upcast(DomTouchEvent::new_uninitialized(
                &self.window,
                &TouchList::new(&self.window, &[], can_gc),
                &TouchList::new(&self.window, &[], can_gc),
                &TouchList::new(&self.window, &[], can_gc),
                can_gc,
            ))),
            "uievent" | "uievents" => Ok(DomRoot::upcast(UIEvent::new_uninitialized(
                &self.window,
                can_gc,
            ))),
            _ => Err(Error::NotSupported),
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-lastmodified
    fn LastModified(&self) -> DOMString {
        DOMString::from(self.last_modified.as_ref().cloned().unwrap_or_else(|| {
            // Ideally this would get the local time using `time`, but `time` always fails to get the local
            // timezone on Unix unless the application is single threaded unless the library is explicitly
            // set to "unsound" mode. Maybe that's fine, but it needs more investigation. see
            // https://nvd.nist.gov/vuln/detail/CVE-2020-26235
            // When `time` supports a thread-safe way of getting the local time zone we could use it here.
            Local::now().format("%m/%d/%Y %H:%M:%S").to_string()
        }))
    }

    // https://dom.spec.whatwg.org/#dom-document-createrange
    fn CreateRange(&self, can_gc: CanGc) -> DomRoot<Range> {
        Range::new_with_doc(self, None, can_gc)
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
        self.title().unwrap_or_else(|| DOMString::from(""))
    }

    // https://html.spec.whatwg.org/multipage/#document.title
    fn SetTitle(&self, title: DOMString, can_gc: CanGc) {
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
                        None,
                        can_gc,
                    );
                    let parent = root.upcast::<Node>();
                    let child = elem.upcast::<Node>();
                    parent
                        .InsertBefore(child, parent.GetFirstChild().as_deref())
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
                            None,
                            can_gc,
                        );
                        head.upcast::<Node>().AppendChild(elem.upcast()).unwrap()
                    },
                    None => return,
                },
            }
        } else {
            return;
        };

        elem.SetTextContent(Some(title), can_gc);
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
                .find(|child| {
                    matches!(
                        child.type_id(),
                        NodeTypeId::Element(ElementTypeId::HTMLElement(
                            HTMLElementTypeId::HTMLBodyElement,
                        )) | NodeTypeId::Element(ElementTypeId::HTMLElement(
                            HTMLElementTypeId::HTMLFrameSetElement,
                        ))
                    )
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
        if old_body.as_deref() == Some(new_body) {
            return Ok(());
        }

        match (self.GetDocumentElement(), &old_body) {
            // Step 3.
            (Some(ref root), Some(child)) => {
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
        NodeList::new_elements_by_name_list(self.window(), self, name)
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-images
    fn Images(&self) -> DomRoot<HTMLCollection> {
        self.images.or_init(|| {
            HTMLCollection::new_with_filter_fn(&self.window, self.upcast(), |element, _| {
                element.is::<HTMLImageElement>()
            })
        })
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-embeds
    fn Embeds(&self) -> DomRoot<HTMLCollection> {
        self.embeds.or_init(|| {
            HTMLCollection::new_with_filter_fn(&self.window, self.upcast(), |element, _| {
                element.is::<HTMLEmbedElement>()
            })
        })
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-plugins
    fn Plugins(&self) -> DomRoot<HTMLCollection> {
        self.Embeds()
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-links
    fn Links(&self) -> DomRoot<HTMLCollection> {
        self.links.or_init(|| {
            HTMLCollection::new_with_filter_fn(&self.window, self.upcast(), |element, _| {
                (element.is::<HTMLAnchorElement>() || element.is::<HTMLAreaElement>()) &&
                    element.has_attribute(&local_name!("href"))
            })
        })
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-forms
    fn Forms(&self) -> DomRoot<HTMLCollection> {
        self.forms.or_init(|| {
            HTMLCollection::new_with_filter_fn(&self.window, self.upcast(), |element, _| {
                element.is::<HTMLFormElement>()
            })
        })
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-scripts
    fn Scripts(&self) -> DomRoot<HTMLCollection> {
        self.scripts.or_init(|| {
            HTMLCollection::new_with_filter_fn(&self.window, self.upcast(), |element, _| {
                element.is::<HTMLScriptElement>()
            })
        })
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-anchors
    fn Anchors(&self) -> DomRoot<HTMLCollection> {
        self.anchors.or_init(|| {
            HTMLCollection::new_with_filter_fn(&self.window, self.upcast(), |element, _| {
                element.is::<HTMLAnchorElement>() && element.has_attribute(&local_name!("href"))
            })
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
    fn Prepend(&self, nodes: Vec<NodeOrString>, can_gc: CanGc) -> ErrorResult {
        self.upcast::<Node>().prepend(nodes, can_gc)
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-append
    fn Append(&self, nodes: Vec<NodeOrString>, can_gc: CanGc) -> ErrorResult {
        self.upcast::<Node>().append(nodes, can_gc)
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-replacechildren
    fn ReplaceChildren(&self, nodes: Vec<NodeOrString>, can_gc: CanGc) -> ErrorResult {
        self.upcast::<Node>().replace_children(nodes, can_gc)
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
            .as_global_scope()
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
            .as_global_scope()
            .resource_threads()
            .send(SetCookiesForUrl(self.url(), cookies, NonHTTP));
        Ok(())
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-bgcolor
    fn BgColor(&self) -> DOMString {
        self.get_body_attribute(&local_name!("bgcolor"))
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-bgcolor
    fn SetBgColor(&self, value: DOMString, can_gc: CanGc) {
        self.set_body_attribute(&local_name!("bgcolor"), value, can_gc)
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-fgcolor
    fn FgColor(&self) -> DOMString {
        self.get_body_attribute(&local_name!("text"))
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-fgcolor
    fn SetFgColor(&self, value: DOMString, can_gc: CanGc) {
        self.set_body_attribute(&local_name!("text"), value, can_gc)
    }

    #[allow(unsafe_code)]
    /// <https://html.spec.whatwg.org/multipage/#dom-tree-accessors:dom-document-nameditem-filter>
    fn NamedGetter(&self, name: DOMString) -> Option<NamedPropertyValue> {
        if name.is_empty() {
            return None;
        }
        let name = Atom::from(name);

        // Step 1.
        let elements_with_name = self.get_elements_with_name(&name);
        let name_iter = elements_with_name
            .iter()
            .filter(|elem| is_named_element_with_name_attribute(elem));
        let elements_with_id = self.get_elements_with_id(&name);
        let id_iter = elements_with_id
            .iter()
            .filter(|elem| is_named_element_with_id_attribute(elem));
        let mut elements = name_iter.chain(id_iter);

        let first = elements.next()?;

        if elements.next().is_none() {
            // Step 2.
            if let Some(nested_window_proxy) = first
                .downcast::<HTMLIFrameElement>()
                .and_then(|iframe| iframe.GetContentWindow())
            {
                return Some(NamedPropertyValue::WindowProxy(nested_window_proxy));
            }

            // Step 3.
            return Some(NamedPropertyValue::Element(DomRoot::from_ref(first)));
        }

        // Step 4.
        #[derive(JSTraceable, MallocSizeOf)]
        struct DocumentNamedGetter {
            #[no_trace]
            name: Atom,
        }
        impl CollectionFilter for DocumentNamedGetter {
            fn filter(&self, elem: &Element, _root: &Node) -> bool {
                let type_ = match elem.upcast::<Node>().type_id() {
                    NodeTypeId::Element(ElementTypeId::HTMLElement(type_)) => type_,
                    _ => return false,
                };
                match type_ {
                    HTMLElementTypeId::HTMLFormElement | HTMLElementTypeId::HTMLIFrameElement => {
                        elem.get_name().as_ref() == Some(&self.name)
                    },
                    HTMLElementTypeId::HTMLImageElement => elem.get_name().is_some_and(|name| {
                        name == *self.name ||
                            !name.is_empty() && elem.get_id().as_ref() == Some(&self.name)
                    }),
                    // TODO handle <embed> and <object>; these depend on whether the element is
                    // “exposed”, a concept that doesn’t fully make sense until embed/object
                    // behaviour is actually implemented
                    _ => false,
                }
            }
        }
        let collection = HTMLCollection::create(
            self.window(),
            self.upcast(),
            Box::new(DocumentNamedGetter { name }),
        );
        Some(NamedPropertyValue::HTMLCollection(collection))
    }

    // https://html.spec.whatwg.org/multipage/#dom-tree-accessors:supported-property-names
    fn SupportedPropertyNames(&self) -> Vec<DOMString> {
        let mut names_with_first_named_element_map: HashMap<&Atom, &Element> = HashMap::new();

        let name_map = self.name_map.borrow();
        for (name, elements) in &(name_map).0 {
            if name.is_empty() {
                continue;
            }
            let mut name_iter = elements
                .iter()
                .filter(|elem| is_named_element_with_name_attribute(elem));
            if let Some(first) = name_iter.next() {
                names_with_first_named_element_map.insert(name, first);
            }
        }
        let id_map = self.id_map.borrow();
        for (id, elements) in &(id_map).0 {
            if id.is_empty() {
                continue;
            }
            let mut id_iter = elements
                .iter()
                .filter(|elem| is_named_element_with_id_attribute(elem));
            if let Some(first) = id_iter.next() {
                match names_with_first_named_element_map.entry(id) {
                    Vacant(entry) => drop(entry.insert(first)),
                    Occupied(mut entry) => {
                        if first.upcast::<Node>().is_before(entry.get().upcast()) {
                            *entry.get_mut() = first;
                        }
                    },
                }
            }
        }

        let mut names_with_first_named_element_vec: Vec<(&Atom, &Element)> =
            names_with_first_named_element_map
                .iter()
                .map(|(k, v)| (*k, *v))
                .collect();
        names_with_first_named_element_vec.sort_unstable_by(|a, b| {
            if a.1 == b.1 {
                // This can happen if an img has an id different from its name,
                // spec does not say which string to put first.
                a.0.cmp(b.0)
            } else if a.1.upcast::<Node>().is_before(b.1.upcast::<Node>()) {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        });

        names_with_first_named_element_vec
            .iter()
            .map(|(k, _v)| DOMString::from(&***k))
            .collect()
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
    fn ElementFromPoint(
        &self,
        x: Finite<f64>,
        y: Finite<f64>,
        can_gc: CanGc,
    ) -> Option<DomRoot<Element>> {
        self.document_or_shadow_root.element_from_point(
            x,
            y,
            self.GetDocumentElement(),
            self.has_browsing_context,
            can_gc,
        )
    }

    // https://drafts.csswg.org/cssom-view/#dom-document-elementsfrompoint
    fn ElementsFromPoint(
        &self,
        x: Finite<f64>,
        y: Finite<f64>,
        can_gc: CanGc,
    ) -> Vec<DomRoot<Element>> {
        self.document_or_shadow_root.elements_from_point(
            x,
            y,
            self.GetDocumentElement(),
            self.has_browsing_context,
            can_gc,
        )
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-open
    fn Open(
        &self,
        _unused1: Option<DOMString>,
        _unused2: Option<DOMString>,
        can_gc: CanGc,
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
        if !self.origin.same_origin(&entry_responsible_document.origin) {
            return Err(Error::Security);
        }

        // Step 5
        if self
            .get_current_parser()
            .is_some_and(|parser| parser.is_active())
        {
            return Ok(DomRoot::from_ref(self));
        }

        // Step 6
        if self.is_prompting_or_unloading() {
            return Ok(DomRoot::from_ref(self));
        }

        // Step 7
        if self.active_parser_was_aborted.get() {
            return Ok(DomRoot::from_ref(self));
        }

        // TODO: prompt to unload.
        // TODO: set unload_event_start and unload_event_end

        self.window().set_navigation_start();

        // Step 8
        // TODO: https://github.com/servo/servo/issues/21937
        if self.has_browsing_context() {
            // spec says "stop document loading",
            // which is a process that does more than just abort
            self.abort(can_gc);
        }

        // Step 9
        for node in self
            .upcast::<Node>()
            .traverse_preorder(ShadowIncluding::Yes)
        {
            node.upcast::<EventTarget>().remove_all_listeners();
        }

        // Step 10
        if self.window.Document() == DomRoot::from_ref(self) {
            self.window.upcast::<EventTarget>().remove_all_listeners();
        }

        // Step 11. Replace all with null within document.
        Node::replace_all(None, self.upcast::<Node>());

        // Specs and tests are in a state of flux about whether
        // we want to clear the selection when we remove the contents;
        // WPT selection/Document-open.html wants us to not clear it
        // as of Feb 1 2020

        // Step 12. If document is fully active, then:
        if self.is_fully_active() {
            // Step 12.1. Let newURL be a copy of entryDocument's URL.
            let mut new_url = entry_responsible_document.url();

            // Step 12.2. If entryDocument is not document, then set newURL's fragment to null.
            if entry_responsible_document != DomRoot::from_ref(self) {
                new_url.set_fragment(None);
            }

            // Step 12.3. Run the URL and history update steps with document and newURL.
            // TODO: https://github.com/servo/servo/issues/21939
            self.set_url(new_url);
        }

        // Step 13. Set document's is initial about:blank to false.
        self.is_initial_about_blank.set(false);

        // Step 14. If document's iframe load in progress flag is set, then set document's mute
        // iframe load flag.
        // TODO: https://github.com/servo/servo/issues/21938

        // Step 15: Set document to no-quirks mode.
        self.set_quirks_mode(QuirksMode::NoQuirks);

        // Step 16. Create a new HTML parser and associate it with document. This is a
        // script-created parser (meaning that it can be closed by the document.open() and
        // document.close() methods, and that the tokenizer will wait for an explicit call to
        // document.close() before emitting an end-of-file token). The encoding confidence is
        // irrelevant.
        let resource_threads = self.window.as_global_scope().resource_threads().clone();
        *self.loader.borrow_mut() =
            DocumentLoader::new_with_threads(resource_threads, Some(self.url()));
        ServoParser::parse_html_script_input(self, self.url());

        // Step 17. Set the insertion point to point at just before the end of the input stream
        // (which at this point will be empty).
        // Handled when creating the parser in step 16

        // Step 18. Update the current document readiness of document to "loading".
        self.ready_state.set(DocumentReadyState::Loading);

        // Step 19. Return document.
        Ok(DomRoot::from_ref(self))
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-open-window
    fn Open_(
        &self,
        url: USVString,
        target: DOMString,
        features: DOMString,
        can_gc: CanGc,
    ) -> Fallible<Option<DomRoot<WindowProxy>>> {
        self.browsing_context()
            .ok_or(Error::InvalidAccess)?
            .open(url, target, features, can_gc)
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-write
    fn Write(&self, text: Vec<DOMString>, can_gc: CanGc) -> ErrorResult {
        if !self.is_html_document() {
            // Step 1.
            return Err(Error::InvalidState);
        }

        // Step 2.
        if self.throw_on_dynamic_markup_insertion_counter.get() > 0 {
            return Err(Error::InvalidState);
        }

        // Step 3 - what specifies the is_active() part here?
        if !self.is_active() || self.active_parser_was_aborted.get() {
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
                self.Open(None, None, can_gc)?;
                self.get_current_parser().unwrap()
            },
        };

        // Step 7.
        // TODO: handle reload override buffer.

        // Steps 6-8.
        parser.write(text, can_gc);

        // Step 9.
        Ok(())
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-writeln
    fn Writeln(&self, mut text: Vec<DOMString>, can_gc: CanGc) -> ErrorResult {
        text.push("\n".into());
        self.Write(text, can_gc)
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-close
    fn Close(&self, can_gc: CanGc) -> ErrorResult {
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
        parser.close(can_gc);

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
    fn ExitFullscreen(&self, can_gc: CanGc) -> Rc<Promise> {
        self.exit_fullscreen(can_gc)
    }

    // check-tidy: no specs after this line
    // Servo only API to get an instance of the controls of a specific
    // media element matching the given id.
    fn ServoGetMediaControls(&self, id: DOMString) -> Fallible<DomRoot<ShadowRoot>> {
        match self.media_controls.borrow().get(&*id) {
            Some(m) => Ok(DomRoot::from_ref(m)),
            None => Err(Error::InvalidAccess),
        }
    }

    // https://w3c.github.io/selection-api/#dom-document-getselection
    fn GetSelection(&self) -> Option<DomRoot<Selection>> {
        if self.has_browsing_context {
            Some(
                self.selection
                    .or_init(|| Selection::new(self, CanGc::note())),
            )
        } else {
            None
        }
    }

    // https://drafts.csswg.org/css-font-loading/#font-face-source
    fn Fonts(&self, can_gc: CanGc) -> DomRoot<FontFaceSet> {
        self.fonts
            .or_init(|| FontFaceSet::new(&self.global(), None, can_gc))
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-document-hidden>
    fn Hidden(&self) -> bool {
        self.visibility_state.get() == DocumentVisibilityState::Hidden
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-document-visibilitystate>
    fn VisibilityState(&self) -> DocumentVisibilityState {
        self.visibility_state.get()
    }

    fn CreateExpression(
        &self,
        expression: DOMString,
        resolver: Option<Rc<XPathNSResolver>>,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<super::types::XPathExpression>> {
        let global = self.global();
        let window = global.as_window();
        let evaluator = XPathEvaluator::new(window, None, can_gc);
        XPathEvaluatorMethods::<crate::DomTypeHolder>::CreateExpression(
            &*evaluator,
            expression,
            resolver,
            can_gc,
        )
    }

    fn CreateNSResolver(&self, node_resolver: &Node, can_gc: CanGc) -> DomRoot<Node> {
        let global = self.global();
        let window = global.as_window();
        let evaluator = XPathEvaluator::new(window, None, can_gc);
        XPathEvaluatorMethods::<crate::DomTypeHolder>::CreateNSResolver(&*evaluator, node_resolver)
    }

    fn Evaluate(
        &self,
        expression: DOMString,
        context_node: &Node,
        resolver: Option<Rc<XPathNSResolver>>,
        type_: u16,
        result: Option<&super::types::XPathResult>,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<super::types::XPathResult>> {
        let global = self.global();
        let window = global.as_window();
        let evaluator = XPathEvaluator::new(window, None, can_gc);
        XPathEvaluatorMethods::<crate::DomTypeHolder>::Evaluate(
            &*evaluator,
            expression,
            context_node,
            resolver,
            type_,
            result,
            can_gc,
        )
    }
}

fn update_with_current_instant(marker: &Cell<Option<CrossProcessInstant>>) {
    if marker.get().is_none() {
        marker.set(Some(CrossProcessInstant::now()))
    }
}

/// <https://w3c.github.io/webappsec-referrer-policy/#determine-policy-for-token>
pub(crate) fn determine_policy_for_token(token: &str) -> ReferrerPolicy {
    match_ignore_ascii_case! { token,
        "never" | "no-referrer" => ReferrerPolicy::NoReferrer,
        "no-referrer-when-downgrade" => ReferrerPolicy::NoReferrerWhenDowngrade,
        "origin" => ReferrerPolicy::Origin,
        "same-origin" => ReferrerPolicy::SameOrigin,
        "strict-origin" => ReferrerPolicy::StrictOrigin,
        "default" | "strict-origin-when-cross-origin" => ReferrerPolicy::StrictOriginWhenCrossOrigin,
        "origin-when-cross-origin" => ReferrerPolicy::OriginWhenCrossOrigin,
        "always" | "unsafe-url" => ReferrerPolicy::UnsafeUrl,
        _ => ReferrerPolicy::EmptyString,
    }
}

/// Specifies the type of focus event that is sent to a pipeline
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum FocusType {
    Element, // The first focus message - focus the element itself
    Parent,  // Focusing a parent element (an iframe)
}

/// Focus events
pub(crate) enum FocusEventType {
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
pub(crate) struct FakeRequestAnimationFrameCallback {
    /// The document.
    #[ignore_malloc_size_of = "non-owning"]
    document: Trusted<Document>,
}

impl FakeRequestAnimationFrameCallback {
    pub(crate) fn invoke(self, can_gc: CanGc) {
        // TODO: Once there is a more generic mechanism to trigger `update_the_rendering` when
        // not driven by the compositor, it should be used here.
        self.document
            .root()
            .note_pending_animation_tick(AnimationTickType::REQUEST_ANIMATION_FRAME);
        with_script_thread(|script_thread| script_thread.update_the_rendering(false, can_gc))
    }
}

#[derive(JSTraceable, MallocSizeOf)]
pub(crate) enum AnimationFrameCallback {
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
                let devtools_sender = document.window().as_global_scope().devtools_chan().unwrap();
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
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
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
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
struct PendingScript {
    element: Dom<HTMLScriptElement>,
    // TODO(sagudev): could this be all no_trace?
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum ReflowTriggerCondition {
    StylesheetsChanged,
    DirtyDescendants,
    PendingRestyles,
    PaintPostponed,
}

fn is_named_element_with_name_attribute(elem: &Element) -> bool {
    let type_ = match elem.upcast::<Node>().type_id() {
        NodeTypeId::Element(ElementTypeId::HTMLElement(type_)) => type_,
        _ => return false,
    };
    match type_ {
        HTMLElementTypeId::HTMLFormElement |
        HTMLElementTypeId::HTMLIFrameElement |
        HTMLElementTypeId::HTMLImageElement => true,
        // TODO handle <embed> and <object>; these depend on whether the element is
        // “exposed”, a concept that doesn’t fully make sense until embed/object
        // behaviour is actually implemented
        _ => false,
    }
}

fn is_named_element_with_id_attribute(elem: &Element) -> bool {
    // TODO handle <embed> and <object>; these depend on whether the element is
    // “exposed”, a concept that doesn’t fully make sense until embed/object
    // behaviour is actually implemented
    elem.is::<HTMLImageElement>() && elem.get_name().is_some_and(|name| !name.is_empty())
}

pub(crate) trait DocumentHelpers<D: DomTypes> {
    fn ensure_safe_to_run_script_or_layout(&self);
}

impl DocumentHelpers<crate::DomTypeHolder> for Document {
    fn ensure_safe_to_run_script_or_layout(&self) {
        Document::ensure_safe_to_run_script_or_layout(self)
    }
}
