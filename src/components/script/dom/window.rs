/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::WindowBinding;
use dom::bindings::utils::{WrapperCache, DOMString, null_string};
use dom::bindings::utils::{CacheableWrapper, BindingObject};
use dom::document::AbstractDocument;
use dom::node::{AbstractNode, ScriptView};

use layout_interface::ReflowForScriptQuery;
use script_task::{ExitMsg, FireTimerMsg, Page, ScriptChan};
use servo_msg::compositor_msg::ScriptListener;

use js::glue::*;
use js::jsapi::{JSObject, JSContext};
use js::{JSVAL_NULL, JSPROP_ENUMERATE};

use std::cast;
use std::comm;
use std::comm::Chan;
use std::int;
use std::io;
use std::ptr;
use js::jsapi::JSVal;
use extra::timer;
use extra::uv_global_loop;

pub enum TimerControlMsg {
    TimerMessage_Fire(~TimerData),
    TimerMessage_Close,
    TimerMessage_TriggerExit //XXXjdm this is just a quick hack to talk to the script task
}

//FIXME If we're going to store the page, find a way to do so safely.
pub struct Window {
    page: *mut Page,
    script_chan: ScriptChan,
    compositor: @ScriptListener,
    wrapper: WrapperCache,
    timer_chan: Chan<TimerControlMsg>,
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
    funval: JSVal,
    args: ~[JSVal],
}

impl Window {
    pub fn Alert(&self, s: &DOMString) {
        // Right now, just print to the console
        io::println(fmt!("ALERT: %s", s.to_str()));
    }

    pub fn Close(&self) {
        self.timer_chan.send(TimerMessage_TriggerExit);
    }

    pub fn Document(&self) -> AbstractDocument {
        unsafe {
            (*self.page).frame.get().document
        }
    }

    pub fn Name(&self) -> DOMString {
        null_string
    }

    pub fn SetName(&self, _name: &DOMString) {
    }

    pub fn Status(&self) -> DOMString {
        null_string
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

    pub fn Confirm(&self, _message: &DOMString) -> bool {
        false
    }

    pub fn Prompt(&self, _message: &DOMString, _default: &DOMString) -> DOMString {
        null_string
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
    pub fn SetTimeout(&self, _cx: *JSContext, callback: JSVal, timeout: i32) -> i32 {
        let timeout = int::max(0, timeout) as uint;

        // Post a delayed message to the per-window timer task; it will dispatch it
        // to the relevant script handler that will deal with it.
        let data = ~TimerData {
            funval: callback,
            args: ~[]
        };
        timer::delayed_send(&uv_global_loop::get(),
                            timeout,
                            &self.timer_chan,
                            TimerMessage_Fire(data));
        return 0; //TODO return handle into list of active timers
    }

    pub fn content_changed(&self) {
        unsafe {
            (*self.page).reflow_all(ReflowForScriptQuery, self.script_chan.clone(), self.compositor);
        }
    }

    pub fn new(page: *mut Page, script_chan: ScriptChan, compositor: @ScriptListener)
               -> @mut Window {
        let script_chan_clone = script_chan.clone();
        let win = @mut Window {
            page: page,
            script_chan: script_chan,
            compositor: compositor,
            wrapper: WrapperCache::new(),
            timer_chan: {
                let (timer_port, timer_chan) = comm::stream::<TimerControlMsg>();
                do spawn {
                    loop {
                        match timer_port.recv() {
                            TimerMessage_Close => break,
                            TimerMessage_Fire(td) => unsafe {script_chan_clone.chan.send(FireTimerMsg((*page).id.clone(), td))},
                            TimerMessage_TriggerExit => script_chan_clone.chan.send(ExitMsg),
                        }
                    }
                }
                timer_chan
            },
        };

        unsafe {
            // TODO(tkuehn): This just grabs the top-level page. Need to handle subframes.
            let compartment = (*page).js_info.get_ref().js_compartment;
            let cache = ptr::to_unsafe_ptr(win.get_wrappercache());
            win.wrap_object_shared(compartment.cx.ptr, ptr::null()); //XXXjdm proper scope
            compartment.define_property(~"window",
                                        RUST_OBJECT_TO_JSVAL((*cache).wrapper),
                                        GetJSClassHookStubPointer(PROPERTY_STUB) as *u8,
                                        GetJSClassHookStubPointer(STRICT_PROPERTY_STUB) as *u8,
                                        JSPROP_ENUMERATE);
        }
        win
    }
}

