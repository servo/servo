/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use dom_struct::dom_struct;
use embedder_traits::{self, AllowOrDeny, EmbedderMsg, PermissionFeature};
use ipc_channel::ipc;
use js::conversions::ConversionResult;
use js::jsapi::JSObject;
use js::jsval::{ObjectValue, UndefinedValue};
use script_bindings::inheritance::Castable;
use servo_config::pref;

use super::window::Window;
use crate::conversions::Convert;
use crate::dom::bindings::codegen::Bindings::PermissionStatusBinding::{
    PermissionDescriptor, PermissionName, PermissionState, PermissionStatusMethods,
};
use crate::dom::bindings::codegen::Bindings::PermissionsBinding::PermissionsMethods;
use crate::dom::bindings::codegen::Bindings::WindowBinding::Window_Binding::WindowMethods;
use crate::dom::bindings::error::Error;
use crate::dom::bindings::reflector::{reflect_dom_object, DomGlobal, Reflector};
use crate::dom::bindings::root::DomRoot;
#[cfg(feature = "bluetooth")]
use crate::dom::bluetooth::Bluetooth;
#[cfg(feature = "bluetooth")]
use crate::dom::bluetoothpermissionresult::BluetoothPermissionResult;
use crate::dom::globalscope::GlobalScope;
use crate::dom::permissionstatus::PermissionStatus;
use crate::dom::promise::Promise;
use crate::realms::{AlreadyInRealm, InRealm};
use crate::script_runtime::{CanGc, JSContext};

pub(crate) trait PermissionAlgorithm {
    type Descriptor;
    #[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
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
    fn permission_revoke(descriptor: &Self::Descriptor, status: &Self::Status, can_gc: CanGc);
}

enum Operation {
    Query,
    Request,
    Revoke,
}

// https://w3c.github.io/permissions/#permissions
#[dom_struct]
pub(crate) struct Permissions {
    reflector_: Reflector,
}

impl Permissions {
    pub(crate) fn new_inherited() -> Permissions {
        Permissions {
            reflector_: Reflector::new(),
        }
    }

    pub(crate) fn new(global: &GlobalScope, can_gc: CanGc) -> DomRoot<Permissions> {
        reflect_dom_object(Box::new(Permissions::new_inherited()), global, can_gc)
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
        can_gc: CanGc,
    ) -> Rc<Promise> {
        // (Query, Request) Step 3.
        let p = match promise {
            Some(promise) => promise,
            None => {
                let in_realm_proof = AlreadyInRealm::assert();
                Promise::new_in_current_realm(InRealm::Already(&in_realm_proof), can_gc)
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
        let status = PermissionStatus::new(&self.global(), &root_desc, can_gc);

        // (Query, Request, Revoke) Step 2.
        match root_desc.name {
            #[cfg(feature = "bluetooth")]
            PermissionName::Bluetooth => {
                let bluetooth_desc = match Bluetooth::create_descriptor(cx, permissionDesc) {
                    Ok(descriptor) => descriptor,
                    Err(error) => {
                        p.reject_error(error);
                        return p;
                    },
                };

                // (Query, Request) Step 5.
                let result = BluetoothPermissionResult::new(&self.global(), &status, can_gc);

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
                            .remove(&root_desc.name);

                        // (Revoke) Step 4.
                        Bluetooth::permission_revoke(&bluetooth_desc, &result, can_gc)
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
                            .remove(&root_desc.name);

                        // (Revoke) Step 4.
                        Permissions::permission_revoke(&root_desc, &status, can_gc);
                    },
                }
            },
        };
        match op {
            // (Revoke) Step 5.
            Operation::Revoke => {
                self.manipulate(Operation::Query, cx, permissionDesc, Some(p), can_gc)
            },

            // (Query, Request) Step 4.
            _ => p,
        }
    }
}

#[allow(non_snake_case)]
impl PermissionsMethods<crate::DomTypeHolder> for Permissions {
    // https://w3c.github.io/permissions/#dom-permissions-query
    fn Query(&self, cx: JSContext, permissionDesc: *mut JSObject, can_gc: CanGc) -> Rc<Promise> {
        self.manipulate(Operation::Query, cx, permissionDesc, None, can_gc)
    }

    // https://w3c.github.io/permissions/#dom-permissions-request
    fn Request(&self, cx: JSContext, permissionDesc: *mut JSObject, can_gc: CanGc) -> Rc<Promise> {
        self.manipulate(Operation::Request, cx, permissionDesc, None, can_gc)
    }

    // https://w3c.github.io/permissions/#dom-permissions-revoke
    fn Revoke(&self, cx: JSContext, permissionDesc: *mut JSObject, can_gc: CanGc) -> Rc<Promise> {
        self.manipulate(Operation::Revoke, cx, permissionDesc, None, can_gc)
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

    /// <https://w3c.github.io/permissions/#dfn-permission-query-algorithm>
    ///
    /// > permission query algorithm:
    /// > Takes an instance of the permission descriptor type and a new or existing instance of
    /// > the permission result type, and updates the permission result type instance with the
    /// > query result. Used by Permissions' query(permissionDesc) method and the
    /// > PermissionStatus update steps. If unspecified, this defaults to the default permission
    /// > query algorithm.
    ///
    /// > The default permission query algorithm, given a PermissionDescriptor
    /// > permissionDesc and a PermissionStatus status, runs the following steps:
    fn permission_query(
        _cx: JSContext,
        _promise: &Rc<Promise>,
        _descriptor: &PermissionDescriptor,
        status: &PermissionStatus,
    ) {
        // Step 1. Set status's state to permissionDesc's permission state.
        status.set_state(descriptor_permission_state(status.get_query(), None));
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
                // https://w3c.github.io/permissions/#request-permission-to-use (Step 3 - 4)
                let permission_name = status.get_query();
                let globalscope = GlobalScope::current().expect("No current global object");
                let state = prompt_user_from_embedder(permission_name, &globalscope);
                globalscope
                    .permission_state_invocation_results()
                    .borrow_mut()
                    .insert(permission_name, state);
            },

            // Step 2.
            _ => return,
        }

        // Step 4.
        Permissions::permission_query(cx, promise, descriptor, status);
    }

    fn permission_revoke(
        _descriptor: &PermissionDescriptor,
        _status: &PermissionStatus,
        _can_gc: CanGc,
    ) {
    }
}

/// <https://w3c.github.io/permissions/#dfn-permission-state>
pub(crate) fn descriptor_permission_state(
    feature: PermissionName,
    env_settings_obj: Option<&GlobalScope>,
) -> PermissionState {
    // Step 1. If settings wasn't passed, set it to the current settings object.
    let global_scope = match env_settings_obj {
        Some(env_settings_obj) => DomRoot::from_ref(env_settings_obj),
        None => GlobalScope::current().expect("No current global object"),
    };

    // Step 2. If settings is a non-secure context, return "denied".
    if !global_scope.is_secure_context() {
        if pref!(dom_permissions_testing_allowed_in_nonsecure_contexts) {
            return PermissionState::Granted;
        }
        return PermissionState::Denied;
    }

    // Step 3. Let feature be descriptor's name.
    // The caller has already converted the descriptor into a name.

    // Step 4. If there exists a policy-controlled feature for feature and settings'
    // relevant global object has an associated Document run the following step:
    //   1. Let document be settings' relevant global object's associated Document.
    //   2. If document is not allowed to use feature, return "denied".
    if let Some(window) = global_scope.downcast::<Window>() {
        if !window.Document().allowed_to_use_feature(feature) {
            return PermissionState::Denied;
        }
    }

    // Step 5. Let key be the result of generating a permission key for descriptor with settings.
    // Step 6. Let entry be the result of getting a permission store entry with descriptor and key.
    // Step 7. If entry is not null, return a PermissionState enum value from entry's state.
    //
    // TODO: We aren't making a key based on the descriptor, but on the descriptor's name. This really
    // only matters for WebBluetooth, which adds more fields to the descriptor beyond the name.
    if let Some(entry) = global_scope
        .permission_state_invocation_results()
        .borrow()
        .get(&feature)
    {
        return *entry;
    }

    // Step 8. Return the PermissionState enum value that represents the permission state
    // of feature, taking into account any permission state constraints for descriptor's
    // name.
    PermissionState::Prompt
}

fn prompt_user_from_embedder(name: PermissionName, global_scope: &GlobalScope) -> PermissionState {
    let Some(webview_id) = global_scope.webview_id() else {
        warn!("Requesting permissions from non-webview-associated global scope");
        return PermissionState::Denied;
    };
    let (sender, receiver) = ipc::channel().expect("Failed to create IPC channel!");
    global_scope.send_to_embedder(EmbedderMsg::PromptPermission(
        webview_id,
        name.convert(),
        sender,
    ));

    match receiver.recv() {
        Ok(AllowOrDeny::Allow) => PermissionState::Granted,
        Ok(AllowOrDeny::Deny) => PermissionState::Denied,
        Err(e) => {
            warn!(
                "Failed to receive permission state from embedder ({:?}).",
                e
            );
            PermissionState::Denied
        },
    }
}

impl Convert<PermissionFeature> for PermissionName {
    fn convert(self) -> PermissionFeature {
        match self {
            PermissionName::Geolocation => PermissionFeature::Geolocation,
            PermissionName::Notifications => PermissionFeature::Notifications,
            PermissionName::Push => PermissionFeature::Push,
            PermissionName::Midi => PermissionFeature::Midi,
            PermissionName::Camera => PermissionFeature::Camera,
            PermissionName::Microphone => PermissionFeature::Microphone,
            PermissionName::Speaker => PermissionFeature::Speaker,
            PermissionName::Device_info => PermissionFeature::DeviceInfo,
            PermissionName::Background_sync => PermissionFeature::BackgroundSync,
            PermissionName::Bluetooth => PermissionFeature::Bluetooth,
            PermissionName::Persistent_storage => PermissionFeature::PersistentStorage,
        }
    }
}
