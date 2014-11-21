/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::EventHandlerBinding::{OnErrorEventHandlerNonNull, EventHandlerNonNull};
use dom::bindings::codegen::Bindings::FunctionBinding::Function;
use dom::bindings::codegen::Bindings::WindowBinding;
use dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use dom::bindings::codegen::InheritTypes::EventTargetCast;
use dom::bindings::error::{Fallible, InvalidCharacter};
use dom::bindings::global;
use dom::bindings::js::{MutNullableJS, JSRef, Temporary, OptionalSettable};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::browsercontext::BrowserContext;
use dom::console::Console;
use dom::document::Document;
use dom::eventtarget::{EventTarget, WindowTypeId, EventTargetHelpers};
use dom::location::Location;
use dom::navigator::Navigator;
use dom::performance::Performance;
use dom::screen::Screen;
use layout_interface::NoQuery;
use page::Page;
use script_task::{ExitWindowMsg, ScriptChan, TriggerLoadMsg, TriggerFragmentMsg};
use script_task::FromWindow;
use script_traits::ScriptControlChan;
use timers::{Interval, NonInterval, TimerId, TimerManager};

use servo_msg::compositor_msg::ScriptListener;
use servo_msg::constellation_msg::LoadData;
use servo_net::image_cache_task::ImageCacheTask;
use servo_util::str::{DOMString,HTML_SPACE_CHARACTERS};

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
use std::rc::Rc;
use time;

#[dom_struct]
pub struct Window {
    eventtarget: EventTarget,
    script_chan: ScriptChan,
    control_chan: ScriptControlChan,
    console: MutNullableJS<Console>,
    location: MutNullableJS<Location>,
    navigator: MutNullableJS<Navigator>,
    image_cache_task: ImageCacheTask,
    compositor: DOMRefCell<Box<ScriptListener+'static>>,
    browser_context: DOMRefCell<Option<BrowserContext>>,
    page: Rc<Page>,
    performance: MutNullableJS<Performance>,
    navigation_start: u64,
    navigation_start_precise: f64,
    screen: MutNullableJS<Screen>,
    timers: TimerManager
}

impl Window {
    pub fn get_cx(&self) -> *mut JSContext {
        let js_info = self.page().js_info();
        (*js_info.as_ref().unwrap().js_context).ptr
    }

    pub fn script_chan<'a>(&'a self) -> &'a ScriptChan {
        &self.script_chan
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

    pub fn navigation_start(&self) -> u64 {
        self.navigation_start
    }

    pub fn navigation_start_precise(&self) -> f64 {
        self.navigation_start_precise
    }

    pub fn get_url(&self) -> Url {
        self.page().get_url()
    }
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
            let page = self.page.clone();
            let location = Location::new(self, page);
            self.location.assign(Some(location));
        }
        self.location.get().unwrap()
    }

    fn Console(self) -> Temporary<Console> {
        if self.console.get().is_none() {
            let console = Console::new(&global::Window(self));
            self.console.assign(Some(console));
        }
        self.console.get().unwrap()
    }

    fn Navigator(self) -> Temporary<Navigator> {
        if self.navigator.get().is_none() {
            let navigator = Navigator::new(self);
            self.navigator.assign(Some(navigator));
        }
        self.navigator.get().unwrap()
    }

    fn SetTimeout(self, _cx: *mut JSContext, callback: Function, timeout: i32, args: Vec<JSVal>) -> i32 {
        self.timers.set_timeout_or_interval(callback,
                                            args,
                                            timeout,
                                            NonInterval,
                                            FromWindow(self.page.id.clone()),
                                            self.script_chan.clone())
    }

    fn ClearTimeout(self, handle: i32) {
        self.timers.clear_timeout_or_interval(handle);
    }

    fn SetInterval(self, _cx: *mut JSContext, callback: Function, timeout: i32, args: Vec<JSVal>) -> i32 {
        self.timers.set_timeout_or_interval(callback,
                                            args,
                                            timeout,
                                            Interval,
                                            FromWindow(self.page.id.clone()),
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

    fn Parent(self) -> Temporary<Window> {
        //TODO - Once we support iframes correctly this needs to return the parent frame
        self.Window()
    }

    fn Performance(self) -> Temporary<Performance> {
        if self.performance.get().is_none() {
            let performance = Performance::new(self);
            self.performance.assign(Some(performance));
        }
        self.performance.get().unwrap()
    }

    event_handler!(click, GetOnclick, SetOnclick)
    event_handler!(load, GetOnload, SetOnload)
    event_handler!(unload, GetOnunload, SetOnunload)
    error_event_handler!(error, GetOnerror, SetOnerror)

    fn Screen(self) -> Temporary<Screen> {
        if self.screen.get().is_none() {
            let screen = Screen::new(self);
            self.screen.assign(Some(screen));
        }
        self.screen.get().unwrap()
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
    fn reflow(self);
    fn flush_layout(self);
    fn wait_until_safe_to_modify_dom(self);
    fn init_browser_context(self, doc: JSRef<Document>);
    fn load_url(self, href: DOMString);
    fn handle_fire_timer(self, timer_id: TimerId);
    fn evaluate_js_with_result(self, code: &str) -> JSVal;
    fn evaluate_script_with_result(self, code: &str, filename: &str) -> JSVal;
}


impl<'a> WindowHelpers for JSRef<'a, Window> {
    fn evaluate_js_with_result(self, code: &str) -> JSVal {
        self.evaluate_script_with_result(code, "")
    }

    fn evaluate_script_with_result(self, code: &str, filename: &str) -> JSVal {
        let global = self.reflector().get_jsobject();
        let code: Vec<u16> = code.as_slice().utf16_units().collect();
        let mut rval = UndefinedValue();
        let filename = filename.to_c_str();
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

    fn reflow(self) {
        self.page().damage();
    }

    fn flush_layout(self) {
        self.page().flush_layout(NoQuery);
    }

    fn wait_until_safe_to_modify_dom(self) {
        // FIXME: This disables concurrent layout while we are modifying the DOM, since
        //        our current architecture is entirely unsafe in the presence of races.
        self.page().join_layout();
    }

    fn init_browser_context(self, doc: JSRef<Document>) {
        *self.browser_context.borrow_mut() = Some(BrowserContext::new(doc));
    }

    /// Commence a new URL load which will either replace this window or scroll to a fragment.
    fn load_url(self, href: DOMString) {
        let base_url = self.page().get_url();
        debug!("current page url is {}", base_url);
        let url = UrlParser::new().base_url(&base_url).parse(href.as_slice());
        // FIXME: handle URL parse errors more gracefully.
        let url = url.unwrap();
        let ScriptChan(ref script_chan) = self.script_chan;
        if href.as_slice().starts_with("#") {
            script_chan.send(TriggerFragmentMsg(self.page.id, url));
        } else {
            script_chan.send(TriggerLoadMsg(self.page.id, LoadData::new(url)));
        }
    }

    fn handle_fire_timer(self, timer_id: TimerId) {
        self.timers.fire_timer(timer_id, self.clone());
        self.flush_layout();
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
            console: Default::default(),
            compositor: DOMRefCell::new(compositor),
            page: page,
            location: Default::default(),
            navigator: Default::default(),
            image_cache_task: image_cache_task,
            browser_context: DOMRefCell::new(None),
            performance: Default::default(),
            navigation_start: time::get_time().sec as u64,
            navigation_start_precise: time::precise_time_s(),
            screen: Default::default(),
            timers: TimerManager::new()
        };

        WindowBinding::Wrap(cx, win)
    }
}
