/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::UNIX_EPOCH;

use dom_struct::dom_struct;
use geolocation_traits::GeolocationRequest;
use ipc_channel::ipc::IpcSender;
use ipc_channel::router::ROUTER;
use rustc_hash::FxHashSet;
use script_bindings::codegen::GenericBindings::GeolocationBinding::Geolocation_Binding::GeolocationMethods;
use script_bindings::codegen::GenericBindings::GeolocationBinding::{
    PositionCallback, PositionOptions,
};
use script_bindings::codegen::GenericBindings::PermissionStatusBinding::{
    PermissionDescriptor, PermissionName,
};
use script_bindings::codegen::GenericBindings::WindowBinding::WindowMethods;
use script_bindings::num::Finite;
use script_bindings::reflector::Reflector;
use script_bindings::root::DomRoot;
use script_bindings::script_runtime::CanGc;

use crate::dom::bindings::codegen::DomTypeHolder::DomTypeHolder;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::{DomGlobal, reflect_dom_object};
use crate::dom::geolocationcoordinates::GeolocationCoordinates;
use crate::dom::geolocationposition::GeolocationPosition;
use crate::dom::globalscope::GlobalScope;

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

    fn geolocation_sender(&self) -> IpcSender<GeolocationRequest> {
        self.global().as_window().geolocation_thread()
    }

    /// https://www.w3.org/TR/geolocation/#request-a-position
    fn request_a_position(
        &self,
        success_callback: Rc<PositionCallback<DomTypeHolder>>,
        options: &PositionOptions,
        watch_id: Option<u32>,
    ) {
        // Step 1: Let watchIDs be geolocation's [[watchIDs]].

        // Step 2: Let document be the geolocation's relevant global object's associated Document.
        let document = self.global().as_window().Document();
        // Step 3: If document is not allowed to use the "geolocation" feature:
        if !document.allowed_to_use_feature(PermissionName::Geolocation) {
            // Step 3.1: If watchId was passed, remove watchId from watchIDs.
            if let Some(watch_id) = watch_id {
                self.watch_ids.borrow_mut().remove(&watch_id);
            }
            // Step 3.2: Call back with error passing errorCallback and PERMISSION_DENIED.
            // TODO: Error callback not implemented yet.
            // Step 3.3: Terminate this algorithm.
            return;
        }
        // TODO: Step 4: If geolocation's environment settings object is a non-secure context:
        // Step 4.1: If watchId was passed, remove watchId from watchIDs.
        // Step 4.2: Call back with error passing errorCallback and PERMISSION_DENIED.
        // Step 4.3: Terminate this algorithm.
        // TODO: Step 5: If document's visibility state is "hidden", wait for the following page visibility change steps to run:
        // Step 5.1: Assert: document's visibility state is "visible".
        // Step 5.2: Continue to the next steps below.
        // TODO: Step 6: Let descriptor be a new PermissionDescriptor whose name is "geolocation".
        // Step 7: In parallel:
        let task_source = self.global().task_manager().geolocation_task_source();
        let sendable_task_source = task_source.to_sendable();
        task_source.queue(task!(request_position: move || {
            let options = geolocation_traits::Options {
                accuracy: if options.enableHighAccuracy {
                    geolocation_traits::Accuracy::High
                } else {
                    geolocation_traits::Accuracy::Low
                },
                maximum_age: options.maximumAge,
                timeout: options.timeout
            };
            let this = Trusted::new(self);
            let (sen, rec) = ipc_channel::ipc::channel().unwrap();
            self.geolocation_sender().send(GeolocationRequest::GetPosition(options, sen)).unwrap();
            ROUTER.add_typed_route(rec, Box::new(move |message| {
                let message = message.unwrap();
                sendable_task_source.queue(task!(return_position: move || {
                    let this = this.root();
                    let global = this.global();
                    match message {
                        Ok(position) => {
                            let coordinates = GeolocationCoordinates::new(&global,
                                Finite::new(position.coords.accuracy).unwrap(),
                                Finite::new(position.coords.latitude).unwrap(),
                                Finite::new(position.coords.longitude).unwrap(),
                                position.coords.altitude.map(Finite::new).flatten(),
                                position.coords.altitude_accuracy.map(Finite::new).flatten(),
                                position.coords.heading.map(Finite::new).flatten(),
                                position.coords.speed.map(Finite::new).flatten(),
                                CanGc::note(),
                            );
                            let position = GeolocationPosition::new(&global, &coordinates, position.timestamp.duration_since(UNIX_EPOCH).unwrap().as_secs(), CanGc::note());
                            success_callback.call(&position);
                        }
                        Err(_e) => {
                            // TODO: report error
                        }
                    }
                }));
            }));
        }));
    }
}

impl GeolocationMethods<DomTypeHolder> for Geolocation {
    /// <https://www.w3.org/TR/geolocation/#dom-geolocation-getcurrentposition>
    fn GetCurrentPosition(
        &self,
        success_callback: Rc<PositionCallback<DomTypeHolder>>,
        options: &PositionOptions,
    ) {
        // Step 1. If this's relevant global object's associated Document is not fully active:
        if !self.global().as_window().Document().is_active() {
            // Step 1.1 Call back with error errorCallback and POSITION_UNAVAILABLE.
            // TODO: Error callback not implemented yet.
            // Step 1.2 Terminate this algorithm.
            return;
        }
        // Step 2. Request a position passing this, successCallback, errorCallback, and options.
        self.request_a_position(success_callback, options, None);
    }

    /// <https://www.w3.org/TR/geolocation/#watchposition-method>
    fn WatchPosition(
        &self,
        success_callback: Rc<PositionCallback<DomTypeHolder>>,
        options: &PositionOptions,
    ) -> i32 {
        // Step 1. If this's relevant global object's associated Document is not fully active:
        if !self.global().as_window().Document().is_active() {
            // Step 1.1 Call back with error errorCallback and POSITION_UNAVAILABLE.
            // Step 1.2 Return 0.
            return 0;
        }
        // Step 2. Let watchId be an implementation-defined unsigned long that is greater than zero.
        let watch_id = self.next_watch_id.get();
        self.next_watch_id.set(watch_id + 1);
        // Step 3. Append watchId to this's [[watchIDs]].
        self.watch_ids.borrow_mut().insert(watch_id);
        // Step 4. Request a position passing this, successCallback, errorCallback, options, and watchId.
        self.request_a_position(success_callback, options, Some(watch_id));
        // Step 5. Return watchId.
        watch_id as i32
    }

    /// <https://www.w3.org/TR/geolocation/#clearwatch-method>
    fn ClearWatch(&self, watch_id: i32) {
        let watch_id = u32::try_from(watch_id).ok();
        if let Some(id) = watch_id {
            self.watch_ids.borrow_mut().remove(&id);
        }
    }
}
