/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::EventHandlerBinding::{OnErrorEventHandlerNonNull, EventHandlerNonNull};
use dom::bindings::codegen::Bindings::FunctionBinding::Function;
use dom::bindings::codegen::Bindings::WindowBinding;
use dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use dom::bindings::codegen::InheritTypes::EventTargetCast;
use dom::bindings::global::global_object_for_js_object;
use dom::bindings::error::Fallible;
use dom::bindings::error::Error::InvalidCharacter;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{MutNullableJS, JSRef, Temporary};
use dom::bindings::utils::Reflectable;
use dom::browsercontext::BrowserContext;
use dom::console::Console;
use dom::document::Document;
use dom::element::Element;
use dom::eventtarget::{EventTarget, EventTargetHelpers, EventTargetTypeId};
use dom::location::Location;
use dom::navigator::Navigator;
use dom::node::window_from_node;
use dom::performance::Performance;
use dom::screen::Screen;
use dom::storage::Storage;
use layout_interface::{ReflowGoal, ReflowQueryType};
use page::Page;
use script_task::{TimerSource, ScriptChan};
use script_task::ScriptMsg;
use script_traits::ScriptControlChan;
use timers::{IsInterval, TimerId, TimerManager, TimerCallback};

use msg::compositor_msg::ScriptListener;
use msg::constellation_msg::LoadData;
use net::image_cache_task::ImageCacheTask;
use net::storage_task::StorageTask;
use util::str::{DOMString,HTML_SPACE_CHARACTERS};

use js::jsapi::JS_EvaluateUCScript;
use js::jsapi::JSContext;
use js::jsapi::{JS_GC, JS_GetRuntime};
use js::jsval::{JSVal, UndefinedValue};
use js::rust::with_compartment;
use url::{Url, UrlParser};

use libc;
use serialize::base64::{FromBase64, ToBase64, STANDARD};
use std::cell::{Ref, RefMut};
use std::default::Default;
use std::ffi::CString;
use std::rc::Rc;
use time;

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
    timers: TimerManager,
}

impl Window {
    pub fn get_cx(&self) -> *mut JSContext {
        let js_info = self.page().js_info();
        (*js_info.as_ref().unwrap().js_context).ptr
    }

    pub fn script_chan(&self) -> Box<ScriptChan+Send> {
        self.script_chan.clone()
    }

    pub fn control_chan<'a>(&'a self) -> &'a ScriptControlChan {
        &self.control_chan
    }

    pub fn image_cache_task<'a>(&'a self) -> &'a ImageCacheTask {
        &self.image_cache_task
    }

    pub fn compositor(&self) -> RefMut<Box<ScriptListener+'static>> {
        self.compositor.borrow_mut()
    }

    pub fn browser_context(&self) -> Ref<Option<BrowserContext>> {
        self.browser_context.borrow()
    }

    pub fn page<'a>(&'a self) -> &'a Page {
        &*self.page
    }

    pub fn page_clone(&self) -> Rc<Page> {
        self.page.clone()
    }

    pub fn get_url(&self) -> Url {
        self.page().get_url()
    }

    pub fn storage_task(&self) -> StorageTask {
        self.page().storage_task.clone()
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
        self.script_chan.send(ScriptMsg::ExitWindow(self.page.id.clone()));
    }

    fn Document(self) -> Temporary<Document> {
        let frame = self.page().frame();
        Temporary::new(frame.as_ref().unwrap().document.clone())
    }

    fn Location(self) -> Temporary<Location> {
        self.Document().root().r().Location()
    }

    fn SessionStorage(self) -> Temporary<Storage> {
        self.session_storage.or_init(|| Storage::new(&GlobalRef::Window(self)))
    }

    fn Console(self) -> Temporary<Console> {
        self.console.or_init(|| Console::new(GlobalRef::Window(self)))
    }

    fn GetFrameElement(self) -> Option<Temporary<Element>> {
        self.browser_context().as_ref().unwrap().frame_element()
    }

    fn Navigator(self) -> Temporary<Navigator> {
        self.navigator.or_init(|| Navigator::new(self))
    }

    fn SetTimeout(self, _cx: *mut JSContext, callback: Function, timeout: i32, args: Vec<JSVal>) -> i32 {
        self.timers.set_timeout_or_interval(TimerCallback::FunctionTimerCallback(callback),
                                            args,
                                            timeout,
                                            IsInterval::NonInterval,
                                            TimerSource::FromWindow(self.page.id.clone()),
                                            self.script_chan.clone())
    }

    fn SetTimeout_(self, _cx: *mut JSContext, callback: DOMString, timeout: i32, args: Vec<JSVal>) -> i32 {
        self.timers.set_timeout_or_interval(TimerCallback::StringTimerCallback(callback),
                                            args,
                                            timeout,
                                            IsInterval::NonInterval,
                                            TimerSource::FromWindow(self.page.id.clone()),
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
                                            TimerSource::FromWindow(self.page.id.clone()),
                                            self.script_chan.clone())
    }

    fn SetInterval_(self, _cx: *mut JSContext, callback: DOMString, timeout: i32, args: Vec<JSVal>) -> i32 {
        self.timers.set_timeout_or_interval(TimerCallback::StringTimerCallback(callback),
                                            args,
                                            timeout,
                                            IsInterval::Interval,
                                            TimerSource::FromWindow(self.page.id.clone()),
                                            self.script_chan.clone())
    }

    fn ClearInterval(self, handle: i32) {
        self.ClearTimeout(handle);
    }

    fn Window(self) -> Temporary<Window> {
        Temporary::from_rooted(self)
    }

    fn Self(self) -> Temporary<Window> {
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
            window.r().browser_context().as_ref().unwrap().active_window()
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
    fn flush_layout(self, goal: ReflowGoal, query: ReflowQueryType);
    fn init_browser_context(self, doc: JSRef<Document>, frame_element: Option<JSRef<Element>>);
    fn load_url(self, href: DOMString);
    fn handle_fire_timer(self, timer_id: TimerId);
    fn IndexedGetter(self, _index: u32, _found: &mut bool) -> Option<Temporary<Window>>;
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
                }
                rval
            }
        })
    }
}

impl<'a> WindowHelpers for JSRef<'a, Window> {
    fn flush_layout(self, goal: ReflowGoal, query: ReflowQueryType) {
        self.page().flush_layout(goal, query);
    }

    fn init_browser_context(self, doc: JSRef<Document>, frame_element: Option<JSRef<Element>>) {
        *self.browser_context.borrow_mut() = Some(BrowserContext::new(doc, frame_element));
    }

    /// Commence a new URL load which will either replace this window or scroll to a fragment.
    fn load_url(self, href: DOMString) {
        let base_url = self.page().get_url();
        debug!("current page url is {}", base_url);
        let url = UrlParser::new().base_url(&base_url).parse(href.as_slice());
        // FIXME: handle URL parse errors more gracefully.
        let url = url.unwrap();
        match url.fragment {
            Some(fragment) => {
                self.script_chan.send(ScriptMsg::TriggerFragment(self.page.id, fragment));
            },
            None => {
                self.script_chan.send(ScriptMsg::TriggerLoad(self.page.id, LoadData::new(url)));
            }
        }
    }

    fn handle_fire_timer(self, timer_id: TimerId) {
        self.timers.fire_timer(timer_id, self);
        self.flush_layout(ReflowGoal::ForDisplay, ReflowQueryType::NoQuery);
    }

    // https://html.spec.whatwg.org/multipage/browsers.html#accessing-other-browsing-contexts
    fn IndexedGetter(self, _index: u32, _found: &mut bool) -> Option<Temporary<Window>> {
        None
    }
}

impl Window {
    pub fn new(cx: *mut JSContext,
               page: Rc<Page>,
               script_chan: Box<ScriptChan+Send>,
               control_chan: ScriptControlChan,
               compositor: Box<ScriptListener+'static>,
               image_cache_task: ImageCacheTask)
               -> Temporary<Window> {
        let win = box Window {
            eventtarget: EventTarget::new_inherited(EventTargetTypeId::Window),
            script_chan: script_chan,
            control_chan: control_chan,
            console: Default::default(),
            compositor: DOMRefCell::new(compositor),
            page: page,
            navigator: Default::default(),
            image_cache_task: image_cache_task,
            browser_context: DOMRefCell::new(None),
            performance: Default::default(),
            navigation_start: time::get_time().sec as u64,
            navigation_start_precise: time::precise_time_ns() as f64,
            screen: Default::default(),
            session_storage: Default::default(),
            timers: TimerManager::new(),
        };

        WindowBinding::Wrap(cx, win)
    }
}
