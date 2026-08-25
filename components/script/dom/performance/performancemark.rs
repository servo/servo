/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::gc::HandleValue;
use js::jsapi::Heap;
use js::jsval::{JSVal, NullValue};
use js::rust::{HandleObject, MutableHandleValue};
use script_bindings::codegen::GenericBindings::PerformanceBinding::PerformanceMarkOptions;
use script_bindings::reflector::reflect_dom_object_with_proto;
use servo_base::cross_process_instant::CrossProcessInstant;
use time::Duration;

use crate::dom::PERFORMANCE_TIMING_ATTRIBUTES;
use crate::dom::bindings::codegen::Bindings::PerformanceMarkBinding::PerformanceMarkMethods;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::bindings::structuredclone;
use crate::dom::bindings::trace::RootedTraceableBox;
use crate::dom::globalscope::GlobalScope;
use crate::dom::performance::performanceentry::{EntryType, PerformanceEntry};
use crate::dom::window::Window;

#[dom_struct]
pub(crate) struct PerformanceMark {
    entry: PerformanceEntry,
    #[ignore_malloc_size_of = "Defined in rust-mozjs"]
    detail: Heap<JSVal>,
}

impl PerformanceMark {
    fn new_inherited(
        name: DOMString,
        start_time: CrossProcessInstant,
        duration: Duration,
    ) -> PerformanceMark {
        PerformanceMark {
            entry: PerformanceEntry::new_inherited(
                name,
                EntryType::Mark,
                Some(start_time),
                duration,
            ),
            detail: Default::default(),
        }
    }

    fn set_detail(&self, handle: HandleValue<'_>) {
        self.detail.set(handle.get());
    }

    pub(crate) fn new_with_proto(
        cx: &mut js::context::JSContext,
        global: &GlobalScope,
        proto: Option<HandleObject>,
        name: DOMString,
        start_time: CrossProcessInstant,
        duration: Duration,
    ) -> DomRoot<PerformanceMark> {
        reflect_dom_object_with_proto(
            cx,
            Box::new(PerformanceMark::new_inherited(name, start_time, duration)),
            global,
            proto,
        )
    }
}

impl PerformanceMarkMethods<crate::DomTypeHolder> for PerformanceMark {
    /// <https://w3c.github.io/user-timing/#dom-performancemark-detail>
    fn Detail(&self, mut retval: MutableHandleValue) {
        retval.set(self.detail.get())
    }

    /// <https://w3c.github.io/user-timing/#the-performancemark-constructor>
    fn Constructor(
        cx: &mut js::context::JSContext,
        global: &GlobalScope,
        proto: Option<HandleObject>,
        mark_name: DOMString,
        mark_options: RootedTraceableBox<PerformanceMarkOptions>,
    ) -> Fallible<DomRoot<PerformanceMark>> {
        // The PerformanceMark constructor must run the following steps:
        // Step 1. If the current global object is a Window object and markName uses the same name
        // as a read only attribute in the PerformanceTiming interface, throw a SyntaxError.
        if global.is::<Window>() && PERFORMANCE_TIMING_ATTRIBUTES.contains(&&*mark_name.str()) {
            return Err(Error::Syntax(Some(
                "Read-only attribute cannot be used as a mark name".to_owned(),
            )));
        }

        // Step 2 - 4. Note: These are handled by the PerformanceMark default constructor below.

        // Step 5. Set entry’s startTime attribute as follows:
        let start_time = match mark_options.startTime {
            // Step 5.1. If markOptions’s startTime member exists, then:
            Some(start_time) => {
                // Step 5.1.1. If markOptions’s startTime is negative, throw a TypeError.
                if start_time.is_sign_negative() {
                    return Err(Error::Type(c"startTime must not be negative".to_owned()));
                }
                // Step 5.1.2. Otherwise, set entry’s startTime to the value of markOptions’s startTime.
                global.performance(cx).time_origin() +
                    Duration::microseconds(start_time.mul_add(1000.0, 0.0) as i64)
            },
            // Step 5.2. Otherwise, set it to the value that would be returned by the Performance object’s now() method.
            None => CrossProcessInstant::now(),
        };

        // Step 6. Set entry’s duration attribute to 0.
        let entry = PerformanceMark::new_with_proto(
            cx,
            global,
            proto,
            mark_name,
            start_time,
            Duration::ZERO,
        );

        // Step 7. If markOptions’s detail is null, set entry’s detail to null.
        rooted!(&in(cx) let mut detail = NullValue());

        // Step 8 Otherwise:
        if !mark_options.detail.get().is_null_or_undefined() {
            // Step 8.1. Let record be the result of calling the StructuredSerialize algorithm on markOptions’s detail.
            let record = structuredclone::write(cx, mark_options.detail.handle(), None)?;

            // Step 8.2. Set entry’s detail to the result of calling the StructuredDeserialize algorithm on record and the current realm.
            structuredclone::read(cx, global, record, detail.handle_mut())?;
        }
        entry.set_detail(detail.handle());

        Ok(entry)
    }
}
