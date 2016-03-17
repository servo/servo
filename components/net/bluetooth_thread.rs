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

macro_rules! send_error(
    ($sender:expr, $error:expr) => (
        return $sender.send(BluetoothObjectMsg::Error { error: String::from($error) }).unwrap();
    );
);

macro_rules! check_cache(
    ($cache:expr, $key:expr) => (
        if $cache.contains_key($key) {
            return $cache.get($key);
        }
    );
);

pub trait BluetoothThreadFactory {
    fn new() -> Self;
}

impl BluetoothThreadFactory for IpcSender<BluetoothMethodMsg> {
    fn new() -> IpcSender<BluetoothMethodMsg> {
        let (sender, receiver) = ipc::channel().unwrap();
        let adapter = match BluetoothAdapter::init() {
            Ok(a) => Some(a),
            Err(_) => None,
        };
        spawn_named("BluetoothThread".to_owned(), move || {
            BluetoothManager::new(receiver, adapter).start();
        });
        sender
    }
}

pub struct BluetoothManager {
    receiver: IpcReceiver<BluetoothMethodMsg>,
    adapter: Option<BluetoothAdapter>,
    service_to_device: HashMap<String, String>,
    characteristic_to_service: HashMap<String, String>,
    descriptor_to_characteristic: HashMap<String, String>,
    cached_devices: HashMap<String, BluetoothDevice>,
    cached_services: HashMap<String, BluetoothGATTService>,
    cached_characteristics: HashMap<String, BluetoothGATTCharacteristic>,
    cached_descriptors: HashMap<String, BluetoothGATTDescriptor>,
}

impl BluetoothManager {
    pub fn new (receiver: IpcReceiver<BluetoothMethodMsg>, adapter: Option<BluetoothAdapter>) -> BluetoothManager {
        BluetoothManager {
            receiver: receiver,
            adapter: adapter,
            service_to_device: HashMap::new(),
            characteristic_to_service: HashMap::new(),
            descriptor_to_characteristic: HashMap::new(),
            cached_devices: HashMap::new(),
            cached_services: HashMap::new(),
            cached_characteristics: HashMap::new(),
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

    // Adapter

    fn get_adapter(&mut self) -> Option<BluetoothAdapter> {
        if self.adapter.is_none()
        || self.adapter.clone().unwrap().get_address().is_err() {
            self.adapter = BluetoothAdapter::init().ok();
        }
        self.adapter.clone()
    }

    // Device

    fn get_devices(&mut self, adapter: &mut BluetoothAdapter) -> Vec<BluetoothDevice> {
        let devices = adapter.get_devices().unwrap_or(vec!());
        for device in &devices {
            self.cached_devices.insert(device.get_address().unwrap_or("".to_owned()), device.clone());
        }
        devices
    }

    fn get_device(&mut self, adapter: &mut BluetoothAdapter, device_id: &str) -> Option<&BluetoothDevice> {
        check_cache!(self.cached_devices, device_id);
        // Update cache
        self.get_devices(adapter);
        check_cache!(self.cached_devices, device_id);
        None
    }

    // Service

    fn get_gatt_services(&mut self, adapter: &mut BluetoothAdapter, device_id: &str) -> Vec<BluetoothGATTService> {
        let services = match self.get_device(adapter, device_id) {
            Some(d) => d.get_gatt_services().unwrap_or(vec!()),
            None => vec!(),
        };
        for service in &services {
            self.cached_services.insert(service.get_object_path(), service.clone());
            self.service_to_device.insert(service.get_object_path(), device_id.to_owned());
        }
        services
    }

    fn get_gatt_service(&mut self,
                        adapter: &mut BluetoothAdapter,
                        service_id: &str)
                        -> Option<&BluetoothGATTService> {
        check_cache!(self.cached_services, service_id);
        let device_id = match self.service_to_device.get_mut(service_id) {
            Some(d) => d.clone(),
            None => return None,
        };
        // Update cache
        self.get_gatt_services(adapter, &device_id);
        check_cache!(self.cached_services, service_id);
        None
    }

    #[allow(dead_code)]
    fn get_gatt_service_by_uuid(&mut self,
                                adapter: &mut BluetoothAdapter,
                                device_id: &str,
                                service_uuid: &str)
                                -> Option<BluetoothGATTService> {
        for service in self.cached_services.values() {
            if service.get_uuid().unwrap_or("".to_owned()) == service_uuid {
                return Some(service.clone());
            }
        }
        // Update cache
        let services = self.get_gatt_services(adapter, device_id);
        for service in services {
            if service.get_uuid().unwrap_or("".to_owned()) == service_uuid {
                return Some(service.clone());
            }
        }
        None
    }

    // Characteristic

    fn get_gatt_characteristics(&mut self,
                                adapter: &mut BluetoothAdapter,
                                service_id: &str)
                                -> Vec<BluetoothGATTCharacteristic> {
        let characteristics = match self.get_gatt_service(adapter, service_id) {
            Some(s) => s.get_gatt_characteristics().unwrap_or(vec!()),
            None => vec!(),
        };

        for characteristic in &characteristics {
            self.cached_characteristics.insert(characteristic.get_object_path(), characteristic.clone());
            self.characteristic_to_service.insert(characteristic.get_object_path(), service_id.to_owned());
        }
        characteristics
    }

    fn get_gatt_characteristic(&mut self,
                               adapter: &mut BluetoothAdapter,
                               characteristic_id: &str)
                               -> Option<&BluetoothGATTCharacteristic> {
        check_cache!(self.cached_characteristics, characteristic_id);
        let service_id = match self.characteristic_to_service.get_mut(characteristic_id) {
            Some(s) => s.clone(),
            None => return None,
        };
        // Update cache
        self.get_gatt_characteristics(adapter, &service_id);
        check_cache!(self.cached_characteristics, characteristic_id);
        None
    }

    #[allow(dead_code)]
    fn get_gatt_characteristic_by_uuid(&mut self,
                                       adapter: &mut BluetoothAdapter,
                                       service_id: &str,
                                       characteristic_uuid: &str)
                                       -> Option<BluetoothGATTCharacteristic> {
        for characteristic in self.cached_characteristics.values() {
            if characteristic.get_uuid().unwrap_or("".to_owned()) == characteristic_uuid {
                return Some(characteristic.clone());
            }
        }
        // Update cache
        let characteristics = self.get_gatt_characteristics(adapter, service_id);
        for characteristic in characteristics {
            if characteristic.get_uuid().unwrap_or("".to_owned()) == characteristic_uuid {
                return Some(characteristic.clone());
            }
        }
        None
    }

    fn get_characteristic_properties(&self, characteristic: &BluetoothGATTCharacteristic) -> [bool; 9] {
        let mut props = [false; 9];
        let flags = characteristic.get_flags().unwrap_or(vec!());
        for flag in flags {
            match flag.as_ref() {
                "broadcast" => props[0] = true,
                "read" => props[1] = true,
                "write_without_response" => props[2] = true,
                "write" => props[3] = true,
                "notify" => props[4] = true,
                "indicate" => props[5] = true,
                "authenticated_signed_writes" => props[6] = true,
                "reliable_write" => props[7] = true,
                "writable_auxiliaries" => props[8] = true,
                _ => (),
            }
        }
        props
    }

    // Descriptor

    fn get_gatt_descriptors(&mut self,
                            adapter: &mut BluetoothAdapter,
                            characteristic_id: &str)
                            -> Vec<BluetoothGATTDescriptor> {
        let descriptors = match self.get_gatt_characteristic(adapter, characteristic_id) {
            Some(c) => c.get_gatt_descriptors().unwrap_or(vec!()),
            None => vec!(),
        };

        for descriptor in &descriptors {
            self.cached_descriptors.insert(descriptor.get_object_path(), descriptor.clone());
            self.descriptor_to_characteristic.insert(descriptor.get_object_path(), characteristic_id.to_owned());
        }
        descriptors
    }

    fn get_gatt_descriptor(&mut self,
                           adapter: &mut BluetoothAdapter,
                           descriptor_id: &str)
                           -> Option<&BluetoothGATTDescriptor> {
        check_cache!(self.cached_descriptors, descriptor_id);
        let characteristic_id = match self.descriptor_to_characteristic.get_mut(descriptor_id) {
            Some(c) => c.clone(),
            None => return None,
        };
        // Update cache
        self.get_gatt_descriptors(adapter, &characteristic_id);
        check_cache!(self.cached_descriptors, descriptor_id);
        None
    }

    #[allow(dead_code)]
    fn get_gatt_descriptor_by_uuid(&mut self,
                                   adapter: &mut BluetoothAdapter,
                                   characteristic_id: &str,
                                   descriptor_uuid: &str)
                                   -> Option<BluetoothGATTDescriptor> {
        for descriptor in self.cached_descriptors.values() {
            if descriptor.get_uuid().unwrap_or("".to_owned()) == descriptor_uuid {
                return Some(descriptor.clone());
            }
        }
        // Update cache
        let descriptors = self.get_gatt_descriptors(adapter, characteristic_id);
        for descriptor in descriptors {
            if descriptor.get_uuid().unwrap_or("".to_owned()) == descriptor_uuid {
                return Some(descriptor.clone());
            }
        }
        None
    }

    // Methods

    fn request_device(&mut self, sender: IpcSender<BluetoothObjectMsg>) {
        let mut adapter = match self.get_adapter() {
            Some(a) => a,
            None => send_error!(sender, "No adapter found"),
        };
        let devices = self.get_devices(&mut adapter);
        if devices.is_empty() {
            send_error!(sender, "No device found");
        }

        //TODO select the proper device
        let device = &devices[0];

        let message = BluetoothObjectMsg::BluetoothDevice {
            id: device.get_address().unwrap_or("".to_owned()),
            name: device.get_name().ok(),
            device_class: device.get_class().ok(),
            vendor_id_source: device.get_vendor_id_source().ok(),
            vendor_id: device.get_vendor_id().ok(),
            product_id: device.get_product_id().ok(),
            product_version: device.get_device_id().ok(),
            appearance: device.get_appearance().ok(),
            tx_power: match device.get_tx_power() {
                Ok(p) => Some(p as i8),
                Err(_) => None,
            },
            rssi: match device.get_rssi() {
                Ok(p) => Some(p as i8),
                Err(_) => None,
            }
        };
        sender.send(message).unwrap();
    }

    pub fn gatt_server_connect(&mut self, device_id: String, sender: IpcSender<BluetoothObjectMsg>) {
        let mut adapter = match self.get_adapter() {
            Some(a) => a,
            None => send_error!(sender, "No adapter found"),
        };

        let connected = match self.get_device(&mut adapter, &device_id) {
            Some(d) => {
                if d.is_connected().unwrap_or(false) {
                    true
                } else {
                    !d.connect().is_err()
                }
            }
            None => send_error!(sender, "No device found"),
        };

        let message = BluetoothObjectMsg::BluetoothServer {
            connected: connected
        };
        sender.send(message).unwrap();
    }

    pub fn gatt_server_disconnect(&mut self, device_id: String, sender: IpcSender<BluetoothObjectMsg>) {
        let mut adapter = match self.get_adapter() {
            Some(a) => a,
            None => send_error!(sender, "No adapter found"),
        };

        let connected = match self.get_device(&mut adapter, &device_id) {
            Some(d) => {
                if d.is_connected().unwrap_or(false) {
                    d.disconnect().is_err()
                } else {
                    false
                }
            }
            None => send_error!(sender, "No device found"),
        };

        let message = BluetoothObjectMsg::BluetoothServer {
            connected: connected
        };
        sender.send(message).unwrap();
    }

    pub fn get_primary_service(&mut self, device_id: String, sender: IpcSender<BluetoothObjectMsg>) {
        let mut adapter = match self.get_adapter() {
            Some(a) => a,
            None => send_error!(sender, "No adapter found"),
        };

        let services = self.get_gatt_services(&mut adapter, &device_id);
        if services.is_empty() {
            send_error!(sender, "No service found");
        }

        for service in services {
            if service.is_primary().unwrap_or(false) {
                let message = BluetoothObjectMsg::BluetoothService {
                    uuid: service.get_uuid().unwrap_or("".to_owned()),
                    is_primary: true,
                    instance_id: service.get_object_path()
                };
                sender.send(message).unwrap();
                return;
            }
        }

        send_error!(sender, "No primary service found");
    }

    pub fn get_characteristic(&mut self, service_id: String, sender: IpcSender<BluetoothObjectMsg>) {
        let mut adapter = match self.get_adapter() {
            Some(a) => a,
            None => send_error!(sender, "No adapter found"),
        };

        let characteristics = self.get_gatt_characteristics(&mut adapter, &service_id);
        if characteristics.is_empty() {
            send_error!(sender, "No characteristic found");
        }

        let characteristic = &characteristics[0];
        let properties = self.get_characteristic_properties(&characteristic);
        let message = BluetoothObjectMsg::BluetoothCharacteristic {
            uuid: characteristic.get_uuid().unwrap_or("".to_owned()),
            instance_id: characteristic.get_object_path(),
            broadcast: properties[0],
            read: properties[1],
            write_without_response: properties[2],
            write: properties[3],
            notify: properties[4],
            indicate: properties[5],
            authenticated_signed_writes: properties[6],
            reliable_write: properties[7],
            writable_auxiliaries: properties[8]
        };
        sender.send(message).unwrap();
    }

    pub fn get_descriptor(&mut self, characteristic_id: String, sender: IpcSender<BluetoothObjectMsg>) {
        let mut adapter = match self.get_adapter() {
            Some(a) => a,
            None => send_error!(sender, "No adapter found"),
        };

        let descriptors = self.get_gatt_descriptors(&mut adapter, &characteristic_id);
        if descriptors.is_empty() {
            send_error!(sender, "No descriptor found");
        }

        let descriptor = &descriptors[0];
        let message = BluetoothObjectMsg::BluetoothDescriptor {
            uuid: descriptor.get_uuid().unwrap_or("".to_owned()),
            instance_id: descriptor.get_object_path(),
        };
        sender.send(message).unwrap();
    }

    pub fn read_value(&mut self, id: String, sender: IpcSender<BluetoothObjectMsg>) {
        let mut adapter = match self.get_adapter() {
            Some(a) => a,
            None => send_error!(sender, "No adapter found"),
        };
        let mut value = match self.get_gatt_characteristic(&mut adapter, &id) {
            Some(c) => Some(c.read_value().unwrap_or(vec!())),
            None => None,
        };
        if value.is_none() {
            value = match self.get_gatt_descriptor(&mut adapter, &id) {
                Some(d) => Some(d.read_value().unwrap_or(vec!())),
                None => None,
            };
        }

        let message = match value {
            Some(v) => BluetoothObjectMsg::BluetoothReadValue { value: v },
            None => send_error!(sender, "No characteristic or descriptor found with that id"),
        };

        sender.send(message).unwrap();
    }

    pub fn write_value(&mut self, id: String, value: Vec<u8>, sender: IpcSender<BluetoothObjectMsg>) {
        let mut adapter = match self.get_adapter() {
            Some(a) => a,
            None => send_error!(sender, "No adapter found"),
        };
        let mut result = match self.get_gatt_characteristic(&mut adapter, &id) {
            Some(c) => Some(c.write_value(value.clone())),
            None => None,
        };
        if result.is_none() {
            result = match self.get_gatt_descriptor(&mut adapter, &id) {
                Some(d) => Some(d.write_value(value.clone())),
                None => None,
            };
        }

        let message = match result {
            Some(v) => match v {
                Ok(_) => BluetoothObjectMsg::BluetoothWriteValue,
                Err(e) => send_error!(sender, e.to_string()),
            },
            None => send_error!(sender, "No characteristic or descriptor found with that id"),
        };

        sender.send(message).unwrap();
    }
}
