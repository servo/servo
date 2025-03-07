/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use dom_struct::dom_struct;
use profile_traits::mem::MemoryReportResult;
use script_traits::ScriptMsg;

use crate::dom::bindings::codegen::Bindings::ServoInternalsBinding::ServoInternalsMethods;
use crate::dom::bindings::error::Error;
use crate::dom::bindings::reflector::{DomGlobal, Reflector, reflect_dom_object};
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::realms::InRealm;
use crate::routed_promise::{RoutedPromiseListener, route_promise};
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct ServoInternals {
    reflector_: Reflector,
}

impl ServoInternals {
    pub fn new_inherited() -> ServoInternals {
        ServoInternals {
            reflector_: Reflector::new(),
        }
    }

    pub(crate) fn new(global: &GlobalScope, can_gc: CanGc) -> DomRoot<ServoInternals> {
        reflect_dom_object(Box::new(ServoInternals::new_inherited()), global, can_gc)
    }
}

impl ServoInternalsMethods<crate::DomTypeHolder> for ServoInternals {
    /// <https://servo.org/internal-no-spec>
    fn ReportMemory(&self, comp: InRealm, can_gc: CanGc) -> Rc<Promise> {
        let global = &self.global();
        let promise = Promise::new_in_current_realm(comp, can_gc);
        let sender = route_promise(&promise, self);
        let script_to_constellation_chan = global.script_to_constellation_chan();
        if script_to_constellation_chan
            .send(ScriptMsg::ReportMemory(sender))
            .is_err()
        {
            promise.reject_error(Error::Operation, can_gc);
        }
        promise
    }
}

impl RoutedPromiseListener<MemoryReportResult> for ServoInternals {
    fn handle_response(&self, response: MemoryReportResult, promise: &Rc<Promise>, can_gc: CanGc) {
        promise.resolve_native(&response.content, can_gc);
    }
}
