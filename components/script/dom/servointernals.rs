/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use constellation_traits::ScriptToConstellationMessage;
use dom_struct::dom_struct;
use js::rust::HandleObject;
use profile_traits::mem::MemoryReportResult;
use script_bindings::error::{Error, Fallible};
use script_bindings::interfaces::ServoInternalsHelpers;
use script_bindings::script_runtime::JSContext;
use script_bindings::str::USVString;
use servo_config::prefs::{self, PrefValue};

use crate::dom::bindings::codegen::Bindings::ServoInternalsBinding::ServoInternalsMethods;
use crate::dom::bindings::reflector::{DomGlobal, Reflector, reflect_dom_object};
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::realms::{AlreadyInRealm, InRealm};
use crate::routed_promise::{RoutedPromiseListener, route_promise};
use crate::script_runtime::CanGc;
use crate::script_thread::ScriptThread;

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
        let task_source = global.task_manager().dom_manipulation_task_source();
        let sender = route_promise(&promise, self, task_source);

        let script_to_constellation_chan = global.script_to_constellation_chan();
        if script_to_constellation_chan
            .send(ScriptToConstellationMessage::ReportMemory(sender))
            .is_err()
        {
            promise.reject_error(Error::Operation, can_gc);
        }
        promise
    }

    /// <https://servo.org/internal-no-spec>
    fn GetBoolPreference(&self, name: USVString) -> Fallible<bool> {
        if let PrefValue::Bool(b) = prefs::get().get_value(&name) {
            return Ok(b);
        }
        Err(Error::TypeMismatch)
    }

    /// <https://servo.org/internal-no-spec>
    fn GetIntPreference(&self, name: USVString) -> Fallible<i64> {
        if let PrefValue::Int(i) = prefs::get().get_value(&name) {
            return Ok(i);
        }
        Err(Error::TypeMismatch)
    }

    /// <https://servo.org/internal-no-spec>
    fn GetStringPreference(&self, name: USVString) -> Fallible<USVString> {
        if let PrefValue::Str(s) = prefs::get().get_value(&name) {
            return Ok(s.into());
        }
        Err(Error::TypeMismatch)
    }

    /// <https://servo.org/internal-no-spec>
    fn SetBoolPreference(&self, name: USVString, value: bool) {
        let mut current_prefs = prefs::get().clone();
        current_prefs.set_value(&name, value.into());
        prefs::set(current_prefs);
    }

    /// <https://servo.org/internal-no-spec>
    fn SetIntPreference(&self, name: USVString, value: i64) {
        let mut current_prefs = prefs::get().clone();
        current_prefs.set_value(&name, value.into());
        prefs::set(current_prefs);
    }

    /// <https://servo.org/internal-no-spec>
    fn SetStringPreference(&self, name: USVString, value: USVString) {
        let mut current_prefs = prefs::get().clone();
        current_prefs.set_value(&name, value.0.into());
        prefs::set(current_prefs);
    }
}

impl RoutedPromiseListener<MemoryReportResult> for ServoInternals {
    fn handle_response(&self, response: MemoryReportResult, promise: &Rc<Promise>, can_gc: CanGc) {
        let stringified = serde_json::to_string(&response.results)
            .unwrap_or_else(|_| "{ error: \"failed to create memory report\"}".to_owned());
        promise.resolve_native(&stringified, can_gc);
    }
}

impl ServoInternalsHelpers for ServoInternals {
    /// The navigator.servo api is exposed to about: pages except about:blank, as
    /// well as any URLs provided by embedders that register new protocol handlers.
    #[allow(unsafe_code)]
    fn is_servo_internal(cx: JSContext, _global: HandleObject) -> bool {
        unsafe {
            let in_realm_proof = AlreadyInRealm::assert_for_cx(cx);
            let global_scope = GlobalScope::from_context(*cx, InRealm::Already(&in_realm_proof));
            let url = global_scope.get_url();
            (url.scheme() == "about" && url.as_str() != "about:blank") ||
                ScriptThread::is_servo_privileged(url)
        }
    }
}
