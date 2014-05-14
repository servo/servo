/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::WindowBinding;
use dom::bindings::js::{JS, JSRef, Temporary, OptionalSettable};
use dom::bindings::trace::{Traceable, Untraceable};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::browsercontext::BrowserContext;
use dom::document::Document;
use dom::element::Element;
use dom::eventtarget::{EventTarget, WindowTypeId};
use dom::console::Console;
use dom::location::Location;
use dom::navigator::Navigator;
use dom::performance::Performance;

use layout_interface::{ReflowForDisplay, DocumentDamageLevel};
use script_task::{ExitWindowMsg, FireTimerMsg, Page, ScriptChan};
use servo_msg::compositor_msg::ScriptListener;
use servo_net::image_cache_task::ImageCacheTask;
use servo_util::str::DOMString;
use servo_util::task::{spawn_named};

use js::jsapi::JSContext;
use js::jsapi::{JS_GC, JS_GetRuntime};
use js::jsval::{NullValue, JSVal};

use collections::hashmap::HashMap;
use std::cmp;
use std::comm::{channel, Sender};
use std::comm::Select;
use std::hash::{Hash, sip};
use std::io::timer::Timer;
use std::rc::Rc;

use time;

use serialize::{Encoder, Encodable};
use url::Url;

#[deriving(Eq, Encodable, TotalEq)]
pub struct TimerId(i32);

#[deriving(Encodable)]
pub struct TimerHandle {
    pub handle: TimerId,
    pub data: TimerData,
    pub cancel_chan: Untraceable<Option<Sender<()>>>,
}

impl Hash for TimerId {
    fn hash(&self, state: &mut sip::SipState) {
        let TimerId(id) = *self;
        id.hash(state);
    }
}

impl TimerHandle {
    fn cancel(&mut self) {
        self.cancel_chan.as_ref().map(|chan| chan.send(()));
    }
}

#[deriving(Encodable)]
pub struct Window {
    pub eventtarget: EventTarget,
    pub script_chan: ScriptChan,
    pub console: Option<JS<Console>>,
    pub location: Option<JS<Location>>,
    pub navigator: Option<JS<Navigator>>,
    pub image_cache_task: ImageCacheTask,
    pub active_timers: HashMap<TimerId, TimerHandle>,
    pub next_timer_handle: i32,
    pub compositor: Untraceable<~ScriptListener>,
    pub browser_context: Option<BrowserContext>,
    pub page: Rc<Page>,
    pub performance: Option<JS<Performance>>,
    pub navigationStart: u64,
    pub navigationStartPrecise: f64,
}

impl Window {
    pub fn get_cx(&self) -> *JSContext {
        let js_info = self.page().js_info();
        (**js_info.get_ref().js_context).ptr
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
        for (_, timer_handle) in self.active_timers.mut_iter() {
            timer_handle.cancel();
        }
    }
}

// Holder for the various JS values associated with setTimeout
// (ie. function value to invoke and all arguments to pass
//      to the function when calling it)
#[deriving(Encodable)]
pub struct TimerData {
    pub is_interval: bool,
    pub funval: Traceable<JSVal>,
}

pub trait WindowMethods {
    fn Alert(&self, s: DOMString);
    fn Close(&self);
    fn Document(&self) -> Temporary<Document>;
    fn Name(&self) -> DOMString;
    fn SetName(&self, _name: DOMString);
    fn Status(&self) -> DOMString;
    fn SetStatus(&self, _status: DOMString);
    fn Closed(&self) -> bool;
    fn Stop(&self);
    fn Focus(&self);
    fn Blur(&self);
    fn GetFrameElement(&self) -> Option<Temporary<Element>>;
    fn Location(&mut self) -> Temporary<Location>;
    fn Console(&mut self) -> Temporary<Console>;
    fn Navigator(&mut self) -> Temporary<Navigator>;
    fn Confirm(&self, _message: DOMString) -> bool;
    fn Prompt(&self, _message: DOMString, _default: DOMString) -> Option<DOMString>;
    fn Print(&self);
    fn ShowModalDialog(&self, _cx: *JSContext, _url: DOMString, _argument: Option<JSVal>) -> JSVal;
    fn SetTimeout(&mut self, _cx: *JSContext, callback: JSVal, timeout: i32) -> i32;
    fn ClearTimeout(&mut self, handle: i32);
    fn SetInterval(&mut self, _cx: *JSContext, callback: JSVal, timeout: i32) -> i32;
    fn ClearInterval(&mut self, handle: i32);
    fn Window(&self) -> Temporary<Window>;
    fn Self(&self) -> Temporary<Window>;
    fn Performance(&mut self) -> Temporary<Performance>;
    fn Debug(&self, message: DOMString);
    fn Gc(&self);
}

impl<'a> WindowMethods for JSRef<'a, Window> {
    fn Alert(&self, s: DOMString) {
        // Right now, just print to the console
        println!("ALERT: {:s}", s);
    }

    fn Close(&self) {
        let ScriptChan(ref chan) = self.script_chan;
        chan.send(ExitWindowMsg(self.page.id.clone()));
    }

    fn Document(&self) -> Temporary<Document> {
        let frame = self.page().frame();
        Temporary::new(frame.get_ref().document.clone())
    }

    fn Name(&self) -> DOMString {
        "".to_owned()
    }

    fn SetName(&self, _name: DOMString) {
    }

    fn Status(&self) -> DOMString {
        "".to_owned()
    }

    fn SetStatus(&self, _status: DOMString) {
    }

    fn Closed(&self) -> bool {
        false
    }

    fn Stop(&self) {
    }

    fn Focus(&self) {
    }

    fn Blur(&self) {
    }

    fn GetFrameElement(&self) -> Option<Temporary<Element>> {
        None
    }

    fn Location(&mut self) -> Temporary<Location> {
        if self.location.is_none() {
            let page = self.deref().page.clone();
            let location = Location::new(self, page);
            self.location.assign(Some(location));
        }
        Temporary::new(self.location.get_ref().clone())
    }

    fn Console(&mut self) -> Temporary<Console> {
        if self.console.is_none() {
            let console = Console::new(self);
            self.console.assign(Some(console));
        }
        Temporary::new(self.console.get_ref().clone())
    }

    fn Navigator(&mut self) -> Temporary<Navigator> {
        if self.navigator.is_none() {
            let navigator = Navigator::new(self);
            self.navigator.assign(Some(navigator));
        }
        Temporary::new(self.navigator.get_ref().clone())
    }

    fn Confirm(&self, _message: DOMString) -> bool {
        false
    }

    fn Prompt(&self, _message: DOMString, _default: DOMString) -> Option<DOMString> {
        None
    }

    fn Print(&self) {
    }

    fn ShowModalDialog(&self, _cx: *JSContext, _url: DOMString, _argument: Option<JSVal>) -> JSVal {
        NullValue()
    }

    fn SetTimeout(&mut self, _cx: *JSContext, callback: JSVal, timeout: i32) -> i32 {
        self.set_timeout_or_interval(callback, timeout, false)
    }

    fn ClearTimeout(&mut self, handle: i32) {
        let mut timer_handle = self.active_timers.pop(&TimerId(handle));
        match timer_handle {
            Some(ref mut handle) => handle.cancel(),
            None => { }
        }
        self.active_timers.remove(&TimerId(handle));
    }

    fn SetInterval(&mut self, _cx: *JSContext, callback: JSVal, timeout: i32) -> i32 {
        self.set_timeout_or_interval(callback, timeout, true)
    }

    fn ClearInterval(&mut self, handle: i32) {
        self.ClearTimeout(handle);
    }

    fn Window(&self) -> Temporary<Window> {
        Temporary::from_rooted(self)
    }

    fn Self(&self) -> Temporary<Window> {
        self.Window()
    }

    fn Performance(&mut self) -> Temporary<Performance> {
        if self.performance.is_none() {
            let performance = Performance::new(self);
            self.performance.assign(Some(performance));
        }
        Temporary::new(self.performance.get_ref().clone())
    }

    fn Debug(&self, message: DOMString) {
        debug!("{:s}", message);
    }

    fn Gc(&self) {
        unsafe {
            JS_GC(JS_GetRuntime(self.get_cx()));
        }
    }
}

impl Reflectable for Window {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.eventtarget.reflector()
    }

    fn mut_reflector<'a>(&'a mut self) -> &'a mut Reflector {
        self.eventtarget.mut_reflector()
    }
}

pub trait WindowHelpers {
    fn damage_and_reflow(&self, damage: DocumentDamageLevel);
    fn wait_until_safe_to_modify_dom(&self);
    fn init_browser_context(&mut self, doc: &JSRef<Document>);
}

trait PrivateWindowHelpers {
    fn set_timeout_or_interval(&mut self, callback: JSVal, timeout: i32, is_interval: bool) -> i32;
}

impl<'a> WindowHelpers for JSRef<'a, Window> {
    fn damage_and_reflow(&self, damage: DocumentDamageLevel) {
        // FIXME This should probably be ReflowForQuery, not Display. All queries currently
        // currently rely on the display list, which means we can't destroy it by
        // doing a query reflow.
        self.page().damage(damage);
        self.page().reflow(ReflowForDisplay, self.script_chan.clone(), *self.compositor);
    }

    fn wait_until_safe_to_modify_dom(&self) {
        // FIXME: This disables concurrent layout while we are modifying the DOM, since
        //        our current architecture is entirely unsafe in the presence of races.
        self.page().join_layout();
    }

    fn init_browser_context(&mut self, doc: &JSRef<Document>) {
        self.browser_context = Some(BrowserContext::new(doc));
    }
}

impl<'a> PrivateWindowHelpers for JSRef<'a, Window> {
    fn set_timeout_or_interval(&mut self, callback: JSVal, timeout: i32, is_interval: bool) -> i32 {
        let timeout = cmp::max(0, timeout) as u64;
        let handle = self.next_timer_handle;
        self.next_timer_handle += 1;

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
            let timeout_port = if is_interval {
                tm.periodic(timeout)
            } else {
                tm.oneshot(timeout)
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
        self.active_timers.insert(timer_id, timer);
        handle
    }
}

impl Window {
    pub fn new(cx: *JSContext,
               page: Rc<Page>,
               script_chan: ScriptChan,
               compositor: ~ScriptListener,
               image_cache_task: ImageCacheTask)
               -> JS<Window> {
        let win = ~Window {
            eventtarget: EventTarget::new_inherited(WindowTypeId),
            script_chan: script_chan,
            console: None,
            compositor: Untraceable::new(compositor),
            page: page,
            location: None,
            navigator: None,
            image_cache_task: image_cache_task,
            active_timers: HashMap::new(),
            next_timer_handle: 0,
            browser_context: None,
            performance: None,
            navigationStart: time::get_time().sec as u64,
            navigationStartPrecise: time::precise_time_s(),
        };

        WindowBinding::Wrap(cx, win)
    }
}
