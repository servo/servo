/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::{Cell, RefCell};
use std::cmp::Ordering;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::{HashMap, HashSet, VecDeque};
use std::default::Default;
use std::ops::Deref;
use std::rc::Rc;
use std::str::FromStr;
use std::sync::{LazyLock, Mutex};
use std::time::Duration;

use base::cross_process_instant::CrossProcessInstant;
use base::generic_channel::GenericSend;
use base::id::WebViewId;
use base::{Epoch, generic_channel};
use bitflags::bitflags;
use chrono::Local;
use constellation_traits::{NavigationHistoryBehavior, ScriptToConstellationMessage};
use content_security_policy::sandboxing_directive::SandboxingFlagSet;
use content_security_policy::{CspList, PolicyDisposition};
use cookie::Cookie;
use data_url::mime::Mime;
use devtools_traits::ScriptToDevtoolsControlMsg;
use dom_struct::dom_struct;
use embedder_traits::{
    AllowOrDeny, AnimationState, CustomHandlersAutomationMode, EmbedderMsg, FocusSequenceNumber,
    Image, LoadStatus,
};
use encoding_rs::{Encoding, UTF_8};
use fonts::WebFontDocumentContext;
use html5ever::{LocalName, Namespace, QualName, local_name, ns};
use hyper_serde::Serde;
use js::rust::{HandleObject, HandleValue, MutableHandleValue};
use layout_api::{
    PendingRestyle, ReflowGoal, ReflowPhasesRun, ReflowStatistics, RestyleReason,
    ScrollContainerQueryFlags, TrustedNodeAddress,
};
use metrics::{InteractiveFlag, InteractiveWindow, ProgressiveWebMetrics};
use net_traits::CookieSource::NonHTTP;
use net_traits::CoreResourceMsg::{GetCookiesForUrl, SetCookiesForUrl};
use net_traits::ReferrerPolicy;
use net_traits::policy_container::PolicyContainer;
use net_traits::pub_domains::is_pub_domain;
use net_traits::request::{
    InsecureRequestsPolicy, PreloadId, PreloadKey, PreloadedResources, RequestBuilder,
};
use net_traits::response::HttpsState;
use percent_encoding::percent_decode;
use profile_traits::ipc as profile_ipc;
use profile_traits::time::TimerMetadataFrameType;
use regex::bytes::Regex;
use rustc_hash::{FxBuildHasher, FxHashMap};
use script_bindings::interfaces::DocumentHelpers;
use script_bindings::script_runtime::JSContext;
use script_traits::{DocumentActivity, ProgressiveWebMetricType};
use servo_arc::Arc;
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
use stylo_atoms::Atom;
use url::{Host, Position};

use crate::animation_timeline::AnimationTimeline;
use crate::animations::Animations;
use crate::document_loader::{DocumentLoader, LoadType};
use crate::dom::attr::Attr;
use crate::dom::beforeunloadevent::BeforeUnloadEvent;
use crate::dom::bindings::callback::ExceptionHandling;
use crate::dom::bindings::cell::{DomRefCell, Ref, RefMut};
use crate::dom::bindings::codegen::Bindings::BeforeUnloadEventBinding::BeforeUnloadEvent_Binding::BeforeUnloadEventMethods;
use crate::dom::bindings::codegen::Bindings::DocumentBinding::{
    DocumentMethods, DocumentReadyState, DocumentVisibilityState, NamedPropertyValue,
};
use crate::dom::bindings::codegen::Bindings::ElementBinding::ScrollLogicalPosition;
use crate::dom::bindings::codegen::Bindings::EventBinding::Event_Binding::EventMethods;
use crate::dom::bindings::codegen::Bindings::HTMLIFrameElementBinding::HTMLIFrameElement_Binding::HTMLIFrameElementMethods;
use crate::dom::bindings::codegen::Bindings::HTMLOrSVGElementBinding::FocusOptions;
#[cfg(any(feature = "webxr", feature = "gamepad"))]
use crate::dom::bindings::codegen::Bindings::NavigatorBinding::Navigator_Binding::NavigatorMethods;
use crate::dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use crate::dom::bindings::codegen::Bindings::NodeFilterBinding::NodeFilter;
use crate::dom::bindings::codegen::Bindings::PerformanceBinding::PerformanceMethods;
use crate::dom::bindings::codegen::Bindings::PermissionStatusBinding::PermissionName;
use crate::dom::bindings::codegen::Bindings::WindowBinding::{
    FrameRequestCallback, ScrollBehavior, WindowMethods,
};
use crate::dom::bindings::codegen::Bindings::XPathEvaluatorBinding::XPathEvaluatorMethods;
use crate::dom::bindings::codegen::Bindings::XPathNSResolverBinding::XPathNSResolver;
use crate::dom::bindings::codegen::UnionTypes::{
    BooleanOrImportNodeOptions, NodeOrString, StringOrElementCreationOptions, TrustedHTMLOrString,
};
use crate::dom::bindings::domname::{
    self, is_valid_attribute_local_name, is_valid_element_local_name, namespace_from_domstring,
};
use crate::dom::bindings::error::{Error, ErrorInfo, ErrorResult, Fallible};
use crate::dom::bindings::frozenarray::CachedFrozenArray;
use crate::dom::bindings::inheritance::{Castable, ElementTypeId, HTMLElementTypeId, NodeTypeId};
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::refcounted::{Trusted, TrustedPromise};
use crate::dom::bindings::reflector::{DomGlobal, reflect_dom_object_with_proto};
use crate::dom::bindings::root::{Dom, DomRoot, LayoutDom, MutNullableDom, ToLayout};
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::bindings::trace::{HashMapTracedValues, NoTrace};
use crate::dom::bindings::weakref::DOMTracker;
use crate::dom::bindings::xmlname::matches_name_production;
use crate::dom::cdatasection::CDATASection;
use crate::dom::comment::Comment;
use crate::dom::compositionevent::CompositionEvent;
use crate::dom::css::cssstylesheet::CSSStyleSheet;
use crate::dom::css::fontfaceset::FontFaceSet;
use crate::dom::css::stylesheetlist::{StyleSheetList, StyleSheetListOwner};
use crate::dom::customelementregistry::{
    CustomElementDefinition, CustomElementReactionStack, CustomElementRegistry,
};
use crate::dom::customevent::CustomEvent;
use crate::dom::document_embedder_controls::DocumentEmbedderControls;
use crate::dom::document_event_handler::DocumentEventHandler;
use crate::dom::documentfragment::DocumentFragment;
use crate::dom::documentorshadowroot::{
    DocumentOrShadowRoot, ServoStylesheetInDocument, StylesheetSource,
};
use crate::dom::documenttype::DocumentType;
use crate::dom::domimplementation::DOMImplementation;
use crate::dom::element::{
    CustomElementCreationMode, Element, ElementCreator, ElementPerformFullscreenEnter,
    ElementPerformFullscreenExit,
};
use crate::dom::event::{Event, EventBubbles, EventCancelable};
use crate::dom::eventtarget::EventTarget;
use crate::dom::execcommand::contenteditable::ContentEditableRange;
use crate::dom::execcommand::execcommands::DocumentExecCommandSupport;
use crate::dom::focusevent::FocusEvent;
use crate::dom::globalscope::GlobalScope;
use crate::dom::hashchangeevent::HashChangeEvent;
use crate::dom::html::htmlanchorelement::HTMLAnchorElement;
use crate::dom::html::htmlareaelement::HTMLAreaElement;
use crate::dom::html::htmlbaseelement::HTMLBaseElement;
use crate::dom::html::htmlcollection::{CollectionFilter, HTMLCollection};
use crate::dom::html::htmlelement::HTMLElement;
use crate::dom::html::htmlembedelement::HTMLEmbedElement;
use crate::dom::html::htmlformelement::{FormControl, FormControlElementHelpers, HTMLFormElement};
use crate::dom::html::htmlheadelement::HTMLHeadElement;
use crate::dom::html::htmlhtmlelement::HTMLHtmlElement;
use crate::dom::html::htmliframeelement::HTMLIFrameElement;
use crate::dom::html::htmlimageelement::HTMLImageElement;
use crate::dom::html::htmlscriptelement::{HTMLScriptElement, ScriptResult};
use crate::dom::html::htmltitleelement::HTMLTitleElement;
use crate::dom::htmldetailselement::DetailsNameGroups;
use crate::dom::intersectionobserver::IntersectionObserver;
use crate::dom::keyboardevent::KeyboardEvent;
use crate::dom::largestcontentfulpaint::LargestContentfulPaint;
use crate::dom::location::Location;
use crate::dom::messageevent::MessageEvent;
use crate::dom::mouseevent::MouseEvent;
use crate::dom::node::{Node, NodeDamage, NodeFlags, NodeTraits, ShadowIncluding};
use crate::dom::nodeiterator::NodeIterator;
use crate::dom::nodelist::NodeList;
use crate::dom::pagetransitionevent::PageTransitionEvent;
use crate::dom::performance::performanceentry::PerformanceEntry;
use crate::dom::performance::performancepainttiming::PerformancePaintTiming;
use crate::dom::processinginstruction::ProcessingInstruction;
use crate::dom::promise::Promise;
use crate::dom::range::Range;
use crate::dom::resizeobserver::{ResizeObservationDepth, ResizeObserver};
use crate::dom::scrolling_box::{ScrollAxisState, ScrollRequirement, ScrollingBox};
use crate::dom::selection::Selection;
use crate::dom::servoparser::ServoParser;
use crate::dom::shadowroot::ShadowRoot;
use crate::dom::storageevent::StorageEvent;
use crate::dom::text::Text;
use crate::dom::touchevent::TouchEvent as DomTouchEvent;
use crate::dom::touchlist::TouchList;
use crate::dom::treewalker::TreeWalker;
use crate::dom::trustedhtml::TrustedHTML;
use crate::dom::types::{HTMLCanvasElement, HTMLDialogElement, VisibilityStateEntry};
use crate::dom::uievent::UIEvent;
use crate::dom::virtualmethods::vtable_for;
use crate::dom::websocket::WebSocket;
use crate::dom::window::Window;
use crate::dom::windowproxy::WindowProxy;
use crate::dom::xpathevaluator::XPathEvaluator;
use crate::dom::xpathexpression::XPathExpression;
use crate::fetch::{DeferredFetchRecordInvokeState, FetchCanceller};
use crate::iframe_collection::IFrameCollection;
use crate::image_animation::ImageAnimationManager;
use crate::messaging::{CommonScriptMsg, MainThreadScriptMsg};
use crate::mime::{APPLICATION, CHARSET};
use crate::network_listener::{FetchResponseListener, NetworkListener};
use crate::realms::{AlreadyInRealm, InRealm, enter_realm};
use crate::script_runtime::{CanGc, ScriptThreadEventCategory};
use crate::script_thread::ScriptThread;
use crate::stylesheet_set::StylesheetSetRef;
use crate::task::NonSendTaskBox;
use crate::task_source::TaskSourceName;
use crate::timers::OneshotTimerCallback;
use crate::xpath::parse_expression;

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

#[derive(JSTraceable, MallocSizeOf)]
pub(crate) struct RefreshRedirectDue {
    #[no_trace]
    pub(crate) url: ServoUrl,
    #[ignore_malloc_size_of = "non-owning"]
    pub(crate) window: DomRoot<Window>,
}
impl RefreshRedirectDue {
    /// Step 13 of <https://html.spec.whatwg.org/multipage/#shared-declarative-refresh-steps>
    pub(crate) fn invoke(self, can_gc: CanGc) {
        // After the refresh has come due (as defined below),
        // if the user has not canceled the redirect and, if meta is given,
        // document's active sandboxing flag set does not have the sandboxed
        // automatic features browsing context flag set,
        // then navigate document's node navigable to urlRecord using document,
        // with historyHandling set to "replace".
        //
        // TODO: check sandbox
        // TODO: Check if meta was given
        let load_data = self
            .window
            .load_data_for_document(self.url.clone(), self.window.pipeline_id());
        self.window
            .load_url(NavigationHistoryBehavior::Replace, false, load_data, can_gc);
    }
}

#[derive(Clone, Copy, Debug, JSTraceable, MallocSizeOf, PartialEq)]
pub(crate) enum IsHTMLDocument {
    HTMLDocument,
    NonHTMLDocument,
}

#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
struct FocusTransaction {
    /// The focused element of this document.
    element: Option<Dom<Element>>,
    /// See [`Document::has_focus`].
    has_focus: bool,
    /// Focus options for the transaction
    focus_options: FocusOptions,
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

/// Reasons why a [`Document`] might need a rendering update that is otherwise
/// untracked via other [`Document`] properties.
#[derive(Clone, Copy, Debug, Default, JSTraceable, MallocSizeOf)]
pub(crate) struct RenderingUpdateReason(u8);

bitflags! {
    impl RenderingUpdateReason: u8 {
        /// When a `ResizeObserver` starts observing a target, this becomes true, which in turn is a
        /// signal to the [`ScriptThread`] that a rendering update should happen.
        const ResizeObserverStartedObservingTarget = 1 << 0;
        /// When an `IntersectionObserver` starts observing a target, this becomes true, which in turn is a
        /// signal to the [`ScriptThread`] that a rendering update should happen.
        const IntersectionObserverStartedObservingTarget = 1 << 1;
        /// All web fonts have loaded and `fonts.ready` promise has been fulfilled. We want to trigger
        /// one more rendering update possibility after this happens, so that any potential screenshot
        /// reflects the up-to-date contents.
        const FontReadyPromiseFulfilled = 1 << 2;
    }
}

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
    /// <https://html.spec.whatwg.org/multipage/#the-document%27s-address>
    #[no_trace]
    url: DomRefCell<ServoUrl>,
    /// <https://html.spec.whatwg.org/multipage/#concept-document-about-base-url>
    #[no_trace]
    about_base_url: DomRefCell<Option<ServoUrl>>,
    #[ignore_malloc_size_of = "defined in selectors"]
    #[no_trace]
    quirks_mode: Cell<QuirksMode>,
    /// A helper used to process and store data related to input event handling.
    event_handler: DocumentEventHandler,
    /// A helper to handle showing and hiding user interface controls in the embedding layer.
    embedder_controls: DocumentEmbedderControls,
    /// Caches for the getElement methods. It is safe to use FxHash for these maps
    /// as Atoms are `string_cache` items that will have the hash computed from a u32.
    id_map: DomRefCell<HashMapTracedValues<Atom, Vec<Dom<Element>>, FxBuildHasher>>,
    name_map: DomRefCell<HashMapTracedValues<Atom, Vec<Dom<Element>>, FxBuildHasher>>,
    tag_map: DomRefCell<HashMapTracedValues<LocalName, Dom<HTMLCollection>, FxBuildHasher>>,
    tagns_map: DomRefCell<HashMapTracedValues<QualName, Dom<HTMLCollection>, FxBuildHasher>>,
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
    stylesheets: DomRefCell<DocumentStylesheetSet<ServoStylesheetInDocument>>,
    stylesheet_list: MutNullableDom<StyleSheetList>,
    ready_state: Cell<DocumentReadyState>,
    /// Whether the DOMContentLoaded event has already been dispatched.
    domcontentloaded_dispatched: Cell<bool>,
    /// The state of this document's focus transaction.
    focus_transaction: DomRefCell<Option<FocusTransaction>>,
    /// The element that currently has the document focus context.
    focused: MutNullableDom<Element>,
    /// The last sequence number sent to the constellation.
    #[no_trace]
    focus_sequence: Cell<FocusSequenceNumber>,
    /// Indicates whether the container is included in the top-level browsing
    /// context's focus chain (not considering system focus). Permanently `true`
    /// for a top-level document.
    has_focus: Cell<bool>,
    /// The script element that is currently executing.
    current_script: MutNullableDom<HTMLScriptElement>,
    /// <https://html.spec.whatwg.org/multipage/#pending-parsing-blocking-script>
    pending_parsing_blocking_script: DomRefCell<Option<PendingScript>>,
    /// Number of stylesheets that block executing the next parser-inserted script
    script_blocking_stylesheets_count: Cell<u32>,
    /// Number of elements that block the rendering of the page.
    /// <https://html.spec.whatwg.org/multipage/#implicitly-potentially-render-blocking>
    render_blocking_element_count: Cell<u32>,
    /// <https://html.spec.whatwg.org/multipage/#list-of-scripts-that-will-execute-when-the-document-has-finished-parsing>
    deferred_scripts: PendingInOrderScriptVec,
    /// <https://html.spec.whatwg.org/multipage/#list-of-scripts-that-will-execute-in-order-as-soon-as-possible>
    asap_in_order_scripts_list: PendingInOrderScriptVec,
    /// <https://html.spec.whatwg.org/multipage/#set-of-scripts-that-will-execute-as-soon-as-possible>
    asap_scripts_set: DomRefCell<Vec<Dom<HTMLScriptElement>>>,
    /// <https://html.spec.whatwg.org/multipage/#animation-frame-callback-identifier>
    /// Current identifier of animation frame callback
    animation_frame_ident: Cell<u32>,
    /// <https://html.spec.whatwg.org/multipage/#list-of-animation-frame-callbacks>
    /// List of animation frame callbacks
    animation_frame_list: DomRefCell<VecDeque<(u32, Option<AnimationFrameCallback>)>>,
    /// Whether we're in the process of running animation callbacks.
    ///
    /// Tracking this is not necessary for correctness. Instead, it is an optimization to avoid
    /// sending needless `ChangeRunningAnimationsState` messages to `Paint`.
    running_animation_callbacks: Cell<bool>,
    /// Tracks all outstanding loads related to this document.
    loader: DomRefCell<DocumentLoader>,
    /// The current active HTML parser, to allow resuming after interruptions.
    current_parser: MutNullableDom<ServoParser>,
    /// The cached first `base` element with an `href` attribute.
    base_element: MutNullableDom<HTMLBaseElement>,
    /// The cached first `base` element, used for its target (doesn't need a href)
    target_base_element: MutNullableDom<HTMLBaseElement>,
    /// This field is set to the document itself for inert documents.
    /// <https://html.spec.whatwg.org/multipage/#appropriate-template-contents-owner-document>
    appropriate_template_contents_owner_document: MutNullableDom<Document>,
    /// Information on elements needing restyle to ship over to layout when the
    /// time comes.
    pending_restyles: DomRefCell<FxHashMap<Dom<Element>, NoTrace<PendingRestyle>>>,
    /// A collection of reasons that the [`Document`] needs to be restyled at the next
    /// opportunity for a reflow. If this is empty, then the [`Document`] does not need to
    /// be restyled.
    #[no_trace]
    needs_restyle: Cell<RestyleReason>,
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
    /// <https://html.spec.whatwg.org/multipage/#map-of-preloaded-resources>
    #[no_trace]
    preloaded_resources: DomRefCell<PreloadedResources>,
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
    /// It is safe to use FxBuildHasher here as Atoms are in the string_cache
    form_id_listener_map:
        DomRefCell<HashMapTracedValues<Atom, HashSet<Dom<Element>>, FxBuildHasher>>,
    #[no_trace]
    interactive_time: DomRefCell<ProgressiveWebMetrics>,
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
    delayed_tasks: DomRefCell<Vec<Box<dyn NonSendTaskBox>>>,
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
    /// A set of dirty HTML canvas elements that need their WebRender images updated the
    /// next time the rendering is updated.
    dirty_canvases: DomRefCell<Vec<Dom<HTMLCanvasElement>>>,
    /// Whether or not animated images need to have their contents updated.
    has_pending_animated_image_update: Cell<bool>,
    /// <https://w3c.github.io/slection-api/#dfn-selection>
    selection: MutNullableDom<Selection>,
    /// A timeline for animations which is used for synchronizing animations.
    /// <https://drafts.csswg.org/web-animations/#timeline>
    animation_timeline: DomRefCell<AnimationTimeline>,
    /// Animations for this Document
    animations: Animations,
    /// Image Animation Manager for this Document
    image_animation_manager: DomRefCell<ImageAnimationManager>,
    /// The nearest inclusive ancestors to all the nodes that require a restyle.
    dirty_root: MutNullableDom<Element>,
    /// <https://html.spec.whatwg.org/multipage/#will-declaratively-refresh>
    declarative_refresh: DomRefCell<Option<DeclarativeRefresh>>,
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
    /// <https://dom.spec.whatwg.org/#document-allow-declarative-shadow-roots>
    allow_declarative_shadow_roots: Cell<bool>,
    /// <https://w3c.github.io/webappsec-upgrade-insecure-requests/#insecure-requests-policy>
    #[no_trace]
    inherited_insecure_requests_policy: Cell<Option<InsecureRequestsPolicy>>,
    //// <https://w3c.github.io/webappsec-mixed-content/#categorize-settings-object>
    has_trustworthy_ancestor_origin: Cell<bool>,
    /// <https://w3c.github.io/IntersectionObserver/#document-intersectionobservertaskqueued>
    intersection_observer_task_queued: Cell<bool>,
    /// Active intersection observers that should be processed by this document in
    /// the update intersection observation steps.
    /// <https://w3c.github.io/IntersectionObserver/#run-the-update-intersection-observations-steps>
    /// > Let observer list be a list of all IntersectionObservers whose root is in the DOM tree of document.
    /// > For the top-level browsing context, this includes implicit root observers.
    ///
    /// Details of which document that should process an observers is discussed further at
    /// <https://github.com/w3c/IntersectionObserver/issues/525>.
    ///
    /// The lifetime of an intersection observer is specified at
    /// <https://github.com/w3c/IntersectionObserver/issues/525>.
    intersection_observers: DomRefCell<Vec<Dom<IntersectionObserver>>>,
    /// The node that is currently highlighted by the devtools
    highlighted_dom_node: MutNullableDom<Node>,
    /// The constructed stylesheet that is adopted by this [Document].
    /// <https://drafts.csswg.org/cssom/#dom-documentorshadowroot-adoptedstylesheets>
    adopted_stylesheets: DomRefCell<Vec<Dom<CSSStyleSheet>>>,
    /// Cached frozen array of [`Self::adopted_stylesheets`]
    #[ignore_malloc_size_of = "mozjs"]
    adopted_stylesheets_frozen_types: CachedFrozenArray,
    /// <https://drafts.csswg.org/cssom-view/#document-pending-scroll-event-targets>
    pending_scroll_event_targets: DomRefCell<Vec<Dom<EventTarget>>>,
    /// Other reasons that a rendering update might be required for this [`Document`].
    rendering_update_reasons: Cell<RenderingUpdateReason>,
    /// Whether or not this [`Document`] is waiting on canvas image updates. If it is
    /// waiting it will not do any new layout until the canvas images are up-to-date in
    /// the renderer.
    waiting_on_canvas_image_updates: Cell<bool>,
    /// The current rendering epoch, which is used to track updates in the renderer.
    ///
    ///   - Every display list update also advances the Epoch, so that the renderer knows
    ///     when a particular display list is ready in order to take a screenshot.
    ///   - Canvas image updates happen asynchronously and are tagged with this Epoch. Until
    ///     those asynchronous updates are complete, the `Document` will not perform any
    ///     more rendering updates.
    #[no_trace]
    current_rendering_epoch: Cell<Epoch>,
    /// The global custom element reaction stack for this script thread.
    #[conditional_malloc_size_of]
    custom_element_reaction_stack: Rc<CustomElementReactionStack>,
    #[no_trace]
    /// <https://html.spec.whatwg.org/multipage/#active-sandboxing-flag-set>,
    active_sandboxing_flag_set: Cell<SandboxingFlagSet>,
    #[no_trace]
    /// The [`SandboxingFlagSet`] use to create the browsing context for this [`Document`].
    /// These are cached here as they cannot always be retrieved readily if the owner of
    /// browsing context (either `<iframe>` or popup) might be in a different `ScriptThread`.
    ///
    /// See
    /// <https://html.spec.whatwg.org/multipage/#determining-the-creation-sandboxing-flags>.
    creation_sandboxing_flag_set: Cell<SandboxingFlagSet>,
    /// The cached favicon for that document.
    #[no_trace]
    favicon: RefCell<Option<Image>>,

    /// All websockets created that are associated with this document.
    websockets: DOMTracker<WebSocket>,

    /// <https://html.spec.whatwg.org/multipage/#details-name-group>
    details_name_groups: DomRefCell<Option<DetailsNameGroups>>,

    /// <https://html.spec.whatwg.org/multipage/#registerprotocolhandler()-automation-mode>
    #[no_trace]
    protocol_handler_automation_mode: RefCell<CustomHandlersAutomationMode>,

    /// Reflect the value of that preferences to prevent paying the cost of a RwLock access.
    layout_animations_test_enabled: bool,

    /// <https://w3c.github.io/editing/docs/execCommand/#state-override>
    state_override: Cell<bool>,

    /// <https://w3c.github.io/editing/docs/execCommand/#value-override>
    value_override: DomRefCell<Option<DOMString>>,
}

impl Document {
    /// <https://html.spec.whatwg.org/multipage/#unloading-document-cleanup-steps>
    fn unloading_cleanup_steps(&self) {
        // Step 1. Let window be document's relevant global object.
        // Step 2. For each WebSocket object webSocket whose relevant global object is window, make disappear webSocket.
        if self.close_outstanding_websockets() {
            // If this affected any WebSocket objects, then make document unsalvageable given document and "websocket".
            self.salvageable.set(false);
        }

        // Step 3. For each WebTransport object transport whose relevant global object is window, run the context cleanup steps given transport.
        // TODO

        // Step 4. If document's salvageable state is false, then:
        if !self.salvageable.get() {
            let global_scope = self.window.as_global_scope();

            // Step 4.1. For each EventSource object eventSource whose relevant global object is equal to window, forcibly close eventSource.
            global_scope.close_event_sources();

            // Step 4.2. Clear window's map of active timers.
            // TODO

            // Ensure the constellation discards all bfcache information for this document.
            let msg = ScriptToConstellationMessage::DiscardDocument;
            let _ = global_scope.script_to_constellation_chan().send(msg);
        }
    }

    pub(crate) fn track_websocket(&self, websocket: &WebSocket) {
        self.websockets.track(websocket);
    }

    fn close_outstanding_websockets(&self) -> bool {
        let mut closed_any_websocket = false;
        self.websockets.for_each(|websocket: DomRoot<WebSocket>| {
            if websocket.make_disappear() {
                closed_any_websocket = true;
            }
        });
        closed_any_websocket
    }

    pub(crate) fn note_node_with_dirty_descendants(&self, node: &Node) {
        debug_assert!(*node.owner_doc() == *self);
        if !node.is_connected() {
            return;
        }

        let parent = match node.parent_in_flat_tree() {
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
                        .inclusive_ancestors_in_flat_tree()
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
            Some(root) if root.is_connected() => root,
            _ => {
                element
                    .upcast::<Node>()
                    .set_flag(NodeFlags::HAS_DIRTY_DESCENDANTS, true);
                self.dirty_root.set(Some(element));
                return;
            },
        };

        for ancestor in element.upcast::<Node>().inclusive_ancestors_in_flat_tree() {
            if ancestor.get_flag(NodeFlags::HAS_DIRTY_DESCENDANTS) {
                return;
            }

            if ancestor.is::<Element>() {
                ancestor.set_flag(NodeFlags::HAS_DIRTY_DESCENDANTS, true);
            }
        }

        let new_dirty_root = element
            .upcast::<Node>()
            .common_ancestor_in_flat_tree(dirty_root.upcast())
            .expect("Couldn't find common ancestor");

        let mut has_dirty_descendants = true;
        for ancestor in dirty_root
            .upcast::<Node>()
            .inclusive_ancestors_in_flat_tree()
        {
            ancestor.set_flag(NodeFlags::HAS_DIRTY_DESCENDANTS, has_dirty_descendants);
            has_dirty_descendants &= *ancestor != *new_dirty_root;
        }

        self.dirty_root
            .set(Some(new_dirty_root.downcast::<Element>().unwrap()));
    }

    pub(crate) fn take_dirty_root(&self) -> Option<DomRoot<Element>> {
        self.dirty_root.take()
    }

    #[inline]
    pub(crate) fn loader(&self) -> Ref<'_, DocumentLoader> {
        self.loader.borrow()
    }

    #[inline]
    pub(crate) fn loader_mut(&self) -> RefMut<'_, DocumentLoader> {
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
        self.content_type.matches(APPLICATION, "xhtml+xml")
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

    #[inline]
    pub(crate) fn current_rendering_epoch(&self) -> Epoch {
        self.current_rendering_epoch.get()
    }

    pub(crate) fn set_activity(&self, cx: &mut js::context::JSContext, activity: DocumentActivity) {
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
            self.window().suspend(cx);
            media.suspend(&client_context_id);
            return;
        }

        self.title_changed();
        self.notify_embedder_favicon();
        self.dirty_all_nodes();
        self.window().resume(CanGc::from_cx(cx));
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

    pub(crate) fn set_protocol_handler_automation_mode(&self, mode: CustomHandlersAutomationMode) {
        *self.protocol_handler_automation_mode.borrow_mut() = mode;
    }

    /// <https://dom.spec.whatwg.org/#concept-document-url>
    pub(crate) fn url(&self) -> ServoUrl {
        self.url.borrow().clone()
    }

    pub(crate) fn set_url(&self, url: ServoUrl) {
        *self.url.borrow_mut() = url;
    }

    pub(crate) fn about_base_url(&self) -> Option<ServoUrl> {
        self.about_base_url.borrow().clone()
    }

    pub(crate) fn set_about_base_url(&self, about_base_url: Option<ServoUrl>) {
        *self.about_base_url.borrow_mut() = about_base_url;
    }

    /// <https://html.spec.whatwg.org/multipage/#fallback-base-url>
    pub(crate) fn fallback_base_url(&self) -> ServoUrl {
        let document_url = self.url();
        // Step 1: If document is an iframe srcdoc document:
        if document_url.as_str() == "about:srcdoc" {
            // Step 1.1: Assert: document's about base URL is non-null.
            // Step 1.2: Return document's about base URL.
            return self
                .about_base_url()
                .expect("about:srcdoc page should always have an about base URL");
        }

        // Step 2: If document's URL matches about:blank and document's about base URL is
        // non-null, then return document's about base URL.
        if document_url.matches_about_blank() {
            if let Some(about_base_url) = self.about_base_url() {
                return about_base_url;
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

    pub(crate) fn add_restyle_reason(&self, reason: RestyleReason) {
        self.needs_restyle.set(self.needs_restyle.get() | reason)
    }

    pub(crate) fn clear_restyle_reasons(&self) {
        self.needs_restyle.set(RestyleReason::empty());
    }

    pub(crate) fn restyle_reason(&self) -> RestyleReason {
        let mut condition = self.needs_restyle.get();
        if self.stylesheets.borrow().has_changed() {
            condition.insert(RestyleReason::StylesheetsChanged);
        }

        // FIXME: This should check the dirty bit on the document,
        // not the document element. Needs some layout changes to make
        // that workable.
        if let Some(root) = self.GetDocumentElement() {
            if root.upcast::<Node>().has_dirty_descendants() {
                condition.insert(RestyleReason::DOMChanged);
            }
        }

        if !self.pending_restyles.borrow().is_empty() {
            condition.insert(RestyleReason::PendingRestyles);
        }

        condition
    }

    /// Returns the first `base` element in the DOM that has an `href` attribute.
    pub(crate) fn base_element(&self) -> Option<DomRoot<HTMLBaseElement>> {
        self.base_element.get()
    }

    /// Returns the first `base` element in the DOM (doesn't need to have an `href` attribute).
    pub(crate) fn target_base_element(&self) -> Option<DomRoot<HTMLBaseElement>> {
        self.target_base_element.get()
    }

    /// Refresh the cached first base element in the DOM.
    pub(crate) fn refresh_base_element(&self) {
        if let Some(base_element) = self.base_element.get() {
            base_element.clear_frozen_base_url();
        }
        let new_base_element = self
            .upcast::<Node>()
            .traverse_preorder(ShadowIncluding::No)
            .filter_map(DomRoot::downcast::<HTMLBaseElement>)
            .find(|element| {
                element
                    .upcast::<Element>()
                    .has_attribute(&local_name!("href"))
            });
        if let Some(ref new_base_element) = new_base_element {
            new_base_element.set_frozen_base_url();
        }
        self.base_element.set(new_base_element.as_deref());

        let new_target_base_element = self
            .upcast::<Node>()
            .traverse_preorder(ShadowIncluding::No)
            .filter_map(DomRoot::downcast::<HTMLBaseElement>)
            .next();
        self.target_base_element
            .set(new_target_base_element.as_deref());
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
        node.dirty(NodeDamage::ContentOrHeritage);
    }

    /// Remove any existing association between the provided id and any elements in this document.
    pub(crate) fn unregister_element_id(&self, to_unregister: &Element, id: Atom, can_gc: CanGc) {
        self.document_or_shadow_root
            .unregister_named_element(&self.id_map, to_unregister, &id);
        self.reset_form_owner_for_listeners(&id, can_gc);
    }

    /// Associate an element present in this document with the provided id.
    pub(crate) fn register_element_id(&self, element: &Element, id: Atom, can_gc: CanGc) {
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
        self.reset_form_owner_for_listeners(&id, can_gc);
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

    /// <https://html.spec.whatwg.org/multipage/#find-a-potential-indicated-element>
    fn find_a_potential_indicated_element(&self, fragment: &str) -> Option<DomRoot<Element>> {
        // Step 1. If there is an element in the document tree whose root is
        // document and that has an ID equal to fragment, then return the first such element in tree order.
        // Step 3. Return null.
        self.get_element_by_id(&Atom::from(fragment))
            // Step 2. If there is an a element in the document tree whose root is
            // document that has a name attribute whose value is equal to fragment,
            // then return the first such element in tree order.
            .or_else(|| self.get_anchor_by_name(fragment))
    }

    /// Attempt to find a named element in this page's document.
    /// <https://html.spec.whatwg.org/multipage/#the-indicated-part-of-the-document>
    fn select_indicated_part(&self, fragment: &str) -> Option<DomRoot<Node>> {
        // Step 1. If document's URL does not equal url with exclude fragments set to true, then return null.
        //
        // Already handled by calling function

        // Step 2. Let fragment be url's fragment.
        //
        // Already handled by calling function

        // Step 3. If fragment is the empty string, then return the special value top of the document.
        if fragment.is_empty() {
            return Some(DomRoot::from_ref(self.upcast()));
        }
        // Step 4. Let potentialIndicatedElement be the result of finding a potential indicated element given document and fragment.
        if let Some(potential_indicated_element) = self.find_a_potential_indicated_element(fragment)
        {
            // Step 5. If potentialIndicatedElement is not null, then return potentialIndicatedElement.
            return Some(DomRoot::upcast(potential_indicated_element));
        }
        // Step 6. Let fragmentBytes be the result of percent-decoding fragment.
        let fragment_bytes = percent_decode(fragment.as_bytes());
        // Step 7. Let decodedFragment be the result of running UTF-8 decode without BOM on fragmentBytes.
        let Ok(decoded_fragment) = fragment_bytes.decode_utf8() else {
            return None;
        };
        // Step 8. Set potentialIndicatedElement to the result of finding a potential indicated element given document and decodedFragment.
        if let Some(potential_indicated_element) =
            self.find_a_potential_indicated_element(&decoded_fragment)
        {
            // Step 9. If potentialIndicatedElement is not null, then return potentialIndicatedElement.
            return Some(DomRoot::upcast(potential_indicated_element));
        }
        // Step 10. If decodedFragment is an ASCII case-insensitive match for the string top, then return the top of the document.
        if decoded_fragment.eq_ignore_ascii_case("top") {
            return Some(DomRoot::from_ref(self.upcast()));
        }
        // Step 11. Return null.
        None
    }

    /// <https://html.spec.whatwg.org/multipage/#scroll-to-the-fragment-identifier>
    pub(crate) fn scroll_to_the_fragment(&self, fragment: &str) {
        // Step 1. If document's indicated part is null, then set document's target element to null.
        //
        // > For an HTML document document, its indicated part is the result of
        // > selecting the indicated part given document and document's URL.
        let Some(indicated_part) = self.select_indicated_part(fragment) else {
            self.set_target_element(None);
            return;
        };
        // Step 2. Otherwise, if document's indicated part is top of the document, then:
        if *indicated_part == *self.upcast() {
            // Step 2.1. Set document's target element to null.
            self.set_target_element(None);
            // Step 2.2. Scroll to the beginning of the document for document. [CSSOMVIEW]
            //
            // FIXME(stshine): this should be the origin of the stacking context space,
            // which may differ under the influence of writing mode.
            self.window.scroll(0.0, 0.0, ScrollBehavior::Instant);
            // Step 2.3. Return.
            return;
        }
        // Step 3. Otherwise:
        // Step 3.2. Let target be document's indicated part.
        let Some(target) = indicated_part.downcast::<Element>() else {
            // Step 3.1. Assert: document's indicated part is an element.
            unreachable!("Indicated part should always be an element");
        };
        // Step 3.3. Set document's target element to target.
        self.set_target_element(Some(target));
        // Step 3.4. Run the ancestor revealing algorithm on target.
        // TODO
        // Step 3.5. Scroll target into view, with behavior set to "auto", block set to "start", and inline set to "nearest". [CSSOMVIEW]
        target.scroll_into_view_with_options(
            ScrollBehavior::Auto,
            ScrollAxisState::new_always_scroll_position(ScrollLogicalPosition::Start),
            ScrollAxisState::new_always_scroll_position(ScrollLogicalPosition::Nearest),
            None,
            None,
        );
        // Step 3.6. Run the focusing steps for target, with the Document's viewport as the fallback target.
        // TODO
        // Step 3.7. Move the sequential focus navigation starting point to target.
        // TODO
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
    /// <https://html.spec.whatwg.org/multipage/#concept-n-script>
    pub(crate) fn scripting_enabled(&self) -> bool {
        // Scripting is enabled for a node node if node's node document's browsing context is non-null,
        // and scripting is enabled for node's relevant settings object.
        self.has_browsing_context() &&
        // Either settings's global object is not a Window object,
        // or settings's global object's associated Document's active sandboxing flag
        // set does not have its sandboxed scripts browsing context flag set.
            !self.has_active_sandboxing_flag(
                SandboxingFlagSet::SANDBOXED_SCRIPTS_BROWSING_CONTEXT_FLAG,
            )
    }

    /// Return the element that currently has focus.
    // https://w3c.github.io/uievents/#events-focusevent-doc-focus
    pub(crate) fn get_focused_element(&self) -> Option<DomRoot<Element>> {
        self.focused.get()
    }

    /// Get the last sequence number sent to the constellation.
    ///
    /// Received focus-related messages with sequence numbers less than the one
    /// returned by this method must be discarded.
    pub fn get_focus_sequence(&self) -> FocusSequenceNumber {
        self.focus_sequence.get()
    }

    /// Generate the next sequence number for focus-related messages.
    fn increment_fetch_focus_sequence(&self) -> FocusSequenceNumber {
        self.focus_sequence.set(FocusSequenceNumber(
            self.focus_sequence
                .get()
                .0
                .checked_add(1)
                .expect("too many focus messages have been sent"),
        ));
        self.focus_sequence.get()
    }

    pub(crate) fn has_focus_transaction(&self) -> bool {
        self.focus_transaction.borrow().is_some()
    }

    /// Initiate a new round of checking for elements requesting focus. The last element to call
    /// `request_focus` before `commit_focus_transaction` is called will receive focus.
    pub(crate) fn begin_focus_transaction(&self) {
        // Initialize it with the current state
        *self.focus_transaction.borrow_mut() = Some(FocusTransaction {
            element: self.focused.get().as_deref().map(Dom::from_ref),
            has_focus: self.has_focus.get(),
            focus_options: FocusOptions {
                preventScroll: true,
            },
        });
    }

    /// <https://html.spec.whatwg.org/multipage/#focus-fixup-rule>
    pub(crate) fn perform_focus_fixup_rule(&self, not_focusable: &Element, can_gc: CanGc) {
        // Return if `not_focusable` is not the designated focused area of the
        // `Document`.
        if Some(not_focusable) != self.focused.get().as_deref() {
            return;
        }

        let implicit_transaction = self.focus_transaction.borrow().is_none();

        if implicit_transaction {
            self.begin_focus_transaction();
        }

        // Designate the viewport as the new focused area of the `Document`, but
        // do not run the focusing steps.
        {
            let mut focus_transaction = self.focus_transaction.borrow_mut();
            focus_transaction.as_mut().unwrap().element = None;
        }

        if implicit_transaction {
            self.commit_focus_transaction(FocusInitiator::Local, can_gc);
        }
    }

    /// Request that the given element receive focus with default options.
    /// See [`Self::request_focus_with_options`] for the details.
    pub(crate) fn request_focus(
        &self,
        elem: Option<&Element>,
        focus_initiator: FocusInitiator,
        can_gc: CanGc,
    ) {
        self.request_focus_with_options(
            elem,
            focus_initiator,
            FocusOptions {
                preventScroll: true,
            },
            can_gc,
        );
    }

    /// Request that the given element receive focus once the current
    /// transaction is complete. `None` specifies to focus the document.
    ///
    /// If there's no ongoing transaction, this method automatically starts and
    /// commits an implicit transaction.
    pub(crate) fn request_focus_with_options(
        &self,
        elem: Option<&Element>,
        focus_initiator: FocusInitiator,
        focus_options: FocusOptions,
        can_gc: CanGc,
    ) {
        // If an element is specified, and it's non-focusable, ignore the
        // request.
        if elem.is_some_and(|e| !e.is_focusable_area()) {
            return;
        }

        let implicit_transaction = self.focus_transaction.borrow().is_none();

        if implicit_transaction {
            self.begin_focus_transaction();
        }

        {
            let mut focus_transaction = self.focus_transaction.borrow_mut();
            let focus_transaction = focus_transaction.as_mut().unwrap();
            focus_transaction.element = elem.map(Dom::from_ref);
            focus_transaction.has_focus = true;
            focus_transaction.focus_options = focus_options;
        }

        if implicit_transaction {
            self.commit_focus_transaction(focus_initiator, can_gc);
        }
    }

    /// Update the local focus state accordingly after being notified that the
    /// document's container is removed from the top-level browsing context's
    /// focus chain (not considering system focus).
    pub(crate) fn handle_container_unfocus(&self, can_gc: CanGc) {
        if self.window().parent_info().is_none() {
            warn!("Top-level document cannot be unfocused");
            return;
        }

        // Since this method is called from an event loop, there mustn't be
        // an in-progress focus transaction
        assert!(
            self.focus_transaction.borrow().is_none(),
            "there mustn't be an in-progress focus transaction at this point"
        );

        // Start an implicit focus transaction
        self.begin_focus_transaction();

        // Update the transaction
        {
            let mut focus_transaction = self.focus_transaction.borrow_mut();
            focus_transaction.as_mut().unwrap().has_focus = false;
        }

        // Commit the implicit focus transaction
        self.commit_focus_transaction(FocusInitiator::Remote, can_gc);
    }

    /// Reassign the focus context to the element that last requested focus during this
    /// transaction, or the document if no elements requested it.
    pub(crate) fn commit_focus_transaction(&self, focus_initiator: FocusInitiator, can_gc: CanGc) {
        let (mut new_focused, new_focus_state, prevent_scroll) = {
            let focus_transaction = self.focus_transaction.borrow();
            let focus_transaction = focus_transaction
                .as_ref()
                .expect("no focus transaction in progress");
            (
                focus_transaction
                    .element
                    .as_ref()
                    .map(|e| DomRoot::from_ref(&**e)),
                focus_transaction.has_focus,
                focus_transaction.focus_options.preventScroll,
            )
        };
        *self.focus_transaction.borrow_mut() = None;

        if !new_focus_state {
            // In many browsers, a document forgets its focused area when the
            // document is removed from the top-level BC's focus chain
            if new_focused.take().is_some() {
                trace!(
                    "Forgetting the document's focused area because the \
                    document's container was removed from the top-level BC's \
                    focus chain"
                );
            }
        }

        let old_focused = self.focused.get();
        let old_focus_state = self.has_focus.get();

        debug!(
            "Committing focus transaction: {:?} → {:?}",
            (&old_focused, old_focus_state),
            (&new_focused, new_focus_state),
        );

        // `*_focused_filtered` indicates the local element (if any) included in
        // the top-level BC's focus chain.
        let old_focused_filtered = old_focused.as_ref().filter(|_| old_focus_state);
        let new_focused_filtered = new_focused.as_ref().filter(|_| new_focus_state);

        let trace_focus_chain = |name, element, doc| {
            trace!(
                "{} local focus chain: {}",
                name,
                match (element, doc) {
                    (Some(e), _) => format!("[{:?}, document]", e),
                    (None, true) => "[document]".to_owned(),
                    (None, false) => "[]".to_owned(),
                }
            );
        };

        trace_focus_chain("Old", old_focused_filtered, old_focus_state);
        trace_focus_chain("New", new_focused_filtered, new_focus_state);

        if old_focused_filtered != new_focused_filtered {
            if let Some(elem) = &old_focused_filtered {
                let node = elem.upcast::<Node>();
                elem.set_focus_state(false);
                // FIXME: pass appropriate relatedTarget
                if node.is_connected() {
                    self.fire_focus_event(FocusEventType::Blur, node.upcast(), None, can_gc);
                }
            }
        }

        if old_focus_state != new_focus_state && !new_focus_state {
            self.fire_focus_event(FocusEventType::Blur, self.global().upcast(), None, can_gc);
        }

        self.focused.set(new_focused.as_deref());
        self.has_focus.set(new_focus_state);

        if old_focus_state != new_focus_state && new_focus_state {
            self.fire_focus_event(FocusEventType::Focus, self.global().upcast(), None, can_gc);
        }

        if old_focused_filtered != new_focused_filtered {
            if let Some(elem) = &new_focused_filtered {
                elem.set_focus_state(true);
                let node = elem.upcast::<Node>();
                if let Some(html_element) = elem.downcast::<HTMLElement>() {
                    html_element.handle_focus_state_for_contenteditable(can_gc);
                }
                // FIXME: pass appropriate relatedTarget
                self.fire_focus_event(FocusEventType::Focus, node.upcast(), None, can_gc);

                // Scroll operation to happen after element gets focus. This is needed to ensure that the
                // focused element is visible. Only scroll if preventScroll was not specified.
                if !prevent_scroll {
                    // We are following the firefox implementation where we are only scrolling to the element
                    // if the element itself it not visible.
                    let scroll_axis = ScrollAxisState {
                        position: ScrollLogicalPosition::Center,
                        requirement: ScrollRequirement::IfNotVisible,
                    };

                    // TODO(stevennovaryo): we doesn't differentiate focus operation from script and from user
                    //                      for a scroll yet.
                    // TODO(#40474): Implement specific ScrollIntoView for a selection of text control element.
                    elem.scroll_into_view_with_options(
                        ScrollBehavior::Smooth,
                        scroll_axis,
                        scroll_axis,
                        None,
                        None,
                    );
                }
            }
        }

        if focus_initiator != FocusInitiator::Local {
            return;
        }

        // We are the initiator of the focus operation, so we must broadcast
        // the change we intend to make.
        match (old_focus_state, new_focus_state) {
            (_, true) => {
                // Advertise the change in the focus chain.
                // <https://html.spec.whatwg.org/multipage/#focus-chain>
                // <https://html.spec.whatwg.org/multipage/#focusing-steps>
                //
                // If the top-level BC doesn't have system focus, this won't
                // have an immediate effect, but it will when we gain system
                // focus again. Therefore we still have to send `ScriptMsg::
                // Focus`.
                //
                // When a container with a non-null nested browsing context is
                // focused, its active document becomes the focused area of the
                // top-level browsing context instead. Therefore we need to let
                // the constellation know if such a container is focused.
                //
                // > The focusing steps for an object `new focus target` [...]
                // >
                // >  3. If `new focus target` is a browsing context container
                // >     with non-null nested browsing context, then set
                // >     `new focus target` to the nested browsing context's
                // >     active document.
                let child_browsing_context_id = new_focused
                    .as_ref()
                    .and_then(|elem| elem.downcast::<HTMLIFrameElement>())
                    .and_then(|iframe| iframe.browsing_context_id());

                let sequence = self.increment_fetch_focus_sequence();

                debug!(
                    "Advertising the focus request to the constellation \
                        with sequence number {} and child BC ID {}",
                    sequence,
                    child_browsing_context_id
                        .as_ref()
                        .map(|id| id as &dyn std::fmt::Display)
                        .unwrap_or(&"(none)"),
                );

                self.window()
                    .send_to_constellation(ScriptToConstellationMessage::Focus(
                        child_browsing_context_id,
                        sequence,
                    ));
            },
            (false, false) => {
                // Our `Document` doesn't have focus, and we intend to keep it
                // this way.
            },
            (true, false) => {
                unreachable!(
                    "Can't lose the document's focus without specifying \
                    another one to focus"
                );
            },
        }
    }

    /// Handles any updates when the document's title has changed.
    pub(crate) fn title_changed(&self) {
        if self.browsing_context().is_some() {
            self.send_title_to_embedder();
            let title = String::from(self.Title());
            self.window
                .send_to_constellation(ScriptToConstellationMessage::TitleChanged(
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
            DOMString::from(str_join(value.str().split_html_space_characters(), " "))
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

    pub(crate) fn send_to_embedder(&self, msg: EmbedderMsg) {
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
            node.dirty(NodeDamage::Other)
        }
    }

    /// <https://drafts.csswg.org/cssom-view/#document-run-the-scroll-steps>
    pub(crate) fn run_the_scroll_steps(&self, can_gc: CanGc) {
        // Step 1.
        // > Run the steps to dispatch pending scrollsnapchanging events for doc.
        // TODO(#7673): Implement scroll snapping

        // Step 2
        // > For each item target in doc’s pending scroll event targets, in the order they
        // > were added to the list, run these substeps:
        // Step 3.
        // > Empty doc’s pending scroll event targets.
        // Since the scroll event callback could trigger another scroll event, we are taking all of the
        // current scroll event to avoid borrow checking error.
        rooted_vec!(let notify_list <- self.pending_scroll_event_targets.take().into_iter());
        for target in notify_list.iter() {
            if target.downcast::<Document>().is_some() {
                // Step 2.1
                // > If target is a Document, fire an event named scroll that bubbles at target.
                target.fire_bubbling_event(Atom::from("scroll"), can_gc);
            } else if target.downcast::<Element>().is_some() {
                // Step 2.2
                // > Otherwise, fire an event named scroll at target.
                target.fire_event(Atom::from("scroll"), can_gc);
            }
        }

        // Step 4.
        // > Run the steps to dispatch pending scrollsnapchange events for doc.
        // TODO(#7673): Implement scroll snapping
    }

    /// Whenever a viewport gets scrolled (whether in response to user interaction or by an
    /// API), the user agent must run these steps:
    /// <https://drafts.csswg.org/cssom-view/#scrolling-events>
    pub(crate) fn handle_viewport_scroll_event(&self) {
        // Step 2.
        // > If doc is a snap container, run the steps to update scrollsnapchanging targets
        // > for doc with doc’s eventual snap target in the block axis as newBlockTarget and
        // > doc’s eventual snap target in the inline axis as newInlineTarget.
        // TODO(#7673): Implement scroll snapping

        // Step 3.
        // > If doc is already in doc’s pending scroll event targets, abort these steps.
        let target = self.upcast::<EventTarget>();
        if self
            .pending_scroll_event_targets
            .borrow()
            .iter()
            .any(|other_target| *other_target == target)
        {
            return;
        }

        // Step 4.
        // > Append doc to doc’s pending scroll event targets.
        self.pending_scroll_event_targets
            .borrow_mut()
            .push(Dom::from_ref(target));
    }

    /// Whenever an element gets scrolled (whether in response to user interaction or by an
    /// API), the user agent must run these steps:
    /// <https://drafts.csswg.org/cssom-view/#scrolling-events>
    pub(crate) fn handle_element_scroll_event(&self, element: &Element) {
        // Step 2.
        // > If the element is a snap container, run the steps to update scrollsnapchanging
        // > targets for the element with the element’s eventual snap target in the block
        // > axis as newBlockTarget and the element’s eventual snap target in the inline axis
        // > as newInlineTarget.
        // TODO(#7673): Implement scroll snapping

        // Step 3.
        // > If the element is already in doc’s pending scroll event targets, abort these steps.
        let target = element.upcast::<EventTarget>();
        if self
            .pending_scroll_event_targets
            .borrow()
            .iter()
            .any(|other_target| *other_target == target)
        {
            return;
        }

        // Step 4.
        // > Append the element to doc’s pending scroll event targets.
        self.pending_scroll_event_targets
            .borrow_mut()
            .push(Dom::from_ref(target));
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
                        fragment.AppendChild(&node, can_gc)?;
                    },
                    NodeOrString::String(string) => {
                        let node = DomRoot::upcast::<Node>(self.CreateTextNode(string, can_gc));
                        // No try!() here because appending a text node
                        // should not fail.
                        fragment.AppendChild(&node, can_gc).unwrap();
                    },
                }
            }
            Ok(fragment)
        }
    }

    pub(crate) fn get_body_attribute(&self, local_name: &LocalName) -> DOMString {
        match self.GetBody() {
            Some(ref body) if body.is_body_element() => {
                body.upcast::<Element>().get_string_attribute(local_name)
            },
            _ => DOMString::new(),
        }
    }

    pub(crate) fn set_body_attribute(
        &self,
        local_name: &LocalName,
        value: DOMString,
        can_gc: CanGc,
    ) {
        if let Some(ref body) = self.GetBody().filter(|elem| elem.is_body_element()) {
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

    pub(crate) fn render_blocking_element_count(&self) -> u32 {
        self.render_blocking_element_count.get()
    }

    pub(crate) fn increment_render_blocking_element_count(&self) {
        let count_cell = &self.render_blocking_element_count;
        count_cell.set(count_cell.get() + 1);
    }

    pub(crate) fn decrement_render_blocking_element_count(&self) {
        let count_cell = &self.render_blocking_element_count;
        assert!(count_cell.get() > 0);
        count_cell.set(count_cell.get() - 1);
    }

    pub(crate) fn invalidate_stylesheets(&self) {
        self.stylesheets.borrow_mut().force_dirty(OriginSet::all());

        // Mark the document element dirty so a reflow will be performed.
        //
        // FIXME(emilio): Use the DocumentStylesheetSet invalidation stuff.
        if let Some(element) = self.GetDocumentElement() {
            element.upcast::<Node>().dirty(NodeDamage::Style);
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

        let had_animation_frame_callbacks;
        {
            let mut animation_frame_list = self.animation_frame_list.borrow_mut();
            had_animation_frame_callbacks = !animation_frame_list.is_empty();
            animation_frame_list.push_back((ident, Some(callback)));
        }

        // No need to send a `ChangeRunningAnimationsState` if we're running animation callbacks:
        // we're guaranteed to already be in the "animation callbacks present" state.
        //
        // This reduces CPU usage by avoiding needless thread wakeups in the common case of
        // repeated rAF.
        if !self.running_animation_callbacks.get() && !had_animation_frame_callbacks {
            self.window().send_to_constellation(
                ScriptToConstellationMessage::ChangeRunningAnimationsState(
                    AnimationState::AnimationCallbacksPresent,
                ),
            );
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

    /// <https://html.spec.whatwg.org/multipage/#run-the-animation-frame-callbacks>
    pub(crate) fn run_the_animation_frame_callbacks(&self, can_gc: CanGc) {
        let _realm = enter_realm(self);

        self.running_animation_callbacks.set(true);
        let timing = self.global().performance().Now();

        let num_callbacks = self.animation_frame_list.borrow().len();
        for _ in 0..num_callbacks {
            let (_, maybe_callback) = self.animation_frame_list.borrow_mut().pop_front().unwrap();
            if let Some(callback) = maybe_callback {
                callback.call(self, *timing, can_gc);
            }
        }
        self.running_animation_callbacks.set(false);

        if self.animation_frame_list.borrow().is_empty() {
            self.window().send_to_constellation(
                ScriptToConstellationMessage::ChangeRunningAnimationsState(
                    AnimationState::NoAnimationCallbacksPresent,
                ),
            );
        }
    }

    pub(crate) fn policy_container(&self) -> Ref<'_, PolicyContainer> {
        self.policy_container.borrow()
    }

    pub(crate) fn set_policy_container(&self, policy_container: PolicyContainer) {
        *self.policy_container.borrow_mut() = policy_container;
    }

    pub(crate) fn set_csp_list(&self, csp_list: Option<CspList>) {
        self.policy_container.borrow_mut().set_csp_list(csp_list);
    }

    pub(crate) fn get_csp_list(&self) -> Ref<'_, Option<CspList>> {
        Ref::map(self.policy_container.borrow(), |policy_container| {
            &policy_container.csp_list
        })
    }

    pub(crate) fn preloaded_resources(&self) -> std::cell::Ref<'_, PreloadedResources> {
        self.preloaded_resources.borrow()
    }

    pub(crate) fn insert_preloaded_resource(&self, key: PreloadKey, preload_id: PreloadId) {
        self.preloaded_resources
            .borrow_mut()
            .insert(key, preload_id);
    }

    pub(crate) fn fetch<Listener: FetchResponseListener>(
        &self,
        load: LoadType,
        mut request: RequestBuilder,
        listener: Listener,
    ) {
        request = request
            .insecure_requests_policy(self.insecure_requests_policy())
            .has_trustworthy_ancestor_origin(self.has_trustworthy_ancestor_or_current_origin());
        let callback = NetworkListener {
            context: std::sync::Arc::new(Mutex::new(Some(listener))),
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

    pub(crate) fn fetch_background<Listener: FetchResponseListener>(
        &self,
        mut request: RequestBuilder,
        listener: Listener,
    ) {
        request = request
            .insecure_requests_policy(self.insecure_requests_policy())
            .has_trustworthy_ancestor_origin(self.has_trustworthy_ancestor_or_current_origin());
        let callback = NetworkListener {
            context: std::sync::Arc::new(Mutex::new(Some(listener))),
            task_source: self
                .owner_global()
                .task_manager()
                .networking_task_source()
                .into(),
        }
        .into_callback();
        self.loader_mut().fetch_async_background(request, callback);
    }

    /// <https://fetch.spec.whatwg.org/#deferred-fetch-control-document>
    fn deferred_fetch_control_document(&self) -> DomRoot<Document> {
        match self.window().window_proxy().frame_element() {
            // Step 1. If document’ node navigable’s container document is null
            // or a document whose origin is not same origin with document, then return document;
            None => DomRoot::from_ref(self),
            // otherwise, return the deferred-fetch control document given document’s node navigable’s container document.
            Some(container) => container.owner_document().deferred_fetch_control_document(),
        }
    }

    /// <https://fetch.spec.whatwg.org/#available-deferred-fetch-quota>
    pub(crate) fn available_deferred_fetch_quota(&self, origin: ImmutableOrigin) -> isize {
        // Step 1. Let controlDocument be document’s deferred-fetch control document.
        let control_document = self.deferred_fetch_control_document();
        // Step 2. Let navigable be controlDocument’s node navigable.
        let navigable = control_document.window();
        // Step 3. Let isTopLevel be true if controlDocument’s node navigable
        // is a top-level traversable; otherwise false.
        let is_top_level = navigable.is_top_level();
        // Step 4. Let deferredFetchAllowed be true if controlDocument is allowed
        // to use the policy-controlled feature "deferred-fetch"; otherwise false.
        // TODO
        let deferred_fetch_allowed = true;
        // Step 5. Let deferredFetchMinimalAllowed be true if controlDocument
        // is allowed to use the policy-controlled feature "deferred-fetch-minimal"; otherwise false.
        // TODO
        let deferred_fetch_minimal_allowed = true;
        // Step 6. Let quota be the result of the first matching statement:
        let mut quota = match is_top_level {
            // isTopLevel is true and deferredFetchAllowed is false
            true if !deferred_fetch_allowed => 0,
            // isTopLevel is true and deferredFetchMinimalAllowed is false
            true if !deferred_fetch_minimal_allowed => 640 * 1024,
            // isTopLevel is true
            true => 512 * 1024,
            // deferredFetchAllowed is true, and navigable’s navigable container’s
            // reserved deferred-fetch quota is normal quota
            // TODO
            _ if deferred_fetch_allowed => 0,
            // deferredFetchMinimalAllowed is true, and navigable’s navigable container’s
            // reserved deferred-fetch quota is minimal quota
            // TODO
            _ if deferred_fetch_minimal_allowed => 8 * 1024,
            // Otherwise
            _ => 0,
        } as isize;
        // Step 7. Let quotaForRequestOrigin be 64 kibibytes.
        let mut quota_for_request_origin = 64 * 1024_isize;
        // Step 8. For each navigable in controlDocument’s node navigable’s
        // inclusive descendant navigables whose active document’s deferred-fetch control document is controlDocument:
        // TODO
        // Step 8.1. For each container in navigable’s active document’s shadow-including inclusive descendants
        // which is a navigable container, decrement quota by container’s reserved deferred-fetch quota.
        // TODO
        // Step 8.2. For each deferred fetch record deferredRecord of navigable’s active document’s
        // relevant settings object’s fetch group’s deferred fetch records:
        for deferred_fetch in navigable.as_global_scope().deferred_fetches() {
            // Step 8.2.1. If deferredRecord’s invoke state is not "pending", then continue.
            if deferred_fetch.invoke_state.get() != DeferredFetchRecordInvokeState::Pending {
                continue;
            }
            // Step 8.2.2. Let requestLength be the total request length of deferredRecord’s request.
            let request_length = deferred_fetch.request.total_request_length();
            // Step 8.2.3. Decrement quota by requestLength.
            quota -= request_length as isize;
            // Step 8.2.4. If deferredRecord’s request’s URL’s origin is same origin with origin,
            // then decrement quotaForRequestOrigin by requestLength.
            if deferred_fetch.request.url().origin() == origin {
                quota_for_request_origin -= request_length as isize;
            }
        }
        // Step 9. If quota is equal or less than 0, then return 0.
        if quota <= 0 {
            return 0;
        }
        // Step 10. If quota is less than quotaForRequestOrigin, then return quota.
        if quota < quota_for_request_origin {
            return quota;
        }
        // Step 11. Return quotaForRequestOrigin.
        quota_for_request_origin
    }

    /// <https://html.spec.whatwg.org/multipage/#update-document-for-history-step-application>
    pub(crate) fn update_document_for_history_step_application(
        &self,
        old_url: &ServoUrl,
        new_url: &ServoUrl,
    ) {
        // Step 6. If documentsEntryChanged is true, then:
        //
        // It is right now since we already have a document and a new_url

        // Step 6.1. Let oldURL be document's latest entry's URL.
        // Passed in as argument

        // Step 6.2. Set document's latest entry to entry.
        // TODO
        // Step 6.3. Restore the history object state given document and entry.
        // TODO
        // Step 6.4. If documentIsNew is false, then:
        // TODO
        // Step 6.4.1. Assert: navigationType is not null.
        // TODO
        // Step 6.4.2. Update the navigation API entries for a same-document navigation given navigation, entry, and navigationType.
        // TODO
        // Step 6.4.3. Fire an event named popstate at document's relevant global object, using PopStateEvent,
        // with the state attribute initialized to document's history object's state and hasUAVisualTransition
        // initialized to true if a visual transition, to display a cached rendered state of the latest entry, was done by the user agent.
        // TODO
        // Step 6.4.4. Restore persisted state given entry.
        // TODO

        // Step 6.4.5. If oldURL's fragment is not equal to entry's URL's fragment,
        // then queue a global task on the DOM manipulation task source given document's relevant global object
        // to fire an event named hashchange at document's relevant global object, using HashChangeEvent,
        // with the oldURL attribute initialized to the serialization of oldURL
        // and the newURL attribute initialized to the serialization of entry's URL.
        if old_url.as_url()[Position::BeforeFragment..] !=
            new_url.as_url()[Position::BeforeFragment..]
        {
            let window = Trusted::new(self.owner_window().deref());
            let old_url = old_url.to_string();
            let new_url = new_url.to_string();
            self.owner_global()
                .task_manager()
                .dom_manipulation_task_source()
                .queue(task!(hashchange_event: move || {
                        let window = window.root();
                        HashChangeEvent::new(
                            &window,
                            atom!("hashchange"),
                            false,
                            false,
                            old_url,
                            new_url,
                            CanGc::note(),
                        )
                        .upcast::<Event>()
                        .fire(window.upcast(), CanGc::note());
                }));
        }
    }

    // https://html.spec.whatwg.org/multipage/#the-end
    // https://html.spec.whatwg.org/multipage/#delay-the-load-event
    pub(crate) fn finish_load(&self, load: LoadType, cx: &mut js::context::JSContext) {
        // This does not delay the load event anymore.
        debug!("Document got finish_load: {:?}", load);
        self.loader.borrow_mut().finish_load(&load);

        match load {
            LoadType::Stylesheet(_) => {
                // A stylesheet finishing to load may unblock any pending
                // parsing-blocking script or deferred script.
                self.process_pending_parsing_blocking_script(cx);

                // Step 3.
                self.process_deferred_scripts(CanGc::from_cx(cx));
            },
            LoadType::PageSource(_) => {
                // We finished loading the page, so if the `Window` is still waiting for
                // the first layout, allow it.
                if self.has_browsing_context && self.is_fully_active() {
                    self.window().allow_layout_if_necessary();
                }

                // Deferred scripts have to wait for page to finish loading,
                // this is the first opportunity to process them.

                // Step 3.
                self.process_deferred_scripts(CanGc::from_cx(cx));
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

    /// <https://html.spec.whatwg.org/multipage/#checking-if-unloading-is-canceled>
    pub(crate) fn check_if_unloading_is_cancelled(
        &self,
        recursive_flag: bool,
        can_gc: CanGc,
    ) -> bool {
        // TODO: Step 1, increase the event loop's termination nesting level by 1.
        // Step 2
        self.incr_ignore_opens_during_unload_counter();
        // Step 3-5.
        let beforeunload_event = BeforeUnloadEvent::new(
            &self.window,
            atom!("beforeunload"),
            EventBubbles::Bubbles,
            EventCancelable::Cancelable,
            can_gc,
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
            let (chan, port) = generic_channel::channel().expect("Failed to create IPC channel!");
            let msg = EmbedderMsg::AllowUnload(self.webview_id(), chan);
            self.send_to_embedder(msg);
            can_unload = port.recv().unwrap() == AllowOrDeny::Allow;
        }
        // Step 9
        if !recursive_flag {
            // `check_if_unloading_is_cancelled` might cause futher modifications to the DOM so collecting here prevents
            // a double borrow if the `IFrameCollection` needs to be validated again.
            let iframes: Vec<_> = self.iframes().iter().collect();
            for iframe in &iframes {
                // TODO: handle the case of cross origin iframes.
                let document = iframe.owner_document();
                can_unload = document.check_if_unloading_is_cancelled(true, can_gc);
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
            self.window
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
            self.window
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

        // Step 18. Run any unloading document cleanup steps for oldDocument that are defined by this specification and other applicable specifications.
        self.unloading_cleanup_steps();

        // https://w3c.github.io/FileAPI/#lifeTime
        self.window.as_global_scope().clean_up_all_file_resources();

        // Step 15, End
        self.decr_ignore_opens_during_unload_counter();

        // Step 20. If oldDocument's salvageable state is false, then destroy oldDocument.
        // TODO
    }

    /// <https://html.spec.whatwg.org/multipage/#completely-finish-loading>
    fn completely_finish_loading(&self) {
        // Step 1. Assert: document's browsing context is non-null.
        // TODO: Adding this assert fails a lot of tests

        // Step 2. Set document's completely loaded time to the current time.
        self.completely_loaded.set(true);
        // Step 3. Let container be document's node navigable's container.
        // TODO

        // Step 4. If container is an iframe element, then queue an element task
        // on the DOM manipulation task source given container to run the iframe load event steps given container.
        //
        // Note: this will also result in the "iframe-load-event-steps" being run.
        // https://html.spec.whatwg.org/multipage/#iframe-load-event-steps
        self.notify_constellation_load();

        // Step 5. Otherwise, if container is non-null, then queue an element task on the DOM manipulation task source
        // given container to fire an event named load at container.
        // TODO

        // Step 13 of https://html.spec.whatwg.org/multipage/#shared-declarative-refresh-steps
        //
        // At least time seconds have elapsed since document's completely loaded time,
        // adjusted to take into account user or user agent preferences.
        if let Some(DeclarativeRefresh::PendingLoad { url, time }) =
            &*self.declarative_refresh.borrow()
        {
            self.window.as_global_scope().schedule_callback(
                OneshotTimerCallback::RefreshRedirectDue(RefreshRedirectDue {
                    window: DomRoot::from_ref(self.window()),
                    url: url.clone(),
                }),
                Duration::from_secs(*time),
            );
        }
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
            is_in_delaying_load_events_mode ||
            // In case we have already aborted this document and receive a
            // a subsequent message to load the document
            self.loader.borrow().events_inhibited();

        if not_ready_for_load {
            // Step 6.
            return;
        }

        self.loader.borrow_mut().inhibit_events();

        // The rest will ever run only once per document.

        // Step 9. Queue a global task on the DOM manipulation task source given
        // the Document's relevant global object to run the following steps:
        debug!("Document loads are complete.");
        let document = Trusted::new(self);
        self.owner_global()
            .task_manager()
            .dom_manipulation_task_source()
            .queue(task!(fire_load_event: move || {
                let document = document.root();
                // Step 9.3. Let window be the Document's relevant global object.
                let window = document.window();
                if !window.is_alive() {
                    return;
                }

                // Step 9.1. Update the current document readiness to "complete".
                document.set_ready_state(DocumentReadyState::Complete, CanGc::note());

                // Step 9.2. If the Document object's browsing context is null, then abort these steps.
                if document.browsing_context().is_none() {
                    return;
                }

                // Step 9.4. Set the Document's load timing info's load event start time to the current high resolution time given window.
                update_with_current_instant(&document.load_event_start);

                // Step 9.5. Fire an event named load at window, with legacy target override flag set.
                let load_event = Event::new(
                    window.upcast(),
                    atom!("load"),
                    EventBubbles::DoesNotBubble,
                    EventCancelable::NotCancelable,
                    CanGc::note(),
                );
                load_event.set_trusted(true);
                debug!("About to dispatch load for {:?}", document.url());
                window.dispatch_event_with_target_override(&load_event, CanGc::note());

                // Step 9.6. Invoke WebDriver BiDi load complete with the Document's browsing context,
                // and a new WebDriver BiDi navigation status whose id is the Document object's during-loading navigation ID
                // for WebDriver BiDi, status is "complete", and url is the Document object's URL.
                // TODO

                // Step 9.7. Set the Document object's during-loading navigation ID for WebDriver BiDi to null.
                // TODO

                // Step 9.8. Set the Document's load timing info's load event end time to the current high resolution time given window.
                update_with_current_instant(&document.load_event_end);

                // Step 9.9. Assert: Document's page showing is false.
                // TODO: Adding this assert fails a lot of tests

                // Step 9.10. Set the Document's page showing to true.
                document.page_showing.set(true);

                // Step 9.11. Fire a page transition event named pageshow at window with false.
                let page_show_event = PageTransitionEvent::new(
                    window,
                    atom!("pageshow"),
                    false, // bubbles
                    false, // cancelable
                    false, // persisted
                    CanGc::note(),
                );
                let page_show_event = page_show_event.upcast::<Event>();
                page_show_event.set_trusted(true);
                page_show_event.fire(window.upcast(), CanGc::note());

                // Step 9.12. Completely finish loading the Document.
                document.completely_finish_loading();

                // Step 9.13. Queue the navigation timing entry for the Document.
                // TODO

                if let Some(fragment) = document.url().fragment() {
                    document.scroll_to_the_fragment(fragment);
                }
            }));

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
        cx: &mut js::context::JSContext,
    ) {
        {
            let mut blocking_script = self.pending_parsing_blocking_script.borrow_mut();
            let entry = blocking_script.as_mut().unwrap();
            assert!(&*entry.element == element);
            entry.loaded(result);
        }
        self.process_pending_parsing_blocking_script(cx);
    }

    fn process_pending_parsing_blocking_script(&self, cx: &mut js::context::JSContext) {
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
                .resume_with_pending_parsing_blocking_script(&element, result, cx);
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
    pub(crate) fn asap_script_loaded(
        &self,
        element: &HTMLScriptElement,
        result: ScriptResult,
        can_gc: CanGc,
    ) {
        {
            let mut scripts = self.asap_scripts_set.borrow_mut();
            let idx = scripts
                .iter()
                .position(|entry| &**entry == element)
                .unwrap();
            scripts.swap_remove(idx);
        }
        element.execute(result, can_gc);
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
        can_gc: CanGc,
    ) {
        self.asap_in_order_scripts_list.loaded(element, result);
        while let Some((element, result)) = self
            .asap_in_order_scripts_list
            .take_next_ready_to_be_executed()
        {
            element.execute(result, can_gc);
        }
    }

    // https://html.spec.whatwg.org/multipage/#list-of-scripts-that-will-execute-when-the-document-has-finished-parsing
    pub(crate) fn add_deferred_script(&self, script: &HTMLScriptElement) {
        self.deferred_scripts.push(script);
    }

    /// <https://html.spec.whatwg.org/multipage/#the-end> step 3.
    /// <https://html.spec.whatwg.org/multipage/#prepare-a-script> step 22.d.
    pub(crate) fn deferred_script_loaded(
        &self,
        element: &HTMLScriptElement,
        result: ScriptResult,
        can_gc: CanGc,
    ) {
        self.deferred_scripts.loaded(element, result);
        self.process_deferred_scripts(can_gc);
    }

    /// <https://html.spec.whatwg.org/multipage/#the-end> step 3.
    fn process_deferred_scripts(&self, can_gc: CanGc) {
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
                element.execute(result, can_gc);
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
            .maybe_set_tti(InteractiveFlag::DOMContentLoaded);

        // Step 4.2.
        // TODO: client message queue.
    }

    /// <https://html.spec.whatwg.org/multipage/#destroy-a-document-and-its-descendants>
    pub(crate) fn destroy_document_and_its_descendants(&self, cx: &mut js::context::JSContext) {
        // Step 1. If document is not fully active, then:
        if !self.is_fully_active() {
            // Step 1.1. Let reason be a string from user-agent specific blocking reasons.
            // If none apply, then let reason be "masked".
            // TODO
            // Step 1.2. Make document unsalvageable given document and reason.
            self.salvageable.set(false);
            // Step 1.3. If document's node navigable is a top-level traversable,
            // build not restored reasons for a top-level traversable and its descendants given document's node navigable.
            // TODO
        }
        // TODO(#31973): all of the steps below are implemented synchronously at the moment.
        // They need to become asynchronous later, at which point the counting of
        // numberDestroyed becomes relevant.

        // Step 2. Let childNavigables be document's child navigables.
        // Step 3. Let numberDestroyed be 0.
        // Step 4. For each childNavigable of childNavigables, queue a global task on
        // the navigation and traversal task source given childNavigable's active
        // window to perform the following steps:
        // Step 4.1. Let incrementDestroyed be an algorithm step which increments numberDestroyed.
        // Step 4.2. Destroy a document and its descendants given childNavigable's active document and incrementDestroyed.
        // Step 5. Wait until numberDestroyed equals childNavigable's size.
        for exited_iframe in self.iframes().iter() {
            debug!("Destroying nested iframe document");
            exited_iframe.destroy_document_and_its_descendants(cx);
        }
        // Step 6. Queue a global task on the navigation and traversal task source
        // given document's relevant global object to perform the following steps:
        // TODO
        // Step 6.1. Destroy document.
        self.destroy(cx);
        // Step 6.2. If afterAllDestruction was given, then run it.
        // TODO
    }

    /// <https://html.spec.whatwg.org/multipage/#destroy-a-document>
    pub(crate) fn destroy(&self, cx: &mut js::context::JSContext) {
        let exited_window = self.window();
        // Step 2. Abort document.
        self.abort(cx);
        // Step 3. Set document's salvageable state to false.
        self.salvageable.set(false);
        // Step 4. Let ports be the list of MessagePorts whose relevant
        // global object's associated Document is document.
        // TODO

        // Step 5. For each port in ports, disentangle port.
        // TODO

        // Step 6. Run any unloading document cleanup steps for document that
        // are defined by this specification and other applicable specifications.
        self.unloading_cleanup_steps();

        // Step 7. Remove any tasks whose document is document from any task queue
        // (without running those tasks).
        exited_window
            .as_global_scope()
            .task_manager()
            .cancel_all_tasks_and_ignore_future_tasks();

        // Step 8. Set document's browsing context to null.
        exited_window.discard_browsing_context();

        // Step 9. Set document's node navigable's active session history entry's
        // document state's document to null.
        // TODO

        // Step 10. Remove document from the owner set of each WorkerGlobalScope
        // object whose set contains document.
        // TODO

        // Step 11. For each workletGlobalScope in document's worklet global scopes,
        // terminate workletGlobalScope.
        // TODO
    }

    /// <https://fetch.spec.whatwg.org/#concept-fetch-group-terminate>
    fn terminate_fetch_group(&self) -> bool {
        let mut load_cancellers = self.loader.borrow_mut().cancel_all_loads();

        // Step 1. For each fetch record record of fetchGroup’s fetch records,
        // if record’s controller is non-null and record’s request’s done flag
        // is unset and keepalive is false, terminate record’s controller.
        for canceller in &mut load_cancellers {
            if !canceller.keep_alive() {
                canceller.terminate();
            }
        }
        // Step 2. Process deferred fetches for fetchGroup.
        self.owner_global().process_deferred_fetches();

        !load_cancellers.is_empty()
    }

    /// <https://html.spec.whatwg.org/multipage/#abort-a-document>
    pub(crate) fn abort(&self, cx: &mut js::context::JSContext) {
        // We need to inhibit the loader before anything else.
        self.loader.borrow_mut().inhibit_events();

        // Step 1.
        for iframe in self.iframes().iter() {
            if let Some(document) = iframe.GetContentDocument() {
                document.abort(cx);
            }
        }

        // Step 2. Cancel any instances of the fetch algorithm in the context of document,
        // discarding any tasks queued for them, and discarding any further data received
        // from the network for them. If this resulted in any instances of the fetch algorithm
        // being canceled or any queued tasks or any network data getting discarded,
        // then make document unsalvageable given document and "fetch".
        self.script_blocking_stylesheets_count.set(0);
        *self.pending_parsing_blocking_script.borrow_mut() = None;
        *self.asap_scripts_set.borrow_mut() = vec![];
        self.asap_in_order_scripts_list.clear();
        self.deferred_scripts.clear();
        let loads_cancelled = self.terminate_fetch_group();
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

        // Step 3. If document's during-loading navigation ID for WebDriver BiDi is non-null, then:
        // TODO

        // Step 4. If document has an active parser, then:
        if let Some(parser) = self.get_current_parser() {
            // Step 4.1. Set document's active parser was aborted to true.
            self.active_parser_was_aborted.set(true);
            // Step 4.2. Abort that parser.
            parser.abort(cx);
            // Step 4.3. Make document unsalvageable given document and "parser-aborted".
            self.salvageable.set(false);
        }
    }

    pub(crate) fn notify_constellation_load(&self) {
        self.window()
            .send_to_constellation(ScriptToConstellationMessage::LoadComplete);
    }

    pub(crate) fn set_current_parser(&self, script: Option<&ServoParser>) {
        self.current_parser.set(script);
    }

    pub(crate) fn get_current_parser(&self) -> Option<DomRoot<ServoParser>> {
        self.current_parser.get()
    }

    pub(crate) fn get_current_parser_line(&self) -> u32 {
        self.get_current_parser()
            .map(|parser| parser.get_current_line())
            .unwrap_or(0)
    }

    /// A reference to the [`IFrameCollection`] of this [`Document`], holding information about
    /// `<iframe>`s found within it.
    pub(crate) fn iframes(&self) -> Ref<'_, IFrameCollection> {
        self.iframes.borrow_mut().validate(self);
        self.iframes.borrow()
    }

    /// A mutable reference to the [`IFrameCollection`] of this [`Document`], holding information about
    /// `<iframe>`s found within it.
    pub(crate) fn iframes_mut(&self) -> RefMut<'_, IFrameCollection> {
        self.iframes.borrow_mut().validate(self);
        self.iframes.borrow_mut()
    }

    pub(crate) fn invalidate_iframes_collection(&self) {
        self.iframes.borrow_mut().invalidate();
    }

    pub(crate) fn get_dom_interactive(&self) -> Option<CrossProcessInstant> {
        self.dom_interactive.get()
    }

    pub(crate) fn set_navigation_start(&self, navigation_start: CrossProcessInstant) {
        self.interactive_time
            .borrow_mut()
            .set_navigation_start(navigation_start);
    }

    pub(crate) fn get_interactive_metrics(&self) -> Ref<'_, ProgressiveWebMetrics> {
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
            self.get_interactive_metrics()
                .maybe_set_tti(InteractiveFlag::TimeToInteractive(
                    self.tti_window.borrow().get_start(),
                ));
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#fire-a-focus-event>
    fn fire_focus_event(
        &self,
        focus_event_type: FocusEventType,
        event_target: &EventTarget,
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
        event.fire(event_target, can_gc);
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

    /// <https://dom.spec.whatwg.org/#document-custom-element-registry>
    pub(crate) fn custom_element_registry(&self) -> DomRoot<CustomElementRegistry> {
        self.window.CustomElements()
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

    pub(crate) fn register_media_controls(&self, id: &str, controls: &ShadowRoot) {
        let did_have_these_media_controls = self
            .media_controls
            .borrow_mut()
            .insert(id.to_string(), Dom::from_ref(controls))
            .is_some();
        debug_assert!(
            !did_have_these_media_controls,
            "Trying to register known media controls"
        );
    }

    pub(crate) fn unregister_media_controls(&self, id: &str) {
        let did_have_these_media_controls = self.media_controls.borrow_mut().remove(id).is_some();
        debug_assert!(
            did_have_these_media_controls,
            "Trying to unregister unknown media controls"
        );
    }

    pub(crate) fn mark_canvas_as_dirty(&self, canvas: &Dom<HTMLCanvasElement>) {
        let mut dirty_canvases = self.dirty_canvases.borrow_mut();
        if dirty_canvases
            .iter()
            .any(|dirty_canvas| dirty_canvas == canvas)
        {
            return;
        }
        dirty_canvases.push(canvas.clone());
    }

    /// Whether or not this [`Document`] needs a rendering update, due to changed
    /// contents or pending events. This is used to decide whether or not to schedule
    /// a call to the "update the rendering" algorithm.
    pub(crate) fn needs_rendering_update(&self) -> bool {
        if !self.is_fully_active() {
            return false;
        }
        if !self.window().layout_blocked() &&
            (!self.restyle_reason().is_empty() ||
                self.window().layout().needs_new_display_list())
        {
            return true;
        }
        if !self.rendering_update_reasons.get().is_empty() {
            return true;
        }
        if self.event_handler.has_pending_input_events() {
            return true;
        }
        if self.has_pending_scroll_events() {
            return true;
        }
        if self.window().has_unhandled_resize_event() {
            return true;
        }
        if self.has_pending_animated_image_update.get() || !self.dirty_canvases.borrow().is_empty()
        {
            return true;
        }

        false
    }

    /// An implementation of step 22 from
    /// <https://html.spec.whatwg.org/multipage/#update-the-rendering>:
    ///
    // > Step 22: For each doc of docs, update the rendering or user interface of
    // > doc and its node navigable to reflect the current state.
    //
    // Returns the set of reflow phases run as a [`ReflowPhasesRun`].
    pub(crate) fn update_the_rendering(&self) -> (ReflowPhasesRun, ReflowStatistics) {
        if self.render_blocking_element_count() > 0 {
            return Default::default();
        }

        let mut phases = ReflowPhasesRun::empty();
        if self.has_pending_animated_image_update.get() {
            self.image_animation_manager
                .borrow()
                .update_active_frames(&self.window, self.current_animation_timeline_value());
            self.has_pending_animated_image_update.set(false);
            phases.insert(ReflowPhasesRun::UpdatedImageData);
        }

        self.current_rendering_epoch
            .set(self.current_rendering_epoch.get().next());
        let current_rendering_epoch = self.current_rendering_epoch.get();

        // All dirty canvases are flushed before updating the rendering.
        let image_keys: Vec<_> = self
            .dirty_canvases
            .borrow_mut()
            .drain(..)
            .filter_map(|canvas| canvas.update_rendering(current_rendering_epoch))
            .collect();

        // The renderer should wait to display the frame until all canvas images are
        // uploaded. This allows canvas image uploading to happen asynchronously.
        let pipeline_id = self.window().pipeline_id();
        if !image_keys.is_empty() {
            phases.insert(ReflowPhasesRun::UpdatedImageData);
            self.waiting_on_canvas_image_updates.set(true);
            self.window().paint_api().delay_new_frame_for_canvas(
                self.webview_id(),
                self.window().pipeline_id(),
                current_rendering_epoch,
                image_keys,
            );
        }

        let (reflow_phases, statistics) = self.window().reflow(ReflowGoal::UpdateTheRendering);
        let phases = phases.union(reflow_phases);

        self.window().paint_api().update_epoch(
            self.webview_id(),
            pipeline_id,
            current_rendering_epoch,
        );

        (phases, statistics)
    }

    pub(crate) fn handle_no_longer_waiting_on_asynchronous_image_updates(&self) {
        self.waiting_on_canvas_image_updates.set(false);
    }

    pub(crate) fn waiting_on_canvas_image_updates(&self) -> bool {
        self.waiting_on_canvas_image_updates.get()
    }

    /// From <https://drafts.csswg.org/css-font-loading/#fontfaceset-pending-on-the-environment>:
    ///
    /// > A FontFaceSet is pending on the environment if any of the following are true:
    /// >  - the document is still loading
    /// >  - the document has pending stylesheet requests
    /// >  - the document has pending layout operations which might cause the user agent to request
    /// >    a font, or which depend on recently-loaded fonts
    ///
    /// Returns true if the promise was fulfilled.
    pub(crate) fn maybe_fulfill_font_ready_promise(&self, can_gc: CanGc) -> bool {
        if !self.is_fully_active() {
            return false;
        }

        let fonts = self.Fonts(can_gc);
        if !fonts.waiting_to_fullfill_promise() {
            return false;
        }
        if self.window().font_context().web_fonts_still_loading() != 0 {
            return false;
        }
        if self.ReadyState() != DocumentReadyState::Complete {
            return false;
        }
        if !self.restyle_reason().is_empty() {
            return false;
        }
        if !self.rendering_update_reasons.get().is_empty() {
            return false;
        }

        let result = fonts.fulfill_ready_promise_if_needed(can_gc);

        // Add a rendering update after the `fonts.ready` promise is fulfilled just for
        // the sake of taking screenshots. This has the effect of delaying screenshots
        // until layout has taken a shot at updating the rendering.
        if result {
            self.add_rendering_update_reason(RenderingUpdateReason::FontReadyPromiseFulfilled);
        }

        result
    }

    pub(crate) fn id_map(
        &self,
    ) -> Ref<'_, HashMapTracedValues<Atom, Vec<Dom<Element>>, FxBuildHasher>> {
        self.id_map.borrow()
    }

    pub(crate) fn name_map(
        &self,
    ) -> Ref<'_, HashMapTracedValues<Atom, Vec<Dom<Element>>, FxBuildHasher>> {
        self.name_map.borrow()
    }

    /// <https://drafts.csswg.org/resize-observer/#dom-resizeobserver-resizeobserver>
    pub(crate) fn add_resize_observer(&self, resize_observer: &ResizeObserver) {
        self.resize_observers
            .borrow_mut()
            .push(Dom::from_ref(resize_observer));
    }

    /// Whether or not this [`Document`] has any active [`ResizeObserver`].
    pub(crate) fn has_resize_observers(&self) -> bool {
        !self.resize_observers.borrow().is_empty()
    }

    /// <https://drafts.csswg.org/resize-observer/#gather-active-observations-h>
    /// <https://drafts.csswg.org/resize-observer/#has-active-resize-observations>
    pub(crate) fn gather_active_resize_observations_at_depth(
        &self,
        depth: &ResizeObservationDepth,
    ) -> bool {
        let mut has_active_resize_observations = false;
        for observer in self.resize_observers.borrow_mut().iter_mut() {
            observer.gather_active_resize_observations_at_depth(
                depth,
                &mut has_active_resize_observations,
            );
        }
        has_active_resize_observations
    }

    /// <https://drafts.csswg.org/resize-observer/#broadcast-active-resize-observations>
    #[expect(clippy::redundant_iter_cloned)]
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
            .encoding_override(Some(&|input| {
                servo_url::encoding::encode_as_url_query_string(input, encoding)
            }))
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

    /// Add an [`IntersectionObserver`] to the [`Document`], to be processed in the [`Document`]'s event loop.
    /// <https://github.com/w3c/IntersectionObserver/issues/525>
    pub(crate) fn add_intersection_observer(&self, intersection_observer: &IntersectionObserver) {
        self.intersection_observers
            .borrow_mut()
            .push(Dom::from_ref(intersection_observer));
    }

    /// Remove an [`IntersectionObserver`] from [`Document`], ommiting it from the event loop.
    /// An observer without any target, ideally should be removed to be conformant with
    /// <https://w3c.github.io/IntersectionObserver/#lifetime>.
    pub(crate) fn remove_intersection_observer(
        &self,
        intersection_observer: &IntersectionObserver,
    ) {
        self.intersection_observers
            .borrow_mut()
            .retain(|observer| *observer != intersection_observer)
    }

    /// <https://w3c.github.io/IntersectionObserver/#update-intersection-observations-algo>
    pub(crate) fn update_intersection_observer_steps(
        &self,
        time: CrossProcessInstant,
        can_gc: CanGc,
    ) {
        // Step 1-2
        for intersection_observer in &*self.intersection_observers.borrow() {
            self.update_single_intersection_observer_steps(intersection_observer, time, can_gc);
        }
    }

    /// Step 2.1-2.2 of <https://w3c.github.io/IntersectionObserver/#update-intersection-observations-algo>
    fn update_single_intersection_observer_steps(
        &self,
        intersection_observer: &IntersectionObserver,
        time: CrossProcessInstant,
        can_gc: CanGc,
    ) {
        // Step 1
        // > Let rootBounds be observer’s root intersection rectangle.
        let root_bounds = intersection_observer.root_intersection_rectangle(self);

        // Step 2
        // > For each target in observer’s internal [[ObservationTargets]] slot,
        // > processed in the same order that observe() was called on each target:
        intersection_observer.update_intersection_observations_steps(
            self,
            time,
            root_bounds,
            can_gc,
        );
    }

    /// <https://w3c.github.io/IntersectionObserver/#notify-intersection-observers-algo>
    pub(crate) fn notify_intersection_observers(&self, can_gc: CanGc) {
        // Step 1
        // > Set document’s IntersectionObserverTaskQueued flag to false.
        self.intersection_observer_task_queued.set(false);

        // Step 2
        // > Let notify list be a list of all IntersectionObservers whose root is in the DOM tree of document.
        // We will copy the observers because callback could modify the current list.
        // It will rooted to prevent GC in the iteration.
        rooted_vec!(let notify_list <- self.intersection_observers.clone().take().into_iter());

        // Step 3
        // > For each IntersectionObserver object observer in notify list, run these steps:
        for intersection_observer in notify_list.iter() {
            // Step 3.1-3.5
            intersection_observer.invoke_callback_if_necessary(can_gc);
        }
    }

    /// <https://w3c.github.io/IntersectionObserver/#queue-intersection-observer-task>
    pub(crate) fn queue_an_intersection_observer_task(&self) {
        // Step 1
        // > If document’s IntersectionObserverTaskQueued flag is set to true, return.
        if self.intersection_observer_task_queued.get() {
            return;
        }

        // Step 2
        // > Set document’s IntersectionObserverTaskQueued flag to true.
        self.intersection_observer_task_queued.set(true);

        // Step 3
        // > Queue a task on the IntersectionObserver task source associated with
        // > the document's event loop to notify intersection observers.
        let document = Trusted::new(self);
        self.owner_global()
            .task_manager()
            .intersection_observer_task_source()
            .queue(task!(notify_intersection_observers: move || {
                document.root().notify_intersection_observers(CanGc::note());
            }));
    }

    pub(crate) fn handle_paint_metric(
        &self,
        metric_type: ProgressiveWebMetricType,
        metric_value: CrossProcessInstant,
        first_reflow: bool,
        can_gc: CanGc,
    ) {
        let metrics = self.interactive_time.borrow();
        match metric_type {
            ProgressiveWebMetricType::FirstPaint |
            ProgressiveWebMetricType::FirstContentfulPaint => {
                let binding = PerformancePaintTiming::new(
                    self.window.as_global_scope(),
                    metric_type,
                    metric_value,
                    can_gc,
                );
                metrics.set_performance_paint_metric(metric_value, first_reflow, metric_type);
                let entry = binding.upcast::<PerformanceEntry>();
                self.window.Performance().queue_entry(entry);
            },
            ProgressiveWebMetricType::LargestContentfulPaint { area } => {
                let binding = LargestContentfulPaint::new(
                    self.window.as_global_scope(),
                    metric_type,
                    metric_value,
                    can_gc,
                );
                metrics.set_largest_contentful_paint(metric_value, area);
                let entry = binding.upcast::<PerformanceEntry>();
                self.window.Performance().queue_entry(entry);
            },
            ProgressiveWebMetricType::TimeToInteractive => {
                unreachable!("Unexpected non-paint metric.")
            },
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#document-write-steps>
    fn write(
        &self,
        cx: &mut js::context::JSContext,
        text: Vec<TrustedHTMLOrString>,
        line_feed: bool,
        containing_class: &str,
        field: &str,
    ) -> ErrorResult {
        // Step 1: Let string be the empty string.
        let mut strings: Vec<String> = Vec::with_capacity(text.len());
        // Step 2: Let isTrusted be false if text contains a string; otherwise true.
        let mut is_trusted = true;
        // Step 3: For each value of text:
        for value in text {
            match value {
                // Step 3.1: If value is a TrustedHTML object, then append value's associated data to string.
                TrustedHTMLOrString::TrustedHTML(trusted_html) => {
                    strings.push(trusted_html.to_string());
                },
                TrustedHTMLOrString::String(str_) => {
                    // Step 2: Let isTrusted be false if text contains a string; otherwise true.
                    is_trusted = false;
                    // Step 3.2: Otherwise, append value to string.
                    strings.push(str_.into());
                },
            };
        }
        let mut string = itertools::join(strings, "");
        // Step 4: If isTrusted is false, set string to the result of invoking the
        // Get Trusted Type compliant string algorithm with TrustedHTML,
        // this's relevant global object, string, sink, and "script".
        if !is_trusted {
            string = TrustedHTML::get_trusted_script_compliant_string(
                &self.global(),
                TrustedHTMLOrString::String(string.into()),
                &format!("{} {}", containing_class, field),
                CanGc::from_cx(cx),
            )?
            .str()
            .to_owned();
        }
        // Step 5: If lineFeed is true, append U+000A LINE FEED to string.
        if line_feed {
            string.push('\n');
        }
        // Step 6: If document is an XML document, then throw an "InvalidStateError" DOMException.
        if !self.is_html_document() {
            return Err(Error::InvalidState(None));
        }

        // Step 7: If document's throw-on-dynamic-markup-insertion counter is greater than 0,
        // then throw an "InvalidStateError" DOMException.
        if self.throw_on_dynamic_markup_insertion_counter.get() > 0 {
            return Err(Error::InvalidState(None));
        }

        // Step 8: If document's active parser was aborted is true, then return.
        if !self.is_active() || self.active_parser_was_aborted.get() {
            return Ok(());
        }

        let parser = match self.get_current_parser() {
            Some(ref parser) if parser.can_write() => DomRoot::from_ref(&**parser),
            // Step 9: If the insertion point is undefined, then:
            _ => {
                // Step 9.1: If document's unload counter is greater than 0 or
                // document's ignore-destructive-writes counter is greater than 0, then return.
                if self.is_prompting_or_unloading() ||
                    self.ignore_destructive_writes_counter.get() > 0
                {
                    return Ok(());
                }
                // Step 9.2: Run the document open steps with document.
                self.Open(cx, None, None)?;
                self.get_current_parser().unwrap()
            },
        };

        // Steps 10-11.
        parser.write(string.into(), cx);

        Ok(())
    }

    pub(crate) fn details_name_groups(&self) -> RefMut<'_, DetailsNameGroups> {
        RefMut::map(
            self.details_name_groups.borrow_mut(),
            |details_name_groups| details_name_groups.get_or_insert_default(),
        )
    }
}

#[derive(MallocSizeOf, PartialEq)]
pub(crate) enum DocumentSource {
    FromParser,
    NotFromParser,
}

pub(crate) trait LayoutDocumentHelpers<'dom> {
    fn is_html_document_for_layout(&self) -> bool;
    fn quirks_mode(self) -> QuirksMode;
    fn style_shared_lock(self) -> &'dom StyleSharedRwLock;
    fn shadow_roots(self) -> Vec<LayoutDom<'dom, ShadowRoot>>;
    fn shadow_roots_styles_changed(self) -> bool;
    fn flush_shadow_roots_stylesheets(self);
}

#[expect(unsafe_code)]
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
pub(crate) fn get_registrable_domain_suffix_of_or_is_equal_to(
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
        about_base_url: Option<ServoUrl>,
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
        allow_declarative_shadow_roots: bool,
        inherited_insecure_requests_policy: Option<InsecureRequestsPolicy>,
        has_trustworthy_ancestor_origin: bool,
        custom_element_reaction_stack: Rc<CustomElementReactionStack>,
        creation_sandboxing_flag_set: SandboxingFlagSet,
    ) -> Document {
        let url = url.unwrap_or_else(|| ServoUrl::parse("about:blank").unwrap());

        let (ready_state, domcontentloaded_dispatched) = if source == DocumentSource::FromParser {
            (DocumentReadyState::Loading, false)
        } else {
            (DocumentReadyState::Complete, true)
        };

        let frame_type = match window.is_top_level() {
            true => TimerMetadataFrameType::RootWindow,
            false => TimerMetadataFrameType::IFrame,
        };
        let interactive_time = ProgressiveWebMetrics::new(
            window.time_profiler_chan().clone(),
            url.clone(),
            frame_type,
        );

        let content_type = content_type.unwrap_or_else(|| {
            match is_html_document {
                // https://dom.spec.whatwg.org/#dom-domimplementation-createhtmldocument
                IsHTMLDocument::HTMLDocument => "text/html",
                // https://dom.spec.whatwg.org/#concept-document-content-type
                IsHTMLDocument::NonHTMLDocument => "application/xml",
            }
            .parse()
            .unwrap()
        });

        let encoding = content_type
            .get_parameter(CHARSET)
            .and_then(|charset| Encoding::for_label(charset.as_bytes()))
            .unwrap_or(UTF_8);

        let has_focus = window.parent_info().is_none();

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
            about_base_url: DomRefCell::new(about_base_url),
            // https://dom.spec.whatwg.org/#concept-document-quirks
            quirks_mode: Cell::new(QuirksMode::NoQuirks),
            event_handler: DocumentEventHandler::new(window),
            embedder_controls: DocumentEmbedderControls::new(window),
            id_map: DomRefCell::new(HashMapTracedValues::new_fx()),
            name_map: DomRefCell::new(HashMapTracedValues::new_fx()),
            // https://dom.spec.whatwg.org/#concept-document-encoding
            encoding: Cell::new(encoding),
            is_html_document: is_html_document == IsHTMLDocument::HTMLDocument,
            activity: Cell::new(activity),
            tag_map: DomRefCell::new(HashMapTracedValues::new_fx()),
            tagns_map: DomRefCell::new(HashMapTracedValues::new_fx()),
            classes_map: DomRefCell::new(HashMapTracedValues::new()),
            images: Default::default(),
            embeds: Default::default(),
            links: Default::default(),
            forms: Default::default(),
            scripts: Default::default(),
            anchors: Default::default(),
            applets: Default::default(),
            iframes: RefCell::new(IFrameCollection::new()),
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
                // StyleSharedRwLock::new()
            },
            stylesheets: DomRefCell::new(DocumentStylesheetSet::new()),
            stylesheet_list: MutNullableDom::new(None),
            ready_state: Cell::new(ready_state),
            domcontentloaded_dispatched: Cell::new(domcontentloaded_dispatched),
            focus_transaction: DomRefCell::new(None),
            focused: Default::default(),
            focus_sequence: Cell::new(FocusSequenceNumber::default()),
            has_focus: Cell::new(has_focus),
            current_script: Default::default(),
            pending_parsing_blocking_script: Default::default(),
            script_blocking_stylesheets_count: Default::default(),
            render_blocking_element_count: Default::default(),
            deferred_scripts: Default::default(),
            asap_in_order_scripts_list: Default::default(),
            asap_scripts_set: Default::default(),
            animation_frame_ident: Cell::new(0),
            animation_frame_list: DomRefCell::new(VecDeque::new()),
            running_animation_callbacks: Cell::new(false),
            loader: DomRefCell::new(doc_loader),
            current_parser: Default::default(),
            base_element: Default::default(),
            target_base_element: Default::default(),
            appropriate_template_contents_owner_document: Default::default(),
            pending_restyles: DomRefCell::new(FxHashMap::default()),
            needs_restyle: Cell::new(RestyleReason::DOMChanged),
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
            preloaded_resources: Default::default(),
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
            dirty_canvases: DomRefCell::new(Default::default()),
            has_pending_animated_image_update: Cell::new(false),
            selection: MutNullableDom::new(None),
            animation_timeline: if pref!(layout_animations_test_enabled) {
                DomRefCell::new(AnimationTimeline::new_for_testing())
            } else {
                DomRefCell::new(AnimationTimeline::new())
            },
            animations: Animations::new(),
            image_animation_manager: DomRefCell::new(ImageAnimationManager::default()),
            dirty_root: Default::default(),
            declarative_refresh: Default::default(),
            resize_observers: Default::default(),
            fonts: Default::default(),
            visibility_state: Cell::new(DocumentVisibilityState::Hidden),
            status_code,
            is_initial_about_blank: Cell::new(is_initial_about_blank),
            allow_declarative_shadow_roots: Cell::new(allow_declarative_shadow_roots),
            inherited_insecure_requests_policy: Cell::new(inherited_insecure_requests_policy),
            has_trustworthy_ancestor_origin: Cell::new(has_trustworthy_ancestor_origin),
            intersection_observer_task_queued: Cell::new(false),
            intersection_observers: Default::default(),
            highlighted_dom_node: Default::default(),
            adopted_stylesheets: Default::default(),
            adopted_stylesheets_frozen_types: CachedFrozenArray::new(),
            pending_scroll_event_targets: Default::default(),
            rendering_update_reasons: Default::default(),
            waiting_on_canvas_image_updates: Cell::new(false),
            current_rendering_epoch: Default::default(),
            custom_element_reaction_stack,
            active_sandboxing_flag_set: Cell::new(SandboxingFlagSet::empty()),
            creation_sandboxing_flag_set: Cell::new(creation_sandboxing_flag_set),
            favicon: RefCell::new(None),
            websockets: DOMTracker::new(),
            details_name_groups: Default::default(),
            protocol_handler_automation_mode: Default::default(),
            layout_animations_test_enabled: pref!(layout_animations_test_enabled),
            state_override: Default::default(),
            value_override: Default::default(),
        }
    }

    /// Returns a policy value that should be used for fetches initiated by this document.
    pub(crate) fn insecure_requests_policy(&self) -> InsecureRequestsPolicy {
        if let Some(csp_list) = self.get_csp_list().as_ref() {
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

    /// Get the [`Document`]'s [`DocumentEventHandler`].
    pub(crate) fn event_handler(&self) -> &DocumentEventHandler {
        &self.event_handler
    }

    /// Get the [`Document`]'s [`DocumentEmbedderControls`].
    pub(crate) fn embedder_controls(&self) -> &DocumentEmbedderControls {
        &self.embedder_controls
    }

    /// Whether or not this [`Document`] has any pending scroll events to be processed during
    /// "update the rendering."
    fn has_pending_scroll_events(&self) -> bool {
        !self.pending_scroll_event_targets.borrow().is_empty()
    }

    /// Add a [`RenderingUpdateReason`] to this [`Document`] which will trigger a
    /// rendering update at a later time.
    pub(crate) fn add_rendering_update_reason(&self, reason: RenderingUpdateReason) {
        self.rendering_update_reasons
            .set(self.rendering_update_reasons.get().union(reason));
    }

    /// Clear all [`RenderingUpdateReason`]s from this [`Document`].
    pub(crate) fn clear_rendering_update_reasons(&self) {
        self.rendering_update_reasons
            .set(RenderingUpdateReason::empty())
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

    #[expect(unsafe_code)]
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
            let mut cx = unsafe { script_bindings::script_runtime::temp_cx() };
            task.run_box(&mut cx);
        }
    }

    /// Enqueue a task to run as soon as any JS and layout blockers are removed.
    pub(crate) fn add_delayed_task<T: 'static + NonSendTaskBox>(&self, task: T) {
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
        about_base_url: Option<ServoUrl>,
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
        allow_declarative_shadow_roots: bool,
        inherited_insecure_requests_policy: Option<InsecureRequestsPolicy>,
        has_trustworthy_ancestor_origin: bool,
        custom_element_reaction_stack: Rc<CustomElementReactionStack>,
        creation_sandboxing_flag_set: SandboxingFlagSet,
        can_gc: CanGc,
    ) -> DomRoot<Document> {
        Self::new_with_proto(
            window,
            None,
            has_browsing_context,
            url,
            about_base_url,
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
            allow_declarative_shadow_roots,
            inherited_insecure_requests_policy,
            has_trustworthy_ancestor_origin,
            custom_element_reaction_stack,
            creation_sandboxing_flag_set,
            can_gc,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn new_with_proto(
        window: &Window,
        proto: Option<HandleObject>,
        has_browsing_context: HasBrowsingContext,
        url: Option<ServoUrl>,
        about_base_url: Option<ServoUrl>,
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
        allow_declarative_shadow_roots: bool,
        inherited_insecure_requests_policy: Option<InsecureRequestsPolicy>,
        has_trustworthy_ancestor_origin: bool,
        custom_element_reaction_stack: Rc<CustomElementReactionStack>,
        creation_sandboxing_flag_set: SandboxingFlagSet,
        can_gc: CanGc,
    ) -> DomRoot<Document> {
        let document = reflect_dom_object_with_proto(
            Box::new(Document::new_inherited(
                window,
                has_browsing_context,
                url,
                about_base_url,
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
                allow_declarative_shadow_roots,
                inherited_insecure_requests_policy,
                has_trustworthy_ancestor_origin,
                custom_element_reaction_stack,
                creation_sandboxing_flag_set,
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
        element.get_name().is_some_and(|n| &*n == name)
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
                    self.allow_declarative_shadow_roots(),
                    Some(self.insecure_requests_policy()),
                    self.has_trustworthy_ancestor_or_current_origin(),
                    self.custom_element_reaction_stack.clone(),
                    self.creation_sandboxing_flag_set(),
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

    pub(crate) fn ensure_pending_restyle(&self, el: &Element) -> RefMut<'_, PendingRestyle> {
        let map = self.pending_restyles.borrow_mut();
        RefMut::map(map, |m| {
            &mut m
                .entry(Dom::from_ref(el))
                .or_insert_with(|| NoTrace(PendingRestyle::default()))
                .0
        })
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

    /// <https://fullscreen.spec.whatwg.org/#dom-element-requestfullscreen>
    pub(crate) fn enter_fullscreen(&self, pending: &Element, can_gc: CanGc) -> Rc<Promise> {
        // Step 1
        // > Let pendingDoc be this’s node document.
        // `Self` is the pending document.

        // Step 2
        // > Let promise be a new promise.
        let in_realm_proof = AlreadyInRealm::assert::<crate::DomTypeHolder>();
        let promise = Promise::new_in_current_realm(InRealm::Already(&in_realm_proof), can_gc);

        // Step 3
        // > If pendingDoc is not fully active, then reject promise with a TypeError exception and return promise.
        if !self.is_fully_active() {
            promise.reject_error(
                Error::Type(c"Document is not fully active".to_owned()),
                can_gc,
            );
            return promise;
        }

        // Step 4
        // > Let error be false.
        let mut error = false;

        // Step 5
        // > If any of the following conditions are false, then set error to true:
        {
            // > - This’s namespace is the HTML namespace or this is an SVG svg or MathML math element. [SVG] [MATHML]
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

            // > - This is not a dialog element.
            if pending.is::<HTMLDialogElement>() {
                error = true;
            }

            // > - The fullscreen element ready check for this returns true.
            if !pending.fullscreen_element_ready_check() {
                error = true;
            }

            // > - Fullscreen is supported.
            // <https://fullscreen.spec.whatwg.org/#fullscreen-is-supported>
            // > Fullscreen is supported if there is no previously-established user preference, security risk, or platform limitation.
            // TODO: Add checks for whether fullscreen is supported as definition.

            // > - This’s relevant global object has transient activation or the algorithm is triggered by a user generated orientation change.
            // TODO: implement screen orientation API
            if !pending.owner_window().has_transient_activation() {
                error = true;
            }
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

        // Step 6
        // > If error is false, then consume user activation given pendingDoc’s relevant global object.
        if !error {
            pending.owner_window().consume_user_activation();
        }

        // Step 8.
        // > If error is false, then resize pendingDoc’s node navigable’s top-level traversable’s active document’s viewport’s dimensions,
        // > optionally taking into account options["navigationUI"]:
        // TODO(#21600): Improve spec compliance of steps 7-13 paralelism.
        // TODO(#42064): Implement fullscreen options, and ensure that this is spec compliant for all embedder.
        if !error {
            let event = EmbedderMsg::NotifyFullscreenStateChanged(self.webview_id(), true);
            self.send_to_embedder(event);
        }

        // Step 7
        // > Return promise, and run the remaining steps in parallel.
        let pipeline_id = self.window().pipeline_id();

        let trusted_pending = Trusted::new(pending);
        let trusted_pending_doc = Trusted::new(self);
        let trusted_promise = TrustedPromise::new(promise.clone());
        let handler = ElementPerformFullscreenEnter::new(
            trusted_pending,
            trusted_pending_doc,
            trusted_promise,
            error,
        );
        let script_msg = CommonScriptMsg::Task(
            ScriptThreadEventCategory::EnterFullscreen,
            handler,
            Some(pipeline_id),
            TaskSourceName::DOMManipulation,
        );
        let msg = MainThreadScriptMsg::Common(script_msg);
        self.window().main_thread_script_chan().send(msg).unwrap();

        promise
    }

    /// <https://fullscreen.spec.whatwg.org/#exit-fullscreen>
    pub(crate) fn exit_fullscreen(&self, can_gc: CanGc) -> Rc<Promise> {
        let global = self.global();

        // Step 1
        // > Let promise be a new promise
        let in_realm_proof = AlreadyInRealm::assert::<crate::DomTypeHolder>();
        let promise = Promise::new_in_current_realm(InRealm::Already(&in_realm_proof), can_gc);

        // Step 2
        // > If doc is not fully active or doc’s fullscreen element is null, then reject promise with a TypeError exception and return promise.
        if !self.is_fully_active() || self.fullscreen_element.get().is_none() {
            promise.reject_error(
                Error::Type(
                    c"No fullscreen element to exit or document is not fully active".to_owned(),
                ),
                can_gc,
            );
            return promise;
        }

        // TODO(#42067): Implement step 3-7, handling fullscreen's propagation across navigables.

        let element = self.fullscreen_element.get().unwrap();
        let window = self.window();

        // Step 10
        // > If resize is true, resize doc’s viewport to its "normal" dimensions.
        // TODO(#21600): Improve spec compliance of steps 8-15 paralelism.
        let event = EmbedderMsg::NotifyFullscreenStateChanged(self.webview_id(), false);
        self.send_to_embedder(event);

        // Step 8
        // > Return promise, and run the remaining steps in parallel.
        let trusted_element = Trusted::new(&*element);
        let trusted_promise = TrustedPromise::new(promise.clone());
        let handler = ElementPerformFullscreenExit::new(trusted_element, trusted_promise);
        let pipeline_id = Some(global.pipeline_id());
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

    fn reset_form_owner_for_listeners(&self, id: &Atom, can_gc: CanGc) {
        let map = self.form_id_listener_map.borrow();
        if let Some(listeners) = map.get(id) {
            for listener in listeners {
                listener
                    .as_maybe_form_control()
                    .expect("Element must be a form control")
                    .reset_form_owner(can_gc);
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
            .and_then(|s| s.owner.get_cssom_object())
    }

    /// Add a stylesheet owned by `owner_node` to the list of document sheets, in the
    /// correct tree position. Additionally, ensure that the owned stylesheet is inserted
    /// before any constructed stylesheet.
    ///
    /// <https://drafts.csswg.org/cssom/#documentorshadowroot-final-css-style-sheets>
    #[cfg_attr(crown, expect(crown::unrooted_must_root))] // Owner needs to be rooted already necessarily.
    pub(crate) fn add_owned_stylesheet(&self, owner_node: &Element, sheet: Arc<Stylesheet>) {
        let stylesheets = &mut *self.stylesheets.borrow_mut();

        // FIXME(stevennovaryo): This is almost identical with the one in ShadowRoot::add_stylesheet.
        let insertion_point = stylesheets
            .iter()
            .map(|(sheet, _origin)| sheet)
            .find(|sheet_in_doc| {
                match &sheet_in_doc.owner {
                    StylesheetSource::Element(other_node) => {
                        owner_node.upcast::<Node>().is_before(other_node.upcast())
                    },
                    // Non-constructed stylesheet should be ordered before the
                    // constructed ones.
                    StylesheetSource::Constructed(_) => true,
                }
            })
            .cloned();

        if self.has_browsing_context() {
            let document_context = self.window.web_font_context();

            self.window.layout_mut().add_stylesheet(
                sheet.clone(),
                insertion_point.as_ref().map(|s| s.sheet.clone()),
                &document_context,
            );
        }

        DocumentOrShadowRoot::add_stylesheet(
            StylesheetSource::Element(Dom::from_ref(owner_node)),
            StylesheetSetRef::Document(stylesheets),
            sheet,
            insertion_point,
            self.style_shared_lock(),
        );
    }

    /// Append a constructed stylesheet to the back of document stylesheet set. Because
    /// it would be the last element, we therefore would not mess with the ordering.
    ///
    /// <https://drafts.csswg.org/cssom/#documentorshadowroot-final-css-style-sheets>
    #[cfg_attr(crown, expect(crown::unrooted_must_root))]
    pub(crate) fn append_constructed_stylesheet(&self, cssom_stylesheet: &CSSStyleSheet) {
        debug_assert!(cssom_stylesheet.is_constructed());

        let stylesheets = &mut *self.stylesheets.borrow_mut();
        let sheet = cssom_stylesheet.style_stylesheet().clone();

        let insertion_point = stylesheets
            .iter()
            .last()
            .map(|(sheet, _origin)| sheet)
            .cloned();

        if self.has_browsing_context() {
            self.window.layout_mut().add_stylesheet(
                sheet.clone(),
                insertion_point.as_ref().map(|s| s.sheet.clone()),
                &self.window.web_font_context(),
            );
        }

        DocumentOrShadowRoot::add_stylesheet(
            StylesheetSource::Constructed(Dom::from_ref(cssom_stylesheet)),
            StylesheetSetRef::Document(stylesheets),
            sheet,
            insertion_point,
            self.style_shared_lock(),
        );
    }

    /// Given a stylesheet, load all web fonts from it in Layout.
    pub(crate) fn load_web_fonts_from_stylesheet(
        &self,
        stylesheet: &Arc<Stylesheet>,
        document_context: &WebFontDocumentContext,
    ) {
        self.window
            .layout()
            .load_web_fonts_from_stylesheet(stylesheet, document_context);
    }

    /// Remove a stylesheet owned by `owner` from the list of document sheets.
    #[cfg_attr(crown, expect(crown::unrooted_must_root))] // Owner needs to be rooted already necessarily.
    pub(crate) fn remove_stylesheet(&self, owner: StylesheetSource, stylesheet: &Arc<Stylesheet>) {
        if self.has_browsing_context() {
            self.window
                .layout_mut()
                .remove_stylesheet(stylesheet.clone());
        }

        DocumentOrShadowRoot::remove_stylesheet(
            owner,
            stylesheet,
            StylesheetSetRef::Document(&mut *self.stylesheets.borrow_mut()),
        )
    }

    pub(crate) fn get_elements_with_id(&self, id: &Atom) -> Ref<'_, [Dom<Element>]> {
        Ref::map(self.id_map.borrow(), |map| {
            map.get(id).map(|vec| &**vec).unwrap_or_default()
        })
    }

    pub(crate) fn get_elements_with_name(&self, name: &Atom) -> Ref<'_, [Dom<Element>]> {
        Ref::map(self.name_map.borrow(), |map| {
            map.get(name).map(|vec| &**vec).unwrap_or_default()
        })
    }

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

    pub(crate) fn advance_animation_timeline_for_testing(&self, delta: f64) {
        self.animation_timeline.borrow_mut().advance_specific(delta);
        let current_timeline_value = self.current_animation_timeline_value();
        self.animations
            .update_for_new_timeline_value(&self.window, current_timeline_value);
    }

    pub(crate) fn maybe_mark_animating_nodes_as_dirty(&self) {
        let current_timeline_value = self.current_animation_timeline_value();
        self.animations
            .mark_animating_nodes_as_dirty(current_timeline_value);
    }

    pub(crate) fn current_animation_timeline_value(&self) -> f64 {
        self.animation_timeline.borrow().current_value()
    }

    pub(crate) fn animations(&self) -> &Animations {
        &self.animations
    }

    pub(crate) fn update_animations_post_reflow(&self) {
        self.animations
            .do_post_reflow_update(&self.window, self.current_animation_timeline_value());
        self.image_animation_manager
            .borrow()
            .maybe_schedule_update_after_layout(
                &self.window,
                self.current_animation_timeline_value(),
            );
    }

    pub(crate) fn cancel_animations_for_node(&self, node: &Node) {
        self.animations.cancel_animations_for_node(node);
        self.image_animation_manager
            .borrow()
            .cancel_animations_for_node(node);
    }

    /// An implementation of <https://drafts.csswg.org/web-animations-1/#update-animations-and-send-events>.
    pub(crate) fn update_animations_and_send_events(&self, cx: &mut js::context::JSContext) {
        // Only update the time if it isn't being managed by a test.
        if !self.layout_animations_test_enabled {
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
            .update_for_new_timeline_value(&self.window, current_timeline_value);
        self.maybe_mark_animating_nodes_as_dirty();

        // > 3. Perform a microtask checkpoint.
        self.window().perform_a_microtask_checkpoint(cx);

        // Steps 4 through 7 occur inside `send_pending_events().`
        let _realm = enter_realm(self);
        self.animations()
            .send_pending_events(self.window(), CanGc::from_cx(cx));
    }

    pub(crate) fn image_animation_manager(&self) -> Ref<'_, ImageAnimationManager> {
        self.image_animation_manager.borrow()
    }

    pub(crate) fn set_has_pending_animated_image_update(&self) {
        self.has_pending_animated_image_update.set(true);
    }

    /// <https://html.spec.whatwg.org/multipage/#shared-declarative-refresh-steps>
    pub(crate) fn shared_declarative_refresh_steps(&self, content: &[u8]) {
        // 1. If document's will declaratively refresh is true, then return.
        if self.will_declaratively_refresh() {
            return;
        }

        // 2-11 Parsing
        static REFRESH_REGEX: LazyLock<Regex> = LazyLock::new(|| {
            // s flag is used to match . on newlines since the only places we use . in the
            // regex is to go "to end of the string"
            // (?s-u:.) is used to consume invalid unicode bytes
            Regex::new(
                r#"(?xs)
                    ^
                    \s* # 3
                    ((?<time>[0-9]+)|\.) # 5-6
                    [0-9.]* # 8
                    (
                        (
                            (\s*;|\s*,|\s) # 10.3
                            \s* # 10.4
                        )
                        (
                            (
                                (U|u)(R|r)(L|l) # 11.2-11.4
                                \s*=\s* # 11.5-11.7
                            )?
                        ('(?<url1>[^']*)'(?s-u:.)*|"(?<url2>[^"]*)"(?s-u:.)*|['"]?(?<url3>(?s-u:.)*)) # 11.8 - 11.10
                        |
                        (?<url4>(?s-u:.)*)
                    )
                )?
                $
            "#,
            )
            .unwrap()
        });

        // 9. Let urlRecord be document's URL.
        let mut url_record = self.url();
        let captures = if let Some(captures) = REFRESH_REGEX.captures(content) {
            captures
        } else {
            return;
        };
        let time = if let Some(time_string) = captures.name("time") {
            u64::from_str(&String::from_utf8_lossy(time_string.as_bytes())).unwrap_or(0)
        } else {
            0
        };
        let captured_url = captures.name("url1").or(captures
            .name("url2")
            .or(captures.name("url3").or(captures.name("url4"))));

        // 11.11 Parse: Set urlRecord to the result of encoding-parsing a URL given urlString, relative to document.
        if let Some(url_match) = captured_url {
            url_record = if let Ok(url) = ServoUrl::parse_with_base(
                Some(&url_record),
                &String::from_utf8_lossy(url_match.as_bytes()),
            ) {
                info!("Refresh to {}", url.debug_compact());
                url
            } else {
                // 11.12 If urlRecord is failure, then return.
                return;
            }
        }
        // 12. Set document's will declaratively refresh to true.
        if self.completely_loaded() {
            // TODO: handle active sandboxing flag
            self.window.as_global_scope().schedule_callback(
                OneshotTimerCallback::RefreshRedirectDue(RefreshRedirectDue {
                    window: DomRoot::from_ref(self.window()),
                    url: url_record,
                }),
                Duration::from_secs(time),
            );
            self.set_declarative_refresh(DeclarativeRefresh::CreatedAfterLoad);
        } else {
            self.set_declarative_refresh(DeclarativeRefresh::PendingLoad {
                url: url_record,
                time,
            });
        }
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
            .queue_entry(entry.upcast::<PerformanceEntry>());

        // Step 4 Run the screen orientation change steps with document.
        // TODO ScreenOrientation hasn't implemented yet

        // Step 5 Run the view transition page visibility change steps with document.
        // TODO ViewTransition hasn't implemented yet

        // Step 6 Run any page visibility change steps which may be defined in other specifications, with visibility
        // state and document. Any other specs' visibility steps will go here.

        // <https://www.w3.org/TR/gamepad/#handling-visibility-change>
        #[cfg(feature = "gamepad")]
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

    /// <https://dom.spec.whatwg.org/#document-allow-declarative-shadow-roots>
    pub(crate) fn allow_declarative_shadow_roots(&self) -> bool {
        self.allow_declarative_shadow_roots.get()
    }

    pub(crate) fn has_trustworthy_ancestor_origin(&self) -> bool {
        self.has_trustworthy_ancestor_origin.get()
    }

    pub(crate) fn has_trustworthy_ancestor_or_current_origin(&self) -> bool {
        self.has_trustworthy_ancestor_origin.get() ||
            self.origin().immutable().is_potentially_trustworthy()
    }

    pub(crate) fn highlight_dom_node(&self, node: Option<&Node>) {
        self.highlighted_dom_node.set(node);
        self.add_restyle_reason(RestyleReason::HighlightedDOMNodeChanged);
    }

    pub(crate) fn highlighted_dom_node(&self) -> Option<DomRoot<Node>> {
        self.highlighted_dom_node.get()
    }

    pub(crate) fn custom_element_reaction_stack(&self) -> Rc<CustomElementReactionStack> {
        self.custom_element_reaction_stack.clone()
    }

    pub(crate) fn has_active_sandboxing_flag(&self, flag: SandboxingFlagSet) -> bool {
        self.active_sandboxing_flag_set.get().contains(flag)
    }

    pub(crate) fn set_active_sandboxing_flag_set(&self, flags: SandboxingFlagSet) {
        self.active_sandboxing_flag_set.set(flags)
    }

    pub(crate) fn creation_sandboxing_flag_set(&self) -> SandboxingFlagSet {
        self.creation_sandboxing_flag_set.get()
    }

    pub(crate) fn creation_sandboxing_flag_set_considering_parent_iframe(
        &self,
    ) -> SandboxingFlagSet {
        self.window()
            .window_proxy()
            .frame_element()
            .and_then(|element| element.downcast::<HTMLIFrameElement>())
            .map(HTMLIFrameElement::sandboxing_flag_set)
            .unwrap_or_else(|| self.creation_sandboxing_flag_set())
    }

    pub(crate) fn viewport_scrolling_box(&self, flags: ScrollContainerQueryFlags) -> ScrollingBox {
        self.window()
            .scrolling_box_query(None, flags)
            .expect("We should always have a ScrollingBox for the Viewport")
    }

    pub(crate) fn notify_embedder_favicon(&self) {
        if let Some(ref image) = *self.favicon.borrow() {
            self.send_to_embedder(EmbedderMsg::NewFavicon(self.webview_id(), image.clone()));
        }
    }

    pub(crate) fn set_favicon(&self, favicon: Image) {
        *self.favicon.borrow_mut() = Some(favicon);
        self.notify_embedder_favicon();
    }

    pub(crate) fn fullscreen_element(&self) -> Option<DomRoot<Element>> {
        self.fullscreen_element.get()
    }

    /// <https://w3c.github.io/editing/docs/execCommand/#state-override>
    pub(crate) fn state_override(&self) -> bool {
        self.state_override.get()
    }

    /// <https://w3c.github.io/editing/docs/execCommand/#value-override>
    pub(crate) fn value_override(&self) -> Option<DOMString> {
        self.value_override.borrow().clone()
    }
}

impl DocumentMethods<crate::DomTypeHolder> for Document {
    /// <https://dom.spec.whatwg.org/#dom-document-document>
    fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<Document>> {
        // The new Document() constructor steps are to set this’s origin to the origin of current global object’s associated Document. [HTML]
        let doc = window.Document();
        let docloader = DocumentLoader::new(&doc.loader());
        Ok(Document::new_with_proto(
            window,
            proto,
            HasBrowsingContext::No,
            None,
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
            doc.allow_declarative_shadow_roots(),
            Some(doc.insecure_requests_policy()),
            doc.has_trustworthy_ancestor_or_current_origin(),
            doc.custom_element_reaction_stack(),
            doc.active_sandboxing_flag_set.get(),
            can_gc,
        ))
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-parsehtmlunsafe>
    fn ParseHTMLUnsafe(
        cx: &mut js::context::JSContext,
        window: &Window,
        s: TrustedHTMLOrString,
    ) -> Fallible<DomRoot<Self>> {
        // Step 1. Let compliantHTML be the result of invoking the
        // Get Trusted Type compliant string algorithm with TrustedHTML, the current global object,
        // html, "Document parseHTMLUnsafe", and "script".
        let compliant_html = TrustedHTML::get_trusted_script_compliant_string(
            window.as_global_scope(),
            s,
            "Document parseHTMLUnsafe",
            CanGc::from_cx(cx),
        )?;

        let url = window.get_url();
        let doc = window.Document();
        let loader = DocumentLoader::new(&doc.loader());

        let content_type = "text/html"
            .parse()
            .expect("Supported type is not a MIME type");
        // Step 2. Let document be a new Document, whose content type is "text/html".
        // Step 3. Set document's allow declarative shadow roots to true.
        let document = Document::new(
            window,
            HasBrowsingContext::No,
            Some(ServoUrl::parse("about:blank").unwrap()),
            None,
            doc.origin().clone(),
            IsHTMLDocument::HTMLDocument,
            Some(content_type),
            None,
            DocumentActivity::Inactive,
            DocumentSource::FromParser,
            loader,
            None,
            None,
            Default::default(),
            false,
            true,
            Some(doc.insecure_requests_policy()),
            doc.has_trustworthy_ancestor_or_current_origin(),
            doc.custom_element_reaction_stack(),
            doc.creation_sandboxing_flag_set(),
            CanGc::from_cx(cx),
        );
        // Step 4. Parse HTML from string given document and compliantHTML.
        ServoParser::parse_html_document(&document, Some(compliant_html), url, None, None, cx);
        // Step 5. Return document.
        document.set_ready_state(DocumentReadyState::Complete, CanGc::from_cx(cx));
        Ok(document)
    }

    /// <https://drafts.csswg.org/cssom/#dom-document-stylesheets>
    fn StyleSheets(&self, can_gc: CanGc) -> DomRoot<StyleSheetList> {
        self.stylesheet_list.or_init(|| {
            StyleSheetList::new(
                &self.window,
                StyleSheetListOwner::Document(Dom::from_ref(self)),
                can_gc,
            )
        })
    }

    /// <https://dom.spec.whatwg.org/#dom-document-implementation>
    fn Implementation(&self, can_gc: CanGc) -> DomRoot<DOMImplementation> {
        self.implementation
            .or_init(|| DOMImplementation::new(self, can_gc))
    }

    /// <https://dom.spec.whatwg.org/#dom-document-url>
    fn URL(&self) -> USVString {
        USVString(String::from(self.url().as_str()))
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-document-activeelement>
    fn GetActiveElement(&self) -> Option<DomRoot<Element>> {
        self.document_or_shadow_root.get_active_element(
            self.get_focused_element(),
            self.GetBody(),
            self.GetDocumentElement(),
        )
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-document-hasfocus>
    fn HasFocus(&self) -> bool {
        // <https://html.spec.whatwg.org/multipage/#has-focus-steps>
        //
        // > The has focus steps, given a `Document` object `target`, are as
        // > follows:
        // >
        // > 1. If `target`'s browsing context's top-level browsing context does
        // >    not have system focus, then return false.

        // > 2. Let `candidate` be `target`'s browsing context's top-level
        // >    browsing context's active document.
        // >
        // > 3. While true:
        // >
        // >    3.1. If `candidate` is target, then return true.
        // >
        // >    3.2. If the focused area of `candidate` is a browsing context
        // >         container with a non-null nested browsing context, then set
        // >         `candidate` to the active document of that browsing context
        // >         container's nested browsing context.
        // >
        // >    3.3. Otherwise, return false.
        if self.window().parent_info().is_none() {
            // 2 → 3 → (3.1 || ⋯ → 3.3)
            self.is_fully_active()
        } else {
            // 2 → 3 → 3.2 → (⋯ → 3.1 || ⋯ → 3.3)
            self.is_fully_active() && self.has_focus.get()
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-document-domain>
    fn Domain(&self) -> DOMString {
        // Step 1. Let effectiveDomain be this's origin's effective domain.
        match self.origin.effective_domain() {
            // Step 2. If effectiveDomain is null, then return the empty string.
            None => DOMString::new(),
            // Step 3. Return effectiveDomain, serialized.
            Some(Host::Domain(domain)) => DOMString::from(domain),
            Some(host) => DOMString::from(host.to_string()),
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-document-domain>
    fn SetDomain(&self, value: DOMString) -> ErrorResult {
        // Step 1. If this's browsing context is null, then throw a "SecurityError" DOMException.
        if !self.has_browsing_context {
            return Err(Error::Security(None));
        }

        // Step 2. If this Document object's active sandboxing flag set has its sandboxed
        // document.domain browsing context flag set, then throw a "SecurityError" DOMException.
        if self.has_active_sandboxing_flag(
            SandboxingFlagSet::SANDBOXED_DOCUMENT_DOMAIN_BROWSING_CONTEXT_FLAG,
        ) {
            return Err(Error::Security(None));
        }

        // Step 3. Let effectiveDomain be this's origin's effective domain.
        let effective_domain = match self.origin.effective_domain() {
            Some(effective_domain) => effective_domain,
            // Step 4. If effectiveDomain is null, then throw a "SecurityError" DOMException.
            None => return Err(Error::Security(None)),
        };

        // Step 5. If the given value is not a registrable domain suffix of and is not equal to effectiveDomain, then throw a "SecurityError" DOMException.
        let host =
            match get_registrable_domain_suffix_of_or_is_equal_to(&value.str(), effective_domain) {
                None => return Err(Error::Security(None)),
                Some(host) => host,
            };

        // Step 6. If the surrounding agent's agent cluster's is origin-keyed is true, then return.
        // TODO

        // Step 7. Set this's origin's domain to the result of parsing the given value.
        self.origin.set_domain(host);

        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-document-referrer>
    fn Referrer(&self) -> DOMString {
        match self.referrer {
            Some(ref referrer) => DOMString::from(referrer.to_string()),
            None => DOMString::new(),
        }
    }

    /// <https://dom.spec.whatwg.org/#dom-document-documenturi>
    fn DocumentURI(&self) -> USVString {
        self.URL()
    }

    /// <https://dom.spec.whatwg.org/#dom-document-compatmode>
    fn CompatMode(&self) -> DOMString {
        DOMString::from(match self.quirks_mode.get() {
            QuirksMode::LimitedQuirks | QuirksMode::NoQuirks => "CSS1Compat",
            QuirksMode::Quirks => "BackCompat",
        })
    }

    /// <https://dom.spec.whatwg.org/#dom-document-characterset>
    fn CharacterSet(&self) -> DOMString {
        DOMString::from(self.encoding.get().name())
    }

    /// <https://dom.spec.whatwg.org/#dom-document-charset>
    fn Charset(&self) -> DOMString {
        self.CharacterSet()
    }

    /// <https://dom.spec.whatwg.org/#dom-document-inputencoding>
    fn InputEncoding(&self) -> DOMString {
        self.CharacterSet()
    }

    /// <https://dom.spec.whatwg.org/#dom-document-content_type>
    fn ContentType(&self) -> DOMString {
        DOMString::from(self.content_type.to_string())
    }

    /// <https://dom.spec.whatwg.org/#dom-document-doctype>
    fn GetDoctype(&self) -> Option<DomRoot<DocumentType>> {
        self.upcast::<Node>().children().find_map(DomRoot::downcast)
    }

    /// <https://dom.spec.whatwg.org/#dom-document-documentelement>
    fn GetDocumentElement(&self) -> Option<DomRoot<Element>> {
        self.upcast::<Node>().child_elements().next()
    }

    /// <https://dom.spec.whatwg.org/#dom-document-getelementsbytagname>
    fn GetElementsByTagName(
        &self,
        qualified_name: DOMString,
        can_gc: CanGc,
    ) -> DomRoot<HTMLCollection> {
        let qualified_name = LocalName::from(qualified_name);
        if let Some(entry) = self.tag_map.borrow_mut().get(&qualified_name) {
            return DomRoot::from_ref(entry);
        }
        let result = HTMLCollection::by_qualified_name(
            &self.window,
            self.upcast(),
            qualified_name.clone(),
            can_gc,
        );
        self.tag_map
            .borrow_mut()
            .insert(qualified_name, Dom::from_ref(&*result));
        result
    }

    /// <https://dom.spec.whatwg.org/#dom-document-getelementsbytagnamens>
    fn GetElementsByTagNameNS(
        &self,
        maybe_ns: Option<DOMString>,
        tag_name: DOMString,
        can_gc: CanGc,
    ) -> DomRoot<HTMLCollection> {
        let ns = namespace_from_domstring(maybe_ns);
        let local = LocalName::from(tag_name);
        let qname = QualName::new(None, ns, local);
        if let Some(collection) = self.tagns_map.borrow().get(&qname) {
            return DomRoot::from_ref(collection);
        }
        let result =
            HTMLCollection::by_qual_tag_name(&self.window, self.upcast(), qname.clone(), can_gc);
        self.tagns_map
            .borrow_mut()
            .insert(qname, Dom::from_ref(&*result));
        result
    }

    /// <https://dom.spec.whatwg.org/#dom-document-getelementsbyclassname>
    fn GetElementsByClassName(&self, classes: DOMString, can_gc: CanGc) -> DomRoot<HTMLCollection> {
        let class_atoms: Vec<Atom> = split_html_space_chars(&classes.str())
            .map(Atom::from)
            .collect();
        if let Some(collection) = self.classes_map.borrow().get(&class_atoms) {
            return DomRoot::from_ref(collection);
        }
        let result = HTMLCollection::by_atomic_class_name(
            &self.window,
            self.upcast(),
            class_atoms.clone(),
            can_gc,
        );
        self.classes_map
            .borrow_mut()
            .insert(class_atoms, Dom::from_ref(&*result));
        result
    }

    /// <https://dom.spec.whatwg.org/#dom-nonelementparentnode-getelementbyid>
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
        // Step 1. If localName is not a valid element local name,
        //      then throw an "InvalidCharacterError" DOMException.
        if !is_valid_element_local_name(&local_name.str()) {
            debug!("Not a valid element name");
            return Err(Error::InvalidCharacter(None));
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
                options.is.as_ref().map(LocalName::from)
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
        // Step 1. Let (namespace, prefix, localName) be the result of
        //      validating and extracting namespace and qualifiedName given "element".
        let context = domname::Context::Element;
        let (namespace, prefix, local_name) =
            domname::validate_and_extract(namespace, &qualified_name, context)?;

        // Step 2. Let is be null.
        // Step 3. If options is a dictionary and options["is"] exists, then set is to it.
        let name = QualName::new(prefix, namespace, local_name);
        let is = match options {
            StringOrElementCreationOptions::String(_) => None,
            StringOrElementCreationOptions::ElementCreationOptions(options) => {
                options.is.as_ref().map(LocalName::from)
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
        // Step 1. If localName is not a valid attribute local name,
        //      then throw an "InvalidCharacterError" DOMException
        if !is_valid_attribute_local_name(&local_name.str()) {
            debug!("Not a valid attribute name");
            return Err(Error::InvalidCharacter(None));
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

    /// <https://dom.spec.whatwg.org/#dom-document-createattributens>
    fn CreateAttributeNS(
        &self,
        namespace: Option<DOMString>,
        qualified_name: DOMString,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<Attr>> {
        // Step 1. Let (namespace, prefix, localName) be the result of validating and
        //      extracting namespace and qualifiedName given "attribute".
        let context = domname::Context::Attribute;
        let (namespace, prefix, local_name) =
            domname::validate_and_extract(namespace, &qualified_name, context)?;
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

    /// <https://dom.spec.whatwg.org/#dom-document-createdocumentfragment>
    fn CreateDocumentFragment(&self, can_gc: CanGc) -> DomRoot<DocumentFragment> {
        DocumentFragment::new(self, can_gc)
    }

    /// <https://dom.spec.whatwg.org/#dom-document-createtextnode>
    fn CreateTextNode(&self, data: DOMString, can_gc: CanGc) -> DomRoot<Text> {
        Text::new(data, self, can_gc)
    }

    /// <https://dom.spec.whatwg.org/#dom-document-createcdatasection>
    fn CreateCDATASection(
        &self,
        data: DOMString,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<CDATASection>> {
        // Step 1
        if self.is_html_document {
            return Err(Error::NotSupported(None));
        }

        // Step 2
        if data.contains("]]>") {
            return Err(Error::InvalidCharacter(None));
        }

        // Step 3
        Ok(CDATASection::new(data, self, can_gc))
    }

    /// <https://dom.spec.whatwg.org/#dom-document-createcomment>
    fn CreateComment(&self, data: DOMString, can_gc: CanGc) -> DomRoot<Comment> {
        Comment::new(data, self, None, can_gc)
    }

    /// <https://dom.spec.whatwg.org/#dom-document-createprocessinginstruction>
    fn CreateProcessingInstruction(
        &self,
        target: DOMString,
        data: DOMString,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<ProcessingInstruction>> {
        // Step 1. If target does not match the Name production, then throw an "InvalidCharacterError" DOMException.
        if !matches_name_production(&target.str()) {
            return Err(Error::InvalidCharacter(None));
        }

        // Step 2.
        if data.contains("?>") {
            return Err(Error::InvalidCharacter(None));
        }

        // Step 3.
        Ok(ProcessingInstruction::new(target, data, self, can_gc))
    }

    /// <https://dom.spec.whatwg.org/#dom-document-importnode>
    fn ImportNode(
        &self,
        node: &Node,
        options: BooleanOrImportNodeOptions,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<Node>> {
        // Step 1. If node is a document or shadow root, then throw a "NotSupportedError" DOMException.
        if node.is::<Document>() || node.is::<ShadowRoot>() {
            return Err(Error::NotSupported(None));
        }
        // Step 2. Let subtree be false.
        let (subtree, registry) = match options {
            // Step 3. Let registry be null.
            // Step 4. If options is a boolean, then set subtree to options.
            BooleanOrImportNodeOptions::Boolean(boolean) => (boolean.into(), None),
            // Step 5. Otherwise:
            BooleanOrImportNodeOptions::ImportNodeOptions(options) => {
                // Step 5.1. Set subtree to the negation of options["selfOnly"].
                let subtree = (!options.selfOnly).into();
                // Step 5.2. If options["customElementRegistry"] exists, then set registry to it.
                let registry = options.customElementRegistry;
                // Step 5.3. If registry’s is scoped is false and registry
                // is not this’s custom element registry, then throw a "NotSupportedError" DOMException.
                // TODO
                (subtree, registry)
            },
        };
        // Step 6. If registry is null, then set registry to the
        // result of looking up a custom element registry given this.
        let registry = registry
            .or_else(|| CustomElementRegistry::lookup_a_custom_element_registry(self.upcast()));

        // Step 7. Return the result of cloning a node given node with
        // document set to this, subtree set to subtree, and fallbackRegistry set to registry.
        Ok(Node::clone(node, Some(self), subtree, registry, can_gc))
    }

    /// <https://dom.spec.whatwg.org/#dom-document-adoptnode>
    fn AdoptNode(&self, node: &Node, can_gc: CanGc) -> Fallible<DomRoot<Node>> {
        // Step 1.
        if node.is::<Document>() {
            return Err(Error::NotSupported(None));
        }

        // Step 2.
        if node.is::<ShadowRoot>() {
            return Err(Error::HierarchyRequest(None));
        }

        // Step 3.
        Node::adopt(node, self, can_gc);

        // Step 4.
        Ok(DomRoot::from_ref(node))
    }

    /// <https://dom.spec.whatwg.org/#dom-document-createevent>
    fn CreateEvent(&self, mut interface: DOMString, can_gc: CanGc) -> Fallible<DomRoot<Event>> {
        interface.make_ascii_lowercase();
        match &*interface.str() {
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
            _ => Err(Error::NotSupported(None)),
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-document-lastmodified>
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

    /// <https://dom.spec.whatwg.org/#dom-document-createrange>
    fn CreateRange(&self, can_gc: CanGc) -> DomRoot<Range> {
        Range::new_with_doc(self, None, can_gc)
    }

    /// <https://dom.spec.whatwg.org/#dom-document-createnodeiteratorroot-whattoshow-filter>
    fn CreateNodeIterator(
        &self,
        root: &Node,
        what_to_show: u32,
        filter: Option<Rc<NodeFilter>>,
        can_gc: CanGc,
    ) -> DomRoot<NodeIterator> {
        NodeIterator::new(self, root, what_to_show, filter, can_gc)
    }

    /// <https://dom.spec.whatwg.org/#dom-document-createtreewalker>
    fn CreateTreeWalker(
        &self,
        root: &Node,
        what_to_show: u32,
        filter: Option<Rc<NodeFilter>>,
    ) -> DomRoot<TreeWalker> {
        TreeWalker::new(self, root, what_to_show, filter)
    }

    /// <https://html.spec.whatwg.org/multipage/#document.title>
    fn Title(&self) -> DOMString {
        self.title().unwrap_or_else(|| DOMString::from(""))
    }

    /// <https://html.spec.whatwg.org/multipage/#document.title>
    fn SetTitle(&self, title: DOMString, can_gc: CanGc) {
        let root = match self.GetDocumentElement() {
            Some(root) => root,
            None => return,
        };

        let node = if root.namespace() == &ns!(svg) && root.local_name() == &local_name!("svg") {
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
                        .InsertBefore(child, parent.GetFirstChild().as_deref(), can_gc)
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
                        head.upcast::<Node>()
                            .AppendChild(elem.upcast(), can_gc)
                            .unwrap()
                    },
                    None => return,
                },
            }
        } else {
            return;
        };

        node.set_text_content_for_element(Some(title), can_gc);
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-document-head>
    fn GetHead(&self) -> Option<DomRoot<HTMLHeadElement>> {
        self.get_html_element()
            .and_then(|root| root.upcast::<Node>().children().find_map(DomRoot::downcast))
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-document-currentscript>
    fn GetCurrentScript(&self) -> Option<DomRoot<HTMLScriptElement>> {
        self.current_script.get()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-document-body>
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

    /// <https://html.spec.whatwg.org/multipage/#dom-document-body>
    fn SetBody(&self, new_body: Option<&HTMLElement>, can_gc: CanGc) -> ErrorResult {
        // Step 1.
        let new_body = match new_body {
            Some(new_body) => new_body,
            None => return Err(Error::HierarchyRequest(None)),
        };

        let node = new_body.upcast::<Node>();
        match node.type_id() {
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLBodyElement)) |
            NodeTypeId::Element(ElementTypeId::HTMLElement(
                HTMLElementTypeId::HTMLFrameSetElement,
            )) => {},
            _ => return Err(Error::HierarchyRequest(None)),
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
                root.ReplaceChild(new_body.upcast(), child.upcast(), can_gc)
                    .unwrap();
            },

            // Step 4.
            (None, _) => return Err(Error::HierarchyRequest(None)),

            // Step 5.
            (Some(ref root), &None) => {
                let root = root.upcast::<Node>();
                root.AppendChild(new_body.upcast(), can_gc).unwrap();
            },
        }
        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-document-getelementsbyname>
    fn GetElementsByName(&self, name: DOMString, can_gc: CanGc) -> DomRoot<NodeList> {
        NodeList::new_elements_by_name_list(self.window(), self, name, can_gc)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-document-images>
    fn Images(&self, can_gc: CanGc) -> DomRoot<HTMLCollection> {
        self.images.or_init(|| {
            HTMLCollection::new_with_filter_fn(
                &self.window,
                self.upcast(),
                |element, _| element.is::<HTMLImageElement>(),
                can_gc,
            )
        })
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-document-embeds>
    fn Embeds(&self, can_gc: CanGc) -> DomRoot<HTMLCollection> {
        self.embeds.or_init(|| {
            HTMLCollection::new_with_filter_fn(
                &self.window,
                self.upcast(),
                |element, _| element.is::<HTMLEmbedElement>(),
                can_gc,
            )
        })
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-document-plugins>
    fn Plugins(&self, can_gc: CanGc) -> DomRoot<HTMLCollection> {
        self.Embeds(can_gc)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-document-links>
    fn Links(&self, can_gc: CanGc) -> DomRoot<HTMLCollection> {
        self.links.or_init(|| {
            HTMLCollection::new_with_filter_fn(
                &self.window,
                self.upcast(),
                |element, _| {
                    (element.is::<HTMLAnchorElement>() || element.is::<HTMLAreaElement>()) &&
                        element.has_attribute(&local_name!("href"))
                },
                can_gc,
            )
        })
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-document-forms>
    fn Forms(&self, can_gc: CanGc) -> DomRoot<HTMLCollection> {
        self.forms.or_init(|| {
            HTMLCollection::new_with_filter_fn(
                &self.window,
                self.upcast(),
                |element, _| element.is::<HTMLFormElement>(),
                can_gc,
            )
        })
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-document-scripts>
    fn Scripts(&self, can_gc: CanGc) -> DomRoot<HTMLCollection> {
        self.scripts.or_init(|| {
            HTMLCollection::new_with_filter_fn(
                &self.window,
                self.upcast(),
                |element, _| element.is::<HTMLScriptElement>(),
                can_gc,
            )
        })
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-document-anchors>
    fn Anchors(&self, can_gc: CanGc) -> DomRoot<HTMLCollection> {
        self.anchors.or_init(|| {
            HTMLCollection::new_with_filter_fn(
                &self.window,
                self.upcast(),
                |element, _| {
                    element.is::<HTMLAnchorElement>() && element.has_attribute(&local_name!("href"))
                },
                can_gc,
            )
        })
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-document-applets>
    fn Applets(&self, can_gc: CanGc) -> DomRoot<HTMLCollection> {
        self.applets
            .or_init(|| HTMLCollection::always_empty(&self.window, self.upcast(), can_gc))
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-document-location>
    fn GetLocation(&self) -> Option<DomRoot<Location>> {
        if self.is_fully_active() {
            Some(self.window.Location())
        } else {
            None
        }
    }

    /// <https://dom.spec.whatwg.org/#dom-parentnode-children>
    fn Children(&self, can_gc: CanGc) -> DomRoot<HTMLCollection> {
        HTMLCollection::children(&self.window, self.upcast(), can_gc)
    }

    /// <https://dom.spec.whatwg.org/#dom-parentnode-firstelementchild>
    fn GetFirstElementChild(&self) -> Option<DomRoot<Element>> {
        self.upcast::<Node>().child_elements().next()
    }

    /// <https://dom.spec.whatwg.org/#dom-parentnode-lastelementchild>
    fn GetLastElementChild(&self) -> Option<DomRoot<Element>> {
        self.upcast::<Node>()
            .rev_children()
            .find_map(DomRoot::downcast)
    }

    /// <https://dom.spec.whatwg.org/#dom-parentnode-childelementcount>
    fn ChildElementCount(&self) -> u32 {
        self.upcast::<Node>().child_elements().count() as u32
    }

    /// <https://dom.spec.whatwg.org/#dom-parentnode-prepend>
    fn Prepend(&self, nodes: Vec<NodeOrString>, can_gc: CanGc) -> ErrorResult {
        self.upcast::<Node>().prepend(nodes, can_gc)
    }

    /// <https://dom.spec.whatwg.org/#dom-parentnode-append>
    fn Append(&self, nodes: Vec<NodeOrString>, can_gc: CanGc) -> ErrorResult {
        self.upcast::<Node>().append(nodes, can_gc)
    }

    /// <https://dom.spec.whatwg.org/#dom-parentnode-replacechildren>
    fn ReplaceChildren(&self, nodes: Vec<NodeOrString>, can_gc: CanGc) -> ErrorResult {
        self.upcast::<Node>().replace_children(nodes, can_gc)
    }

    /// <https://dom.spec.whatwg.org/#dom-parentnode-queryselector>
    fn QuerySelector(&self, selectors: DOMString) -> Fallible<Option<DomRoot<Element>>> {
        let root = self.upcast::<Node>();
        root.query_selector(selectors)
    }

    /// <https://dom.spec.whatwg.org/#dom-parentnode-queryselectorall>
    fn QuerySelectorAll(&self, selectors: DOMString) -> Fallible<DomRoot<NodeList>> {
        let root = self.upcast::<Node>();
        root.query_selector_all(selectors)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-document-readystate>
    fn ReadyState(&self) -> DocumentReadyState {
        self.ready_state.get()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-document-defaultview>
    fn GetDefaultView(&self) -> Option<DomRoot<Window>> {
        if self.has_browsing_context {
            Some(DomRoot::from_ref(&*self.window))
        } else {
            None
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-document-cookie>
    fn GetCookie(&self) -> Fallible<DOMString> {
        if self.is_cookie_averse() {
            return Ok(DOMString::new());
        }

        if !self.origin.is_tuple() {
            return Err(Error::Security(None));
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

    /// <https://html.spec.whatwg.org/multipage/#dom-document-cookie>
    fn SetCookie(&self, cookie: DOMString) -> ErrorResult {
        if self.is_cookie_averse() {
            return Ok(());
        }

        if !self.origin.is_tuple() {
            return Err(Error::Security(None));
        }

        if !cookie.is_valid_for_cookie() {
            return Ok(());
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

    /// <https://html.spec.whatwg.org/multipage/#dom-document-bgcolor>
    fn BgColor(&self) -> DOMString {
        self.get_body_attribute(&local_name!("bgcolor"))
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-document-bgcolor>
    fn SetBgColor(&self, value: DOMString, can_gc: CanGc) {
        self.set_body_attribute(&local_name!("bgcolor"), value, can_gc)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-document-fgcolor>
    fn FgColor(&self) -> DOMString {
        self.get_body_attribute(&local_name!("text"))
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-document-fgcolor>
    fn SetFgColor(&self, value: DOMString, can_gc: CanGc) {
        self.set_body_attribute(&local_name!("text"), value, can_gc)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-tree-accessors:dom-document-nameditem-filter>
    fn NamedGetter(&self, name: DOMString, can_gc: CanGc) -> Option<NamedPropertyValue> {
        if name.is_empty() {
            return None;
        }
        let name = Atom::from(name);

        // Step 1. Let elements be the list of named elements with the name name that are in a document tree
        // with the Document as their root.
        let elements_with_name = self.get_elements_with_name(&name);
        let name_iter = elements_with_name
            .iter()
            .filter(|elem| is_named_element_with_name_attribute(elem));
        let elements_with_id = self.get_elements_with_id(&name);
        let id_iter = elements_with_id
            .iter()
            .filter(|elem| is_named_element_with_id_attribute(elem));
        let mut elements = name_iter.chain(id_iter);

        // Step 2. If elements has only one element, and that element is an iframe element,
        // and that iframe element's content navigable is not null, then return the active
        // WindowProxy of the element's content navigable.

        // NOTE: We have to check if all remaining elements are equal to the first, since
        // the same element may appear in both lists.
        let first = elements.next()?;
        if elements.all(|other| first == other) {
            if let Some(nested_window_proxy) = first
                .downcast::<HTMLIFrameElement>()
                .and_then(|iframe| iframe.GetContentWindow())
            {
                return Some(NamedPropertyValue::WindowProxy(nested_window_proxy));
            }

            // Step 3. Otherwise, if elements has only one element, return that element.
            return Some(NamedPropertyValue::Element(DomRoot::from_ref(first)));
        }

        // Step 4. Otherwise, return an HTMLCollection rooted at the Document node,
        // whose filter matches only named elements with the name name.
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
            can_gc,
        );
        Some(NamedPropertyValue::HTMLCollection(collection))
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-tree-accessors:supported-property-names>
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

    /// <https://html.spec.whatwg.org/multipage/#dom-document-clear>
    fn Clear(&self) {
        // This method intentionally does nothing
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-document-captureevents>
    fn CaptureEvents(&self) {
        // This method intentionally does nothing
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-document-releaseevents>
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

    /// <https://drafts.csswg.org/cssom-view/#dom-document-elementfrompoint>
    fn ElementFromPoint(&self, x: Finite<f64>, y: Finite<f64>) -> Option<DomRoot<Element>> {
        self.document_or_shadow_root.element_from_point(
            x,
            y,
            self.GetDocumentElement(),
            self.has_browsing_context,
        )
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-document-elementsfrompoint>
    fn ElementsFromPoint(&self, x: Finite<f64>, y: Finite<f64>) -> Vec<DomRoot<Element>> {
        self.document_or_shadow_root.elements_from_point(
            x,
            y,
            self.GetDocumentElement(),
            self.has_browsing_context,
        )
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-document-scrollingelement>
    fn GetScrollingElement(&self) -> Option<DomRoot<Element>> {
        // Step 1. If the Document is in quirks mode, follow these steps:
        if self.quirks_mode() == QuirksMode::Quirks {
            // Step 1.1. If the body element exists,
            if let Some(ref body) = self.GetBody() {
                let e = body.upcast::<Element>();
                // and it is not potentially scrollable, return the body element and abort these steps.
                // For this purpose, a value of overflow:clip on the body element’s parent element
                // must be treated as overflow:hidden.
                if !e.is_potentially_scrollable_body_for_scrolling_element() {
                    return Some(DomRoot::from_ref(e));
                }
            }

            // Step 1.2. Return null and abort these steps.
            return None;
        }

        // Step 2. If there is a root element, return the root element and abort these steps.
        // Step 3. Return null.
        self.GetDocumentElement()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-document-open>
    fn Open(
        &self,
        cx: &mut js::context::JSContext,
        _unused1: Option<DOMString>,
        _unused2: Option<DOMString>,
    ) -> Fallible<DomRoot<Document>> {
        // Step 1
        if !self.is_html_document() {
            return Err(Error::InvalidState(None));
        }

        // Step 2
        if self.throw_on_dynamic_markup_insertion_counter.get() > 0 {
            return Err(Error::InvalidState(None));
        }

        // Step 3
        let entry_responsible_document = GlobalScope::entry().as_window().Document();

        // Step 4
        if !self.origin.same_origin(&entry_responsible_document.origin) {
            return Err(Error::Security(None));
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
            self.abort(cx);
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
        Node::replace_all(None, self.upcast::<Node>(), CanGc::from_cx(cx));

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

    /// <https://html.spec.whatwg.org/multipage/#dom-document-open-window>
    fn Open_(
        &self,
        cx: &mut js::context::JSContext,
        url: USVString,
        target: DOMString,
        features: DOMString,
    ) -> Fallible<Option<DomRoot<WindowProxy>>> {
        self.browsing_context()
            .ok_or(Error::InvalidAccess(None))?
            .open(url, target, features, CanGc::from_cx(cx))
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-document-write>
    fn Write(
        &self,
        cx: &mut js::context::JSContext,
        text: Vec<TrustedHTMLOrString>,
    ) -> ErrorResult {
        // The document.write(...text) method steps are to run the document write steps
        // with this, text, false, and "Document write".
        self.write(cx, text, false, "Document", "write")
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-document-writeln>
    fn Writeln(
        &self,
        cx: &mut js::context::JSContext,
        text: Vec<TrustedHTMLOrString>,
    ) -> ErrorResult {
        // The document.writeln(...text) method steps are to run the document write steps
        // with this, text, true, and "Document writeln".
        self.write(cx, text, true, "Document", "writeln")
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-document-close>
    fn Close(&self, cx: &mut js::context::JSContext) -> ErrorResult {
        if !self.is_html_document() {
            // Step 1. If this is an XML document, then throw an "InvalidStateError" DOMException.
            return Err(Error::InvalidState(None));
        }

        // Step 2. If this's throw-on-dynamic-markup-insertion counter is greater than zero,
        // then throw an "InvalidStateError" DOMException.
        if self.throw_on_dynamic_markup_insertion_counter.get() > 0 {
            return Err(Error::InvalidState(None));
        }

        // Step 3. If there is no script-created parser associated with this, then return.
        let parser = match self.get_current_parser() {
            Some(ref parser) if parser.is_script_created() => DomRoot::from_ref(&**parser),
            _ => {
                return Ok(());
            },
        };

        // parser.close implements the remainder of this algorithm
        parser.close(cx);

        Ok(())
    }

    /// <https://w3c.github.io/editing/docs/execCommand/#execcommand()>
    fn ExecCommand(
        &self,
        command_id: DOMString,
        _show_ui: bool,
        value: TrustedHTMLOrString,
        can_gc: CanGc,
    ) -> Fallible<bool> {
        let value = if command_id == "insertHTML" {
            TrustedHTML::get_trusted_script_compliant_string(
                self.window.as_global_scope(),
                value,
                "Document execCommand",
                can_gc,
            )?
        } else {
            match value {
                TrustedHTMLOrString::TrustedHTML(trusted_html) => trusted_html.data().clone(),
                TrustedHTMLOrString::String(value) => value,
            }
        };

        Ok(self.exec_command_for_command_id(command_id, value, can_gc))
    }

    /// <https://w3c.github.io/editing/docs/execCommand/#querycommandenabled()>
    fn QueryCommandEnabled(&self, command_id: DOMString, can_gc: CanGc) -> bool {
        // Step 2. Return true if command is both supported and enabled, false otherwise.
        self.check_support_and_enabled(command_id, can_gc).is_some()
    }

    /// <https://w3c.github.io/editing/docs/execCommand/#querycommandsupported()>
    fn QueryCommandSupported(&self, command_id: DOMString) -> bool {
        // > When the queryCommandSupported(command) method on the Document interface is invoked,
        // the user agent must return true if command is supported and available
        // within the current script on the current site, and false otherwise.
        self.is_command_supported(command_id)
    }

    /// <https://w3c.github.io/editing/docs/execCommand/#querycommandindeterm()>
    fn QueryCommandIndeterm(&self, command_id: DOMString) -> bool {
        self.is_command_indeterminate(command_id)
    }

    /// <https://w3c.github.io/editing/docs/execCommand/#querycommandstate()>
    fn QueryCommandState(&self, command_id: DOMString) -> bool {
        self.command_state_for_command(command_id)
    }

    /// <https://w3c.github.io/editing/docs/execCommand/#querycommandvalue()>
    fn QueryCommandValue(&self, command_id: DOMString) -> DOMString {
        self.command_value_for_command(command_id)
    }

    // https://fullscreen.spec.whatwg.org/#handler-document-onfullscreenerror
    event_handler!(fullscreenerror, GetOnfullscreenerror, SetOnfullscreenerror);

    // https://fullscreen.spec.whatwg.org/#handler-document-onfullscreenchange
    event_handler!(
        fullscreenchange,
        GetOnfullscreenchange,
        SetOnfullscreenchange
    );

    /// <https://fullscreen.spec.whatwg.org/#dom-document-fullscreenenabled>
    fn FullscreenEnabled(&self) -> bool {
        self.get_allow_fullscreen()
    }

    /// <https://fullscreen.spec.whatwg.org/#dom-document-fullscreen>
    fn Fullscreen(&self) -> bool {
        self.fullscreen_element.get().is_some()
    }

    /// <https://fullscreen.spec.whatwg.org/#dom-document-fullscreenelement>
    fn GetFullscreenElement(&self) -> Option<DomRoot<Element>> {
        DocumentOrShadowRoot::get_fullscreen_element(&self.node, self.fullscreen_element.get())
    }

    /// <https://fullscreen.spec.whatwg.org/#dom-document-exitfullscreen>
    fn ExitFullscreen(&self, can_gc: CanGc) -> Rc<Promise> {
        self.exit_fullscreen(can_gc)
    }

    // check-tidy: no specs after this line
    // Servo only API to get an instance of the controls of a specific
    // media element matching the given id.
    fn ServoGetMediaControls(&self, id: DOMString) -> Fallible<DomRoot<ShadowRoot>> {
        match self.media_controls.borrow().get(&*id.str()) {
            Some(m) => Ok(DomRoot::from_ref(m)),
            None => Err(Error::InvalidAccess(None)),
        }
    }

    /// <https://w3c.github.io/selection-api/#dom-document-getselection>
    fn GetSelection(&self, can_gc: CanGc) -> Option<DomRoot<Selection>> {
        if self.has_browsing_context {
            Some(self.selection.or_init(|| Selection::new(self, can_gc)))
        } else {
            None
        }
    }

    /// <https://drafts.csswg.org/css-font-loading/#font-face-source>
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
        let parsed_expression =
            parse_expression(&expression.str(), resolver, self.is_html_document())?;
        Ok(XPathExpression::new(
            &self.window,
            None,
            can_gc,
            parsed_expression,
        ))
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
        result_type: u16,
        result: Option<&super::types::XPathResult>,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<super::types::XPathResult>> {
        let parsed_expression =
            parse_expression(&expression.str(), resolver, self.is_html_document())?;
        XPathExpression::new(&self.window, None, can_gc, parsed_expression).evaluate_internal(
            context_node,
            result_type,
            result,
            can_gc,
        )
    }

    /// <https://drafts.csswg.org/cssom/#dom-documentorshadowroot-adoptedstylesheets>
    fn AdoptedStyleSheets(&self, context: JSContext, can_gc: CanGc, retval: MutableHandleValue) {
        self.adopted_stylesheets_frozen_types.get_or_init(
            || {
                self.adopted_stylesheets
                    .borrow()
                    .clone()
                    .iter()
                    .map(|sheet| sheet.as_rooted())
                    .collect()
            },
            context,
            retval,
            can_gc,
        );
    }

    /// <https://drafts.csswg.org/cssom/#dom-documentorshadowroot-adoptedstylesheets>
    fn SetAdoptedStyleSheets(
        &self,
        context: JSContext,
        val: HandleValue,
        can_gc: CanGc,
    ) -> ErrorResult {
        let result = DocumentOrShadowRoot::set_adopted_stylesheet_from_jsval(
            context,
            self.adopted_stylesheets.borrow_mut().as_mut(),
            val,
            &StyleSheetListOwner::Document(Dom::from_ref(self)),
            can_gc,
        );

        // If update is successful, clear the FrozenArray cache.
        if result.is_ok() {
            self.adopted_stylesheets_frozen_types.clear()
        }

        result
    }
}

fn update_with_current_instant(marker: &Cell<Option<CrossProcessInstant>>) {
    if marker.get().is_none() {
        marker.set(Some(CrossProcessInstant::now()))
    }
}

/// Specifies the type of focus event that is sent to a pipeline
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum FocusType {
    Element, // The first focus message - focus the element itself
    Parent,  // Focusing a parent element (an iframe)
}

/// Specifies the initiator of a focus operation.
#[derive(Clone, Copy, PartialEq)]
pub enum FocusInitiator {
    /// The operation is initiated by this document and to be broadcasted
    /// through the constellation.
    Local,
    /// The operation is initiated somewhere else, and we are updating our
    /// internal state accordingly.
    Remote,
}

/// Focus events
pub(crate) enum FocusEventType {
    Focus, // Element gained focus. Doesn't bubble.
    Blur,  // Element lost focus. Doesn't bubble.
}

#[derive(JSTraceable, MallocSizeOf)]
pub(crate) enum AnimationFrameCallback {
    DevtoolsFramerateTick {
        actor_name: String,
    },
    FrameRequestCallback {
        #[conditional_malloc_size_of]
        callback: Rc<FrameRequestCallback>,
    },
}

impl AnimationFrameCallback {
    fn call(&self, document: &Document, now: f64, can_gc: CanGc) {
        match *self {
            AnimationFrameCallback::DevtoolsFramerateTick { ref actor_name } => {
                let msg = ScriptToDevtoolsControlMsg::FramerateTick(actor_name.clone(), now);
                let devtools_sender = document.window().as_global_scope().devtools_chan().unwrap();
                devtools_sender.send(msg).unwrap();
            },
            AnimationFrameCallback::FrameRequestCallback { ref callback } => {
                // TODO(jdm): The spec says that any exceptions should be suppressed:
                // https://github.com/servo/servo/issues/6928
                let _ = callback.Call__(Finite::wrap(now), ExceptionHandling::Report, can_gc);
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

impl DocumentHelpers for Document {
    fn ensure_safe_to_run_script_or_layout(&self) {
        Document::ensure_safe_to_run_script_or_layout(self)
    }
}

/// Iterator for same origin ancestor navigables, returning the active documents of the navigables.
/// <https://html.spec.whatwg.org/multipage/#ancestor-navigables>
// TODO: Find a way for something equivalent for cross origin document.
pub(crate) struct SameoriginAncestorNavigablesIterator {
    document: DomRoot<Document>,
}

impl SameoriginAncestorNavigablesIterator {
    pub(crate) fn new(document: DomRoot<Document>) -> Self {
        Self { document }
    }
}

impl Iterator for SameoriginAncestorNavigablesIterator {
    type Item = DomRoot<Document>;

    fn next(&mut self) -> Option<Self::Item> {
        let window_proxy = self.document.browsing_context()?;
        self.document = window_proxy.parent()?.document()?;
        Some(self.document.clone())
    }
}

/// Iterator for same origin descendant navigables in a shadow-including tree order, returning the
/// active documents of the navigables.
/// <https://html.spec.whatwg.org/multipage/#descendant-navigables>
// TODO: Find a way for something equivalent for cross origin document.
pub(crate) struct SameOriginDescendantNavigablesIterator {
    stack: Vec<Box<dyn Iterator<Item = DomRoot<HTMLIFrameElement>>>>,
}

impl SameOriginDescendantNavigablesIterator {
    pub(crate) fn new(document: DomRoot<Document>) -> Self {
        let iframes: Vec<DomRoot<HTMLIFrameElement>> = document.iframes().iter().collect();
        Self {
            stack: vec![Box::new(iframes.into_iter())],
        }
    }

    fn get_next_iframe(&mut self) -> Option<DomRoot<HTMLIFrameElement>> {
        let mut cur_iframe = self.stack.last_mut()?.next();
        while cur_iframe.is_none() {
            self.stack.pop();
            cur_iframe = self.stack.last_mut()?.next();
        }
        cur_iframe
    }
}

impl Iterator for SameOriginDescendantNavigablesIterator {
    type Item = DomRoot<Document>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(iframe) = self.get_next_iframe() {
            let Some(pipeline_id) = iframe.pipeline_id() else {
                continue;
            };

            if let Some(document) = ScriptThread::find_document(pipeline_id) {
                let child_iframes: Vec<DomRoot<HTMLIFrameElement>> =
                    document.iframes().iter().collect();
                self.stack.push(Box::new(child_iframes.into_iter()));
                return Some(document);
            } else {
                continue;
            };
        }
        None
    }
}
