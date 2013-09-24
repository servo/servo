/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::WindowBinding;
use dom::bindings::utils::{WrapperCache, DOMString, Traceable};
use dom::bindings::utils::{CacheableWrapper, BindingObject, null_str_as_empty};
use dom::document::AbstractDocument;
use dom::node::{AbstractNode, ScriptView};
use dom::navigator::Navigator;

use layout_interface::ReflowForDisplay;
use script_task::{ExitWindowMsg, FireTimerMsg, Page, ScriptChan};
use servo_msg::compositor_msg::ScriptListener;
use servo_net::image_cache_task::ImageCacheTask;

use js::glue::*;
use js::jsapi::{JSObject, JSContext, JS_DefineProperty, JS_CallTracer};
use js::jsapi::{JSPropertyOp, JSStrictPropertyOp, JSTracer, JSTRACE_OBJECT};
use js::{JSVAL_NULL, JSPROP_ENUMERATE};

use std::cast;
use std::cell::Cell;
use std::comm;
use std::comm::SharedChan;
use std::hashmap::HashSet;
use std::io;
use std::ptr;
use std::int;
use std::libc;
use std::rt::rtio::RtioTimer;
use std::rt::io::timer::Timer;
use std::task::spawn_with;
use js::jsapi::JSVal;

pub enum TimerControlMsg {
    TimerMessage_Fire(~TimerData),
    TimerMessage_Close,
    TimerMessage_TriggerExit //XXXjdm this is just a quick hack to talk to the script task
}

pub struct Window {
    page: @mut Page,
    script_chan: ScriptChan,
    compositor: @ScriptListener,
    wrapper: WrapperCache,
    timer_chan: SharedChan<TimerControlMsg>,
    navigator: Option<@mut Navigator>,
    image_cache_task: ImageCacheTask,
    active_timers: ~HashSet<i32>,
    next_timer_handle: i32,
}

#[unsafe_destructor]
impl Drop for Window {
    fn drop(&self) {
        self.timer_chan.send(TimerMessage_Close);
    }
}

// Holder for the various JS values associated with setTimeout
// (ie. function value to invoke and all arguments to pass
//      to the function when calling it)
pub struct TimerData {
    handle: i32,
    funval: JSVal,
    args: ~[JSVal],
}

impl Window {
    pub fn Alert(&self, s: &DOMString) {
        // Right now, just print to the console
        io::println(fmt!("ALERT: %s", null_str_as_empty(s)));
    }

    pub fn Close(&self) {
        self.timer_chan.send(TimerMessage_TriggerExit);
    }

    pub fn Document(&self) -> AbstractDocument {
        self.page.frame.unwrap().document
    }

    pub fn Name(&self) -> DOMString {
        None
    }

    pub fn SetName(&self, _name: &DOMString) {
    }

    pub fn Status(&self) -> DOMString {
        None
    }

    pub fn SetStatus(&self, _status: &DOMString) {
    }

    pub fn Closed(&self) -> bool {
        false
    }

    pub fn Stop(&self) {
    }

    pub fn Focus(&self) {
    }

    pub fn Blur(&self) {
    }

    pub fn GetFrameElement(&self) -> Option<AbstractNode<ScriptView>> {
        None
    }

    pub fn Navigator(&mut self) -> @mut Navigator {
        if self.navigator.is_none() {
            self.navigator = Some(Navigator::new());
        }
        self.navigator.unwrap()
    }

    pub fn Confirm(&self, _message: &DOMString) -> bool {
        false
    }

    pub fn Prompt(&self, _message: &DOMString, _default: &DOMString) -> DOMString {
        None
    }

    pub fn Print(&self) {
    }

    pub fn ShowModalDialog(&self, _cx: *JSContext, _url: &DOMString, _argument: JSVal) -> JSVal {
        JSVAL_NULL
    }
}

impl CacheableWrapper for Window {
    fn get_wrappercache(&mut self) -> &mut WrapperCache {
        unsafe { cast::transmute(&self.wrapper) }
    }

    fn wrap_object_shared(@mut self, cx: *JSContext, scope: *JSObject) -> *JSObject {
        let mut unused = false;
        WindowBinding::Wrap(cx, scope, self, &mut unused)
    }
}

impl BindingObject for Window {
    fn GetParentObject(&self, _cx: *JSContext) -> Option<@mut CacheableWrapper> {
        None
    }
}

impl Window {
    pub fn SetTimeout(&mut self, _cx: *JSContext, callback: JSVal, timeout: i32) -> i32 {
        let timeout = int::max(0, timeout) as u64;
        let handle = self.next_timer_handle;
        self.next_timer_handle += 1;

        // Post a delayed message to the per-window timer task; it will dispatch it
        // to the relevant script handler that will deal with it.
        let tm = Cell::new(Timer::new().unwrap());
        let chan = self.timer_chan.clone();
        do spawn {
            let mut tm = tm.take();
            tm.sleep(timeout);
            chan.send(TimerMessage_Fire(~TimerData {
                handle: handle,
                funval: callback,
                args: ~[]
            }));
        }
        self.active_timers.insert(handle);
        handle
    }

    pub fn ClearTimeout(&mut self, handle: i32) {
        self.active_timers.remove(&handle);
    }

    pub fn content_changed(&self) {
        // FIXME This should probably be ReflowForQuery, not Display. All queries currently
        // currently rely on the display list, which means we can't destroy it by
        // doing a query reflow.
        self.page.reflow_all(ReflowForDisplay, self.script_chan.clone(), self.compositor);
    }

    pub fn wait_until_safe_to_modify_dom(&self) {
        // FIXME: This disables concurrent layout while we are modifying the DOM, since
        //        our current architecture is entirely unsafe in the presence of races.
        self.page.join_layout();
    }

    #[fixed_stack_segment]
    pub fn new(cx: *JSContext,
               page: @mut Page,
               script_chan: ScriptChan,
               compositor: @ScriptListener,
               image_cache_task: ImageCacheTask)
               -> @mut Window {
        let win = @mut Window {
            page: page,
            script_chan: script_chan.clone(),
            compositor: compositor,
            wrapper: WrapperCache::new(),
            timer_chan: {
                let (timer_port, timer_chan) = comm::stream::<TimerControlMsg>();
                let id = page.id.clone();
                do spawn_with(script_chan) |script_chan| {
                    loop {
                        match timer_port.recv() {
                            TimerMessage_Close => break,
                            TimerMessage_Fire(td) => script_chan.send(FireTimerMsg(id, td)),
                            TimerMessage_TriggerExit => script_chan.send(ExitWindowMsg(id)),
                        }
                    }
                }
                SharedChan::new(timer_chan)
            },
            navigator: None,
            image_cache_task: image_cache_task,
            active_timers: ~HashSet::new(),
            next_timer_handle: 0
        };

        unsafe {
            let cache = ptr::to_unsafe_ptr(win.get_wrappercache());
            win.wrap_object_shared(cx, ptr::null()); //XXXjdm proper scope
            let global = (*cache).wrapper;
            do "window".to_c_str().with_ref |name| {
                JS_DefineProperty(cx, global,  name,
                                  RUST_OBJECT_TO_JSVAL(global),
                                  Some(GetJSClassHookStubPointer(PROPERTY_STUB) as JSPropertyOp),
                                  Some(GetJSClassHookStubPointer(STRICT_PROPERTY_STUB) as JSStrictPropertyOp),
                                  JSPROP_ENUMERATE);
            }
        }
        win
    }
}

impl Traceable for Window {
    #[fixed_stack_segment]
    fn trace(&self, tracer: *mut JSTracer) {
        debug!("tracing window");
        unsafe {
            match self.page.frame {
                Some(frame) => {
                    do frame.document.with_base |doc| {
                        (*tracer).debugPrinter = ptr::null();
                        (*tracer).debugPrintIndex = -1;
                        do "document".to_c_str().with_ref |name| {
                            (*tracer).debugPrintArg = name as *libc::c_void;
                            debug!("tracing document");
                            JS_CallTracer(tracer as *JSTracer,
                                          doc.wrapper.wrapper,
                                          JSTRACE_OBJECT as u32);
                        }
                    }
                }
                None => ()
            }
        }
    }
}
