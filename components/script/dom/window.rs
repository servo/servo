/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use app_units::Au;
use devtools_traits::{ScriptToDevtoolsControlMsg, TimelineMarker, TimelineMarkerType, WorkerId};
use dom::bindings::callback::ExceptionHandling;
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::DocumentBinding::{DocumentMethods, DocumentReadyState};
use dom::bindings::codegen::Bindings::EventHandlerBinding::{EventHandlerNonNull, OnErrorEventHandlerNonNull};
use dom::bindings::codegen::Bindings::FunctionBinding::Function;
use dom::bindings::codegen::Bindings::WindowBinding::{ScrollBehavior, ScrollToOptions};
use dom::bindings::codegen::Bindings::WindowBinding::{self, FrameRequestCallback, WindowMethods};
use dom::bindings::error::{Error, Fallible, report_pending_exception};
use dom::bindings::global::GlobalRef;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::RootedReference;
use dom::bindings::js::{JS, MutNullableHeap, Root};
use dom::bindings::num::Finite;
use dom::bindings::reflector::Reflectable;
use dom::bindings::utils::{GlobalStaticData, WindowProxyHandler};
use dom::browsingcontext::BrowsingContext;
use dom::console::Console;
use dom::crypto::Crypto;
use dom::cssstyledeclaration::{CSSModificationAccess, CSSStyleDeclaration};
use dom::document::Document;
use dom::element::Element;
use dom::eventtarget::EventTarget;
use dom::location::Location;
use dom::navigator::Navigator;
use dom::node::{Node, TrustedNodeAddress, from_untrusted_node_address, window_from_node};
use dom::performance::Performance;
use dom::screen::Screen;
use dom::storage::Storage;
use euclid::{Point2D, Rect, Size2D};
use gfx_traits::LayerId;
use ipc_channel::ipc::{self, IpcSender};
use js::jsapi::{Evaluate2, MutableHandleValue};
use js::jsapi::{HandleValue, JSContext};
use js::jsapi::{JSAutoCompartment, JSAutoRequest, JS_GC, JS_GetRuntime};
use js::rust::CompileOptionsWrapper;
use js::rust::Runtime;
use layout_interface::{ContentBoxResponse, ContentBoxesResponse, ResolvedStyleResponse, ScriptReflow};
use layout_interface::{LayoutChan, LayoutRPC, Msg, Reflow, ReflowQueryType, MarginStyleResponse};
use libc;
use msg::constellation_msg::{ConstellationChan, LoadData, PipelineId, SubpageId, WindowSizeData};
use msg::webdriver_msg::{WebDriverJSError, WebDriverJSResult};
use net_traits::ResourceThread;
use net_traits::bluetooth_thread::BluetoothMethodMsg;
use net_traits::image_cache_thread::{ImageCacheChan, ImageCacheThread};
use net_traits::storage_thread::{StorageThread, StorageType};
use num::traits::ToPrimitive;
use page::Page;
use profile_traits::mem;
use reporter::CSSErrorReporter;
use rustc_serialize::base64::{FromBase64, STANDARD, ToBase64};
use script_runtime::{ScriptChan, ScriptPort};
use script_thread::SendableMainThreadScriptChan;
use script_thread::{MainThreadScriptChan, MainThreadScriptMsg, RunnableWrapper};
use script_traits::{ConstellationControlMsg, UntrustedNodeAddress};
use script_traits::{DocumentState, MsDuration, ScriptToCompositorMsg, TimerEvent, TimerEventId};
use script_traits::{MozBrowserEvent, ScriptMsg as ConstellationMsg, TimerEventRequest, TimerSource};
use std::ascii::AsciiExt;
use std::borrow::ToOwned;
use std::cell::Cell;
use std::collections::HashSet;
use std::default::Default;
use std::ffi::CString;
use std::io::{Write, stderr, stdout};
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::TryRecvError::{Disconnected, Empty};
use std::sync::mpsc::{Sender, channel};
use std::sync::{Arc, Mutex};
use string_cache::Atom;
use style::context::ReflowGoal;
use style::error_reporting::ParseErrorReporter;
use style::selector_impl::PseudoElement;
use task_source::TaskSource;
use task_source::dom_manipulation::{DOMManipulationTaskSource, DOMManipulationTask};
use task_source::file_reading::FileReadingTaskSource;
use task_source::history_traversal::HistoryTraversalTaskSource;
use task_source::networking::NetworkingTaskSource;
use task_source::user_interaction::UserInteractionTaskSource;
use time;
use timers::{IsInterval, OneshotTimerCallback, OneshotTimerHandle, OneshotTimers, TimerCallback};
use url::Url;
use util::geometry::{self, MAX_RECT};
use util::str::{DOMString, HTML_SPACE_CHARACTERS};
use util::{breakpoint, opts};
use webdriver_handlers::jsval_to_webdriver;

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
}

pub type ScrollPoint = Point2D<Au>;

#[dom_struct]
pub struct Window {
    eventtarget: EventTarget,
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
    console: MutNullableHeap<JS<Console>>,
    crypto: MutNullableHeap<JS<Crypto>>,
    navigator: MutNullableHeap<JS<Navigator>>,
    #[ignore_heap_size_of = "channels are hard"]
    image_cache_thread: ImageCacheThread,
    #[ignore_heap_size_of = "channels are hard"]
    image_cache_chan: ImageCacheChan,
    #[ignore_heap_size_of = "TODO(#6911) newtypes containing unmeasurable types are hard"]
    compositor: IpcSender<ScriptToCompositorMsg>,
    browsing_context: MutNullableHeap<JS<BrowsingContext>>,
    page: Rc<Page>,
    performance: MutNullableHeap<JS<Performance>>,
    navigation_start: u64,
    navigation_start_precise: f64,
    screen: MutNullableHeap<JS<Screen>>,
    session_storage: MutNullableHeap<JS<Storage>>,
    local_storage: MutNullableHeap<JS<Storage>>,
    #[ignore_heap_size_of = "channels are hard"]
    scheduler_chan: IpcSender<TimerEventRequest>,
    timers: OneshotTimers,

    next_worker_id: Cell<WorkerId>,

    /// For sending messages to the memory profiler.
    #[ignore_heap_size_of = "channels are hard"]
    mem_profiler_chan: mem::ProfilerChan,

    /// For providing instructions to an optional devtools server.
    #[ignore_heap_size_of = "channels are hard"]
    devtools_chan: Option<IpcSender<ScriptToDevtoolsControlMsg>>,
    /// For sending timeline markers. Will be ignored if
    /// no devtools server
    #[ignore_heap_size_of = "TODO(#6909) need to measure HashSet"]
    devtools_markers: DOMRefCell<HashSet<TimelineMarkerType>>,
    #[ignore_heap_size_of = "channels are hard"]
    devtools_marker_sender: DOMRefCell<Option<IpcSender<TimelineMarker>>>,

    /// A flag to indicate whether the developer tools have requested live updates of
    /// page changes.
    devtools_wants_updates: Cell<bool>,

    next_subpage_id: Cell<SubpageId>,

    /// Pending resize event, if any.
    resize_event: Cell<Option<WindowSizeData>>,

    /// Pipeline id associated with this page.
    id: PipelineId,

    /// Subpage id associated with this page, if any.
    parent_info: Option<(PipelineId, SubpageId)>,

    /// Global static data related to the DOM.
    dom_static: GlobalStaticData,

    /// The JavaScript runtime.
    #[ignore_heap_size_of = "Rc<T> is hard"]
    js_runtime: DOMRefCell<Option<Rc<Runtime>>>,

    /// A handle for communicating messages to the layout thread.
    #[ignore_heap_size_of = "channels are hard"]
    layout_chan: LayoutChan,

    /// A handle to perform RPC calls into the layout, quickly.
    #[ignore_heap_size_of = "trait objects are hard"]
    layout_rpc: Box<LayoutRPC + 'static>,

    /// The current size of the window, in pixels.
    window_size: Cell<Option<WindowSizeData>>,

    /// Associated resource thread for use by DOM objects like XMLHttpRequest
    #[ignore_heap_size_of = "channels are hard"]
    resource_thread: Arc<ResourceThread>,

    /// A handle for communicating messages to the bluetooth thread.
    #[ignore_heap_size_of = "channels are hard"]
    bluetooth_thread: IpcSender<BluetoothMethodMsg>,

    /// A handle for communicating messages to the storage thread.
    #[ignore_heap_size_of = "channels are hard"]
    storage_thread: StorageThread,

    /// A handle for communicating messages to the constellation thread.
    #[ignore_heap_size_of = "channels are hard"]
    constellation_chan: ConstellationChan<ConstellationMsg>,

    /// Pending scroll to fragment event, if any
    fragment_name: DOMRefCell<Option<String>>,

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
    ignore_further_async_events: Arc<AtomicBool>,

    error_reporter: CSSErrorReporter,
}

impl Window {
    #[allow(unsafe_code)]
    pub fn clear_js_runtime_for_script_deallocation(&self) {
        unsafe {
            *self.js_runtime.borrow_for_script_deallocation() = None;
            self.browsing_context.set(None);
            self.current_state.set(WindowState::Zombie);
            self.ignore_further_async_events.store(true, Ordering::Relaxed);
        }
    }

    pub fn get_cx(&self) -> *mut JSContext {
        self.js_runtime.borrow().as_ref().unwrap().cx()
    }

    pub fn dom_manipulation_task_source(&self) -> Box<TaskSource<DOMManipulationTask> + Send> {
        self.dom_manipulation_task_source.clone()
    }

    pub fn user_interaction_task_source(&self) -> Box<ScriptChan + Send> {
        self.user_interaction_task_source.clone()
    }

    pub fn networking_task_source(&self) -> Box<ScriptChan + Send> {
        self.networking_task_source.clone()
    }

    pub fn history_traversal_task_source(&self) -> Box<ScriptChan + Send> {
        self.history_traversal_task_source.clone()
    }

    pub fn file_reading_task_source(&self) -> Box<ScriptChan + Send> {
        self.file_reading_task_source.clone()
    }

    pub fn main_thread_script_chan(&self) -> &Sender<MainThreadScriptMsg> {
        &self.script_chan.0
    }

    pub fn image_cache_chan(&self) -> ImageCacheChan {
        self.image_cache_chan.clone()
    }

    pub fn get_next_worker_id(&self) -> WorkerId {
        let worker_id = self.next_worker_id.get();
        let WorkerId(id_num) = worker_id;
        self.next_worker_id.set(WorkerId(id_num + 1));
        worker_id
    }

    pub fn pipeline(&self) -> PipelineId {
        self.id
    }

    pub fn subpage(&self) -> Option<SubpageId> {
        self.parent_info.map(|p| p.1)
    }

    pub fn parent_info(&self) -> Option<(PipelineId, SubpageId)> {
        self.parent_info
    }

    pub fn new_script_pair(&self) -> (Box<ScriptChan + Send>, Box<ScriptPort + Send>) {
        let (tx, rx) = channel();
        (box SendableMainThreadScriptChan(tx), box rx)
    }

    pub fn image_cache_thread(&self) -> &ImageCacheThread {
        &self.image_cache_thread
    }

    pub fn compositor(&self) -> &IpcSender<ScriptToCompositorMsg> {
        &self.compositor
    }

    pub fn browsing_context(&self) -> Root<BrowsingContext> {
        self.browsing_context.get().unwrap()
    }

    pub fn page(&self) -> &Page {
        &*self.page
    }

    pub fn bluetooth_thread(&self) -> IpcSender<BluetoothMethodMsg> {
        self.bluetooth_thread.clone()
    }

    pub fn storage_thread(&self) -> StorageThread {
        self.storage_thread.clone()
    }

    pub fn css_error_reporter(&self) -> Box<ParseErrorReporter + Send> {
        self.error_reporter.clone()
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
        Ok(DOMString::from(octets.to_base64(STANDARD)))
    }
}

// https://html.spec.whatwg.org/multipage/#atob
pub fn base64_atob(input: DOMString) -> Fallible<DOMString> {
    // "Remove all space characters from input."
    // serialize::base64::from_base64 ignores \r and \n,
    // but it treats the other space characters as
    // invalid input.
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

    match input.from_base64() {
        Ok(data) => Ok(DOMString::from(data.iter().map(|&b| b as char).collect::<String>())),
        Err(..) => Err(Error::InvalidCharacter)
    }
}

impl WindowMethods for Window {
    // https://html.spec.whatwg.org/multipage/#dom-alert
    fn Alert(&self, s: DOMString) {
        // Right now, just print to the console
        // Ensure that stderr doesn't trample through the alert() we use to
        // communicate test results (see executorservo.py in wptrunner).
        let stderr = stderr();
        let mut stderr = stderr.lock();
        let stdout = stdout();
        let mut stdout = stdout.lock();
        writeln!(&mut stdout, "ALERT: {}", s).unwrap();
        stdout.flush().unwrap();
        stderr.flush().unwrap();

        // https://developer.mozilla.org/en-US/docs/Web/Events/mozbrowsershowmodalprompt
        let event = MozBrowserEvent::ShowModalPrompt("alert".to_owned(), "Alert".to_owned(),
                                                     String::from(s), "".to_owned());
        self.Document().trigger_mozbrowser_event(event);
    }

    // https://html.spec.whatwg.org/multipage/#dom-window-close
    fn Close(&self) {
        self.main_thread_script_chan().send(MainThreadScriptMsg::ExitWindow(self.id.clone())).unwrap();
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-2
    fn Document(&self) -> Root<Document> {
        self.browsing_context().active_document()
    }

    // https://html.spec.whatwg.org/multipage/#dom-location
    fn Location(&self) -> Root<Location> {
        self.Document().GetLocation().unwrap()
    }

    // https://html.spec.whatwg.org/multipage/#dom-sessionstorage
    fn SessionStorage(&self) -> Root<Storage> {
        self.session_storage.or_init(|| Storage::new(&GlobalRef::Window(self), StorageType::Session))
    }

    // https://html.spec.whatwg.org/multipage/#dom-localstorage
    fn LocalStorage(&self) -> Root<Storage> {
        self.local_storage.or_init(|| Storage::new(&GlobalRef::Window(self), StorageType::Local))
    }

    // https://developer.mozilla.org/en-US/docs/Web/API/Console
    fn Console(&self) -> Root<Console> {
        self.console.or_init(|| Console::new(GlobalRef::Window(self)))
    }

    // https://dvcs.w3.org/hg/webcrypto-api/raw-file/tip/spec/Overview.html#dfn-GlobalCrypto
    fn Crypto(&self) -> Root<Crypto> {
        self.crypto.or_init(|| Crypto::new(GlobalRef::Window(self)))
    }

    // https://html.spec.whatwg.org/multipage/#dom-frameelement
    fn GetFrameElement(&self) -> Option<Root<Element>> {
        self.browsing_context().frame_element().map(Root::from_ref)
    }

    // https://html.spec.whatwg.org/multipage/#dom-navigator
    fn Navigator(&self) -> Root<Navigator> {
        self.navigator.or_init(|| Navigator::new(self))
    }

    // https://html.spec.whatwg.org/multipage/#dom-windowtimers-settimeout
    fn SetTimeout(&self, _cx: *mut JSContext, callback: Rc<Function>, timeout: i32, args: Vec<HandleValue>) -> i32 {
        self.timers.set_timeout_or_interval(GlobalRef::Window(self),
                                            TimerCallback::FunctionTimerCallback(callback),
                                            args,
                                            timeout,
                                            IsInterval::NonInterval,
                                            TimerSource::FromWindow(self.id.clone()))
    }

    // https://html.spec.whatwg.org/multipage/#dom-windowtimers-settimeout
    fn SetTimeout_(&self, _cx: *mut JSContext, callback: DOMString, timeout: i32, args: Vec<HandleValue>) -> i32 {
        self.timers.set_timeout_or_interval(GlobalRef::Window(self),
                                            TimerCallback::StringTimerCallback(callback),
                                            args,
                                            timeout,
                                            IsInterval::NonInterval,
                                            TimerSource::FromWindow(self.id.clone()))
    }

    // https://html.spec.whatwg.org/multipage/#dom-windowtimers-cleartimeout
    fn ClearTimeout(&self, handle: i32) {
        self.timers.clear_timeout_or_interval(GlobalRef::Window(self), handle);
    }

    // https://html.spec.whatwg.org/multipage/#dom-windowtimers-setinterval
    fn SetInterval(&self, _cx: *mut JSContext, callback: Rc<Function>, timeout: i32, args: Vec<HandleValue>) -> i32 {
        self.timers.set_timeout_or_interval(GlobalRef::Window(self),
                                            TimerCallback::FunctionTimerCallback(callback),
                                            args,
                                            timeout,
                                            IsInterval::Interval,
                                            TimerSource::FromWindow(self.id.clone()))
    }

    // https://html.spec.whatwg.org/multipage/#dom-windowtimers-setinterval
    fn SetInterval_(&self, _cx: *mut JSContext, callback: DOMString, timeout: i32, args: Vec<HandleValue>) -> i32 {
        self.timers.set_timeout_or_interval(GlobalRef::Window(self),
                                            TimerCallback::StringTimerCallback(callback),
                                            args,
                                            timeout,
                                            IsInterval::Interval,
                                            TimerSource::FromWindow(self.id.clone()))
    }

    // https://html.spec.whatwg.org/multipage/#dom-windowtimers-clearinterval
    fn ClearInterval(&self, handle: i32) {
        self.ClearTimeout(handle);
    }

    // https://html.spec.whatwg.org/multipage/#dom-window
    fn Window(&self) -> Root<Window> {
        Root::from_ref(self)
    }

    // https://html.spec.whatwg.org/multipage/#dom-self
    fn Self_(&self) -> Root<Window> {
        self.Window()
    }

    // https://html.spec.whatwg.org/multipage/#dom-frames
    fn Frames(&self) -> Root<Window> {
        self.Window()
    }

    // https://html.spec.whatwg.org/multipage/#dom-parent
    fn Parent(&self) -> Root<Window> {
        self.parent().unwrap_or(self.Window())
    }

    // https://html.spec.whatwg.org/multipage/#dom-top
    fn Top(&self) -> Root<Window> {
        let mut window = self.Window();
        while let Some(parent) = window.parent() {
            window = parent;
        }
        window
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

    // https://html.spec.whatwg.org/multipage/#handler-window-onunload
    event_handler!(unload, GetOnunload, SetOnunload);

    // https://html.spec.whatwg.org/multipage/#handler-window-onstorage
    event_handler!(storage, GetOnstorage, SetOnstorage);

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
        let doc = self.Document();

        let callback = move |now: f64| {
            // TODO: @jdm The spec says that any exceptions should be suppressed;
            // https://github.com/servo/servo/issues/6928
            let _ = callback.Call__(Finite::wrap(now), ExceptionHandling::Report);
        };

        doc.request_animation_frame(Box::new(callback))
    }

    /// https://html.spec.whatwg.org/multipage/#dom-window-cancelanimationframe
    fn CancelAnimationFrame(&self, ident: u32) {
        let doc = self.Document();
        doc.cancel_animation_frame(ident);
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

    fn Trap(&self) {
        breakpoint();
    }

    #[allow(unsafe_code)]
    fn WebdriverCallback(&self, cx: *mut JSContext, val: HandleValue) {
        let rv = unsafe { jsval_to_webdriver(cx, val) };
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
        CSSStyleDeclaration::new(self, element, pseudo, CSSModificationAccess::Readonly)
    }

    // https://drafts.csswg.org/cssom-view/#dom-window-innerheight
    //TODO Include Scrollbar
    fn InnerHeight(&self) -> i32 {
        self.window_size.get()
                        .and_then(|e| e.visible_viewport.height.get().to_i32())
                        .unwrap_or(0)
    }

    // https://drafts.csswg.org/cssom-view/#dom-window-innerwidth
    //TODO Include Scrollbar
    fn InnerWidth(&self) -> i32 {
        self.window_size.get()
                        .and_then(|e| e.visible_viewport.width.get().to_i32())
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
        self.compositor.send(ScriptToCompositorMsg::ResizeTo(size)).unwrap()
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
        self.compositor.send(ScriptToCompositorMsg::MoveTo(point)).unwrap()
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
}

pub trait ScriptHelpers {
    fn evaluate_js_on_global_with_result(self, code: &str,
                                         rval: MutableHandleValue);
    fn evaluate_script_on_global_with_result(self, code: &str, filename: &str,
                                             rval: MutableHandleValue);
}

impl<'a, T: Reflectable> ScriptHelpers for &'a T {
    fn evaluate_js_on_global_with_result(self, code: &str,
                                         rval: MutableHandleValue) {
        self.evaluate_script_on_global_with_result(code, "", rval)
    }

    #[allow(unsafe_code)]
    fn evaluate_script_on_global_with_result(self, code: &str, filename: &str,
                                             rval: MutableHandleValue) {
        let global = self.global();
        let cx = global.r().get_cx();
        let _ar = JSAutoRequest::new(cx);
        let globalhandle = global.r().reflector().get_jsobject();
        let code: Vec<u16> = code.encode_utf16().collect();
        let filename = CString::new(filename).unwrap();

        let _ac = JSAutoCompartment::new(cx, globalhandle.get());
        let options = CompileOptionsWrapper::new(cx, filename.as_ptr(), 0);
        unsafe {
            if !Evaluate2(cx, options.ptr, code.as_ptr(),
                          code.len() as libc::size_t,
                          rval) {
                debug!("error evaluating JS string");
                report_pending_exception(cx, globalhandle.get());
            }
        }
    }
}

impl Window {
    pub fn get_runnable_wrapper(&self) -> RunnableWrapper {
        RunnableWrapper {
            cancelled: self.ignore_further_async_events.clone()
        }
    }

    pub fn clear_js_runtime(&self) {
        self.Document().upcast::<Node>().teardown();

        // The above code may not catch all DOM objects
        // (e.g. DOM objects removed from the tree that haven't
        // been collected yet). Forcing a GC here means that
        // those DOM objects will be able to call dispose()
        // to free their layout data before the layout thread
        // exits. Without this, those remaining objects try to
        // send a message to free their layout data to the
        // layout thread when the script thread is dropped,
        // which causes a panic!
        self.Gc();

        self.current_state.set(WindowState::Zombie);
        *self.js_runtime.borrow_mut() = None;
        self.browsing_context.set(None);
        self.ignore_further_async_events.store(true, Ordering::Relaxed);
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
                let content_size = e.upcast::<Node>().bounding_content_box();
                let content_height = content_size.size.height.to_f64_px();
                let content_width = content_size.size.width.to_f64_px();
                (xfinite.max(0.0f64).min(content_width - width),
                 yfinite.max(0.0f64).min(content_height - height))
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
        self.perform_a_scroll(x.to_f32().unwrap_or(0.0f32), y.to_f32().unwrap_or(0.0f32), behavior, None);
    }

    /// https://drafts.csswg.org/cssom-view/#perform-a-scroll
    pub fn perform_a_scroll(&self, x: f32, y: f32, behavior: ScrollBehavior, element: Option<&Element>) {
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
        let size = self.current_viewport.get().size;
        self.current_viewport.set(Rect::new(Point2D::new(Au::from_f32_px(x), Au::from_f32_px(y)), size));

        self.compositor.send(ScriptToCompositorMsg::ScrollFragmentPoint(
                                                         self.pipeline(), LayerId::null(), point, smooth)).unwrap()
    }

    pub fn client_window(&self) -> (Size2D<u32>, Point2D<i32>) {
        let (send, recv) = ipc::channel::<(Size2D<u32>, Point2D<i32>)>().unwrap();
        self.compositor.send(ScriptToCompositorMsg::GetClientWindow(send)).unwrap();
        recv.recv().unwrap_or((Size2D::zero(), Point2D::zero()))
    }

    /// Reflows the page unconditionally if possible and not suppressed. This
    /// method will wait for the layout thread to complete (but see the `TODO`
    /// below). If there is no window size yet, the page is presumed invisible
    /// and no reflow is performed. If reflow is suppressed, no reflow will be
    /// performed for ForDisplay goals.
    ///
    /// TODO(pcwalton): Only wait for style recalc, since we have off-main-thread layout.
    pub fn force_reflow(&self, goal: ReflowGoal, query_type: ReflowQueryType, reason: ReflowReason) {
        // Check if we need to unsuppress reflow. Note that this needs to be
        // *before* any early bailouts, or reflow might never be unsuppresed!
        match reason {
            ReflowReason::FirstLoad | ReflowReason::RefreshTick => self.suppress_reflow.set(false),
            _ => (),
        }

        // If there is no window size, we have nothing to do.
        let window_size = match self.window_size.get() {
            Some(window_size) => window_size,
            None => return,
        };

        let for_display = query_type == ReflowQueryType::NoQuery;
        if for_display && self.suppress_reflow.get() {
            debug!("Suppressing reflow pipeline {} for goal {:?} reason {:?} before FirstLoad or RefreshTick",
                   self.id, goal, reason);
            return;
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
            debug_reflow_events(self.id, &goal, &query_type, &reason);
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
        };

        let LayoutChan(ref chan) = self.layout_chan;
        chan.send(Msg::Reflow(reflow)).unwrap();

        debug!("script: layout forked");

        match join_port.try_recv() {
            Err(Empty) => {
                info!("script: waiting on layout");
                join_port.recv().unwrap();
            }
            Ok(_) => {}
            Err(Disconnected) => {
                panic!("Layout thread failed while script was waiting for a result.");
            }
        }

        debug!("script: layout joined");

        // Pending reflows require display, so only reset the pending reflow count if this reflow
        // was to be displayed.
        if goal == ReflowGoal::ForDisplay {
            self.pending_reflow_count.set(0);
        }

        if let Some(marker) = marker {
            self.emit_timeline_marker(marker.end());
        }
    }

    /// Reflows the page if it's possible to do so and the page is dirty. This method will wait
    /// for the layout thread to complete (but see the `TODO` below). If there is no window size
    /// yet, the page is presumed invisible and no reflow is performed.
    ///
    /// TODO(pcwalton): Only wait for style recalc, since we have off-main-thread layout.
    pub fn reflow(&self, goal: ReflowGoal, query_type: ReflowQueryType, reason: ReflowReason) {
        let for_display = query_type == ReflowQueryType::NoQuery;

        if !for_display || self.Document().needs_reflow() {
            self.force_reflow(goal, query_type, reason);

            // If window_size is `None`, we don't reflow, so the document stays dirty.
            // Otherwise, we shouldn't need a reflow immediately after a reflow.
            assert!(!self.Document().needs_reflow() ||
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
        if (opts::get().output_file.is_some() || opts::get().exit_after_load) && for_display {
            let document = self.Document();

            // Checks if the html element has reftest-wait attribute present.
            // See http://testthewebforward.org/docs/reftests.html
            let html_element = document.GetDocumentElement();
            let reftest_wait = html_element.map_or(false, |elem| {
                elem.has_class(&Atom::from("reftest-wait"))
            });

            let ready_state = document.ReadyState();

            if ready_state == DocumentReadyState::Complete && !reftest_wait {
                let event = ConstellationMsg::SetDocumentState(self.id, DocumentState::Idle);
                self.constellation_chan().0.send(event).unwrap();
            }
        }
    }

    pub fn layout(&self) -> &LayoutRPC {
        &*self.layout_rpc
    }

    pub fn content_box_query(&self, content_box_request: TrustedNodeAddress) -> Rect<Au> {
        self.reflow(ReflowGoal::ForScriptQuery,
                    ReflowQueryType::ContentBoxQuery(content_box_request),
                    ReflowReason::Query);
        let ContentBoxResponse(rect) = self.layout_rpc.content_box();
        rect
    }

    pub fn content_boxes_query(&self, content_boxes_request: TrustedNodeAddress) -> Vec<Rect<Au>> {
        self.reflow(ReflowGoal::ForScriptQuery,
                    ReflowQueryType::ContentBoxesQuery(content_boxes_request),
                    ReflowReason::Query);
        let ContentBoxesResponse(rects) = self.layout_rpc.content_boxes();
        rects
    }

    pub fn client_rect_query(&self, node_geometry_request: TrustedNodeAddress) -> Rect<i32> {
        self.reflow(ReflowGoal::ForScriptQuery,
                    ReflowQueryType::NodeGeometryQuery(node_geometry_request),
                    ReflowReason::Query);
        self.layout_rpc.node_geometry().client_rect
    }

    pub fn hit_test_query(&self, hit_test_request: Point2D<f32>, update_cursor: bool)
                          -> Option<UntrustedNodeAddress> {
        self.reflow(ReflowGoal::ForScriptQuery,
                    ReflowQueryType::HitTestQuery(hit_test_request, update_cursor),
                    ReflowReason::Query);
        self.layout_rpc.hit_test().node_address
    }

    pub fn scroll_area_query(&self, node: TrustedNodeAddress) -> Rect<i32> {
        self.reflow(ReflowGoal::ForScriptQuery,
                    ReflowQueryType::NodeScrollGeometryQuery(node),
                    ReflowReason::Query);
        self.layout_rpc.node_scroll_area().client_rect
    }

    pub fn resolved_style_query(&self,
                            element: TrustedNodeAddress,
                            pseudo: Option<PseudoElement>,
                            property: &Atom) -> Option<DOMString> {
        self.reflow(ReflowGoal::ForScriptQuery,
                    ReflowQueryType::ResolvedStyleQuery(element, pseudo, property.clone()),
                    ReflowReason::Query);
        let ResolvedStyleResponse(resolved) = self.layout_rpc.resolved_style();
        resolved.map(DOMString::from)
    }

    pub fn offset_parent_query(&self, node: TrustedNodeAddress) -> (Option<Root<Element>>, Rect<Au>) {
        self.reflow(ReflowGoal::ForScriptQuery,
                    ReflowQueryType::OffsetParentQuery(node),
                    ReflowReason::Query);
        let response = self.layout_rpc.offset_parent();
        let js_runtime = self.js_runtime.borrow();
        let js_runtime = js_runtime.as_ref().unwrap();
        let element = response.node_address.and_then(|parent_node_address| {
            let node = from_untrusted_node_address(js_runtime.rt(), parent_node_address);
            Root::downcast(node)
        });
        (element, response.rect)
    }

    pub fn margin_style_query(&self, node: TrustedNodeAddress) -> MarginStyleResponse {
        self.reflow(ReflowGoal::ForScriptQuery,
                    ReflowQueryType::MarginStyleQuery(node),
                    ReflowReason::Query);
        self.layout_rpc.margin_style()
    }

    pub fn init_browsing_context(&self, browsing_context: &BrowsingContext) {
        assert!(self.browsing_context.get().is_none());
        self.browsing_context.set(Some(&browsing_context));
    }

    /// Commence a new URL load which will either replace this window or scroll to a fragment.
    pub fn load_url(&self, url: Url) {
        self.main_thread_script_chan().send(
            MainThreadScriptMsg::Navigate(self.id, LoadData::new(url))).unwrap();
    }

    pub fn handle_fire_timer(&self, timer_id: TimerEventId) {
        self.timers.fire_timer(timer_id, self);
        self.reflow(ReflowGoal::ForDisplay, ReflowQueryType::NoQuery, ReflowReason::Timer);
    }

    pub fn set_fragment_name(&self, fragment: Option<String>) {
        *self.fragment_name.borrow_mut() = fragment;
    }

    pub fn steal_fragment_name(&self) -> Option<String> {
        self.fragment_name.borrow_mut().take()
    }

    pub fn set_window_size(&self, size: WindowSizeData) {
        self.window_size.set(Some(size));
    }

    pub fn window_size(&self) -> Option<WindowSizeData> {
        self.window_size.get()
    }

    pub fn get_url(&self) -> Url {
        (*self.Document().url()).clone()
    }

    pub fn resource_thread(&self) -> ResourceThread {
        (*self.resource_thread).clone()
    }

    pub fn mem_profiler_chan(&self) -> mem::ProfilerChan {
        self.mem_profiler_chan.clone()
    }

    pub fn devtools_chan(&self) -> Option<IpcSender<ScriptToDevtoolsControlMsg>> {
        self.devtools_chan.clone()
    }

    pub fn layout_chan(&self) -> LayoutChan {
        self.layout_chan.clone()
    }

    pub fn constellation_chan(&self) -> ConstellationChan<ConstellationMsg> {
        self.constellation_chan.clone()
    }

    pub fn scheduler_chan(&self) -> IpcSender<TimerEventRequest> {
        self.scheduler_chan.clone()
    }

    pub fn schedule_callback(&self, callback: OneshotTimerCallback, duration: MsDuration) -> OneshotTimerHandle {
        self.timers.schedule_callback(callback,
                                      duration,
                                      TimerSource::FromWindow(self.id.clone()))
    }

    pub fn unschedule_callback(&self, handle: OneshotTimerHandle) {
        self.timers.unschedule_callback(handle);
    }

    pub fn windowproxy_handler(&self) -> WindowProxyHandler {
        WindowProxyHandler(self.dom_static.windowproxy_handler.0)
    }

    pub fn get_next_subpage_id(&self) -> SubpageId {
        let subpage_id = self.next_subpage_id.get();
        let SubpageId(id_num) = subpage_id;
        self.next_subpage_id.set(SubpageId(id_num + 1));
        subpage_id
    }

    pub fn get_pending_reflow_count(&self) -> u32 {
        self.pending_reflow_count.get()
    }

    pub fn add_pending_reflow(&self) {
        self.pending_reflow_count.set(self.pending_reflow_count.get() + 1);
    }

    pub fn set_resize_event(&self, event: WindowSizeData) {
        self.resize_event.set(Some(event));
    }

    pub fn steal_resize_event(&self) -> Option<WindowSizeData> {
        let event = self.resize_event.get();
        self.resize_event.set(None);
        event
    }

    pub fn set_page_clip_rect_with_new_viewport(&self, viewport: Rect<f32>) -> bool {
        let rect = geometry::f32_rect_to_au_rect(viewport.clone());
        self.current_viewport.set(rect);
        // We use a clipping rectangle that is five times the size of the of the viewport,
        // so that we don't collect display list items for areas too far outside the viewport,
        // but also don't trigger reflows every time the viewport changes.
        static VIEWPORT_EXPANSION: f32 = 2.0; // 2 lengths on each side plus original length is 5 total.
        let proposed_clip_rect = geometry::f32_rect_to_au_rect(
            viewport.inflate(viewport.size.width * VIEWPORT_EXPANSION,
            viewport.size.height * VIEWPORT_EXPANSION));
        let clip_rect = self.page_clip_rect.get();
        if proposed_clip_rect == clip_rect {
            return false;
        }

        let had_clip_rect = clip_rect != MAX_RECT;
        if had_clip_rect && !should_move_clip_rect(clip_rect, viewport) {
            return false;
        }

        self.page_clip_rect.set(proposed_clip_rect);

        // If we didn't have a clip rect, the previous display doesn't need rebuilding
        // because it was built for infinite clip (MAX_RECT).
        had_clip_rect
    }

    pub fn set_devtools_wants_updates(&self, value: bool) {
        self.devtools_wants_updates.set(value);
    }

    // https://html.spec.whatwg.org/multipage/#accessing-other-browsing-contexts
    pub fn IndexedGetter(&self, _index: u32, _found: &mut bool) -> Option<Root<Window>> {
        None
    }

    pub fn thaw(&self) {
        self.timers.resume();

        // Push the document title to the compositor since we are
        // activating this document due to a navigation.
        self.Document().title_changed();
    }

    pub fn freeze(&self) {
        self.timers.suspend();
    }

    pub fn need_emit_timeline_marker(&self, timeline_type: TimelineMarkerType) -> bool {
        let markers = self.devtools_markers.borrow();
        markers.contains(&timeline_type)
    }

    pub fn emit_timeline_marker(&self, marker: TimelineMarker) {
        let sender = self.devtools_marker_sender.borrow();
        let sender = sender.as_ref().expect("There is no marker sender");
        sender.send(marker).unwrap();
    }

    pub fn set_devtools_timeline_markers(&self,
                                         markers: Vec<TimelineMarkerType>,
                                         reply: IpcSender<TimelineMarker>) {
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

    pub fn parent(&self) -> Option<Root<Window>> {
        let browsing_context = self.browsing_context();

        browsing_context.frame_element().map(|frame_element| {
            let window = window_from_node(frame_element);
            let context = window.browsing_context();
            context.active_window()
        })
    }
}

impl Window {
    pub fn new(runtime: Rc<Runtime>,
               page: Rc<Page>,
               script_chan: MainThreadScriptChan,
               dom_task_source: DOMManipulationTaskSource,
               user_task_source: UserInteractionTaskSource,
               network_task_source: NetworkingTaskSource,
               history_task_source: HistoryTraversalTaskSource,
               file_task_source: FileReadingTaskSource,
               image_cache_chan: ImageCacheChan,
               compositor: IpcSender<ScriptToCompositorMsg>,
               image_cache_thread: ImageCacheThread,
               resource_thread: Arc<ResourceThread>,
               bluetooth_thread: IpcSender<BluetoothMethodMsg>,
               storage_thread: StorageThread,
               mem_profiler_chan: mem::ProfilerChan,
               devtools_chan: Option<IpcSender<ScriptToDevtoolsControlMsg>>,
               constellation_chan: ConstellationChan<ConstellationMsg>,
               control_chan: IpcSender<ConstellationControlMsg>,
               scheduler_chan: IpcSender<TimerEventRequest>,
               timer_event_chan: IpcSender<TimerEvent>,
               layout_chan: LayoutChan,
               id: PipelineId,
               parent_info: Option<(PipelineId, SubpageId)>,
               window_size: Option<WindowSizeData>)
               -> Root<Window> {
        let layout_rpc: Box<LayoutRPC> = {
            let (rpc_send, rpc_recv) = channel();
            let LayoutChan(ref lchan) = layout_chan;
            lchan.send(Msg::GetRPC(rpc_send)).unwrap();
            rpc_recv.recv().unwrap()
        };
        let error_reporter = CSSErrorReporter {
            pipelineid: id,
            script_chan: Arc::new(Mutex::new(control_chan)),
        };
        let current_time = time::get_time();
        let win = box Window {
            eventtarget: EventTarget::new_inherited(),
            script_chan: script_chan,
            dom_manipulation_task_source: dom_task_source,
            user_interaction_task_source: user_task_source,
            networking_task_source: network_task_source,
            history_traversal_task_source: history_task_source,
            file_reading_task_source: file_task_source,
            image_cache_chan: image_cache_chan,
            console: Default::default(),
            crypto: Default::default(),
            compositor: compositor,
            page: page,
            navigator: Default::default(),
            image_cache_thread: image_cache_thread,
            mem_profiler_chan: mem_profiler_chan,
            devtools_chan: devtools_chan,
            browsing_context: Default::default(),
            performance: Default::default(),
            navigation_start: (current_time.sec * 1000 + current_time.nsec as i64 / 1000000) as u64,
            navigation_start_precise: time::precise_time_ns() as f64,
            screen: Default::default(),
            session_storage: Default::default(),
            local_storage: Default::default(),
            scheduler_chan: scheduler_chan.clone(),
            timers: OneshotTimers::new(timer_event_chan, scheduler_chan),
            next_worker_id: Cell::new(WorkerId(0)),
            id: id,
            parent_info: parent_info,
            dom_static: GlobalStaticData::new(),
            js_runtime: DOMRefCell::new(Some(runtime.clone())),
            resource_thread: resource_thread,
            bluetooth_thread: bluetooth_thread,
            storage_thread: storage_thread,
            constellation_chan: constellation_chan,
            page_clip_rect: Cell::new(MAX_RECT),
            fragment_name: DOMRefCell::new(None),
            resize_event: Cell::new(None),
            next_subpage_id: Cell::new(SubpageId(0)),
            layout_chan: layout_chan,
            layout_rpc: layout_rpc,
            window_size: Cell::new(window_size),
            current_viewport: Cell::new(Rect::zero()),
            suppress_reflow: Cell::new(true),
            pending_reflow_count: Cell::new(0),
            current_state: Cell::new(WindowState::Alive),

            devtools_marker_sender: DOMRefCell::new(None),
            devtools_markers: DOMRefCell::new(HashSet::new()),
            devtools_wants_updates: Cell::new(false),
            webdriver_script_chan: DOMRefCell::new(None),
            ignore_further_async_events: Arc::new(AtomicBool::new(false)),
            error_reporter: error_reporter
        };

        WindowBinding::Wrap(runtime.cx(), win)
    }
    pub fn live_devtools_updates(&self) -> bool {
        return self.devtools_wants_updates.get();
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
        ReflowQueryType::HitTestQuery(_n, _o) => "\tHitTestQuery",
        ReflowQueryType::NodeGeometryQuery(_n) => "\tNodeGeometryQuery",
        ReflowQueryType::NodeScrollGeometryQuery(_n) => "\tNodeScrollGeometryQuery",
        ReflowQueryType::ResolvedStyleQuery(_, _, _) => "\tResolvedStyleQuery",
        ReflowQueryType::OffsetParentQuery(_n) => "\tOffsetParentQuery",
        ReflowQueryType::MarginStyleQuery(_n) => "\tMarginStyleQuery",
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
    });

    println!("{}", debug_msg);
}
