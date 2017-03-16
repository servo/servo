/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::PermissionStatusBinding::{PermissionDescriptor, PermissionName, PermissionState};
use dom::bindings::codegen::Bindings::PermissionStatusBinding::PermissionStatusMethods;
use dom::bindings::codegen::Bindings::PermissionsBinding::{self, PermissionsMethods};
use dom::bindings::error::Error;
use dom::bindings::js::Root;
use dom::bindings::reflector::{DomObject, Reflector, reflect_dom_object};
use dom::bluetooth::Bluetooth;
use dom::bluetoothpermissionresult::BluetoothPermissionResult;
use dom::globalscope::GlobalScope;
use dom::permissionstatus::PermissionStatus;
use dom::promise::Promise;
use dom_struct::dom_struct;
use js::conversions::ConversionResult;
use js::jsapi::{JSContext, JSObject};
use js::jsval::{ObjectValue, UndefinedValue};
use servo_config::opts;
use servo_config::prefs::PREFS;
use std::rc::Rc;
#[cfg(target_os = "linux")]
use tinyfiledialogs::{self, MessageBoxIcon, YesNo};

#[cfg(target_os = "linux")]
const DIALOG_TITLE: &'static str = "Permission request dialog";
const NONSECURE_DIALOG_MESSAGE: &'static str = "feature is only safe to use in secure context,\
 but servo can't guarantee\n that the current context is secure. Do you want to proceed and grant permission?";
const REQUEST_DIALOG_MESSAGE: &'static str = "Do you want to grant permission for";
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

    pub fn new(global: &GlobalScope) -> Root<Permissions> {
        reflect_dom_object(box Permissions::new_inherited(),
                           global,
                           PermissionsBinding::Wrap)
    }

    #[allow(unrooted_must_root)]
    // https://w3c.github.io/permissions/#dom-permissions-query
    // https://w3c.github.io/permissions/#dom-permissions-request
    // https://w3c.github.io/permissions/#dom-permissions-revoke
    fn manipulate(&self,
                  op: Operation,
                  cx: *mut JSContext,
                  permissionDesc: *mut JSObject,
                  promise: Option<Rc<Promise>>)
                  -> Rc<Promise> {
        // (Query, Request) Step 3.
        let p = match promise {
            Some(promise) => promise,
            None => Promise::new(&self.global()),
        };

        // (Query, Request, Revoke) Step 1.
        let root_desc = match Permissions::create_descriptor(cx, permissionDesc) {
            Ok(descriptor) => descriptor,
            Err(error) => {
                p.reject_error(cx, error);
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
                        p.reject_error(cx, error);
                        return p;
                    },
                };

                // (Query, Request) Step 5.
                let result = BluetoothPermissionResult::new(&self.global(), &status);

                match &op {
                    // (Request) Step 6 - 8.
                    &Operation::Request => Bluetooth::permission_request(cx, &p, &bluetooth_desc, &result),

                    // (Query) Step 6 - 7.
                    &Operation::Query => Bluetooth::permission_query(cx, &p, &bluetooth_desc, &result),

                    &Operation::Revoke => {
                        // (Revoke) Step 3.
                        let globalscope = self.global();
                        globalscope.as_window()
                                   .permission_state_invocation_results()
                                   .borrow_mut()
                                   .remove(&root_desc.name.to_string());

                        // (Revoke) Step 4.
                        Bluetooth::permission_revoke(&bluetooth_desc, &result)
                    },
                }
            },
            _ => {
                match &op {
                    &Operation::Request => {
                        // (Request) Step 6.
                        Permissions::permission_request(cx, &p, &root_desc, &status);

                        // (Request) Step 7. The default algorithm always resolve

                        // (Request) Step 8.
                        p.resolve_native(cx, &status);
                    },
                    &Operation::Query => {
                        // (Query) Step 6.
                        Permissions::permission_query(cx, &p, &root_desc, &status);

                        // (Query) Step 7.
                        p.resolve_native(cx, &status);
                    },

                    &Operation::Revoke => {
                        // (Revoke) Step 3.
                        let globalscope = self.global();
                        globalscope.as_window()
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

impl PermissionsMethods for Permissions {
    #[allow(unrooted_must_root)]
    #[allow(unsafe_code)]
    // https://w3c.github.io/permissions/#dom-permissions-query
    unsafe fn Query(&self, cx: *mut JSContext, permissionDesc: *mut JSObject) -> Rc<Promise> {
        self.manipulate(Operation::Query, cx, permissionDesc, None)
    }

    #[allow(unrooted_must_root)]
    #[allow(unsafe_code)]
    // https://w3c.github.io/permissions/#dom-permissions-request
    unsafe fn Request(&self, cx: *mut JSContext, permissionDesc: *mut JSObject) -> Rc<Promise> {
        self.manipulate(Operation::Request, cx, permissionDesc, None)
    }

    #[allow(unrooted_must_root)]
    #[allow(unsafe_code)]
    // https://w3c.github.io/permissions/#dom-permissions-revoke
    unsafe fn Revoke(&self, cx: *mut JSContext, permissionDesc: *mut JSObject) -> Rc<Promise> {
        self.manipulate(Operation::Revoke, cx, permissionDesc, None)
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

        match status.State() {
            // Step 3.
            PermissionState::Prompt => {
                let perm_name = status.get_query();
                // https://w3c.github.io/permissions/#request-permission-to-use (Step 3 - 4)
                let state =
                    prompt_user(&format!("{} {} ?", REQUEST_DIALOG_MESSAGE, perm_name.clone()));

                let globalscope = GlobalScope::current();
                globalscope.as_window()
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
pub fn get_descriptor_permission_state(permission_name: PermissionName,
                                       env_settings_obj: Option<&GlobalScope>)
                                       -> PermissionState {
    // Step 1.
    let settings = match env_settings_obj {
        Some(env_settings_obj) => Root::from_ref(env_settings_obj),
        None => GlobalScope::current(),
    };

    // Step 2.
    // TODO: The `is the environment settings object a non-secure context` check is missing.
    // The current solution is a workaround with a message box to warn about this,
    // if the feature is not allowed in non-secure contexcts,
    // and let the user decide to grant the permission or not.
    let state = match allowed_in_nonsecure_contexts(&permission_name) {
        true => PermissionState::Prompt,
        false => {
            match PREFS.get("dom.permissions.testing.allowed_in_nonsecure_contexts").as_boolean().unwrap_or(false) {
                true => PermissionState::Granted,
                false => {
                    settings.as_window()
                            .permission_state_invocation_results()
                            .borrow_mut()
                            .remove(&permission_name.to_string());
                    prompt_user(&format!("The {} {}", permission_name, NONSECURE_DIALOG_MESSAGE))
                },
            }
        },
    };

    // Step 3.
    if let Some(prev_result) = settings.as_window()
                                       .permission_state_invocation_results()
                                       .borrow()
                                       .get(&permission_name.to_string()) {
        return prev_result.clone();
    }

    // Store the invocation result
    settings.as_window()
            .permission_state_invocation_results()
            .borrow_mut()
            .insert(permission_name.to_string(), state);

    // Step 4.
    state
}

#[cfg(target_os = "linux")]
fn prompt_user(message: &str) -> PermissionState {
    if opts::get().headless {
        return PermissionState::Denied;
    }
    match tinyfiledialogs::message_box_yes_no(DIALOG_TITLE,
                                              message,
                                              MessageBoxIcon::Question,
                                              YesNo::No) {
        YesNo::Yes => PermissionState::Granted,
        YesNo::No => PermissionState::Denied,
    }
}

#[cfg(not(target_os = "linux"))]
fn prompt_user(_message: &str) -> PermissionState {
    // TODO popup only supported on linux
    PermissionState::Denied
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
