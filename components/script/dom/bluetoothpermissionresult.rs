/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use bluetooth_traits::{BluetoothRequest, BluetoothResponse};
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::BluetoothPermissionResultBinding::{self, BluetoothPermissionResultMethods};
use dom::bindings::codegen::Bindings::NavigatorBinding::NavigatorBinding::NavigatorMethods;
use dom::bindings::codegen::Bindings::PermissionStatusBinding::{PermissionName, PermissionState};
use dom::bindings::codegen::Bindings::PermissionStatusBinding::PermissionStatusBinding::PermissionStatusMethods;
use dom::bindings::codegen::Bindings::WindowBinding::WindowBinding::WindowMethods;
use dom::bindings::error::Error;
use dom::bindings::js::{JS, Root};
use dom::bindings::reflector::{DomObject, reflect_dom_object};
use dom::bindings::str::DOMString;
use dom::bluetooth::{AsyncBluetoothListener, Bluetooth, AllowedBluetoothDevice};
use dom::bluetoothdevice::BluetoothDevice;
use dom::globalscope::GlobalScope;
use dom::permissionstatus::PermissionStatus;
use dom::promise::Promise;
use dom_struct::dom_struct;
use ipc_channel::ipc::IpcSender;
use js::jsapi::JSContext;
use std::rc::Rc;

// https://webbluetoothcg.github.io/web-bluetooth/#bluetoothpermissionresult
#[dom_struct]
pub struct BluetoothPermissionResult {
    status: PermissionStatus,
    devices: DOMRefCell<Vec<JS<BluetoothDevice>>>,
}

impl BluetoothPermissionResult {
    #[allow(unrooted_must_root)]
    fn new_inherited(status: &PermissionStatus) -> BluetoothPermissionResult {
        let result = BluetoothPermissionResult {
            status: PermissionStatus::new_inherited(status.get_query()),
            devices: DOMRefCell::new(Vec::new()),
        };
        result.status.set_state(status.State());
        result
    }

    pub fn new(global: &GlobalScope, status: &PermissionStatus) -> Root<BluetoothPermissionResult> {
        reflect_dom_object(box BluetoothPermissionResult::new_inherited(status),
                           global,
                           BluetoothPermissionResultBinding::Wrap)
    }

    pub fn get_bluetooth(&self) -> Root<Bluetooth> {
        self.global().as_window().Navigator().Bluetooth()
    }

    pub fn get_bluetooth_thread(&self) -> IpcSender<BluetoothRequest> {
        self.global().as_window().bluetooth_thread()
    }

    pub fn get_query(&self) -> PermissionName {
        self.status.get_query()
    }

    pub fn set_state(&self, state: PermissionState) {
        self.status.set_state(state)
    }

    pub fn get_state(&self) -> PermissionState {
        self.status.State()
    }

    #[allow(unrooted_must_root)]
    pub fn set_devices(&self, devices: Vec<JS<BluetoothDevice>>) {
        *self.devices.borrow_mut() = devices;
    }
}

impl BluetoothPermissionResultMethods for BluetoothPermissionResult {
    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothpermissionresult-devices
    fn Devices(&self) -> Vec<Root<BluetoothDevice>> {
        let device_vec: Vec<Root<BluetoothDevice>> =
            self.devices.borrow().iter().map(|d| Root::from_ref(&**d)).collect();
        device_vec
    }
}

impl AsyncBluetoothListener for BluetoothPermissionResult {
    fn handle_response(&self, response: BluetoothResponse, promise_cx: *mut JSContext, promise: &Rc<Promise>) {
        match response {
            // https://webbluetoothcg.github.io/web-bluetooth/#request-bluetooth-devices
            // Step 3, 11, 13 - 14.
            BluetoothResponse::RequestDevice(device) => {
                self.set_state(PermissionState::Granted);
                let bluetooth = self.get_bluetooth();
                let mut device_instance_map = bluetooth.get_device_map().borrow_mut();
                if let Some(ref existing_device) = device_instance_map.get(&device.id) {
                    // https://webbluetoothcg.github.io/web-bluetooth/#request-the-bluetooth-permission
                    // Step 3.
                    self.set_devices(vec!(JS::from_ref(&*existing_device)));

                    // https://w3c.github.io/permissions/#dom-permissions-request
                    // Step 8.
                    return promise.resolve_native(promise_cx, self);
                }
                let bt_device = BluetoothDevice::new(&self.global(),
                                                     DOMString::from(device.id.clone()),
                                                     device.name.map(DOMString::from),
                                                     &bluetooth);
                device_instance_map.insert(device.id.clone(), JS::from_ref(&bt_device));
                self.global().as_window().bluetooth_extra_permission_data().add_new_allowed_device(
                    AllowedBluetoothDevice {
                        deviceId: DOMString::from(device.id),
                        mayUseGATT: true,
                    }
                );
                // https://webbluetoothcg.github.io/web-bluetooth/#request-the-bluetooth-permission
                // Step 3.
                self.set_devices(vec!(JS::from_ref(&bt_device)));

                // https://w3c.github.io/permissions/#dom-permissions-request
                // Step 8.
                promise.resolve_native(promise_cx, self);
            },
            _ => promise.reject_error(promise_cx, Error::Type("Something went wrong...".to_owned())),
        }
    }
}
