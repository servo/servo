/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use bluetooth_traits::{BluetoothRequest, BluetoothResponse, GATTType};
use dom::bindings::codegen::Bindings::BluetoothDeviceBinding::BluetoothDeviceMethods;
use dom::bindings::codegen::Bindings::BluetoothRemoteGATTServerBinding;
use dom::bindings::codegen::Bindings::BluetoothRemoteGATTServerBinding::BluetoothRemoteGATTServerMethods;
use dom::bindings::error::Error;
use dom::bindings::error::ErrorResult;
use dom::bindings::js::{JS, Root};
use dom::bindings::reflector::{DomObject, Reflector, reflect_dom_object};
use dom::bluetooth::{AsyncBluetoothListener, get_gatt_children, response_async};
use dom::bluetoothdevice::BluetoothDevice;
use dom::bluetoothuuid::{BluetoothServiceUUID, BluetoothUUID};
use dom::globalscope::GlobalScope;
use dom::promise::Promise;
use ipc_channel::ipc::IpcSender;
use js::jsapi::JSContext;
use std::cell::Cell;
use std::rc::Rc;

// https://webbluetoothcg.github.io/web-bluetooth/#bluetoothremotegattserver
#[dom_struct]
pub struct BluetoothRemoteGATTServer {
    reflector_: Reflector,
    device: JS<BluetoothDevice>,
    connected: Cell<bool>,
}

impl BluetoothRemoteGATTServer {
    pub fn new_inherited(device: &BluetoothDevice) -> BluetoothRemoteGATTServer {
        BluetoothRemoteGATTServer {
            reflector_: Reflector::new(),
            device: JS::from_ref(device),
            connected: Cell::new(false),
        }
    }

    pub fn new(global: &GlobalScope, device: &BluetoothDevice) -> Root<BluetoothRemoteGATTServer> {
        reflect_dom_object(box BluetoothRemoteGATTServer::new_inherited(device),
                           global,
                           BluetoothRemoteGATTServerBinding::Wrap)
    }

    fn get_bluetooth_thread(&self) -> IpcSender<BluetoothRequest> {
        self.global().as_window().bluetooth_thread()
    }

    pub fn set_connected(&self, connected: bool) {
        self.connected.set(connected);
    }
}

impl BluetoothRemoteGATTServerMethods for BluetoothRemoteGATTServer {
    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattserver-device
    fn Device(&self) -> Root<BluetoothDevice> {
        Root::from_ref(&self.device)
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattserver-connected
    fn Connected(&self) -> bool {
        self.connected.get()
    }

    #[allow(unrooted_must_root)]
    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattserver-connect
    fn Connect(&self) -> Rc<Promise> {
        // Step 1.
        let p = Promise::new(&self.global());
        let sender = response_async(&p, self);

        // TODO: Step 3: Check if the UA is currently using the Bluetooth system.

        // TODO: Step 4: Implement activeAlgorithms internal slot for BluetoothRemoteGATTServer.

        // TODO: Step 5.1 - 5.2: Implement activeAlgorithms internal slot for BluetoothRemoteGATTServer.

        // Note: Steps 2, 5.1.1 and 5.1.3 are in components/bluetooth/lib.rs in the gatt_server_connect function.
        // Steps 5.2.3 - 5.2.5  are in response function.
        self.get_bluetooth_thread().send(
            BluetoothRequest::GATTServerConnect(String::from(self.Device().Id()), sender)).unwrap();
        // Step 5: return promise.
        return p;
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattserver-disconnect
    fn Disconnect(&self) -> ErrorResult {
        // TODO: Step 1: Implement activeAlgorithms internal slot for BluetoothRemoteGATTServer.

        // Step 2.
        if !self.Connected() {
            return Ok(())
        }

        // Step 3.
        self.Device().clean_up_disconnected_device();

        // Step 4 - 5:
        self.Device().garbage_collect_the_connection()
    }

    #[allow(unrooted_must_root)]
    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattserver-getprimaryservice
    fn GetPrimaryService(&self, service: BluetoothServiceUUID) -> Rc<Promise> {
        // Step 1. is in get_gatt_children
        // Step 2.
        get_gatt_children(self, true, BluetoothUUID::service, Some(service), String::from(self.Device().Id()),
                          self.Device().get_gatt().Connected(), GATTType::PrimaryService)
    }

    #[allow(unrooted_must_root)]
    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattserver-getprimaryservices
    fn GetPrimaryServices(&self, service: Option<BluetoothServiceUUID>) -> Rc<Promise> {
        // Step 1. is in get_gatt_children
        // Step 2.
        get_gatt_children(self, false, BluetoothUUID::service, service, String::from(self.Device().Id()),
                          self.Connected(), GATTType::PrimaryService)

    }
}

impl AsyncBluetoothListener for BluetoothRemoteGATTServer {
    fn handle_response(&self, response: BluetoothResponse, promise_cx: *mut JSContext, promise: &Rc<Promise>) {
        match response {
            // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattserver-connect
            BluetoothResponse::GATTServerConnect(connected) => {
                // Step 5.2.3
                if self.Device().is_represented_device_null() {
                    if let Err(e) = self.Device().garbage_collect_the_connection() {
                        return promise.reject_error(promise_cx, Error::from(e));
                    }
                    return promise.reject_error(promise_cx, Error::Network);
                }

                // Step 5.2.4.
                self.connected.set(connected);

                // Step 5.2.5.
                promise.resolve_native(promise_cx, self);
            },
            // https://webbluetoothcg.github.io/web-bluetooth/#getgattchildren
            // Step 7.
            BluetoothResponse::GetPrimaryServices(services_vec, single) => {
                let device = self.Device();
                if single {
                    promise.resolve_native(promise_cx, &device.get_or_create_service(&services_vec[0], &self));
                    return;
                }
                let mut services = vec!();
                for service in services_vec {
                    let bt_service = device.get_or_create_service(&service, &self);
                    services.push(bt_service);
                }
                promise.resolve_native(promise_cx, &services);
            },
            _ => promise.reject_error(promise_cx, Error::Type("Something went wrong...".to_owned())),
        }
    }
}
