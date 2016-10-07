/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use bluetooth_traits::BluetoothMethodMsg;
use bluetooth_traits::blacklist::{Blacklist, uuid_is_blacklisted};
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::BluetoothCharacteristicPropertiesBinding::
    BluetoothCharacteristicPropertiesMethods;
use dom::bindings::codegen::Bindings::BluetoothDeviceBinding::BluetoothDeviceMethods;
use dom::bindings::codegen::Bindings::BluetoothRemoteGATTCharacteristicBinding;
use dom::bindings::codegen::Bindings::BluetoothRemoteGATTCharacteristicBinding::
    BluetoothRemoteGATTCharacteristicMethods;
use dom::bindings::codegen::Bindings::BluetoothRemoteGATTServerBinding::BluetoothRemoteGATTServerMethods;
use dom::bindings::codegen::Bindings::BluetoothRemoteGATTServiceBinding::BluetoothRemoteGATTServiceMethods;
use dom::bindings::error::{ErrorResult, Fallible};
use dom::bindings::error::Error::{self, InvalidModification, Network, NotSupported, Security};
use dom::bindings::js::{JS, MutHeap, Root};
use dom::bindings::reflector::{Reflectable, Reflector, reflect_dom_object};
use dom::bindings::str::{ByteString, DOMString};
use dom::bluetooth::result_to_promise;
use dom::bluetoothcharacteristicproperties::BluetoothCharacteristicProperties;
use dom::bluetoothremotegattdescriptor::BluetoothRemoteGATTDescriptor;
use dom::bluetoothremotegattservice::BluetoothRemoteGATTService;
use dom::bluetoothuuid::{BluetoothDescriptorUUID, BluetoothUUID};
use dom::globalscope::GlobalScope;
use dom::promise::Promise;
use ipc_channel::ipc::{self, IpcSender};
use std::rc::Rc;

// Maximum length of an attribute value.
// https://www.bluetooth.org/DocMan/handlers/DownloadDoc.ashx?doc_id=286439 (Vol. 3, page 2169)
pub const MAXIMUM_ATTRIBUTE_LENGTH: usize = 512;

// https://webbluetoothcg.github.io/web-bluetooth/#bluetoothremotegattcharacteristic
#[dom_struct]
pub struct BluetoothRemoteGATTCharacteristic {
    reflector_: Reflector,
    service: MutHeap<JS<BluetoothRemoteGATTService>>,
    uuid: DOMString,
    properties: MutHeap<JS<BluetoothCharacteristicProperties>>,
    value: DOMRefCell<Option<ByteString>>,
    instance_id: String,
}

impl BluetoothRemoteGATTCharacteristic {
    pub fn new_inherited(service: &BluetoothRemoteGATTService,
                         uuid: DOMString,
                         properties: &BluetoothCharacteristicProperties,
                         instance_id: String)
                         -> BluetoothRemoteGATTCharacteristic {
        BluetoothRemoteGATTCharacteristic {
            reflector_: Reflector::new(),
            service: MutHeap::new(service),
            uuid: uuid,
            properties: MutHeap::new(properties),
            value: DOMRefCell::new(None),
            instance_id: instance_id,
        }
    }

    pub fn new(global: &GlobalScope,
               service: &BluetoothRemoteGATTService,
               uuid: DOMString,
               properties: &BluetoothCharacteristicProperties,
               instanceID: String)
               -> Root<BluetoothRemoteGATTCharacteristic> {
        reflect_dom_object(box BluetoothRemoteGATTCharacteristic::new_inherited(service,
                                                                                uuid,
                                                                                properties,
                                                                                instanceID),
                           global,
                           BluetoothRemoteGATTCharacteristicBinding::Wrap)
    }

    fn get_bluetooth_thread(&self) -> IpcSender<BluetoothMethodMsg> {
        self.global().as_window().bluetooth_thread()
    }

    fn get_instance_id(&self) -> String {
        self.instance_id.clone()
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattcharacteristic-getdescriptor
    fn get_descriptor(&self, descriptor: BluetoothDescriptorUUID) -> Fallible<Root<BluetoothRemoteGATTDescriptor>> {
        let uuid = try!(BluetoothUUID::descriptor(descriptor)).to_string();
        if uuid_is_blacklisted(uuid.as_ref(), Blacklist::All) {
            return Err(Security)
        }
        if !self.Service().Device().Gatt().Connected() {
            return Err(Network)
        }
        let (sender, receiver) = ipc::channel().unwrap();
        self.get_bluetooth_thread().send(
            BluetoothMethodMsg::GetDescriptor(self.get_instance_id(), uuid, sender)).unwrap();
        let descriptor = receiver.recv().unwrap();
        match descriptor {
            Ok(descriptor) => {
                let context = self.service.get().get_device().get_context();
                let mut descriptor_map = context.get_descriptor_map().borrow_mut();
                if let Some(existing_descriptor) = descriptor_map.get(&descriptor.instance_id) {
                    return Ok(existing_descriptor.get());
                }
                let bt_descriptor = BluetoothRemoteGATTDescriptor::new(&self.global(),
                                                                       self,
                                                                       DOMString::from(descriptor.uuid),
                                                                       descriptor.instance_id.clone());
                descriptor_map.insert(descriptor.instance_id, MutHeap::new(&bt_descriptor));
                Ok(bt_descriptor)
            },
            Err(error) => {
                Err(Error::from(error))
            },
        }
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattcharacteristic-getdescriptors
    fn get_descriptors(&self,
                       descriptor: Option<BluetoothDescriptorUUID>)
                       -> Fallible<Vec<Root<BluetoothRemoteGATTDescriptor>>> {
        let mut uuid: Option<String> = None;
        if let Some(d) = descriptor {
            uuid = Some(try!(BluetoothUUID::descriptor(d)).to_string());
            if let Some(ref uuid) = uuid {
                if uuid_is_blacklisted(uuid.as_ref(), Blacklist::All) {
                    return Err(Security)
                }
            }
        };
        if !self.Service().Device().Gatt().Connected() {
            return Err(Network)
        }
        let mut descriptors = vec!();
        let (sender, receiver) = ipc::channel().unwrap();
        self.get_bluetooth_thread().send(
            BluetoothMethodMsg::GetDescriptors(self.get_instance_id(), uuid, sender)).unwrap();
        let descriptors_vec = receiver.recv().unwrap();
        match descriptors_vec {
            Ok(descriptor_vec) => {
                let context = self.service.get().get_device().get_context();
                let mut descriptor_map = context.get_descriptor_map().borrow_mut();
                for descriptor in descriptor_vec {
                    let bt_descriptor = match descriptor_map.get(&descriptor.instance_id) {
                        Some(existing_descriptor) => existing_descriptor.get(),
                        None => {
                            BluetoothRemoteGATTDescriptor::new(&self.global(),
                                                               self,
                                                               DOMString::from(descriptor.uuid),
                                                               descriptor.instance_id.clone())
                        },
                    };
                    if !descriptor_map.contains_key(&descriptor.instance_id) {
                        descriptor_map.insert(descriptor.instance_id, MutHeap::new(&bt_descriptor));
                    }
                    descriptors.push(bt_descriptor);
                }
                Ok(descriptors)
            },
            Err(error) => {
                Err(Error::from(error))
            },
        }
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattcharacteristic-readvalue
    fn read_value(&self) -> Fallible<ByteString> {
        if uuid_is_blacklisted(self.uuid.as_ref(), Blacklist::Reads) {
            return Err(Security)
        }
        let (sender, receiver) = ipc::channel().unwrap();
        if !self.Service().Device().Gatt().Connected() {
            return Err(Network)
        }
        if !self.Properties().Read() {
            return Err(NotSupported)
        }
        self.get_bluetooth_thread().send(
            BluetoothMethodMsg::ReadValue(self.get_instance_id(), sender)).unwrap();
        let result = receiver.recv().unwrap();
        let value = match result {
            Ok(val) => {
                ByteString::new(val)
            },
            Err(error) => {
                return Err(Error::from(error))
            },
        };
        *self.value.borrow_mut() = Some(value.clone());
        Ok(value)
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattcharacteristic-writevalue
    fn write_value(&self, value: Vec<u8>) -> ErrorResult {
        if uuid_is_blacklisted(self.uuid.as_ref(), Blacklist::Writes) {
            return Err(Security)
        }
        if value.len() > MAXIMUM_ATTRIBUTE_LENGTH {
            return Err(InvalidModification)
        }
        if !self.Service().Device().Gatt().Connected() {
            return Err(Network)
        }

        if !(self.Properties().Write() ||
             self.Properties().WriteWithoutResponse() ||
             self.Properties().AuthenticatedSignedWrites()) {
            return Err(NotSupported)
        }
        let (sender, receiver) = ipc::channel().unwrap();
        self.get_bluetooth_thread().send(
            BluetoothMethodMsg::WriteValue(self.get_instance_id(), value.clone(), sender)).unwrap();
        let result = receiver.recv().unwrap();
        match result {
            Ok(_) => Ok(*self.value.borrow_mut() = Some(ByteString::new(value))),
            Err(error) => {
                Err(Error::from(error))
            },
        }
    }
}

impl BluetoothRemoteGATTCharacteristicMethods for BluetoothRemoteGATTCharacteristic {
    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattcharacteristic-properties
    fn Properties(&self) -> Root<BluetoothCharacteristicProperties> {
        self.properties.get()
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattcharacteristic-service
    fn Service(&self) -> Root<BluetoothRemoteGATTService> {
        self.service.get()
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattcharacteristic-uuid
    fn Uuid(&self) -> DOMString {
        self.uuid.clone()
    }

    #[allow(unrooted_must_root)]
    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattcharacteristic-getdescriptor
    fn GetDescriptor(&self, descriptor: BluetoothDescriptorUUID) -> Rc<Promise> {
        result_to_promise(&self.global(), self.get_descriptor(descriptor))
    }

    #[allow(unrooted_must_root)]
    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattcharacteristic-getdescriptors
    fn GetDescriptors(&self,
                      descriptor: Option<BluetoothDescriptorUUID>)
                      -> Rc<Promise> {
        result_to_promise(&self.global(), self.get_descriptors(descriptor))
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattcharacteristic-value
    fn GetValue(&self) -> Option<ByteString> {
        self.value.borrow().clone()
    }

    #[allow(unrooted_must_root)]
    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattcharacteristic-readvalue
    fn ReadValue(&self) -> Rc<Promise> {
        result_to_promise(&self.global(), self.read_value())
    }

    #[allow(unrooted_must_root)]
    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattcharacteristic-writevalue
    fn WriteValue(&self, value: Vec<u8>) -> Rc<Promise> {
        result_to_promise(&self.global(), self.write_value(value))
    }
}
