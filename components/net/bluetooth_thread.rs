/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use device::bluetooth::BluetoothAdapter;
use device::bluetooth::BluetoothDevice;
use device::bluetooth::BluetoothGATTCharacteristic;
use device::bluetooth::BluetoothGATTDescriptor;
use device::bluetooth::BluetoothGATTService;
use ipc_channel::ipc::{self, IpcReceiver, IpcSender};
use net_traits::bluetooth_thread::{BluetoothMethodMsg, BluetoothObjectMsg};
use std::borrow::ToOwned;
use std::collections::HashMap;
use std::string::String;
use util::thread::spawn_named;

pub trait BluetoothThreadFactory {
    fn new() -> Self;
}

impl BluetoothThreadFactory for IpcSender<BluetoothMethodMsg> {
    fn new() -> IpcSender<BluetoothMethodMsg> {
        let (sender, receiver) = ipc::channel().unwrap();
        let adapter = BluetoothAdapter::init();
        spawn_named("BluetoothThread".to_owned(), move || {
            BluetoothManager::new(receiver, adapter).start();
        });
        sender
    }
}

pub struct BluetoothManager {
    receiver: IpcReceiver<BluetoothMethodMsg>,
    adapter: BluetoothAdapter,
    service_to_device: HashMap<String, String>,
    characteristic_to_service: HashMap<String, String>,
    descriptor_to_characteristic: HashMap<String, String>,
    cached_devices: HashMap<String, BluetoothDevice>,
    cached_services: HashMap<String, BluetoothGATTService>,
    cached_characteristic: HashMap<String, BluetoothGATTCharacteristic>,
    cached_descriptors: HashMap<String, BluetoothGATTDescriptor>,
}

impl BluetoothManager {
    pub fn new (receiver: IpcReceiver<BluetoothMethodMsg>, adapter: BluetoothAdapter) -> BluetoothManager {
        BluetoothManager {
            receiver: receiver,
            adapter: adapter,
            service_to_device: HashMap::new(),
            characteristic_to_service: HashMap::new(),
            descriptor_to_characteristic: HashMap::new(),
            cached_devices: HashMap::new(),
            cached_services: HashMap::new(),
            cached_characteristic: HashMap::new(),
            cached_descriptors: HashMap::new(),
        }
    }

    fn start(&mut self) {
        loop {
            match self.receiver.recv().unwrap() {
                BluetoothMethodMsg::RequestDevice(sender) => {
                    self.request_device(sender)
                }
                BluetoothMethodMsg::GATTServerConnect(device_id, sender) => {
                    self.gatt_server_connect(device_id, sender)
                }
                BluetoothMethodMsg::GATTServerDisconnect(device_id, sender) => {
                    self.gatt_server_disconnect(device_id, sender)
                }
                BluetoothMethodMsg::GetPrimaryService(device_id, sender) => {
                    self.get_primary_service(device_id, sender)
                }
                BluetoothMethodMsg::GetCharacteristic(service_id, sender) => {
                    self.get_characteristic(service_id, sender)
                }
                BluetoothMethodMsg::GetDescriptor(characteristic_id, sender) => {
                    self.get_descriptor(characteristic_id, sender)
                }
                BluetoothMethodMsg::ReadValue(id, sender) => {
                    self.read_value(id, sender)
                }
                BluetoothMethodMsg::WriteValue(id, value, sender) => {
                    self.write_value(id, value, sender)
                }
                BluetoothMethodMsg::Exit => {
                    break
                }
            }
        }
    }

    fn request_device(&mut self, sender: IpcSender<BluetoothObjectMsg>) {
        if !self.adapter.is_initialized() {
            sender.send(BluetoothObjectMsg::Error { error: String::from("No adapter found") }).unwrap();
            return;
        }
        let devices = self.adapter.get_devices(true);
        if devices.is_empty() {
            sender.send(BluetoothObjectMsg::Error { error: String::from("No device found") }).unwrap();
            return;
        }

        for device in &devices {
            self.cached_devices.insert(device.get_address(), device.clone());
        }

        let device = &devices[0];

        let message = BluetoothObjectMsg::BluetoothDevice {
            id: device.get_address(),
            name: Some(device.get_name()),
            device_class: Some(device.get_class()),
            vendor_id_source: Some(device.get_vendor_id_source()),
            vendor_id: Some(device.get_vendor_id()),
            product_id: Some(device.get_product_id()),
            product_version: Some(device.get_device_id()),
            appearance: Some(device.get_appearance()),
            tx_power: Some(device.get_tx_power() as i8),
            rssi: Some(device.get_rssi() as i8)
        };
        sender.send(message).unwrap();
    }

    pub fn gatt_server_connect(&mut self, device_id: String, sender: IpcSender<BluetoothObjectMsg>) {
        if !self.adapter.is_initialized() {
            sender.send(BluetoothObjectMsg::Error { error: String::from("No adapter found") }).unwrap();
            return;
        }

        let device = match self.adapter.get_device(device_id, true) {
            Some(d) => d,
            None => {
                sender.send(BluetoothObjectMsg::Error { error: String::from("No device found") }).unwrap();
                return;
            }
        };

        //TODO check result
        device.connect();
        let message = BluetoothObjectMsg::BluetoothServer {
            connected: true
        };
        sender.send(message).unwrap();
    }

    pub fn gatt_server_disconnect(&mut self, device_id: String, sender: IpcSender<BluetoothObjectMsg>) {
        if !self.adapter.is_initialized() {
            sender.send(BluetoothObjectMsg::Error { error: String::from("No adapter found") }).unwrap();
            return;
        }

        let device = match self.adapter.get_device(device_id, true) {
            Some(d) => d,
            None => {
                sender.send(BluetoothObjectMsg::Error { error: String::from("No device found") }).unwrap();
                return;
            }
        };

        //FIXME check result
        device.disconnect();
        let message = BluetoothObjectMsg::BluetoothServer {
            connected: false
        };
        sender.send(message).unwrap();
    }

    pub fn get_primary_service(&mut self, device_id: String, sender: IpcSender<BluetoothObjectMsg>) {
        if !self.adapter.is_initialized() {
            sender.send(BluetoothObjectMsg::Error { error: String::from("No adapter found") }).unwrap();
            return;
        }

        let device = match self.adapter.get_device(device_id.clone(), true) {
            Some(d) => d,
            None => {
                sender.send(BluetoothObjectMsg::Error { error: String::from("No device found") }).unwrap();
                return;
            }
        };

        let services = device.get_gatt_services();
        if services.is_empty() {
            sender.send(BluetoothObjectMsg::Error { error: String::from("No service found") }).unwrap();
            return;
        }

        for service in services {
            self.service_to_device.insert(service.get_object_path(), device_id.clone());
            self.cached_services.insert(service.get_object_path(), service.clone());
            if service.is_primary() {
                let message = BluetoothObjectMsg::BluetoothService {
                    uuid: service.get_uuid(),
                    is_primary: true,
                    instance_id: service.get_object_path()
                };
                sender.send(message).unwrap();
                return;
            }
        }

        sender.send(BluetoothObjectMsg::Error { error: String::from("No primary service found") }).unwrap();
    }

    pub fn get_characteristic(&mut self, service_id: String, sender: IpcSender<BluetoothObjectMsg>) {
        if !self.adapter.is_initialized() {
            sender.send(BluetoothObjectMsg::Error { error: String::from("No adapter found") }).unwrap();
            return;
        }

        let device_id = match self.service_to_device.get(&service_id) {
            Some(d) => d,
            _ => {
                sender.send(
                    BluetoothObjectMsg::Error {
                        error: String::from("Unknown device id for service: ") + &service_id }).unwrap();
                return;
            }
        };

        let device = match self.adapter.get_device(device_id.clone(), true) {
            Some(d) => d,
            None => {
                sender.send(BluetoothObjectMsg::Error { error: String::from("No device found") }).unwrap();
                return;
            }
        };

        let services = device.get_gatt_services();
        if services.is_empty() {
            sender.send(BluetoothObjectMsg::Error { error: String::from("No service found") }).unwrap();
            return;
        }

        for service in services {
            if service.get_object_path() == service_id {
                let characteristics = service.get_characteristics();
                if characteristics.is_empty() {
                    sender.send(BluetoothObjectMsg::Error { error: String::from("No characteristic found") }).unwrap();
                    return;
                }
                for characteristic in &characteristics {
                    self.characteristic_to_service.insert(characteristic.get_object_path(), service_id.clone());
                    self.cached_characteristic.insert(characteristic.get_object_path(), characteristic.clone());
                }
                let characteristic = &characteristics[0];
                let message = BluetoothObjectMsg::BluetoothCharacteristic {
                    uuid: characteristic.get_uuid(),
                    instance_id: characteristic.get_object_path(),
                    // TODO extract members from flags
                    broadcast: false,
                    read: false,
                    write_without_response: false,
                    write: false,
                    notify: false,
                    indicate: false,
                    authenticated_signed_writes: false,
                    reliable_write: false,
                    writable_auxiliaries: false
                };
                sender.send(message).unwrap();
                return;
            }
        }

        sender.send(BluetoothObjectMsg::Error { error: String::from("No characteristic found") }).unwrap();
    }

    pub fn get_descriptor(&mut self, characteristic_id: String, sender: IpcSender<BluetoothObjectMsg>) {
        if !self.adapter.is_initialized() {
            sender.send(BluetoothObjectMsg::Error { error: String::from("No adapter found") }).unwrap();
            return;
        }

        let service_id: String = match self.characteristic_to_service.get(&characteristic_id) {
            Some(s) => s.clone(),
            _ => {
                sender.send(
                    BluetoothObjectMsg::Error { error:
                        String::from("Unknown service id for characteristic: ") + &characteristic_id }).unwrap();
                return;
            }
        };

        let device_id = match self.service_to_device.get(&service_id) {
            Some(d) => d,
            _ => {
                sender.send(
                    BluetoothObjectMsg::Error { error:
                        String::from("Unknown device id for service: ") + &service_id }).unwrap();
                return;
            }
        };

        let device = match self.adapter.get_device(device_id.clone(), true) {
            Some(d) => d,
            None => {
                sender.send(BluetoothObjectMsg::Error { error: String::from("No device found") }).unwrap();
                return;
            }
        };

        let services = device.get_gatt_services();
        if services.is_empty() {
            sender.send(BluetoothObjectMsg::Error { error: String::from("No service found") }).unwrap();
            return;
        }

        for service in services {
            if service.get_object_path() == service_id {
                let characteristics = service.get_characteristics();
                if characteristics.is_empty() {
                    sender.send(BluetoothObjectMsg::Error { error: String::from("No characteristic found") }).unwrap();
                    return;
                }

                for characteristic in characteristics {
                    if characteristic.get_object_path() == characteristic_id {
                        let descriptors = characteristic.get_descriptors();
                        if descriptors.is_empty() {
                            sender.send(
                                BluetoothObjectMsg::Error { error:
                                    String::from("No descriptor found") }).unwrap();
                            return;
                        }
                        for descriptor in &descriptors {
                            self.descriptor_to_characteristic.insert(descriptor.get_object_path(),
                                                                     characteristic_id.clone());
                            self.cached_descriptors.insert(descriptor.get_object_path(), descriptor.clone());
                        }
                        let descriptor = &descriptors[0];
                        let message = BluetoothObjectMsg::BluetoothDescriptor {
                            uuid: descriptor.get_uuid(),
                            instance_id: descriptor.get_object_path(),
                        };
                        sender.send(message).unwrap();
                        return;
                    }
                }
            }
        }

        sender.send(BluetoothObjectMsg::Error { error: String::from("No characteristic found") }).unwrap();

    }

    pub fn read_value(&self, _id: String, sender: IpcSender<BluetoothObjectMsg>) {
        let message = BluetoothObjectMsg::BluetoothValue {
            value: vec!(10, 11, 12, 13, 14, 15, 16)
        };
        sender.send(message).unwrap();
    }

    pub fn write_value(&self, _id: String, _value: Vec<u8>, _sender: IpcSender<BluetoothObjectMsg>) {
        unimplemented!()
    }
}
