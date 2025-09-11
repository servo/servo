/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::default::Default;

use dom_struct::dom_struct;
use servo_url::{ImmutableOrigin, ServoUrl};
use uuid::Uuid;

use crate::dom::bindings::codegen::Bindings::ClientBinding::{ClientMethods, FrameType};
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object};
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::serviceworker::ServiceWorker;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct Client {
    reflector_: Reflector,
    active_worker: MutNullableDom<ServiceWorker>,
    #[no_trace]
    url: ServoUrl,
    frame_type: FrameType,
    #[ignore_malloc_size_of = "Defined in uuid"]
    #[no_trace]
    id: Uuid,
}

impl Client {
    fn new_inherited(url: ServoUrl) -> Client {
        Client {
            reflector_: Reflector::new(),
            active_worker: Default::default(),
            url,
            frame_type: FrameType::None,
            id: Uuid::new_v4(),
        }
    }

    pub(crate) fn new(window: &Window, can_gc: CanGc) -> DomRoot<Client> {
        reflect_dom_object(
            Box::new(Client::new_inherited(window.get_url())),
            window,
            can_gc,
        )
    }

    pub(crate) fn creation_url(&self) -> ServoUrl {
        self.url.clone()
    }

    pub(crate) fn get_controller(&self) -> Option<DomRoot<ServiceWorker>> {
        self.active_worker.get()
    }

    #[expect(dead_code)]
    pub(crate) fn set_controller(&self, worker: &ServiceWorker) {
        self.active_worker.set(Some(worker));
    }

    /// <https://storage.spec.whatwg.org/#obtain-a-storage-key-for-non-storage-purposes>
    #[expect(dead_code)]
    pub(crate) fn obtain_storage_key_for_non_storage_purposes(&self) -> (ImmutableOrigin,) {
        // Step 1. Let origin be environment’s origin if environment is an environment settings object;
        // otherwise environment’s creation URL’s origin.
        let origin = self.url.origin();
        // Return a tuple consisting of origin.
        (origin,)
    }

    /// <https://storage.spec.whatwg.org/#obtain-a-storage-key>
    #[expect(dead_code)]
    pub(crate) fn obtain_storage_key(&self) -> Result<(ImmutableOrigin,), ()> {
        // Step 1. Let key be the result of running obtain a storage key for non-storage purposes with environment.
        let key = self.obtain_storage_key_for_non_storage_purposes();
        // Step 2. If key’s origin is an opaque origin, then return failure.
        if matches!(key.0, ImmutableOrigin::Opaque(..)) {
            return Err(());
        }
        // TODO: Step 3. If the user has disabled storage, then return failure.
        // Step 4. Return key.
        Ok(key)
    }
}

impl ClientMethods<crate::DomTypeHolder> for Client {
    /// <https://w3c.github.io/ServiceWorker/#client-url-attribute>
    fn Url(&self) -> USVString {
        USVString(self.url.as_str().to_owned())
    }

    /// <https://w3c.github.io/ServiceWorker/#client-frametype>
    fn FrameType(&self) -> FrameType {
        self.frame_type
    }

    /// <https://w3c.github.io/ServiceWorker/#client-id>
    fn Id(&self) -> DOMString {
        let uid_str = format!("{}", self.id);
        DOMString::from_string(uid_str)
    }
}
