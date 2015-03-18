/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::EventHandlerBinding::{OnErrorEventHandlerNonNull, EventHandlerNonNull};
use dom::bindings::codegen::Bindings::FunctionBinding::Function;
use dom::bindings::codegen::Bindings::WindowBinding;
use dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use dom::bindings::codegen::InheritTypes::{NodeCast, EventTargetCast};
use dom::bindings::global::global_object_for_js_object;
use dom::bindings::error::{report_pending_exception, Fallible};
use dom::bindings::error::Error::InvalidCharacter;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{MutNullableJS, JSRef, Temporary, OptionalRootable, RootedReference};
use dom::bindings::utils::{GlobalStaticData, Reflectable, WindowProxyHandler};
use dom::browsercontext::BrowserContext;
use dom::console::Console;
use dom::document::{Document, DocumentHelpers};
use dom::element::Element;
use dom::eventtarget::{EventTarget, EventTargetHelpers, EventTargetTypeId};
use dom::location::Location;
use dom::navigator::Navigator;
use dom::node::{window_from_node, Node, TrustedNodeAddress, NodeHelpers};
use dom::performance::Performance;
use dom::screen::Screen;
use dom::storage::Storage;
use layout_interface::{ReflowGoal, ReflowQueryType, LayoutRPC, LayoutChan, Reflow, Msg};
use layout_interface::{ContentBoxResponse, ContentBoxesResponse};
use page::Page;
use script_task::{TimerSource, ScriptChan};
use script_task::ScriptMsg;
use script_traits::ScriptControlChan;
use timers::{IsInterval, TimerId, TimerManager, TimerCallback};

use devtools_traits::DevtoolsControlChan;
use msg::compositor_msg::ScriptListener;
use msg::constellation_msg::{LoadData, PipelineId, SubpageId, ConstellationChan, WindowSizeData};
use net::image_cache_task::ImageCacheTask;
use net::resource_task::ResourceTask;
use net::storage_task::{StorageTask, StorageType};
use util::geometry::{self, Au, MAX_RECT};
use util::opts;
use util::str::{DOMString,HTML_SPACE_CHARACTERS};

use geom::{Point2D, Rect, Size2D};
use js::jsapi::JS_EvaluateUCScript;
use js::jsapi::JSContext;
use js::jsapi::{JS_GC, JS_GetRuntime};
use js::jsval::{JSVal, UndefinedValue};
use js::rust::{Cx, with_compartment};
use url::{Url, UrlParser};

use libc;
use rustc_serialize::base64::{FromBase64, ToBase64, STANDARD};
use std::cell::{Cell, Ref, RefMut};
use std::default::Default;
use std::ffi::CString;
use std::mem;
use std::num::Float;
use std::rc::Rc;
use std::sync::mpsc::{channel, Receiver};
use std::sync::mpsc::TryRecvError::{Empty, Disconnected};
use time;

/// Extra information concerning the reason for reflowing.
pub enum ReflowReason {
    CachedPageNeededReflow,
    FirstLoad,
    KeyEvent,
    MouseEvent,
    Query,
    ReceivedReflowEvent,
    Timer,
    Viewport,
    WindowResize,
    DOMContentLoaded,
    DocumentLoaded,
}

#[dom_struct]
pub struct Window {
    eventtarget: EventTarget,
    script_chan: Box<ScriptChan+Send>,
    control_chan: ScriptControlChan,
    console: MutNullableJS<Console>,
    navigator: MutNullableJS<Navigator>,
    image_cache_task: ImageCacheTask,
    compositor: DOMRefCell<Box<ScriptListener+'static>>,
    browser_context: DOMRefCell<Option<BrowserContext>>,
    page: Rc<Page>,
    performance: MutNullableJS<Performance>,
    navigation_start: u64,
    navigation_start_precise: f64,
    screen: MutNullableJS<Screen>,
    session_storage: MutNullableJS<Storage>,
    local_storage: MutNullableJS<Storage>,
    timers: TimerManager,

    /// For providing instructions to an optional devtools server.
    devtools_chan: Option<DevtoolsControlChan>,

    /// A flag to indicate whether the developer tools have requested live updates of
    /// page changes.
    devtools_wants_updates: Cell<bool>,

    next_subpage_id: Cell<SubpageId>,

    /// Pending resize event, if any.
    resize_event: Cell<Option<WindowSizeData>>,

    /// Pipeline id associated with this page.
    id: PipelineId,

    /// Subpage id associated with this page, if any.
    subpage_id: Option<SubpageId>,

    /// Unique id for last reflow request; used for confirming completion reply.
    last_reflow_id: Cell<uint>,

    /// Global static data related to the DOM.
    dom_static: GlobalStaticData,

    /// The JavaScript context.
    js_context: DOMRefCell<Option<Rc<Cx>>>,

    /// A handle for communicating messages to the layout task.
    layout_chan: LayoutChan,

    /// A handle to perform RPC calls into the layout, quickly.
    layout_rpc: Box<LayoutRPC+'static>,

    /// The port that we will use to join layout. If this is `None`, then layout is not running.
    layout_join_port: DOMRefCell<Option<Receiver<()>>>,

    /// The current size of the window, in pixels.
    window_size: Cell<Option<WindowSizeData>>,

    /// Associated resource task for use by DOM objects like XMLHttpRequest
    resource_task: ResourceTask,

    /// A handle for communicating messages to the storage task.
    storage_task: StorageTask,

    /// A handle for communicating messages to the constellation task.
    constellation_chan: ConstellationChan,

    /// Pending scroll to fragment event, if any
    fragment_name: DOMRefCell<Option<String>>,

    /// An enlarged rectangle around the page contents visible in the viewport, used
    /// to prevent creating display list items for content that is far away from the viewport.
    page_clip_rect: Cell<Rect<Au>>,
}

impl Window {
    pub fn get_cx(&self) -> *mut JSContext {
        self.js_context.borrow().as_ref().unwrap().ptr
    }

    pub fn script_chan(&self) -> Box<ScriptChan+Send> {
        self.script_chan.clone()
    }

    pub fn pipeline(&self) -> PipelineId {
        self.id.clone()
    }

    pub fn subpage(&self) -> Option<SubpageId> {
        self.subpage_id.clone()
    }

    pub fn control_chan<'a>(&'a self) -> &'a ScriptControlChan {
        &self.control_chan
    }

    pub fn image_cache_task<'a>(&'a self) -> &'a ImageCacheTask {
        &self.image_cache_task
    }

    pub fn compositor<'a>(&'a self) -> RefMut<'a, Box<ScriptListener+'static>> {
        self.compositor.borrow_mut()
    }

    pub fn browser_context<'a>(&'a self) -> Ref<'a, Option<BrowserContext>> {
        self.browser_context.borrow()
    }

    pub fn page<'a>(&'a self) -> &'a Page {
        &*self.page
    }

    pub fn storage_task(&self) -> StorageTask {
        self.storage_task.clone()
    }
}

// http://www.whatwg.org/html/#atob
pub fn base64_btoa(btoa: DOMString) -> Fallible<DOMString> {
    let input = btoa.as_slice();
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
        Ok(octets.as_slice().to_base64(STANDARD))
    }
}

// http://www.whatwg.org/html/#atob
pub fn base64_atob(atob: DOMString) -> Fallible<DOMString> {
    // "Let input be the string being parsed."
    let input = atob.as_slice();

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
    let mut input = without_spaces.as_slice();

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
    if input.chars()
        .find(|&c| !(c == '+' || c == '/' || c.is_alphanumeric()))
        .is_some() {
            return Err(InvalidCharacter)
        }

    match input.from_base64() {
        Ok(data) => Ok(data.iter().map(|&b| b as char).collect::<String>()),
        Err(..) => Err(InvalidCharacter)
    }
}

impl<'a> WindowMethods for JSRef<'a, Window> {
    fn Alert(self, s: DOMString) {
        // Right now, just print to the console
        println!("ALERT: {}", s);
    }

    fn Close(self) {
        self.script_chan.send(ScriptMsg::ExitWindow(self.id.clone())).unwrap();
    }

    fn Document(self) -> Temporary<Document> {
        // FIXME(https://github.com/rust-lang/rust/issues/23338)
        let context = self.browser_context();
        context.as_ref().unwrap().active_document()
    }

    fn Location(self) -> Temporary<Location> {
        self.Document().root().r().Location()
    }

    fn SessionStorage(self) -> Temporary<Storage> {
        self.session_storage.or_init(|| Storage::new(&GlobalRef::Window(self), StorageType::Session))
    }

    fn LocalStorage(self) -> Temporary<Storage> {
        self.local_storage.or_init(|| Storage::new(&GlobalRef::Window(self), StorageType::Local))
    }

    fn Console(self) -> Temporary<Console> {
        self.console.or_init(|| Console::new(GlobalRef::Window(self)))
    }

    fn GetFrameElement(self) -> Option<Temporary<Element>> {
        // FIXME(https://github.com/rust-lang/rust/issues/23338)
        let context = self.browser_context();
        context.as_ref().unwrap().frame_element()
    }

    fn Navigator(self) -> Temporary<Navigator> {
        self.navigator.or_init(|| Navigator::new(self))
    }

    fn SetTimeout(self, _cx: *mut JSContext, callback: Function, timeout: i32, args: Vec<JSVal>) -> i32 {
        self.timers.set_timeout_or_interval(TimerCallback::FunctionTimerCallback(callback),
                                            args,
                                            timeout,
                                            IsInterval::NonInterval,
                                            TimerSource::FromWindow(self.id.clone()),
                                            self.script_chan.clone())
    }

    fn SetTimeout_(self, _cx: *mut JSContext, callback: DOMString, timeout: i32, args: Vec<JSVal>) -> i32 {
        self.timers.set_timeout_or_interval(TimerCallback::StringTimerCallback(callback),
                                            args,
                                            timeout,
                                            IsInterval::NonInterval,
                                            TimerSource::FromWindow(self.id.clone()),
                                            self.script_chan.clone())
    }

    fn ClearTimeout(self, handle: i32) {
        self.timers.clear_timeout_or_interval(handle);
    }

    fn SetInterval(self, _cx: *mut JSContext, callback: Function, timeout: i32, args: Vec<JSVal>) -> i32 {
        self.timers.set_timeout_or_interval(TimerCallback::FunctionTimerCallback(callback),
                                            args,
                                            timeout,
                                            IsInterval::Interval,
                                            TimerSource::FromWindow(self.id.clone()),
                                            self.script_chan.clone())
    }

    fn SetInterval_(self, _cx: *mut JSContext, callback: DOMString, timeout: i32, args: Vec<JSVal>) -> i32 {
        self.timers.set_timeout_or_interval(TimerCallback::StringTimerCallback(callback),
                                            args,
                                            timeout,
                                            IsInterval::Interval,
                                            TimerSource::FromWindow(self.id.clone()),
                                            self.script_chan.clone())
    }

    fn ClearInterval(self, handle: i32) {
        self.ClearTimeout(handle);
    }

    fn Window(self) -> Temporary<Window> {
        Temporary::from_rooted(self)
    }

    fn Self_(self) -> Temporary<Window> {
        self.Window()
    }

    // http://www.whatwg.org/html/#dom-frames
    fn Frames(self) -> Temporary<Window> {
        self.Window()
    }

    // https://html.spec.whatwg.org/multipage/browsers.html#dom-parent
    fn Parent(self) -> Temporary<Window> {
        let browser_context = self.browser_context();
        let browser_context = browser_context.as_ref().unwrap();

        browser_context.frame_element().map_or(self.Window(), |fe| {
            let frame_element = fe.root();
            let window = window_from_node(frame_element.r()).root();
            // FIXME(https://github.com/rust-lang/rust/issues/23338)
            let r = window.r();
            let context = r.browser_context();
            context.as_ref().unwrap().active_window()
        })
    }

    fn Performance(self) -> Temporary<Performance> {
        self.performance.or_init(|| {
            Performance::new(self, self.navigation_start,
                             self.navigation_start_precise)
        })
    }

    global_event_handlers!();
    event_handler!(unload, GetOnunload, SetOnunload);
    error_event_handler!(error, GetOnerror, SetOnerror);

    fn Screen(self) -> Temporary<Screen> {
        self.screen.or_init(|| Screen::new(self))
    }

    fn Debug(self, message: DOMString) {
        debug!("{}", message);
    }

    #[allow(unsafe_blocks)]
    fn Gc(self) {
        unsafe {
            JS_GC(JS_GetRuntime(self.get_cx()));
        }
    }

    fn Btoa(self, btoa: DOMString) -> Fallible<DOMString> {
        base64_btoa(btoa)
    }

    fn Atob(self, atob: DOMString) -> Fallible<DOMString> {
        base64_atob(atob)
    }
}

pub trait WindowHelpers {
    fn clear_js_context(self);
    fn clear_js_context_for_script_deallocation(self);
    fn init_browser_context(self, doc: JSRef<Document>, frame_element: Option<JSRef<Element>>);
    fn load_url(self, href: DOMString);
    fn handle_fire_timer(self, timer_id: TimerId);
    fn reflow(self, goal: ReflowGoal, query_type: ReflowQueryType, reason: ReflowReason);
    fn join_layout(self);
    fn layout(&self) -> &LayoutRPC;
    fn content_box_query(self, content_box_request: TrustedNodeAddress) -> Rect<Au>;
    fn content_boxes_query(self, content_boxes_request: TrustedNodeAddress) -> Vec<Rect<Au>>;
    fn handle_reflow_complete_msg(self, reflow_id: uint);
    fn handle_resize_inactive_msg(self, new_size: WindowSizeData);
    fn set_fragment_name(self, fragment: Option<String>);
    fn steal_fragment_name(self) -> Option<String>;
    fn set_window_size(self, size: WindowSizeData);
    fn window_size(self) -> Option<WindowSizeData>;
    fn get_url(self) -> Url;
    fn resource_task(self) -> ResourceTask;
    fn devtools_chan(self) -> Option<DevtoolsControlChan>;
    fn layout_chan(self) -> LayoutChan;
    fn constellation_chan(self) -> ConstellationChan;
    fn windowproxy_handler(self) -> WindowProxyHandler;
    fn get_next_subpage_id(self) -> SubpageId;
    fn layout_is_idle(self) -> bool;
    fn set_resize_event(self, event: WindowSizeData);
    fn steal_resize_event(self) -> Option<WindowSizeData>;
    fn set_page_clip_rect_with_new_viewport(self, viewport: Rect<f32>) -> bool;
    fn set_devtools_wants_updates(self, value: bool);
    fn IndexedGetter(self, _index: u32, _found: &mut bool) -> Option<Temporary<Window>>;
    fn thaw(self);
    fn freeze(self);
}

pub trait ScriptHelpers {
    fn evaluate_js_on_global_with_result(self, code: &str) -> JSVal;
    fn evaluate_script_on_global_with_result(self, code: &str, filename: &str) -> JSVal;
}

impl<'a, T: Reflectable> ScriptHelpers for JSRef<'a, T> {
    fn evaluate_js_on_global_with_result(self, code: &str) -> JSVal {
        self.evaluate_script_on_global_with_result(code, "")
    }

    #[allow(unsafe_blocks)]
    fn evaluate_script_on_global_with_result(self, code: &str, filename: &str) -> JSVal {
        let this = self.reflector().get_jsobject();
        let cx = global_object_for_js_object(this).root().r().get_cx();
        let global = global_object_for_js_object(this).root().r().reflector().get_jsobject();
        let code: Vec<u16> = code.as_slice().utf16_units().collect();
        let mut rval = UndefinedValue();
        let filename = CString::from_slice(filename.as_bytes());

        with_compartment(cx, global, || {
            unsafe {
                if JS_EvaluateUCScript(cx, global, code.as_ptr(),
                                       code.len() as libc::c_uint,
                                       filename.as_ptr(), 1, &mut rval) == 0 {
                    debug!("error evaluating JS string");
                    report_pending_exception(cx, global);
                }
                rval
            }
        })
    }
}

impl<'a> WindowHelpers for JSRef<'a, Window> {
    fn clear_js_context(self) {
        let document = self.Document().root();
        NodeCast::from_ref(document.r()).teardown();

        *self.js_context.borrow_mut() = None;
        *self.browser_context.borrow_mut() = None;
    }

    #[allow(unsafe_blocks)]
    fn clear_js_context_for_script_deallocation(self) {
        unsafe {
            *self.js_context.borrow_for_script_deallocation() = None;
            *self.browser_context.borrow_for_script_deallocation() = None;
        }
    }

    /// Reflows the page if it's possible to do so and the page is dirty. This method will wait
    /// for the layout thread to complete (but see the `TODO` below). If there is no window size
    /// yet, the page is presumed invisible and no reflow is performed.
    ///
    /// TODO(pcwalton): Only wait for style recalc, since we have off-main-thread layout.
    fn reflow(self, goal: ReflowGoal, query_type: ReflowQueryType, reason: ReflowReason) {
        let document = self.Document().root();
        let root = document.r().GetDocumentElement().root();
        let root = match root.r() {
            Some(root) => root,
            None => return,
        };

        debug!("script: performing reflow for goal {:?}", goal);

        let root: JSRef<Node> = NodeCast::from_ref(root);
        if !root.get_has_dirty_descendants() {
            debug!("root has no dirty descendants; avoiding reflow");
            return
        }

        let window_size = match self.window_size.get() {
            Some(window_size) => window_size,
            None => return,
        };

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
        let reflow = box Reflow {
            document_root: root.to_trusted_node_address(),
            url: self.get_url(),
            iframe: self.subpage_id.is_some(),
            goal: goal,
            window_size: window_size,
            script_chan: self.control_chan.clone(),
            script_join_chan: join_chan,
            id: last_reflow_id.get(),
            query_type: query_type,
            page_clip_rect: self.page_clip_rect.get(),
        };

        let LayoutChan(ref chan) = self.layout_chan;
        chan.send(Msg::Reflow(reflow)).unwrap();

        debug!("script: layout forked");

        self.join_layout();
    }

    // FIXME(cgaebel): join_layout is racey. What if the compositor triggers a
    // reflow between the "join complete" message and returning from this
    // function?

    /// Sends a ping to layout and waits for the response. The response will arrive when the
    /// layout task has finished any pending request messages.
    fn join_layout(self) {
        let mut layout_join_port = self.layout_join_port.borrow_mut();
        if let Some(join_port) = mem::replace(&mut *layout_join_port, None) {
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
        self.reflow(ReflowGoal::ForScriptQuery, ReflowQueryType::ContentBoxQuery(content_box_request), ReflowReason::Query);
        self.join_layout(); //FIXME: is this necessary, or is layout_rpc's mutex good enough?
        let ContentBoxResponse(rect) = self.layout_rpc.content_box();
        rect
    }

    fn content_boxes_query(self, content_boxes_request: TrustedNodeAddress) -> Vec<Rect<Au>> {
        self.reflow(ReflowGoal::ForScriptQuery, ReflowQueryType::ContentBoxesQuery(content_boxes_request), ReflowReason::Query);
        self.join_layout(); //FIXME: is this necessary, or is layout_rpc's mutex good enough?
        let ContentBoxesResponse(rects) = self.layout_rpc.content_boxes();
        rects
    }

    fn handle_reflow_complete_msg(self, reflow_id: uint) {
        let last_reflow_id = self.last_reflow_id.get();
        if last_reflow_id == reflow_id {
            *self.layout_join_port.borrow_mut() = None;
        }
    }

    fn handle_resize_inactive_msg(self, new_size: WindowSizeData) {
        self.window_size.set(Some(new_size));
    }

    fn init_browser_context(self, doc: JSRef<Document>, frame_element: Option<JSRef<Element>>) {
        *self.browser_context.borrow_mut() = Some(BrowserContext::new(doc, frame_element));
    }

    /// Commence a new URL load which will either replace this window or scroll to a fragment.
    fn load_url(self, href: DOMString) {
        let base_url = self.get_url();
        debug!("current page url is {}", base_url);
        let url = UrlParser::new().base_url(&base_url).parse(href.as_slice());
        // FIXME: handle URL parse errors more gracefully.
        let url = url.unwrap();
        match url.fragment {
            Some(fragment) => {
                self.script_chan.send(ScriptMsg::TriggerFragment(self.id, fragment)).unwrap();
            },
            None => {
                self.script_chan.send(ScriptMsg::Navigate(self.id, LoadData::new(url))).unwrap();
            }
        }
    }

    fn handle_fire_timer(self, timer_id: TimerId) {
        self.timers.fire_timer(timer_id, self);
        self.reflow(ReflowGoal::ForDisplay, ReflowQueryType::NoQuery, ReflowReason::Timer);
    }

    fn set_fragment_name(self, fragment: Option<String>) {
        *self.fragment_name.borrow_mut() = fragment;
    }

    fn steal_fragment_name(self) -> Option<String> {
        // FIXME(https://github.com/rust-lang/rust/issues/23338)
        let mut name = self.fragment_name.borrow_mut();
        name.take()
    }

    fn set_window_size(self, size: WindowSizeData) {
        self.window_size.set(Some(size));
    }

    fn window_size(self) -> Option<WindowSizeData> {
        self.window_size.get()
    }

    fn get_url(self) -> Url {
        let doc = self.Document().root();
        doc.r().url()
    }

    fn resource_task(self) -> ResourceTask {
        self.resource_task.clone()
    }

    fn devtools_chan(self) -> Option<DevtoolsControlChan> {
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
        // FIXME(https://github.com/rust-lang/rust/issues/23338)
        let port = self.layout_join_port.borrow();
        port.is_none()
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

    // https://html.spec.whatwg.org/multipage/browsers.html#accessing-other-browsing-contexts
    fn IndexedGetter(self, _index: u32, _found: &mut bool) -> Option<Temporary<Window>> {
        None
    }

    fn thaw(self) {
        self.timers.resume();
    }

    fn freeze(self) {
        self.timers.suspend();
    }
}

impl Window {
    pub fn new(js_context: Rc<Cx>,
               page: Rc<Page>,
               script_chan: Box<ScriptChan+Send>,
               control_chan: ScriptControlChan,
               compositor: Box<ScriptListener+'static>,
               image_cache_task: ImageCacheTask,
               resource_task: ResourceTask,
               storage_task: StorageTask,
               devtools_chan: Option<DevtoolsControlChan>,
               constellation_chan: ConstellationChan,
               layout_chan: LayoutChan,
               id: PipelineId,
               subpage_id: Option<SubpageId>,
               window_size: Option<WindowSizeData>)
               -> Temporary<Window> {
        let layout_rpc: Box<LayoutRPC> = {
            let (rpc_send, rpc_recv) = channel();
            let LayoutChan(ref lchan) = layout_chan;
            lchan.send(Msg::GetRPC(rpc_send)).unwrap();
            rpc_recv.recv().unwrap()
        };

        let win = box Window {
            eventtarget: EventTarget::new_inherited(EventTargetTypeId::Window),
            script_chan: script_chan,
            control_chan: control_chan,
            console: Default::default(),
            compositor: DOMRefCell::new(compositor),
            page: page,
            navigator: Default::default(),
            image_cache_task: image_cache_task,
            devtools_chan: devtools_chan,
            browser_context: DOMRefCell::new(None),
            performance: Default::default(),
            navigation_start: time::get_time().sec as u64,
            navigation_start_precise: time::precise_time_ns() as f64,
            screen: Default::default(),
            session_storage: Default::default(),
            local_storage: Default::default(),
            timers: TimerManager::new(),
            id: id,
            subpage_id: subpage_id,
            dom_static: GlobalStaticData::new(),
            js_context: DOMRefCell::new(Some(js_context.clone())),
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
            devtools_wants_updates: Cell::new(false),
        };

        WindowBinding::Wrap(js_context.ptr, win)
    }
}

fn should_move_clip_rect(clip_rect: Rect<Au>, new_viewport: Rect<f32>) -> bool{
    let clip_rect = Rect(Point2D(geometry::to_frac_px(clip_rect.origin.x) as f32,
                                 geometry::to_frac_px(clip_rect.origin.y) as f32),
                         Size2D(geometry::to_frac_px(clip_rect.size.width) as f32,
                                geometry::to_frac_px(clip_rect.size.height) as f32));

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
    let mut debug_msg = String::from_str("****");
    match *goal {
        ReflowGoal::ForDisplay => debug_msg.push_str("\tForDisplay"),
        ReflowGoal::ForScriptQuery => debug_msg.push_str("\tForScriptQuery"),
    }

    match *query_type {
        ReflowQueryType::NoQuery => debug_msg.push_str("\tNoQuery"),
        ReflowQueryType::ContentBoxQuery(_n) => debug_msg.push_str("\tContentBoxQuery"),
        ReflowQueryType::ContentBoxesQuery(_n) => debug_msg.push_str("\tContentBoxesQuery"),
    }

    match *reason {
        ReflowReason::CachedPageNeededReflow => debug_msg.push_str("\tCachedPageNeededReflow"),
        ReflowReason::FirstLoad => debug_msg.push_str("\tFirstLoad"),
        ReflowReason::KeyEvent => debug_msg.push_str("\tKeyEvent"),
        ReflowReason::MouseEvent => debug_msg.push_str("\tMouseEvent"),
        ReflowReason::Query => debug_msg.push_str("\tQuery"),
        ReflowReason::ReceivedReflowEvent => debug_msg.push_str("\tReceivedReflowEvent"),
        ReflowReason::Timer => debug_msg.push_str("\tTimer"),
        ReflowReason::Viewport => debug_msg.push_str("\tViewport"),
        ReflowReason::WindowResize => debug_msg.push_str("\tWindowResize"),
        ReflowReason::DOMContentLoaded => debug_msg.push_str("\tDOMContentLoaded"),
        ReflowReason::DocumentLoaded => debug_msg.push_str("\tDocumentLoaded"),
    }

    println!("{}", debug_msg);
}
