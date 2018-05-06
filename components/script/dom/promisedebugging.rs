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
use js::jsapi::{GetPromiseResult, HandleObject};
use js::rust::HandleValue;
use js::rust::wrappers::JS_ErrorFromException;
use std::cell::Cell;

pub struct PromiseDebugging {
  pub id_predix: Cell<String>,
}

impl PromiseDebugging {
    #[allow(unsafe_code)]
    pub fn AddUncaughtRejection(global: DomRoot<GlobalScope>, promise: HandleObject) {
        global.set_uncaught_rejections(promise);

        unsafe {
            PromiseDebugging::flush_uncaught_rejections_internal(global, promise);
        }
    }

    #[allow(unsafe_code)]
    pub unsafe fn flush_uncaught_rejections_internal(global: DomRoot<GlobalScope>, promise: HandleObject) {
        let cx = global.get_cx();

        let result = GetPromiseResult(promise);
        rooted!(in(cx) let object = result.to_object());
        let error_report = JS_ErrorFromException(cx, object.handle());
        println!("{:?}, {:?}", result, error_report);

        let event = PromiseRejectionEvent::new(
            &global.upcast::<GlobalScope>(),
            atom!("unhandledrejection"),
            EventBubbles::DoesNotBubble,
            EventCancelable::Cancelable,
            // FIXME(CYBAI): Use real rejected `promise` and `reason`
            Promise::new(&global.upcast::<GlobalScope>()),
            HandleValue::null()
        );

        event.upcast::<Event>().fire(global.upcast::<EventTarget>());
    }
}
