/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use dom_struct::dom_struct;
use js::context::JSContext;
use js::gc::MutableHandleValue;
use js::jsapi::Heap;
use js::jsval::UndefinedValue;
use js::realm::CurrentRealm;
use js::rust::HandleObject;
use profile_traits::mem::MemoryReportResult;
use script_bindings::conversions::SafeToJSValConvertible;
use script_bindings::error::{Error, Fallible};
use script_bindings::interfaces::ServoInternalsHelpers;
use script_bindings::reflector::{Reflector, reflect_dom_object_with_cx};
use script_bindings::str::USVString;
use servo_config::prefs::{self, PrefValue, Preferences};
use servo_constellation_traits::ScriptToConstellationMessage;

use crate::dom::bindings::codegen::Bindings::ServoInternalsBinding::ServoInternalsMethods;
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::routed_promise::{RoutedPromiseListener, callback_promise};
use crate::script_thread::ScriptThread;

fn pref_to_jsval(cx: &mut js::context::JSContext, pref: &PrefValue, rval: MutableHandleValue) {
    match pref {
        PrefValue::Bool(b) => b.safe_to_jsval(cx, rval),
        PrefValue::Int(i) => i.safe_to_jsval(cx, rval),
        PrefValue::UInt(u) => u.safe_to_jsval(cx, rval),
        PrefValue::Str(s) => s.safe_to_jsval(cx, rval),
        PrefValue::Float(f) => f.safe_to_jsval(cx, rval),
        PrefValue::Array(arr) => {
            rooted_vec!(let mut js_arr);
            for item in arr {
                rooted!(&in(cx) let mut js_val = UndefinedValue());
                pref_to_jsval(cx, item, js_val.handle_mut());
                js_arr.push(Heap::boxed(js_val.get()));
            }
            js_arr.safe_to_jsval(cx, rval);
        },
    }
}

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

    pub(crate) fn new(cx: &mut JSContext, global: &GlobalScope) -> DomRoot<ServoInternals> {
        reflect_dom_object_with_cx(Box::new(ServoInternals::new_inherited()), global, cx)
    }
}

impl ServoInternalsMethods<crate::DomTypeHolder> for ServoInternals {
    /// <https://servo.org/internal-no-spec>
    fn ReportMemory(&self, cx: &mut CurrentRealm) -> Rc<Promise> {
        let promise = Promise::new_in_realm(cx);
        let global = self.global();
        let task_manager = global.task_manager();
        let task_source = task_manager.dom_manipulation_task_source();
        let callback = callback_promise(&promise, self, task_source);

        let script_to_constellation_chan = global.script_to_constellation_chan();
        if script_to_constellation_chan
            .send(ScriptToConstellationMessage::ReportMemory(callback))
            .is_err()
        {
            promise.reject_error(cx, Error::Operation(None));
        }
        promise
    }

    /// <https://servo.org/internal-no-spec>
    fn GarbageCollectAllContexts(&self) {
        let global = &self.global();

        let script_to_constellation_chan = global.script_to_constellation_chan();
        let _ = script_to_constellation_chan
            .send(ScriptToConstellationMessage::TriggerGarbageCollection);
    }

    /// <https://servo.org/internal-no-spec>
    fn PreferenceList(&self) -> Vec<USVString> {
        Preferences::all_fields()
            .into_iter()
            .map(|s| USVString::from(s.to_string()))
            .collect()
    }

    /// <https://servo.org/internal-no-spec>
    fn PreferenceType(&self, name: USVString) -> Fallible<USVString> {
        if !Preferences::exists(&name) {
            return Err(Error::NotFound(None));
        }
        let type_name = Preferences::type_of(&name).split("::").last().unwrap();
        Ok(USVString::from(type_name.to_string()))
    }

    /// <https://servo.org/internal-no-spec>
    fn DefaultPreferenceValue(
        &self,
        cx: &mut JSContext,
        name: USVString,
        rval: MutableHandleValue,
    ) -> Fallible<()> {
        if !Preferences::exists(&name) {
            return Err(Error::NotFound(None));
        }
        let pref = Preferences::default().get_value(&name);
        pref_to_jsval(cx, &pref, rval);
        Ok(())
    }

    /// <https://servo.org/internal-no-spec>
    fn GetPreference(
        &self,
        cx: &mut JSContext,
        name: USVString,
        rval: MutableHandleValue,
    ) -> Fallible<()> {
        if !Preferences::exists(&name) {
            return Err(Error::NotFound(None));
        }
        let pref = prefs::get().get_value(&name);
        pref_to_jsval(cx, &pref, rval);
        Ok(())
    }

    /// <https://servo.org/internal-no-spec>
    fn GetBoolPreference(&self, name: USVString) -> Fallible<bool> {
        if !Preferences::exists(&name) {
            return Err(Error::NotFound(None));
        }
        if let PrefValue::Bool(b) = prefs::get().get_value(&name) {
            return Ok(b);
        }
        Err(Error::TypeMismatch(None))
    }

    /// <https://servo.org/internal-no-spec>
    fn GetIntPreference(&self, name: USVString) -> Fallible<i64> {
        if !Preferences::exists(&name) {
            return Err(Error::NotFound(None));
        }
        if let PrefValue::Int(i) = prefs::get().get_value(&name) {
            return Ok(i);
        }
        Err(Error::TypeMismatch(None))
    }

    /// <https://servo.org/internal-no-spec>
    fn GetStringPreference(&self, name: USVString) -> Fallible<USVString> {
        if !Preferences::exists(&name) {
            return Err(Error::NotFound(None));
        }
        if let PrefValue::Str(s) = prefs::get().get_value(&name) {
            return Ok(s.into());
        }
        Err(Error::TypeMismatch(None))
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
    fn handle_response(
        &self,
        cx: &mut JSContext,
        response: MemoryReportResult,
        promise: &Rc<Promise>,
    ) {
        let stringified = serde_json::to_string(&response.results)
            .unwrap_or_else(|_| "{ error: \"failed to create memory report\"}".to_owned());
        promise.resolve_native(cx, &stringified);
    }
}

impl ServoInternalsHelpers for ServoInternals {
    /// The navigator.servo api is exposed to about: pages except about:blank, as
    /// well as any URLs provided by embedders that register new protocol handlers.
    fn is_servo_internal(cx: &mut JSContext, _global: HandleObject) -> bool {
        let realm = CurrentRealm::assert(cx);
        let global_scope = GlobalScope::from_current_realm(&realm);
        let url = global_scope.get_url();
        (url.scheme() == "about" && url.as_str() != "about:blank") ||
            ScriptThread::is_servo_privileged(url) ||
            prefs::get().expose_servointernals_globally
    }
}
