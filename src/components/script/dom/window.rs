/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::EventHandlerBinding::{OnErrorEventHandlerNonNull, EventHandlerNonNull};
use dom::bindings::codegen::Bindings::WindowBinding;
use dom::bindings::codegen::InheritTypes::EventTargetCast;
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

use layout_interface::{ReflowForDisplay, DocumentDamageLevel};
use script_task::{ExitWindowMsg, FireTimerMsg, Page, ScriptChan, TriggerLoadMsg, TriggerFragmentMsg};
use servo_msg::compositor_msg::ScriptListener;
use servo_net::image_cache_task::ImageCacheTask;
use servo_util::str::DOMString;
use servo_util::task::{spawn_named};
use servo_util::url::parse_url;

use js::jsapi::JSContext;
use js::jsapi::{JS_GC, JS_GetRuntime};
use js::jsval::JSVal;

use collections::hashmap::HashMap;
use std::cell::Cell;
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
    pub console: Cell<Option<JS<Console>>>,
    pub location: Cell<Option<JS<Location>>>,
    pub navigator: Cell<Option<JS<Navigator>>>,
    pub image_cache_task: ImageCacheTask,
    pub active_timers: Box<HashMap<TimerId, TimerHandle>>,
    pub next_timer_handle: i32,
    pub compositor: Untraceable<Box<ScriptListener>>,
    pub browser_context: Option<BrowserContext>,
    pub page: Rc<Page>,
    pub performance: Cell<Option<JS<Performance>>>,
    pub navigationStart: u64,
    pub navigationStartPrecise: f64,
}

impl Window {
    pub fn get_cx(&self) -> *mut JSContext {
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
    fn Location(&self) -> Temporary<Location>;
    fn Console(&self) -> Temporary<Console>;
    fn Navigator(&self) -> Temporary<Navigator>;
    fn SetTimeout(&mut self, _cx: *mut JSContext, callback: JSVal, timeout: i32) -> i32;
    fn ClearTimeout(&mut self, handle: i32);
    fn SetInterval(&mut self, _cx: *mut JSContext, callback: JSVal, timeout: i32) -> i32;
    fn ClearInterval(&mut self, handle: i32);
    fn Window(&self) -> Temporary<Window>;
    fn Self(&self) -> Temporary<Window>;
    fn Performance(&self) -> Temporary<Performance>;
    fn GetOnclick(&self) -> Option<EventHandlerNonNull>;
    fn SetOnclick(&mut self, listener: Option<EventHandlerNonNull>);
    fn GetOnload(&self) -> Option<EventHandlerNonNull>;
    fn SetOnload(&mut self, listener: Option<EventHandlerNonNull>);
    fn GetOnunload(&self) -> Option<EventHandlerNonNull>;
    fn SetOnunload(&mut self, listener: Option<EventHandlerNonNull>);
    fn GetOnerror(&self) -> Option<OnErrorEventHandlerNonNull>;
    fn SetOnerror(&mut self, listener: Option<OnErrorEventHandlerNonNull>);
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

    fn Location(&self) -> Temporary<Location> {
        if self.location.get().is_none() {
            let page = self.deref().page.clone();
            let location = Location::new(self, page);
            self.location.assign(Some(location));
        }
        Temporary::new(self.location.get().get_ref().clone())
    }

    fn Console(&self) -> Temporary<Console> {
        if self.console.get().is_none() {
            let console = Console::new(self);
            self.console.assign(Some(console));
        }
        Temporary::new(self.console.get().get_ref().clone())
    }

    fn Navigator(&self) -> Temporary<Navigator> {
        if self.navigator.get().is_none() {
            let navigator = Navigator::new(self);
            self.navigator.assign(Some(navigator));
        }
        Temporary::new(self.navigator.get().get_ref().clone())
    }

    fn SetTimeout(&mut self, _cx: *mut JSContext, callback: JSVal, timeout: i32) -> i32 {
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

    fn SetInterval(&mut self, _cx: *mut JSContext, callback: JSVal, timeout: i32) -> i32 {
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

    fn Performance(&self) -> Temporary<Performance> {
        if self.performance.get().is_none() {
            let performance = Performance::new(self);
            self.performance.assign(Some(performance));
        }
        Temporary::new(self.performance.get().get_ref().clone())
    }

    fn GetOnclick(&self) -> Option<EventHandlerNonNull> {
        let eventtarget: &JSRef<EventTarget> = EventTargetCast::from_ref(self);
        eventtarget.get_event_handler_common("click")
    }

    fn SetOnclick(&mut self, listener: Option<EventHandlerNonNull>) {
        let eventtarget: &mut JSRef<EventTarget> = EventTargetCast::from_mut_ref(self);
        eventtarget.set_event_handler_common("click", listener)
    }

    fn GetOnload(&self) -> Option<EventHandlerNonNull> {
        let eventtarget: &JSRef<EventTarget> = EventTargetCast::from_ref(self);
        eventtarget.get_event_handler_common("load")
    }

    fn SetOnload(&mut self, listener: Option<EventHandlerNonNull>) {
        let eventtarget: &mut JSRef<EventTarget> = EventTargetCast::from_mut_ref(self);
        eventtarget.set_event_handler_common("load", listener)
    }

    fn GetOnunload(&self) -> Option<EventHandlerNonNull> {
        let eventtarget: &JSRef<EventTarget> = EventTargetCast::from_ref(self);
        eventtarget.get_event_handler_common("unload")
    }

    fn SetOnunload(&mut self, listener: Option<EventHandlerNonNull>) {
        let eventtarget: &mut JSRef<EventTarget> = EventTargetCast::from_mut_ref(self);
        eventtarget.set_event_handler_common("unload", listener)
    }

    fn GetOnerror(&self) -> Option<OnErrorEventHandlerNonNull> {
        let eventtarget: &JSRef<EventTarget> = EventTargetCast::from_ref(self);
        eventtarget.get_event_handler_common("error")
    }

    fn SetOnerror(&mut self, listener: Option<OnErrorEventHandlerNonNull>) {
        let eventtarget: &mut JSRef<EventTarget> = EventTargetCast::from_mut_ref(self);
        eventtarget.set_event_handler_common("error", listener)
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
    fn load_url(&self, href: DOMString);
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

    /// Commence a new URL load which will either replace this window or scroll to a fragment.
    fn load_url(&self, href: DOMString) {
        let base_url = Some(self.page().get_url());
        debug!("current page url is {:?}", base_url);
        let url = parse_url(href.as_slice(), base_url);
        let ScriptChan(ref script_chan) = self.script_chan;
        if href.as_slice().starts_with("#") {
            script_chan.send(TriggerFragmentMsg(self.page.id, url));
        } else {
            script_chan.send(TriggerLoadMsg(self.page.id, url));
        }
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
    pub fn new(cx: *mut JSContext,
               page: Rc<Page>,
               script_chan: ScriptChan,
               compositor: Box<ScriptListener>,
               image_cache_task: ImageCacheTask)
               -> JS<Window> {
        let win = box Window {
            eventtarget: EventTarget::new_inherited(WindowTypeId),
            script_chan: script_chan,
            console: Cell::new(None),
            compositor: Untraceable::new(compositor),
            page: page,
            location: Cell::new(None),
            navigator: Cell::new(None),
            image_cache_task: image_cache_task,
            active_timers: box HashMap::new(),
            next_timer_handle: 0,
            browser_context: None,
            performance: Cell::new(None),
            navigationStart: time::get_time().sec as u64,
            navigationStartPrecise: time::precise_time_s(),
        };

        WindowBinding::Wrap(cx, win)
    }
}
