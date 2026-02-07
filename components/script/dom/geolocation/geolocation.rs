/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use dom_struct::dom_struct;
use js::gc::HandleValue;
use js::jsapi::IsCallable;
use rustc_hash::FxHashSet;
use script_bindings::callback::ExceptionHandling;
use script_bindings::codegen::GenericBindings::GeolocationBinding::Geolocation_Binding::GeolocationMethods;
use script_bindings::codegen::GenericBindings::GeolocationBinding::{
    PositionCallback, PositionErrorCallback, PositionOptions,
};
use script_bindings::codegen::GenericBindings::PermissionStatusBinding::PermissionName;
use script_bindings::codegen::GenericBindings::WindowBinding::WindowMethods;
use script_bindings::domstring::DOMString;
use script_bindings::error::{Error, Fallible};
use script_bindings::reflector::Reflector;
use script_bindings::root::DomRoot;
use script_bindings::script_runtime::CanGc;

use crate::dom::bindings::codegen::DomTypeHolder::DomTypeHolder;
use crate::dom::bindings::import::base::SafeJSContext;
use crate::dom::bindings::reflector::{DomGlobal, reflect_dom_object};
use crate::dom::geolocationpositionerror::GeolocationPositionError;
use crate::dom::globalscope::GlobalScope;

fn cast_error_callback(
    cx: SafeJSContext,
    error_callback: HandleValue,
) -> Fallible<Option<Rc<PositionErrorCallback<DomTypeHolder>>>> {
    if error_callback.get().is_object() {
        let error_callback = error_callback.to_object();
        #[expect(unsafe_code)]
        unsafe {
            if IsCallable(error_callback) {
                Ok(Some(PositionErrorCallback::new(
                    SafeJSContext::from_ptr(cx.raw_cx()),
                    error_callback,
                )))
            } else {
                Err(Error::Type("Value is not callable.".to_string()))
            }
        }
    } else if error_callback.get().is_null_or_undefined() {
        Ok(None)
    } else {
        Err(Error::Type("Value is not an object.".to_string()))
    }
}

#[dom_struct]
pub struct Geolocation {
    reflector_: Reflector,
    /// <https://www.w3.org/TR/geolocation/#dfn-watchids>
    watch_ids: RefCell<FxHashSet<u32>>,
    next_watch_id: Cell<u32>,
}

impl Geolocation {
    fn new_inherited() -> Self {
        Geolocation {
            reflector_: Reflector::new(),
            watch_ids: RefCell::new(FxHashSet::default()),
            next_watch_id: Cell::new(1),
        }
    }

    pub(crate) fn new(global: &GlobalScope, can_gc: CanGc) -> DomRoot<Self> {
        reflect_dom_object(Box::new(Self::new_inherited()), global, can_gc)
    }

    /// <https://www.w3.org/TR/geolocation/#dfn-request-a-position>
    fn request_position(
        &self,
        _success_callback: Rc<PositionCallback<DomTypeHolder>>,
        error_callback: Option<Rc<PositionErrorCallback<DomTypeHolder>>>,
        _options: &PositionOptions,
        watch_id: Option<u32>,
        can_gc: CanGc,
    ) -> Fallible<()> {
        // Step 1. Let watchIDs be geolocation's [[watchIDs]].
        // Step 2. Let document be the geolocation's relevant global object's associated Document.
        let document = self.global().as_window().Document();
        // Step 3. If document is not allowed to use the "geolocation" feature:
        if !document.allowed_to_use_feature(PermissionName::Geolocation) {
            if let Some(id) = watch_id {
                // Step 3.1 If watchId was passed, remove watchId from watchIDs.
                self.watch_ids.borrow_mut().remove(&id);
            }
            // Step 3.2. Call back with error passing errorCallback and PERMISSION_DENIED.
            if let Some(error_callback) = error_callback {
                error_callback.Call_(
                    self,
                    &GeolocationPositionError::permission_denied(
                        &self.global(),
                        DOMString::from("User denied Geolocation".to_string()),
                        can_gc,
                    ),
                    ExceptionHandling::Report,
                    can_gc,
                )?;
            }
            // Step 3.3 Terminate this algorithm.
            return Ok(());
        }
        // Step 4. If geolocation's environment settings object is a non-secure context:
        if !self.global().is_secure_context() {
            if let Some(id) = watch_id {
                // Step 4.1 If watchId was passed, remove watchId from watchIDs.
                self.watch_ids.borrow_mut().remove(&id);
            }
            // Step 4.2. Call back with error passing errorCallback and PERMISSION_DENIED.
            if let Some(error_callback) = error_callback {
                error_callback.Call_(
                    self,
                    &GeolocationPositionError::permission_denied(
                        &self.global(),
                        DOMString::from("Insecure context for Geolocation".to_string()),
                        can_gc,
                    ),
                    ExceptionHandling::Report,
                    can_gc,
                )?;
            }
            // Step 4.3 Terminate this algorithm.
            return Ok(());
        }
        // TODO: Step 5
        // TODO: Step 6. Let descriptor be a new PermissionDescriptor whose name is "geolocation".

        Ok(())
    }
}

impl GeolocationMethods<DomTypeHolder> for Geolocation {
    /// <https://www.w3.org/TR/geolocation/#dom-geolocation-getcurrentposition>
    fn GetCurrentPosition(
        &self,
        context: SafeJSContext,
        success_callback: Rc<PositionCallback<DomTypeHolder>>,
        error_callback: HandleValue,
        options: &PositionOptions,
        can_gc: CanGc,
    ) -> Fallible<()> {
        let error_callback = cast_error_callback(context, error_callback)?;
        // Step 1. If this's relevant global object's associated Document is not fully active:
        if !self.global().as_window().Document().is_active() {
            // Step 1.1 Call back with error errorCallback and POSITION_UNAVAILABLE.
            if let Some(error_callback) = error_callback {
                error_callback.Call_(
                    self,
                    &GeolocationPositionError::position_unavailable(
                        &self.global(),
                        DOMString::from("Document is not fully active".to_string()),
                        can_gc,
                    ),
                    ExceptionHandling::Report,
                    can_gc,
                )?;
            }
            // Step 1.2 Terminate this algorithm.
            return Ok(());
        }
        // Step 2. Request a position passing this, successCallback, errorCallback, and options.
        self.request_position(success_callback, error_callback, options, None, can_gc)
    }

    /// <https://www.w3.org/TR/geolocation/#watchposition-method>
    fn WatchPosition(
        &self,
        context: SafeJSContext,
        success_callback: Rc<PositionCallback<DomTypeHolder>>,
        error_callback: HandleValue,
        options: &PositionOptions,
        can_gc: CanGc,
    ) -> Fallible<i32> {
        let error_callback = cast_error_callback(context, error_callback)?;
        // Step 1. If this's relevant global object's associated Document is not fully active:
        if !self.global().as_window().Document().is_active() {
            // Step 1.1 Call back with error errorCallback and POSITION_UNAVAILABLE.
            if let Some(error_callback) = error_callback {
                error_callback.Call_(
                    self,
                    &GeolocationPositionError::position_unavailable(
                        &self.global(),
                        DOMString::from("Document is not fully active".to_string()),
                        can_gc,
                    ),
                    ExceptionHandling::Report,
                    can_gc,
                )?;
            }
            // Step 1.2 Return 0.
            return Ok(0);
        }
        // Step 2. Let watchId be an implementation-defined unsigned long that is greater than zero.
        let watch_id = self.next_watch_id.get();
        self.next_watch_id.set(watch_id + 1);
        // Step 3. Append watchId to this's [[watchIDs]].
        self.watch_ids.borrow_mut().insert(watch_id);
        // Step 4. Request a position passing this, successCallback, errorCallback, options, and watchId.
        self.request_position(
            success_callback,
            error_callback,
            options,
            Some(watch_id),
            can_gc,
        )?;
        // Step 5. Return watchId.
        Ok(watch_id as i32)
    }

    /// <https://www.w3.org/TR/geolocation/#clearwatch-method>
    fn ClearWatch(&self, watch_id: i32) {
        let watch_id = u32::try_from(watch_id).ok();
        if let Some(id) = watch_id {
            self.watch_ids.borrow_mut().remove(&id);
        }
    }
}
