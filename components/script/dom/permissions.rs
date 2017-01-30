/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::PermissionStatusBinding::{PermissionDescriptor, PermissionName, PermissionState};
use dom::bindings::codegen::Bindings::PermissionsBinding::{self, PermissionsMethods};
use dom::bindings::error::Error;
use dom::bindings::js::Root;
use dom::bindings::reflector::{DomObject, Reflector, reflect_dom_object};
use dom::globalscope::GlobalScope;
use dom::permissionstatus::PermissionStatus;
use dom::promise::Promise;
use js::conversions::ConversionResult;
use js::jsapi::{JSContext, JSObject};
use js::jsval::{ObjectValue, UndefinedValue};
use std::rc::Rc;
#[cfg(target_os = "linux")]
use tinyfiledialogs::{self, MessageBoxIcon, YesNo};

#[cfg(target_os = "linux")]
const DIALOG_TITLE: &'static str = "Permission request dialog";
#[cfg(target_os = "linux")]
const QUERY_DIALOG_MESSAGE: &'static str = "Can't guarantee, that the current context is secure.
\t\tStill grant permission for";
const ROOT_DESC_CONVERSION_ERROR: &'static str = "Can't convert to an IDL value of type PermissionDescriptor";

pub trait PermissionAlgorithm {
    type Descriptor;
    type Status;
    fn create_descriptor(cx: *mut JSContext,
                         permission_descriptor_obj: *mut JSObject)
                         -> Result<Self::Descriptor, Error>;
    fn permission_query(cx: *mut JSContext, promise: &Rc<Promise>,
                        descriptor: &Self::Descriptor, status: &Self::Status);
    fn permission_request(cx: *mut JSContext, promise: &Rc<Promise>,
                          descriptor: &Self::Descriptor, status: &Self::Status);
    fn permission_revoke(descriptor: &Self::Descriptor, status: &Self::Status);
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

    pub fn new(global: &GlobalScope) -> Root<Permissions> {
        reflect_dom_object(box Permissions::new_inherited(),
                           global,
                           PermissionsBinding::Wrap)
    }
}

impl PermissionsMethods for Permissions {
    #[allow(unrooted_must_root)]
    #[allow(unsafe_code)]
    // https://w3c.github.io/permissions/#dom-permissions-query
    unsafe fn Query(&self, cx: *mut JSContext, permissionDesc: *mut JSObject) -> Rc<Promise> {
        // Step 3.
        let p = Promise::new(&self.global());

        // Step 1.
        let root_desc = match Permissions::create_descriptor(cx, permissionDesc) {
            Ok(descriptor) => descriptor,
            Err(error) => {
                p.reject_error(cx, error);
                return p;
            },
        };

        // Step 5.
        let status = PermissionStatus::new(&self.global(), &root_desc);

        // Step 2.
        match root_desc.name {
            _ => {
                // Step 6.
                Permissions::permission_query(cx, &p, &root_desc, &status);

                // Step 7.
                p.resolve_native(cx, &status);
            },
        };

        // Step 4.
        return p;
    }

    #[allow(unrooted_must_root)]
    #[allow(unsafe_code)]
    // https://w3c.github.io/permissions/#dom-permissions-request
    unsafe fn Request(&self, cx: *mut JSContext, permissionDesc: *mut JSObject) -> Rc<Promise> {
        // Step 3.
        let p = Promise::new(&self.global());

        // Step 1.
        let root_desc = match Permissions::create_descriptor(cx, permissionDesc) {
            Ok(descriptor) => descriptor,
            Err(error) => {
                p.reject_error(cx, error);
                return p;
            },
        };

        // Step 5.
        let status = PermissionStatus::new(&self.global(), &root_desc);

        // Step 2.
        match root_desc.name {
            _ => {
                // Step 6.
                Permissions::permission_request(cx, &p, &root_desc, &status);

                // Step 7. The default algorithm always resolve

                // Step 8.
                p.resolve_native(cx, &status);
            },
        };
        // Step 4.
        return p;
    }

    #[allow(unrooted_must_root)]
    #[allow(unsafe_code)]
    // https://w3c.github.io/permissions/#dom-permissions-revoke
    unsafe fn Revoke(&self, cx: *mut JSContext, permissionDesc: *mut JSObject) -> Rc<Promise> {
        // Step 1.
        let root_desc = match Permissions::create_descriptor(cx, permissionDesc) {
            Ok(descriptor) => descriptor,
            Err(error) => {
                let p = Promise::new(&self.global());
                p.reject_error(cx, error);
                return p;
            },
        };

        let status = PermissionStatus::new(&self.global(), &root_desc);

        // Step 2.
        match root_desc.name {
            _ => {
                Permissions::permission_revoke(&root_desc, &status);
            },
        };

        // Step 5.
        return self.Query(cx, permissionDesc);
    }
}

impl PermissionAlgorithm for Permissions {
    type Descriptor = PermissionDescriptor;
    type Status = PermissionStatus;

    #[allow(unsafe_code)]
    fn create_descriptor(cx: *mut JSContext,
                         permission_descriptor_obj: *mut JSObject)
                         -> Result<PermissionDescriptor, Error> {
        rooted!(in(cx) let mut property = UndefinedValue());
        property.handle_mut().set(ObjectValue(permission_descriptor_obj));
        unsafe {
            match PermissionDescriptor::new(cx, property.handle()) {
                Ok(ConversionResult::Success(descriptor)) => Ok(descriptor),
                Ok(ConversionResult::Failure(error)) => Err(Error::Type(error.into_owned())),
                Err(_) => Err(Error::Type(String::from(ROOT_DESC_CONVERSION_ERROR))),
            }
        }
    }

    // https://w3c.github.io/permissions/#boolean-permission-query-algorithm
    fn permission_query(_cx: *mut JSContext,
                        _promise: &Rc<Promise>,
                        _descriptor: &PermissionDescriptor,
                        status: &PermissionStatus) {
        // Step 1.
        status.set_state(get_descriptor_permission_state(status.get_query(), None));
    }

    // https://w3c.github.io/permissions/#boolean-permission-request-algorithm
    fn permission_request(cx: *mut JSContext,
                          promise: &Rc<Promise>,
                          descriptor: &PermissionDescriptor,
                          status: &PermissionStatus) {
        // Step 1.
        Permissions::permission_query(cx, promise, descriptor, status);

        // TODO: Step 2 - 4: `environment settings object` is not implemented in Servo yet.
        // For this reason in the `get_descriptor_permission_state` function we can't decide
        // if we have a secure context or not, or store the previous invocation results.
        // Without these the remaining steps can't be implemented properly.
    }

    fn permission_revoke(_descriptor: &PermissionDescriptor, _status: &PermissionStatus) {}
}

// https://w3c.github.io/permissions/#permission-state
pub fn get_descriptor_permission_state(permission_name: PermissionName ,
                                       _env_settings_obj: Option<*mut JSObject>)
                                       -> PermissionState {
    // TODO: Step 1: If settings wasn’t passed, set it to the current settings object.
    // TODO: `environment settings object` is not implemented in Servo yet.

    // Step 2.
    // TODO: The `is the environment settings object a non-secure context` check is missing.
    // The current solution is a workaround with a message box to warn about this,
    // if the feature is not allowed in non-secure contexcts,
    // and let the user decide to grant the permission or not.
    if !allowed_in_nonsecure_contexts(&permission_name) {
        if cfg!(target_os = "linux") {
            match tinyfiledialogs::message_box_yes_no(DIALOG_TITLE,
                                                      &format!("{} {:?} ?", QUERY_DIALOG_MESSAGE, permission_name),
                                                      MessageBoxIcon::Question,
                                                      YesNo::No) {
                YesNo::Yes => return PermissionState::Granted,
                YesNo::No => return PermissionState::Denied,
            };
        } else {
            return PermissionState::Denied;
        }
    }

    // TODO: Step 3: Store the invocation results
    // TODO: `environment settings object` is not implemented in Servo yet.

    // Step 4.
    PermissionState::Granted
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
