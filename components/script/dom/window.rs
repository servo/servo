/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use app_units::Au;
use base64;
use bluetooth_traits::BluetoothRequest;
use cssparser::Parser;
use devtools_traits::{ScriptToDevtoolsControlMsg, TimelineMarker, TimelineMarkerType};
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::DocumentBinding::{DocumentMethods, DocumentReadyState};
use dom::bindings::codegen::Bindings::EventHandlerBinding::EventHandlerNonNull;
use dom::bindings::codegen::Bindings::EventHandlerBinding::OnBeforeUnloadEventHandlerNonNull;
use dom::bindings::codegen::Bindings::EventHandlerBinding::OnErrorEventHandlerNonNull;
use dom::bindings::codegen::Bindings::FunctionBinding::Function;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::codegen::Bindings::PermissionStatusBinding::PermissionState;
use dom::bindings::codegen::Bindings::RequestBinding::RequestInit;
use dom::bindings::codegen::Bindings::WindowBinding::{self, FrameRequestCallback, WindowMethods};
use dom::bindings::codegen::Bindings::WindowBinding::{ScrollBehavior, ScrollToOptions};
use dom::bindings::codegen::UnionTypes::RequestOrUSVString;
use dom::bindings::error::{Error, ErrorResult, Fallible};
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{JS, MutNullableJS, Root};
use dom::bindings::num::Finite;
use dom::bindings::refcounted::Trusted;
use dom::bindings::reflector::DomObject;
use dom::bindings::str::DOMString;
use dom::bindings::structuredclone::StructuredCloneData;
use dom::bindings::trace::RootedTraceableBox;
use dom::bindings::utils::{GlobalStaticData, WindowProxyHandler};
use dom::bluetooth::BluetoothExtraPermissionData;
use dom::crypto::Crypto;
use dom::cssstyledeclaration::{CSSModificationAccess, CSSStyleDeclaration, CSSStyleOwner};
use dom::document::{AnimationFrameCallback, Document};
use dom::element::Element;
use dom::event::Event;
use dom::globalscope::GlobalScope;
use dom::history::History;
use dom::htmliframeelement::build_mozbrowser_custom_event;
use dom::location::Location;
use dom::mediaquerylist::{MediaQueryList, WeakMediaQueryListVec};
use dom::messageevent::MessageEvent;
use dom::navigator::Navigator;
use dom::node::{Node, NodeDamage, document_from_node, from_untrusted_node_address};
use dom::performance::Performance;
use dom::promise::Promise;
use dom::screen::Screen;
use dom::storage::Storage;
use dom::testrunner::TestRunner;
use dom::windowproxy::WindowProxy;
use dom_struct::dom_struct;
use euclid::{Point2D, Rect, Size2D};
use fetch;
use ipc_channel::ipc::{self, IpcSender};
use ipc_channel::router::ROUTER;
use js::jsapi::{HandleObject, HandleValue, JSAutoCompartment, JSContext};
use js::jsapi::{JS_GC, JS_GetRuntime};
use js::jsval::UndefinedValue;
use js::rust::Runtime;
use layout_image::fetch_image_for_layout;
use msg::constellation_msg::{FrameType, PipelineId};
use net_traits::{ResourceThreads, ReferrerPolicy};
use net_traits::image_cache::{ImageCache, ImageResponder, ImageResponse};
use net_traits::image_cache::{PendingImageId, PendingImageResponse};
use net_traits::storage_thread::StorageType;
use num_traits::ToPrimitive;
use open;
use profile_traits::mem::ProfilerChan as MemProfilerChan;
use profile_traits::time::ProfilerChan as TimeProfilerChan;
use script_layout_interface::{TrustedNodeAddress, PendingImageState};
use script_layout_interface::message::{Msg, Reflow, ReflowQueryType, ScriptReflow};
use script_layout_interface::reporter::CSSErrorReporter;
use script_layout_interface::rpc::{ContentBoxResponse, ContentBoxesResponse, LayoutRPC};
use script_layout_interface::rpc::{MarginStyleResponse, NodeScrollRootIdResponse};
use script_layout_interface::rpc::{ResolvedStyleResponse, TextIndexResponse};
use script_runtime::{CommonScriptMsg, ScriptChan, ScriptPort, ScriptThreadEventCategory};
use script_thread::{MainThreadScriptChan, MainThreadScriptMsg, Runnable, RunnableWrapper};
use script_thread::{SendableMainThreadScriptChan, ImageCacheMsg, ScriptThread};
use script_traits::{ConstellationControlMsg, LoadData, MozBrowserEvent, UntrustedNodeAddress};
use script_traits::{DocumentState, TimerEvent, TimerEventId};
use script_traits::{ScriptMsg as ConstellationMsg, TimerSchedulerMsg, WindowSizeData, WindowSizeType};
use script_traits::webdriver_msg::{WebDriverJSError, WebDriverJSResult};
use servo_atoms::Atom;
use servo_config::opts;
use servo_config::prefs::PREFS;
use servo_geometry::{f32_rect_to_au_rect, max_rect};
use servo_url::{Host, MutableOrigin, ImmutableOrigin, ServoUrl};
use std::ascii::AsciiExt;
use std::borrow::ToOwned;
use std::cell::Cell;
use std::collections::{HashMap, HashSet};
use std::collections::hash_map::Entry;
use std::default::Default;
use std::env;
use std::fs;
use std::io::{Write, stderr, stdout};
use std::mem;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Sender, channel};
use std::sync::mpsc::TryRecvError::{Disconnected, Empty};
use style::context::ReflowGoal;
use style::error_reporting::ParseErrorReporter;
use style::media_queries;
use style::parser::{PARSING_MODE_DEFAULT, ParserContext as CssParserContext};
use style::properties::PropertyId;
use style::properties::longhands::overflow_x;
use style::selector_parser::PseudoElement;
use style::str::HTML_SPACE_CHARACTERS;
use style::stylesheets::CssRuleType;
use task_source::dom_manipulation::DOMManipulationTaskSource;
use task_source::file_reading::FileReadingTaskSource;
use task_source::history_traversal::HistoryTraversalTaskSource;
use task_source::networking::NetworkingTaskSource;
use task_source::user_interaction::UserInteractionTaskSource;
use time;
use timers::{IsInterval, TimerCallback};
#[cfg(any(target_os = "macos", target_os = "linux", target_os = "windows"))]
use tinyfiledialogs::{self, MessageBoxIcon};
use url::Position;
use webdriver_handlers::jsval_to_webdriver;
use webrender_traits::ClipId;
use webvr_traits::WebVRMsg;

/// Current state of the window object
#[derive(JSTraceable, Copy, Clone, Debug, PartialEq, HeapSizeOf)]
enum WindowState {
    Alive,
    Zombie,     // Pipeline is closed, but the window hasn't been GCed yet.
}

/// Extra information concerning the reason for reflowing.
#[derive(Debug, HeapSizeOf)]
pub enum ReflowReason {
    CachedPageNeededReflow,
    RefreshTick,
    FirstLoad,
    KeyEvent,
    MouseEvent,
    Query,
    Timer,
    Viewport,
    WindowResize,
    DOMContentLoaded,
    DocumentLoaded,
    StylesheetLoaded,
    ImageLoaded,
    RequestAnimationFrame,
    WebFontLoaded,
    FramedContentChanged,
    IFrameLoadEvent,
    MissingExplicitReflow,
    ElementStateChanged,
}

#[dom_struct]
pub struct Window {
    globalscope: GlobalScope,
    #[ignore_heap_size_of = "trait objects are hard"]
    script_chan: MainThreadScriptChan,
    #[ignore_heap_size_of = "task sources are hard"]
    dom_manipulation_task_source: DOMManipulationTaskSource,
    #[ignore_heap_size_of = "task sources are hard"]
    user_interaction_task_source: UserInteractionTaskSource,
    #[ignore_heap_size_of = "task sources are hard"]
    networking_task_source: NetworkingTaskSource,
    #[ignore_heap_size_of = "task sources are hard"]
    history_traversal_task_source: HistoryTraversalTaskSource,
    #[ignore_heap_size_of = "task sources are hard"]
    file_reading_task_source: FileReadingTaskSource,
    navigator: MutNullableJS<Navigator>,
    #[ignore_heap_size_of = "Arc"]
    image_cache: Arc<ImageCache>,
    #[ignore_heap_size_of = "channels are hard"]
    image_cache_chan: Sender<ImageCacheMsg>,
    window_proxy: MutNullableJS<WindowProxy>,
    document: MutNullableJS<Document>,
    history: MutNullableJS<History>,
    performance: MutNullableJS<Performance>,
    navigation_start: u64,
    navigation_start_precise: f64,
    screen: MutNullableJS<Screen>,
    session_storage: MutNullableJS<Storage>,
    local_storage: MutNullableJS<Storage>,
    status: DOMRefCell<DOMString>,

    /// For sending timeline markers. Will be ignored if
    /// no devtools server
    devtools_markers: DOMRefCell<HashSet<TimelineMarkerType>>,
    #[ignore_heap_size_of = "channels are hard"]
    devtools_marker_sender: DOMRefCell<Option<IpcSender<Option<TimelineMarker>>>>,

    /// Pending resize event, if any.
    resize_event: Cell<Option<(WindowSizeData, WindowSizeType)>>,

    /// Parent id associated with this page, if any.
    parent_info: Option<(PipelineId, FrameType)>,

    /// Global static data related to the DOM.
    dom_static: GlobalStaticData,

    /// The JavaScript runtime.
    #[ignore_heap_size_of = "Rc<T> is hard"]
    js_runtime: DOMRefCell<Option<Rc<Runtime>>>,

    /// A handle for communicating messages to the layout thread.
    #[ignore_heap_size_of = "channels are hard"]
    layout_chan: Sender<Msg>,

    /// A handle to perform RPC calls into the layout, quickly.
    #[ignore_heap_size_of = "trait objects are hard"]
    layout_rpc: Box<LayoutRPC + Send + 'static>,

    /// The current size of the window, in pixels.
    window_size: Cell<Option<WindowSizeData>>,

    /// A handle for communicating messages to the bluetooth thread.
    #[ignore_heap_size_of = "channels are hard"]
    bluetooth_thread: IpcSender<BluetoothRequest>,

    bluetooth_extra_permission_data: BluetoothExtraPermissionData,

    /// An enlarged rectangle around the page contents visible in the viewport, used
    /// to prevent creating display list items for content that is far away from the viewport.
    page_clip_rect: Cell<Rect<Au>>,

    /// Flag to suppress reflows. The first reflow will come either with
    /// RefreshTick or with FirstLoad. Until those first reflows, we want to
    /// suppress others like MissingExplicitReflow.
    suppress_reflow: Cell<bool>,

    /// A counter of the number of pending reflows for this window.
    pending_reflow_count: Cell<u32>,

    /// A channel for communicating results of async scripts back to the webdriver server
    #[ignore_heap_size_of = "channels are hard"]
    webdriver_script_chan: DOMRefCell<Option<IpcSender<WebDriverJSResult>>>,

    /// The current state of the window object
    current_state: Cell<WindowState>,

    current_viewport: Cell<Rect<Au>>,

    /// A flag to prevent async events from attempting to interact with this window.
    #[ignore_heap_size_of = "defined in std"]
    ignore_further_async_events: DOMRefCell<Arc<AtomicBool>>,

    error_reporter: CSSErrorReporter,

    /// A list of scroll offsets for each scrollable element.
    scroll_offsets: DOMRefCell<HashMap<UntrustedNodeAddress, Point2D<f32>>>,

    /// All the MediaQueryLists we need to update
    media_query_lists: WeakMediaQueryListVec,

    test_runner: MutNullableJS<TestRunner>,

    /// A handle for communicating messages to the webvr thread, if available.
    #[ignore_heap_size_of = "channels are hard"]
    webvr_thread: Option<IpcSender<WebVRMsg>>,

    /// A map for storing the previous permission state read results.
    permission_state_invocation_results: DOMRefCell<HashMap<String, PermissionState>>,

    /// All of the elements that have an outstanding image request that was
    /// initiated by layout during a reflow. They are stored in the script thread
    /// to ensure that the element can be marked dirty when the image data becomes
    /// available at some point in the future.
    pending_layout_images: DOMRefCell<HashMap<PendingImageId, Vec<JS<Node>>>>,

    /// Directory to store unminified scripts for this window if unminify-js
    /// opt is enabled.
    unminified_js_dir: DOMRefCell<Option<String>>,
}

impl Window {
    #[allow(unsafe_code)]
    pub fn clear_js_runtime_for_script_deallocation(&self) {
        unsafe {
            *self.js_runtime.borrow_for_script_deallocation() = None;
            self.window_proxy.set(None);
            self.current_state.set(WindowState::Zombie);
            self.ignore_further_async_events.borrow().store(true, Ordering::Relaxed);
        }
    }

    pub fn origin(&self) -> &MutableOrigin {
        self.globalscope.origin()
    }

    pub fn get_cx(&self) -> *mut JSContext {
        self.js_runtime.borrow().as_ref().unwrap().cx()
    }

    pub fn dom_manipulation_task_source(&self) -> DOMManipulationTaskSource {
        self.dom_manipulation_task_source.clone()
    }

    pub fn user_interaction_task_source(&self) -> UserInteractionTaskSource {
        self.user_interaction_task_source.clone()
    }

    pub fn networking_task_source(&self) -> NetworkingTaskSource {
        self.networking_task_source.clone()
    }

    pub fn history_traversal_task_source(&self) -> Box<ScriptChan + Send> {
        self.history_traversal_task_source.clone()
    }

    pub fn file_reading_task_source(&self) -> FileReadingTaskSource {
        self.file_reading_task_source.clone()
    }

    pub fn main_thread_script_chan(&self) -> &Sender<MainThreadScriptMsg> {
        &self.script_chan.0
    }

    pub fn parent_info(&self) -> Option<(PipelineId, FrameType)> {
        self.parent_info
    }

    pub fn new_script_pair(&self) -> (Box<ScriptChan + Send>, Box<ScriptPort + Send>) {
        let (tx, rx) = channel();
        (box SendableMainThreadScriptChan(tx), box rx)
    }

    pub fn image_cache(&self) -> Arc<ImageCache> {
        self.image_cache.clone()
    }

    /// This can panic if it is called after the browsing context has been discarded
    pub fn window_proxy(&self) -> Root<WindowProxy> {
        self.window_proxy.get().unwrap()
    }

    pub fn maybe_window_proxy(&self) -> Option<Root<WindowProxy>> {
        self.window_proxy.get()
    }

    pub fn bluetooth_thread(&self) -> IpcSender<BluetoothRequest> {
        self.bluetooth_thread.clone()
    }

    pub fn bluetooth_extra_permission_data(&self) -> &BluetoothExtraPermissionData {
         &self.bluetooth_extra_permission_data
    }

    pub fn css_error_reporter(&self) -> &ParseErrorReporter {
        &self.error_reporter
    }

    /// Sets a new list of scroll offsets.
    ///
    /// This is called when layout gives us new ones and WebRender is in use.
    pub fn set_scroll_offsets(&self, offsets: HashMap<UntrustedNodeAddress, Point2D<f32>>) {
        *self.scroll_offsets.borrow_mut() = offsets
    }

    pub fn current_viewport(&self) -> Rect<Au> {
        self.current_viewport.clone().get()
    }

    pub fn webvr_thread(&self) -> Option<IpcSender<WebVRMsg>> {
        self.webvr_thread.clone()
    }

    pub fn permission_state_invocation_results(&self) -> &DOMRefCell<HashMap<String, PermissionState>> {
        &self.permission_state_invocation_results
    }

    pub fn pending_image_notification(&self, response: PendingImageResponse) {
        //XXXjdm could be more efficient to send the responses to the layout thread,
        //       rather than making the layout thread talk to the image cache to
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
            ImageResponse::MetadataLoaded(_) => {}
            ImageResponse::Loaded(_) |
            ImageResponse::PlaceholderLoaded(_) |
            ImageResponse::None => { nodes.remove(); }
        }
        self.add_pending_reflow();
    }
}

#[cfg(any(target_os = "macos", target_os = "linux", target_os = "windows"))]
fn display_alert_dialog(message: &str) {
    if !opts::get().headless {
        tinyfiledialogs::message_box_ok("Alert!", message, MessageBoxIcon::Warning);
    }
}

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
fn display_alert_dialog(_message: &str) {
    // tinyfiledialogs not supported on Android
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
        Ok(DOMString::from(base64::encode(&octets)))
    }
}

// https://html.spec.whatwg.org/multipage/#atob
pub fn base64_atob(input: DOMString) -> Fallible<DOMString> {
    // "Remove all space characters from input."
    fn is_html_space(c: char) -> bool {
        HTML_SPACE_CHARACTERS.iter().any(|&m| m == c)
    }
    let without_spaces = input.chars()
        .filter(|&c| !is_html_space(c))
        .collect::<String>();
    let mut input = &*without_spaces;

    // "If the length of input divides by 4 leaving no remainder, then:
    //  if input ends with one or two U+003D EQUALS SIGN (=) characters,
    //  remove them from input."
    if input.len() % 4 == 0 {
        if input.ends_with("==") {
            input = &input[..input.len() - 2]
        } else if input.ends_with("=") {
            input = &input[..input.len() - 1]
        }
    }

    // "If the length of input divides by 4 leaving a remainder of 1,
    //  throw an InvalidCharacterError exception and abort these steps."
    if input.len() % 4 == 1 {
        return Err(Error::InvalidCharacter)
    }

    // "If input contains a character that is not in the following list of
    //  characters and character ranges, throw an InvalidCharacterError
    //  exception and abort these steps:
    //
    //  U+002B PLUS SIGN (+)
    //  U+002F SOLIDUS (/)
    //  Alphanumeric ASCII characters"
    if input.chars().any(|c| c != '+' && c != '/' && !c.is_alphanumeric()) {
        return Err(Error::InvalidCharacter)
    }

    match base64::decode(&input) {
        Ok(data) => Ok(DOMString::from(data.iter().map(|&b| b as char).collect::<String>())),
        Err(..) => Err(Error::InvalidCharacter)
    }
}

impl WindowMethods for Window {
    // https://html.spec.whatwg.org/multipage/#dom-alert
    fn Alert_(&self) {
        self.Alert(DOMString::new());
    }

    // https://html.spec.whatwg.org/multipage/#dom-alert
    fn Alert(&self, s: DOMString) {
        // Right now, just print to the console
        // Ensure that stderr doesn't trample through the alert() we use to
        // communicate test results (see executorservo.py in wptrunner).
        {
            let stderr = stderr();
            let mut stderr = stderr.lock();
            let stdout = stdout();
            let mut stdout = stdout.lock();
            writeln!(&mut stdout, "ALERT: {}", s).unwrap();
            stdout.flush().unwrap();
            stderr.flush().unwrap();
        }

        let (sender, receiver) = ipc::channel().unwrap();
        let global_scope = self.upcast::<GlobalScope>();
        global_scope
            .constellation_chan()
            .send(ConstellationMsg::Alert(global_scope.pipeline_id(), s.to_string(), sender))
            .unwrap();

        let should_display_alert_dialog = receiver.recv().unwrap();
        if should_display_alert_dialog {
            display_alert_dialog(&s);
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-window-close
    fn Close(&self) {
        self.main_thread_script_chan()
            .send(MainThreadScriptMsg::ExitWindow(self.upcast::<GlobalScope>().pipeline_id()))
            .unwrap();
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-2
    fn Document(&self) -> Root<Document> {
        self.document.get().expect("Document accessed before initialization.")
    }

    // https://html.spec.whatwg.org/multipage/#dom-history
    fn History(&self) -> Root<History> {
        self.history.or_init(|| History::new(self))
    }

    // https://html.spec.whatwg.org/multipage/#dom-location
    fn Location(&self) -> Root<Location> {
        self.Document().GetLocation().unwrap()
    }

    // https://html.spec.whatwg.org/multipage/#dom-sessionstorage
    fn SessionStorage(&self) -> Root<Storage> {
        self.session_storage.or_init(|| Storage::new(self, StorageType::Session))
    }

    // https://html.spec.whatwg.org/multipage/#dom-localstorage
    fn LocalStorage(&self) -> Root<Storage> {
        self.local_storage.or_init(|| Storage::new(self, StorageType::Local))
    }

    // https://dvcs.w3.org/hg/webcrypto-api/raw-file/tip/spec/Overview.html#dfn-GlobalCrypto
    fn Crypto(&self) -> Root<Crypto> {
        self.upcast::<GlobalScope>().crypto()
    }

    // https://html.spec.whatwg.org/multipage/#dom-frameelement
    fn GetFrameElement(&self) -> Option<Root<Element>> {
        // Steps 1-3.
        let window_proxy = match self.window_proxy.get() {
            None => return None,
            Some(window_proxy) => window_proxy,
        };
        // Step 4-5.
        let container = match window_proxy.frame_element() {
            None => return None,
            Some(container) => container,
        };
        // Step 6.
        let container_doc = document_from_node(container);
        let current_doc = GlobalScope::current().as_window().Document();
        if !current_doc.origin().same_origin_domain(container_doc.origin()) {
            return None;
        }
        // Step 7.
        Some(Root::from_ref(container))
    }

    // https://html.spec.whatwg.org/multipage/#dom-navigator
    fn Navigator(&self) -> Root<Navigator> {
        self.navigator.or_init(|| Navigator::new(self))
    }

    #[allow(unsafe_code)]
    // https://html.spec.whatwg.org/multipage/#dom-windowtimers-settimeout
    unsafe fn SetTimeout(&self, _cx: *mut JSContext, callback: Rc<Function>, timeout: i32,
                         args: Vec<HandleValue>) -> i32 {
        self.upcast::<GlobalScope>().set_timeout_or_interval(
            TimerCallback::FunctionTimerCallback(callback),
            args,
            timeout,
            IsInterval::NonInterval)
    }

    #[allow(unsafe_code)]
    // https://html.spec.whatwg.org/multipage/#dom-windowtimers-settimeout
    unsafe fn SetTimeout_(&self, _cx: *mut JSContext, callback: DOMString,
                          timeout: i32, args: Vec<HandleValue>) -> i32 {
        self.upcast::<GlobalScope>().set_timeout_or_interval(
            TimerCallback::StringTimerCallback(callback),
            args,
            timeout,
            IsInterval::NonInterval)
    }

    // https://html.spec.whatwg.org/multipage/#dom-windowtimers-cleartimeout
    fn ClearTimeout(&self, handle: i32) {
        self.upcast::<GlobalScope>().clear_timeout_or_interval(handle);
    }

    #[allow(unsafe_code)]
    // https://html.spec.whatwg.org/multipage/#dom-windowtimers-setinterval
    unsafe fn SetInterval(&self, _cx: *mut JSContext, callback: Rc<Function>,
                          timeout: i32, args: Vec<HandleValue>) -> i32 {
        self.upcast::<GlobalScope>().set_timeout_or_interval(
            TimerCallback::FunctionTimerCallback(callback),
            args,
            timeout,
            IsInterval::Interval)
    }

    #[allow(unsafe_code)]
    // https://html.spec.whatwg.org/multipage/#dom-windowtimers-setinterval
    unsafe fn SetInterval_(&self, _cx: *mut JSContext, callback: DOMString,
                           timeout: i32, args: Vec<HandleValue>) -> i32 {
        self.upcast::<GlobalScope>().set_timeout_or_interval(
            TimerCallback::StringTimerCallback(callback),
            args,
            timeout,
            IsInterval::Interval)
    }

    // https://html.spec.whatwg.org/multipage/#dom-windowtimers-clearinterval
    fn ClearInterval(&self, handle: i32) {
        self.ClearTimeout(handle);
    }

    // https://html.spec.whatwg.org/multipage/#dom-window
    fn Window(&self) -> Root<WindowProxy> {
        self.window_proxy()
    }

    // https://html.spec.whatwg.org/multipage/#dom-self
    fn Self_(&self) -> Root<WindowProxy> {
        self.window_proxy()
    }

    // https://html.spec.whatwg.org/multipage/#dom-frames
    fn Frames(&self) -> Root<WindowProxy> {
        self.window_proxy()
    }

    // https://html.spec.whatwg.org/multipage/#dom-parent
    fn GetParent(&self) -> Option<Root<WindowProxy>> {
        // Steps 1-3.
        let window_proxy = match self.maybe_window_proxy() {
            Some(window_proxy) => window_proxy,
            None => return None,
        };
        if window_proxy.is_browsing_context_discarded() {
            return None;
        }
        // Step 4.
        if let Some(parent) = window_proxy.parent() {
            return Some(Root::from_ref(parent));
        }
        // Step 5.
        Some(window_proxy)
    }

    // https://html.spec.whatwg.org/multipage/#dom-top
    fn GetTop(&self) -> Option<Root<WindowProxy>> {
        // Steps 1-3.
        let window_proxy = match self.maybe_window_proxy() {
            Some(window_proxy) => window_proxy,
            None => return None,
        };
        if window_proxy.is_browsing_context_discarded() {
            return None;
        }
        // Steps 4-5.
        Some(Root::from_ref(window_proxy.top()))
    }

    // https://dvcs.w3.org/hg/webperf/raw-file/tip/specs/
    // NavigationTiming/Overview.html#sec-window.performance-attribute
    fn Performance(&self) -> Root<Performance> {
        self.performance.or_init(|| {
            Performance::new(self, self.navigation_start,
                             self.navigation_start_precise)
        })
    }

    // https://html.spec.whatwg.org/multipage/#globaleventhandlers
    global_event_handlers!();

    // https://html.spec.whatwg.org/multipage/#windoweventhandlers
    window_event_handlers!();

    // https://developer.mozilla.org/en-US/docs/Web/API/Window/screen
    fn Screen(&self) -> Root<Screen> {
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

    /// https://html.spec.whatwg.org/multipage/#dom-window-requestanimationframe
    fn RequestAnimationFrame(&self, callback: Rc<FrameRequestCallback>) -> u32 {
        self.Document()
            .request_animation_frame(AnimationFrameCallback::FrameRequestCallback { callback })
    }

    /// https://html.spec.whatwg.org/multipage/#dom-window-cancelanimationframe
    fn CancelAnimationFrame(&self, ident: u32) {
        let doc = self.Document();
        doc.cancel_animation_frame(ident);
    }

    #[allow(unsafe_code)]
    // https://html.spec.whatwg.org/multipage/#dom-window-postmessage
    unsafe fn PostMessage(&self,
                   cx: *mut JSContext,
                   message: HandleValue,
                   origin: DOMString)
                   -> ErrorResult {
        // Step 3-5.
        let origin = match &origin[..] {
            "*" => None,
            "/" => {
                // TODO(#12715): Should be the origin of the incumbent settings
                //               object, not self's.
                Some(self.Document().origin().immutable().clone())
            },
            url => match ServoUrl::parse(&url) {
                Ok(url) => Some(url.origin().clone()),
                Err(_) => return Err(Error::Syntax),
            }
        };

        // Step 1-2, 6-8.
        // TODO(#12717): Should implement the `transfer` argument.
        let data = try!(StructuredCloneData::write(cx, message));

        // Step 9.
        self.post_message(origin, data);
        Ok(())
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
            JS_GC(JS_GetRuntime(self.get_cx()));
        }
    }

    #[allow(unsafe_code)]
    fn Trap(&self) {
        unsafe { ::std::intrinsics::breakpoint() }
    }

    #[allow(unsafe_code)]
    unsafe fn WebdriverCallback(&self, cx: *mut JSContext, val: HandleValue) {
        let rv = jsval_to_webdriver(cx, val);
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
    fn GetComputedStyle(&self,
                        element: &Element,
                        pseudo: Option<DOMString>) -> Root<CSSStyleDeclaration> {
        // Steps 1-4.
        let pseudo = match pseudo.map(|mut s| { s.make_ascii_lowercase(); s }) {
            Some(ref pseudo) if pseudo == ":before" || pseudo == "::before" =>
                Some(PseudoElement::Before),
            Some(ref pseudo) if pseudo == ":after" || pseudo == "::after" =>
                Some(PseudoElement::After),
            _ => None
        };

        // Step 5.
        CSSStyleDeclaration::new(self,
                                 CSSStyleOwner::Element(JS::from_ref(element)),
                                 pseudo,
                                 CSSModificationAccess::Readonly)
    }

    // https://drafts.csswg.org/cssom-view/#dom-window-innerheight
    //TODO Include Scrollbar
    fn InnerHeight(&self) -> i32 {
        self.window_size.get()
                        .and_then(|e| e.initial_viewport.height.to_i32())
                        .unwrap_or(0)
    }

    // https://drafts.csswg.org/cssom-view/#dom-window-innerwidth
    //TODO Include Scrollbar
    fn InnerWidth(&self) -> i32 {
        self.window_size.get()
                        .and_then(|e| e.initial_viewport.width.to_i32())
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
    fn ScrollBy(&self, options: &ScrollToOptions)  {
        // Step 1
        let x = options.left.unwrap_or(0.0f64);
        let y = options.top.unwrap_or(0.0f64);
        self.ScrollBy_(x, y);
        self.scroll(x, y, options.parent.behavior);
    }

    // https://drafts.csswg.org/cssom-view/#dom-window-scrollby
    fn ScrollBy_(&self, x: f64, y: f64)  {
        // Step 3
        let left = x + self.ScrollX() as f64;
        // Step 4
        let top =  y + self.ScrollY() as f64;

        // Step 5
        self.scroll(left, top, ScrollBehavior::Auto);
    }

    // https://drafts.csswg.org/cssom-view/#dom-window-resizeto
    fn ResizeTo(&self, x: i32, y: i32) {
        // Step 1
        //TODO determine if this operation is allowed
        let size = Size2D::new(x.to_u32().unwrap_or(1), y.to_u32().unwrap_or(1));
        self.upcast::<GlobalScope>()
            .constellation_chan()
            .send(ConstellationMsg::ResizeTo(size))
            .unwrap()
    }

    // https://drafts.csswg.org/cssom-view/#dom-window-resizeby
    fn ResizeBy(&self, x: i32, y: i32) {
        let (size, _) = self.client_window();
        // Step 1
        self.ResizeTo(x + size.width.to_i32().unwrap_or(1), y + size.height.to_i32().unwrap_or(1))
    }

    // https://drafts.csswg.org/cssom-view/#dom-window-moveto
    fn MoveTo(&self, x: i32, y: i32) {
        // Step 1
        //TODO determine if this operation is allowed
        let point = Point2D::new(x, y);
        self.upcast::<GlobalScope>()
            .constellation_chan()
            .send(ConstellationMsg::MoveTo(point))
            .unwrap()
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
        let dpr = self.window_size.get().map_or(1.0f32, |data| data.device_pixel_ratio.get());
        Finite::wrap(dpr as f64)
    }

    // https://html.spec.whatwg.org/multipage/#dom-window-status
    fn Status(&self) -> DOMString {
        self.status.borrow().clone()
    }

    // https://html.spec.whatwg.org/multipage/#dom-window-status
    fn SetStatus(&self, status: DOMString) {
        *self.status.borrow_mut() = status
    }

    // check-tidy: no specs after this line
    fn OpenURLInDefaultBrowser(&self, href: DOMString) -> ErrorResult {
        let url = try!(ServoUrl::parse(&href).map_err(|e| {
            Error::Type(format!("Couldn't parse URL: {}", e))
        }));
        match open::that(url.as_str()) {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::Type(format!("Couldn't open URL: {}", e))),
        }
    }

    // https://drafts.csswg.org/cssom-view/#dom-window-matchmedia
    fn MatchMedia(&self, query: DOMString) -> Root<MediaQueryList> {
        let mut parser = Parser::new(&query);
        let url = self.get_url();
        let quirks_mode = self.Document().quirks_mode();
        let context = CssParserContext::new_for_cssom(&url, self.css_error_reporter(), Some(CssRuleType::Media),
                                                      PARSING_MODE_DEFAULT,
                                                      quirks_mode);
        let media_query_list = media_queries::parse_media_query_list(&context, &mut parser);
        let document = self.Document();
        let mql = MediaQueryList::new(&document, media_query_list);
        self.media_query_lists.push(&*mql);
        mql
    }

    #[allow(unrooted_must_root)]
    // https://fetch.spec.whatwg.org/#fetch-method
    fn Fetch(&self, input: RequestOrUSVString, init: RootedTraceableBox<RequestInit>) -> Rc<Promise> {
        fetch::Fetch(&self.upcast(), input, init)
    }

    fn TestRunner(&self) -> Root<TestRunner> {
        self.test_runner.or_init(|| TestRunner::new(self.upcast()))
    }
}

impl Window {
    pub fn get_runnable_wrapper(&self) -> RunnableWrapper {
        RunnableWrapper {
            cancelled: Some(self.ignore_further_async_events.borrow().clone()),
        }
    }

    /// Cancels all the tasks associated with that window.
    ///
    /// This sets the current `ignore_further_async_events` sentinel value to
    /// `true` and replaces it with a brand new one for future tasks.
    pub fn cancel_all_tasks(&self) {
        let cancelled = mem::replace(&mut *self.ignore_further_async_events.borrow_mut(), Default::default());
        cancelled.store(true, Ordering::Relaxed);
    }

    pub fn clear_js_runtime(&self) {
        // We tear down the active document, which causes all the attached
        // nodes to dispose of their layout data. This messages the layout
        // thread, informing it that it can safely free the memory.
        self.Document().upcast::<Node>().teardown();

        // The above code may not catch all DOM objects (e.g. DOM
        // objects removed from the tree that haven't been collected
        // yet). There should not be any such DOM nodes with layout
        // data, but if there are, then when they are dropped, they
        // will attempt to send a message to the closed layout thread.
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
        self.window_proxy.set(None);
        self.ignore_further_async_events.borrow().store(true, Ordering::SeqCst);
    }

    /// https://drafts.csswg.org/cssom-view/#dom-window-scroll
    pub fn scroll(&self, x_: f64, y_: f64, behavior: ScrollBehavior) {
        // Step 3
        let xfinite = if x_.is_finite() { x_ } else { 0.0f64 };
        let yfinite = if y_.is_finite() { y_ } else { 0.0f64 };

        // Step 4
        if self.window_size.get().is_none() {
            return;
        }

        // Step 5
        //TODO remove scrollbar width
        let width = self.InnerWidth() as f64;
        // Step 6
        //TODO remove scrollbar height
        let height = self.InnerHeight() as f64;

        // Step 7 & 8
        //TODO use overflow direction
        let body = self.Document().GetBody();
        let (x, y) = match body {
            Some(e) => {
                let content_size = e.upcast::<Node>().bounding_content_box_or_zero();
                let content_height = content_size.size.height.to_f64_px();
                let content_width = content_size.size.width.to_f64_px();
                (xfinite.min(content_width - width).max(0.0f64),
                 yfinite.min(content_height - height).max(0.0f64))
            },
            None => {
                (xfinite.max(0.0f64), yfinite.max(0.0f64))
            }
        };

        // Step 10
        //TODO handling ongoing smooth scrolling
        if x == self.ScrollX() as f64 && y == self.ScrollY() as f64 {
            return;
        }

        //TODO Step 11
        //let document = self.Document();
        // Step 12
        let global_scope = self.upcast::<GlobalScope>();
        self.perform_a_scroll(x.to_f32().unwrap_or(0.0f32),
                              y.to_f32().unwrap_or(0.0f32),
                              global_scope.pipeline_id().root_scroll_node(),
                              behavior,
                              None);
    }

    /// https://drafts.csswg.org/cssom-view/#perform-a-scroll
    pub fn perform_a_scroll(&self,
                            x: f32,
                            y: f32,
                            scroll_root_id: ClipId,
                            behavior: ScrollBehavior,
                            element: Option<&Element>) {
        //TODO Step 1
        let point = Point2D::new(x, y);
        let smooth = match behavior {
            ScrollBehavior::Auto => {
                element.map_or(false, |_element| {
                    // TODO check computed scroll-behaviour CSS property
                    true
                })
            }
            ScrollBehavior::Instant => false,
            ScrollBehavior::Smooth => true
        };

        // TODO (farodin91): Raise an event to stop the current_viewport
        self.update_viewport_for_scroll(x, y);

        let global_scope = self.upcast::<GlobalScope>();
        let message = ConstellationMsg::ScrollFragmentPoint(scroll_root_id, point, smooth);
        global_scope.constellation_chan().send(message).unwrap();
    }

    pub fn update_viewport_for_scroll(&self, x: f32, y: f32) {
        let size = self.current_viewport.get().size;
        let new_viewport = Rect::new(Point2D::new(Au::from_f32_px(x), Au::from_f32_px(y)), size);
        self.current_viewport.set(new_viewport)
    }

    pub fn client_window(&self) -> (Size2D<u32>, Point2D<i32>) {
        let (send, recv) = ipc::channel::<(Size2D<u32>, Point2D<i32>)>().unwrap();
        self.upcast::<GlobalScope>()
            .constellation_chan()
            .send(ConstellationMsg::GetClientWindow(send))
            .unwrap();
        recv.recv().unwrap_or((Size2D::zero(), Point2D::zero()))
    }

    /// Advances the layout animation clock by `delta` milliseconds, and then
    /// forces a reflow if `tick` is true.
    pub fn advance_animation_clock(&self, delta: i32, tick: bool) {
        self.layout_chan.send(Msg::AdvanceClockMs(delta, tick)).unwrap();
    }

    /// Reflows the page unconditionally if possible and not suppressed. This
    /// method will wait for the layout thread to complete (but see the `TODO`
    /// below). If there is no window size yet, the page is presumed invisible
    /// and no reflow is performed. If reflow is suppressed, no reflow will be
    /// performed for ForDisplay goals.
    ///
    /// TODO(pcwalton): Only wait for style recalc, since we have
    /// off-main-thread layout.
    ///
    /// Returns true if layout actually happened, false otherwise.
    #[allow(unsafe_code)]
    pub fn force_reflow(&self,
                        goal: ReflowGoal,
                        query_type: ReflowQueryType,
                        reason: ReflowReason) -> bool {
        // Check if we need to unsuppress reflow. Note that this needs to be
        // *before* any early bailouts, or reflow might never be unsuppresed!
        match reason {
            ReflowReason::FirstLoad |
            ReflowReason::RefreshTick => self.suppress_reflow.set(false),
            _ => (),
        }

        // If there is no window size, we have nothing to do.
        let window_size = match self.window_size.get() {
            Some(window_size) => window_size,
            None => return false,
        };

        let for_display = query_type == ReflowQueryType::NoQuery;
        if for_display && self.suppress_reflow.get() {
            debug!("Suppressing reflow pipeline {} for goal {:?} reason {:?} before FirstLoad or RefreshTick",
                   self.upcast::<GlobalScope>().pipeline_id(), goal, reason);
            return false;
        }

        debug!("script: performing reflow for goal {:?} reason {:?}", goal, reason);

        let marker = if self.need_emit_timeline_marker(TimelineMarkerType::Reflow) {
            Some(TimelineMarker::start("Reflow".to_owned()))
        } else {
            None
        };

        // Layout will let us know when it's done.
        let (join_chan, join_port) = channel();

        // On debug mode, print the reflow event information.
        if opts::get().relayout_event {
            debug_reflow_events(self.upcast::<GlobalScope>().pipeline_id(), &goal, &query_type, &reason);
        }

        let document = self.Document();
        let stylesheets_changed = document.get_and_reset_stylesheets_changed_since_reflow();

        // Send new document and relevant styles to layout.
        let reflow = ScriptReflow {
            reflow_info: Reflow {
                goal: goal,
                page_clip_rect: self.page_clip_rect.get(),
            },
            document: self.Document().upcast::<Node>().to_trusted_node_address(),
            document_stylesheets: document.stylesheets(),
            stylesheets_changed: stylesheets_changed,
            window_size: window_size,
            script_join_chan: join_chan,
            query_type: query_type,
            dom_count: self.Document().dom_count(),
        };

        self.layout_chan.send(Msg::Reflow(reflow)).unwrap();

        debug!("script: layout forked");

        let complete = match join_port.try_recv() {
            Err(Empty) => {
                info!("script: waiting on layout");
                join_port.recv().unwrap()
            }
            Ok(reflow_complete) => reflow_complete,
            Err(Disconnected) => {
                panic!("Layout thread failed while script was waiting for a result.");
            }
        };

        debug!("script: layout joined");

        // Pending reflows require display, so only reset the pending reflow count if this reflow
        // was to be displayed.
        if goal == ReflowGoal::ForDisplay {
            self.pending_reflow_count.set(0);
        }

        if let Some(marker) = marker {
            self.emit_timeline_marker(marker.end());
        }

        for image in complete.pending_images {
            let id = image.id;
            let js_runtime = self.js_runtime.borrow();
            let js_runtime = js_runtime.as_ref().unwrap();
            let node = unsafe { from_untrusted_node_address(js_runtime.rt(), image.node) };

            if let PendingImageState::Unrequested(ref url) = image.state {
                fetch_image_for_layout(url.clone(), &*node, id, self.image_cache.clone());
            }

            let mut images = self.pending_layout_images.borrow_mut();
            let nodes = images.entry(id).or_insert(vec![]);
            if nodes.iter().find(|n| &***n as *const _ == &*node as *const _).is_none() {
                let (responder, responder_listener) = ipc::channel().unwrap();
                let pipeline = self.upcast::<GlobalScope>().pipeline_id();
                let image_cache_chan = self.image_cache_chan.clone();
                ROUTER.add_route(responder_listener.to_opaque(), box move |message| {
                    let _ = image_cache_chan.send((pipeline, message.to().unwrap()));
                });
                self.image_cache.add_listener(id, ImageResponder::new(responder, id));
                nodes.push(JS::from_ref(&*node));
            }
        }

        unsafe {
            ScriptThread::note_newly_transitioning_nodes(complete.newly_transitioning_nodes);
        }

        true
    }

    /// Reflows the page if it's possible to do so and the page is dirty. This
    /// method will wait for the layout thread to complete (but see the `TODO`
    /// below). If there is no window size yet, the page is presumed invisible
    /// and no reflow is performed.
    ///
    /// TODO(pcwalton): Only wait for style recalc, since we have
    /// off-main-thread layout.
    ///
    /// Returns true if layout actually happened, false otherwise.
    /// This return value is useful for script queries, that wait for a lock
    /// that layout might hold if the first layout hasn't happened yet (which
    /// may happen in the only case a query reflow may bail out, that is, if the
    /// viewport size is not present). See #11223 for an example of that.
    pub fn reflow(&self,
                  goal: ReflowGoal,
                  query_type: ReflowQueryType,
                  reason: ReflowReason) -> bool {
        let for_display = query_type == ReflowQueryType::NoQuery;

        let mut issued_reflow = false;
        if !for_display || self.Document().needs_reflow() {
            issued_reflow = self.force_reflow(goal, query_type, reason);

            // If window_size is `None`, we don't reflow, so the document stays
            // dirty. Otherwise, we shouldn't need a reflow immediately after a
            // reflow, except if we're waiting for a deferred paint.
            assert!(!self.Document().needs_reflow() ||
                    (!for_display && self.Document().needs_paint()) ||
                    self.window_size.get().is_none() ||
                    self.suppress_reflow.get());
        } else {
            debug!("Document doesn't need reflow - skipping it (reason {:?})", reason);
        }

        // If writing a screenshot, check if the script has reached a state
        // where it's safe to write the image. This means that:
        // 1) The reflow is for display (otherwise it could be a query)
        // 2) The html element doesn't contain the 'reftest-wait' class
        // 3) The load event has fired.
        // When all these conditions are met, notify the constellation
        // that this pipeline is ready to write the image (from the script thread
        // perspective at least).
        if (opts::get().output_file.is_some() ||
            opts::get().exit_after_load ||
            opts::get().webdriver_port.is_some()) && for_display {
            let document = self.Document();

            // Checks if the html element has reftest-wait attribute present.
            // See http://testthewebforward.org/docs/reftests.html
            let html_element = document.GetDocumentElement();
            let reftest_wait = html_element.map_or(false, |elem| {
                elem.has_class(&Atom::from("reftest-wait"))
            });

            let ready_state = document.ReadyState();

            let pending_images = self.pending_layout_images.borrow().is_empty();
            if ready_state == DocumentReadyState::Complete && !reftest_wait && pending_images {
                let global_scope = self.upcast::<GlobalScope>();
                let event = ConstellationMsg::SetDocumentState(global_scope.pipeline_id(), DocumentState::Idle);
                global_scope.constellation_chan().send(event).unwrap();
            }
        }

        issued_reflow
    }

    pub fn layout(&self) -> &LayoutRPC {
        &*self.layout_rpc
    }

    pub fn content_box_query(&self, content_box_request: TrustedNodeAddress) -> Option<Rect<Au>> {
        if !self.reflow(ReflowGoal::ForScriptQuery,
                        ReflowQueryType::ContentBoxQuery(content_box_request),
                        ReflowReason::Query) {
            return None;
        }
        let ContentBoxResponse(rect) = self.layout_rpc.content_box();
        rect
    }

    pub fn content_boxes_query(&self, content_boxes_request: TrustedNodeAddress) -> Vec<Rect<Au>> {
        if !self.reflow(ReflowGoal::ForScriptQuery,
                        ReflowQueryType::ContentBoxesQuery(content_boxes_request),
                        ReflowReason::Query) {
            return vec![];
        }
        let ContentBoxesResponse(rects) = self.layout_rpc.content_boxes();
        rects
    }

    pub fn client_rect_query(&self, node_geometry_request: TrustedNodeAddress) -> Rect<i32> {
        if !self.reflow(ReflowGoal::ForScriptQuery,
                        ReflowQueryType::NodeGeometryQuery(node_geometry_request),
                        ReflowReason::Query) {
            return Rect::zero();
        }
        self.layout_rpc.node_geometry().client_rect
    }

    pub fn hit_test_query(&self,
                          client_point: Point2D<f32>,
                          update_cursor: bool)
                          -> Option<UntrustedNodeAddress> {
        let translated_point =
            Point2D::new(client_point.x + self.PageXOffset() as f32,
                         client_point.y + self.PageYOffset() as f32);

        if !self.reflow(ReflowGoal::ForScriptQuery,
                        ReflowQueryType::HitTestQuery(translated_point,
                                                      client_point,
                                                      update_cursor),
                        ReflowReason::Query) {
            return None
        }

        self.layout_rpc.hit_test().node_address
    }

    pub fn scroll_area_query(&self, node: TrustedNodeAddress) -> Rect<i32> {
        if !self.reflow(ReflowGoal::ForScriptQuery,
                        ReflowQueryType::NodeScrollGeometryQuery(node),
                        ReflowReason::Query) {
            return Rect::zero();
        }
        self.layout_rpc.node_scroll_area().client_rect
    }

    pub fn overflow_query(&self,
                          node: TrustedNodeAddress) -> Point2D<overflow_x::computed_value::T> {
        // NB: This is only called if the document is fully active, and the only
        // reason to bail out from a query is if there's no viewport, so this
        // *must* issue a reflow.
        assert!(self.reflow(ReflowGoal::ForScriptQuery,
                            ReflowQueryType::NodeOverflowQuery(node),
                            ReflowReason::Query));

        self.layout_rpc.node_overflow().0.unwrap()
    }

    pub fn scroll_offset_query(&self, node: &Node) -> Point2D<f32> {
        let mut node = Root::from_ref(node);
        loop {
            if let Some(scroll_offset) = self.scroll_offsets
                                             .borrow()
                                             .get(&node.to_untrusted_node_address()) {
                return *scroll_offset
            }
            node = match node.GetParentNode() {
                Some(node) => node,
                None => break,
            }
        }
        let offset = self.current_viewport.get().origin;
        Point2D::new(offset.x.to_f32_px(), offset.y.to_f32_px())
    }

    // https://drafts.csswg.org/cssom-view/#dom-element-scroll
    pub fn scroll_node(&self,
                       node: TrustedNodeAddress,
                       x_: f64,
                       y_: f64,
                       behavior: ScrollBehavior) {
        if !self.reflow(ReflowGoal::ForScriptQuery,
                        ReflowQueryType::NodeScrollRootIdQuery(node),
                        ReflowReason::Query) {
            return;
        }
        let NodeScrollRootIdResponse(scroll_root_id) = self.layout_rpc.node_scroll_root_id();

        // Step 12
        self.perform_a_scroll(x_.to_f32().unwrap_or(0.0f32),
                              y_.to_f32().unwrap_or(0.0f32),
                              scroll_root_id,
                              behavior,
                              None);
    }

    pub fn resolved_style_query(&self,
                                element: TrustedNodeAddress,
                                pseudo: Option<PseudoElement>,
                                property: PropertyId) -> DOMString {
        if !self.reflow(ReflowGoal::ForScriptQuery,
                        ReflowQueryType::ResolvedStyleQuery(element, pseudo, property),
                        ReflowReason::Query) {
            return DOMString::new();
        }
        let ResolvedStyleResponse(resolved) = self.layout_rpc.resolved_style();
        DOMString::from(resolved)
    }

    #[allow(unsafe_code)]
    pub fn offset_parent_query(&self, node: TrustedNodeAddress) -> (Option<Root<Element>>, Rect<Au>) {
        if !self.reflow(ReflowGoal::ForScriptQuery,
                        ReflowQueryType::OffsetParentQuery(node),
                        ReflowReason::Query) {
            return (None, Rect::zero());
        }

        let response = self.layout_rpc.offset_parent();
        let js_runtime = self.js_runtime.borrow();
        let js_runtime = js_runtime.as_ref().unwrap();
        let element = response.node_address.and_then(|parent_node_address| {
            let node = unsafe { from_untrusted_node_address(js_runtime.rt(), parent_node_address) };
            Root::downcast(node)
        });
        (element, response.rect)
    }

    pub fn margin_style_query(&self, node: TrustedNodeAddress) -> MarginStyleResponse {
        if !self.reflow(ReflowGoal::ForScriptQuery,
                        ReflowQueryType::MarginStyleQuery(node),
                        ReflowReason::Query) {
            return MarginStyleResponse::empty();
        }
        self.layout_rpc.margin_style()
    }

    pub fn text_index_query(&self, node: TrustedNodeAddress, mouse_x: i32, mouse_y: i32) -> TextIndexResponse {
        if !self.reflow(ReflowGoal::ForScriptQuery,
                        ReflowQueryType::TextIndexQuery(node, mouse_x, mouse_y),
                        ReflowReason::Query) {
            return TextIndexResponse(None);
        }
        self.layout_rpc.text_index()
    }

    #[allow(unsafe_code)]
    pub fn init_window_proxy(&self, window_proxy: &WindowProxy) {
        assert!(self.window_proxy.get().is_none());
        self.window_proxy.set(Some(&window_proxy));
    }

    #[allow(unsafe_code)]
    pub fn init_document(&self, document: &Document) {
        assert!(self.document.get().is_none());
        assert!(document.window() == self);
        self.document.set(Some(&document));
        if !opts::get().unminify_js {
            return;
        }
        // Create a folder for the document host to store unminified scripts.
        if let Some(&Host::Domain(ref host)) = document.url().origin().host() {
            let mut path = env::current_dir().unwrap();
            path.push("unminified-js");
            path.push(host);
            let _ = fs::remove_dir_all(&path);
            match fs::create_dir_all(&path) {
                Ok(_) => {
                    *self.unminified_js_dir.borrow_mut() = Some(path.into_os_string().into_string().unwrap());
                    debug!("Created folder for {:?} unminified scripts {:?}", host, self.unminified_js_dir.borrow());
                },
                Err(_) => warn!("Could not create unminified dir for {:?}", host),
            }
        }
    }

    /// Commence a new URL load which will either replace this window or scroll to a fragment.
    pub fn load_url(&self, url: ServoUrl, replace: bool, force_reload: bool,
                    referrer_policy: Option<ReferrerPolicy>) {
        let doc = self.Document();
        let referrer_policy = referrer_policy.or(doc.get_referrer_policy());

        // https://html.spec.whatwg.org/multipage/#navigating-across-documents
        if !force_reload && url.as_url()[..Position::AfterQuery] ==
            doc.url().as_url()[..Position::AfterQuery] {
                // Step 5
                if let Some(fragment) = url.fragment() {
                    doc.check_and_scroll_fragment(fragment);
                    doc.set_url(url.clone());
                    return
                }
        }

        let pipeline_id = self.upcast::<GlobalScope>().pipeline_id();
        self.main_thread_script_chan().send(
            MainThreadScriptMsg::Navigate(pipeline_id,
                LoadData::new(url, Some(pipeline_id), referrer_policy, Some(doc.url())), replace)).unwrap();
    }

    pub fn handle_fire_timer(&self, timer_id: TimerEventId) {
        self.upcast::<GlobalScope>().fire_timer(timer_id);
        self.reflow(ReflowGoal::ForDisplay,
                    ReflowQueryType::NoQuery,
                    ReflowReason::Timer);
    }

    pub fn set_window_size(&self, size: WindowSizeData) {
        self.window_size.set(Some(size));
    }

    pub fn window_size(&self) -> Option<WindowSizeData> {
        self.window_size.get()
    }

    pub fn get_url(&self) -> ServoUrl {
        self.Document().url()
    }

    pub fn layout_chan(&self) -> &Sender<Msg> {
        &self.layout_chan
    }

    pub fn windowproxy_handler(&self) -> WindowProxyHandler {
        WindowProxyHandler(self.dom_static.windowproxy_handler.0)
    }

    pub fn get_pending_reflow_count(&self) -> u32 {
        self.pending_reflow_count.get()
    }

    pub fn add_pending_reflow(&self) {
        self.pending_reflow_count.set(self.pending_reflow_count.get() + 1);
    }

    pub fn set_resize_event(&self, event: WindowSizeData, event_type: WindowSizeType) {
        self.resize_event.set(Some((event, event_type)));
    }

    pub fn steal_resize_event(&self) -> Option<(WindowSizeData, WindowSizeType)> {
        let event = self.resize_event.get();
        self.resize_event.set(None);
        event
    }

    pub fn set_page_clip_rect_with_new_viewport(&self, viewport: Rect<f32>) -> bool {
        let rect = f32_rect_to_au_rect(viewport.clone());
        self.current_viewport.set(rect);
        // We use a clipping rectangle that is five times the size of the of the viewport,
        // so that we don't collect display list items for areas too far outside the viewport,
        // but also don't trigger reflows every time the viewport changes.
        static VIEWPORT_EXPANSION: f32 = 2.0; // 2 lengths on each side plus original length is 5 total.
        let proposed_clip_rect = f32_rect_to_au_rect(
            viewport.inflate(viewport.size.width * VIEWPORT_EXPANSION,
            viewport.size.height * VIEWPORT_EXPANSION));
        let clip_rect = self.page_clip_rect.get();
        if proposed_clip_rect == clip_rect {
            return false;
        }

        let had_clip_rect = clip_rect != max_rect();
        if had_clip_rect && !should_move_clip_rect(clip_rect, viewport) {
            return false;
        }

        self.page_clip_rect.set(proposed_clip_rect);

        // If we didn't have a clip rect, the previous display doesn't need rebuilding
        // because it was built for infinite clip (max_rect()).
        had_clip_rect
    }

    // https://html.spec.whatwg.org/multipage/#accessing-other-browsing-contexts
    pub fn IndexedGetter(&self, _index: u32, _found: &mut bool) -> Option<Root<Window>> {
        None
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

    pub fn set_devtools_timeline_markers(&self,
                                         markers: Vec<TimelineMarkerType>,
                                         reply: IpcSender<Option<TimelineMarker>>) {
        *self.devtools_marker_sender.borrow_mut() = Some(reply);
        self.devtools_markers.borrow_mut().extend(markers.into_iter());
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
        match self.parent_info {
            Some((_, FrameType::IFrame)) => false,
            _ => true,
        }
    }

    /// Returns whether this window is mozbrowser.
    pub fn is_mozbrowser(&self) -> bool {
        PREFS.is_mozbrowser_enabled() && self.parent_info().is_none()
    }

    /// Returns whether mozbrowser is enabled and `obj` has been created
    /// in a top-level `Window` global.
    #[allow(unsafe_code)]
    pub unsafe fn global_is_mozbrowser(_: *mut JSContext, obj: HandleObject) -> bool {
        GlobalScope::from_object(obj.get())
            .downcast::<Window>()
            .map_or(false, |window| window.is_mozbrowser())
    }

    #[allow(unsafe_code)]
    pub fn dispatch_mozbrowser_event(&self, event: MozBrowserEvent) {
        assert!(PREFS.is_mozbrowser_enabled());
        let custom_event = build_mozbrowser_custom_event(&self, event);
        custom_event.upcast::<Event>().fire(self.upcast());
    }

    pub fn evaluate_media_queries_and_report_changes(&self) {
        self.media_query_lists.evaluate_and_report_changes();
    }

    /// Slow down/speed up timers based on visibility.
    pub fn alter_resource_utilization(&self, visible: bool) {
        if visible {
            self.upcast::<GlobalScope>().speed_up_timers();
        } else {
            self.upcast::<GlobalScope>().slow_down_timers();
        }
    }

    pub fn unminified_js_dir(&self) -> Option<String> {
        self.unminified_js_dir.borrow().clone()
    }
}

impl Window {
    #[allow(unsafe_code)]
    pub fn new(runtime: Rc<Runtime>,
               script_chan: MainThreadScriptChan,
               dom_task_source: DOMManipulationTaskSource,
               user_task_source: UserInteractionTaskSource,
               network_task_source: NetworkingTaskSource,
               history_task_source: HistoryTraversalTaskSource,
               file_task_source: FileReadingTaskSource,
               image_cache_chan: Sender<ImageCacheMsg>,
               image_cache: Arc<ImageCache>,
               resource_threads: ResourceThreads,
               bluetooth_thread: IpcSender<BluetoothRequest>,
               mem_profiler_chan: MemProfilerChan,
               time_profiler_chan: TimeProfilerChan,
               devtools_chan: Option<IpcSender<ScriptToDevtoolsControlMsg>>,
               constellation_chan: IpcSender<ConstellationMsg>,
               control_chan: IpcSender<ConstellationControlMsg>,
               scheduler_chan: IpcSender<TimerSchedulerMsg>,
               timer_event_chan: IpcSender<TimerEvent>,
               layout_chan: Sender<Msg>,
               id: PipelineId,
               parent_info: Option<(PipelineId, FrameType)>,
               window_size: Option<WindowSizeData>,
               origin: MutableOrigin,
               webvr_thread: Option<IpcSender<WebVRMsg>>)
               -> Root<Window> {
        let layout_rpc: Box<LayoutRPC + Send> = {
            let (rpc_send, rpc_recv) = channel();
            layout_chan.send(Msg::GetRPC(rpc_send)).unwrap();
            rpc_recv.recv().unwrap()
        };
        let error_reporter = CSSErrorReporter {
            pipelineid: id,
            script_chan: Arc::new(Mutex::new(control_chan)),
        };
        let current_time = time::get_time();
        let win = box Window {
            globalscope:
                GlobalScope::new_inherited(
                    id,
                    devtools_chan,
                    mem_profiler_chan,
                    time_profiler_chan,
                    constellation_chan,
                    scheduler_chan,
                    resource_threads,
                    timer_event_chan,
                    origin),
            script_chan: script_chan,
            dom_manipulation_task_source: dom_task_source,
            user_interaction_task_source: user_task_source,
            networking_task_source: network_task_source,
            history_traversal_task_source: history_task_source,
            file_reading_task_source: file_task_source,
            image_cache_chan: image_cache_chan,
            image_cache: image_cache.clone(),
            navigator: Default::default(),
            history: Default::default(),
            window_proxy: Default::default(),
            document: Default::default(),
            performance: Default::default(),
            navigation_start: (current_time.sec * 1000 + current_time.nsec as i64 / 1000000) as u64,
            navigation_start_precise: time::precise_time_ns() as f64,
            screen: Default::default(),
            session_storage: Default::default(),
            local_storage: Default::default(),
            status: DOMRefCell::new(DOMString::new()),
            parent_info: parent_info,
            dom_static: GlobalStaticData::new(),
            js_runtime: DOMRefCell::new(Some(runtime.clone())),
            bluetooth_thread: bluetooth_thread,
            bluetooth_extra_permission_data: BluetoothExtraPermissionData::new(),
            page_clip_rect: Cell::new(max_rect()),
            resize_event: Cell::new(None),
            layout_chan: layout_chan,
            layout_rpc: layout_rpc,
            window_size: Cell::new(window_size),
            current_viewport: Cell::new(Rect::zero()),
            suppress_reflow: Cell::new(true),
            pending_reflow_count: Cell::new(0),
            current_state: Cell::new(WindowState::Alive),

            devtools_marker_sender: DOMRefCell::new(None),
            devtools_markers: DOMRefCell::new(HashSet::new()),
            webdriver_script_chan: DOMRefCell::new(None),
            ignore_further_async_events: Default::default(),
            error_reporter: error_reporter,
            scroll_offsets: DOMRefCell::new(HashMap::new()),
            media_query_lists: WeakMediaQueryListVec::new(),
            test_runner: Default::default(),
            webvr_thread: webvr_thread,
            permission_state_invocation_results: DOMRefCell::new(HashMap::new()),
            pending_layout_images: DOMRefCell::new(HashMap::new()),
            unminified_js_dir: DOMRefCell::new(None),
        };

        unsafe {
            WindowBinding::Wrap(runtime.cx(), win)
        }
    }
}

fn should_move_clip_rect(clip_rect: Rect<Au>, new_viewport: Rect<f32>) -> bool {
    let clip_rect = Rect::new(Point2D::new(clip_rect.origin.x.to_f32_px(),
                                           clip_rect.origin.y.to_f32_px()),
                              Size2D::new(clip_rect.size.width.to_f32_px(),
                                          clip_rect.size.height.to_f32_px()));

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

fn debug_reflow_events(id: PipelineId, goal: &ReflowGoal, query_type: &ReflowQueryType, reason: &ReflowReason) {
    let mut debug_msg = format!("**** pipeline={}", id);
    debug_msg.push_str(match *goal {
        ReflowGoal::ForDisplay => "\tForDisplay",
        ReflowGoal::ForScriptQuery => "\tForScriptQuery",
    });

    debug_msg.push_str(match *query_type {
        ReflowQueryType::NoQuery => "\tNoQuery",
        ReflowQueryType::ContentBoxQuery(_n) => "\tContentBoxQuery",
        ReflowQueryType::ContentBoxesQuery(_n) => "\tContentBoxesQuery",
        ReflowQueryType::HitTestQuery(..) => "\tHitTestQuery",
        ReflowQueryType::NodesFromPoint(..) => "\tNodesFromPoint",
        ReflowQueryType::NodeGeometryQuery(_n) => "\tNodeGeometryQuery",
        ReflowQueryType::NodeOverflowQuery(_n) => "\tNodeOverFlowQuery",
        ReflowQueryType::NodeScrollGeometryQuery(_n) => "\tNodeScrollGeometryQuery",
        ReflowQueryType::NodeScrollRootIdQuery(_n) => "\tNodeScrollRootIdQuery",
        ReflowQueryType::ResolvedStyleQuery(_, _, _) => "\tResolvedStyleQuery",
        ReflowQueryType::OffsetParentQuery(_n) => "\tOffsetParentQuery",
        ReflowQueryType::MarginStyleQuery(_n) => "\tMarginStyleQuery",
        ReflowQueryType::TextIndexQuery(..) => "\tTextIndexQuery",
    });

    debug_msg.push_str(match *reason {
        ReflowReason::CachedPageNeededReflow => "\tCachedPageNeededReflow",
        ReflowReason::RefreshTick => "\tRefreshTick",
        ReflowReason::FirstLoad => "\tFirstLoad",
        ReflowReason::KeyEvent => "\tKeyEvent",
        ReflowReason::MouseEvent => "\tMouseEvent",
        ReflowReason::Query => "\tQuery",
        ReflowReason::Timer => "\tTimer",
        ReflowReason::Viewport => "\tViewport",
        ReflowReason::WindowResize => "\tWindowResize",
        ReflowReason::DOMContentLoaded => "\tDOMContentLoaded",
        ReflowReason::DocumentLoaded => "\tDocumentLoaded",
        ReflowReason::StylesheetLoaded => "\tStylesheetLoaded",
        ReflowReason::ImageLoaded => "\tImageLoaded",
        ReflowReason::RequestAnimationFrame => "\tRequestAnimationFrame",
        ReflowReason::WebFontLoaded => "\tWebFontLoaded",
        ReflowReason::FramedContentChanged => "\tFramedContentChanged",
        ReflowReason::IFrameLoadEvent => "\tIFrameLoadEvent",
        ReflowReason::MissingExplicitReflow => "\tMissingExplicitReflow",
        ReflowReason::ElementStateChanged => "\tElementStateChanged",
    });

    println!("{}", debug_msg);
}

struct PostMessageHandler {
    destination: Trusted<Window>,
    origin: Option<ImmutableOrigin>,
    message: StructuredCloneData,
}

impl PostMessageHandler {
    fn new(window: &Window,
           origin: Option<ImmutableOrigin>,
           message: StructuredCloneData) -> PostMessageHandler {
        PostMessageHandler {
            destination: Trusted::new(window),
            origin: origin,
            message: message,
        }
    }
}

impl Runnable for PostMessageHandler {
    // https://html.spec.whatwg.org/multipage/#dom-window-postmessage steps 10-12.
    fn handler(self: Box<PostMessageHandler>) {
        let this = *self;
        let window = this.destination.root();

        // Step 10.
        let doc = window.Document();
        if let Some(source) = this.origin {
            if !source.same_origin(doc.origin()) {
                return;
            }
        }

        let cx = window.get_cx();
        let globalhandle = window.reflector().get_jsobject();
        let _ac = JSAutoCompartment::new(cx, globalhandle.get());

        rooted!(in(cx) let mut message = UndefinedValue());
        this.message.read(window.upcast(), message.handle_mut());

        // Step 11-12.
        // TODO(#12719): set the other attributes.
        MessageEvent::dispatch_jsval(window.upcast(),
                                     window.upcast(),
                                     message.handle());
    }
}

impl Window {
    pub fn post_message(&self, origin: Option<ImmutableOrigin>, data: StructuredCloneData) {
        let runnable = PostMessageHandler::new(self, origin, data);
        let msg = CommonScriptMsg::RunnableMsg(ScriptThreadEventCategory::DomEvent, box runnable);
        // TODO(#12718): Use the "posted message task source".
        let _ = self.script_chan.send(msg);
    }
}
