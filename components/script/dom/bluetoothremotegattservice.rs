/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use bluetooth_traits::{BluetoothResponse, GATTType};
use dom::bindings::codegen::Bindings::BluetoothRemoteGATTServerBinding::BluetoothRemoteGATTServerMethods;
use dom::bindings::codegen::Bindings::BluetoothRemoteGATTServiceBinding;
use dom::bindings::codegen::Bindings::BluetoothRemoteGATTServiceBinding::BluetoothRemoteGATTServiceMethods;
use dom::bindings::error::Error;
use dom::bindings::reflector::reflect_dom_object;
use dom::bindings::root::{Dom, DomRoot};
use dom::bindings::str::DOMString;
use dom::bluetooth::{AsyncBluetoothListener, get_gatt_children};
use dom::bluetoothdevice::BluetoothDevice;
use dom::bluetoothuuid::{BluetoothCharacteristicUUID, BluetoothServiceUUID, BluetoothUUID};
use dom::eventtarget::EventTarget;
use dom::globalscope::GlobalScope;
use dom::promise::Promise;
use dom_struct::dom_struct;
use std::rc::Rc;

// https://webbluetoothcg.github.io/web-bluetooth/#bluetoothremotegattservice
#[dom_struct]
pub struct BluetoothRemoteGATTService {
    eventtarget: EventTarget,
    device: Dom<BluetoothDevice>,
    uuid: DOMString,
    is_primary: bool,
    instance_id: String,
}

impl BluetoothRemoteGATTService {
    pub fn new_inherited(device: &BluetoothDevice,
                         uuid: DOMString,
                         is_primary: bool,
                         instance_id: String)
                         -> BluetoothRemoteGATTService {
        BluetoothRemoteGATTService {
            eventtarget: EventTarget::new_inherited(),
            device: Dom::from_ref(device),
            uuid: uuid,
            is_primary: is_primary,
            instance_id: instance_id,
        }
    }

    pub fn new(global: &GlobalScope,
               device: &BluetoothDevice,
               uuid: DOMString,
               isPrimary: bool,
               instanceID: String)
               -> DomRoot<BluetoothRemoteGATTService> {
        reflect_dom_object(
            Box::new(BluetoothRemoteGATTService::new_inherited(
                device, uuid, isPrimary, instanceID
            )),
            global,
            BluetoothRemoteGATTServiceBinding::Wrap
        )
    }

    fn get_instance_id(&self) -> String {
        self.instance_id.clone()
    }
}

impl BluetoothRemoteGATTServiceMethods for BluetoothRemoteGATTService {
    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattservice-device
    fn Device(&self) -> DomRoot<BluetoothDevice> {
        DomRoot::from_ref(&self.device)
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattservice-isprimary
    fn IsPrimary(&self) -> bool {
        self.is_primary
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattservice-uuid
    fn Uuid(&self) -> DOMString {
        self.uuid.clone()
    }

    #[allow(unrooted_must_root)]
    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattservice-getcharacteristic
    fn GetCharacteristic(&self,
                         characteristic: BluetoothCharacteristicUUID)
                         -> Rc<Promise> {
        get_gatt_children(self, true, BluetoothUUID::characteristic, Some(characteristic), self.get_instance_id(),
                          self.Device().get_gatt().Connected(), GATTType::Characteristic)
    }

    #[allow(unrooted_must_root)]
    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattservice-getcharacteristics
    fn GetCharacteristics(&self,
                          characteristic: Option<BluetoothCharacteristicUUID>)
                          -> Rc<Promise> {
        get_gatt_children(self, false, BluetoothUUID::characteristic, characteristic, self.get_instance_id(),
                          self.Device().get_gatt().Connected(), GATTType::Characteristic)
    }

    #[allow(unrooted_must_root)]
    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattservice-getincludedservice
    fn GetIncludedService(&self,
                          service: BluetoothServiceUUID)
                          -> Rc<Promise> {
        get_gatt_children(self, false, BluetoothUUID::service, Some(service), self.get_instance_id(),
                          self.Device().get_gatt().Connected(), GATTType::IncludedService)
    }


    #[allow(unrooted_must_root)]
    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattservice-getincludedservices
    fn GetIncludedServices(&self,
                          service: Option<BluetoothServiceUUID>)
                          -> Rc<Promise> {
        get_gatt_children(self, false, BluetoothUUID::service, service, self.get_instance_id(),
                          self.Device().get_gatt().Connected(), GATTType::IncludedService)
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-serviceeventhandlers-onserviceadded
    event_handler!(serviceadded, GetOnserviceadded, SetOnserviceadded);

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-serviceeventhandlers-onservicechanged
    event_handler!(servicechanged, GetOnservicechanged, SetOnservicechanged);

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-serviceeventhandlers-onserviceremoved
    event_handler!(serviceremoved, GetOnserviceremoved, SetOnserviceremoved);
}

impl AsyncBluetoothListener for BluetoothRemoteGATTService {
    fn handle_response(&self, response: BluetoothResponse, promise: &Rc<Promise>) {
        let device = self.Device();
        match response {
            // https://webbluetoothcg.github.io/web-bluetooth/#getgattchildren
            // Step 7.
            BluetoothResponse::GetCharacteristics(characteristics_vec, single) => {
                if single {
                    promise.resolve_native(&device.get_or_create_characteristic(&characteristics_vec[0], &self));
                    return;
                }
                let mut characteristics = vec!();
                for characteristic in characteristics_vec {
                    let bt_characteristic = device.get_or_create_characteristic(&characteristic, &self);
                    characteristics.push(bt_characteristic);
                }
                promise.resolve_native(&characteristics);
            },
            // https://webbluetoothcg.github.io/web-bluetooth/#getgattchildren
            // Step 7.
            BluetoothResponse::GetIncludedServices(services_vec, single) => {
                if single {
                    return promise.resolve_native(&device.get_or_create_service(&services_vec[0], &device.get_gatt()));
                }
                let mut services = vec!();
                for service in services_vec {
                    let bt_service = device.get_or_create_service(&service, &device.get_gatt());
                    services.push(bt_service);
                }
                promise.resolve_native(&services);
            },
            _ => promise.reject_error(Error::Type("Something went wrong...".to_owned())),
        }
    }
}
