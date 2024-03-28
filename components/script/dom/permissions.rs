/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use dom_struct::dom_struct;
use embedder_traits::{self, EmbedderMsg, PermissionPrompt, PermissionRequest};
use ipc_channel::ipc;
use js::conversions::ConversionResult;
use js::jsapi::JSObject;
use js::jsval::{ObjectValue, UndefinedValue};
use servo_config::pref;

use crate::dom::bindings::codegen::Bindings::PermissionStatusBinding::{
    PermissionDescriptor, PermissionName, PermissionState, PermissionStatusMethods,
};
use crate::dom::bindings::codegen::Bindings::PermissionsBinding::PermissionsMethods;
use crate::dom::bindings::error::Error;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bluetooth::Bluetooth;
use crate::dom::bluetoothpermissionresult::BluetoothPermissionResult;
use crate::dom::globalscope::GlobalScope;
use crate::dom::permissionstatus::PermissionStatus;
use crate::dom::promise::Promise;
use crate::realms::{AlreadyInRealm, InRealm};
use crate::script_runtime::JSContext;

pub trait PermissionAlgorithm {
    type Descriptor;
    type Status;
    fn create_descriptor(
        cx: JSContext,
        permission_descriptor_obj: *mut JSObject,
    ) -> Result<Self::Descriptor, Error>;
    fn permission_query(
        cx: JSContext,
        promise: &Rc<Promise>,
        descriptor: &Self::Descriptor,
        status: &Self::Status,
    );
    fn permission_request(
        cx: JSContext,
        promise: &Rc<Promise>,
        descriptor: &Self::Descriptor,
        status: &Self::Status,
    );
    fn permission_revoke(descriptor: &Self::Descriptor, status: &Self::Status);
}

enum Operation {
    Query,
    Request,
    Revoke,
}

// https://w3c.github.io/permissions/#permissions
#[dom_struct]
pub struct Permissions {
    reflector_: Reflector,
}

impl Permissions {
    pub fn new_inherited() -> Permissions {
        Permissions {
            reflector_: Reflector::new(),
        }
    }

    pub fn new(global: &GlobalScope) -> DomRoot<Permissions> {
        reflect_dom_object(Box::new(Permissions::new_inherited()), global)
    }

    // https://w3c.github.io/permissions/#dom-permissions-query
    // https://w3c.github.io/permissions/#dom-permissions-request
    // https://w3c.github.io/permissions/#dom-permissions-revoke
    #[allow(non_snake_case)]
    fn manipulate(
        &self,
        op: Operation,
        cx: JSContext,
        permissionDesc: *mut JSObject,
        promise: Option<Rc<Promise>>,
    ) -> Rc<Promise> {
        // (Query, Request) Step 3.
        let p = match promise {
            Some(promise) => promise,
            None => {
                let in_realm_proof = AlreadyInRealm::assert();
                Promise::new_in_current_realm(InRealm::Already(&in_realm_proof))
            },
        };

        // (Query, Request, Revoke) Step 1.
        let root_desc = match Permissions::create_descriptor(cx, permissionDesc) {
            Ok(descriptor) => descriptor,
            Err(error) => {
                p.reject_error(error);
                return p;
            },
        };

        // (Query, Request) Step 5.
        let status = PermissionStatus::new(&self.global(), &root_desc);

        // (Query, Request, Revoke) Step 2.
        match root_desc.name {
            PermissionName::Bluetooth => {
                let bluetooth_desc = match Bluetooth::create_descriptor(cx, permissionDesc) {
                    Ok(descriptor) => descriptor,
                    Err(error) => {
                        p.reject_error(error);
                        return p;
                    },
                };

                // (Query, Request) Step 5.
                let result = BluetoothPermissionResult::new(&self.global(), &status);

                match op {
                    // (Request) Step 6 - 8.
                    Operation::Request => {
                        Bluetooth::permission_request(cx, &p, &bluetooth_desc, &result)
                    },

                    // (Query) Step 6 - 7.
                    Operation::Query => {
                        Bluetooth::permission_query(cx, &p, &bluetooth_desc, &result)
                    },

                    Operation::Revoke => {
                        // (Revoke) Step 3.
                        let globalscope = self.global();
                        globalscope
                            .permission_state_invocation_results()
                            .borrow_mut()
                            .remove(&root_desc.name.to_string());

                        // (Revoke) Step 4.
                        Bluetooth::permission_revoke(&bluetooth_desc, &result)
                    },
                }
            },
            _ => {
                match op {
                    Operation::Request => {
                        // (Request) Step 6.
                        Permissions::permission_request(cx, &p, &root_desc, &status);

                        // (Request) Step 7. The default algorithm always resolve

                        // (Request) Step 8.
                        p.resolve_native(&status);
                    },
                    Operation::Query => {
                        // (Query) Step 6.
                        Permissions::permission_query(cx, &p, &root_desc, &status);

                        // (Query) Step 7.
                        p.resolve_native(&status);
                    },

                    Operation::Revoke => {
                        // (Revoke) Step 3.
                        let globalscope = self.global();
                        globalscope
                            .permission_state_invocation_results()
                            .borrow_mut()
                            .remove(&root_desc.name.to_string());

                        // (Revoke) Step 4.
                        Permissions::permission_revoke(&root_desc, &status);
                    },
                }
            },
        };
        match op {
            // (Revoke) Step 5.
            Operation::Revoke => self.manipulate(Operation::Query, cx, permissionDesc, Some(p)),

            // (Query, Request) Step 4.
            _ => p,
        }
    }
}

#[allow(non_snake_case)]
impl PermissionsMethods for Permissions {
    // https://w3c.github.io/permissions/#dom-permissions-query
    fn Query(&self, cx: JSContext, permissionDesc: *mut JSObject) -> Rc<Promise> {
        self.manipulate(Operation::Query, cx, permissionDesc, None)
    }

    // https://w3c.github.io/permissions/#dom-permissions-request
    fn Request(&self, cx: JSContext, permissionDesc: *mut JSObject) -> Rc<Promise> {
        self.manipulate(Operation::Request, cx, permissionDesc, None)
    }

    // https://w3c.github.io/permissions/#dom-permissions-revoke
    fn Revoke(&self, cx: JSContext, permissionDesc: *mut JSObject) -> Rc<Promise> {
        self.manipulate(Operation::Revoke, cx, permissionDesc, None)
    }
}

impl PermissionAlgorithm for Permissions {
    type Descriptor = PermissionDescriptor;
    type Status = PermissionStatus;

    fn create_descriptor(
        cx: JSContext,
        permission_descriptor_obj: *mut JSObject,
    ) -> Result<PermissionDescriptor, Error> {
        rooted!(in(*cx) let mut property = UndefinedValue());
        property
            .handle_mut()
            .set(ObjectValue(permission_descriptor_obj));
        match PermissionDescriptor::new(cx, property.handle()) {
            Ok(ConversionResult::Success(descriptor)) => Ok(descriptor),
            Ok(ConversionResult::Failure(error)) => Err(Error::Type(error.into_owned())),
            Err(_) => Err(Error::JSFailed),
        }
    }

    // https://w3c.github.io/permissions/#boolean-permission-query-algorithm
    fn permission_query(
        _cx: JSContext,
        _promise: &Rc<Promise>,
        _descriptor: &PermissionDescriptor,
        status: &PermissionStatus,
    ) {
        // Step 1.
        status.set_state(get_descriptor_permission_state(status.get_query(), None));
    }

    // https://w3c.github.io/permissions/#boolean-permission-request-algorithm
    fn permission_request(
        cx: JSContext,
        promise: &Rc<Promise>,
        descriptor: &PermissionDescriptor,
        status: &PermissionStatus,
    ) {
        // Step 1.
        Permissions::permission_query(cx, promise, descriptor, status);

        match status.State() {
            // Step 3.
            PermissionState::Prompt => {
                let perm_name = status.get_query();
                let prompt =
                    PermissionPrompt::Request(embedder_traits::PermissionName::from(perm_name));

                // https://w3c.github.io/permissions/#request-permission-to-use (Step 3 - 4)
                let globalscope = GlobalScope::current().expect("No current global object");
                let state = prompt_user_from_embedder(prompt, &globalscope);
                globalscope
                    .permission_state_invocation_results()
                    .borrow_mut()
                    .insert(perm_name.to_string(), state);
            },

            // Step 2.
            _ => return,
        }

        // Step 4.
        Permissions::permission_query(cx, promise, descriptor, status);
    }

    fn permission_revoke(_descriptor: &PermissionDescriptor, _status: &PermissionStatus) {}
}

// https://w3c.github.io/permissions/#permission-state
pub fn get_descriptor_permission_state(
    permission_name: PermissionName,
    env_settings_obj: Option<&GlobalScope>,
) -> PermissionState {
    // Step 1.
    let globalscope = match env_settings_obj {
        Some(env_settings_obj) => DomRoot::from_ref(env_settings_obj),
        None => GlobalScope::current().expect("No current global object"),
    };

    // Step 2.
    // TODO: The `is the environment settings object a non-secure context` check is missing.
    // The current solution is a workaround with a message box to warn about this,
    // if the feature is not allowed in non-secure contexcts,
    // and let the user decide to grant the permission or not.
    let state = if allowed_in_nonsecure_contexts(&permission_name) {
        PermissionState::Prompt
    } else if pref!(dom.permissions.testing.allowed_in_nonsecure_contexts) {
        PermissionState::Granted
    } else {
        globalscope
            .permission_state_invocation_results()
            .borrow_mut()
            .remove(&permission_name.to_string());
        prompt_user_from_embedder(
            PermissionPrompt::Insecure(embedder_traits::PermissionName::from(permission_name)),
            &globalscope,
        )
    };

    // Step 3.
    if let Some(prev_result) = globalscope
        .permission_state_invocation_results()
        .borrow()
        .get(&permission_name.to_string())
    {
        return *prev_result;
    }

    // Store the invocation result
    globalscope
        .permission_state_invocation_results()
        .borrow_mut()
        .insert(permission_name.to_string(), state);

    // Step 4.
    state
}

// https://w3c.github.io/permissions/#allowed-in-non-secure-contexts
fn allowed_in_nonsecure_contexts(permission_name: &PermissionName) -> bool {
    match *permission_name {
        // https://w3c.github.io/permissions/#dom-permissionname-geolocation
        PermissionName::Geolocation => true,
        // https://w3c.github.io/permissions/#dom-permissionname-notifications
        PermissionName::Notifications => true,
        // https://w3c.github.io/permissions/#dom-permissionname-push
        PermissionName::Push => false,
        // https://w3c.github.io/permissions/#dom-permissionname-midi
        PermissionName::Midi => true,
        // https://w3c.github.io/permissions/#dom-permissionname-camera
        PermissionName::Camera => false,
        // https://w3c.github.io/permissions/#dom-permissionname-microphone
        PermissionName::Microphone => false,
        // https://w3c.github.io/permissions/#dom-permissionname-speaker
        PermissionName::Speaker => false,
        // https://w3c.github.io/permissions/#dom-permissionname-device-info
        PermissionName::Device_info => false,
        // https://w3c.github.io/permissions/#dom-permissionname-background-sync
        PermissionName::Background_sync => false,
        // https://webbluetoothcg.github.io/web-bluetooth/#dom-permissionname-bluetooth
        PermissionName::Bluetooth => false,
        // https://storage.spec.whatwg.org/#dom-permissionname-persistent-storage
        PermissionName::Persistent_storage => false,
    }
}

fn prompt_user_from_embedder(prompt: PermissionPrompt, gs: &GlobalScope) -> PermissionState {
    let (sender, receiver) = ipc::channel().expect("Failed to create IPC channel!");
    gs.send_to_embedder(EmbedderMsg::PromptPermission(prompt, sender));

    match receiver.recv() {
        Ok(PermissionRequest::Granted) => PermissionState::Granted,
        Ok(PermissionRequest::Denied) => PermissionState::Denied,
        Err(e) => {
            warn!(
                "Failed to receive permission state from embedder ({:?}).",
                e
            );
            PermissionState::Denied
        },
    }
}

impl From<PermissionName> for embedder_traits::PermissionName {
    fn from(permission_name: PermissionName) -> Self {
        match permission_name {
            PermissionName::Geolocation => embedder_traits::PermissionName::Geolocation,
            PermissionName::Notifications => embedder_traits::PermissionName::Notifications,
            PermissionName::Push => embedder_traits::PermissionName::Push,
            PermissionName::Midi => embedder_traits::PermissionName::Midi,
            PermissionName::Camera => embedder_traits::PermissionName::Camera,
            PermissionName::Microphone => embedder_traits::PermissionName::Microphone,
            PermissionName::Speaker => embedder_traits::PermissionName::Speaker,
            PermissionName::Device_info => embedder_traits::PermissionName::DeviceInfo,
            PermissionName::Background_sync => embedder_traits::PermissionName::BackgroundSync,
            PermissionName::Bluetooth => embedder_traits::PermissionName::Bluetooth,
            PermissionName::Persistent_storage => {
                embedder_traits::PermissionName::PersistentStorage
            },
        }
    }
}
