/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use js::gc::HandleValue;
use js::jsapi::Heap;
use js::jsval::JSVal;
use js::rust::MutableHandleValue;

use crate::dom::bindings::codegen::Bindings::PerformanceMarkBinding::PerformanceMarkMethods;
use crate::script_runtime::JSContext;

impl_performance_entry_struct!(
    PerformanceMarkBinding,
    PerformanceMark, EntryType::Mark,
    {
        #[ignore_malloc_size_of = "Defined in rust-mozjs"]
        detail: Heap<JSVal>,
    }
);

impl PerformanceMark {
    pub(crate) fn set_detail(&self, handle: HandleValue<'_>) {
        self.detail.set(handle.get());
    }
}

impl PerformanceMarkMethods<crate::DomTypeHolder> for PerformanceMark {
    fn Detail(&self, _cx: JSContext, mut retval: MutableHandleValue) {
        retval.set(self.detail.get())
    }
}
