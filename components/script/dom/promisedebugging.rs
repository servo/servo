/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::inheritance::Castable;
use dom::bindings::root::DomRoot;
use dom::event::{Event, EventBubbles, EventCancelable};
use dom::eventtarget::EventTarget;
use dom::globalscope::GlobalScope;
use dom::promise::Promise;
use dom::promiserejectionevent::PromiseRejectionEvent;
use js::jsapi::{Heap, JSObject, HandleObject as RawHandleObject};
use js::rust::wrappers::GetPromiseResult;
use std::cell::Cell;

pub struct PromiseDebugging {
  pub id_predix: Cell<String>,
}

impl PromiseDebugging {
    #[allow(unsafe_code)]
    pub fn AddUncaughtRejection(global: DomRoot<GlobalScope>, promise: RawHandleObject) {
        global.set_uncaught_rejections(promise);

        unsafe {
            PromiseDebugging::flush_uncaught_rejections_internal(global);
        }
    }

    #[allow(unsafe_code)]
    pub unsafe fn flush_uncaught_rejections_internal(global: DomRoot<GlobalScope>) {
        let cx = global.get_cx();

        global.get_uncaught_rejections().borrow().iter().for_each(|promise: &Box<Heap<*mut JSObject>>| {
            rooted!(in(cx) let result = GetPromiseResult(promise.handle()));
            let event = PromiseRejectionEvent::new(
                &global.upcast::<GlobalScope>(),
                atom!("unhandledrejection"),
                EventBubbles::DoesNotBubble,
                EventCancelable::Cancelable,
                Promise::new_with_js_promise(promise.handle(), cx),
                result.handle()
            );

            event.upcast::<Event>().fire(global.upcast::<EventTarget>());
        });

        global.get_uncaught_rejections().borrow_mut().clear();
    }
}
