/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::rc::Rc;

use base::generic_channel::GenericSender;
use bluetooth_traits::{BluetoothRequest, BluetoothResponse, GATTType};
use dom_struct::dom_struct;
use js::realm::CurrentRealm;

use crate::dom::bindings::codegen::Bindings::BluetoothDeviceBinding::BluetoothDeviceMethods;
use crate::dom::bindings::codegen::Bindings::BluetoothRemoteGATTServerBinding::BluetoothRemoteGATTServerMethods;
use crate::dom::bindings::error::{Error, ErrorResult};
use crate::dom::bindings::reflector::{DomGlobal, Reflector, reflect_dom_object};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bluetooth::{AsyncBluetoothListener, get_gatt_children, response_async};
use crate::dom::bluetoothdevice::BluetoothDevice;
use crate::dom::bluetoothuuid::{BluetoothServiceUUID, BluetoothUUID};
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::script_runtime::CanGc;

// https://webbluetoothcg.github.io/web-bluetooth/#bluetoothremotegattserver
#[dom_struct]
pub(crate) struct BluetoothRemoteGATTServer {
    reflector_: Reflector,
    device: Dom<BluetoothDevice>,
    connected: Cell<bool>,
}

impl BluetoothRemoteGATTServer {
    pub(crate) fn new_inherited(device: &BluetoothDevice) -> BluetoothRemoteGATTServer {
        BluetoothRemoteGATTServer {
            reflector_: Reflector::new(),
            device: Dom::from_ref(device),
            connected: Cell::new(false),
        }
    }

    pub(crate) fn new(
        cx: &mut js::context::JSContext,
        global: &GlobalScope,
        device: &BluetoothDevice,
    ) -> DomRoot<BluetoothRemoteGATTServer> {
        reflect_dom_object(
            Box::new(BluetoothRemoteGATTServer::new_inherited(device)),
            global,
            CanGc::from_cx(cx),
        )
    }

    fn get_bluetooth_thread(&self) -> GenericSender<BluetoothRequest> {
        self.global().as_window().bluetooth_thread()
    }

    pub(crate) fn set_connected(&self, connected: bool) {
        self.connected.set(connected);
    }
}

impl BluetoothRemoteGATTServerMethods<crate::DomTypeHolder> for BluetoothRemoteGATTServer {
    /// <https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattserver-device>
    fn Device(&self) -> DomRoot<BluetoothDevice> {
        DomRoot::from_ref(&self.device)
    }

    /// <https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattserver-connected>
    fn Connected(&self) -> bool {
        self.connected.get()
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattserver-connect
    fn Connect(&self, cx: &mut CurrentRealm) -> Rc<Promise> {
        // Step 1.
        let p = Promise::new_in_realm(cx);
        let sender = response_async(&p, self);

        // TODO: Step 3: Check if the UA is currently using the Bluetooth system.

        // TODO: Step 4: Implement activeAlgorithms internal slot for BluetoothRemoteGATTServer.

        // TODO: Step 5.1 - 5.2: Implement activeAlgorithms internal slot for BluetoothRemoteGATTServer.

        // Note: Steps 2, 5.1.1 and 5.1.3 are in components/bluetooth/lib.rs in the gatt_server_connect function.
        // Steps 5.2.3 - 5.2.5  are in response function.
        self.get_bluetooth_thread()
            .send(BluetoothRequest::GATTServerConnect(
                String::from(self.Device().Id()),
                sender,
            ))
            .unwrap();
        // Step 5: return promise.
        p
    }

    /// <https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattserver-disconnect>
    fn Disconnect(&self, cx: &mut js::context::JSContext) -> ErrorResult {
        // TODO: Step 1: Implement activeAlgorithms internal slot for BluetoothRemoteGATTServer.

        // Step 2.
        if !self.Connected() {
            return Ok(());
        }

        // Step 3.
        self.Device().clean_up_disconnected_device(cx);

        // Step 4 - 5:
        self.Device().garbage_collect_the_connection(cx)
    }

    /// <https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattserver-getprimaryservice>
    fn GetPrimaryService(
        &self,
        cx: &mut js::context::JSContext,
        service: BluetoothServiceUUID,
    ) -> Rc<Promise> {
        let is_connected = self.Device().get_gatt(cx).Connected();
        // Step 1 - 2.
        get_gatt_children(
            cx,
            self,
            true,
            BluetoothUUID::service,
            Some(service),
            String::from(self.Device().Id()),
            is_connected,
            GATTType::PrimaryService,
        )
    }

    /// <https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattserver-getprimaryservices>
    fn GetPrimaryServices(
        &self,
        cx: &mut js::context::JSContext,
        service: Option<BluetoothServiceUUID>,
    ) -> Rc<Promise> {
        // Step 1 - 2.
        get_gatt_children(
            cx,
            self,
            false,
            BluetoothUUID::service,
            service,
            String::from(self.Device().Id()),
            self.Connected(),
            GATTType::PrimaryService,
        )
    }
}

impl AsyncBluetoothListener for BluetoothRemoteGATTServer {
    fn handle_response(
        &self,
        cx: &mut js::context::JSContext,
        response: BluetoothResponse,
        promise: &Rc<Promise>,
    ) {
        match response {
            // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattserver-connect
            BluetoothResponse::GATTServerConnect(connected) => {
                let can_gc = CanGc::from_cx(cx);
                // Step 5.2.3
                if self.Device().is_represented_device_null() {
                    if let Err(e) = self.Device().garbage_collect_the_connection(cx) {
                        return promise.reject_error(e, can_gc);
                    }
                    return promise.reject_error(Error::Network(None), can_gc);
                }

                // Step 5.2.4.
                self.connected.set(connected);

                // Step 5.2.5.
                promise.resolve_native(self, can_gc);
            },
            // https://webbluetoothcg.github.io/web-bluetooth/#getgattchildren
            // Step 7.
            BluetoothResponse::GetPrimaryServices(services_vec, single) => {
                let can_gc = CanGc::from_cx(cx);
                let device = self.Device();
                if single {
                    promise.resolve_native(
                        &device.get_or_create_service(cx, &services_vec[0], self),
                        can_gc,
                    );
                    return;
                }
                let mut services = vec![];
                for service in services_vec {
                    let bt_service = device.get_or_create_service(cx, &service, self);
                    services.push(bt_service);
                }
                promise.resolve_native(&services, can_gc);
            },
            _ => promise.reject_error(
                Error::Type(c"Something went wrong...".to_owned()),
                CanGc::from_cx(cx),
            ),
        }
    }
}
