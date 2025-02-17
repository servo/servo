/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use bluetooth_traits::{BluetoothRequest, BluetoothResponse};
use dom_struct::dom_struct;
use ipc_channel::ipc::IpcSender;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::BluetoothPermissionResultBinding::BluetoothPermissionResultMethods;
use crate::dom::bindings::codegen::Bindings::NavigatorBinding::Navigator_Binding::NavigatorMethods;
use crate::dom::bindings::codegen::Bindings::PermissionStatusBinding::PermissionStatus_Binding::PermissionStatusMethods;
use crate::dom::bindings::codegen::Bindings::PermissionStatusBinding::{
    PermissionName, PermissionState,
};
use crate::dom::bindings::codegen::Bindings::WindowBinding::Window_Binding::WindowMethods;
use crate::dom::bindings::error::Error;
use crate::dom::bindings::reflector::{reflect_dom_object, DomGlobal};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::bluetooth::{AllowedBluetoothDevice, AsyncBluetoothListener, Bluetooth};
use crate::dom::bluetoothdevice::BluetoothDevice;
use crate::dom::globalscope::GlobalScope;
use crate::dom::permissionstatus::PermissionStatus;
use crate::dom::promise::Promise;
use crate::script_runtime::CanGc;

// https://webbluetoothcg.github.io/web-bluetooth/#bluetoothpermissionresult
#[dom_struct]
pub(crate) struct BluetoothPermissionResult {
    status: PermissionStatus,
    devices: DomRefCell<Vec<Dom<BluetoothDevice>>>,
}

impl BluetoothPermissionResult {
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    fn new_inherited(status: &PermissionStatus) -> BluetoothPermissionResult {
        let result = BluetoothPermissionResult {
            status: PermissionStatus::new_inherited(status.get_query()),
            devices: DomRefCell::new(Vec::new()),
        };
        result.status.set_state(status.State());
        result
    }

    pub(crate) fn new(
        global: &GlobalScope,
        status: &PermissionStatus,
        can_gc: CanGc,
    ) -> DomRoot<BluetoothPermissionResult> {
        reflect_dom_object(
            Box::new(BluetoothPermissionResult::new_inherited(status)),
            global,
            can_gc,
        )
    }

    pub(crate) fn get_bluetooth(&self) -> DomRoot<Bluetooth> {
        self.global().as_window().Navigator().Bluetooth()
    }

    pub(crate) fn get_bluetooth_thread(&self) -> IpcSender<BluetoothRequest> {
        self.global().as_window().bluetooth_thread()
    }

    pub(crate) fn get_query(&self) -> PermissionName {
        self.status.get_query()
    }

    pub(crate) fn set_state(&self, state: PermissionState) {
        self.status.set_state(state)
    }

    pub(crate) fn get_state(&self) -> PermissionState {
        self.status.State()
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn set_devices(&self, devices: Vec<Dom<BluetoothDevice>>) {
        *self.devices.borrow_mut() = devices;
    }
}

impl BluetoothPermissionResultMethods<crate::DomTypeHolder> for BluetoothPermissionResult {
    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothpermissionresult-devices
    fn Devices(&self) -> Vec<DomRoot<BluetoothDevice>> {
        let device_vec: Vec<DomRoot<BluetoothDevice>> = self
            .devices
            .borrow()
            .iter()
            .map(|d| DomRoot::from_ref(&**d))
            .collect();
        device_vec
    }
}

impl AsyncBluetoothListener for BluetoothPermissionResult {
    fn handle_response(&self, response: BluetoothResponse, promise: &Rc<Promise>, can_gc: CanGc) {
        match response {
            // https://webbluetoothcg.github.io/web-bluetooth/#request-bluetooth-devices
            // Step 3, 11, 13 - 14.
            BluetoothResponse::RequestDevice(device) => {
                self.set_state(PermissionState::Granted);
                let bluetooth = self.get_bluetooth();
                let mut device_instance_map = bluetooth.get_device_map().borrow_mut();
                if let Some(existing_device) = device_instance_map.get(&device.id) {
                    // https://webbluetoothcg.github.io/web-bluetooth/#request-the-bluetooth-permission
                    // Step 3.
                    self.set_devices(vec![Dom::from_ref(existing_device)]);

                    // https://w3c.github.io/permissions/#dom-permissions-request
                    // Step 8.
                    return promise.resolve_native(self);
                }
                let bt_device = BluetoothDevice::new(
                    &self.global(),
                    DOMString::from(device.id.clone()),
                    device.name.map(DOMString::from),
                    &bluetooth,
                    can_gc,
                );
                device_instance_map.insert(device.id.clone(), Dom::from_ref(&bt_device));
                self.global()
                    .as_window()
                    .bluetooth_extra_permission_data()
                    .add_new_allowed_device(AllowedBluetoothDevice {
                        deviceId: DOMString::from(device.id),
                        mayUseGATT: true,
                    });
                // https://webbluetoothcg.github.io/web-bluetooth/#request-the-bluetooth-permission
                // Step 3.
                self.set_devices(vec![Dom::from_ref(&bt_device)]);

                // https://w3c.github.io/permissions/#dom-permissions-request
                // Step 8.
                promise.resolve_native(self);
            },
            _ => promise.reject_error(Error::Type("Something went wrong...".to_owned())),
        }
    }
}
