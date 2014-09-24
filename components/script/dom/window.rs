/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::EventHandlerBinding::{OnErrorEventHandlerNonNull, EventHandlerNonNull};
use dom::bindings::codegen::Bindings::WindowBinding;
use dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use dom::bindings::codegen::InheritTypes::EventTargetCast;
use dom::bindings::error::{Fallible, InvalidCharacter};
use dom::bindings::global;
use dom::bindings::js::{JS, JSRef, Temporary, OptionalSettable};
use dom::bindings::trace::{Traceable, Untraceable};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::browsercontext::BrowserContext;
use dom::console::Console;
use dom::document::Document;
use dom::eventtarget::{EventTarget, WindowTypeId, EventTargetHelpers};
use dom::location::Location;
use dom::navigator::Navigator;
use dom::performance::Performance;
use dom::screen::Screen;
use layout_interface::{ReflowGoal, DocumentDamageLevel};
use page::Page;
use script_task::{ExitWindowMsg, FireTimerMsg, ScriptChan, TriggerLoadMsg, TriggerFragmentMsg};
use script_traits::ScriptControlChan;

use servo_msg::compositor_msg::ScriptListener;
use servo_net::image_cache_task::ImageCacheTask;
use servo_util::str::{DOMString,HTML_SPACE_CHARACTERS};
use servo_util::task::{spawn_named};

use js::jsapi::{JS_CallFunctionValue, JS_EvaluateUCScript};
use js::jsapi::JSContext;
use js::jsapi::{JS_GC, JS_GetRuntime};
use js::jsval::JSVal;
use js::jsval::{UndefinedValue, NullValue};
use js::rust::with_compartment;
use url::{Url, UrlParser};

use libc;
use serialize::base64::{FromBase64, ToBase64, STANDARD};
use std::collections::hashmap::HashMap;
use std::cell::{Cell, RefCell};
use std::cmp;
use std::comm::{channel, Sender};
use std::comm::Select;
use std::hash::{Hash, sip};
use std::io::timer::Timer;
use std::ptr;
use std::rc::Rc;
use std::time::duration::Duration;
use time;

#[deriving(PartialEq, Eq)]
#[jstraceable]
pub struct TimerId(i32);

#[jstraceable]
pub struct TimerHandle {
    handle: TimerId,
    pub data: TimerData,
    cancel_chan: Untraceable<Option<Sender<()>>>,
}

impl Hash for TimerId {
    fn hash(&self, state: &mut sip::SipState) {
        let TimerId(id) = *self;
        id.hash(state);
    }
}

impl TimerHandle {
    fn cancel(&mut self) {
        self.cancel_chan.as_ref().map(|chan| chan.send_opt(()).ok());
    }
}

#[jstraceable]
#[must_root]
pub struct Window {
    eventtarget: EventTarget,
    pub script_chan: ScriptChan,
    pub control_chan: ScriptControlChan,
    console: Cell<Option<JS<Console>>>,
    location: Cell<Option<JS<Location>>>,
    navigator: Cell<Option<JS<Navigator>>>,
    pub image_cache_task: ImageCacheTask,
    pub active_timers: Traceable<RefCell<HashMap<TimerId, TimerHandle>>>,
    next_timer_handle: Traceable<Cell<i32>>,
    pub compositor: Untraceable<Box<ScriptListener+'static>>,
    pub browser_context: Traceable<RefCell<Option<BrowserContext>>>,
    pub page: Rc<Page>,
    performance: Cell<Option<JS<Performance>>>,
    pub navigationStart: u64,
    pub navigationStartPrecise: f64,
    screen: Cell<Option<JS<Screen>>>,
}

impl Window {
    pub fn get_cx(&self) -> *mut JSContext {
        let js_info = self.page().js_info();
        (**js_info.as_ref().unwrap().js_context).ptr
    }

    pub fn page<'a>(&'a self) -> &'a Page {
        &*self.page
    }
    pub fn get_url(&self) -> Url {
        self.page().get_url()
    }
}

#[unsafe_destructor]
impl Drop for Window {
    fn drop(&mut self) {
        for (_, timer_handle) in self.active_timers.borrow_mut().iter_mut() {
            timer_handle.cancel();
        }
    }
}

// Holder for the various JS values associated with setTimeout
// (ie. function value to invoke and all arguments to pass
//      to the function when calling it)
#[jstraceable]
pub struct TimerData {
    pub is_interval: bool,
    pub funval: Traceable<JSVal>,
}

// http://www.whatwg.org/html/#atob
pub fn base64_btoa(btoa: DOMString) -> Fallible<DOMString> {
    let input = btoa.as_slice();
    // "The btoa() method must throw an InvalidCharacterError exception if
    //  the method's first argument contains any character whose code point
    //  is greater than U+00FF."
    if input.chars().any(|c: char| c > '\u00FF') {
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
    let mut input = atob.as_slice();

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
    input = without_spaces.as_slice();

    // "If the length of input divides by 4 leaving no remainder, then:
    //  if input ends with one or two U+003D EQUALS SIGN (=) characters,
    //  remove them from input."
    if input.len() % 4 == 0 {
        if input.ends_with("==") {
            input = input.slice_to(input.len() - 2)
        } else if input.ends_with("=") {
            input = input.slice_to(input.len() - 1)
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
        println!("ALERT: {:s}", s);
    }

    fn Close(self) {
        let ScriptChan(ref chan) = self.script_chan;
        chan.send(ExitWindowMsg(self.page.id.clone()));
    }

    fn Document(self) -> Temporary<Document> {
        let frame = self.page().frame();
        Temporary::new(frame.as_ref().unwrap().document.clone())
    }

    fn Location(self) -> Temporary<Location> {
        if self.location.get().is_none() {
            let page = self.deref().page.clone();
            let location = Location::new(self, page);
            self.location.assign(Some(location));
        }
        Temporary::new(self.location.get().as_ref().unwrap().clone())
    }

    fn Console(self) -> Temporary<Console> {
        if self.console.get().is_none() {
            let console = Console::new(&global::Window(self));
            self.console.assign(Some(console));
        }
        Temporary::new(self.console.get().as_ref().unwrap().clone())
    }

    fn Navigator(self) -> Temporary<Navigator> {
        if self.navigator.get().is_none() {
            let navigator = Navigator::new(self);
            self.navigator.assign(Some(navigator));
        }
        Temporary::new(self.navigator.get().as_ref().unwrap().clone())
    }

    fn SetTimeout(self, _cx: *mut JSContext, callback: JSVal, timeout: i32) -> i32 {
        self.set_timeout_or_interval(callback, timeout, false)
    }

    fn ClearTimeout(self, handle: i32) {
        let mut timers = self.active_timers.deref().borrow_mut();
        let mut timer_handle = timers.pop(&TimerId(handle));
        match timer_handle {
            Some(ref mut handle) => handle.cancel(),
            None => { }
        }
        timers.remove(&TimerId(handle));
    }

    fn SetInterval(self, _cx: *mut JSContext, callback: JSVal, timeout: i32) -> i32 {
        self.set_timeout_or_interval(callback, timeout, true)
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

    fn Parent(self) -> Temporary<Window> {
        //TODO - Once we support iframes correctly this needs to return the parent frame
        self.Window()
    }

    fn Performance(self) -> Temporary<Performance> {
        if self.performance.get().is_none() {
            let performance = Performance::new(self);
            self.performance.assign(Some(performance));
        }
        Temporary::new(self.performance.get().as_ref().unwrap().clone())
    }

    fn GetOnclick(self) -> Option<EventHandlerNonNull> {
        let eventtarget: JSRef<EventTarget> = EventTargetCast::from_ref(self);
        eventtarget.get_event_handler_common("click")
    }

    fn SetOnclick(self, listener: Option<EventHandlerNonNull>) {
        let eventtarget: JSRef<EventTarget> = EventTargetCast::from_ref(self);
        eventtarget.set_event_handler_common("click", listener)
    }

    fn GetOnload(self) -> Option<EventHandlerNonNull> {
        let eventtarget: JSRef<EventTarget> = EventTargetCast::from_ref(self);
        eventtarget.get_event_handler_common("load")
    }

    fn SetOnload(self, listener: Option<EventHandlerNonNull>) {
        let eventtarget: JSRef<EventTarget> = EventTargetCast::from_ref(self);
        eventtarget.set_event_handler_common("load", listener)
    }

    fn GetOnunload(self) -> Option<EventHandlerNonNull> {
        let eventtarget: JSRef<EventTarget> = EventTargetCast::from_ref(self);
        eventtarget.get_event_handler_common("unload")
    }

    fn SetOnunload(self, listener: Option<EventHandlerNonNull>) {
        let eventtarget: JSRef<EventTarget> = EventTargetCast::from_ref(self);
        eventtarget.set_event_handler_common("unload", listener)
    }

    fn GetOnerror(self) -> Option<OnErrorEventHandlerNonNull> {
        let eventtarget: JSRef<EventTarget> = EventTargetCast::from_ref(self);
        eventtarget.get_event_handler_common("error")
    }

    fn SetOnerror(self, listener: Option<OnErrorEventHandlerNonNull>) {
        let eventtarget: JSRef<EventTarget> = EventTargetCast::from_ref(self);
        eventtarget.set_event_handler_common("error", listener)
    }

    fn Screen(self) -> Temporary<Screen> {
        if self.screen.get().is_none() {
            let screen = Screen::new(self);
            self.screen.assign(Some(screen));
        }
        Temporary::new(self.screen.get().as_ref().unwrap().clone())
    }

    fn Debug(self, message: DOMString) {
        debug!("{:s}", message);
    }

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

impl Reflectable for Window {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.eventtarget.reflector()
    }
}

pub trait WindowHelpers {
    fn damage_and_reflow(self, damage: DocumentDamageLevel);
    fn flush_layout(self, goal: ReflowGoal);
    fn wait_until_safe_to_modify_dom(self);
    fn init_browser_context(self, doc: JSRef<Document>);
    fn load_url(self, href: DOMString);
    fn handle_fire_timer(self, timer_id: TimerId, cx: *mut JSContext);
    fn evaluate_js_with_result(self, code: &str) -> JSVal;
}

trait PrivateWindowHelpers {
    fn set_timeout_or_interval(self, callback: JSVal, timeout: i32, is_interval: bool) -> i32;
}

impl<'a> WindowHelpers for JSRef<'a, Window> {
    fn evaluate_js_with_result(self, code: &str) -> JSVal {
        let global = self.reflector().get_jsobject();
        let code: Vec<u16> = code.as_slice().utf16_units().collect();
        let mut rval = UndefinedValue();
        let filename = "".to_c_str();
        let cx = self.get_cx();

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

    fn damage_and_reflow(self, damage: DocumentDamageLevel) {
        self.page().damage(damage);
        self.page().avoided_reflows.set(self.page().avoided_reflows.get() + 1);
    }

    fn flush_layout(self, goal: ReflowGoal) {
        self.page().flush_layout(goal);
    }

    fn wait_until_safe_to_modify_dom(self) {
        // FIXME: This disables concurrent layout while we are modifying the DOM, since
        //        our current architecture is entirely unsafe in the presence of races.
        self.page().join_layout();
    }

    fn init_browser_context(self, doc: JSRef<Document>) {
        *self.browser_context.deref().borrow_mut() = Some(BrowserContext::new(doc));
    }

    /// Commence a new URL load which will either replace this window or scroll to a fragment.
    fn load_url(self, href: DOMString) {
        let base_url = self.page().get_url();
        debug!("current page url is {:?}", base_url);
        let url = UrlParser::new().base_url(&base_url).parse(href.as_slice());
        // FIXME: handle URL parse errors more gracefully.
        let url = url.unwrap();
        let ScriptChan(ref script_chan) = self.script_chan;
        if href.as_slice().starts_with("#") {
            script_chan.send(TriggerFragmentMsg(self.page.id, url));
        } else {
            script_chan.send(TriggerLoadMsg(self.page.id, url));
        }
    }

    fn handle_fire_timer(self, timer_id: TimerId, cx: *mut JSContext) {
        let this_value = self.reflector().get_jsobject();

        let data = match self.active_timers.deref().borrow().find(&timer_id) {
            None => return,
            Some(timer_handle) => timer_handle.data,
        };

        // TODO: Support extra arguments. This requires passing a `*JSVal` array as `argv`.
        with_compartment(cx, this_value, || {
            let mut rval = NullValue();
            unsafe {
                JS_CallFunctionValue(cx, this_value, *data.funval,
                                     0, ptr::null_mut(), &mut rval);
            }
        });

        if !data.is_interval {
            self.active_timers.deref().borrow_mut().remove(&timer_id);
        }
    }
}

impl<'a> PrivateWindowHelpers for JSRef<'a, Window> {
    fn set_timeout_or_interval(self, callback: JSVal, timeout: i32, is_interval: bool) -> i32 {
        let timeout = cmp::max(0, timeout) as u64;
        let handle = self.next_timer_handle.deref().get();
        self.next_timer_handle.deref().set(handle + 1);

        // Post a delayed message to the per-window timer task; it will dispatch it
        // to the relevant script handler that will deal with it.
        let tm = Timer::new().unwrap();
        let (cancel_chan, cancel_port) = channel();
        let chan = self.script_chan.clone();
        let page_id = self.page.id.clone();
        let spawn_name = if is_interval {
            "Window:SetInterval"
        } else {
            "Window:SetTimeout"
        };
        spawn_named(spawn_name, proc() {
            let mut tm = tm;
            let duration = Duration::milliseconds(timeout as i64);
            let timeout_port = if is_interval {
                tm.periodic(duration)
            } else {
                tm.oneshot(duration)
            };
            let cancel_port = cancel_port;

            let select = Select::new();
            let mut timeout_handle = select.handle(&timeout_port);
            unsafe { timeout_handle.add() };
            let mut cancel_handle = select.handle(&cancel_port);
            unsafe { cancel_handle.add() };

            loop {
                let id = select.wait();
                if id == timeout_handle.id() {
                    timeout_port.recv();
                    let ScriptChan(ref chan) = chan;
                    chan.send(FireTimerMsg(page_id, TimerId(handle)));
                    if !is_interval {
                        break;
                    }
                } else if id == cancel_handle.id() {
                    break;
                }
            }
        });
        let timer_id = TimerId(handle);
        let timer = TimerHandle {
            handle: timer_id,
            cancel_chan: Untraceable::new(Some(cancel_chan)),
            data: TimerData {
                is_interval: is_interval,
                funval: Traceable::new(callback),
            }
        };
        self.active_timers.deref().borrow_mut().insert(timer_id, timer);
        handle
    }
}

impl Window {
    pub fn new(cx: *mut JSContext,
               page: Rc<Page>,
               script_chan: ScriptChan,
               control_chan: ScriptControlChan,
               compositor: Box<ScriptListener+'static>,
               image_cache_task: ImageCacheTask)
               -> Temporary<Window> {
        let win = box Window {
            eventtarget: EventTarget::new_inherited(WindowTypeId),
            script_chan: script_chan,
            control_chan: control_chan,
            console: Cell::new(None),
            compositor: Untraceable::new(compositor),
            page: page,
            location: Cell::new(None),
            navigator: Cell::new(None),
            image_cache_task: image_cache_task,
            active_timers: Traceable::new(RefCell::new(HashMap::new())),
            next_timer_handle: Traceable::new(Cell::new(0)),
            browser_context: Traceable::new(RefCell::new(None)),
            performance: Cell::new(None),
            navigationStart: time::get_time().sec as u64,
            navigationStartPrecise: time::precise_time_s(),
            screen: Cell::new(None),
        };

        WindowBinding::Wrap(cx, win)
    }
}
