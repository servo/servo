/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::{Cow, ToOwned};
use std::cell::Cell;
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::default::Default;
use std::io::{stderr, stdout, Write};
use std::ptr::NonNull;
use std::rc::Rc;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::{cmp, env};

use app_units::Au;
use backtrace::Backtrace;
use base64::Engine;
use bluetooth_traits::BluetoothRequest;
use canvas_traits::webgl::WebGLChan;
use crossbeam_channel::{unbounded, Sender, TryRecvError};
use cssparser::{Parser, ParserInput, SourceLocation};
use devtools_traits::{ScriptToDevtoolsControlMsg, TimelineMarker, TimelineMarkerType};
use dom_struct::dom_struct;
use embedder_traits::{EmbedderMsg, PromptDefinition, PromptOrigin, PromptResult};
use euclid::default::{Point2D as UntypedPoint2D, Rect as UntypedRect};
use euclid::{Point2D, Rect, Scale, Size2D, Vector2D};
use gfx_traits::combine_id_with_fragment_type;
use ipc_channel::ipc::{self, IpcSender};
use ipc_channel::router::ROUTER;
use js::conversions::ToJSValConvertible;
use js::jsapi::{GCReason, Heap, JSAutoRealm, JSObject, StackFormat, JSPROP_ENUMERATE, JS_GC};
use js::jsval::{JSVal, NullValue, UndefinedValue};
use js::rust::wrappers::JS_DefineProperty;
use js::rust::{
    CustomAutoRooter, CustomAutoRooterGuard, HandleObject, HandleValue, MutableHandleObject,
};
use malloc_size_of::MallocSizeOf;
use media::WindowGLContext;
use msg::constellation_msg::{BrowsingContextId, PipelineId};
use net_traits::image_cache::{
    ImageCache, ImageResponder, ImageResponse, PendingImageId, PendingImageResponse,
};
use net_traits::storage_thread::StorageType;
use net_traits::ResourceThreads;
use num_traits::ToPrimitive;
use parking_lot::Mutex as ParkMutex;
use profile_traits::ipc as ProfiledIpc;
use profile_traits::mem::ProfilerChan as MemProfilerChan;
use profile_traits::time::ProfilerChan as TimeProfilerChan;
use script_layout_interface::{
    Layout, PendingImageState, QueryMsg, Reflow, ReflowGoal, ScriptReflow, TrustedNodeAddress,
};
use script_traits::webdriver_msg::{WebDriverJSError, WebDriverJSResult};
use script_traits::{
    ConstellationControlMsg, DocumentState, HistoryEntryReplacement, LoadData, ScriptMsg,
    ScriptToConstellationChan, ScrollState, StructuredSerializedData, TimerEventId,
    TimerSchedulerMsg, WebrenderIpcSender, WindowSizeData, WindowSizeType,
};
use selectors::attr::CaseSensitivity;
use servo_arc::Arc as ServoArc;
use servo_atoms::Atom;
use servo_geometry::{f32_rect_to_au_rect, MaxRect};
use servo_url::{ImmutableOrigin, MutableOrigin, ServoUrl};
use style::dom::OpaqueNode;
use style::error_reporting::{ContextualParseError, ParseErrorReporter};
use style::media_queries;
use style::parser::ParserContext as CssParserContext;
use style::properties::style_structs::Font;
use style::properties::PropertyId;
use style::selector_parser::PseudoElement;
use style::str::HTML_SPACE_CHARACTERS;
use style::stylesheets::{CssRuleType, Origin, UrlExtraData};
use style_traits::{CSSPixel, DevicePixel, ParsingMode};
use url::Position;
use webrender_api::units::{DeviceIntPoint, DeviceIntSize, LayoutPixel};
use webrender_api::{DocumentId, ExternalScrollId};

use super::bindings::trace::HashMapTracedValues;
use crate::dom::bindings::cell::{DomRefCell, Ref};
use crate::dom::bindings::codegen::Bindings::DocumentBinding::{
    DocumentMethods, DocumentReadyState,
};
use crate::dom::bindings::codegen::Bindings::HTMLIFrameElementBinding::HTMLIFrameElementMethods;
use crate::dom::bindings::codegen::Bindings::HistoryBinding::History_Binding::HistoryMethods;
use crate::dom::bindings::codegen::Bindings::ImageBitmapBinding::{
    ImageBitmapOptions, ImageBitmapSource,
};
use crate::dom::bindings::codegen::Bindings::MediaQueryListBinding::MediaQueryList_Binding::MediaQueryListMethods;
use crate::dom::bindings::codegen::Bindings::RequestBinding::RequestInit;
use crate::dom::bindings::codegen::Bindings::VoidFunctionBinding::VoidFunction;
use crate::dom::bindings::codegen::Bindings::WindowBinding::{
    self, FrameRequestCallback, ScrollBehavior, ScrollToOptions, WindowMethods,
    WindowPostMessageOptions,
};
use crate::dom::bindings::codegen::UnionTypes::{RequestOrUSVString, StringOrFunction};
use crate::dom::bindings::error::{Error, ErrorResult, Fallible};
use crate::dom::bindings::inheritance::{Castable, ElementTypeId, HTMLElementTypeId, NodeTypeId};
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::DomObject;
use crate::dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::bindings::structuredclone;
use crate::dom::bindings::trace::{JSTraceable, RootedTraceableBox};
use crate::dom::bindings::utils::{GlobalStaticData, WindowProxyHandler};
use crate::dom::bindings::weakref::DOMTracker;
use crate::dom::bluetooth::BluetoothExtraPermissionData;
use crate::dom::crypto::Crypto;
use crate::dom::cssstyledeclaration::{CSSModificationAccess, CSSStyleDeclaration, CSSStyleOwner};
use crate::dom::customelementregistry::CustomElementRegistry;
use crate::dom::document::{AnimationFrameCallback, Document, ReflowTriggerCondition};
use crate::dom::element::Element;
use crate::dom::event::{Event, EventStatus};
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::hashchangeevent::HashChangeEvent;
use crate::dom::history::History;
use crate::dom::htmlcollection::{CollectionFilter, HTMLCollection};
use crate::dom::htmliframeelement::HTMLIFrameElement;
use crate::dom::identityhub::Identities;
use crate::dom::location::Location;
use crate::dom::mediaquerylist::{MediaQueryList, MediaQueryListMatchState};
use crate::dom::mediaquerylistevent::MediaQueryListEvent;
use crate::dom::messageevent::MessageEvent;
use crate::dom::navigator::Navigator;
use crate::dom::node::{document_from_node, from_untrusted_node_address, Node, NodeDamage};
use crate::dom::performance::Performance;
use crate::dom::promise::Promise;
use crate::dom::screen::Screen;
use crate::dom::selection::Selection;
use crate::dom::storage::Storage;
use crate::dom::testrunner::TestRunner;
use crate::dom::webglrenderingcontext::WebGLCommandSender;
use crate::dom::windowproxy::WindowProxy;
use crate::dom::worklet::Worklet;
use crate::dom::workletglobalscope::WorkletGlobalScopeType;
use crate::layout_image::fetch_image_for_layout;
use crate::microtask::MicrotaskQueue;
use crate::realms::InRealm;
use crate::script_runtime::{
    CommonScriptMsg, JSContext, Runtime, ScriptChan, ScriptPort, ScriptThreadEventCategory,
};
use crate::script_thread::{
    ImageCacheMsg, MainThreadScriptChan, MainThreadScriptMsg, ScriptThread,
    SendableMainThreadScriptChan,
};
use crate::task_manager::TaskManager;
use crate::task_source::{TaskSource, TaskSourceName};
use crate::timers::{IsInterval, TimerCallback};
use crate::webdriver_handlers::jsval_to_webdriver;
use crate::{fetch, window_named_properties};

/// Current state of the window object
#[derive(Clone, Copy, Debug, JSTraceable, MallocSizeOf, PartialEq)]
enum WindowState {
    Alive,
    Zombie, // Pipeline is closed, but the window hasn't been GCed yet.
}

/// Extra information concerning the reason for reflowing.
#[derive(Debug, MallocSizeOf)]
pub enum ReflowReason {
    CachedPageNeededReflow,
    DOMContentLoaded,
    DocumentLoaded,
    ElementStateChanged,
    FirstLoad,
    FramedContentChanged,
    IFrameLoadEvent,
    ImageLoaded,
    KeyEvent,
    MissingExplicitReflow,
    MouseEvent,
    PendingReflow,
    Query,
    RefreshTick,
    RequestAnimationFrame,
    ScrollFromScript,
    StylesheetLoaded,
    Timer,
    Viewport,
    WebFontLoaded,
    WindowResize,
    WorkletLoaded,
}

#[dom_struct]
pub struct Window {
    globalscope: GlobalScope,
    #[ignore_malloc_size_of = "trait objects are hard"]
    script_chan: MainThreadScriptChan,
    task_manager: TaskManager,
    navigator: MutNullableDom<Navigator>,
    #[ignore_malloc_size_of = "Arc"]
    #[no_trace]
    image_cache: Arc<dyn ImageCache>,
    #[ignore_malloc_size_of = "channels are hard"]
    #[no_trace]
    image_cache_chan: Sender<ImageCacheMsg>,
    window_proxy: MutNullableDom<WindowProxy>,
    document: MutNullableDom<Document>,
    location: MutNullableDom<Location>,
    history: MutNullableDom<History>,
    custom_element_registry: MutNullableDom<CustomElementRegistry>,
    performance: MutNullableDom<Performance>,
    navigation_start: Cell<u64>,
    navigation_start_precise: Cell<u64>,
    screen: MutNullableDom<Screen>,
    session_storage: MutNullableDom<Storage>,
    local_storage: MutNullableDom<Storage>,
    status: DomRefCell<DOMString>,

    /// For sending timeline markers. Will be ignored if
    /// no devtools server
    #[no_trace]
    devtools_markers: DomRefCell<HashSet<TimelineMarkerType>>,
    #[ignore_malloc_size_of = "channels are hard"]
    #[no_trace]
    devtools_marker_sender: DomRefCell<Option<IpcSender<Option<TimelineMarker>>>>,

    /// Pending resize event, if any.
    #[no_trace]
    resize_event: Cell<Option<(WindowSizeData, WindowSizeType)>>,

    /// Parent id associated with this page, if any.
    #[no_trace]
    parent_info: Option<PipelineId>,

    /// Global static data related to the DOM.
    dom_static: GlobalStaticData,

    /// The JavaScript runtime.
    #[ignore_malloc_size_of = "Rc<T> is hard"]
    js_runtime: DomRefCell<Option<Rc<Runtime>>>,

    /// The current size of the window, in pixels.
    #[no_trace]
    window_size: Cell<WindowSizeData>,

    /// A handle for communicating messages to the bluetooth thread.
    #[ignore_malloc_size_of = "channels are hard"]
    #[no_trace]
    bluetooth_thread: IpcSender<BluetoothRequest>,

    bluetooth_extra_permission_data: BluetoothExtraPermissionData,

    /// An enlarged rectangle around the page contents visible in the viewport, used
    /// to prevent creating display list items for content that is far away from the viewport.
    #[no_trace]
    page_clip_rect: Cell<UntypedRect<Au>>,

    /// Flag to suppress reflows. The first reflow will come either with
    /// RefreshTick or with FirstLoad. Until those first reflows, we want to
    /// suppress others like MissingExplicitReflow.
    suppress_reflow: Cell<bool>,

    /// A counter of the number of pending reflows for this window.
    pending_reflow_count: Cell<u32>,

    /// A channel for communicating results of async scripts back to the webdriver server
    #[ignore_malloc_size_of = "channels are hard"]
    #[no_trace]
    webdriver_script_chan: DomRefCell<Option<IpcSender<WebDriverJSResult>>>,

    /// The current state of the window object
    current_state: Cell<WindowState>,

    #[no_trace]
    current_viewport: Cell<UntypedRect<Au>>,

    error_reporter: CSSErrorReporter,

    /// A list of scroll offsets for each scrollable element.
    #[no_trace]
    scroll_offsets: DomRefCell<HashMap<OpaqueNode, Vector2D<f32, LayoutPixel>>>,

    /// All the MediaQueryLists we need to update
    media_query_lists: DOMTracker<MediaQueryList>,

    test_runner: MutNullableDom<TestRunner>,

    /// A handle for communicating messages to the WebGL thread, if available.
    #[ignore_malloc_size_of = "channels are hard"]
    #[no_trace]
    webgl_chan: Option<WebGLChan>,

    #[ignore_malloc_size_of = "defined in webxr"]
    #[no_trace]
    webxr_registry: webxr_api::Registry,

    /// All of the elements that have an outstanding image request that was
    /// initiated by layout during a reflow. They are stored in the script thread
    /// to ensure that the element can be marked dirty when the image data becomes
    /// available at some point in the future.
    pending_layout_images: DomRefCell<HashMapTracedValues<PendingImageId, Vec<Dom<Node>>>>,

    /// Directory to store unminified scripts for this window if unminify-js
    /// opt is enabled.
    unminified_js_dir: DomRefCell<Option<String>>,

    /// Directory with stored unminified scripts
    local_script_source: Option<String>,

    /// Worklets
    test_worklet: MutNullableDom<Worklet>,
    /// <https://drafts.css-houdini.org/css-paint-api-1/#paint-worklet>
    paint_worklet: MutNullableDom<Worklet>,
    /// The Webrender Document id associated with this window.
    #[ignore_malloc_size_of = "defined in webrender_api"]
    #[no_trace]
    webrender_document: DocumentId,

    /// Flag to identify whether mutation observers are present(true)/absent(false)
    exists_mut_observer: Cell<bool>,

    /// Webrender API Sender
    #[ignore_malloc_size_of = "Wraps an IpcSender"]
    #[no_trace]
    webrender_api_sender: WebrenderIpcSender,

    /// Indicate whether a SetDocumentStatus message has been sent after a reflow is complete.
    /// It is used to avoid sending idle message more than once, which is unneccessary.
    has_sent_idle_message: Cell<bool>,

    /// Emits notifications when there is a relayout.
    relayout_event: bool,

    /// True if it is safe to write to the image.
    prepare_for_screenshot: bool,

    /// Unminify Javascript.
    unminify_js: bool,

    /// Where to load userscripts from, if any. An empty string will load from
    /// the resources/user-agent-js directory, and if the option isn't passed userscripts
    /// won't be loaded.
    userscripts_path: Option<String>,

    /// Replace unpaired surrogates in DOM strings with U+FFFD.
    /// See <https://github.com/servo/servo/issues/6564>
    replace_surrogates: bool,

    /// Window's GL context from application
    #[ignore_malloc_size_of = "defined in script_thread"]
    #[no_trace]
    player_context: WindowGLContext,

    throttled: Cell<bool>,

    /// A shared marker for the validity of any cached layout values. A value of true
    /// indicates that any such values remain valid; any new layout that invalidates
    /// those values will cause the marker to be set to false.
    #[ignore_malloc_size_of = "Rc is hard"]
    layout_marker: DomRefCell<Rc<Cell<bool>>>,

    /// <https://dom.spec.whatwg.org/#window-current-event>
    current_event: DomRefCell<Option<Dom<Event>>>,
}

impl Window {
    pub fn task_manager(&self) -> &TaskManager {
        &self.task_manager
    }

    pub fn get_exists_mut_observer(&self) -> bool {
        self.exists_mut_observer.get()
    }

    pub fn set_exists_mut_observer(&self) {
        self.exists_mut_observer.set(true);
    }

    #[allow(unsafe_code)]
    pub fn clear_js_runtime_for_script_deallocation(&self) {
        self.upcast::<GlobalScope>()
            .remove_web_messaging_and_dedicated_workers_infra();
        unsafe {
            *self.js_runtime.borrow_for_script_deallocation() = None;
            self.window_proxy.set(None);
            self.current_state.set(WindowState::Zombie);
            self.ignore_all_tasks();
        }
    }

    /// A convenience method for
    /// <https://html.spec.whatwg.org/multipage/#a-browsing-context-is-discarded>
    pub fn discard_browsing_context(&self) {
        let proxy = match self.window_proxy.get() {
            Some(proxy) => proxy,
            None => panic!("Discarding a BC from a window that has none"),
        };
        proxy.discard_browsing_context();
        // Step 4 of https://html.spec.whatwg.org/multipage/#discard-a-document
        // Other steps performed when the `PipelineExit` message
        // is handled by the ScriptThread.
        self.ignore_all_tasks();
    }

    /// Cancel all current, and ignore all subsequently queued, tasks.
    pub fn ignore_all_tasks(&self) {
        let mut ignore_flags = self.task_manager.task_cancellers.borrow_mut();
        for task_source_name in TaskSourceName::all() {
            let flag = ignore_flags.entry(task_source_name).or_default();
            flag.store(true, Ordering::SeqCst);
        }
    }

    /// Get a sender to the time profiler thread.
    pub fn time_profiler_chan(&self) -> &TimeProfilerChan {
        self.globalscope.time_profiler_chan()
    }

    pub fn origin(&self) -> &MutableOrigin {
        self.globalscope.origin()
    }

    #[allow(unsafe_code)]
    pub fn get_cx(&self) -> JSContext {
        unsafe { JSContext::from_ptr(self.js_runtime.borrow().as_ref().unwrap().cx()) }
    }

    pub fn get_js_runtime(&self) -> Ref<Option<Rc<Runtime>>> {
        self.js_runtime.borrow()
    }

    pub fn main_thread_script_chan(&self) -> &Sender<MainThreadScriptMsg> {
        &self.script_chan.0
    }

    pub fn parent_info(&self) -> Option<PipelineId> {
        self.parent_info
    }

    pub fn new_script_pair(&self) -> (Box<dyn ScriptChan + Send>, Box<dyn ScriptPort + Send>) {
        let (tx, rx) = unbounded();
        (Box::new(SendableMainThreadScriptChan(tx)), Box::new(rx))
    }

    pub fn image_cache(&self) -> Arc<dyn ImageCache> {
        self.image_cache.clone()
    }

    /// This can panic if it is called after the browsing context has been discarded
    pub fn window_proxy(&self) -> DomRoot<WindowProxy> {
        self.window_proxy.get().unwrap()
    }

    /// Returns the window proxy if it has not been discarded.
    /// <https://html.spec.whatwg.org/multipage/#a-browsing-context-is-discarded>
    pub fn undiscarded_window_proxy(&self) -> Option<DomRoot<WindowProxy>> {
        self.window_proxy.get().and_then(|window_proxy| {
            if window_proxy.is_browsing_context_discarded() {
                None
            } else {
                Some(window_proxy)
            }
        })
    }

    pub fn bluetooth_thread(&self) -> IpcSender<BluetoothRequest> {
        self.bluetooth_thread.clone()
    }

    pub fn bluetooth_extra_permission_data(&self) -> &BluetoothExtraPermissionData {
        &self.bluetooth_extra_permission_data
    }

    pub fn css_error_reporter(&self) -> Option<&dyn ParseErrorReporter> {
        Some(&self.error_reporter)
    }

    /// Sets a new list of scroll offsets.
    ///
    /// This is called when layout gives us new ones and WebRender is in use.
    pub fn set_scroll_offsets(&self, offsets: HashMap<OpaqueNode, Vector2D<f32, LayoutPixel>>) {
        *self.scroll_offsets.borrow_mut() = offsets
    }

    pub fn current_viewport(&self) -> UntypedRect<Au> {
        self.current_viewport.clone().get()
    }

    pub(crate) fn webgl_chan(&self) -> Option<WebGLCommandSender> {
        self.webgl_chan
            .as_ref()
            .map(|chan| WebGLCommandSender::new(chan.clone()))
    }

    pub fn webxr_registry(&self) -> webxr_api::Registry {
        self.webxr_registry.clone()
    }

    fn new_paint_worklet(&self) -> DomRoot<Worklet> {
        debug!("Creating new paint worklet.");
        Worklet::new(self, WorkletGlobalScopeType::Paint)
    }

    pub fn pending_image_notification(&self, response: PendingImageResponse) {
        //XXXjdm could be more efficient to send the responses to layout,
        //       rather than making layout talk to the image cache to
        //       obtain the same data.
        let mut images = self.pending_layout_images.borrow_mut();
        let nodes = images.entry(response.id);
        let nodes = match nodes {
            Entry::Occupied(nodes) => nodes,
            Entry::Vacant(_) => return,
        };
        for node in nodes.get() {
            node.dirty(NodeDamage::OtherNodeDamage);
        }
        match response.response {
            ImageResponse::MetadataLoaded(_) => {},
            ImageResponse::Loaded(_, _) |
            ImageResponse::PlaceholderLoaded(_, _) |
            ImageResponse::None => {
                nodes.remove();
            },
        }
        self.add_pending_reflow();
    }

    pub fn get_webrender_api_sender(&self) -> WebrenderIpcSender {
        self.webrender_api_sender.clone()
    }

    pub fn get_userscripts_path(&self) -> Option<String> {
        self.userscripts_path.clone()
    }

    pub fn replace_surrogates(&self) -> bool {
        self.replace_surrogates
    }

    pub fn unminify_js(&self) -> bool {
        self.unminify_js
    }

    pub fn get_player_context(&self) -> WindowGLContext {
        self.player_context.clone()
    }

    // see note at https://dom.spec.whatwg.org/#concept-event-dispatch step 2
    pub fn dispatch_event_with_target_override(&self, event: &Event) -> EventStatus {
        if self.has_document() {
            assert!(self.Document().can_invoke_script());
        }
        event.dispatch(self.upcast(), true)
    }
}

// https://html.spec.whatwg.org/multipage/#atob
pub fn base64_btoa(input: DOMString) -> Fallible<DOMString> {
    // "The btoa() method must throw an InvalidCharacterError exception if
    //  the method's first argument contains any character whose code point
    //  is greater than U+00FF."
    if input.chars().any(|c: char| c > '\u{FF}') {
        Err(Error::InvalidCharacter)
    } else {
        // "Otherwise, the user agent must convert that argument to a
        //  sequence of octets whose nth octet is the eight-bit
        //  representation of the code point of the nth character of
        //  the argument,"
        let octets = input.chars().map(|c: char| c as u8).collect::<Vec<u8>>();

        // "and then must apply the base64 algorithm to that sequence of
        //  octets, and return the result. [RFC4648]"
        let config =
            base64::engine::general_purpose::GeneralPurposeConfig::new().with_encode_padding(true);
        let engine = base64::engine::GeneralPurpose::new(&base64::alphabet::STANDARD, config);
        Ok(DOMString::from(engine.encode(octets)))
    }
}

// https://html.spec.whatwg.org/multipage/#atob
pub fn base64_atob(input: DOMString) -> Fallible<DOMString> {
    // "Remove all space characters from input."
    fn is_html_space(c: char) -> bool {
        HTML_SPACE_CHARACTERS.iter().any(|&m| m == c)
    }
    let without_spaces = input
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
        return Err(Error::InvalidCharacter);
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
        return Err(Error::InvalidCharacter);
    }

    let config = base64::engine::general_purpose::GeneralPurposeConfig::new()
        .with_decode_padding_mode(base64::engine::DecodePaddingMode::RequireNone)
        .with_decode_allow_trailing_bits(true);
    let engine = base64::engine::GeneralPurpose::new(&base64::alphabet::STANDARD, config);

    let data = engine.decode(input).map_err(|_| Error::InvalidCharacter)?;
    Ok(data.iter().map(|&b| b as char).collect::<String>().into())
}

impl WindowMethods for Window {
    // https://html.spec.whatwg.org/multipage/#dom-alert
    fn Alert_(&self) {
        self.Alert(DOMString::new());
    }

    // https://html.spec.whatwg.org/multipage/#dom-alert
    fn Alert(&self, s: DOMString) {
        // Print to the console.
        // Ensure that stderr doesn't trample through the alert() we use to
        // communicate test results (see executorservo.py in wptrunner).
        {
            let stderr = stderr();
            let mut stderr = stderr.lock();
            let stdout = stdout();
            let mut stdout = stdout.lock();
            writeln!(&mut stdout, "\nALERT: {}", s).unwrap();
            stdout.flush().unwrap();
            stderr.flush().unwrap();
        }
        let (sender, receiver) =
            ProfiledIpc::channel(self.global().time_profiler_chan().clone()).unwrap();
        let prompt = PromptDefinition::Alert(s.to_string(), sender);
        let msg = EmbedderMsg::Prompt(prompt, PromptOrigin::Untrusted);
        self.send_to_embedder(msg);
        receiver.recv().unwrap();
    }

    // https://html.spec.whatwg.org/multipage/#dom-confirm
    fn Confirm(&self, s: DOMString) -> bool {
        let (sender, receiver) =
            ProfiledIpc::channel(self.global().time_profiler_chan().clone()).unwrap();
        let prompt = PromptDefinition::OkCancel(s.to_string(), sender);
        let msg = EmbedderMsg::Prompt(prompt, PromptOrigin::Untrusted);
        self.send_to_embedder(msg);
        receiver.recv().unwrap() == PromptResult::Primary
    }

    // https://html.spec.whatwg.org/multipage/#dom-prompt
    fn Prompt(&self, message: DOMString, default: DOMString) -> Option<DOMString> {
        let (sender, receiver) =
            ProfiledIpc::channel(self.global().time_profiler_chan().clone()).unwrap();
        let prompt = PromptDefinition::Input(message.to_string(), default.to_string(), sender);
        let msg = EmbedderMsg::Prompt(prompt, PromptOrigin::Untrusted);
        self.send_to_embedder(msg);
        receiver.recv().unwrap().map(|s| s.into())
    }

    // https://html.spec.whatwg.org/multipage/#dom-window-stop
    fn Stop(&self) {
        // TODO: Cancel ongoing navigation.
        let doc = self.Document();
        doc.abort();
    }

    // https://html.spec.whatwg.org/multipage/#dom-open
    fn Open(
        &self,
        url: USVString,
        target: DOMString,
        features: DOMString,
    ) -> Fallible<Option<DomRoot<WindowProxy>>> {
        self.window_proxy().open(url, target, features)
    }

    // https://html.spec.whatwg.org/multipage/#dom-opener
    fn Opener(&self, cx: JSContext, in_realm_proof: InRealm) -> JSVal {
        // Step 1, Let current be this Window object's browsing context.
        let current = match self.window_proxy.get() {
            Some(proxy) => proxy,
            // Step 2, If current is null, then return null.
            None => return NullValue(),
        };
        // Still step 2, since the window's BC is the associated doc's BC,
        // see https://html.spec.whatwg.org/multipage/#window-bc
        // and a doc's BC is null if it has been discarded.
        // see https://html.spec.whatwg.org/multipage/#concept-document-bc
        if current.is_browsing_context_discarded() {
            return NullValue();
        }
        // Step 3 to 5.
        current.opener(*cx, in_realm_proof)
    }

    #[allow(unsafe_code)]
    // https://html.spec.whatwg.org/multipage/#dom-opener
    fn SetOpener(&self, cx: JSContext, value: HandleValue) {
        // Step 1.
        if value.is_null() {
            return self.window_proxy().disown();
        }
        // Step 2.
        let obj = self.reflector().get_jsobject();
        unsafe {
            assert!(JS_DefineProperty(
                *cx,
                obj,
                "opener\0".as_ptr() as *const libc::c_char,
                value,
                JSPROP_ENUMERATE as u32
            ));
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-window-closed
    fn Closed(&self) -> bool {
        self.window_proxy
            .get()
            .map(|ref proxy| proxy.is_browsing_context_discarded() || proxy.is_closing())
            .unwrap_or(true)
    }

    // https://html.spec.whatwg.org/multipage/#dom-window-close
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
            let is_script_closable = (self.is_top_level() && history_length == 1) || is_auxiliary;

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
                    if document.prompt_to_unload(false) {
                        // Step 4, unload.
                        document.unload(false);

                        // https://html.spec.whatwg.org/multipage/#a-browsing-context-is-discarded
                        // which calls into https://html.spec.whatwg.org/multipage/#discard-a-document.
                        window.discard_browsing_context();

                        window.send_to_constellation(ScriptMsg::DiscardTopLevelBrowsingContext);
                    }
                });
                self.task_manager()
                    .dom_manipulation_task_source()
                    .queue(task, self.upcast::<GlobalScope>())
                    .expect("Queuing window_close_browsing_context task to work");
            }
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-2
    fn Document(&self) -> DomRoot<Document> {
        self.document
            .get()
            .expect("Document accessed before initialization.")
    }

    // https://html.spec.whatwg.org/multipage/#dom-history
    fn History(&self) -> DomRoot<History> {
        self.history.or_init(|| History::new(self))
    }

    // https://html.spec.whatwg.org/multipage/#dom-window-customelements
    fn CustomElements(&self) -> DomRoot<CustomElementRegistry> {
        self.custom_element_registry
            .or_init(|| CustomElementRegistry::new(self))
    }

    // https://html.spec.whatwg.org/multipage/#dom-location
    fn Location(&self) -> DomRoot<Location> {
        self.location.or_init(|| Location::new(self))
    }

    // https://html.spec.whatwg.org/multipage/#dom-sessionstorage
    fn SessionStorage(&self) -> DomRoot<Storage> {
        self.session_storage
            .or_init(|| Storage::new(self, StorageType::Session))
    }

    // https://html.spec.whatwg.org/multipage/#dom-localstorage
    fn LocalStorage(&self) -> DomRoot<Storage> {
        self.local_storage
            .or_init(|| Storage::new(self, StorageType::Local))
    }

    // https://dvcs.w3.org/hg/webcrypto-api/raw-file/tip/spec/Overview.html#dfn-GlobalCrypto
    fn Crypto(&self) -> DomRoot<Crypto> {
        self.upcast::<GlobalScope>().crypto()
    }

    // https://html.spec.whatwg.org/multipage/#dom-frameelement
    fn GetFrameElement(&self) -> Option<DomRoot<Element>> {
        // Steps 1-3.
        let window_proxy = self.window_proxy.get()?;

        // Step 4-5.
        let container = window_proxy.frame_element()?;

        // Step 6.
        let container_doc = document_from_node(container);
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

    // https://html.spec.whatwg.org/multipage/#dom-navigator
    fn Navigator(&self) -> DomRoot<Navigator> {
        self.navigator.or_init(|| Navigator::new(self))
    }

    // https://html.spec.whatwg.org/multipage/#dom-windowtimers-settimeout
    fn SetTimeout(
        &self,
        _cx: JSContext,
        callback: StringOrFunction,
        timeout: i32,
        args: Vec<HandleValue>,
    ) -> i32 {
        let callback = match callback {
            StringOrFunction::String(i) => TimerCallback::StringTimerCallback(i),
            StringOrFunction::Function(i) => TimerCallback::FunctionTimerCallback(i),
        };
        self.upcast::<GlobalScope>().set_timeout_or_interval(
            callback,
            args,
            timeout,
            IsInterval::NonInterval,
        )
    }

    // https://html.spec.whatwg.org/multipage/#dom-windowtimers-cleartimeout
    fn ClearTimeout(&self, handle: i32) {
        self.upcast::<GlobalScope>()
            .clear_timeout_or_interval(handle);
    }

    // https://html.spec.whatwg.org/multipage/#dom-windowtimers-setinterval
    fn SetInterval(
        &self,
        _cx: JSContext,
        callback: StringOrFunction,
        timeout: i32,
        args: Vec<HandleValue>,
    ) -> i32 {
        let callback = match callback {
            StringOrFunction::String(i) => TimerCallback::StringTimerCallback(i),
            StringOrFunction::Function(i) => TimerCallback::FunctionTimerCallback(i),
        };
        self.upcast::<GlobalScope>().set_timeout_or_interval(
            callback,
            args,
            timeout,
            IsInterval::Interval,
        )
    }

    // https://html.spec.whatwg.org/multipage/#dom-windowtimers-clearinterval
    fn ClearInterval(&self, handle: i32) {
        self.ClearTimeout(handle);
    }

    // https://html.spec.whatwg.org/multipage/#dom-queuemicrotask
    fn QueueMicrotask(&self, callback: Rc<VoidFunction>) {
        self.upcast::<GlobalScope>()
            .queue_function_as_microtask(callback);
    }

    // https://html.spec.whatwg.org/multipage/#dom-createimagebitmap
    fn CreateImageBitmap(
        &self,
        image: ImageBitmapSource,
        options: &ImageBitmapOptions,
    ) -> Rc<Promise> {
        let p = self
            .upcast::<GlobalScope>()
            .create_image_bitmap(image, options);
        p
    }

    // https://html.spec.whatwg.org/multipage/#dom-window
    fn Window(&self) -> DomRoot<WindowProxy> {
        self.window_proxy()
    }

    // https://html.spec.whatwg.org/multipage/#dom-self
    fn Self_(&self) -> DomRoot<WindowProxy> {
        self.window_proxy()
    }

    // https://html.spec.whatwg.org/multipage/#dom-frames
    fn Frames(&self) -> DomRoot<WindowProxy> {
        self.window_proxy()
    }

    // https://html.spec.whatwg.org/multipage/#accessing-other-browsing-contexts
    fn Length(&self) -> u32 {
        let doc = self.Document();
        doc.iter_iframes().count() as u32
    }

    // https://html.spec.whatwg.org/multipage/#dom-parent
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

    // https://html.spec.whatwg.org/multipage/#dom-top
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
            let global_scope = self.upcast::<GlobalScope>();
            Performance::new(global_scope, self.navigation_start_precise.get())
        })
    }

    // https://html.spec.whatwg.org/multipage/#globaleventhandlers
    global_event_handlers!();

    // https://html.spec.whatwg.org/multipage/#windoweventhandlers
    window_event_handlers!();

    // https://developer.mozilla.org/en-US/docs/Web/API/Window/screen
    fn Screen(&self) -> DomRoot<Screen> {
        self.screen.or_init(|| Screen::new(self))
    }

    // https://html.spec.whatwg.org/multipage/#dom-windowbase64-btoa
    fn Btoa(&self, btoa: DOMString) -> Fallible<DOMString> {
        base64_btoa(btoa)
    }

    // https://html.spec.whatwg.org/multipage/#dom-windowbase64-atob
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

    // https://html.spec.whatwg.org/multipage/#dom-window-postmessage
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

    // https://html.spec.whatwg.org/multipage/#dom-window-captureevents
    fn CaptureEvents(&self) {
        // This method intentionally does nothing
    }

    // https://html.spec.whatwg.org/multipage/#dom-window-releaseevents
    fn ReleaseEvents(&self) {
        // This method intentionally does nothing
    }

    // check-tidy: no specs after this line
    fn Debug(&self, message: DOMString) {
        debug!("{}", message);
    }

    #[allow(unsafe_code)]
    fn Gc(&self) {
        unsafe {
            JS_GC(*self.get_cx(), GCReason::API);
        }
    }

    #[allow(unsafe_code)]
    fn Js_backtrace(&self) {
        unsafe {
            capture_stack!(in(*self.get_cx()) let stack);
            let js_stack = stack.and_then(|s| s.as_string(None, StackFormat::SpiderMonkey));
            let rust_stack = Backtrace::new();
            println!(
                "Current JS stack:\n{}\nCurrent Rust stack:\n{:?}",
                js_stack.unwrap_or_default(),
                rust_stack
            );
        }
    }

    #[allow(unsafe_code)]
    fn WebdriverCallback(&self, cx: JSContext, val: HandleValue) {
        let rv = unsafe { jsval_to_webdriver(*cx, &self.globalscope, val) };
        let opt_chan = self.webdriver_script_chan.borrow_mut().take();
        if let Some(chan) = opt_chan {
            chan.send(rv).unwrap();
        }
    }

    fn WebdriverTimeout(&self) {
        let opt_chan = self.webdriver_script_chan.borrow_mut().take();
        if let Some(chan) = opt_chan {
            chan.send(Err(WebDriverJSError::Timeout)).unwrap();
        }
    }

    // https://drafts.csswg.org/cssom/#dom-window-getcomputedstyle
    fn GetComputedStyle(
        &self,
        element: &Element,
        pseudo: Option<DOMString>,
    ) -> DomRoot<CSSStyleDeclaration> {
        // Steps 1-4.
        let pseudo = match pseudo.map(|mut s| {
            s.make_ascii_lowercase();
            s
        }) {
            Some(ref pseudo) if pseudo == ":before" || pseudo == "::before" => {
                Some(PseudoElement::Before)
            },
            Some(ref pseudo) if pseudo == ":after" || pseudo == "::after" => {
                Some(PseudoElement::After)
            },
            _ => None,
        };

        // Step 5.
        CSSStyleDeclaration::new(
            self,
            CSSStyleOwner::Element(Dom::from_ref(element)),
            pseudo,
            CSSModificationAccess::Readonly,
        )
    }

    // https://drafts.csswg.org/cssom-view/#dom-window-innerheight
    //TODO Include Scrollbar
    fn InnerHeight(&self) -> i32 {
        self.window_size
            .get()
            .initial_viewport
            .height
            .to_i32()
            .unwrap_or(0)
    }

    // https://drafts.csswg.org/cssom-view/#dom-window-innerwidth
    //TODO Include Scrollbar
    fn InnerWidth(&self) -> i32 {
        self.window_size
            .get()
            .initial_viewport
            .width
            .to_i32()
            .unwrap_or(0)
    }

    // https://drafts.csswg.org/cssom-view/#dom-window-scrollx
    fn ScrollX(&self) -> i32 {
        self.current_viewport.get().origin.x.to_px()
    }

    // https://drafts.csswg.org/cssom-view/#dom-window-pagexoffset
    fn PageXOffset(&self) -> i32 {
        self.ScrollX()
    }

    // https://drafts.csswg.org/cssom-view/#dom-window-scrolly
    fn ScrollY(&self) -> i32 {
        self.current_viewport.get().origin.y.to_px()
    }

    // https://drafts.csswg.org/cssom-view/#dom-window-pageyoffset
    fn PageYOffset(&self) -> i32 {
        self.ScrollY()
    }

    // https://drafts.csswg.org/cssom-view/#dom-window-scroll
    fn Scroll(&self, options: &ScrollToOptions) {
        // Step 1
        let left = options.left.unwrap_or(0.0f64);
        let top = options.top.unwrap_or(0.0f64);
        self.scroll(left, top, options.parent.behavior);
    }

    // https://drafts.csswg.org/cssom-view/#dom-window-scroll
    fn Scroll_(&self, x: f64, y: f64) {
        self.scroll(x, y, ScrollBehavior::Auto);
    }

    // https://drafts.csswg.org/cssom-view/#dom-window-scrollto
    fn ScrollTo(&self, options: &ScrollToOptions) {
        self.Scroll(options);
    }

    // https://drafts.csswg.org/cssom-view/#dom-window-scrollto
    fn ScrollTo_(&self, x: f64, y: f64) {
        self.scroll(x, y, ScrollBehavior::Auto);
    }

    // https://drafts.csswg.org/cssom-view/#dom-window-scrollby
    fn ScrollBy(&self, options: &ScrollToOptions) {
        // Step 1
        let x = options.left.unwrap_or(0.0f64);
        let y = options.top.unwrap_or(0.0f64);
        self.ScrollBy_(x, y);
        self.scroll(x, y, options.parent.behavior);
    }

    // https://drafts.csswg.org/cssom-view/#dom-window-scrollby
    fn ScrollBy_(&self, x: f64, y: f64) {
        // Step 3
        let left = x + self.ScrollX() as f64;
        // Step 4
        let top = y + self.ScrollY() as f64;

        // Step 5
        self.scroll(left, top, ScrollBehavior::Auto);
    }

    // https://drafts.csswg.org/cssom-view/#dom-window-resizeto
    fn ResizeTo(&self, width: i32, height: i32) {
        // Step 1
        //TODO determine if this operation is allowed
        let dpr = self.device_pixel_ratio();
        let size = Size2D::new(width, height).to_f32() * dpr;
        self.send_to_embedder(EmbedderMsg::ResizeTo(size.to_i32()));
    }

    // https://drafts.csswg.org/cssom-view/#dom-window-resizeby
    fn ResizeBy(&self, x: i32, y: i32) {
        let (size, _) = self.client_window();
        // Step 1
        self.ResizeTo(
            x + size.width.to_i32().unwrap_or(1),
            y + size.height.to_i32().unwrap_or(1),
        )
    }

    // https://drafts.csswg.org/cssom-view/#dom-window-moveto
    fn MoveTo(&self, x: i32, y: i32) {
        // Step 1
        //TODO determine if this operation is allowed
        let dpr = self.device_pixel_ratio();
        let point = Point2D::new(x, y).to_f32() * dpr;
        let msg = EmbedderMsg::MoveTo(point.to_i32());
        self.send_to_embedder(msg);
    }

    // https://drafts.csswg.org/cssom-view/#dom-window-moveby
    fn MoveBy(&self, x: i32, y: i32) {
        let (_, origin) = self.client_window();
        // Step 1
        self.MoveTo(x + origin.x, y + origin.y)
    }

    // https://drafts.csswg.org/cssom-view/#dom-window-screenx
    fn ScreenX(&self) -> i32 {
        let (_, origin) = self.client_window();
        origin.x
    }

    // https://drafts.csswg.org/cssom-view/#dom-window-screeny
    fn ScreenY(&self) -> i32 {
        let (_, origin) = self.client_window();
        origin.y
    }

    // https://drafts.csswg.org/cssom-view/#dom-window-outerheight
    fn OuterHeight(&self) -> i32 {
        let (size, _) = self.client_window();
        size.height.to_i32().unwrap_or(1)
    }

    // https://drafts.csswg.org/cssom-view/#dom-window-outerwidth
    fn OuterWidth(&self) -> i32 {
        let (size, _) = self.client_window();
        size.width.to_i32().unwrap_or(1)
    }

    // https://drafts.csswg.org/cssom-view/#dom-window-devicepixelratio
    fn DevicePixelRatio(&self) -> Finite<f64> {
        Finite::wrap(self.device_pixel_ratio().get() as f64)
    }

    // https://html.spec.whatwg.org/multipage/#dom-window-status
    fn Status(&self) -> DOMString {
        self.status.borrow().clone()
    }

    // https://html.spec.whatwg.org/multipage/#dom-window-status
    fn SetStatus(&self, status: DOMString) {
        *self.status.borrow_mut() = status
    }

    // https://drafts.csswg.org/cssom-view/#dom-window-matchmedia
    fn MatchMedia(&self, query: DOMString) -> DomRoot<MediaQueryList> {
        let mut input = ParserInput::new(&query);
        let mut parser = Parser::new(&mut input);
        let url_data = UrlExtraData(self.get_url().get_arc());
        let quirks_mode = self.Document().quirks_mode();
        let context = CssParserContext::new(
            Origin::Author,
            &url_data,
            Some(CssRuleType::Media),
            ParsingMode::DEFAULT,
            quirks_mode,
            /* namespaces = */ Default::default(),
            self.css_error_reporter(),
            None,
        );
        let media_query_list = media_queries::MediaList::parse(&context, &mut parser);
        let document = self.Document();
        let mql = MediaQueryList::new(&document, media_query_list);
        self.media_query_lists.track(&*mql);
        mql
    }

    // https://fetch.spec.whatwg.org/#fetch-method
    fn Fetch(
        &self,
        input: RequestOrUSVString,
        init: RootedTraceableBox<RequestInit>,
        comp: InRealm,
    ) -> Rc<Promise> {
        fetch::Fetch(self.upcast(), input, init, comp)
    }

    fn TestRunner(&self) -> DomRoot<TestRunner> {
        self.test_runner.or_init(|| TestRunner::new(self.upcast()))
    }

    fn RunningAnimationCount(&self) -> u32 {
        self.document
            .get()
            .map_or(0, |d| d.animations().running_animation_count() as u32)
    }

    // https://html.spec.whatwg.org/multipage/#dom-name
    fn SetName(&self, name: DOMString) {
        if let Some(proxy) = self.undiscarded_window_proxy() {
            proxy.set_name(name);
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-name
    fn Name(&self) -> DOMString {
        match self.undiscarded_window_proxy() {
            Some(proxy) => proxy.get_name(),
            None => "".into(),
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-origin
    fn Origin(&self) -> USVString {
        USVString(self.origin().immutable().ascii_serialization())
    }

    // https://w3c.github.io/selection-api/#dom-window-getselection
    fn GetSelection(&self) -> Option<DomRoot<Selection>> {
        self.document.get().and_then(|d| d.GetSelection())
    }

    // https://dom.spec.whatwg.org/#dom-window-event
    #[allow(unsafe_code)]
    fn Event(&self, cx: JSContext) -> JSVal {
        rooted!(in(*cx) let mut rval = UndefinedValue());
        if let Some(ref event) = *self.current_event.borrow() {
            unsafe {
                event
                    .reflector()
                    .get_jsobject()
                    .to_jsval(*cx, rval.handle_mut());
            }
        }
        rval.get()
    }

    fn IsSecureContext(&self) -> bool {
        self.upcast::<GlobalScope>().is_secure_context()
    }

    // https://html.spec.whatwg.org/multipage/#named-access-on-the-window-object
    #[allow(unsafe_code)]
    fn NamedGetter(&self, _cx: JSContext, name: DOMString) -> Option<NonNull<JSObject>> {
        if name.is_empty() {
            return None;
        }
        let document = self.Document();

        // https://html.spec.whatwg.org/multipage/#document-tree-child-browsing-context-name-property-set
        let iframes: Vec<_> = document
            .iter_iframes()
            .filter(|iframe| {
                if let Some(window) = iframe.GetContentWindow() {
                    return window.get_name() == name;
                }
                false
            })
            .collect();

        let iframe_iter = iframes.iter().map(|iframe| iframe.upcast::<Element>());

        let name = Atom::from(&*name);

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
                unsafe {
                    return Some(NonNull::new_unchecked(
                        nested_window_proxy.reflector().get_jsobject().get(),
                    ));
                }
            }
        }

        let mut elements = iframe_iter.chain(name_iter).chain(id_iter);

        let first = elements.next()?;

        if elements.next().is_none() {
            // Step 3.
            unsafe {
                return Some(NonNull::new_unchecked(
                    first.reflector().get_jsobject().get(),
                ));
            }
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
        );
        unsafe {
            Some(NonNull::new_unchecked(
                collection.reflector().get_jsobject().get(),
            ))
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-tree-accessors:supported-property-names
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
}

impl Window {
    // https://heycam.github.io/webidl/#named-properties-object
    // https://html.spec.whatwg.org/multipage/#named-access-on-the-window-object
    #[allow(unsafe_code)]
    pub fn create_named_properties_object(
        cx: JSContext,
        proto: HandleObject,
        object: MutableHandleObject,
    ) {
        window_named_properties::create(cx, proto, object)
    }

    pub(crate) fn set_current_event(&self, event: Option<&Event>) -> Option<DomRoot<Event>> {
        let current = self
            .current_event
            .borrow()
            .as_ref()
            .map(|e| DomRoot::from_ref(&**e));
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
                Err(_) => return Err(Error::Syntax),
            },
        };

        // Step 9.
        self.post_message(target_origin, source_origin, &source.window_proxy(), data);
        Ok(())
    }

    // https://drafts.css-houdini.org/css-paint-api-1/#paint-worklet
    pub fn paint_worklet(&self) -> DomRoot<Worklet> {
        self.paint_worklet.or_init(|| self.new_paint_worklet())
    }

    pub fn get_navigation_start(&self) -> u64 {
        self.navigation_start_precise.get()
    }

    pub fn has_document(&self) -> bool {
        self.document.get().is_some()
    }

    /// Cancels all the tasks associated with that window.
    ///
    /// This sets the current `task_manager.task_cancellers` sentinel value to
    /// `true` and replaces it with a brand new one for future tasks.
    pub fn cancel_all_tasks(&self) {
        let mut ignore_flags = self.task_manager.task_cancellers.borrow_mut();
        for task_source_name in TaskSourceName::all() {
            let flag = ignore_flags.entry(task_source_name).or_default();
            let cancelled = std::mem::take(&mut *flag);
            cancelled.store(true, Ordering::SeqCst);
        }
    }

    /// Cancels all the tasks from a given task source.
    /// This sets the current sentinel value to
    /// `true` and replaces it with a brand new one for future tasks.
    pub fn cancel_all_tasks_from_source(&self, task_source_name: TaskSourceName) {
        let mut ignore_flags = self.task_manager.task_cancellers.borrow_mut();
        let flag = ignore_flags.entry(task_source_name).or_default();
        let cancelled = std::mem::take(&mut *flag);
        cancelled.store(true, Ordering::SeqCst);
    }

    pub fn clear_js_runtime(&self) {
        self.upcast::<GlobalScope>()
            .remove_web_messaging_and_dedicated_workers_infra();

        // Clean up any active promises
        // https://github.com/servo/servo/issues/15318
        if let Some(custom_elements) = self.custom_element_registry.get() {
            custom_elements.teardown();
        }

        // The above code may not catch all DOM objects (e.g. DOM
        // objects removed from the tree that haven't been collected
        // yet). There should not be any such DOM nodes with layout
        // data, but if there are, then when they are dropped, they
        // will attempt to send a message to layout.
        // This causes memory safety issues, because the DOM node uses
        // the layout channel from its window, and the window has
        // already been GC'd.  For nodes which do not have a live
        // pointer, we can avoid this by GCing now:
        self.Gc();
        // but there may still be nodes being kept alive by user
        // script.
        // TODO: ensure that this doesn't happen!

        self.current_state.set(WindowState::Zombie);
        *self.js_runtime.borrow_mut() = None;

        // If this is the currently active pipeline,
        // nullify the window_proxy.
        if let Some(proxy) = self.window_proxy.get() {
            let pipeline_id = self.upcast::<GlobalScope>().pipeline_id();
            if let Some(currently_active) = proxy.currently_active() {
                if currently_active == pipeline_id {
                    self.window_proxy.set(None);
                }
            }
        }

        if let Some(performance) = self.performance.get() {
            performance.clear_and_disable_performance_entry_buffer();
        }
        self.ignore_all_tasks();
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-window-scroll>
    pub fn scroll(&self, x_: f64, y_: f64, behavior: ScrollBehavior) {
        // Step 3
        let xfinite = if x_.is_finite() { x_ } else { 0.0f64 };
        let yfinite = if y_.is_finite() { y_ } else { 0.0f64 };

        // TODO Step 4 - determine if a window has a viewport

        // Step 5 & 6
        // TODO: Remove scrollbar dimensions.
        let viewport = self.window_size.get().initial_viewport;

        // Step 7 & 8
        // TODO: Consider `block-end` and `inline-end` overflow direction.
        let scrolling_area = self.scrolling_area_query(None);
        let x = xfinite
            .min(scrolling_area.width() as f64 - viewport.width as f64)
            .max(0.0f64);
        let y = yfinite
            .min(scrolling_area.height() as f64 - viewport.height as f64)
            .max(0.0f64);

        // Step 10
        //TODO handling ongoing smooth scrolling
        if x == self.ScrollX() as f64 && y == self.ScrollY() as f64 {
            return;
        }

        //TODO Step 11
        //let document = self.Document();
        // Step 12
        let x = x.to_f32().unwrap_or(0.0f32);
        let y = y.to_f32().unwrap_or(0.0f32);
        self.update_viewport_for_scroll(x, y);
        self.perform_a_scroll(
            x,
            y,
            self.upcast::<GlobalScope>().pipeline_id().root_scroll_id(),
            behavior,
            None,
        );
    }

    /// <https://drafts.csswg.org/cssom-view/#perform-a-scroll>
    pub fn perform_a_scroll(
        &self,
        x: f32,
        y: f32,
        scroll_id: ExternalScrollId,
        _behavior: ScrollBehavior,
        _element: Option<&Element>,
    ) {
        // TODO Step 1
        // TODO(mrobinson, #18709): Add smooth scrolling support to WebRender so that we can
        // properly process ScrollBehavior here.
        self.reflow(
            ReflowGoal::UpdateScrollNode(ScrollState {
                scroll_id,
                scroll_offset: Vector2D::new(-x, -y),
            }),
            ReflowReason::ScrollFromScript,
        );
    }

    pub fn update_viewport_for_scroll(&self, x: f32, y: f32) {
        let size = self.current_viewport.get().size;
        let new_viewport = Rect::new(Point2D::new(Au::from_f32_px(x), Au::from_f32_px(y)), size);
        self.current_viewport.set(new_viewport)
    }

    pub fn device_pixel_ratio(&self) -> Scale<f32, CSSPixel, DevicePixel> {
        self.window_size.get().device_pixel_ratio
    }

    fn client_window(&self) -> (Size2D<u32, CSSPixel>, Point2D<i32, CSSPixel>) {
        let timer_profile_chan = self.global().time_profiler_chan().clone();
        let (send, recv) =
            ProfiledIpc::channel::<(DeviceIntSize, DeviceIntPoint)>(timer_profile_chan).unwrap();
        self.send_to_constellation(ScriptMsg::GetClientWindow(send));
        let (size, point) = recv.recv().unwrap_or((Size2D::zero(), Point2D::zero()));
        let dpr = self.device_pixel_ratio();
        (
            (size.to_f32() / dpr).to_u32(),
            (point.to_f32() / dpr).to_i32(),
        )
    }

    /// Prepares to tick animations and then does a reflow which also advances the
    /// layout animation clock.
    #[allow(unsafe_code)]
    pub fn advance_animation_clock(&self, delta_ms: i32) {
        let pipeline_id = self.upcast::<GlobalScope>().pipeline_id();
        self.Document()
            .advance_animation_timeline_for_testing(delta_ms as f64 / 1000.);
        ScriptThread::handle_tick_all_animations_for_testing(pipeline_id);
    }

    /// Reflows the page unconditionally if possible and not suppressed. This method will wait for
    /// the layout to complete. If there is no window size yet, the page is presumed invisible and
    /// no reflow is performed. If reflow is suppressed, no reflow will be performed for ForDisplay
    /// goals.
    ///
    /// Returns true if layout actually happened, false otherwise.
    #[allow(unsafe_code)]
    pub(crate) fn force_reflow(
        &self,
        reflow_goal: ReflowGoal,
        reason: ReflowReason,
        condition: Option<ReflowTriggerCondition>,
    ) -> bool {
        self.Document().ensure_safe_to_run_script_or_layout();
        // Check if we need to unsuppress reflow. Note that this needs to be
        // *before* any early bailouts, or reflow might never be unsuppresed!
        match reason {
            ReflowReason::FirstLoad | ReflowReason::RefreshTick => self.suppress_reflow.set(false),
            _ => (),
        }

        let for_display = reflow_goal == ReflowGoal::Full;
        let pipeline_id = self.upcast::<GlobalScope>().pipeline_id();
        if for_display && self.suppress_reflow.get() {
            debug!(
                "Suppressing reflow pipeline {} for reason {:?} before FirstLoad or RefreshTick",
                pipeline_id, reason
            );
            return false;
        }

        if condition != Some(ReflowTriggerCondition::PaintPostponed) {
            debug!(
                "Invalidating layout cache due to reflow condition {:?}",
                condition
            );
            // Invalidate any existing cached layout values.
            self.layout_marker.borrow().set(false);
            // Create a new layout caching token.
            *self.layout_marker.borrow_mut() = Rc::new(Cell::new(true));
        } else {
            debug!("Not invalidating cached layout values for paint-only reflow.");
        }

        debug!("script: performing reflow for reason {:?}", reason);

        let marker = if self.need_emit_timeline_marker(TimelineMarkerType::Reflow) {
            Some(TimelineMarker::start("Reflow".to_owned()))
        } else {
            None
        };

        // Layout will let us know when it's done.
        let (join_chan, join_port) = unbounded();

        // On debug mode, print the reflow event information.
        if self.relayout_event {
            debug_reflow_events(pipeline_id, &reflow_goal, &reason);
        }

        let document = self.Document();

        let stylesheets_changed = document.flush_stylesheets_for_reflow();

        // If this reflow is for display, ensure webgl canvases are composited with
        // up-to-date contents.
        if for_display {
            document.flush_dirty_webgpu_canvases();
            document.flush_dirty_webgl_canvases();
        }

        let pending_restyles = document.drain_pending_restyles();

        let dirty_root = document
            .take_dirty_root()
            .filter(|_| !stylesheets_changed)
            .or_else(|| document.GetDocumentElement())
            .map(|root| root.upcast::<Node>().to_trusted_node_address());

        // Send new document and relevant styles to layout.
        let needs_display = reflow_goal.needs_display();
        let reflow = ScriptReflow {
            reflow_info: Reflow {
                page_clip_rect: self.page_clip_rect.get(),
            },
            document: document.upcast::<Node>().to_trusted_node_address(),
            dirty_root,
            stylesheets_changed,
            window_size: self.window_size.get(),
            origin: self.origin().immutable().clone(),
            reflow_goal,
            script_join_chan: join_chan,
            dom_count: document.dom_count(),
            pending_restyles,
            animation_timeline_value: document.current_animation_timeline_value(),
            animations: document.animations().sets.clone(),
        };

        let _ = self.with_layout(move |layout| layout.reflow(reflow));

        let complete = match join_port.try_recv() {
            Err(TryRecvError::Empty) => {
                debug!("script: waiting on layout");
                join_port.recv().unwrap()
            },
            Ok(reflow_complete) => reflow_complete,
            Err(TryRecvError::Disconnected) => {
                panic!("Layout failed while script was waiting for a result.");
            },
        };

        debug!("script: layout complete");

        // Pending reflows require display, so only reset the pending reflow count if this reflow
        // was to be displayed.
        if needs_display {
            self.pending_reflow_count.set(0);
        }

        if let Some(marker) = marker {
            self.emit_timeline_marker(marker.end());
        }

        for image in complete.pending_images {
            let id = image.id;
            let node = unsafe { from_untrusted_node_address(image.node) };

            if let PendingImageState::Unrequested(ref url) = image.state {
                fetch_image_for_layout(url.clone(), &node, id, self.image_cache.clone());
            }

            let mut images = self.pending_layout_images.borrow_mut();
            let nodes = images.entry(id).or_default();
            if !nodes.iter().any(|n| std::ptr::eq(&**n, &*node)) {
                let (responder, responder_listener) =
                    ProfiledIpc::channel(self.global().time_profiler_chan().clone()).unwrap();
                let image_cache_chan = self.image_cache_chan.clone();
                ROUTER.add_route(
                    responder_listener.to_opaque(),
                    Box::new(move |message| {
                        let _ = image_cache_chan.send((pipeline_id, message.to().unwrap()));
                    }),
                );
                self.image_cache
                    .add_listener(id, ImageResponder::new(responder, id));
                nodes.push(Dom::from_ref(&*node));
            }
        }

        document.update_animations_post_reflow();
        self.update_constellation_epoch();

        true
    }

    /// Reflows the page if it's possible to do so and the page is dirty.
    ///
    /// Returns true if layout actually happened, false otherwise.
    /// This return value is useful for script queries, that wait for a lock
    /// that layout might hold if the first layout hasn't happened yet (which
    /// may happen in the only case a query reflow may bail out, that is, if the
    /// viewport size is not present). See #11223 for an example of that.
    pub fn reflow(&self, reflow_goal: ReflowGoal, reason: ReflowReason) -> bool {
        self.Document().ensure_safe_to_run_script_or_layout();
        let for_display = reflow_goal == ReflowGoal::Full;

        let mut issued_reflow = false;
        let condition = self.Document().needs_reflow();
        if !for_display || condition.is_some() {
            issued_reflow = self.force_reflow(reflow_goal, reason, condition);

            // We shouldn't need a reflow immediately after a
            // reflow, except if we're waiting for a deferred paint.
            let condition = self.Document().needs_reflow();
            assert!(
                {
                    condition.is_none() ||
                        (!for_display &&
                            condition == Some(ReflowTriggerCondition::PaintPostponed)) ||
                        self.suppress_reflow.get()
                },
                "condition was {:?}",
                condition
            );
        } else {
            debug!(
                "Document doesn't need reflow - skipping it (reason {:?})",
                reason
            );
        }

        // If writing a screenshot, check if the script has reached a state
        // where it's safe to write the image. This means that:
        // 1) The reflow is for display (otherwise it could be a query)
        // 2) The html element doesn't contain the 'reftest-wait' class
        // 3) The load event has fired.
        // When all these conditions are met, notify the constellation
        // that this pipeline is ready to write the image (from the script thread
        // perspective at least).
        if self.prepare_for_screenshot && for_display {
            let document = self.Document();

            // Checks if the html element has reftest-wait attribute present.
            // See http://testthewebforward.org/docs/reftests.html
            let html_element = document.GetDocumentElement();
            let reftest_wait = html_element.map_or(false, |elem| {
                elem.has_class(&atom!("reftest-wait"), CaseSensitivity::CaseSensitive)
            });

            let pending_web_fonts = self
                .with_layout(move |layout| layout.waiting_for_web_fonts_to_load())
                .unwrap();

            let has_sent_idle_message = self.has_sent_idle_message.get();
            let is_ready_state_complete = document.ReadyState() == DocumentReadyState::Complete;
            let pending_images = !self.pending_layout_images.borrow().is_empty();

            if !has_sent_idle_message &&
                is_ready_state_complete &&
                !reftest_wait &&
                !pending_images &&
                !pending_web_fonts
            {
                debug!(
                    "{:?}: Sending DocumentState::Idle to Constellation",
                    self.pipeline_id()
                );
                let event = ScriptMsg::SetDocumentState(DocumentState::Idle);
                self.send_to_constellation(event);
                self.has_sent_idle_message.set(true);
            }
        }

        issued_reflow
    }

    /// If writing a screenshot, synchronously update the layout epoch that it set
    /// in the constellation.
    pub(crate) fn update_constellation_epoch(&self) {
        if !self.prepare_for_screenshot {
            return;
        }

        let epoch = self
            .with_layout(move |layout| layout.current_epoch())
            .unwrap();

        debug!(
            "{:?}: Updating constellation epoch: {epoch:?}",
            self.pipeline_id()
        );
        let (sender, receiver) = ipc::channel().expect("Failed to create IPC channel!");
        let event = ScriptMsg::SetLayoutEpoch(epoch, sender);
        self.send_to_constellation(event);
        receiver.recv().unwrap();
    }

    pub fn layout_reflow(&self, query_msg: QueryMsg) -> bool {
        self.reflow(
            ReflowGoal::LayoutQuery(query_msg, time::precise_time_ns()),
            ReflowReason::Query,
        )
    }

    pub fn resolved_font_style_query(&self, node: &Node, value: String) -> Option<ServoArc<Font>> {
        if !self.layout_reflow(QueryMsg::ResolvedFontStyleQuery) {
            return None;
        }

        let document = self.Document();
        self.with_layout(|layout| {
            layout.query_resolved_font_style(
                node.to_trusted_node_address(),
                &value,
                document.animations().sets.clone(),
                document.current_animation_timeline_value(),
            )
        })
        .unwrap()
    }

    pub fn content_box_query(&self, node: &Node) -> Option<UntypedRect<Au>> {
        if !self.layout_reflow(QueryMsg::ContentBox) {
            return None;
        }
        self.with_layout(|layout| layout.query_content_box(node.to_opaque()))
            .unwrap_or(None)
    }

    pub fn content_boxes_query(&self, node: &Node) -> Vec<UntypedRect<Au>> {
        if !self.layout_reflow(QueryMsg::ContentBoxes) {
            return vec![];
        }
        self.with_layout(|layout| layout.query_content_boxes(node.to_opaque()))
            .unwrap_or_default()
    }

    pub fn client_rect_query(&self, node: &Node) -> UntypedRect<i32> {
        if !self.layout_reflow(QueryMsg::ClientRectQuery) {
            return Rect::zero();
        }
        self.with_layout(|layout| layout.query_client_rect(node.to_opaque()))
            .unwrap_or_default()
    }

    /// Find the scroll area of the given node, if it is not None. If the node
    /// is None, find the scroll area of the viewport.
    pub fn scrolling_area_query(&self, node: Option<&Node>) -> UntypedRect<i32> {
        let opaque = node.map(|node| node.to_opaque());
        if !self.layout_reflow(QueryMsg::ScrollingAreaQuery) {
            return Rect::zero();
        }
        self.with_layout(|layout| layout.query_scrolling_area(opaque))
            .unwrap_or_default()
    }

    pub fn scroll_offset_query(&self, node: &Node) -> Vector2D<f32, LayoutPixel> {
        if let Some(scroll_offset) = self.scroll_offsets.borrow().get(&node.to_opaque()) {
            return *scroll_offset;
        }
        Vector2D::new(0.0, 0.0)
    }

    // https://drafts.csswg.org/cssom-view/#element-scrolling-members
    pub fn scroll_node(&self, node: &Node, x_: f64, y_: f64, behavior: ScrollBehavior) {
        // The scroll offsets are immediatly updated since later calls
        // to topScroll and others may access the properties before
        // webrender has a chance to update the offsets.
        self.scroll_offsets
            .borrow_mut()
            .insert(node.to_opaque(), Vector2D::new(x_ as f32, y_ as f32));
        let scroll_id = ExternalScrollId(
            combine_id_with_fragment_type(
                node.to_opaque().id(),
                gfx_traits::FragmentType::FragmentBody,
            ),
            self.pipeline_id().into(),
        );

        // Step 12
        self.perform_a_scroll(
            x_.to_f32().unwrap_or(0.0f32),
            y_.to_f32().unwrap_or(0.0f32),
            scroll_id,
            behavior,
            None,
        );
    }

    pub fn resolved_style_query(
        &self,
        element: TrustedNodeAddress,
        pseudo: Option<PseudoElement>,
        property: PropertyId,
    ) -> DOMString {
        if !self.layout_reflow(QueryMsg::ResolvedStyleQuery) {
            return DOMString::new();
        }

        let document = self.Document();
        DOMString::from(
            self.with_layout(|layout| {
                layout.query_resolved_style(
                    element,
                    pseudo,
                    property,
                    document.animations().sets.clone(),
                    document.current_animation_timeline_value(),
                )
            })
            .unwrap(),
        )
    }

    pub fn inner_window_dimensions_query(
        &self,
        browsing_context: BrowsingContextId,
    ) -> Option<Size2D<f32, CSSPixel>> {
        if !self.layout_reflow(QueryMsg::InnerWindowDimensionsQuery) {
            return None;
        }
        self.with_layout(|layout| layout.query_inner_window_dimension(browsing_context))
            .unwrap()
    }

    #[allow(unsafe_code)]
    pub fn offset_parent_query(&self, node: &Node) -> (Option<DomRoot<Element>>, UntypedRect<Au>) {
        if !self.layout_reflow(QueryMsg::OffsetParentQuery) {
            return (None, Rect::zero());
        }

        let response = self
            .with_layout(|layout| layout.query_offset_parent(node.to_opaque()))
            .unwrap();
        let element = response.node_address.and_then(|parent_node_address| {
            let node = unsafe { from_untrusted_node_address(parent_node_address) };
            DomRoot::downcast(node)
        });
        (element, response.rect)
    }

    pub fn text_index_query(
        &self,
        node: &Node,
        point_in_node: UntypedPoint2D<f32>,
    ) -> Option<usize> {
        if !self.layout_reflow(QueryMsg::TextIndexQuery) {
            return None;
        }
        self.with_layout(|layout| layout.query_text_indext(node.to_opaque(), point_in_node))
            .unwrap()
    }

    #[allow(unsafe_code)]
    pub fn init_window_proxy(&self, window_proxy: &WindowProxy) {
        assert!(self.window_proxy.get().is_none());
        self.window_proxy.set(Some(window_proxy));
    }

    #[allow(unsafe_code)]
    pub fn init_document(&self, document: &Document) {
        assert!(self.document.get().is_none());
        assert!(document.window() == self);
        self.document.set(Some(document));
        if !self.unminify_js {
            return;
        }
        // Set a path for the document host to store unminified scripts.
        let mut path = env::current_dir().unwrap();
        path.push("unminified-js");
        *self.unminified_js_dir.borrow_mut() = Some(path.into_os_string().into_string().unwrap());
    }

    /// Commence a new URL load which will either replace this window or scroll to a fragment.
    ///
    /// <https://html.spec.whatwg.org/multipage/#navigating-across-documents>
    pub fn load_url(
        &self,
        replace: HistoryEntryReplacement,
        force_reload: bool,
        load_data: LoadData,
    ) {
        let doc = self.Document();
        // TODO: Important re security. See https://github.com/servo/servo/issues/23373
        // Step 3: check that the source browsing-context is "allowed to navigate" this window.
        if !force_reload &&
            load_data.url.as_url()[..Position::AfterQuery] ==
                doc.url().as_url()[..Position::AfterQuery]
        {
            // Step 6
            if let Some(fragment) = load_data.url.fragment() {
                self.send_to_constellation(ScriptMsg::NavigatedToFragment(
                    load_data.url.clone(),
                    replace,
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
                        new_url);
                    event.upcast::<Event>().fire(this.upcast::<EventTarget>());
                });
                // FIXME(nox): Why are errors silenced here?
                let _ = self.script_chan.send(CommonScriptMsg::Task(
                    ScriptThreadEventCategory::DomEvent,
                    Box::new(
                        self.task_manager
                            .task_canceller(TaskSourceName::DOMManipulation)
                            .wrap_task(task),
                    ),
                    Some(self.pipeline_id()),
                    TaskSourceName::DOMManipulation,
                ));
                doc.set_url(load_data.url.clone());
                return;
            }
        }

        let pipeline_id = self.upcast::<GlobalScope>().pipeline_id();

        // Step 4 and 5
        let window_proxy = self.window_proxy();
        if let Some(active) = window_proxy.currently_active() {
            if pipeline_id == active && doc.is_prompting_or_unloading() {
                return;
            }
        }

        // Step 8
        if doc.prompt_to_unload(false) {
            let window_proxy = self.window_proxy();
            if window_proxy.parent().is_some() {
                // Step 10
                // If browsingContext is a nested browsing context,
                // then put it in the delaying load events mode.
                window_proxy.start_delaying_load_events_mode();
            }
            // TODO: step 11, navigationType.
            // Step 12, 13
            ScriptThread::navigate(
                window_proxy.browsing_context_id(),
                pipeline_id,
                load_data,
                replace,
            );
        };
    }

    pub fn handle_fire_timer(&self, timer_id: TimerEventId) {
        self.upcast::<GlobalScope>().fire_timer(timer_id);
        self.reflow(ReflowGoal::Full, ReflowReason::Timer);
    }

    pub fn set_window_size(&self, size: WindowSizeData) {
        self.window_size.set(size);
    }

    pub fn window_size(&self) -> WindowSizeData {
        self.window_size.get()
    }

    pub fn get_url(&self) -> ServoUrl {
        self.Document().url()
    }

    pub fn with_layout<T>(&self, call: impl FnOnce(&mut dyn Layout) -> T) -> Result<T, ()> {
        ScriptThread::with_layout(self.pipeline_id(), call)
    }

    pub fn windowproxy_handler(&self) -> WindowProxyHandler {
        WindowProxyHandler(self.dom_static.windowproxy_handler.0)
    }

    pub fn get_pending_reflow_count(&self) -> u32 {
        self.pending_reflow_count.get()
    }

    pub fn add_pending_reflow(&self) {
        self.pending_reflow_count
            .set(self.pending_reflow_count.get() + 1);
    }

    pub fn set_resize_event(&self, event: WindowSizeData, event_type: WindowSizeType) {
        self.resize_event.set(Some((event, event_type)));
    }

    pub fn steal_resize_event(&self) -> Option<(WindowSizeData, WindowSizeType)> {
        let event = self.resize_event.get();
        self.resize_event.set(None);
        event
    }

    pub fn set_page_clip_rect_with_new_viewport(&self, viewport: UntypedRect<f32>) -> bool {
        let rect = f32_rect_to_au_rect(viewport);
        self.current_viewport.set(rect);
        // We use a clipping rectangle that is five times the size of the of the viewport,
        // so that we don't collect display list items for areas too far outside the viewport,
        // but also don't trigger reflows every time the viewport changes.
        static VIEWPORT_EXPANSION: f32 = 2.0; // 2 lengths on each side plus original length is 5 total.
        let proposed_clip_rect = f32_rect_to_au_rect(viewport.inflate(
            viewport.size.width * VIEWPORT_EXPANSION,
            viewport.size.height * VIEWPORT_EXPANSION,
        ));
        let clip_rect = self.page_clip_rect.get();
        if proposed_clip_rect == clip_rect {
            return false;
        }

        let had_clip_rect = clip_rect != MaxRect::max_rect();
        if had_clip_rect && !should_move_clip_rect(clip_rect, viewport) {
            return false;
        }

        self.page_clip_rect.set(proposed_clip_rect);

        // If we didn't have a clip rect, the previous display doesn't need rebuilding
        // because it was built for infinite clip (MaxRect::amax_rect()).
        had_clip_rect
    }

    pub fn suspend(&self) {
        // Suspend timer events.
        self.upcast::<GlobalScope>().suspend();

        // Set the window proxy to be a cross-origin window.
        if self.window_proxy().currently_active() == Some(self.global().pipeline_id()) {
            self.window_proxy().unset_currently_active();
        }

        // A hint to the JS runtime that now would be a good time to
        // GC any unreachable objects generated by user script,
        // or unattached DOM nodes. Attached DOM nodes can't be GCd yet,
        // as the document might be reactivated later.
        self.Gc();
    }

    pub fn resume(&self) {
        // Resume timer events.
        self.upcast::<GlobalScope>().resume();

        // Set the window proxy to be this object.
        self.window_proxy().set_currently_active(self);

        // Push the document title to the compositor since we are
        // activating this document due to a navigation.
        self.Document().title_changed();
    }

    pub fn need_emit_timeline_marker(&self, timeline_type: TimelineMarkerType) -> bool {
        let markers = self.devtools_markers.borrow();
        markers.contains(&timeline_type)
    }

    pub fn emit_timeline_marker(&self, marker: TimelineMarker) {
        let sender = self.devtools_marker_sender.borrow();
        let sender = sender.as_ref().expect("There is no marker sender");
        sender.send(Some(marker)).unwrap();
    }

    pub fn set_devtools_timeline_markers(
        &self,
        markers: Vec<TimelineMarkerType>,
        reply: IpcSender<Option<TimelineMarker>>,
    ) {
        *self.devtools_marker_sender.borrow_mut() = Some(reply);
        self.devtools_markers.borrow_mut().extend(markers);
    }

    pub fn drop_devtools_timeline_markers(&self, markers: Vec<TimelineMarkerType>) {
        let mut devtools_markers = self.devtools_markers.borrow_mut();
        for marker in markers {
            devtools_markers.remove(&marker);
        }
        if devtools_markers.is_empty() {
            *self.devtools_marker_sender.borrow_mut() = None;
        }
    }

    pub fn set_webdriver_script_chan(&self, chan: Option<IpcSender<WebDriverJSResult>>) {
        *self.webdriver_script_chan.borrow_mut() = chan;
    }

    pub fn is_alive(&self) -> bool {
        self.current_state.get() == WindowState::Alive
    }

    // https://html.spec.whatwg.org/multipage/#top-level-browsing-context
    pub fn is_top_level(&self) -> bool {
        self.parent_info.is_none()
    }

    /// Evaluate media query lists and report changes
    /// <https://drafts.csswg.org/cssom-view/#evaluate-media-queries-and-report-changes>
    pub fn evaluate_media_queries_and_report_changes(&self) {
        rooted_vec!(let mut mql_list);
        self.media_query_lists.for_each(|mql| {
            if let MediaQueryListMatchState::Changed(_) = mql.evaluate_changes() {
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
            );
            event.upcast::<Event>().fire(mql.upcast::<EventTarget>());
        }
        self.Document().react_to_environment_changes();
    }

    /// Set whether to use less resources by running timers at a heavily limited rate.
    pub fn set_throttled(&self, throttled: bool) {
        self.throttled.set(throttled);
        if throttled {
            self.upcast::<GlobalScope>().slow_down_timers();
        } else {
            self.upcast::<GlobalScope>().speed_up_timers();
        }
    }

    pub fn throttled(&self) -> bool {
        self.throttled.get()
    }

    pub fn unminified_js_dir(&self) -> Option<String> {
        self.unminified_js_dir.borrow().clone()
    }

    pub fn local_script_source(&self) -> &Option<String> {
        &self.local_script_source
    }

    pub fn set_navigation_start(&self) {
        let current_time = time::get_time();
        let now = (current_time.sec * 1000 + current_time.nsec as i64 / 1000000) as u64;
        self.navigation_start.set(now);
        self.navigation_start_precise.set(time::precise_time_ns());
    }

    pub fn send_to_embedder(&self, msg: EmbedderMsg) {
        self.send_to_constellation(ScriptMsg::ForwardToEmbedder(msg));
    }

    pub fn send_to_constellation(&self, msg: ScriptMsg) {
        self.upcast::<GlobalScope>()
            .script_to_constellation_chan()
            .send(msg)
            .unwrap();
    }

    pub fn webrender_document(&self) -> DocumentId {
        self.webrender_document
    }

    pub fn in_immersive_xr_session(&self) -> bool {
        self.navigator
            .get()
            .as_ref()
            .and_then(|nav| nav.xr())
            .map_or(false, |xr| xr.pending_or_active_session())
    }
}

impl Window {
    #[allow(unsafe_code)]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        runtime: Rc<Runtime>,
        script_chan: MainThreadScriptChan,
        task_manager: TaskManager,
        image_cache_chan: Sender<ImageCacheMsg>,
        image_cache: Arc<dyn ImageCache>,
        resource_threads: ResourceThreads,
        bluetooth_thread: IpcSender<BluetoothRequest>,
        mem_profiler_chan: MemProfilerChan,
        time_profiler_chan: TimeProfilerChan,
        devtools_chan: Option<IpcSender<ScriptToDevtoolsControlMsg>>,
        constellation_chan: ScriptToConstellationChan,
        control_chan: IpcSender<ConstellationControlMsg>,
        scheduler_chan: IpcSender<TimerSchedulerMsg>,
        pipelineid: PipelineId,
        parent_info: Option<PipelineId>,
        window_size: WindowSizeData,
        origin: MutableOrigin,
        creator_url: ServoUrl,
        navigation_start: u64,
        navigation_start_precise: u64,
        webgl_chan: Option<WebGLChan>,
        webxr_registry: webxr_api::Registry,
        microtask_queue: Rc<MicrotaskQueue>,
        webrender_document: DocumentId,
        webrender_api_sender: WebrenderIpcSender,
        relayout_event: bool,
        prepare_for_screenshot: bool,
        unminify_js: bool,
        local_script_source: Option<String>,
        userscripts_path: Option<String>,
        is_headless: bool,
        replace_surrogates: bool,
        user_agent: Cow<'static, str>,
        player_context: WindowGLContext,
        gpu_id_hub: Arc<ParkMutex<Identities>>,
        inherited_secure_context: Option<bool>,
    ) -> DomRoot<Self> {
        let error_reporter = CSSErrorReporter {
            pipelineid,
            script_chan: Arc::new(Mutex::new(control_chan)),
        };
        let win = Box::new(Self {
            globalscope: GlobalScope::new_inherited(
                pipelineid,
                devtools_chan,
                mem_profiler_chan,
                time_profiler_chan,
                constellation_chan,
                scheduler_chan,
                resource_threads,
                origin,
                Some(creator_url),
                microtask_queue,
                is_headless,
                user_agent,
                gpu_id_hub,
                inherited_secure_context,
            ),
            script_chan,
            task_manager,
            image_cache_chan,
            image_cache,
            navigator: Default::default(),
            location: Default::default(),
            history: Default::default(),
            custom_element_registry: Default::default(),
            window_proxy: Default::default(),
            document: Default::default(),
            performance: Default::default(),
            navigation_start: Cell::new(navigation_start),
            navigation_start_precise: Cell::new(navigation_start_precise),
            screen: Default::default(),
            session_storage: Default::default(),
            local_storage: Default::default(),
            status: DomRefCell::new(DOMString::new()),
            parent_info,
            dom_static: GlobalStaticData::new(),
            js_runtime: DomRefCell::new(Some(runtime.clone())),
            bluetooth_thread,
            bluetooth_extra_permission_data: BluetoothExtraPermissionData::new(),
            page_clip_rect: Cell::new(MaxRect::max_rect()),
            resize_event: Default::default(),
            window_size: Cell::new(window_size),
            current_viewport: Cell::new(Rect::zero()),
            suppress_reflow: Cell::new(true),
            pending_reflow_count: Default::default(),
            current_state: Cell::new(WindowState::Alive),
            devtools_marker_sender: Default::default(),
            devtools_markers: Default::default(),
            webdriver_script_chan: Default::default(),
            error_reporter,
            scroll_offsets: Default::default(),
            media_query_lists: DOMTracker::new(),
            test_runner: Default::default(),
            webgl_chan,
            webxr_registry,
            pending_layout_images: Default::default(),
            unminified_js_dir: Default::default(),
            local_script_source,
            test_worklet: Default::default(),
            paint_worklet: Default::default(),
            webrender_document,
            exists_mut_observer: Cell::new(false),
            webrender_api_sender,
            has_sent_idle_message: Cell::new(false),
            relayout_event,
            prepare_for_screenshot,
            unminify_js,
            userscripts_path,
            replace_surrogates,
            player_context,
            throttled: Cell::new(false),
            layout_marker: DomRefCell::new(Rc::new(Cell::new(true))),
            current_event: DomRefCell::new(None),
        });

        unsafe { WindowBinding::Wrap(JSContext::from_ptr(runtime.cx()), win) }
    }

    pub fn pipeline_id(&self) -> PipelineId {
        self.upcast::<GlobalScope>().pipeline_id()
    }

    /// Create a new cached instance of the given value.
    pub fn cache_layout_value<T>(&self, value: T) -> LayoutValue<T>
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
pub struct LayoutValue<T: MallocSizeOf> {
    #[ignore_malloc_size_of = "Rc is hard"]
    is_valid: Rc<Cell<bool>>,
    value: T,
}

#[allow(unsafe_code)]
unsafe impl<T: JSTraceable + MallocSizeOf> JSTraceable for LayoutValue<T> {
    unsafe fn trace(&self, trc: *mut js::jsapi::JSTracer) {
        self.value.trace(trc)
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
    pub fn get(&self) -> Result<T, ()> {
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

fn debug_reflow_events(id: PipelineId, reflow_goal: &ReflowGoal, reason: &ReflowReason) {
    let goal_string = match *reflow_goal {
        ReflowGoal::Full => "\tFull",
        ReflowGoal::TickAnimations => "\tTickAnimations",
        ReflowGoal::UpdateScrollNode(_) => "\tUpdateScrollNode",
        ReflowGoal::LayoutQuery(ref query_msg, _) => match *query_msg {
            QueryMsg::ContentBox => "\tContentBoxQuery",
            QueryMsg::ContentBoxes => "\tContentBoxesQuery",
            QueryMsg::NodesFromPointQuery => "\tNodesFromPointQuery",
            QueryMsg::ClientRectQuery => "\tClientRectQuery",
            QueryMsg::ScrollingAreaQuery => "\tNodeScrollGeometryQuery",
            QueryMsg::ResolvedStyleQuery => "\tResolvedStyleQuery",
            QueryMsg::ResolvedFontStyleQuery => "\nResolvedFontStyleQuery",
            QueryMsg::OffsetParentQuery => "\tOffsetParentQuery",
            QueryMsg::StyleQuery => "\tStyleQuery",
            QueryMsg::TextIndexQuery => "\tTextIndexQuery",
            QueryMsg::ElementInnerTextQuery => "\tElementInnerTextQuery",
            QueryMsg::InnerWindowDimensionsQuery => "\tInnerWindowDimensionsQuery",
        },
    };

    println!("**** pipeline={}\t{}\t{:?}", id, goal_string, reason);
}

impl Window {
    // https://html.spec.whatwg.org/multipage/#dom-window-postmessage step 7.
    pub fn post_message(
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
            if let Ok(ports) = structuredclone::read(this.upcast(), data, message_clone.handle_mut()) {
                // Step 7.6, 7.7
                MessageEvent::dispatch_jsval(
                    this.upcast(),
                    this.upcast(),
                    message_clone.handle(),
                    Some(&source_origin.ascii_serialization()),
                    Some(&*source),
                    ports,
                );
            } else {
                // Step 4, fire messageerror.
                MessageEvent::dispatch_error(
                    this.upcast(),
                    this.upcast(),
                );
            }
        });
        // FIXME(nox): Why are errors silenced here?
        // TODO(#12718): Use the "posted message task source".
        // TODO: When switching to the right task source, update the task_canceller call too.
        let _ = self.script_chan.send(CommonScriptMsg::Task(
            ScriptThreadEventCategory::DomEvent,
            Box::new(
                self.task_manager
                    .task_canceller(TaskSourceName::DOMManipulation)
                    .wrap_task(task),
            ),
            Some(self.pipeline_id()),
            TaskSourceName::DOMManipulation,
        ));
    }
}

#[derive(Clone, MallocSizeOf)]
pub struct CSSErrorReporter {
    pub pipelineid: PipelineId,
    // Arc+Mutex combo is necessary to make this struct Sync,
    // which is necessary to fulfill the bounds required by the
    // uses of the ParseErrorReporter trait.
    #[ignore_malloc_size_of = "Arc is defined in libstd"]
    pub script_chan: Arc<Mutex<IpcSender<ConstellationControlMsg>>>,
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

        //TODO: report a real filename
        let _ = self
            .script_chan
            .lock()
            .unwrap()
            .send(ConstellationControlMsg::ReportCSSError(
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
