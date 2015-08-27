/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::callback::ExceptionHandling;
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use dom::bindings::codegen::Bindings::EventHandlerBinding::{OnErrorEventHandlerNonNull, EventHandlerNonNull};
use dom::bindings::codegen::Bindings::FunctionBinding::Function;
use dom::bindings::codegen::Bindings::WindowBinding::{self, WindowMethods, FrameRequestCallback};
use dom::bindings::codegen::InheritTypes::{NodeCast, ElementCast, EventTargetCast, WindowDerived};
use dom::bindings::error::Error::InvalidCharacter;
use dom::bindings::error::{report_pending_exception, Fallible};
use dom::bindings::global::GlobalRef;
use dom::bindings::global::global_object_for_js_object;
use dom::bindings::js::RootedReference;
use dom::bindings::js::{JS, Root, MutNullableHeap};
use dom::bindings::num::Finite;
use dom::bindings::utils::{GlobalStaticData, Reflectable, WindowProxyHandler};
use dom::browsercontext::BrowsingContext;
use dom::console::Console;
use dom::crypto::Crypto;
use dom::cssstyledeclaration::{CSSModificationAccess, CSSStyleDeclaration};
use dom::document::{Document, DocumentHelpers};
use dom::element::Element;
use dom::eventtarget::{EventTarget, EventTargetHelpers, EventTargetTypeId};
use dom::location::Location;
use dom::navigator::Navigator;
use dom::node::{window_from_node, TrustedNodeAddress, NodeHelpers, from_untrusted_node_address};
use dom::performance::Performance;
use dom::screen::Screen;
use dom::storage::Storage;
use layout_interface::{ContentBoxResponse, ContentBoxesResponse, ResolvedStyleResponse, ScriptReflow};
use layout_interface::{ReflowGoal, ReflowQueryType, LayoutRPC, LayoutChan, Reflow, Msg};
use page::Page;
use script_task::{SendableMainThreadScriptChan, MainThreadScriptChan};
use script_task::{TimerSource, ScriptChan, ScriptPort, MainThreadScriptMsg};
use script_traits::ConstellationControlMsg;
use timers::{IsInterval, TimerId, TimerManager, TimerCallback};
use webdriver_handlers::jsval_to_webdriver;

use devtools_traits::{ScriptToDevtoolsControlMsg, TimelineMarker, TimelineMarkerType};
use devtools_traits::{TracingMetadata};
use msg::compositor_msg::ScriptToCompositorMsg;
use msg::constellation_msg::{LoadData, PipelineId, SubpageId, ConstellationChan, WindowSizeData, WorkerId};
use msg::webdriver_msg::{WebDriverJSError, WebDriverJSResult};
use net_traits::ResourceTask;
use net_traits::image_cache_task::{ImageCacheChan, ImageCacheTask};
use net_traits::storage_task::{StorageTask, StorageType};
use profile_traits::mem;
use string_cache::Atom;
use util::geometry::{self, Au, MAX_RECT};
use util::str::{DOMString, HTML_SPACE_CHARACTERS};
use util::{breakpoint, opts};

use euclid::{Point2D, Rect, Size2D};
use ipc_channel::ipc::IpcSender;
use js::jsapi::{Evaluate2, MutableHandleValue};
use js::jsapi::{JSContext, HandleValue};
use js::jsapi::{JS_GC, JS_GetRuntime, JSAutoCompartment, JSAutoRequest};
use js::rust::CompileOptionsWrapper;
use js::rust::Runtime;
use selectors::parser::PseudoElement;
use url::Url;

use libc;
use rustc_serialize::base64::{FromBase64, ToBase64, STANDARD};
use std::ascii::AsciiExt;
use std::borrow::ToOwned;
use std::cell::{Cell, Ref, RefCell};
use std::collections::HashSet;
use std::default::Default;
use std::ffi::CString;
use std::io::{stdout, stderr, Write};
use std::mem as std_mem;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::mpsc::TryRecvError::{Empty, Disconnected};
use std::sync::mpsc::{channel, Receiver, Sender};
use time;

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
    ImageLoaded,
    RequestAnimationFrame,
}

#[dom_struct]
pub struct Window {
    eventtarget: EventTarget,
    #[ignore_heap_size_of = "trait objects are hard"]
    script_chan: MainThreadScriptChan,
    #[ignore_heap_size_of = "channels are hard"]
    control_chan: Sender<ConstellationControlMsg>,
    console: MutNullableHeap<JS<Console>>,
    crypto: MutNullableHeap<JS<Crypto>>,
    navigator: MutNullableHeap<JS<Navigator>>,
    #[ignore_heap_size_of = "channels are hard"]
    image_cache_task: ImageCacheTask,
    #[ignore_heap_size_of = "channels are hard"]
    image_cache_chan: ImageCacheChan,
    #[ignore_heap_size_of = "TODO(#6911) newtypes containing unmeasurable types are hard"]
    compositor: IpcSender<ScriptToCompositorMsg>,
    browsing_context: DOMRefCell<Option<BrowsingContext>>,
    page: Rc<Page>,
    performance: MutNullableHeap<JS<Performance>>,
    navigation_start: u64,
    navigation_start_precise: f64,
    screen: MutNullableHeap<JS<Screen>>,
    session_storage: MutNullableHeap<JS<Storage>>,
    local_storage: MutNullableHeap<JS<Storage>>,
    timers: TimerManager,

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
    devtools_markers: RefCell<HashSet<TimelineMarkerType>>,
    #[ignore_heap_size_of = "channels are hard"]
    devtools_marker_sender: RefCell<Option<IpcSender<TimelineMarker>>>,

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

    /// Unique id for last reflow request; used for confirming completion reply.
    last_reflow_id: Cell<u32>,

    /// Global static data related to the DOM.
    dom_static: GlobalStaticData,

    /// The JavaScript runtime.
    #[ignore_heap_size_of = "Rc<T> is hard"]
    js_runtime: DOMRefCell<Option<Rc<Runtime>>>,

    /// A handle for communicating messages to the layout task.
    #[ignore_heap_size_of = "channels are hard"]
    layout_chan: LayoutChan,

    /// A handle to perform RPC calls into the layout, quickly.
    #[ignore_heap_size_of = "trait objects are hard"]
    layout_rpc: Box<LayoutRPC + 'static>,

    /// The port that we will use to join layout. If this is `None`, then layout is not running.
    #[ignore_heap_size_of = "channels are hard"]
    layout_join_port: DOMRefCell<Option<Receiver<()>>>,

    /// The current size of the window, in pixels.
    window_size: Cell<Option<WindowSizeData>>,

    /// Associated resource task for use by DOM objects like XMLHttpRequest
    #[ignore_heap_size_of = "channels are hard"]
    resource_task: Arc<ResourceTask>,

    /// A handle for communicating messages to the storage task.
    #[ignore_heap_size_of = "channels are hard"]
    storage_task: StorageTask,

    /// A handle for communicating messages to the constellation task.
    #[ignore_heap_size_of = "channels are hard"]
    constellation_chan: ConstellationChan,

    /// Pending scroll to fragment event, if any
    fragment_name: DOMRefCell<Option<String>>,

    /// An enlarged rectangle around the page contents visible in the viewport, used
    /// to prevent creating display list items for content that is far away from the viewport.
    page_clip_rect: Cell<Rect<Au>>,

    /// A counter of the number of pending reflows for this window.
    pending_reflow_count: Cell<u32>,

    /// A channel for communicating results of async scripts back to the webdriver server
    #[ignore_heap_size_of = "channels are hard"]
    webdriver_script_chan: RefCell<Option<IpcSender<WebDriverJSResult>>>,

    /// The current state of the window object
    current_state: Cell<WindowState>,
}

impl Window {
    #[allow(unsafe_code)]
    pub fn clear_js_runtime_for_script_deallocation(&self) {
        unsafe {
            *self.js_runtime.borrow_for_script_deallocation() = None;
            *self.browsing_context.borrow_for_script_deallocation() = None;
            self.current_state.set(WindowState::Zombie);
        }
    }

    pub fn get_cx(&self) -> *mut JSContext {
        self.js_runtime.borrow().as_ref().unwrap().cx()
    }

    pub fn script_chan(&self) -> Box<ScriptChan + Send> {
        self.script_chan.clone()
    }

    pub fn main_thread_script_chan(&self) -> &Sender<MainThreadScriptMsg> {
        let MainThreadScriptChan(ref sender) = self.script_chan;
        sender
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

    pub fn image_cache_task(&self) -> &ImageCacheTask {
        &self.image_cache_task
    }

    pub fn compositor(&self) -> &IpcSender<ScriptToCompositorMsg> {
        &self.compositor
    }

    pub fn browsing_context(&self) -> Ref<Option<BrowsingContext>> {
        self.browsing_context.borrow()
    }

    pub fn page(&self) -> &Page {
        &*self.page
    }

    pub fn storage_task(&self) -> StorageTask {
        self.storage_task.clone()
    }
}

// https://www.whatwg.org/html/#atob
pub fn base64_btoa(input: DOMString) -> Fallible<DOMString> {
    // "The btoa() method must throw an InvalidCharacterError exception if
    //  the method's first argument contains any character whose code point
    //  is greater than U+00FF."
    if input.chars().any(|c: char| c > '\u{FF}') {
        Err(InvalidCharacter)
    } else {
        // "Otherwise, the user agent must convert that argument to a
        //  sequence of octets whose nth octet is the eight-bit
        //  representation of the code point of the nth character of
        //  the argument,"
        let octets = input.chars().map(|c: char| c as u8).collect::<Vec<u8>>();

        // "and then must apply the base64 algorithm to that sequence of
        //  octets, and return the result. [RFC4648]"
        Ok(octets.to_base64(STANDARD))
    }
}

// https://www.whatwg.org/html/#atob
pub fn base64_atob(input: DOMString) -> Fallible<DOMString> {
    // "Remove all space characters from input."
    // serialize::base64::from_base64 ignores \r and \n,
    // but it treats the other space characters as
    // invalid input.
    fn is_html_space(c: char) -> bool {
        HTML_SPACE_CHARACTERS.iter().any(|&m| m == c)
    }
    let without_spaces = input.chars()
        .filter(|&c| ! is_html_space(c))
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
        return Err(InvalidCharacter)
    }

    // "If input contains a character that is not in the following list of
    //  characters and character ranges, throw an InvalidCharacterError
    //  exception and abort these steps:
    //
    //  U+002B PLUS SIGN (+)
    //  U+002F SOLIDUS (/)
    //  Alphanumeric ASCII characters"
    if input.chars().any(|c| c != '+' && c != '/' && !c.is_alphanumeric()) {
        return Err(InvalidCharacter)
    }

    match input.from_base64() {
        Ok(data) => Ok(data.iter().map(|&b| b as char).collect::<String>()),
        Err(..) => Err(InvalidCharacter)
    }
}

impl<'a> WindowMethods for &'a Window {
    // https://html.spec.whatwg.org/#dom-alert
    fn Alert(self, s: DOMString) {
        // Right now, just print to the console
        // Ensure that stderr doesn't trample through the alert() we use to
        // communicate test results.
        let stderr = stderr();
        let mut stderr = stderr.lock();
        let stdout = stdout();
        let mut stdout = stdout.lock();
        writeln!(&mut stdout, "ALERT: {}", s).unwrap();
        stdout.flush().unwrap();
        stderr.flush().unwrap();
    }

    // https://html.spec.whatwg.org/multipage/#dom-window-close
    fn Close(self) {
        self.main_thread_script_chan().send(MainThreadScriptMsg::ExitWindow(self.id.clone())).unwrap();
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-0
    fn Document(self) -> Root<Document> {
        self.browsing_context().as_ref().unwrap().active_document()
    }

    // https://html.spec.whatwg.org/#dom-location
    fn Location(self) -> Root<Location> {
        self.Document().r().Location()
    }

    // https://html.spec.whatwg.org/#dom-sessionstorage
    fn SessionStorage(self) -> Root<Storage> {
        self.session_storage.or_init(|| Storage::new(&GlobalRef::Window(self), StorageType::Session))
    }

    // https://html.spec.whatwg.org/#dom-localstorage
    fn LocalStorage(self) -> Root<Storage> {
        self.local_storage.or_init(|| Storage::new(&GlobalRef::Window(self), StorageType::Local))
    }

    // https://developer.mozilla.org/en-US/docs/Web/API/Console
    fn Console(self) -> Root<Console> {
        self.console.or_init(|| Console::new(GlobalRef::Window(self)))
    }

    // https://dvcs.w3.org/hg/webcrypto-api/raw-file/tip/spec/Overview.html#dfn-GlobalCrypto
    fn Crypto(self) -> Root<Crypto> {
        self.crypto.or_init(|| Crypto::new(GlobalRef::Window(self)))
    }

    // https://html.spec.whatwg.org/#dom-frameelement
    fn GetFrameElement(self) -> Option<Root<Element>> {
        self.browsing_context().as_ref().unwrap().frame_element()
    }

    // https://html.spec.whatwg.org/#dom-navigator
    fn Navigator(self) -> Root<Navigator> {
        self.navigator.or_init(|| Navigator::new(self))
    }

    // https://html.spec.whatwg.org/#dom-windowtimers-settimeout
    fn SetTimeout(self, _cx: *mut JSContext, callback: Rc<Function>, timeout: i32, args: Vec<HandleValue>) -> i32 {
        self.timers.set_timeout_or_interval(TimerCallback::FunctionTimerCallback(callback),
                                            args,
                                            timeout,
                                            IsInterval::NonInterval,
                                            TimerSource::FromWindow(self.id.clone()),
                                            self.script_chan.clone())
    }

    // https://html.spec.whatwg.org/#dom-windowtimers-settimeout
    fn SetTimeout_(self, _cx: *mut JSContext, callback: DOMString, timeout: i32, args: Vec<HandleValue>) -> i32 {
        self.timers.set_timeout_or_interval(TimerCallback::StringTimerCallback(callback),
                                            args,
                                            timeout,
                                            IsInterval::NonInterval,
                                            TimerSource::FromWindow(self.id.clone()),
                                            self.script_chan.clone())
    }

    // https://html.spec.whatwg.org/#dom-windowtimers-cleartimeout
    fn ClearTimeout(self, handle: i32) {
        self.timers.clear_timeout_or_interval(handle);
    }

    // https://html.spec.whatwg.org/#dom-windowtimers-setinterval
    fn SetInterval(self, _cx: *mut JSContext, callback: Rc<Function>, timeout: i32, args: Vec<HandleValue>) -> i32 {
        self.timers.set_timeout_or_interval(TimerCallback::FunctionTimerCallback(callback),
                                            args,
                                            timeout,
                                            IsInterval::Interval,
                                            TimerSource::FromWindow(self.id.clone()),
                                            self.script_chan.clone())
    }

    // https://html.spec.whatwg.org/#dom-windowtimers-setinterval
    fn SetInterval_(self, _cx: *mut JSContext, callback: DOMString, timeout: i32, args: Vec<HandleValue>) -> i32 {
        self.timers.set_timeout_or_interval(TimerCallback::StringTimerCallback(callback),
                                            args,
                                            timeout,
                                            IsInterval::Interval,
                                            TimerSource::FromWindow(self.id.clone()),
                                            self.script_chan.clone())
    }

    // https://html.spec.whatwg.org/#dom-windowtimers-clearinterval
    fn ClearInterval(self, handle: i32) {
        self.ClearTimeout(handle);
    }

    // https://html.spec.whatwg.org/multipage/#dom-window
    fn Window(self) -> Root<Window> {
        Root::from_ref(self)
    }

    // https://html.spec.whatwg.org/multipage/#dom-self
    fn Self_(self) -> Root<Window> {
        self.Window()
    }

    // https://www.whatwg.org/html/#dom-frames
    fn Frames(self) -> Root<Window> {
        self.Window()
    }

    // https://html.spec.whatwg.org/multipage/#dom-parent
    fn Parent(self) -> Root<Window> {
        self.parent().unwrap_or(self.Window())
    }

    // https://html.spec.whatwg.org/multipage/#dom-top
    fn Top(self) -> Root<Window> {
        let mut window = self.Window();
        while let Some(parent) = window.parent() {
            window = parent;
        }
        window
    }

    // https://dvcs.w3.org/hg/webperf/raw-file/tip/specs/
    // NavigationTiming/Overview.html#sec-window.performance-attribute
    fn Performance(self) -> Root<Performance> {
        self.performance.or_init(|| {
            Performance::new(self, self.navigation_start,
                             self.navigation_start_precise)
        })
    }

    global_event_handlers!();
    event_handler!(unload, GetOnunload, SetOnunload);
    error_event_handler!(error, GetOnerror, SetOnerror);

    // https://developer.mozilla.org/en-US/docs/Web/API/Window/screen
    fn Screen(self) -> Root<Screen> {
        self.screen.or_init(|| Screen::new(self))
    }

    // https://html.spec.whatwg.org/multipage/#dom-windowbase64-btoa
    fn Btoa(self, btoa: DOMString) -> Fallible<DOMString> {
        base64_btoa(btoa)
    }

    // https://html.spec.whatwg.org/multipage/#dom-windowbase64-atob
    fn Atob(self, atob: DOMString) -> Fallible<DOMString> {
        base64_atob(atob)
    }

    /// https://html.spec.whatwg.org/multipage/#dom-window-requestanimationframe
    fn RequestAnimationFrame(self, callback: Rc<FrameRequestCallback>) -> i32 {
        let doc = self.Document();

        let callback  = move |now: f64| {
            // TODO: @jdm The spec says that any exceptions should be suppressed;
            // https://github.com/servo/servo/issues/6928
            let _ = callback.Call__(Finite::wrap(now), ExceptionHandling::Report);
        };

        doc.r().request_animation_frame(Box::new(callback))
    }

    /// https://html.spec.whatwg.org/multipage/#dom-window-cancelanimationframe
    fn CancelAnimationFrame(self, ident: i32) {
        let doc = self.Document();
        doc.r().cancel_animation_frame(ident);
    }

    // check-tidy: no specs after this line
    fn Debug(self, message: DOMString) {
        debug!("{}", message);
    }

    #[allow(unsafe_code)]
    fn Gc(self) {
        unsafe {
            JS_GC(JS_GetRuntime(self.get_cx()));
        }
    }

    fn Trap(self) {
        breakpoint();
    }

    fn WebdriverCallback(self, cx: *mut JSContext, val: HandleValue) {
        let rv = jsval_to_webdriver(cx, val);
        let opt_chan = self.webdriver_script_chan.borrow_mut().take();
        if let Some(chan) = opt_chan {
            chan.send(rv).unwrap();
        }
    }

    fn WebdriverTimeout(self) {
        let opt_chan = self.webdriver_script_chan.borrow_mut().take();
        if let Some(chan) = opt_chan {
            chan.send(Err(WebDriverJSError::Timeout)).unwrap();
        }
    }

    // https://drafts.csswg.org/cssom/#dom-window-getcomputedstyle
    fn GetComputedStyle(self,
                        element: &Element,
                        pseudo: Option<DOMString>) -> Root<CSSStyleDeclaration> {
        // Steps 1-4.
        let pseudo = match pseudo.map(|s| s.to_ascii_lowercase()) {
            Some(ref pseudo) if pseudo == ":before" || pseudo == "::before" =>
                Some(PseudoElement::Before),
            Some(ref pseudo) if pseudo == ":after" || pseudo == "::after" =>
                Some(PseudoElement::After),
            _ => None
        };

        // Step 5.
        CSSStyleDeclaration::new(self, element, pseudo, CSSModificationAccess::Readonly)
    }
}

pub trait WindowHelpers {
    fn clear_js_runtime(self);
    fn init_browsing_context(self, doc: &Document, frame_element: Option<&Element>);
    fn load_url(self, url: Url);
    fn handle_fire_timer(self, timer_id: TimerId);
    fn force_reflow(self, goal: ReflowGoal, query_type: ReflowQueryType, reason: ReflowReason);
    fn reflow(self, goal: ReflowGoal, query_type: ReflowQueryType, reason: ReflowReason);
    fn join_layout(self);
    fn layout(&self) -> &LayoutRPC;
    fn content_box_query(self, content_box_request: TrustedNodeAddress) -> Rect<Au>;
    fn content_boxes_query(self, content_boxes_request: TrustedNodeAddress) -> Vec<Rect<Au>>;
    fn client_rect_query(self, node_geometry_request: TrustedNodeAddress) -> Rect<i32>;
    fn resolved_style_query(self, element: TrustedNodeAddress,
                            pseudo: Option<PseudoElement>, property: &Atom) -> Option<String>;
    fn offset_parent_query(self, node: TrustedNodeAddress) -> (Option<Root<Element>>, Rect<Au>);
    fn handle_reflow_complete_msg(self, reflow_id: u32);
    fn set_fragment_name(self, fragment: Option<String>);
    fn steal_fragment_name(self) -> Option<String>;
    fn set_window_size(self, size: WindowSizeData);
    fn window_size(self) -> Option<WindowSizeData>;
    fn get_url(self) -> Url;
    fn resource_task(self) -> ResourceTask;
    fn mem_profiler_chan(self) -> mem::ProfilerChan;
    fn devtools_chan(self) -> Option<IpcSender<ScriptToDevtoolsControlMsg>>;
    fn layout_chan(self) -> LayoutChan;
    fn constellation_chan(self) -> ConstellationChan;
    fn windowproxy_handler(self) -> WindowProxyHandler;
    fn get_next_subpage_id(self) -> SubpageId;
    fn layout_is_idle(self) -> bool;
    fn get_pending_reflow_count(self) -> u32;
    fn add_pending_reflow(self);
    fn set_resize_event(self, event: WindowSizeData);
    fn steal_resize_event(self) -> Option<WindowSizeData>;
    fn set_page_clip_rect_with_new_viewport(self, viewport: Rect<f32>) -> bool;
    fn set_devtools_wants_updates(self, value: bool);
    fn IndexedGetter(self, _index: u32, _found: &mut bool) -> Option<Root<Window>>;
    fn thaw(self);
    fn freeze(self);
    fn need_emit_timeline_marker(self, timeline_type: TimelineMarkerType) -> bool;
    fn emit_timeline_marker(self, marker: TimelineMarker);
    fn set_devtools_timeline_marker(self,
                                    marker: TimelineMarkerType,
                                    reply: IpcSender<TimelineMarker>);
    fn drop_devtools_timeline_markers(self);
    fn set_webdriver_script_chan(self, chan: Option<IpcSender<WebDriverJSResult>>);
    fn is_alive(self) -> bool;
    fn parent(self) -> Option<Root<Window>>;
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
        let this = self.reflector().get_jsobject();
        let global = global_object_for_js_object(this.get());
        let cx = global.r().get_cx();
        let _ar = JSAutoRequest::new(cx);
        let globalhandle = global.r().reflector().get_jsobject();
        let code: Vec<u16> = code.utf16_units().collect();
        let filename = CString::new(filename).unwrap();

        let _ac = JSAutoCompartment::new(cx, globalhandle.get());
        let options = CompileOptionsWrapper::new(cx, filename.as_ptr(), 0);
        unsafe {
            if Evaluate2(cx, options.ptr, code.as_ptr() as *const i16,
                         code.len() as libc::size_t,
                         rval) == 0 {
                debug!("error evaluating JS string");
                report_pending_exception(cx, globalhandle.get());
            }
        }
    }
}

impl<'a> WindowHelpers for &'a Window {
    fn clear_js_runtime(self) {
        let document = self.Document();
        NodeCast::from_ref(document.r()).teardown();

        // The above code may not catch all DOM objects
        // (e.g. DOM objects removed from the tree that haven't
        // been collected yet). Forcing a GC here means that
        // those DOM objects will be able to call dispose()
        // to free their layout data before the layout task
        // exits. Without this, those remaining objects try to
        // send a message to free their layout data to the
        // layout task when the script task is dropped,
        // which causes a panic!
        self.Gc();

        self.current_state.set(WindowState::Zombie);
        *self.js_runtime.borrow_mut() = None;
        *self.browsing_context.borrow_mut() = None;
    }

    /// Reflows the page unconditionally. This method will wait for the layout thread to complete
    /// (but see the `TODO` below). If there is no window size yet, the page is presumed invisible
    /// and no reflow is performed.
    ///
    /// TODO(pcwalton): Only wait for style recalc, since we have off-main-thread layout.
    fn force_reflow(self, goal: ReflowGoal, query_type: ReflowQueryType, reason: ReflowReason) {
        let document = self.Document();
        let root = document.r().GetDocumentElement();
        let root = match root.r() {
            Some(root) => root,
            None => return,
        };
        let root = NodeCast::from_ref(root);

        let window_size = match self.window_size.get() {
            Some(window_size) => window_size,
            None => return,
        };

        debug!("script: performing reflow for goal {:?} reason {:?}", goal, reason);

        if self.need_emit_timeline_marker(TimelineMarkerType::Reflow) {
            let marker = TimelineMarker::new("Reflow".to_owned(), TracingMetadata::IntervalStart);
            self.emit_timeline_marker(marker);
        }

        // Layout will let us know when it's done.
        let (join_chan, join_port) = channel();

        {
            let mut layout_join_port = self.layout_join_port.borrow_mut();
            *layout_join_port = Some(join_port);
        }

        let last_reflow_id = &self.last_reflow_id;
        last_reflow_id.set(last_reflow_id.get() + 1);

        // On debug mode, print the reflow event information.
        if opts::get().relayout_event {
            debug_reflow_events(&goal, &query_type, &reason);
        }

        // Send new document and relevant styles to layout.
        let reflow = box ScriptReflow {
            reflow_info: Reflow {
                goal: goal,
                page_clip_rect: self.page_clip_rect.get(),
            },
            document_root: root.to_trusted_node_address(),
            window_size: window_size,
            script_chan: self.control_chan.clone(),
            script_join_chan: join_chan,
            id: last_reflow_id.get(),
            query_type: query_type,
        };

        let LayoutChan(ref chan) = self.layout_chan;
        chan.send(Msg::Reflow(reflow)).unwrap();

        debug!("script: layout forked");

        self.join_layout();

        self.pending_reflow_count.set(0);

        if self.need_emit_timeline_marker(TimelineMarkerType::Reflow) {
            let marker = TimelineMarker::new("Reflow".to_owned(), TracingMetadata::IntervalEnd);
            self.emit_timeline_marker(marker);
        }
    }

    /// Reflows the page if it's possible to do so and the page is dirty. This method will wait
    /// for the layout thread to complete (but see the `TODO` below). If there is no window size
    /// yet, the page is presumed invisible and no reflow is performed.
    ///
    /// TODO(pcwalton): Only wait for style recalc, since we have off-main-thread layout.
    fn reflow(self, goal: ReflowGoal, query_type: ReflowQueryType, reason: ReflowReason) {
        let document = self.Document();
        let root = document.r().GetDocumentElement();
        let root = match root.r() {
            Some(root) => root,
            None => return,
        };

        let root = NodeCast::from_ref(root);
        if query_type == ReflowQueryType::NoQuery && !root.get_has_dirty_descendants() {
            debug!("root has no dirty descendants; avoiding reflow (reason {:?})", reason);
            return
        }

        self.force_reflow(goal, query_type, reason)
    }

    // FIXME(cgaebel): join_layout is racey. What if the compositor triggers a
    // reflow between the "join complete" message and returning from this
    // function?

    /// Sends a ping to layout and waits for the response. The response will arrive when the
    /// layout task has finished any pending request messages.
    fn join_layout(self) {
        let mut layout_join_port = self.layout_join_port.borrow_mut();
        if let Some(join_port) = std_mem::replace(&mut *layout_join_port, None) {
            match join_port.try_recv() {
                Err(Empty) => {
                    info!("script: waiting on layout");
                    join_port.recv().unwrap();
                }
                Ok(_) => {}
                Err(Disconnected) => {
                    panic!("Layout task failed while script was waiting for a result.");
                }
            }

            debug!("script: layout joined")
        }
    }

    fn layout(&self) -> &LayoutRPC {
        &*self.layout_rpc
    }

    fn content_box_query(self, content_box_request: TrustedNodeAddress) -> Rect<Au> {
        self.reflow(ReflowGoal::ForScriptQuery,
                    ReflowQueryType::ContentBoxQuery(content_box_request),
                    ReflowReason::Query);
        self.join_layout(); //FIXME: is this necessary, or is layout_rpc's mutex good enough?
        let ContentBoxResponse(rect) = self.layout_rpc.content_box();
        rect
    }

    fn content_boxes_query(self, content_boxes_request: TrustedNodeAddress) -> Vec<Rect<Au>> {
        self.reflow(ReflowGoal::ForScriptQuery,
                    ReflowQueryType::ContentBoxesQuery(content_boxes_request),
                    ReflowReason::Query);
        self.join_layout(); //FIXME: is this necessary, or is layout_rpc's mutex good enough?
        let ContentBoxesResponse(rects) = self.layout_rpc.content_boxes();
        rects
    }

    fn client_rect_query(self, node_geometry_request: TrustedNodeAddress) -> Rect<i32> {
        self.reflow(ReflowGoal::ForScriptQuery,
                    ReflowQueryType::NodeGeometryQuery(node_geometry_request),
                    ReflowReason::Query);
        self.layout_rpc.node_geometry().client_rect
    }

    fn resolved_style_query(self,
                            element: TrustedNodeAddress,
                            pseudo: Option<PseudoElement>,
                            property: &Atom) -> Option<String> {
        self.reflow(ReflowGoal::ForScriptQuery,
                    ReflowQueryType::ResolvedStyleQuery(element, pseudo, property.clone()),
                    ReflowReason::Query);
        let ResolvedStyleResponse(resolved) = self.layout_rpc.resolved_style();
        resolved
    }

    fn offset_parent_query(self, node: TrustedNodeAddress) -> (Option<Root<Element>>, Rect<Au>) {
        self.reflow(ReflowGoal::ForScriptQuery,
                    ReflowQueryType::OffsetParentQuery(node),
                    ReflowReason::Query);
        let response = self.layout_rpc.offset_parent();
        let js_runtime = self.js_runtime.borrow();
        let js_runtime = js_runtime.as_ref().unwrap();
        let element = match response.node_address {
            Some(parent_node_address) => {
                let node = from_untrusted_node_address(js_runtime.rt(),
                                                       parent_node_address);
                let element = ElementCast::to_ref(node.r());
                element.map(Root::from_ref)
            }
            None => {
                None
            }
        };
        (element, response.rect)
    }

    fn handle_reflow_complete_msg(self, reflow_id: u32) {
        let last_reflow_id = self.last_reflow_id.get();
        if last_reflow_id == reflow_id {
            *self.layout_join_port.borrow_mut() = None;
        }
    }

    fn init_browsing_context(self, doc: &Document, frame_element: Option<&Element>) {
        let mut browsing_context = self.browsing_context.borrow_mut();
        *browsing_context = Some(BrowsingContext::new(doc, frame_element));
        (*browsing_context).as_mut().unwrap().create_window_proxy();
    }

    /// Commence a new URL load which will either replace this window or scroll to a fragment.
    fn load_url(self, url: Url) {
        self.main_thread_script_chan().send(
            MainThreadScriptMsg::Navigate(self.id, LoadData::new(url))).unwrap();
    }

    fn handle_fire_timer(self, timer_id: TimerId) {
        self.timers.fire_timer(timer_id, self);
        self.reflow(ReflowGoal::ForDisplay, ReflowQueryType::NoQuery, ReflowReason::Timer);
    }

    fn set_fragment_name(self, fragment: Option<String>) {
        *self.fragment_name.borrow_mut() = fragment;
    }

    fn steal_fragment_name(self) -> Option<String> {
        self.fragment_name.borrow_mut().take()
    }

    fn set_window_size(self, size: WindowSizeData) {
        self.window_size.set(Some(size));
    }

    fn window_size(self) -> Option<WindowSizeData> {
        self.window_size.get()
    }

    fn get_url(self) -> Url {
        let doc = self.Document();
        doc.r().url()
    }

    fn resource_task(self) -> ResourceTask {
        (*self.resource_task).clone()
    }

    fn mem_profiler_chan(self) -> mem::ProfilerChan {
        self.mem_profiler_chan.clone()
    }

    fn devtools_chan(self) -> Option<IpcSender<ScriptToDevtoolsControlMsg>> {
        self.devtools_chan.clone()
    }

    fn layout_chan(self) -> LayoutChan {
        self.layout_chan.clone()
    }

    fn constellation_chan(self) -> ConstellationChan {
        self.constellation_chan.clone()
    }

    fn windowproxy_handler(self) -> WindowProxyHandler {
        WindowProxyHandler(self.dom_static.windowproxy_handler.0)
    }

    fn get_next_subpage_id(self) -> SubpageId {
        let subpage_id = self.next_subpage_id.get();
        let SubpageId(id_num) = subpage_id;
        self.next_subpage_id.set(SubpageId(id_num + 1));
        subpage_id
    }

    fn layout_is_idle(self) -> bool {
        self.layout_join_port.borrow().is_none()
    }

    fn get_pending_reflow_count(self) -> u32 {
        self.pending_reflow_count.get()
    }

    fn add_pending_reflow(self) {
        self.pending_reflow_count.set(self.pending_reflow_count.get() + 1);
    }

    fn set_resize_event(self, event: WindowSizeData) {
        self.resize_event.set(Some(event));
    }

    fn steal_resize_event(self) -> Option<WindowSizeData> {
        let event = self.resize_event.get();
        self.resize_event.set(None);
        event
    }

    fn set_page_clip_rect_with_new_viewport(self, viewport: Rect<f32>) -> bool {
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

    fn set_devtools_wants_updates(self, value: bool) {
        self.devtools_wants_updates.set(value);
    }

    // https://html.spec.whatwg.org/multipage/#accessing-other-browsing-contexts
    fn IndexedGetter(self, _index: u32, _found: &mut bool) -> Option<Root<Window>> {
        None
    }

    fn thaw(self) {
        self.timers.resume();

        // Push the document title to the compositor since we are
        // activating this document due to a navigation.
        let document = self.Document();
        document.r().title_changed();
    }

    fn freeze(self) {
        self.timers.suspend();
    }

    fn need_emit_timeline_marker(self, timeline_type: TimelineMarkerType) -> bool {
        let markers = self.devtools_markers.borrow();
        markers.contains(&timeline_type)
    }

    fn emit_timeline_marker(self, marker: TimelineMarker) {
        let sender = self.devtools_marker_sender.borrow();
        let sender = sender.as_ref().expect("There is no marker sender");
        sender.send(marker).unwrap();
    }

    fn set_devtools_timeline_marker(self,
                                    marker: TimelineMarkerType,
                                    reply: IpcSender<TimelineMarker>) {
        *self.devtools_marker_sender.borrow_mut() = Some(reply);
        self.devtools_markers.borrow_mut().insert(marker);
    }

    fn drop_devtools_timeline_markers(self) {
        self.devtools_markers.borrow_mut().clear();
        *self.devtools_marker_sender.borrow_mut() = None;
    }

    fn set_webdriver_script_chan(self, chan: Option<IpcSender<WebDriverJSResult>>) {
        *self.webdriver_script_chan.borrow_mut() = chan;
    }

    fn is_alive(self) -> bool {
        self.current_state.get() == WindowState::Alive
    }

    fn parent(self) -> Option<Root<Window>> {
        let browsing_context = self.browsing_context();
        let browsing_context = browsing_context.as_ref().unwrap();

        browsing_context.frame_element().map(|frame_element| {
            let window = window_from_node(frame_element.r());
            // FIXME(https://github.com/rust-lang/rust/issues/23338)
            let r = window.r();
            let context = r.browsing_context();
            context.as_ref().unwrap().active_window()
        })
    }
}

impl Window {
    pub fn new(runtime: Rc<Runtime>,
               page: Rc<Page>,
               script_chan: MainThreadScriptChan,
               image_cache_chan: ImageCacheChan,
               control_chan: Sender<ConstellationControlMsg>,
               compositor: IpcSender<ScriptToCompositorMsg>,
               image_cache_task: ImageCacheTask,
               resource_task: Arc<ResourceTask>,
               storage_task: StorageTask,
               mem_profiler_chan: mem::ProfilerChan,
               devtools_chan: Option<IpcSender<ScriptToDevtoolsControlMsg>>,
               constellation_chan: ConstellationChan,
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

        let win = box Window {
            eventtarget: EventTarget::new_inherited(EventTargetTypeId::Window),
            script_chan: script_chan,
            image_cache_chan: image_cache_chan,
            control_chan: control_chan,
            console: Default::default(),
            crypto: Default::default(),
            compositor: compositor,
            page: page,
            navigator: Default::default(),
            image_cache_task: image_cache_task,
            mem_profiler_chan: mem_profiler_chan,
            devtools_chan: devtools_chan,
            browsing_context: DOMRefCell::new(None),
            performance: Default::default(),
            navigation_start: time::get_time().sec as u64,
            navigation_start_precise: time::precise_time_ns() as f64,
            screen: Default::default(),
            session_storage: Default::default(),
            local_storage: Default::default(),
            timers: TimerManager::new(),
            next_worker_id: Cell::new(WorkerId(0)),
            id: id,
            parent_info: parent_info,
            dom_static: GlobalStaticData::new(),
            js_runtime: DOMRefCell::new(Some(runtime.clone())),
            resource_task: resource_task,
            storage_task: storage_task,
            constellation_chan: constellation_chan,
            page_clip_rect: Cell::new(MAX_RECT),
            fragment_name: DOMRefCell::new(None),
            last_reflow_id: Cell::new(0),
            resize_event: Cell::new(None),
            next_subpage_id: Cell::new(SubpageId(0)),
            layout_chan: layout_chan,
            layout_rpc: layout_rpc,
            layout_join_port: DOMRefCell::new(None),
            window_size: Cell::new(window_size),
            pending_reflow_count: Cell::new(0),
            current_state: Cell::new(WindowState::Alive),

            devtools_marker_sender: RefCell::new(None),
            devtools_markers: RefCell::new(HashSet::new()),
            devtools_wants_updates: Cell::new(false),
            webdriver_script_chan: RefCell::new(None),
        };

        WindowBinding::Wrap(runtime.cx(), win)
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

fn debug_reflow_events(goal: &ReflowGoal, query_type: &ReflowQueryType, reason: &ReflowReason) {
    let mut debug_msg = "****".to_owned();
    debug_msg.push_str(match *goal {
        ReflowGoal::ForDisplay => "\tForDisplay",
        ReflowGoal::ForScriptQuery => "\tForScriptQuery",
    });

    debug_msg.push_str(match *query_type {
        ReflowQueryType::NoQuery => "\tNoQuery",
        ReflowQueryType::ContentBoxQuery(_n) => "\tContentBoxQuery",
        ReflowQueryType::ContentBoxesQuery(_n) => "\tContentBoxesQuery",
        ReflowQueryType::NodeGeometryQuery(_n) => "\tNodeGeometryQuery",
        ReflowQueryType::ResolvedStyleQuery(_, _, _) => "\tResolvedStyleQuery",
        ReflowQueryType::OffsetParentQuery(_n) => "\tOffsetParentQuery",
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
        ReflowReason::ImageLoaded => "\tImageLoaded",
        ReflowReason::RequestAnimationFrame => "\tRequestAnimationFrame",
    });

    println!("{}", debug_msg);
}

impl WindowDerived for EventTarget {
    fn is_window(&self) -> bool {
        self.type_id() == &EventTargetTypeId::Window
    }
}
