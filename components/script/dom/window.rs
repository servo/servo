/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::ToOwned;
use std::cell::{Cell, RefCell, RefMut};
use std::cmp;
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::default::Default;
use std::ffi::c_void;
use std::io::{Write, stderr, stdout};
use std::rc::Rc;
use std::sync::Arc;
use std::time::{Duration, Instant};

use app_units::Au;
use backtrace::Backtrace;
use base::cross_process_instant::CrossProcessInstant;
use base::generic_channel::{self, GenericCallback, GenericSender};
use base::id::{BrowsingContextId, PipelineId, WebViewId};
use base64::Engine;
#[cfg(feature = "bluetooth")]
use bluetooth_traits::BluetoothRequest;
use canvas_traits::webgl::WebGLChan;
use compositing_traits::CrossProcessPaintApi;
use constellation_traits::{
    LoadData, LoadOrigin, NavigationHistoryBehavior, ScreenshotReadinessResponse,
    ScriptToConstellationChan, ScriptToConstellationMessage, StructuredSerializedData,
    WindowSizeType,
};
use content_security_policy::Violation;
use content_security_policy::sandboxing_directive::SandboxingFlagSet;
use crossbeam_channel::{Sender, unbounded};
use cssparser::SourceLocation;
use devtools_traits::{ScriptToDevtoolsControlMsg, TimelineMarker, TimelineMarkerType};
use dom_struct::dom_struct;
use embedder_traits::user_content_manager::{UserContentManager, UserScript};
use embedder_traits::{
    AlertResponse, ConfirmResponse, EmbedderMsg, JavaScriptEvaluationError, PromptResponse,
    ScriptToEmbedderChan, SimpleDialogRequest, Theme, UntrustedNodeAddress, ViewportDetails,
    WebDriverJSResult, WebDriverLoadStatus,
};
use euclid::default::{Point2D as UntypedPoint2D, Rect as UntypedRect};
use euclid::{Point2D, Scale, Size2D, Vector2D};
use fonts::{CspViolationHandler, FontContext, WebFontDocumentContext};
use ipc_channel::ipc::IpcSender;
use js::glue::DumpJSStack;
use js::jsapi::{
    GCReason, Heap, JS_GC, JSAutoRealm, JSContext as RawJSContext, JSObject, JSPROP_ENUMERATE,
};
use js::jsval::{NullValue, UndefinedValue};
use js::rust::wrappers::JS_DefineProperty;
use js::rust::{
    CustomAutoRooter, CustomAutoRooterGuard, HandleObject, HandleValue, MutableHandleObject,
    MutableHandleValue,
};
use layout_api::{
    BoxAreaType, ElementsFromPointFlags, ElementsFromPointResult, FragmentType, Layout,
    LayoutImageDestination, PendingImage, PendingImageState, PendingRasterizationImage,
    PhysicalSides, QueryMsg, ReflowGoal, ReflowPhasesRun, ReflowRequest, ReflowRequestRestyle,
    RestyleReason, ScrollContainerQueryFlags, ScrollContainerResponse, TrustedNodeAddress,
    combine_id_with_fragment_type,
};
use malloc_size_of::MallocSizeOf;
use media::WindowGLContext;
use net_traits::ResourceThreads;
use net_traits::image_cache::{
    ImageCache, ImageCacheResponseCallback, ImageCacheResponseMessage, ImageLoadListener,
    ImageResponse, PendingImageId, PendingImageResponse, RasterizationCompleteResponse,
};
use num_traits::ToPrimitive;
use profile_traits::generic_channel as ProfiledGenericChannel;
use profile_traits::mem::ProfilerChan as MemProfilerChan;
use profile_traits::time::ProfilerChan as TimeProfilerChan;
use rustc_hash::{FxBuildHasher, FxHashMap};
use script_bindings::codegen::GenericBindings::WindowBinding::ScrollToOptions;
use script_bindings::conversions::SafeToJSValConvertible;
use script_bindings::interfaces::WindowHelpers;
use script_bindings::root::Root;
use script_traits::{ConstellationInputEvent, ScriptThreadMessage};
use selectors::attr::CaseSensitivity;
use servo_arc::Arc as ServoArc;
use servo_config::pref;
use servo_geometry::DeviceIndependentIntRect;
use servo_url::{ImmutableOrigin, MutableOrigin, ServoUrl};
use storage_traits::StorageThreads;
use storage_traits::webstorage_thread::StorageType;
use style::error_reporting::{ContextualParseError, ParseErrorReporter};
use style::properties::PropertyId;
use style::properties::style_structs::Font;
use style::selector_parser::PseudoElement;
use style::str::HTML_SPACE_CHARACTERS;
use style::stylesheets::UrlExtraData;
use style_traits::CSSPixel;
use stylo_atoms::Atom;
use url::Position;
use webrender_api::ExternalScrollId;
use webrender_api::units::{DeviceIntSize, DevicePixel, LayoutPixel, LayoutPoint};

use super::bindings::codegen::Bindings::MessagePortBinding::StructuredSerializeOptions;
use super::bindings::trace::HashMapTracedValues;
use super::types::SVGSVGElement;
use crate::dom::bindings::cell::{DomRefCell, Ref};
use crate::dom::bindings::codegen::Bindings::DocumentBinding::{
    DocumentMethods, DocumentReadyState, NamedPropertyValue,
};
use crate::dom::bindings::codegen::Bindings::HTMLIFrameElementBinding::HTMLIFrameElementMethods;
use crate::dom::bindings::codegen::Bindings::HistoryBinding::History_Binding::HistoryMethods;
use crate::dom::bindings::codegen::Bindings::ImageBitmapBinding::{
    ImageBitmapOptions, ImageBitmapSource,
};
use crate::dom::bindings::codegen::Bindings::MediaQueryListBinding::MediaQueryList_Binding::MediaQueryListMethods;
use crate::dom::bindings::codegen::Bindings::ReportingObserverBinding::Report;
use crate::dom::bindings::codegen::Bindings::RequestBinding::{RequestInfo, RequestInit};
use crate::dom::bindings::codegen::Bindings::VoidFunctionBinding::VoidFunction;
use crate::dom::bindings::codegen::Bindings::WindowBinding::{
    self, DeferredRequestInit, FrameRequestCallback, ScrollBehavior, WindowMethods,
    WindowPostMessageOptions,
};
use crate::dom::bindings::codegen::UnionTypes::{
    RequestOrUSVString, TrustedScriptOrString, TrustedScriptOrStringOrFunction,
};
use crate::dom::bindings::error::{
    Error, ErrorInfo, ErrorResult, Fallible, javascript_error_info_from_error_info,
};
use crate::dom::bindings::inheritance::{Castable, ElementTypeId, HTMLElementTypeId, NodeTypeId};
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::{DomGlobal, DomObject};
use crate::dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::bindings::structuredclone;
use crate::dom::bindings::trace::{CustomTraceable, JSTraceable, RootedTraceableBox};
use crate::dom::bindings::utils::GlobalStaticData;
use crate::dom::bindings::weakref::DOMTracker;
#[cfg(feature = "bluetooth")]
use crate::dom::bluetooth::BluetoothExtraPermissionData;
use crate::dom::cookiestore::CookieStore;
use crate::dom::crypto::Crypto;
use crate::dom::csp::GlobalCspReporting;
use crate::dom::css::cssstyledeclaration::{
    CSSModificationAccess, CSSStyleDeclaration, CSSStyleOwner,
};
use crate::dom::customelementregistry::CustomElementRegistry;
use crate::dom::document::{AnimationFrameCallback, Document};
use crate::dom::element::Element;
use crate::dom::event::{Event, EventBubbles, EventCancelable};
use crate::dom::eventtarget::EventTarget;
use crate::dom::fetchlaterresult::FetchLaterResult;
use crate::dom::globalscope::GlobalScope;
use crate::dom::hashchangeevent::HashChangeEvent;
use crate::dom::history::History;
use crate::dom::html::htmlcollection::{CollectionFilter, HTMLCollection};
use crate::dom::html::htmliframeelement::HTMLIFrameElement;
use crate::dom::idbfactory::IDBFactory;
use crate::dom::inputevent::HitTestResult;
use crate::dom::location::Location;
use crate::dom::medialist::MediaList;
use crate::dom::mediaquerylist::{MediaQueryList, MediaQueryListMatchState};
use crate::dom::mediaquerylistevent::MediaQueryListEvent;
use crate::dom::messageevent::MessageEvent;
use crate::dom::navigator::Navigator;
use crate::dom::node::{Node, NodeDamage, NodeTraits, from_untrusted_node_address};
use crate::dom::performance::performance::Performance;
use crate::dom::promise::Promise;
use crate::dom::reportingendpoint::{ReportingEndpoint, SendReportsToEndpoints};
use crate::dom::reportingobserver::ReportingObserver;
use crate::dom::screen::Screen;
use crate::dom::scrolling_box::{ScrollingBox, ScrollingBoxSource};
use crate::dom::selection::Selection;
use crate::dom::shadowroot::ShadowRoot;
use crate::dom::storage::Storage;
#[cfg(feature = "bluetooth")]
use crate::dom::testrunner::TestRunner;
use crate::dom::trustedtypepolicyfactory::TrustedTypePolicyFactory;
use crate::dom::types::{ImageBitmap, UIEvent};
use crate::dom::webgl::webglrenderingcontext::WebGLCommandSender;
#[cfg(feature = "webgpu")]
use crate::dom::webgpu::identityhub::IdentityHub;
use crate::dom::windowproxy::{WindowProxy, WindowProxyHandler};
use crate::dom::worklet::Worklet;
use crate::dom::workletglobalscope::WorkletGlobalScopeType;
use crate::layout_image::fetch_image_for_layout;
use crate::messaging::{MainThreadScriptMsg, ScriptEventLoopReceiver, ScriptEventLoopSender};
use crate::microtask::{Microtask, UserMicrotask};
use crate::realms::{InRealm, enter_realm};
use crate::script_runtime::{CanGc, JSContext, Runtime};
use crate::script_thread::{ScriptThread, with_script_thread};
use crate::script_window_proxies::ScriptWindowProxies;
use crate::task_source::SendableTaskSource;
use crate::timers::{IsInterval, TimerCallback};
use crate::unminify::unminified_path;
use crate::webdriver_handlers::{find_node_by_unique_id_in_document, jsval_to_webdriver};
use crate::{fetch, window_named_properties};

/// A callback to call when a response comes back from the `ImageCache`.
///
/// This is wrapped in a struct so that we can implement `MallocSizeOf`
/// for this type.
#[derive(MallocSizeOf)]
pub struct PendingImageCallback(
    #[ignore_malloc_size_of = "dyn Fn is currently impossible to measure"]
    Box<dyn Fn(PendingImageResponse) + 'static>,
);

/// Current state of the window object
#[derive(Clone, Copy, Debug, JSTraceable, MallocSizeOf, PartialEq)]
enum WindowState {
    Alive,
    Zombie, // Pipeline is closed, but the window hasn't been GCed yet.
}

/// How long we should wait before performing the initial reflow after `<body>` is parsed,
/// assuming that `<body>` take this long to parse.
const INITIAL_REFLOW_DELAY: Duration = Duration::from_millis(200);

/// During loading and parsing, layouts are suppressed to avoid flashing incomplete page
/// contents.
///
/// Exceptions:
///  - Parsing the body takes so long, that layouts are no longer suppressed in order
///    to show the user that the page is loading.
///  - Script triggers a layout query or scroll event in which case, we want to layout
///    but not display the contents.
///
/// For more information see: <https://github.com/servo/servo/pull/6028>.
#[derive(Clone, Copy, MallocSizeOf)]
enum LayoutBlocker {
    /// The first load event hasn't been fired and we have not started to parse the `<body>` yet.
    WaitingForParse,
    /// The body is being parsed the `<body>` starting at the `Instant` specified.
    Parsing(Instant),
    /// The body finished parsing and the `load` event has been fired or parsing took so
    /// long, that we are going to do layout anyway. Note that subsequent changes to the body
    /// can trigger parsing again, but the `Window` stays in this state.
    FiredLoadEventOrParsingTimerExpired,
}

impl LayoutBlocker {
    fn layout_blocked(&self) -> bool {
        !matches!(self, Self::FiredLoadEventOrParsingTimerExpired)
    }
}

/// An id used to cancel navigations; for now only used for planned form navigations.
/// Loosely based on <https://html.spec.whatwg.org/multipage/#ongoing-navigation>.
#[derive(Clone, Copy, Debug, Default, JSTraceable, MallocSizeOf, PartialEq)]
pub(crate) struct OngoingNavigation(u32);

type PendingImageRasterizationKey = (PendingImageId, DeviceIntSize);

/// Ancillary data of pending image request that was initiated by layout during a reflow.
/// This data is used to faciliate invalidating layout when the image data becomes available
/// at some point in the future.
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
#[derive(JSTraceable, MallocSizeOf)]
struct PendingLayoutImageAncillaryData {
    node: Dom<Node>,
    #[no_trace]
    destination: LayoutImageDestination,
}

#[dom_struct]
pub(crate) struct Window {
    globalscope: GlobalScope,
    /// The webview that contains this [`Window`].
    ///
    /// This may not be the top-level [`Window`], in the case of frames.
    #[no_trace]
    webview_id: WebViewId,
    script_chan: Sender<MainThreadScriptMsg>,
    #[no_trace]
    #[ignore_malloc_size_of = "TODO: Add MallocSizeOf support to layout"]
    layout: RefCell<Box<dyn Layout>>,
    navigator: MutNullableDom<Navigator>,
    #[ignore_malloc_size_of = "ImageCache"]
    #[no_trace]
    image_cache: Arc<dyn ImageCache>,
    #[no_trace]
    image_cache_sender: Sender<ImageCacheResponseMessage>,
    window_proxy: MutNullableDom<WindowProxy>,
    document: MutNullableDom<Document>,
    location: MutNullableDom<Location>,
    history: MutNullableDom<History>,
    indexeddb: MutNullableDom<IDBFactory>,
    custom_element_registry: MutNullableDom<CustomElementRegistry>,
    performance: MutNullableDom<Performance>,
    #[no_trace]
    navigation_start: Cell<CrossProcessInstant>,
    screen: MutNullableDom<Screen>,
    session_storage: MutNullableDom<Storage>,
    local_storage: MutNullableDom<Storage>,
    status: DomRefCell<DOMString>,
    trusted_types: MutNullableDom<TrustedTypePolicyFactory>,

    /// The start of something resembling
    /// <https://html.spec.whatwg.org/multipage/#ongoing-navigation>
    ongoing_navigation: Cell<OngoingNavigation>,

    /// For sending timeline markers. Will be ignored if
    /// no devtools server
    #[no_trace]
    devtools_markers: DomRefCell<HashSet<TimelineMarkerType>>,
    #[no_trace]
    devtools_marker_sender: DomRefCell<Option<GenericSender<Option<TimelineMarker>>>>,

    /// Most recent unhandled resize event, if any.
    #[no_trace]
    unhandled_resize_event: DomRefCell<Option<(ViewportDetails, WindowSizeType)>>,

    /// Platform theme.
    #[no_trace]
    theme: Cell<Theme>,

    /// Parent id associated with this page, if any.
    #[no_trace]
    parent_info: Option<PipelineId>,

    /// Global static data related to the DOM.
    dom_static: GlobalStaticData,

    /// The JavaScript runtime.
    #[conditional_malloc_size_of]
    js_runtime: DomRefCell<Option<Rc<Runtime>>>,

    /// The [`ViewportDetails`] of this [`Window`]'s frame.
    #[no_trace]
    viewport_details: Cell<ViewportDetails>,

    /// A handle for communicating messages to the bluetooth thread.
    #[no_trace]
    #[cfg(feature = "bluetooth")]
    bluetooth_thread: GenericSender<BluetoothRequest>,

    #[cfg(feature = "bluetooth")]
    bluetooth_extra_permission_data: BluetoothExtraPermissionData,

    /// See the documentation for [`LayoutBlocker`]. Essentially, this flag prevents
    /// layouts from happening before the first load event, apart from a few exceptional
    /// cases.
    #[no_trace]
    layout_blocker: Cell<LayoutBlocker>,

    /// A channel for communicating results of async scripts back to the webdriver server
    #[no_trace]
    webdriver_script_chan: DomRefCell<Option<IpcSender<WebDriverJSResult>>>,

    /// A channel to notify webdriver if there is a navigation
    #[no_trace]
    webdriver_load_status_sender: RefCell<Option<GenericSender<WebDriverLoadStatus>>>,

    /// The current state of the window object
    current_state: Cell<WindowState>,

    error_reporter: CSSErrorReporter,

    /// All the MediaQueryLists we need to update
    media_query_lists: DOMTracker<MediaQueryList>,

    #[cfg(feature = "bluetooth")]
    test_runner: MutNullableDom<TestRunner>,

    /// A handle for communicating messages to the WebGL thread, if available.
    #[ignore_malloc_size_of = "channels are hard"]
    #[no_trace]
    webgl_chan: Option<WebGLChan>,

    #[ignore_malloc_size_of = "defined in webxr"]
    #[no_trace]
    #[cfg(feature = "webxr")]
    webxr_registry: Option<webxr_api::Registry>,

    /// When an element triggers an image load or starts watching an image load from the
    /// `ImageCache` it adds an entry to this list. When those loads are triggered from
    /// layout, they also add an etry to [`Self::pending_layout_images`].
    #[no_trace]
    pending_image_callbacks: DomRefCell<FxHashMap<PendingImageId, Vec<PendingImageCallback>>>,

    /// All of the elements that have an outstanding image request that was
    /// initiated by layout during a reflow. They are stored in the [`ScriptThread`]
    /// to ensure that the element can be marked dirty when the image data becomes
    /// available at some point in the future.
    pending_layout_images: DomRefCell<
        HashMapTracedValues<PendingImageId, Vec<PendingLayoutImageAncillaryData>, FxBuildHasher>,
    >,

    /// Vector images for which layout has intiated rasterization at a specific size
    /// and whose results are not yet available. They are stored in the [`ScriptThread`]
    /// so that the element can be marked dirty once the rasterization is completed.
    pending_images_for_rasterization: DomRefCell<
        HashMapTracedValues<PendingImageRasterizationKey, Vec<Dom<Node>>, FxBuildHasher>,
    >,

    /// Directory to store unminified css for this window if unminify-css
    /// opt is enabled.
    unminified_css_dir: DomRefCell<Option<String>>,

    /// Directory with stored unminified scripts
    local_script_source: Option<String>,

    /// Worklets
    test_worklet: MutNullableDom<Worklet>,
    /// <https://drafts.css-houdini.org/css-paint-api-1/#paint-worklet>
    paint_worklet: MutNullableDom<Worklet>,

    /// Flag to identify whether mutation observers are present(true)/absent(false)
    exists_mut_observer: Cell<bool>,

    /// Cross-process access to `Paint`.
    #[ignore_malloc_size_of = "Wraps an IpcSender"]
    #[no_trace]
    paint_api: CrossProcessPaintApi,

    /// Indicate whether a SetDocumentStatus message has been sent after a reflow is complete.
    /// It is used to avoid sending idle message more than once, which is unnecessary.
    has_sent_idle_message: Cell<bool>,

    /// Unminify Css.
    unminify_css: bool,

    /// User content manager
    #[no_trace]
    user_content_manager: UserContentManager,

    /// Window's GL context from application
    #[ignore_malloc_size_of = "defined in script_thread"]
    #[no_trace]
    player_context: WindowGLContext,

    throttled: Cell<bool>,

    /// A shared marker for the validity of any cached layout values. A value of true
    /// indicates that any such values remain valid; any new layout that invalidates
    /// those values will cause the marker to be set to false.
    #[conditional_malloc_size_of]
    layout_marker: DomRefCell<Rc<Cell<bool>>>,

    /// <https://dom.spec.whatwg.org/#window-current-event>
    current_event: DomRefCell<Option<Dom<Event>>>,

    /// <https://w3c.github.io/reporting/#windoworworkerglobalscope-registered-reporting-observer-list>
    reporting_observer_list: DomRefCell<Vec<DomRoot<ReportingObserver>>>,

    /// <https://w3c.github.io/reporting/#windoworworkerglobalscope-reports>
    report_list: DomRefCell<Vec<Report>>,

    /// <https://w3c.github.io/reporting/#windoworworkerglobalscope-endpoints>
    #[no_trace]
    endpoints_list: DomRefCell<Vec<ReportingEndpoint>>,

    /// The window proxies the script thread knows.
    #[conditional_malloc_size_of]
    script_window_proxies: Rc<ScriptWindowProxies>,

    /// Whether or not this [`Window`] has a pending screenshot readiness request.
    has_pending_screenshot_readiness_request: Cell<bool>,
}

impl Window {
    pub(crate) fn webview_id(&self) -> WebViewId {
        self.webview_id
    }

    pub(crate) fn as_global_scope(&self) -> &GlobalScope {
        self.upcast::<GlobalScope>()
    }

    pub(crate) fn layout(&self) -> Ref<'_, Box<dyn Layout>> {
        self.layout.borrow()
    }

    pub(crate) fn layout_mut(&self) -> RefMut<'_, Box<dyn Layout>> {
        self.layout.borrow_mut()
    }

    pub(crate) fn get_exists_mut_observer(&self) -> bool {
        self.exists_mut_observer.get()
    }

    pub(crate) fn set_exists_mut_observer(&self) {
        self.exists_mut_observer.set(true);
    }

    #[expect(unsafe_code)]
    pub(crate) fn clear_js_runtime_for_script_deallocation(&self) {
        self.as_global_scope()
            .remove_web_messaging_and_dedicated_workers_infra();
        unsafe {
            *self.js_runtime.borrow_for_script_deallocation() = None;
            self.window_proxy.set(None);
            self.current_state.set(WindowState::Zombie);
            self.as_global_scope()
                .task_manager()
                .cancel_all_tasks_and_ignore_future_tasks();
        }
    }

    /// A convenience method for
    /// <https://html.spec.whatwg.org/multipage/#a-browsing-context-is-discarded>
    pub(crate) fn discard_browsing_context(&self) {
        let proxy = match self.window_proxy.get() {
            Some(proxy) => proxy,
            None => panic!("Discarding a BC from a window that has none"),
        };
        proxy.discard_browsing_context();
        // Step 4 of https://html.spec.whatwg.org/multipage/#discard-a-document
        // Other steps performed when the `PipelineExit` message
        // is handled by the ScriptThread.
        self.as_global_scope()
            .task_manager()
            .cancel_all_tasks_and_ignore_future_tasks();
    }

    /// Get a sender to the time profiler thread.
    pub(crate) fn time_profiler_chan(&self) -> &TimeProfilerChan {
        self.globalscope.time_profiler_chan()
    }

    pub(crate) fn origin(&self) -> &MutableOrigin {
        self.globalscope.origin()
    }

    #[expect(unsafe_code)]
    pub(crate) fn get_cx(&self) -> JSContext {
        unsafe { JSContext::from_ptr(js::rust::Runtime::get().unwrap().as_ptr()) }
    }

    pub(crate) fn get_js_runtime(&self) -> Ref<'_, Option<Rc<Runtime>>> {
        self.js_runtime.borrow()
    }

    pub(crate) fn main_thread_script_chan(&self) -> &Sender<MainThreadScriptMsg> {
        &self.script_chan
    }

    pub(crate) fn parent_info(&self) -> Option<PipelineId> {
        self.parent_info
    }

    pub(crate) fn new_script_pair(&self) -> (ScriptEventLoopSender, ScriptEventLoopReceiver) {
        let (sender, receiver) = unbounded();
        (
            ScriptEventLoopSender::MainThread(sender),
            ScriptEventLoopReceiver::MainThread(receiver),
        )
    }

    pub(crate) fn event_loop_sender(&self) -> ScriptEventLoopSender {
        ScriptEventLoopSender::MainThread(self.script_chan.clone())
    }

    pub(crate) fn image_cache(&self) -> Arc<dyn ImageCache> {
        self.image_cache.clone()
    }

    /// This can panic if it is called after the browsing context has been discarded
    pub(crate) fn window_proxy(&self) -> DomRoot<WindowProxy> {
        self.window_proxy.get().unwrap()
    }

    pub(crate) fn append_reporting_observer(&self, reporting_observer: DomRoot<ReportingObserver>) {
        self.reporting_observer_list
            .borrow_mut()
            .push(reporting_observer);
    }

    pub(crate) fn remove_reporting_observer(&self, reporting_observer: &ReportingObserver) {
        let index = {
            let list = self.reporting_observer_list.borrow();
            list.iter()
                .position(|observer| &**observer == reporting_observer)
        };

        if let Some(index) = index {
            self.reporting_observer_list.borrow_mut().remove(index);
        }
    }

    pub(crate) fn registered_reporting_observers(&self) -> Vec<DomRoot<ReportingObserver>> {
        self.reporting_observer_list.borrow().clone()
    }

    pub(crate) fn append_report(&self, report: Report) {
        self.report_list.borrow_mut().push(report);
        let trusted_window = Trusted::new(self);
        self.upcast::<GlobalScope>()
            .task_manager()
            .dom_manipulation_task_source()
            .queue(task!(send_to_reporting_endpoints: move || {
                let window = trusted_window.root();
                let reports = std::mem::take(&mut *window.report_list.borrow_mut());
                window.upcast::<GlobalScope>().send_reports_to_endpoints(
                    reports,
                    window.endpoints_list.borrow().clone(),
                );
            }));
    }

    pub(crate) fn buffered_reports(&self) -> Vec<Report> {
        self.report_list.borrow().clone()
    }

    pub(crate) fn set_endpoints_list(&self, endpoints: Vec<ReportingEndpoint>) {
        *self.endpoints_list.borrow_mut() = endpoints;
    }

    /// Returns the window proxy if it has not been discarded.
    /// <https://html.spec.whatwg.org/multipage/#a-browsing-context-is-discarded>
    pub(crate) fn undiscarded_window_proxy(&self) -> Option<DomRoot<WindowProxy>> {
        self.window_proxy.get().and_then(|window_proxy| {
            if window_proxy.is_browsing_context_discarded() {
                None
            } else {
                Some(window_proxy)
            }
        })
    }

    /// Returns the window proxy of the webview, which is the top-level ancestor browsing context.
    /// <https://html.spec.whatwg.org/multipage/#top-level-browsing-context>
    pub(crate) fn webview_window_proxy(&self) -> Option<DomRoot<WindowProxy>> {
        self.undiscarded_window_proxy().and_then(|window_proxy| {
            self.script_window_proxies
                .find_window_proxy(window_proxy.webview_id().into())
        })
    }

    #[cfg(feature = "bluetooth")]
    pub(crate) fn bluetooth_thread(&self) -> GenericSender<BluetoothRequest> {
        self.bluetooth_thread.clone()
    }

    #[cfg(feature = "bluetooth")]
    pub(crate) fn bluetooth_extra_permission_data(&self) -> &BluetoothExtraPermissionData {
        &self.bluetooth_extra_permission_data
    }

    pub(crate) fn css_error_reporter(&self) -> &CSSErrorReporter {
        &self.error_reporter
    }

    pub(crate) fn webgl_chan(&self) -> Option<WebGLCommandSender> {
        self.webgl_chan
            .as_ref()
            .map(|chan| WebGLCommandSender::new(chan.clone()))
    }

    #[cfg(feature = "webxr")]
    pub(crate) fn webxr_registry(&self) -> Option<webxr_api::Registry> {
        self.webxr_registry.clone()
    }

    fn new_paint_worklet(&self, can_gc: CanGc) -> DomRoot<Worklet> {
        debug!("Creating new paint worklet.");
        Worklet::new(self, WorkletGlobalScopeType::Paint, can_gc)
    }

    pub(crate) fn register_image_cache_listener(
        &self,
        id: PendingImageId,
        callback: impl Fn(PendingImageResponse) + 'static,
    ) -> ImageCacheResponseCallback {
        self.pending_image_callbacks
            .borrow_mut()
            .entry(id)
            .or_default()
            .push(PendingImageCallback(Box::new(callback)));

        let image_cache_sender = self.image_cache_sender.clone();
        Box::new(move |message| {
            let _ = image_cache_sender.send(message);
        })
    }

    fn pending_layout_image_notification(&self, response: PendingImageResponse) {
        let mut images = self.pending_layout_images.borrow_mut();
        let nodes = images.entry(response.id);
        let nodes = match nodes {
            Entry::Occupied(nodes) => nodes,
            Entry::Vacant(_) => return,
        };
        if matches!(
            response.response,
            ImageResponse::Loaded(_, _) | ImageResponse::FailedToLoadOrDecode
        ) {
            for ancillary_data in nodes.get() {
                match ancillary_data.destination {
                    LayoutImageDestination::BoxTreeConstruction => {
                        ancillary_data.node.dirty(NodeDamage::Other);
                    },
                    LayoutImageDestination::DisplayListBuilding => {
                        self.layout().set_needs_new_display_list();
                    },
                }
            }
        }

        match response.response {
            ImageResponse::MetadataLoaded(_) => {},
            ImageResponse::Loaded(_, _) | ImageResponse::FailedToLoadOrDecode => {
                nodes.remove();
            },
        }
    }

    pub(crate) fn handle_image_rasterization_complete_notification(
        &self,
        response: RasterizationCompleteResponse,
    ) {
        let mut images = self.pending_images_for_rasterization.borrow_mut();
        let nodes = images.entry((response.image_id, response.requested_size));
        let nodes = match nodes {
            Entry::Occupied(nodes) => nodes,
            Entry::Vacant(_) => return,
        };
        for node in nodes.get() {
            node.dirty(NodeDamage::Other);
        }
        nodes.remove();
    }

    pub(crate) fn pending_image_notification(&self, response: PendingImageResponse) {
        // We take the images here, in order to prevent maintaining a mutable borrow when
        // image callbacks are called. These, in turn, can trigger garbage collection.
        // Normally this shouldn't trigger more pending image notifications, but just in
        // case we do not want to cause a double borrow here.
        let mut images = std::mem::take(&mut *self.pending_image_callbacks.borrow_mut());
        let Entry::Occupied(callbacks) = images.entry(response.id) else {
            let _ = std::mem::replace(&mut *self.pending_image_callbacks.borrow_mut(), images);
            return;
        };

        for callback in callbacks.get() {
            callback.0(response.clone());
        }

        match response.response {
            ImageResponse::MetadataLoaded(_) => {},
            ImageResponse::Loaded(_, _) | ImageResponse::FailedToLoadOrDecode => {
                callbacks.remove();
            },
        }

        let _ = std::mem::replace(&mut *self.pending_image_callbacks.borrow_mut(), images);
    }

    pub(crate) fn paint_api(&self) -> &CrossProcessPaintApi {
        &self.paint_api
    }

    pub(crate) fn userscripts(&self) -> &[UserScript] {
        self.user_content_manager.scripts()
    }

    pub(crate) fn get_player_context(&self) -> WindowGLContext {
        self.player_context.clone()
    }

    // see note at https://dom.spec.whatwg.org/#concept-event-dispatch step 2
    pub(crate) fn dispatch_event_with_target_override(&self, event: &Event, can_gc: CanGc) {
        event.dispatch(self.upcast(), true, can_gc);
    }

    pub(crate) fn font_context(&self) -> &Arc<FontContext> {
        self.as_global_scope()
            .font_context()
            .expect("A `Window` should always have a `FontContext`")
    }

    pub(crate) fn ongoing_navigation(&self) -> OngoingNavigation {
        self.ongoing_navigation.get()
    }

    /// <https://html.spec.whatwg.org/multipage/#set-the-ongoing-navigation>
    pub(crate) fn set_ongoing_navigation(&self) -> OngoingNavigation {
        // Note: since this value, for now, is only used in a single `ScriptThread`,
        // we just increment it (it is not a uuid), which implies not
        // using a `newValue` variable.
        let new_value = self.ongoing_navigation.get().0.wrapping_add(1);

        // 1. If navigable's ongoing navigation is equal to newValue, then return.
        // Note: cannot happen in the way it is currently used.

        // TODO: 2. Inform the navigation API about aborting navigation given navigable.

        // 3. Set navigable's ongoing navigation to newValue.
        self.ongoing_navigation.set(OngoingNavigation(new_value));

        // Note: Return the ongoing navigation for the caller to use.
        OngoingNavigation(new_value)
    }

    /// <https://html.spec.whatwg.org/multipage/#nav-stop>
    fn stop_loading(&self, can_gc: CanGc) {
        // 1. Let document be navigable's active document.
        let doc = self.Document();

        // 2. If document's unload counter is 0,
        // and navigable's ongoing navigation is a navigation ID,
        // then set the ongoing navigation for navigable to null.
        //
        // Note: since the concept of `navigable` is nascent in Servo,
        // for now we do two things:
        // - increment the `ongoing_navigation`(preventing planned form navigations).
        // - Send a `AbortLoadUrl` message(in case the navigation
        // already started at the constellation).
        self.set_ongoing_navigation();

        // 3. Abort a document and its descendants given document.
        doc.abort(can_gc);
    }

    /// <https://html.spec.whatwg.org/multipage/#cannot-show-simple-dialogs>
    fn cannot_show_simple_dialogs(&self) -> bool {
        // Step 1: If the active sandboxing flag set of window's associated Document has
        // the sandboxed modals flag set, then return true.
        if self
            .Document()
            .has_active_sandboxing_flag(SandboxingFlagSet::SANDBOXED_MODALS_FLAG)
        {
            return true;
        }

        // Step 2: If window's relevant settings object's origin and window's relevant settings
        // object's top-level origin are not same origin-domain, then return true.
        //
        // TODO: This check doesn't work currently because it seems that comparing two
        // opaque domains doesn't work between GlobalScope::top_level_creation_url and
        // Document::origin().

        // Step 3: If window's relevant agent's event loop's termination nesting level is nonzero,
        // then optionally return true.
        // TODO: This is unsupported currently.

        // Step 4: Optionally, return true. (For example, the user agent might give the
        // user the option to ignore all modal dialogs, and would thus abort at this step
        // whenever the method was invoked.)
        // TODO: The embedder currently cannot block an alert before it is sent to the embedder. This
        // requires changes to the API.

        // Step 5: Return false.
        false
    }

    pub(crate) fn perform_a_microtask_checkpoint(&self, can_gc: CanGc) {
        with_script_thread(|script_thread| script_thread.perform_a_microtask_checkpoint(can_gc));
    }

    pub(crate) fn web_font_context(&self) -> WebFontDocumentContext {
        let global = self.as_global_scope();
        WebFontDocumentContext {
            policy_container: global.policy_container(),
            document_url: global.api_base_url(),
            has_trustworthy_ancestor_origin: global.has_trustworthy_ancestor_origin(),
            insecure_requests_policy: global.insecure_requests_policy(),
            csp_handler: Box::new(FontCspHandler {
                global: Trusted::new(global),
                task_source: global
                    .task_manager()
                    .dom_manipulation_task_source()
                    .to_sendable(),
            }),
        }
    }
}

#[derive(Debug)]
struct FontCspHandler {
    global: Trusted<GlobalScope>,
    task_source: SendableTaskSource,
}

impl CspViolationHandler for FontCspHandler {
    fn process_violations(&self, violations: Vec<Violation>) {
        let global = self.global.clone();
        self.task_source.queue(task!(csp_violation: move || {
            global.root().report_csp_violations(violations, None, None);
        }));
    }

    fn clone(&self) -> Box<dyn CspViolationHandler> {
        Box::new(Self {
            global: self.global.clone(),
            task_source: self.task_source.clone(),
        })
    }
}

// https://html.spec.whatwg.org/multipage/#atob
pub(crate) fn base64_btoa(input: DOMString) -> Fallible<DOMString> {
    // "The btoa() method must throw an InvalidCharacterError exception if
    //  the method's first argument contains any character whose code point
    //  is greater than U+00FF."
    if input.str().chars().any(|c: char| c > '\u{FF}') {
        Err(Error::InvalidCharacter(None))
    } else {
        // "Otherwise, the user agent must convert that argument to a
        //  sequence of octets whose nth octet is the eight-bit
        //  representation of the code point of the nth character of
        //  the argument,"
        let octets = input
            .str()
            .chars()
            .map(|c: char| c as u8)
            .collect::<Vec<u8>>();

        // "and then must apply the base64 algorithm to that sequence of
        //  octets, and return the result. [RFC4648]"
        let config =
            base64::engine::general_purpose::GeneralPurposeConfig::new().with_encode_padding(true);
        let engine = base64::engine::GeneralPurpose::new(&base64::alphabet::STANDARD, config);
        Ok(DOMString::from(engine.encode(octets)))
    }
}

// https://html.spec.whatwg.org/multipage/#atob
pub(crate) fn base64_atob(input: DOMString) -> Fallible<DOMString> {
    // "Remove all space characters from input."
    fn is_html_space(c: char) -> bool {
        HTML_SPACE_CHARACTERS.contains(&c)
    }
    let without_spaces = input
        .str()
        .chars()
        .filter(|&c| !is_html_space(c))
        .collect::<String>();
    let mut input = &*without_spaces;

    // "If the length of input divides by 4 leaving no remainder, then:
    //  if input ends with one or two U+003D EQUALS SIGN (=) characters,
    //  remove them from input."
    if input.len() % 4 == 0 {
        if input.ends_with("==") {
            input = &input[..input.len() - 2]
        } else if input.ends_with('=') {
            input = &input[..input.len() - 1]
        }
    }

    // "If the length of input divides by 4 leaving a remainder of 1,
    //  throw an InvalidCharacterError exception and abort these steps."
    if input.len() % 4 == 1 {
        return Err(Error::InvalidCharacter(None));
    }

    // "If input contains a character that is not in the following list of
    //  characters and character ranges, throw an InvalidCharacterError
    //  exception and abort these steps:
    //
    //  U+002B PLUS SIGN (+)
    //  U+002F SOLIDUS (/)
    //  Alphanumeric ASCII characters"
    if input
        .chars()
        .any(|c| c != '+' && c != '/' && !c.is_alphanumeric())
    {
        return Err(Error::InvalidCharacter(None));
    }

    let config = base64::engine::general_purpose::GeneralPurposeConfig::new()
        .with_decode_padding_mode(base64::engine::DecodePaddingMode::RequireNone)
        .with_decode_allow_trailing_bits(true);
    let engine = base64::engine::GeneralPurpose::new(&base64::alphabet::STANDARD, config);

    let data = engine
        .decode(input)
        .map_err(|_| Error::InvalidCharacter(None))?;
    Ok(data.iter().map(|&b| b as char).collect::<String>().into())
}

impl WindowMethods<crate::DomTypeHolder> for Window {
    /// <https://html.spec.whatwg.org/multipage/#dom-alert>
    fn Alert_(&self) {
        // Step 2: If the method was invoked with no arguments, then let message be the
        // empty string; otherwise, let message be the method's first argument.
        self.Alert(DOMString::new());
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-alert>
    fn Alert(&self, mut message: DOMString) {
        // Step 1: If we cannot show simple dialogs for this, then return.
        if self.cannot_show_simple_dialogs() {
            return;
        }

        // Step 2 is handled in the other variant of this method.
        //
        // Step 3: Set message to the result of normalizing newlines given message.
        message.normalize_newlines();

        // Step 4. Set message to the result of optionally truncating message.
        // This is up to the embedder.

        // Step 5: Let userPromptHandler be WebDriver BiDi user prompt opened with this,
        // "alert", and message.
        // TODO: Add support for WebDriver BiDi.

        // Step 6: If userPromptHandler is "none", then:
        //  1. Show message to the user, treating U+000A LF as a line break.
        //  2. Optionally, pause while waiting for the user to acknowledge the message.
        {
            // Print to the console.
            // Ensure that stderr doesn't trample through the alert() we use to
            // communicate test results (see executorservo.py in wptrunner).
            let stderr = stderr();
            let mut stderr = stderr.lock();
            let stdout = stdout();
            let mut stdout = stdout.lock();
            writeln!(&mut stdout, "\nALERT: {message}").unwrap();
            stdout.flush().unwrap();
            stderr.flush().unwrap();
        }

        let (sender, receiver) =
            ProfiledGenericChannel::channel(self.global().time_profiler_chan().clone()).unwrap();
        let dialog = SimpleDialogRequest::Alert {
            id: self.Document().embedder_controls().next_control_id(),
            message: message.to_string(),
            response_sender: sender,
        };
        self.send_to_embedder(EmbedderMsg::ShowSimpleDialog(self.webview_id(), dialog));
        receiver.recv().unwrap_or_else(|_| {
            // If the receiver is closed, we assume the dialog was cancelled.
            debug!("Alert dialog was cancelled or failed to show.");
            AlertResponse::Ok
        });

        // Step 7: Invoke WebDriver BiDi user prompt closed with this, "alert", and true.
        // TODO: Implement support for WebDriver BiDi.
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-confirm>
    fn Confirm(&self, mut message: DOMString) -> bool {
        // Step 1: If we cannot show simple dialogs for this, then return false.
        if self.cannot_show_simple_dialogs() {
            return false;
        }

        // Step 2: Set message to the result of normalizing newlines given message.
        message.normalize_newlines();

        // Step 3: Set message to the result of optionally truncating message.
        // We let the embedder handle this.

        // Step 4: Show message to the user, treating U+000A LF as a line break, and ask
        // the user to respond with a positive or negative response.
        let (sender, receiver) =
            ProfiledGenericChannel::channel(self.global().time_profiler_chan().clone()).unwrap();
        let dialog = SimpleDialogRequest::Confirm {
            id: self.Document().embedder_controls().next_control_id(),
            message: message.to_string(),
            response_sender: sender,
        };
        self.send_to_embedder(EmbedderMsg::ShowSimpleDialog(self.webview_id(), dialog));

        // Step 5: Let userPromptHandler be WebDriver BiDi user prompt opened with this,
        // "confirm", and message.
        //
        // Step 6: Let accepted be false.
        //
        // Step 7: If userPromptHandler is "none", then:
        //  1. Pause until the user responds either positively or negatively.
        //  2. If the user responded positively, then set accepted to true.
        //
        // Step 8: If userPromptHandler is "accept", then set accepted to true.
        //
        // Step 9: Invoke WebDriver BiDi user prompt closed with this, "confirm", and accepted.
        // TODO: Implement WebDriver BiDi and handle these steps.
        //
        // Step 10: Return accepted.
        match receiver.recv() {
            Ok(ConfirmResponse::Ok) => true,
            Ok(ConfirmResponse::Cancel) => false,
            Err(_) => {
                warn!("Confirm dialog was cancelled or failed to show.");
                false
            },
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-prompt>
    fn Prompt(&self, mut message: DOMString, default: DOMString) -> Option<DOMString> {
        // Step 1: If we cannot show simple dialogs for this, then return null.
        if self.cannot_show_simple_dialogs() {
            return None;
        }

        // Step 2: Set message to the result of normalizing newlines given message.
        message.normalize_newlines();

        // Step 3. Set message to the result of optionally truncating message.
        // Step 4: Set default to the result of optionally truncating default.
        // We let the embedder handle these steps.

        // Step 5: Show message to the user, treating U+000A LF as a line break, and ask
        // the user to either respond with a string value or abort. The response must be
        // defaulted to the value given by default.
        let (sender, receiver) =
            ProfiledGenericChannel::channel(self.global().time_profiler_chan().clone()).unwrap();
        let dialog = SimpleDialogRequest::Prompt {
            id: self.Document().embedder_controls().next_control_id(),
            message: message.to_string(),
            default: default.to_string(),
            response_sender: sender,
        };
        self.send_to_embedder(EmbedderMsg::ShowSimpleDialog(self.webview_id(), dialog));

        // Step 6: Let userPromptHandler be WebDriver BiDi user prompt opened with this,
        // "prompt", and message.
        // TODO: Add support for WebDriver BiDi.
        //
        // Step 7: Let result be null.
        //
        // Step 8: If userPromptHandler is "none", then:
        //  1. Pause while waiting for the user's response.
        //  2. If the user did not abort, then set result to the string that the user responded with.
        //
        // Step 9: Otherwise, if userPromptHandler is "accept", then set result to the empty string.
        // TODO: Implement this.
        //
        // Step 10: Invoke WebDriver BiDi user prompt closed with this, "prompt", false if
        // result is null or true otherwise, and result.
        // TODO: Add support for WebDriver BiDi.
        //
        // Step 11: Return result.
        match receiver.recv() {
            Ok(PromptResponse::Ok(input)) => Some(input.into()),
            Ok(PromptResponse::Cancel) => None,
            Err(_) => {
                warn!("Prompt dialog was cancelled or failed to show.");
                None
            },
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-window-stop>
    fn Stop(&self, can_gc: CanGc) {
        // 1. If this's navigable is null, then return.
        // Note: Servo doesn't have a concept of navigable yet.

        // 2. Stop loading this's navigable.
        self.stop_loading(can_gc);
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-window-focus>
    fn Focus(&self) {
        // > 1. Let `current` be this `Window` object's browsing context.
        // >
        // > 2. If `current` is null, then return.
        let current = match self.undiscarded_window_proxy() {
            Some(proxy) => proxy,
            None => return,
        };

        // > 3. Run the focusing steps with `current`.
        current.focus();

        // > 4. If current is a top-level browsing context, user agents are
        // >    encouraged to trigger some sort of notification to indicate to
        // >    the user that the page is attempting to gain focus.
        //
        // TODO: Step 4
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-window-blur>
    fn Blur(&self) {
        // > User agents are encouraged to ignore calls to this `blur()` method
        // > entirely.
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-open>
    fn Open(
        &self,
        url: USVString,
        target: DOMString,
        features: DOMString,
        can_gc: CanGc,
    ) -> Fallible<Option<DomRoot<WindowProxy>>> {
        self.window_proxy().open(url, target, features, can_gc)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-opener>
    fn GetOpener(
        &self,
        cx: JSContext,
        in_realm_proof: InRealm,
        mut retval: MutableHandleValue,
    ) -> Fallible<()> {
        // Step 1, Let current be this Window object's browsing context.
        let current = match self.window_proxy.get() {
            Some(proxy) => proxy,
            // Step 2, If current is null, then return null.
            None => {
                retval.set(NullValue());
                return Ok(());
            },
        };
        // Still step 2, since the window's BC is the associated doc's BC,
        // see https://html.spec.whatwg.org/multipage/#window-bc
        // and a doc's BC is null if it has been discarded.
        // see https://html.spec.whatwg.org/multipage/#concept-document-bc
        if current.is_browsing_context_discarded() {
            retval.set(NullValue());
            return Ok(());
        }
        // Step 3 to 5.
        current.opener(*cx, in_realm_proof, retval);
        Ok(())
    }

    #[expect(unsafe_code)]
    /// <https://html.spec.whatwg.org/multipage/#dom-opener>
    fn SetOpener(&self, cx: JSContext, value: HandleValue) -> ErrorResult {
        // Step 1.
        if value.is_null() {
            if let Some(proxy) = self.window_proxy.get() {
                proxy.disown();
            }
            return Ok(());
        }
        // Step 2.
        let obj = self.reflector().get_jsobject();
        unsafe {
            let result =
                JS_DefineProperty(*cx, obj, c"opener".as_ptr(), value, JSPROP_ENUMERATE as u32);

            if result { Ok(()) } else { Err(Error::JSFailed) }
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-window-closed>
    fn Closed(&self) -> bool {
        self.window_proxy
            .get()
            .map(|ref proxy| proxy.is_browsing_context_discarded() || proxy.is_closing())
            .unwrap_or(true)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-window-close>
    fn Close(&self) {
        // Step 1, Let current be this Window object's browsing context.
        // Step 2, If current is null or its is closing is true, then return.
        let window_proxy = match self.window_proxy.get() {
            Some(proxy) => proxy,
            None => return,
        };
        if window_proxy.is_closing() {
            return;
        }
        // Note: check the length of the "session history", as opposed to the joint session history?
        // see https://github.com/whatwg/html/issues/3734
        if let Ok(history_length) = self.History().GetLength() {
            let is_auxiliary = window_proxy.is_auxiliary();

            // https://html.spec.whatwg.org/multipage/#script-closable
            let is_script_closable = (self.is_top_level() && history_length == 1) ||
                is_auxiliary ||
                pref!(dom_allow_scripts_to_close_windows);

            // TODO: rest of Step 3:
            // Is the incumbent settings object's responsible browsing context familiar with current?
            // Is the incumbent settings object's responsible browsing context allowed to navigate current?
            if is_script_closable {
                // Step 3.1, set current's is closing to true.
                window_proxy.close();

                // Step 3.2, queue a task on the DOM manipulation task source to close current.
                let this = Trusted::new(self);
                let task = task!(window_close_browsing_context: move || {
                    let window = this.root();
                    let document = window.Document();
                    // https://html.spec.whatwg.org/multipage/#closing-browsing-contexts
                    // Step 1, check if traversable is closing, was already done above.
                    // Steps 2 and 3, prompt to unload for all inclusive descendant navigables.
                    // TODO: We should be prompting for all inclusive descendant navigables,
                    // but we pass false here, which suggests we are not doing that. Why?
                    if document.prompt_to_unload(false, CanGc::note()) {
                        // Step 4, unload.
                        document.unload(false, CanGc::note());

                        // https://html.spec.whatwg.org/multipage/#a-browsing-context-is-discarded
                        // which calls into https://html.spec.whatwg.org/multipage/#discard-a-document.
                        window.discard_browsing_context();

                        window.send_to_constellation(ScriptToConstellationMessage::DiscardTopLevelBrowsingContext);
                    }
                });
                self.as_global_scope()
                    .task_manager()
                    .dom_manipulation_task_source()
                    .queue(task);
            }
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-document-2>
    fn Document(&self) -> DomRoot<Document> {
        self.document
            .get()
            .expect("Document accessed before initialization.")
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-history>
    fn History(&self) -> DomRoot<History> {
        self.history.or_init(|| History::new(self, CanGc::note()))
    }

    /// <https://w3c.github.io/IndexedDB/#factory-interface>
    fn IndexedDB(&self) -> DomRoot<IDBFactory> {
        self.indexeddb.or_init(|| {
            let global_scope = self.upcast::<GlobalScope>();
            IDBFactory::new(global_scope, CanGc::note())
        })
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-window-customelements>
    fn CustomElements(&self) -> DomRoot<CustomElementRegistry> {
        self.custom_element_registry
            .or_init(|| CustomElementRegistry::new(self, CanGc::note()))
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-location>
    fn Location(&self) -> DomRoot<Location> {
        self.location.or_init(|| Location::new(self, CanGc::note()))
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-sessionstorage>
    fn SessionStorage(&self) -> DomRoot<Storage> {
        self.session_storage
            .or_init(|| Storage::new(self, StorageType::Session, CanGc::note()))
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-localstorage>
    fn LocalStorage(&self) -> DomRoot<Storage> {
        self.local_storage
            .or_init(|| Storage::new(self, StorageType::Local, CanGc::note()))
    }

    /// <https://cookiestore.spec.whatwg.org/#Window>
    fn CookieStore(&self, can_gc: CanGc) -> DomRoot<CookieStore> {
        self.global().cookie_store(can_gc)
    }

    /// <https://dvcs.w3.org/hg/webcrypto-api/raw-file/tip/spec/Overview.html#dfn-GlobalCrypto>
    fn Crypto(&self) -> DomRoot<Crypto> {
        self.as_global_scope().crypto(CanGc::note())
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-frameelement>
    fn GetFrameElement(&self) -> Option<DomRoot<Element>> {
        // Steps 1-3.
        let window_proxy = self.window_proxy.get()?;

        // Step 4-5.
        let container = window_proxy.frame_element()?;

        // Step 6.
        let container_doc = container.owner_document();
        let current_doc = GlobalScope::current()
            .expect("No current global object")
            .as_window()
            .Document();
        if !current_doc
            .origin()
            .same_origin_domain(container_doc.origin())
        {
            return None;
        }
        // Step 7.
        Some(DomRoot::from_ref(container))
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-reporterror>
    fn ReportError(&self, cx: JSContext, error: HandleValue, can_gc: CanGc) {
        self.as_global_scope()
            .report_an_exception(cx, error, can_gc);
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-navigator>
    fn Navigator(&self) -> DomRoot<Navigator> {
        self.navigator
            .or_init(|| Navigator::new(self, CanGc::note()))
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-clientinformation>
    fn ClientInformation(&self) -> DomRoot<Navigator> {
        self.Navigator()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-settimeout>
    fn SetTimeout(
        &self,
        _cx: JSContext,
        callback: TrustedScriptOrStringOrFunction,
        timeout: i32,
        args: Vec<HandleValue>,
        can_gc: CanGc,
    ) -> Fallible<i32> {
        let callback = match callback {
            TrustedScriptOrStringOrFunction::String(i) => {
                TimerCallback::StringTimerCallback(TrustedScriptOrString::String(i))
            },
            TrustedScriptOrStringOrFunction::TrustedScript(i) => {
                TimerCallback::StringTimerCallback(TrustedScriptOrString::TrustedScript(i))
            },
            TrustedScriptOrStringOrFunction::Function(i) => TimerCallback::FunctionTimerCallback(i),
        };
        self.as_global_scope().set_timeout_or_interval(
            callback,
            args,
            Duration::from_millis(timeout.max(0) as u64),
            IsInterval::NonInterval,
            can_gc,
        )
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-windowtimers-cleartimeout>
    fn ClearTimeout(&self, handle: i32) {
        self.as_global_scope().clear_timeout_or_interval(handle);
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-windowtimers-setinterval>
    fn SetInterval(
        &self,
        _cx: JSContext,
        callback: TrustedScriptOrStringOrFunction,
        timeout: i32,
        args: Vec<HandleValue>,
        can_gc: CanGc,
    ) -> Fallible<i32> {
        let callback = match callback {
            TrustedScriptOrStringOrFunction::String(i) => {
                TimerCallback::StringTimerCallback(TrustedScriptOrString::String(i))
            },
            TrustedScriptOrStringOrFunction::TrustedScript(i) => {
                TimerCallback::StringTimerCallback(TrustedScriptOrString::TrustedScript(i))
            },
            TrustedScriptOrStringOrFunction::Function(i) => TimerCallback::FunctionTimerCallback(i),
        };
        self.as_global_scope().set_timeout_or_interval(
            callback,
            args,
            Duration::from_millis(timeout.max(0) as u64),
            IsInterval::Interval,
            can_gc,
        )
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-windowtimers-clearinterval>
    fn ClearInterval(&self, handle: i32) {
        self.ClearTimeout(handle);
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-queuemicrotask>
    fn QueueMicrotask(&self, callback: Rc<VoidFunction>) {
        ScriptThread::enqueue_microtask(Microtask::User(UserMicrotask {
            callback,
            pipeline: self.pipeline_id(),
        }));
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-createimagebitmap>
    fn CreateImageBitmap(
        &self,
        image: ImageBitmapSource,
        options: &ImageBitmapOptions,
        can_gc: CanGc,
    ) -> Rc<Promise> {
        ImageBitmap::create_image_bitmap(
            self.as_global_scope(),
            image,
            0,
            0,
            None,
            None,
            options,
            can_gc,
        )
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-createimagebitmap>
    fn CreateImageBitmap_(
        &self,
        image: ImageBitmapSource,
        sx: i32,
        sy: i32,
        sw: i32,
        sh: i32,
        options: &ImageBitmapOptions,
        can_gc: CanGc,
    ) -> Rc<Promise> {
        ImageBitmap::create_image_bitmap(
            self.as_global_scope(),
            image,
            sx,
            sy,
            Some(sw),
            Some(sh),
            options,
            can_gc,
        )
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-window>
    fn Window(&self) -> DomRoot<WindowProxy> {
        self.window_proxy()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-self>
    fn Self_(&self) -> DomRoot<WindowProxy> {
        self.window_proxy()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-frames>
    fn Frames(&self) -> DomRoot<WindowProxy> {
        self.window_proxy()
    }

    /// <https://html.spec.whatwg.org/multipage/#accessing-other-browsing-contexts>
    fn Length(&self) -> u32 {
        self.Document().iframes().iter().count() as u32
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-parent>
    fn GetParent(&self) -> Option<DomRoot<WindowProxy>> {
        // Steps 1-3.
        let window_proxy = self.undiscarded_window_proxy()?;

        // Step 4.
        if let Some(parent) = window_proxy.parent() {
            return Some(DomRoot::from_ref(parent));
        }
        // Step 5.
        Some(window_proxy)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-top>
    fn GetTop(&self) -> Option<DomRoot<WindowProxy>> {
        // Steps 1-3.
        let window_proxy = self.undiscarded_window_proxy()?;

        // Steps 4-5.
        Some(DomRoot::from_ref(window_proxy.top()))
    }

    // https://dvcs.w3.org/hg/webperf/raw-file/tip/specs/
    // NavigationTiming/Overview.html#sec-window.performance-attribute
    fn Performance(&self) -> DomRoot<Performance> {
        self.performance.or_init(|| {
            Performance::new(
                self.as_global_scope(),
                self.navigation_start.get(),
                CanGc::note(),
            )
        })
    }

    // https://html.spec.whatwg.org/multipage/#globaleventhandlers
    global_event_handlers!();

    // https://html.spec.whatwg.org/multipage/#windoweventhandlers
    window_event_handlers!();

    /// <https://developer.mozilla.org/en-US/docs/Web/API/Window/screen>
    fn Screen(&self) -> DomRoot<Screen> {
        self.screen.or_init(|| Screen::new(self, CanGc::note()))
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-windowbase64-btoa>
    fn Btoa(&self, btoa: DOMString) -> Fallible<DOMString> {
        base64_btoa(btoa)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-windowbase64-atob>
    fn Atob(&self, atob: DOMString) -> Fallible<DOMString> {
        base64_atob(atob)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-window-requestanimationframe>
    fn RequestAnimationFrame(&self, callback: Rc<FrameRequestCallback>) -> u32 {
        self.Document()
            .request_animation_frame(AnimationFrameCallback::FrameRequestCallback { callback })
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-window-cancelanimationframe>
    fn CancelAnimationFrame(&self, ident: u32) {
        let doc = self.Document();
        doc.cancel_animation_frame(ident);
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-window-postmessage>
    fn PostMessage(
        &self,
        cx: JSContext,
        message: HandleValue,
        target_origin: USVString,
        transfer: CustomAutoRooterGuard<Vec<*mut JSObject>>,
    ) -> ErrorResult {
        let incumbent = GlobalScope::incumbent().expect("no incumbent global?");
        let source = incumbent.as_window();
        let source_origin = source.Document().origin().immutable().clone();

        self.post_message_impl(&target_origin, source_origin, source, cx, message, transfer)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-messageport-postmessage>
    fn PostMessage_(
        &self,
        cx: JSContext,
        message: HandleValue,
        options: RootedTraceableBox<WindowPostMessageOptions>,
    ) -> ErrorResult {
        let mut rooted = CustomAutoRooter::new(
            options
                .parent
                .transfer
                .iter()
                .map(|js: &RootedTraceableBox<Heap<*mut JSObject>>| js.get())
                .collect(),
        );
        let transfer = CustomAutoRooterGuard::new(*cx, &mut rooted);

        let incumbent = GlobalScope::incumbent().expect("no incumbent global?");
        let source = incumbent.as_window();

        let source_origin = source.Document().origin().immutable().clone();

        self.post_message_impl(
            &options.targetOrigin,
            source_origin,
            source,
            cx,
            message,
            transfer,
        )
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-window-captureevents>
    fn CaptureEvents(&self) {
        // This method intentionally does nothing
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-window-releaseevents>
    fn ReleaseEvents(&self) {
        // This method intentionally does nothing
    }

    // check-tidy: no specs after this line
    fn Debug(&self, message: DOMString) {
        debug!("{}", message);
    }

    #[expect(unsafe_code)]
    fn Gc(&self) {
        unsafe {
            JS_GC(*self.get_cx(), GCReason::API);
        }
    }

    #[expect(unsafe_code)]
    fn Js_backtrace(&self) {
        unsafe {
            println!("Current JS stack:");
            dump_js_stack(*self.get_cx());
            let rust_stack = Backtrace::new();
            println!("Current Rust stack:\n{:?}", rust_stack);
        }
    }

    fn WebdriverCallback(&self, cx: JSContext, value: HandleValue, realm: InRealm, can_gc: CanGc) {
        let webdriver_script_sender = self.webdriver_script_chan.borrow_mut().take();
        if let Some(webdriver_script_sender) = webdriver_script_sender {
            let result = jsval_to_webdriver(cx, &self.globalscope, value, realm, can_gc);
            let _ = webdriver_script_sender.send(result);
        }
    }

    fn WebdriverException(&self, cx: JSContext, value: HandleValue, can_gc: CanGc) {
        let webdriver_script_sender = self.webdriver_script_chan.borrow_mut().take();
        if let Some(webdriver_script_sender) = webdriver_script_sender {
            let _ =
                webdriver_script_sender.send(Err(JavaScriptEvaluationError::EvaluationFailure(
                    Some(javascript_error_info_from_error_info(
                        cx,
                        &ErrorInfo::from_value(value, cx, can_gc),
                        value,
                        can_gc,
                    )),
                )));
        }
    }

    fn WebdriverElement(&self, id: DOMString) -> Option<DomRoot<Element>> {
        find_node_by_unique_id_in_document(&self.Document(), id.into()).and_then(Root::downcast)
    }

    fn WebdriverFrame(&self, browsing_context_id: DOMString) -> Option<DomRoot<WindowProxy>> {
        self.Document()
            .iframes()
            .iter()
            .find(|iframe| {
                iframe
                    .browsing_context_id()
                    .as_ref()
                    .map(BrowsingContextId::to_string) ==
                    Some(browsing_context_id.to_string())
            })
            .and_then(|iframe| iframe.GetContentWindow())
    }

    fn WebdriverWindow(&self, webview_id: DOMString) -> DomRoot<WindowProxy> {
        let window_proxy = &self
            .window_proxy
            .get()
            .expect("Should always have a WindowProxy when calling WebdriverWindow");
        // Window must be top level browsing context.
        assert!(window_proxy.browsing_context_id() == window_proxy.webview_id());
        assert!(self.webview_id().to_string() == webview_id);
        DomRoot::from_ref(window_proxy)
    }

    fn WebdriverShadowRoot(&self, id: DOMString) -> Option<DomRoot<ShadowRoot>> {
        find_node_by_unique_id_in_document(&self.Document(), id.into()).and_then(Root::downcast)
    }

    /// <https://drafts.csswg.org/cssom/#dom-window-getcomputedstyle>
    fn GetComputedStyle(
        &self,
        element: &Element,
        pseudo: Option<DOMString>,
    ) -> DomRoot<CSSStyleDeclaration> {
        // Step 2: Let obj be elt.
        // We don't store CSSStyleOwner directly because it stores a `Dom` which must be
        // rooted. This avoids the rooting the value temporarily.
        let mut is_null = false;

        // Step 3: If pseudoElt is provided, is not the empty string, and starts with a colon, then:
        // Step 3.1: Parse pseudoElt as a <pseudo-element-selector>, and let type be the result.
        let pseudo = pseudo.map(|mut s| {
            s.make_ascii_lowercase();
            s
        });
        let pseudo = match pseudo {
            Some(ref pseudo) if pseudo == ":before" || pseudo == "::before" => {
                Some(PseudoElement::Before)
            },
            Some(ref pseudo) if pseudo == ":after" || pseudo == "::after" => {
                Some(PseudoElement::After)
            },
            Some(ref pseudo) if pseudo == "::selection" => Some(PseudoElement::Selection),
            Some(ref pseudo) if pseudo == "::marker" => Some(PseudoElement::Marker),
            Some(ref pseudo) if pseudo.starts_with(':') => {
                // Step 3.2: If type is failure, or is a ::slotted() or ::part()
                // pseudo-element, let obj be null.
                is_null = true;
                None
            },
            _ => None,
        };

        // Step 4. Let decls be an empty list of CSS declarations.
        // Step 5: If obj is not null, and elt is connected, part of the flat tree, and
        // its shadow-including root has a browsing context which either doesn’t have a
        // browsing context container, or whose browsing context container is being
        // rendered, set decls to a list of all longhand properties that are supported CSS
        // properties, in lexicographical order, with the value being the resolved value
        // computed for obj using the style rules associated with doc.  Additionally,
        // append to decls all the custom properties whose computed value for obj is not
        // the guaranteed-invalid value.
        //
        // Note: The specification says to generate the list of declarations beforehand, yet
        // also says the list should be alive. This is why we do not do step 4 and 5 here.
        // See: https://github.com/w3c/csswg-drafts/issues/6144
        //
        // Step 6:  Return a live CSSStyleProperties object with the following properties:
        CSSStyleDeclaration::new(
            self,
            if is_null {
                CSSStyleOwner::Null
            } else {
                CSSStyleOwner::Element(Dom::from_ref(element))
            },
            pseudo,
            CSSModificationAccess::Readonly,
            CanGc::note(),
        )
    }

    // https://drafts.csswg.org/cssom-view/#dom-window-innerheight
    // TODO Include Scrollbar
    fn InnerHeight(&self) -> i32 {
        self.viewport_details
            .get()
            .size
            .height
            .to_i32()
            .unwrap_or(0)
    }

    // https://drafts.csswg.org/cssom-view/#dom-window-innerwidth
    // TODO Include Scrollbar
    fn InnerWidth(&self) -> i32 {
        self.viewport_details.get().size.width.to_i32().unwrap_or(0)
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-window-scrollx>
    fn ScrollX(&self) -> i32 {
        self.scroll_offset().x as i32
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-window-pagexoffset>
    fn PageXOffset(&self) -> i32 {
        self.ScrollX()
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-window-scrolly>
    fn ScrollY(&self) -> i32 {
        self.scroll_offset().y as i32
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-window-pageyoffset>
    fn PageYOffset(&self) -> i32 {
        self.ScrollY()
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-window-scroll>
    fn Scroll(&self, options: &ScrollToOptions) {
        // Step 1: If invoked with one argument, follow these substeps:
        // Step 1.1: Let options be the argument.
        // Step 1.2: Let x be the value of the left dictionary member of options, if
        // present, or the viewport’s current scroll position on the x axis otherwise.
        let x = options.left.unwrap_or(0.0) as f32;

        // Step 1.3: Let y be the value of the top dictionary member of options, if
        // present, or the viewport’s current scroll position on the y axis otherwise.
        let y = options.top.unwrap_or(0.0) as f32;

        // The rest of the specification continues from `Self::scroll`.
        self.scroll(x, y, options.parent.behavior);
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-window-scroll>
    fn Scroll_(&self, x: f64, y: f64) {
        // Step 2: If invoked with two arguments, follow these substeps:
        // Step 2.1 Let options be null converted to a ScrollToOptions dictionary. [WEBIDL]
        // Step 2.2: Let x and y be the arguments, respectively.
        self.scroll(x as f32, y as f32, ScrollBehavior::Auto);
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-window-scrollto>
    ///
    /// > When the scrollTo() method is invoked, the user agent must act as if the
    /// > scroll() method was invoked with the same arguments.
    fn ScrollTo(&self, options: &ScrollToOptions) {
        self.Scroll(options);
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-window-scrollto>:
    ///
    /// > When the scrollTo() method is invoked, the user agent must act as if the
    /// > scroll() method was invoked with the same arguments.
    fn ScrollTo_(&self, x: f64, y: f64) {
        self.Scroll_(x, y)
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-window-scrollby>
    fn ScrollBy(&self, options: &ScrollToOptions) {
        // When the scrollBy() method is invoked, the user agent must run these steps:
        // Step 1: If invoked with two arguments, follow these substeps:
        //   This doesn't apply here.

        // Step 2: Normalize non-finite values for the left and top dictionary members of options.
        let mut options = options.clone();
        let x = options.left.unwrap_or(0.0);
        let x = if x.is_finite() { x } else { 0.0 };
        let y = options.top.unwrap_or(0.0);
        let y = if y.is_finite() { y } else { 0.0 };

        // Step 3: Add the value of scrollX to the left dictionary member.
        options.left.replace(x + self.ScrollX() as f64);

        // Step 4. Add the value of scrollY to the top dictionary member.
        options.top.replace(y + self.ScrollY() as f64);

        // Step 5: Act as if the scroll() method was invoked with options as the only argument.
        self.Scroll(&options)
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-window-scrollby>
    fn ScrollBy_(&self, x: f64, y: f64) {
        // When the scrollBy() method is invoked, the user agent must run these steps:
        // Step 1: If invoked with two arguments, follow these substeps:
        // Step 1.1: Let options be null converted to a ScrollToOptions dictionary.
        let mut options = ScrollToOptions::empty();

        // Step 1.2: Let x and y be the arguments, respectively.
        // Step 1.3: Let the left dictionary member of options have the value x.
        options.left.replace(x);

        // Step 1.5:  Let the top dictionary member of options have the value y.
        options.top.replace(y);

        // Now follow the specification for the one argument option.
        self.ScrollBy(&options);
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-window-resizeto>
    fn ResizeTo(&self, width: i32, height: i32) {
        // Step 1
        let window_proxy = match self.window_proxy.get() {
            Some(proxy) => proxy,
            None => return,
        };

        // If target is not an auxiliary browsing context that was created by a script
        // (as opposed to by an action of the user), then return.
        if !window_proxy.is_auxiliary() {
            return;
        }

        let dpr = self.device_pixel_ratio();
        let size = Size2D::new(width, height).to_f32() * dpr;
        self.send_to_embedder(EmbedderMsg::ResizeTo(self.webview_id(), size.to_i32()));
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-window-resizeby>
    fn ResizeBy(&self, x: i32, y: i32) {
        let size = self.client_window().size();
        // Step 1
        self.ResizeTo(x + size.width, y + size.height)
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-window-moveto>
    fn MoveTo(&self, x: i32, y: i32) {
        // Step 1
        // TODO determine if this operation is allowed
        let dpr = self.device_pixel_ratio();
        let point = Point2D::new(x, y).to_f32() * dpr;
        let msg = EmbedderMsg::MoveTo(self.webview_id(), point.to_i32());
        self.send_to_embedder(msg);
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-window-moveby>
    fn MoveBy(&self, x: i32, y: i32) {
        let origin = self.client_window().min;
        // Step 1
        self.MoveTo(x + origin.x, y + origin.y)
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-window-screenx>
    fn ScreenX(&self) -> i32 {
        self.client_window().min.x
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-window-screeny>
    fn ScreenY(&self) -> i32 {
        self.client_window().min.y
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-window-outerheight>
    fn OuterHeight(&self) -> i32 {
        self.client_window().height()
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-window-outerwidth>
    fn OuterWidth(&self) -> i32 {
        self.client_window().width()
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-window-devicepixelratio>
    fn DevicePixelRatio(&self) -> Finite<f64> {
        Finite::wrap(self.device_pixel_ratio().get() as f64)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-window-status>
    fn Status(&self) -> DOMString {
        self.status.borrow().clone()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-window-status>
    fn SetStatus(&self, status: DOMString) {
        *self.status.borrow_mut() = status
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-window-matchmedia>
    fn MatchMedia(&self, query: DOMString) -> DomRoot<MediaQueryList> {
        let media_query_list = MediaList::parse_media_list(&query.str(), self);
        let document = self.Document();
        let mql = MediaQueryList::new(&document, media_query_list, CanGc::note());
        self.media_query_lists.track(&*mql);
        mql
    }

    /// <https://fetch.spec.whatwg.org/#dom-global-fetch>
    fn Fetch(
        &self,
        input: RequestOrUSVString,
        init: RootedTraceableBox<RequestInit>,
        comp: InRealm,
        can_gc: CanGc,
    ) -> Rc<Promise> {
        fetch::Fetch(self.upcast(), input, init, comp, can_gc)
    }

    /// <https://fetch.spec.whatwg.org/#dom-window-fetchlater>
    fn FetchLater(
        &self,
        input: RequestInfo,
        init: RootedTraceableBox<DeferredRequestInit>,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<FetchLaterResult>> {
        fetch::FetchLater(self, input, init, can_gc)
    }

    #[cfg(feature = "bluetooth")]
    fn TestRunner(&self) -> DomRoot<TestRunner> {
        self.test_runner
            .or_init(|| TestRunner::new(self.upcast(), CanGc::note()))
    }

    fn RunningAnimationCount(&self) -> u32 {
        self.document
            .get()
            .map_or(0, |d| d.animations().running_animation_count() as u32)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-name>
    fn SetName(&self, name: DOMString) {
        if let Some(proxy) = self.undiscarded_window_proxy() {
            proxy.set_name(name);
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-name>
    fn Name(&self) -> DOMString {
        match self.undiscarded_window_proxy() {
            Some(proxy) => proxy.get_name(),
            None => "".into(),
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-origin>
    fn Origin(&self) -> USVString {
        USVString(self.origin().immutable().ascii_serialization())
    }

    /// <https://w3c.github.io/selection-api/#dom-window-getselection>
    fn GetSelection(&self) -> Option<DomRoot<Selection>> {
        self.document
            .get()
            .and_then(|d| d.GetSelection(CanGc::note()))
    }

    /// <https://dom.spec.whatwg.org/#dom-window-event>
    fn Event(&self, cx: JSContext, rval: MutableHandleValue) {
        if let Some(ref event) = *self.current_event.borrow() {
            event
                .reflector()
                .get_jsobject()
                .safe_to_jsval(cx, rval, CanGc::note());
        }
    }

    fn IsSecureContext(&self) -> bool {
        self.as_global_scope().is_secure_context()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-window-nameditem>
    fn NamedGetter(&self, name: DOMString) -> Option<NamedPropertyValue> {
        if name.is_empty() {
            return None;
        }
        let document = self.Document();

        // https://html.spec.whatwg.org/multipage/#document-tree-child-browsing-context-name-property-set
        let iframes: Vec<_> = document
            .iframes()
            .iter()
            .filter(|iframe| {
                if let Some(window) = iframe.GetContentWindow() {
                    return window.get_name() == name;
                }
                false
            })
            .collect();

        let iframe_iter = iframes.iter().map(|iframe| iframe.upcast::<Element>());

        let name = Atom::from(name);

        // Step 1.
        let elements_with_name = document.get_elements_with_name(&name);
        let name_iter = elements_with_name
            .iter()
            .map(|element| &**element)
            .filter(|elem| is_named_element_with_name_attribute(elem));
        let elements_with_id = document.get_elements_with_id(&name);
        let id_iter = elements_with_id
            .iter()
            .map(|element| &**element)
            .filter(|elem| is_named_element_with_id_attribute(elem));

        // Step 2.
        for elem in iframe_iter.clone() {
            if let Some(nested_window_proxy) = elem
                .downcast::<HTMLIFrameElement>()
                .and_then(|iframe| iframe.GetContentWindow())
            {
                return Some(NamedPropertyValue::WindowProxy(nested_window_proxy));
            }
        }

        let mut elements = iframe_iter.chain(name_iter).chain(id_iter);

        let first = elements.next()?;

        if elements.next().is_none() {
            // Step 3.
            return Some(NamedPropertyValue::Element(DomRoot::from_ref(first)));
        }

        // Step 4.
        #[derive(JSTraceable, MallocSizeOf)]
        struct WindowNamedGetter {
            #[no_trace]
            name: Atom,
        }
        impl CollectionFilter for WindowNamedGetter {
            fn filter(&self, elem: &Element, _root: &Node) -> bool {
                let type_ = match elem.upcast::<Node>().type_id() {
                    NodeTypeId::Element(ElementTypeId::HTMLElement(type_)) => type_,
                    _ => return false,
                };
                if elem.get_id().as_ref() == Some(&self.name) {
                    return true;
                }
                match type_ {
                    HTMLElementTypeId::HTMLEmbedElement |
                    HTMLElementTypeId::HTMLFormElement |
                    HTMLElementTypeId::HTMLImageElement |
                    HTMLElementTypeId::HTMLObjectElement => {
                        elem.get_name().as_ref() == Some(&self.name)
                    },
                    _ => false,
                }
            }
        }
        let collection = HTMLCollection::create(
            self,
            document.upcast(),
            Box::new(WindowNamedGetter { name }),
            CanGc::note(),
        );
        Some(NamedPropertyValue::HTMLCollection(collection))
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-tree-accessors:supported-property-names>
    fn SupportedPropertyNames(&self) -> Vec<DOMString> {
        let mut names_with_first_named_element_map: HashMap<&Atom, &Element> = HashMap::new();

        let document = self.Document();
        let name_map = document.name_map();
        for (name, elements) in &name_map.0 {
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
        let id_map = document.id_map();
        for (id, elements) in &id_map.0 {
            if id.is_empty() {
                continue;
            }
            let mut id_iter = elements
                .iter()
                .filter(|elem| is_named_element_with_id_attribute(elem));
            if let Some(first) = id_iter.next() {
                match names_with_first_named_element_map.entry(id) {
                    Entry::Vacant(entry) => drop(entry.insert(first)),
                    Entry::Occupied(mut entry) => {
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
                cmp::Ordering::Less
            } else {
                cmp::Ordering::Greater
            }
        });

        names_with_first_named_element_vec
            .iter()
            .map(|(k, _v)| DOMString::from(&***k))
            .collect()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-structuredclone>
    fn StructuredClone(
        &self,
        cx: JSContext,
        value: HandleValue,
        options: RootedTraceableBox<StructuredSerializeOptions>,
        can_gc: CanGc,
        retval: MutableHandleValue,
    ) -> Fallible<()> {
        self.as_global_scope()
            .structured_clone(cx, value, options, retval, can_gc)
    }

    fn TrustedTypes(&self, can_gc: CanGc) -> DomRoot<TrustedTypePolicyFactory> {
        self.trusted_types
            .or_init(|| TrustedTypePolicyFactory::new(self.as_global_scope(), can_gc))
    }
}

impl Window {
    pub(crate) fn scroll_offset(&self) -> Vector2D<f32, LayoutPixel> {
        self.scroll_offset_query_with_external_scroll_id(self.pipeline_id().root_scroll_id())
    }

    // https://heycam.github.io/webidl/#named-properties-object
    // https://html.spec.whatwg.org/multipage/#named-access-on-the-window-object
    pub(crate) fn create_named_properties_object(
        cx: JSContext,
        proto: HandleObject,
        object: MutableHandleObject,
    ) {
        window_named_properties::create(cx, proto, object)
    }

    pub(crate) fn current_event(&self) -> Option<DomRoot<Event>> {
        self.current_event
            .borrow()
            .as_ref()
            .map(|e| DomRoot::from_ref(&**e))
    }

    pub(crate) fn set_current_event(&self, event: Option<&Event>) -> Option<DomRoot<Event>> {
        let current = self.current_event();
        *self.current_event.borrow_mut() = event.map(Dom::from_ref);
        current
    }

    /// <https://html.spec.whatwg.org/multipage/#window-post-message-steps>
    fn post_message_impl(
        &self,
        target_origin: &USVString,
        source_origin: ImmutableOrigin,
        source: &Window,
        cx: JSContext,
        message: HandleValue,
        transfer: CustomAutoRooterGuard<Vec<*mut JSObject>>,
    ) -> ErrorResult {
        // Step 1-2, 6-8.
        let data = structuredclone::write(cx, message, Some(transfer))?;

        // Step 3-5.
        let target_origin = match target_origin.0[..].as_ref() {
            "*" => None,
            "/" => Some(source_origin.clone()),
            url => match ServoUrl::parse(url) {
                Ok(url) => Some(url.origin().clone()),
                Err(_) => return Err(Error::Syntax(None)),
            },
        };

        // Step 9.
        self.post_message(target_origin, source_origin, &source.window_proxy(), data);
        Ok(())
    }

    // https://drafts.css-houdini.org/css-paint-api-1/#paint-worklet
    pub(crate) fn paint_worklet(&self) -> DomRoot<Worklet> {
        self.paint_worklet
            .or_init(|| self.new_paint_worklet(CanGc::note()))
    }

    pub(crate) fn has_document(&self) -> bool {
        self.document.get().is_some()
    }

    pub(crate) fn clear_js_runtime(&self) {
        self.as_global_scope()
            .remove_web_messaging_and_dedicated_workers_infra();

        // Clean up any active promises
        // https://github.com/servo/servo/issues/15318
        if let Some(custom_elements) = self.custom_element_registry.get() {
            custom_elements.teardown();
        }

        self.current_state.set(WindowState::Zombie);
        *self.js_runtime.borrow_mut() = None;

        // If this is the currently active pipeline,
        // nullify the window_proxy.
        if let Some(proxy) = self.window_proxy.get() {
            let pipeline_id = self.pipeline_id();
            if let Some(currently_active) = proxy.currently_active() {
                if currently_active == pipeline_id {
                    self.window_proxy.set(None);
                }
            }
        }

        if let Some(performance) = self.performance.get() {
            performance.clear_and_disable_performance_entry_buffer();
        }
        self.as_global_scope()
            .task_manager()
            .cancel_all_tasks_and_ignore_future_tasks();
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-window-scroll>
    pub(crate) fn scroll(&self, x: f32, y: f32, behavior: ScrollBehavior) {
        // Step 3: Normalize non-finite values for x and y.
        let xfinite = if x.is_finite() { x } else { 0.0 };
        let yfinite = if y.is_finite() { y } else { 0.0 };

        // Step 4: If there is no viewport, abort these steps.
        // Currently every frame has a viewport in Servo.

        // Step 5. Let `viewport width` be the width of the viewport excluding the width
        // of the scroll bar, if any.
        // Step 6. `Let viewport height` be the height of the viewport excluding the
        // height of the scroll bar, if any.
        //
        // TODO: Servo does not yet support scrollbars.
        let viewport = self.viewport_details.get().size;

        // Step 7:
        // If the viewport has rightward overflow direction
        //    Let x be max(0, min(x, viewport scrolling area width - viewport width)).
        // If the viewport has leftward overflow direction
        //    Let x be min(0, max(x, viewport width - viewport scrolling area width)).
        // TODO: Implement this.

        // Step 8:
        // If the viewport has downward overflow direction
        //    Let y be max(0, min(y, viewport scrolling area height - viewport height)).
        // If the viewport has upward overflow direction
        //    Let y be min(0, max(y, viewport height - viewport scrolling area height)).
        // TODO: Implement this.

        // Step 9: Let position be the scroll position the viewport would have by aligning
        // the x-coordinate x of the viewport scrolling area with the left of the viewport
        // and aligning the y-coordinate y of the viewport scrolling area with the top of
        // the viewport.
        let scrolling_area = self.scrolling_area_query(None).to_f32();
        let x = xfinite.clamp(0.0, 0.0f32.max(scrolling_area.width() - viewport.width));
        let y = yfinite.clamp(0.0, 0.0f32.max(scrolling_area.height() - viewport.height));

        // Step 10: If position is the same as the viewport’s current scroll position, and
        // the viewport does not have an ongoing smooth scroll, abort these steps.
        let scroll_offset = self.scroll_offset();
        if x == scroll_offset.x && y == scroll_offset.y {
            return;
        }

        // Step 11: Let document be the viewport’s associated Document.
        // Step 12: Perform a scroll of the viewport to position, document’s root element
        // as the associated element, if there is one, or null otherwise, and the scroll
        // behavior being the value of the behavior dictionary member of options.
        self.perform_a_scroll(x, y, self.pipeline_id().root_scroll_id(), behavior, None);
    }

    /// <https://drafts.csswg.org/cssom-view/#perform-a-scroll>
    pub(crate) fn perform_a_scroll(
        &self,
        x: f32,
        y: f32,
        scroll_id: ExternalScrollId,
        _behavior: ScrollBehavior,
        element: Option<&Element>,
    ) {
        // TODO Step 1
        // TODO(mrobinson, #18709): Add smooth scrolling support to WebRender so that we can
        // properly process ScrollBehavior here.
        let reflow_phases_run =
            self.reflow(ReflowGoal::UpdateScrollNode(scroll_id, Vector2D::new(x, y)));
        if reflow_phases_run.needs_frame() {
            self.paint_api()
                .generate_frame(vec![self.webview_id().into()]);
        }

        // > If the scroll position did not change as a result of the user interaction or programmatic
        // > invocation, where no translations were applied as a result, then no scrollend event fires
        // > because no scrolling occurred.
        // Even though the note mention the scrollend, it is relevant to the scroll as well.
        if reflow_phases_run.contains(ReflowPhasesRun::UpdatedScrollNodeOffset) {
            match element {
                Some(el) => self.Document().handle_element_scroll_event(el),
                None => self.Document().handle_viewport_scroll_event(),
            };
        }
    }

    pub(crate) fn device_pixel_ratio(&self) -> Scale<f32, CSSPixel, DevicePixel> {
        self.viewport_details.get().hidpi_scale_factor
    }

    fn client_window(&self) -> DeviceIndependentIntRect {
        let (sender, receiver) = generic_channel::channel().expect("Failed to create IPC channel!");

        self.send_to_embedder(EmbedderMsg::GetWindowRect(self.webview_id(), sender));

        receiver.recv().unwrap_or_default()
    }

    /// Prepares to tick animations and then does a reflow which also advances the
    /// layout animation clock.
    pub(crate) fn advance_animation_clock(&self, delta_ms: i32) {
        self.Document()
            .advance_animation_timeline_for_testing(delta_ms as f64 / 1000.);
        ScriptThread::handle_tick_all_animations_for_testing(self.pipeline_id());
    }

    /// Reflows the page unconditionally if possible and not suppressed. This method will wait for
    /// the layout to complete. If there is no window size yet, the page is presumed invisible and
    /// no reflow is performed. If reflow is suppressed, no reflow will be performed for ForDisplay
    /// goals.
    ///
    /// NOTE: This method should almost never be called directly! Layout and rendering updates should
    /// happen as part of the HTML event loop via *update the rendering*.
    pub(crate) fn reflow(&self, reflow_goal: ReflowGoal) -> ReflowPhasesRun {
        let document = self.Document();

        // Never reflow inactive Documents.
        if !document.is_fully_active() {
            return ReflowPhasesRun::empty();
        }

        self.Document().ensure_safe_to_run_script_or_layout();

        // If layouts are blocked, we block all layouts that are for display only. Other
        // layouts (for queries and scrolling) are not blocked, as they do not display
        // anything and script expects the layout to be up-to-date after they run.
        let pipeline_id = self.pipeline_id();
        if reflow_goal == ReflowGoal::UpdateTheRendering &&
            self.layout_blocker.get().layout_blocked()
        {
            debug!("Suppressing pre-load-event reflow pipeline {pipeline_id}");
            return ReflowPhasesRun::empty();
        }

        debug!("script: performing reflow for goal {reflow_goal:?}");
        let marker = if self.need_emit_timeline_marker(TimelineMarkerType::Reflow) {
            Some(TimelineMarker::start("Reflow".to_owned()))
        } else {
            None
        };

        let restyle_reason = document.restyle_reason();
        document.clear_restyle_reasons();
        let restyle = if restyle_reason.needs_restyle() {
            debug!("Invalidating layout cache due to reflow condition {restyle_reason:?}",);
            // Invalidate any existing cached layout values.
            self.layout_marker.borrow().set(false);
            // Create a new layout caching token.
            *self.layout_marker.borrow_mut() = Rc::new(Cell::new(true));

            let stylesheets_changed = document.flush_stylesheets_for_reflow();
            let pending_restyles = document.drain_pending_restyles();
            let dirty_root = document
                .take_dirty_root()
                .filter(|_| !stylesheets_changed)
                .or_else(|| document.GetDocumentElement())
                .map(|root| root.upcast::<Node>().to_trusted_node_address());

            Some(ReflowRequestRestyle {
                reason: restyle_reason,
                dirty_root,
                stylesheets_changed,
                pending_restyles,
            })
        } else {
            None
        };

        let document_context = self.web_font_context();

        // Send new document and relevant styles to layout.
        let reflow = ReflowRequest {
            document: document.upcast::<Node>().to_trusted_node_address(),
            epoch: document.current_rendering_epoch(),
            restyle,
            viewport_details: self.viewport_details.get(),
            origin: self.origin().immutable().clone(),
            reflow_goal,
            dom_count: document.dom_count(),
            animation_timeline_value: document.current_animation_timeline_value(),
            animations: document.animations().sets.clone(),
            animating_images: document.image_animation_manager().animating_images(),
            highlighted_dom_node: document.highlighted_dom_node().map(|node| node.to_opaque()),
            document_context,
        };

        let Some(reflow_result) = self.layout.borrow_mut().reflow(reflow) else {
            return ReflowPhasesRun::empty();
        };

        debug!("script: layout complete");
        if let Some(marker) = marker {
            self.emit_timeline_marker(marker.end());
        }

        self.handle_pending_images_post_reflow(
            reflow_result.pending_images,
            reflow_result.pending_rasterization_images,
            reflow_result.pending_svg_elements_for_serialization,
        );

        if let Some(iframe_sizes) = reflow_result.iframe_sizes {
            document
                .iframes_mut()
                .handle_new_iframe_sizes_after_layout(self, iframe_sizes);
        }

        document.update_animations_post_reflow();

        reflow_result.reflow_phases_run
    }

    pub(crate) fn request_screenshot_readiness(&self, can_gc: CanGc) {
        self.has_pending_screenshot_readiness_request.set(true);
        self.maybe_resolve_pending_screenshot_readiness_requests(can_gc);
    }

    pub(crate) fn maybe_resolve_pending_screenshot_readiness_requests(&self, can_gc: CanGc) {
        let pending_request = self.has_pending_screenshot_readiness_request.get();
        if !pending_request {
            return;
        }

        let document = self.Document();
        if document.ReadyState() != DocumentReadyState::Complete {
            return;
        }

        if document.render_blocking_element_count() > 0 {
            return;
        }

        // Checks if the html element has reftest-wait attribute present.
        // See http://testthewebforward.org/docs/reftests.html
        // and https://web-platform-tests.org/writing-tests/crashtest.html
        if document.GetDocumentElement().is_some_and(|elem| {
            elem.has_class(&atom!("reftest-wait"), CaseSensitivity::CaseSensitive) ||
                elem.has_class(&Atom::from("test-wait"), CaseSensitivity::CaseSensitive)
        }) {
            return;
        }

        if self.font_context().web_fonts_still_loading() != 0 {
            return;
        }

        if self.Document().Fonts(can_gc).waiting_to_fullfill_promise() {
            return;
        }

        if !self.pending_layout_images.borrow().is_empty() ||
            !self.pending_images_for_rasterization.borrow().is_empty()
        {
            return;
        }

        let document = self.Document();
        if document.needs_rendering_update() {
            return;
        }

        // When all these conditions are met, notify the Constellation that we are ready to
        // have our screenshot taken, when the given layout Epoch has been rendered.
        let epoch = document.current_rendering_epoch();
        let pipeline_id = self.pipeline_id();
        debug!("Ready to take screenshot of {pipeline_id:?} at epoch={epoch:?}");

        self.send_to_constellation(
            ScriptToConstellationMessage::RespondToScreenshotReadinessRequest(
                ScreenshotReadinessResponse::Ready(epoch),
            ),
        );
        self.has_pending_screenshot_readiness_request.set(false);
    }

    /// If parsing has taken a long time and reflows are still waiting for the `load` event,
    /// start allowing them. See <https://github.com/servo/servo/pull/6028>.
    pub(crate) fn reflow_if_reflow_timer_expired(&self) {
        // Only trigger a long parsing time reflow if we are in the first parse of `<body>`
        // and it started more than `INITIAL_REFLOW_DELAY` ago.
        if !matches!(
            self.layout_blocker.get(),
            LayoutBlocker::Parsing(instant) if instant + INITIAL_REFLOW_DELAY < Instant::now()
        ) {
            return;
        }
        self.allow_layout_if_necessary();
    }

    /// Block layout for this `Window` until parsing is done. If parsing takes a long time,
    /// we want to layout anyway, so schedule a moment in the future for when layouts are
    /// allowed even though parsing isn't finished and we havne't sent a load event.
    pub(crate) fn prevent_layout_until_load_event(&self) {
        // If we have already started parsing or have already fired a load event, then
        // don't delay the first layout any longer.
        if !matches!(self.layout_blocker.get(), LayoutBlocker::WaitingForParse) {
            return;
        }

        self.layout_blocker
            .set(LayoutBlocker::Parsing(Instant::now()));
    }

    /// Inform the [`Window`] that layout is allowed either because `load` has happened
    /// or because parsing the `<body>` took so long that we cannot wait any longer.
    pub(crate) fn allow_layout_if_necessary(&self) {
        if matches!(
            self.layout_blocker.get(),
            LayoutBlocker::FiredLoadEventOrParsingTimerExpired
        ) {
            return;
        }

        self.layout_blocker
            .set(LayoutBlocker::FiredLoadEventOrParsingTimerExpired);

        // We do this immediately instead of scheduling a future task, because this can
        // happen if parsing is taking a very long time, which means that the
        // `ScriptThread` is busy doing the parsing and not doing layouts.
        //
        // TOOD(mrobinson): It's expected that this is necessary when in the process of
        // parsing, as we need to interrupt it to update contents, but why is this
        // necessary when parsing finishes? Not doing the synchronous update in that case
        // causes iframe tests to become flaky. It seems there's an issue with the timing of
        // iframe size updates.
        //
        // See <https://github.com/servo/servo/issues/14719>
        if self.Document().update_the_rendering().needs_frame() {
            self.paint_api()
                .generate_frame(vec![self.webview_id().into()]);
        }
    }

    pub(crate) fn layout_blocked(&self) -> bool {
        self.layout_blocker.get().layout_blocked()
    }

    /// Trigger a reflow that is required by a certain queries.
    pub(crate) fn layout_reflow(&self, query_msg: QueryMsg) {
        self.reflow(ReflowGoal::LayoutQuery(query_msg));
    }

    pub(crate) fn resolved_font_style_query(
        &self,
        node: &Node,
        value: String,
    ) -> Option<ServoArc<Font>> {
        self.layout_reflow(QueryMsg::ResolvedFontStyleQuery);

        let document = self.Document();
        let animations = document.animations().sets.clone();
        self.layout.borrow().query_resolved_font_style(
            node.to_trusted_node_address(),
            &value,
            animations,
            document.current_animation_timeline_value(),
        )
    }

    /// Query the used padding values for the given node, but do not force a reflow.
    /// This is used for things like `ResizeObserver` which should observe the value
    /// from the most recent reflow, but do not need it to reflect the current state of
    /// the DOM / style.
    pub(crate) fn padding_query_without_reflow(&self, node: &Node) -> Option<PhysicalSides> {
        let layout = self.layout.borrow();
        layout.query_padding(node.to_trusted_node_address())
    }

    /// Do the same kind of query as `Self::box_area_query`, but do not force a reflow.
    /// This is used for things like `IntersectionObserver` which should observe the value
    /// from the most recent reflow, but do not need it to reflect the current state of
    /// the DOM / style.
    pub(crate) fn box_area_query_without_reflow(
        &self,
        node: &Node,
        area: BoxAreaType,
        exclude_transform_and_inline: bool,
    ) -> Option<UntypedRect<Au>> {
        let layout = self.layout.borrow();
        layout.ensure_stacking_context_tree(self.viewport_details.get());
        layout.query_box_area(
            node.to_trusted_node_address(),
            area,
            exclude_transform_and_inline,
        )
    }

    pub(crate) fn box_area_query(
        &self,
        node: &Node,
        area: BoxAreaType,
        exclude_transform_and_inline: bool,
    ) -> Option<UntypedRect<Au>> {
        self.layout_reflow(QueryMsg::BoxArea);
        self.box_area_query_without_reflow(node, area, exclude_transform_and_inline)
    }

    pub(crate) fn box_areas_query(&self, node: &Node, area: BoxAreaType) -> Vec<UntypedRect<Au>> {
        self.layout_reflow(QueryMsg::BoxAreas);
        self.layout
            .borrow()
            .query_box_areas(node.to_trusted_node_address(), area)
    }

    pub(crate) fn client_rect_query(&self, node: &Node) -> UntypedRect<i32> {
        self.layout_reflow(QueryMsg::ClientRectQuery);
        self.layout
            .borrow()
            .query_client_rect(node.to_trusted_node_address())
    }

    pub(crate) fn current_css_zoom_query(&self, node: &Node) -> f32 {
        self.layout_reflow(QueryMsg::CurrentCSSZoomQuery);
        self.layout
            .borrow()
            .query_current_css_zoom(node.to_trusted_node_address())
    }

    /// Find the scroll area of the given node, if it is not None. If the node
    /// is None, find the scroll area of the viewport.
    pub(crate) fn scrolling_area_query(&self, node: Option<&Node>) -> UntypedRect<i32> {
        self.layout_reflow(QueryMsg::ScrollingAreaOrOffsetQuery);
        self.layout
            .borrow()
            .query_scrolling_area(node.map(Node::to_trusted_node_address))
    }

    pub(crate) fn scroll_offset_query(&self, node: &Node) -> Vector2D<f32, LayoutPixel> {
        let external_scroll_id = ExternalScrollId(
            combine_id_with_fragment_type(node.to_opaque().id(), FragmentType::FragmentBody),
            self.pipeline_id().into(),
        );
        self.scroll_offset_query_with_external_scroll_id(external_scroll_id)
    }

    fn scroll_offset_query_with_external_scroll_id(
        &self,
        external_scroll_id: ExternalScrollId,
    ) -> Vector2D<f32, LayoutPixel> {
        self.layout_reflow(QueryMsg::ScrollingAreaOrOffsetQuery);
        self.scroll_offset_query_with_external_scroll_id_no_reflow(external_scroll_id)
    }

    fn scroll_offset_query_with_external_scroll_id_no_reflow(
        &self,
        external_scroll_id: ExternalScrollId,
    ) -> Vector2D<f32, LayoutPixel> {
        self.layout
            .borrow()
            .scroll_offset(external_scroll_id)
            .unwrap_or_default()
    }

    /// <https://drafts.csswg.org/cssom-view/#scroll-an-element>
    // TODO(stevennovaryo): Need to update the scroll API to follow the spec since it is quite outdated.
    pub(crate) fn scroll_an_element(
        &self,
        element: &Element,
        x: f32,
        y: f32,
        behavior: ScrollBehavior,
    ) {
        let scroll_id = ExternalScrollId(
            combine_id_with_fragment_type(
                element.upcast::<Node>().to_opaque().id(),
                FragmentType::FragmentBody,
            ),
            self.pipeline_id().into(),
        );

        // Step 6.
        // > Perform a scroll of box to position, element as the associated element and behavior as
        // > the scroll behavior.
        self.perform_a_scroll(x, y, scroll_id, behavior, Some(element));
    }

    pub(crate) fn resolved_style_query(
        &self,
        element: TrustedNodeAddress,
        pseudo: Option<PseudoElement>,
        property: PropertyId,
    ) -> DOMString {
        self.layout_reflow(QueryMsg::ResolvedStyleQuery);

        let document = self.Document();
        let animations = document.animations().sets.clone();
        DOMString::from(self.layout.borrow().query_resolved_style(
            element,
            pseudo,
            property,
            animations,
            document.current_animation_timeline_value(),
        ))
    }

    /// If the given |browsing_context_id| refers to an `<iframe>` that is an element
    /// in this [`Window`] and that `<iframe>` has been laid out, return its size.
    /// Otherwise, return `None`.
    pub(crate) fn get_iframe_viewport_details_if_known(
        &self,
        browsing_context_id: BrowsingContextId,
    ) -> Option<ViewportDetails> {
        // Reflow might fail, but do a best effort to return the right size.
        self.layout_reflow(QueryMsg::InnerWindowDimensionsQuery);
        self.Document()
            .iframes()
            .get(browsing_context_id)
            .and_then(|iframe| iframe.size)
    }

    #[expect(unsafe_code)]
    pub(crate) fn offset_parent_query(
        &self,
        node: &Node,
    ) -> (Option<DomRoot<Element>>, UntypedRect<Au>) {
        self.layout_reflow(QueryMsg::OffsetParentQuery);
        let response = self
            .layout
            .borrow()
            .query_offset_parent(node.to_trusted_node_address());
        let element = response.node_address.and_then(|parent_node_address| {
            let node = unsafe { from_untrusted_node_address(parent_node_address) };
            DomRoot::downcast(node)
        });
        (element, response.rect)
    }

    pub(crate) fn scroll_container_query(
        &self,
        node: Option<&Node>,
        flags: ScrollContainerQueryFlags,
    ) -> Option<ScrollContainerResponse> {
        self.layout_reflow(QueryMsg::ScrollParentQuery);
        self.layout
            .borrow()
            .query_scroll_container(node.map(Node::to_trusted_node_address), flags)
    }

    #[expect(unsafe_code)]
    pub(crate) fn scrolling_box_query(
        &self,
        node: Option<&Node>,
        flags: ScrollContainerQueryFlags,
    ) -> Option<ScrollingBox> {
        self.scroll_container_query(node, flags)
            .and_then(|response| {
                Some(match response {
                    ScrollContainerResponse::Viewport(overflow) => {
                        (ScrollingBoxSource::Viewport(self.Document()), overflow)
                    },
                    ScrollContainerResponse::Element(parent_node_address, overflow) => {
                        let node = unsafe { from_untrusted_node_address(parent_node_address) };
                        (
                            ScrollingBoxSource::Element(DomRoot::downcast(node)?),
                            overflow,
                        )
                    },
                })
            })
            .map(|(source, overflow)| ScrollingBox::new(source, overflow))
    }

    pub(crate) fn text_index_query(
        &self,
        node: &Node,
        point_in_node: UntypedPoint2D<f32>,
    ) -> Option<usize> {
        self.layout_reflow(QueryMsg::TextIndexQuery);
        self.layout
            .borrow()
            .query_text_indext(node.to_opaque(), point_in_node)
    }

    pub(crate) fn elements_from_point_query(
        &self,
        point: LayoutPoint,
        flags: ElementsFromPointFlags,
    ) -> Vec<ElementsFromPointResult> {
        self.layout_reflow(QueryMsg::ElementsFromPoint);
        self.layout().query_elements_from_point(point, flags)
    }

    pub(crate) fn hit_test_from_input_event(
        &self,
        input_event: &ConstellationInputEvent,
    ) -> Option<HitTestResult> {
        self.hit_test_from_point_in_viewport(
            input_event.hit_test_result.as_ref()?.point_in_viewport,
        )
    }

    #[expect(unsafe_code)]
    pub(crate) fn hit_test_from_point_in_viewport(
        &self,
        point_in_frame: Point2D<f32, CSSPixel>,
    ) -> Option<HitTestResult> {
        let result = self
            .elements_from_point_query(point_in_frame.cast_unit(), ElementsFromPointFlags::empty())
            .into_iter()
            .nth(0)?;

        let point_relative_to_initial_containing_block =
            point_in_frame + self.scroll_offset().cast_unit();

        // SAFETY: This is safe because `Window::query_elements_from_point` has ensured that
        // layout has run and any OpaqueNodes that no longer refer to real nodes are gone.
        let address = UntrustedNodeAddress(result.node.0 as *const c_void);
        Some(HitTestResult {
            node: unsafe { from_untrusted_node_address(address) },
            cursor: result.cursor,
            point_in_node: result.point_in_target,
            point_in_frame,
            point_relative_to_initial_containing_block,
        })
    }

    pub(crate) fn init_window_proxy(&self, window_proxy: &WindowProxy) {
        assert!(self.window_proxy.get().is_none());
        self.window_proxy.set(Some(window_proxy));
    }

    pub(crate) fn init_document(&self, document: &Document) {
        assert!(self.document.get().is_none());
        assert!(document.window() == self);
        self.document.set(Some(document));

        if self.unminify_css {
            *self.unminified_css_dir.borrow_mut() = Some(unminified_path("unminified-css"));
        }
    }

    /// Commence a new URL load which will either replace this window or scroll to a fragment.
    ///
    /// <https://html.spec.whatwg.org/multipage/#navigating-across-documents>
    pub(crate) fn load_url(
        &self,
        history_handling: NavigationHistoryBehavior,
        force_reload: bool,
        load_data: LoadData,
        can_gc: CanGc,
    ) {
        let doc = self.Document();

        // Step 3. Let initiatorOriginSnapshot be sourceDocument's origin.
        let initiator_origin_snapshot = &load_data.load_origin;

        // TODO: Important re security. See https://github.com/servo/servo/issues/23373
        // Step 5. check that the source browsing-context is "allowed to navigate" this window.
        if !force_reload &&
            load_data.url.as_url()[..Position::AfterQuery] ==
                doc.url().as_url()[..Position::AfterQuery]
        {
            // Step 6
            // TODO: Fragment handling appears to have moved to step 13
            if let Some(fragment) = load_data.url.fragment() {
                let webdriver_sender = self.webdriver_load_status_sender.borrow().clone();
                if let Some(ref sender) = webdriver_sender {
                    let _ = sender.send(WebDriverLoadStatus::NavigationStart);
                }

                self.send_to_constellation(ScriptToConstellationMessage::NavigatedToFragment(
                    load_data.url.clone(),
                    history_handling,
                ));
                doc.check_and_scroll_fragment(fragment);
                let this = Trusted::new(self);
                let old_url = doc.url().into_string();
                let new_url = load_data.url.clone().into_string();
                let task = task!(hashchange_event: move || {
                    let this = this.root();
                    let event = HashChangeEvent::new(
                        &this,
                        atom!("hashchange"),
                        false,
                        false,
                        old_url,
                        new_url,
                        CanGc::note());
                    event.upcast::<Event>().fire(this.upcast::<EventTarget>(), CanGc::note());
                    if let Some(sender) = webdriver_sender {
                        let _ = sender.send(WebDriverLoadStatus::NavigationStop);
                    }
                });
                self.as_global_scope()
                    .task_manager()
                    .dom_manipulation_task_source()
                    .queue(task);
                doc.set_url(load_data.url.clone());
                return;
            }
        }

        // Step 4 and 5
        let pipeline_id = self.pipeline_id();
        let window_proxy = self.window_proxy();
        if let Some(active) = window_proxy.currently_active() {
            if pipeline_id == active && doc.is_prompting_or_unloading() {
                return;
            }
        }

        // Step 8
        if doc.prompt_to_unload(false, can_gc) {
            let window_proxy = self.window_proxy();
            if window_proxy.parent().is_some() {
                // Step 10
                // If browsingContext is a nested browsing context,
                // then put it in the delaying load events mode.
                window_proxy.start_delaying_load_events_mode();
            }

            // Step 11. If historyHandling is "auto", then:
            let resolved_history_handling = if history_handling == NavigationHistoryBehavior::Auto {
                // Step 11.1. If url equals navigable's active document's URL, and
                // initiatorOriginSnapshot is same origin with targetNavigable's active document's
                // origin, then set historyHandling to "replace".
                // Note: `targetNavigable` is not actually defined in the spec, "active document" is
                // assumed to be the correct reference based on WPT results
                if let LoadOrigin::Script(initiator_origin) = initiator_origin_snapshot {
                    if load_data.url == doc.url() && initiator_origin.same_origin(doc.origin()) {
                        NavigationHistoryBehavior::Replace
                    } else {
                        NavigationHistoryBehavior::Push
                    }
                } else {
                    // Step 11.2. Otherwise, set historyHandling to "push".
                    NavigationHistoryBehavior::Push
                }
            // Step 12. If the navigation must be a replace given url and navigable's active
            // document, then set historyHandling to "replace".
            } else if load_data.url.scheme() == "javascript" || doc.is_initial_about_blank() {
                NavigationHistoryBehavior::Replace
            } else {
                history_handling
            };

            if let Some(sender) = self.webdriver_load_status_sender.borrow().as_ref() {
                let _ = sender.send(WebDriverLoadStatus::NavigationStart);
            }

            // Step 13
            ScriptThread::navigate(
                self.webview_id,
                pipeline_id,
                load_data,
                resolved_history_handling,
            );
        };
    }

    /// Handle a potential change to the [`ViewportDetails`] of this [`Window`],
    /// triggering a reflow if any change occurred.
    pub(crate) fn set_viewport_details(&self, viewport_details: ViewportDetails) {
        self.viewport_details.set(viewport_details);
        if !self.layout_mut().set_viewport_details(viewport_details) {
            return;
        }
        self.Document()
            .add_restyle_reason(RestyleReason::ViewportChanged);
    }

    pub(crate) fn viewport_details(&self) -> ViewportDetails {
        self.viewport_details.get()
    }

    /// Get the theme of this [`Window`].
    pub(crate) fn theme(&self) -> Theme {
        self.theme.get()
    }

    /// Handle a theme change request, triggering a reflow is any actual change occurred.
    pub(crate) fn set_theme(&self, new_theme: Theme) {
        self.theme.set(new_theme);
        if !self.layout_mut().set_theme(new_theme) {
            return;
        }
        self.Document()
            .add_restyle_reason(RestyleReason::ThemeChanged);
    }

    pub(crate) fn get_url(&self) -> ServoUrl {
        self.Document().url()
    }

    pub(crate) fn windowproxy_handler(&self) -> &'static WindowProxyHandler {
        self.dom_static.windowproxy_handler
    }

    pub(crate) fn add_resize_event(&self, event: ViewportDetails, event_type: WindowSizeType) {
        // Whenever we receive a new resize event we forget about all the ones that came before
        // it, to avoid unnecessary relayouts
        *self.unhandled_resize_event.borrow_mut() = Some((event, event_type))
    }

    pub(crate) fn take_unhandled_resize_event(&self) -> Option<(ViewportDetails, WindowSizeType)> {
        self.unhandled_resize_event.borrow_mut().take()
    }

    /// Whether or not this [`Window`] has any resize events that have not been processed.
    pub(crate) fn has_unhandled_resize_event(&self) -> bool {
        self.unhandled_resize_event.borrow().is_some()
    }

    pub(crate) fn suspend(&self, can_gc: CanGc) {
        // Suspend timer events.
        self.as_global_scope().suspend();

        // Set the window proxy to be a cross-origin window.
        if self.window_proxy().currently_active() == Some(self.global().pipeline_id()) {
            self.window_proxy().unset_currently_active(can_gc);
        }

        // A hint to the JS runtime that now would be a good time to
        // GC any unreachable objects generated by user script,
        // or unattached DOM nodes. Attached DOM nodes can't be GCd yet,
        // as the document might be reactivated later.
        self.Gc();
    }

    pub(crate) fn resume(&self, can_gc: CanGc) {
        // Resume timer events.
        self.as_global_scope().resume();

        // Set the window proxy to be this object.
        self.window_proxy().set_currently_active(self, can_gc);

        // Push the document title to `Paint` since we are
        // activating this document due to a navigation.
        self.Document().title_changed();
    }

    pub(crate) fn need_emit_timeline_marker(&self, timeline_type: TimelineMarkerType) -> bool {
        let markers = self.devtools_markers.borrow();
        markers.contains(&timeline_type)
    }

    pub(crate) fn emit_timeline_marker(&self, marker: TimelineMarker) {
        let sender = self.devtools_marker_sender.borrow();
        let sender = sender.as_ref().expect("There is no marker sender");
        sender.send(Some(marker)).unwrap();
    }

    pub(crate) fn set_devtools_timeline_markers(
        &self,
        markers: Vec<TimelineMarkerType>,
        reply: GenericSender<Option<TimelineMarker>>,
    ) {
        *self.devtools_marker_sender.borrow_mut() = Some(reply);
        self.devtools_markers.borrow_mut().extend(markers);
    }

    pub(crate) fn drop_devtools_timeline_markers(&self, markers: Vec<TimelineMarkerType>) {
        let mut devtools_markers = self.devtools_markers.borrow_mut();
        for marker in markers {
            devtools_markers.remove(&marker);
        }
        if devtools_markers.is_empty() {
            *self.devtools_marker_sender.borrow_mut() = None;
        }
    }

    pub(crate) fn set_webdriver_script_chan(&self, chan: Option<IpcSender<WebDriverJSResult>>) {
        *self.webdriver_script_chan.borrow_mut() = chan;
    }

    pub(crate) fn set_webdriver_load_status_sender(
        &self,
        sender: Option<GenericSender<WebDriverLoadStatus>>,
    ) {
        *self.webdriver_load_status_sender.borrow_mut() = sender;
    }

    pub(crate) fn is_alive(&self) -> bool {
        self.current_state.get() == WindowState::Alive
    }

    // https://html.spec.whatwg.org/multipage/#top-level-browsing-context
    pub(crate) fn is_top_level(&self) -> bool {
        self.parent_info.is_none()
    }

    /// An implementation of:
    /// <https://drafts.csswg.org/cssom-view/#document-run-the-resize-steps>
    ///
    /// Returns true if there were any pending resize events.
    pub(crate) fn run_the_resize_steps(&self, can_gc: CanGc) -> bool {
        let Some((new_size, size_type)) = self.take_unhandled_resize_event() else {
            return false;
        };

        if self.viewport_details() == new_size {
            return false;
        }

        let _realm = enter_realm(self);
        debug!(
            "Resizing Window for pipeline {:?} from {:?} to {new_size:?}",
            self.pipeline_id(),
            self.viewport_details(),
        );
        self.set_viewport_details(new_size);

        // The document needs to be repainted, because the initial containing
        // block is now a different size. This should be triggered before the
        // event is fired below so that any script queries trigger a restyle.
        self.Document()
            .add_restyle_reason(RestyleReason::ViewportChanged);

        // If viewport units were used, all nodes need to be restyled, because
        // we currently do not track which ones rely on viewport units.
        if self.layout().device().used_viewport_units() {
            self.Document().dirty_all_nodes();
        }

        // http://dev.w3.org/csswg/cssom-view/#resizing-viewports
        if size_type == WindowSizeType::Resize {
            let uievent = UIEvent::new(
                self,
                DOMString::from("resize"),
                EventBubbles::DoesNotBubble,
                EventCancelable::NotCancelable,
                Some(self),
                0i32,
                0u32,
                can_gc,
            );
            uievent.upcast::<Event>().fire(self.upcast(), can_gc);
        }

        true
    }

    /// Evaluate media query lists and report changes
    /// <https://drafts.csswg.org/cssom-view/#evaluate-media-queries-and-report-changes>
    pub(crate) fn evaluate_media_queries_and_report_changes(&self, can_gc: CanGc) {
        let _realm = enter_realm(self);

        rooted_vec!(let mut mql_list);
        self.media_query_lists.for_each(|mql| {
            if let MediaQueryListMatchState::Changed = mql.evaluate_changes() {
                // Recording list of changed Media Queries
                mql_list.push(Dom::from_ref(&*mql));
            }
        });
        // Sending change events for all changed Media Queries
        for mql in mql_list.iter() {
            let event = MediaQueryListEvent::new(
                &mql.global(),
                atom!("change"),
                false,
                false,
                mql.Media(),
                mql.Matches(),
                can_gc,
            );
            event
                .upcast::<Event>()
                .fire(mql.upcast::<EventTarget>(), can_gc);
        }
    }

    /// Set whether to use less resources by running timers at a heavily limited rate.
    pub(crate) fn set_throttled(&self, throttled: bool) {
        self.throttled.set(throttled);
        if throttled {
            self.as_global_scope().slow_down_timers();
        } else {
            self.as_global_scope().speed_up_timers();
        }
    }

    pub(crate) fn throttled(&self) -> bool {
        self.throttled.get()
    }

    pub(crate) fn unminified_css_dir(&self) -> Option<String> {
        self.unminified_css_dir.borrow().clone()
    }

    pub(crate) fn local_script_source(&self) -> &Option<String> {
        &self.local_script_source
    }

    pub(crate) fn set_navigation_start(&self) {
        self.navigation_start.set(CrossProcessInstant::now());
    }

    pub(crate) fn send_to_embedder(&self, msg: EmbedderMsg) {
        self.as_global_scope()
            .script_to_embedder_chan()
            .send(msg)
            .unwrap();
    }

    pub(crate) fn send_to_constellation(&self, msg: ScriptToConstellationMessage) {
        self.as_global_scope()
            .script_to_constellation_chan()
            .send(msg)
            .unwrap();
    }

    #[cfg(feature = "webxr")]
    pub(crate) fn in_immersive_xr_session(&self) -> bool {
        self.navigator
            .get()
            .as_ref()
            .and_then(|nav| nav.xr())
            .is_some_and(|xr| xr.pending_or_active_session())
    }

    #[cfg(not(feature = "webxr"))]
    pub(crate) fn in_immersive_xr_session(&self) -> bool {
        false
    }

    #[expect(unsafe_code)]
    fn handle_pending_images_post_reflow(
        &self,
        pending_images: Vec<PendingImage>,
        pending_rasterization_images: Vec<PendingRasterizationImage>,
        pending_svg_element_for_serialization: Vec<UntrustedNodeAddress>,
    ) {
        let pipeline_id = self.pipeline_id();
        for image in pending_images {
            let id = image.id;
            let node = unsafe { from_untrusted_node_address(image.node) };

            if let PendingImageState::Unrequested(ref url) = image.state {
                fetch_image_for_layout(url.clone(), &node, id, self.image_cache.clone());
            }

            let mut images = self.pending_layout_images.borrow_mut();
            if !images.contains_key(&id) {
                let trusted_node = Trusted::new(&*node);
                let sender = self.register_image_cache_listener(id, move |response| {
                    trusted_node
                        .root()
                        .owner_window()
                        .pending_layout_image_notification(response);
                });

                self.image_cache
                    .add_listener(ImageLoadListener::new(sender, pipeline_id, id));
            }

            let nodes = images.entry(id).or_default();
            if !nodes.iter().any(|n| std::ptr::eq(&*(n.node), &*node)) {
                nodes.push(PendingLayoutImageAncillaryData {
                    node: Dom::from_ref(&*node),
                    destination: image.destination,
                });
            }
        }

        for image in pending_rasterization_images {
            let node = unsafe { from_untrusted_node_address(image.node) };

            let mut images = self.pending_images_for_rasterization.borrow_mut();
            if !images.contains_key(&(image.id, image.size)) {
                let image_cache_sender = self.image_cache_sender.clone();
                self.image_cache.add_rasterization_complete_listener(
                    pipeline_id,
                    image.id,
                    image.size,
                    Box::new(move |response| {
                        let _ = image_cache_sender.send(response);
                    }),
                );
            }

            let nodes = images.entry((image.id, image.size)).or_default();
            if !nodes.iter().any(|n| std::ptr::eq(&**n, &*node)) {
                nodes.push(Dom::from_ref(&*node));
            }
        }

        for node in pending_svg_element_for_serialization.into_iter() {
            let node = unsafe { from_untrusted_node_address(node) };
            let svg = node.downcast::<SVGSVGElement>().unwrap();
            svg.serialize_and_cache_subtree();
            node.dirty(NodeDamage::Other);
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        webview_id: WebViewId,
        runtime: Rc<Runtime>,
        script_chan: Sender<MainThreadScriptMsg>,
        layout: Box<dyn Layout>,
        font_context: Arc<FontContext>,
        image_cache_sender: Sender<ImageCacheResponseMessage>,
        image_cache: Arc<dyn ImageCache>,
        resource_threads: ResourceThreads,
        storage_threads: StorageThreads,
        #[cfg(feature = "bluetooth")] bluetooth_thread: GenericSender<BluetoothRequest>,
        mem_profiler_chan: MemProfilerChan,
        time_profiler_chan: TimeProfilerChan,
        devtools_chan: Option<GenericCallback<ScriptToDevtoolsControlMsg>>,
        constellation_chan: ScriptToConstellationChan,
        embedder_chan: ScriptToEmbedderChan,
        control_chan: GenericSender<ScriptThreadMessage>,
        pipeline_id: PipelineId,
        parent_info: Option<PipelineId>,
        viewport_details: ViewportDetails,
        origin: MutableOrigin,
        creation_url: ServoUrl,
        top_level_creation_url: ServoUrl,
        navigation_start: CrossProcessInstant,
        webgl_chan: Option<WebGLChan>,
        #[cfg(feature = "webxr")] webxr_registry: Option<webxr_api::Registry>,
        paint_api: CrossProcessPaintApi,
        unminify_js: bool,
        unminify_css: bool,
        local_script_source: Option<String>,
        user_content_manager: UserContentManager,
        player_context: WindowGLContext,
        #[cfg(feature = "webgpu")] gpu_id_hub: Arc<IdentityHub>,
        inherited_secure_context: Option<bool>,
        theme: Theme,
    ) -> DomRoot<Self> {
        let error_reporter = CSSErrorReporter {
            pipelineid: pipeline_id,
            script_chan: control_chan,
        };

        let win = Box::new(Self {
            webview_id,
            globalscope: GlobalScope::new_inherited(
                pipeline_id,
                devtools_chan,
                mem_profiler_chan,
                time_profiler_chan,
                constellation_chan,
                embedder_chan,
                resource_threads,
                storage_threads,
                origin,
                creation_url,
                Some(top_level_creation_url),
                #[cfg(feature = "webgpu")]
                gpu_id_hub,
                inherited_secure_context,
                unminify_js,
                Some(font_context.clone()),
            ),
            ongoing_navigation: Default::default(),
            script_chan,
            layout: RefCell::new(layout),
            image_cache_sender,
            image_cache,
            navigator: Default::default(),
            location: Default::default(),
            history: Default::default(),
            indexeddb: Default::default(),
            custom_element_registry: Default::default(),
            window_proxy: Default::default(),
            document: Default::default(),
            performance: Default::default(),
            navigation_start: Cell::new(navigation_start),
            screen: Default::default(),
            session_storage: Default::default(),
            local_storage: Default::default(),
            status: DomRefCell::new(DOMString::new()),
            parent_info,
            dom_static: GlobalStaticData::new(),
            js_runtime: DomRefCell::new(Some(runtime.clone())),
            #[cfg(feature = "bluetooth")]
            bluetooth_thread,
            #[cfg(feature = "bluetooth")]
            bluetooth_extra_permission_data: BluetoothExtraPermissionData::new(),
            unhandled_resize_event: Default::default(),
            viewport_details: Cell::new(viewport_details),
            layout_blocker: Cell::new(LayoutBlocker::WaitingForParse),
            current_state: Cell::new(WindowState::Alive),
            devtools_marker_sender: Default::default(),
            devtools_markers: Default::default(),
            webdriver_script_chan: Default::default(),
            webdriver_load_status_sender: Default::default(),
            error_reporter,
            media_query_lists: DOMTracker::new(),
            #[cfg(feature = "bluetooth")]
            test_runner: Default::default(),
            webgl_chan,
            #[cfg(feature = "webxr")]
            webxr_registry,
            pending_image_callbacks: Default::default(),
            pending_layout_images: Default::default(),
            pending_images_for_rasterization: Default::default(),
            unminified_css_dir: Default::default(),
            local_script_source,
            test_worklet: Default::default(),
            paint_worklet: Default::default(),
            exists_mut_observer: Cell::new(false),
            paint_api,
            has_sent_idle_message: Cell::new(false),
            unminify_css,
            user_content_manager,
            player_context,
            throttled: Cell::new(false),
            layout_marker: DomRefCell::new(Rc::new(Cell::new(true))),
            current_event: DomRefCell::new(None),
            theme: Cell::new(theme),
            trusted_types: Default::default(),
            reporting_observer_list: Default::default(),
            report_list: Default::default(),
            endpoints_list: Default::default(),
            script_window_proxies: ScriptThread::window_proxies(),
            has_pending_screenshot_readiness_request: Default::default(),
        });

        WindowBinding::Wrap::<crate::DomTypeHolder>(GlobalScope::get_cx(), win)
    }

    pub(crate) fn pipeline_id(&self) -> PipelineId {
        self.as_global_scope().pipeline_id()
    }

    /// Create a new cached instance of the given value.
    pub(crate) fn cache_layout_value<T>(&self, value: T) -> LayoutValue<T>
    where
        T: Copy + MallocSizeOf,
    {
        LayoutValue::new(self.layout_marker.borrow().clone(), value)
    }
}

/// An instance of a value associated with a particular snapshot of layout. This stored
/// value can only be read as long as the associated layout marker that is considered
/// valid. It will automatically become unavailable when the next layout operation is
/// performed.
#[derive(MallocSizeOf)]
pub(crate) struct LayoutValue<T: MallocSizeOf> {
    #[conditional_malloc_size_of]
    is_valid: Rc<Cell<bool>>,
    value: T,
}

#[expect(unsafe_code)]
unsafe impl<T: JSTraceable + MallocSizeOf> JSTraceable for LayoutValue<T> {
    unsafe fn trace(&self, trc: *mut js::jsapi::JSTracer) {
        unsafe { self.value.trace(trc) };
    }
}

impl<T: Copy + MallocSizeOf> LayoutValue<T> {
    fn new(marker: Rc<Cell<bool>>, value: T) -> Self {
        LayoutValue {
            is_valid: marker,
            value,
        }
    }

    /// Retrieve the stored value if it is still valid.
    pub(crate) fn get(&self) -> Result<T, ()> {
        if self.is_valid.get() {
            return Ok(self.value);
        }
        Err(())
    }
}

fn should_move_clip_rect(clip_rect: UntypedRect<Au>, new_viewport: UntypedRect<f32>) -> bool {
    let clip_rect = UntypedRect::new(
        Point2D::new(
            clip_rect.origin.x.to_f32_px(),
            clip_rect.origin.y.to_f32_px(),
        ),
        Size2D::new(
            clip_rect.size.width.to_f32_px(),
            clip_rect.size.height.to_f32_px(),
        ),
    );

    // We only need to move the clip rect if the viewport is getting near the edge of
    // our preexisting clip rect. We use half of the size of the viewport as a heuristic
    // for "close."
    static VIEWPORT_SCROLL_MARGIN_SIZE: f32 = 0.5;
    let viewport_scroll_margin = new_viewport.size * VIEWPORT_SCROLL_MARGIN_SIZE;

    (clip_rect.origin.x - new_viewport.origin.x).abs() <= viewport_scroll_margin.width ||
        (clip_rect.max_x() - new_viewport.max_x()).abs() <= viewport_scroll_margin.width ||
        (clip_rect.origin.y - new_viewport.origin.y).abs() <= viewport_scroll_margin.height ||
        (clip_rect.max_y() - new_viewport.max_y()).abs() <= viewport_scroll_margin.height
}

impl Window {
    // https://html.spec.whatwg.org/multipage/#dom-window-postmessage step 7.
    pub(crate) fn post_message(
        &self,
        target_origin: Option<ImmutableOrigin>,
        source_origin: ImmutableOrigin,
        source: &WindowProxy,
        data: StructuredSerializedData,
    ) {
        let this = Trusted::new(self);
        let source = Trusted::new(source);
        let task = task!(post_serialised_message: move || {
            let this = this.root();
            let source = source.root();
            let document = this.Document();

            // Step 7.1.
            if let Some(ref target_origin) = target_origin {
                if !target_origin.same_origin(document.origin()) {
                    return;
                }
            }

            // Steps 7.2.-7.5.
            let cx = this.get_cx();
            let obj = this.reflector().get_jsobject();
            let _ac = JSAutoRealm::new(*cx, obj.get());
            rooted!(in(*cx) let mut message_clone = UndefinedValue());
            if let Ok(ports) = structuredclone::read(this.upcast(), data, message_clone.handle_mut(), CanGc::note()) {
                // Step 7.6, 7.7
                MessageEvent::dispatch_jsval(
                    this.upcast(),
                    this.upcast(),
                    message_clone.handle(),
                    Some(&source_origin.ascii_serialization()),
                    Some(&*source),
                    ports,
                    CanGc::note()
                );
            } else {
                // Step 4, fire messageerror.
                MessageEvent::dispatch_error(
                    this.upcast(),
                    this.upcast(),
                    CanGc::note()
                );
            }
        });
        // TODO(#12718): Use the "posted message task source".
        self.as_global_scope()
            .task_manager()
            .dom_manipulation_task_source()
            .queue(task);
    }
}

#[derive(Clone, MallocSizeOf)]
pub(crate) struct CSSErrorReporter {
    pub(crate) pipelineid: PipelineId,
    pub(crate) script_chan: GenericSender<ScriptThreadMessage>,
}
unsafe_no_jsmanaged_fields!(CSSErrorReporter);

impl ParseErrorReporter for CSSErrorReporter {
    fn report_error(
        &self,
        url: &UrlExtraData,
        location: SourceLocation,
        error: ContextualParseError,
    ) {
        if log_enabled!(log::Level::Info) {
            info!(
                "Url:\t{}\n{}:{} {}",
                url.0.as_str(),
                location.line,
                location.column,
                error
            )
        }

        // TODO: report a real filename
        let _ = self.script_chan.send(ScriptThreadMessage::ReportCSSError(
            self.pipelineid,
            url.0.to_string(),
            location.line,
            location.column,
            error.to_string(),
        ));
    }
}

fn is_named_element_with_name_attribute(elem: &Element) -> bool {
    let type_ = match elem.upcast::<Node>().type_id() {
        NodeTypeId::Element(ElementTypeId::HTMLElement(type_)) => type_,
        _ => return false,
    };
    matches!(
        type_,
        HTMLElementTypeId::HTMLEmbedElement |
            HTMLElementTypeId::HTMLFormElement |
            HTMLElementTypeId::HTMLImageElement |
            HTMLElementTypeId::HTMLObjectElement
    )
}

fn is_named_element_with_id_attribute(elem: &Element) -> bool {
    elem.is_html_element()
}

#[expect(unsafe_code)]
#[unsafe(no_mangle)]
/// Helper for interactive debugging sessions in lldb/gdb.
unsafe extern "C" fn dump_js_stack(cx: *mut RawJSContext) {
    unsafe {
        DumpJSStack(cx, true, false, false);
    }
}

impl WindowHelpers for Window {
    fn create_named_properties_object(
        cx: JSContext,
        proto: HandleObject,
        object: MutableHandleObject,
    ) {
        Self::create_named_properties_object(cx, proto, object)
    }
}
