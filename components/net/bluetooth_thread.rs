/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use device::bluetooth::BluetoothAdapter;
use device::bluetooth::BluetoothDevice;
use device::bluetooth::BluetoothGATTCharacteristic;
use device::bluetooth::BluetoothGATTDescriptor;
use device::bluetooth::BluetoothGATTService;
use ipc_channel::ipc::{self, IpcReceiver, IpcSender};
use net_traits::bluetooth_scanfilter::{BluetoothScanfilter, BluetoothScanfilterSequence, RequestDeviceoptions};
use net_traits::bluetooth_thread::{BluetoothMethodMsg, BluetoothObjectMsg};
use std::borrow::ToOwned;
use std::collections::HashMap;
use std::string::String;
use util::thread::spawn_named;

const ADAPTER_ERROR: &'static str = "No adapter found";
const DEVICE_ERROR: &'static str = "No device found";
const DEVICE_MATCH_ERROR: &'static str = "No device found, that matches the given options";
const PRIMARY_SERVICE_ERROR: &'static str = "No primary service found";
const CHARACTERISTIC_ERROR: &'static str = "No characteristic found";
const DESCRIPTOR_ERROR: &'static str = "No descriptor found";
const VALUE_ERROR: &'static str = "No characteristic or descriptor found with that id";

bitflags! {
    flags Flags: u32 {
        const BROADCAST                   = 0b000000001,
        const READ                        = 0b000000010,
        const WRITE_WITHOUT_RESPONSE      = 0b000000100,
        const WRITE                       = 0b000001000,
        const NOTIFY                      = 0b000010000,
        const INDICATE                    = 0b000100000,
        const AUTHENTICATED_SIGNED_WRITES = 0b001000000,
        const RELIABLE_WRITE              = 0b010000000,
        const WRITABLE_AUXILIARIES        = 0b100000000,
    }
}

macro_rules! send_error(
    ($sender:expr, $error:expr) => (
        return $sender.send(BluetoothObjectMsg::Error { error: String::from($error) }).unwrap();
    );
);

macro_rules! return_if_cached(
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
        let adapter = BluetoothAdapter::init().ok();
        spawn_named("BluetoothThread".to_owned(), move || {
            BluetoothManager::new(receiver, adapter).start();
        });
        sender
    }
}

fn matches_filter(device: &BluetoothDevice, filter: &BluetoothScanfilter) -> bool {
    if filter.is_empty_or_invalid() {
        return false;
    }

    if !filter.get_name().is_empty() {
        if device.get_name().ok() != Some(filter.get_name().to_string()) {
            return false;
        }
    }

    if !filter.get_name_prefix().is_empty() {
        if let Ok(device_name) = device.get_name() {
            if !device_name.starts_with(filter.get_name_prefix()) {
                return false;
            }
        } else {
            return false;
        }
    }

    if !filter.get_services().is_empty() {
        if let Ok(device_uuids) = device.get_uuids() {
            for service in filter.get_services() {
                if device_uuids.iter().find(|x| x == &service).is_none() {
                    return false;
                }
            }
        }
    }
    return true;
}

fn matches_filters(device: &BluetoothDevice, filters: &BluetoothScanfilterSequence) -> bool {
    if filters.has_empty_or_invalid_filter() {
        return false;
    }

    return filters.iter().any(|f| matches_filter(device, f))
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
        while let Ok(msg) = self.receiver.recv() {
            match msg {
                BluetoothMethodMsg::RequestDevice(options, sender) => {
                    self.request_device(options, sender)
                },
                BluetoothMethodMsg::GATTServerConnect(device_id, sender) => {
                    self.gatt_server_connect(device_id, sender)
                },
                BluetoothMethodMsg::GATTServerDisconnect(device_id, sender) => {
                    self.gatt_server_disconnect(device_id, sender)
                },
                BluetoothMethodMsg::GetPrimaryService(device_id, uuid, sender) => {
                    self.get_primary_service(device_id, uuid, sender)
                },
                BluetoothMethodMsg::GetPrimaryServices(device_id, uuid, sender) => {
                    self.get_primary_services(device_id, uuid, sender)
                },
                BluetoothMethodMsg::GetCharacteristic(service_id, uuid, sender) => {
                    self.get_characteristic(service_id, uuid, sender)
                },
                BluetoothMethodMsg::GetCharacteristics(service_id, uuid, sender) => {
                    self.get_characteristics(service_id, uuid, sender)
                },
                BluetoothMethodMsg::GetDescriptor(characteristic_id, uuid, sender) => {
                    self.get_descriptor(characteristic_id, uuid, sender)
                },
                BluetoothMethodMsg::GetDescriptors(characteristic_id, uuid, sender) => {
                    self.get_descriptors(characteristic_id, uuid, sender)
                },
                BluetoothMethodMsg::ReadValue(id, sender) => {
                    self.read_value(id, sender)
                },
                BluetoothMethodMsg::WriteValue(id, value, sender) => {
                    self.write_value(id, value, sender)
                },
                BluetoothMethodMsg::Exit => {
                    break
                },
            }
        }
    }

    // Adapter

    fn get_or_create_adapter(&mut self) -> Option<BluetoothAdapter> {
        let adapter_valid = self.adapter.as_ref().map_or(false, |a| a.get_address().is_ok());
        if !adapter_valid {
            self.adapter = BluetoothAdapter::init().ok();
        }
        self.adapter.clone()
    }

    // Device

    fn get_and_cache_devices(&mut self, adapter: &mut BluetoothAdapter) -> Vec<BluetoothDevice> {
        let devices = adapter.get_devices().unwrap_or(vec!());
        for device in &devices {
            if let Ok(address) = device.get_address() {
                self.cached_devices.insert(address, device.clone());
            }
        }
        self.cached_devices.iter().map(|(_, d)| d.clone()).collect()
    }

    fn get_device(&mut self, adapter: &mut BluetoothAdapter, device_id: &str) -> Option<&BluetoothDevice> {
        return_if_cached!(self.cached_devices, device_id);
        self.get_and_cache_devices(adapter);
        return_if_cached!(self.cached_devices, device_id);
        None
    }

    // Service

    fn get_and_cache_gatt_services(&mut self,
                                   adapter: &mut BluetoothAdapter,
                                   device_id: &str)
                                   -> Vec<BluetoothGATTService> {
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

    fn get_gatt_service(&mut self, adapter: &mut BluetoothAdapter, service_id: &str) -> Option<&BluetoothGATTService> {
        return_if_cached!(self.cached_services, service_id);
        let device_id = match self.service_to_device.get(service_id) {
            Some(d) => d.clone(),
            None => return None,
        };
        self.get_and_cache_gatt_services(adapter, &device_id);
        return_if_cached!(self.cached_services, service_id);
        None
    }

    fn get_gatt_services_by_uuid(&mut self,
                                 adapter: &mut BluetoothAdapter,
                                 device_id: &str,
                                 service_uuid: &str)
                                 -> Vec<BluetoothGATTService> {
        let services = self.get_and_cache_gatt_services(adapter, device_id);
        services.into_iter().filter(|s| s.get_uuid().ok() == Some(service_uuid.to_string())).collect()
    }

    // Characteristic

    fn get_and_cache_gatt_characteristics(&mut self,
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
        return_if_cached!(self.cached_characteristics, characteristic_id);
        let service_id = match self.characteristic_to_service.get(characteristic_id) {
            Some(s) => s.clone(),
            None => return None,
        };
        self.get_and_cache_gatt_characteristics(adapter, &service_id);
        return_if_cached!(self.cached_characteristics, characteristic_id);
        None
    }

    fn get_gatt_characteristics_by_uuid(&mut self,
                                        adapter: &mut BluetoothAdapter,
                                        service_id: &str,
                                        characteristic_uuid: &str)
                                        -> Vec<BluetoothGATTCharacteristic> {
        let characteristics = self.get_and_cache_gatt_characteristics(adapter, service_id);
        characteristics.into_iter()
                       .filter(|c| c.get_uuid().ok() == Some(characteristic_uuid.to_string()))
                       .collect()
    }

    fn get_characteristic_properties(&self, characteristic: &BluetoothGATTCharacteristic) -> Flags {
        let mut props: Flags = Flags::empty();
        let flags = characteristic.get_flags().unwrap_or(vec!());
        for flag in flags {
            match flag.as_ref() {
                "broadcast" => props.insert(BROADCAST),
                "read" => props.insert(READ),
                "write_without_response" => props.insert(WRITE_WITHOUT_RESPONSE),
                "write" => props.insert(WRITE),
                "notify" => props.insert(NOTIFY),
                "indicate" => props.insert(INDICATE),
                "authenticated_signed_writes" => props.insert(AUTHENTICATED_SIGNED_WRITES),
                "reliable_write" => props.insert(RELIABLE_WRITE),
                "writable_auxiliaries" => props.insert(WRITABLE_AUXILIARIES),
                _ => (),
            }
        }
        props
    }

    // Descriptor

    fn get_and_cache_gatt_descriptors(&mut self,
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
        return_if_cached!(self.cached_descriptors, descriptor_id);
        let characteristic_id = match self.descriptor_to_characteristic.get(descriptor_id) {
            Some(c) => c.clone(),
            None => return None,
        };
        self.get_and_cache_gatt_descriptors(adapter, &characteristic_id);
        return_if_cached!(self.cached_descriptors, descriptor_id);
        None
    }

    fn get_gatt_descriptors_by_uuid(&mut self,
                                    adapter: &mut BluetoothAdapter,
                                    characteristic_id: &str,
                                    descriptor_uuid: &str)
                                    -> Vec<BluetoothGATTDescriptor> {
        let descriptors = self.get_and_cache_gatt_descriptors(adapter, characteristic_id);
        descriptors.into_iter()
                   .filter(|d| d.get_uuid().ok() == Some(descriptor_uuid.to_string()))
                   .collect()
    }

    // Methods

    fn request_device(&mut self, options: RequestDeviceoptions, sender: IpcSender<BluetoothObjectMsg>) {
        let mut adapter = match self.get_or_create_adapter() {
            Some(a) => a,
            None => send_error!(sender, ADAPTER_ERROR),
        };
        let devices = self.get_and_cache_devices(&mut adapter);
        if devices.is_empty() {
            send_error!(sender, DEVICE_ERROR);
        }

        let matched_devices: Vec<BluetoothDevice> = devices.into_iter()
                                                           .filter(|d| matches_filters(d, options.get_filters()))
                                                           .collect();
        for device in matched_devices {
            if let Ok(address) = device.get_address() {
                let message = BluetoothObjectMsg::BluetoothDevice {
                    id: address,
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
                    },
                };
                return sender.send(message).unwrap();
            }
        }
        send_error!(sender, DEVICE_MATCH_ERROR);
    }

    fn gatt_server_connect(&mut self, device_id: String, sender: IpcSender<BluetoothObjectMsg>) {
        let mut adapter = match self.get_or_create_adapter() {
            Some(a) => a,
            None => send_error!(sender, ADAPTER_ERROR),
        };

        let connected = match self.get_device(&mut adapter, &device_id) {
            Some(d) => {
                if d.is_connected().unwrap_or(false) {
                    true
                } else {
                    d.connect().is_ok()
                }
            },
            None => send_error!(sender, DEVICE_ERROR),
        };

        let message = BluetoothObjectMsg::BluetoothServer {
            connected: connected
        };
        sender.send(message).unwrap();
    }

    fn gatt_server_disconnect(&mut self, device_id: String, sender: IpcSender<BluetoothObjectMsg>) {
        let mut adapter = match self.get_or_create_adapter() {
            Some(a) => a,
            None => send_error!(sender, ADAPTER_ERROR),
        };

        let connected = match self.get_device(&mut adapter, &device_id) {
            Some(d) => {
                if d.is_connected().unwrap_or(false) {
                    d.disconnect().is_ok()
                } else {
                    false
                }
            },
            None => send_error!(sender, DEVICE_ERROR),
        };

        let message = BluetoothObjectMsg::BluetoothServer {
            connected: connected
        };
        sender.send(message).unwrap();
    }

    fn get_primary_service(&mut self, device_id: String, uuid: String, sender: IpcSender<BluetoothObjectMsg>) {
        let mut adapter = match self.get_or_create_adapter() {
            Some(a) => a,
            None => send_error!(sender, ADAPTER_ERROR),
        };
        let services = self.get_gatt_services_by_uuid(&mut adapter, &device_id, &uuid);
        if services.is_empty() {
            send_error!(sender, PRIMARY_SERVICE_ERROR);
        }
        for service in services {
            if service.is_primary().unwrap_or(false) {
                if let Ok(uuid) = service.get_uuid() {
                    let message = BluetoothObjectMsg::BluetoothService {
                        uuid: uuid,
                        is_primary: true,
                        instance_id: service.get_object_path(),
                    };
                    return sender.send(message).unwrap();
                }
            }
        }
        send_error!(sender, PRIMARY_SERVICE_ERROR);
    }

    fn get_primary_services(&mut self,
                            device_id: String,
                            uuid: Option<String>,
                            sender: IpcSender<BluetoothObjectMsg>) {
        let mut adapter = match self.get_or_create_adapter() {
            Some(a) => a,
            None => send_error!(sender, ADAPTER_ERROR),
        };
        let services = match uuid {
            Some(id) => self.get_gatt_services_by_uuid(&mut adapter, &device_id, &id),
            None => self.get_and_cache_gatt_services(&mut adapter, &device_id),
        };
        if services.is_empty() {
            send_error!(sender, PRIMARY_SERVICE_ERROR);
        }
        let mut services_vec = vec!();
        for service in services {
            if service.is_primary().unwrap_or(false) {
                if let Ok(uuid) = service.get_uuid() {
                    services_vec.push(BluetoothObjectMsg::BluetoothService {
                        uuid: uuid,
                        is_primary: true,
                        instance_id: service.get_object_path(),
                    });
                }
            }
        }
        if services_vec.is_empty() {
            send_error!(sender, PRIMARY_SERVICE_ERROR);
        }
        let message = BluetoothObjectMsg::BluetoothServices { services_vec: services_vec };
        sender.send(message).unwrap();
    }

    fn get_characteristic(&mut self, service_id: String, uuid: String, sender: IpcSender<BluetoothObjectMsg>) {
        let mut adapter = match self.get_or_create_adapter() {
            Some(a) => a,
            None => send_error!(sender, ADAPTER_ERROR),
        };
        let characteristics = self.get_gatt_characteristics_by_uuid(&mut adapter, &service_id, &uuid);
        if characteristics.is_empty() {
            send_error!(sender, CHARACTERISTIC_ERROR);
        }
        for characteristic in characteristics {
            if let Ok(uuid) = characteristic.get_uuid() {
                let properties = self.get_characteristic_properties(&characteristic);
                let message = BluetoothObjectMsg::BluetoothCharacteristic {
                    uuid: uuid,
                    instance_id: characteristic.get_object_path(),
                    broadcast: properties.contains(BROADCAST),
                    read: properties.contains(READ),
                    write_without_response: properties.contains(WRITE_WITHOUT_RESPONSE),
                    write: properties.contains(WRITE),
                    notify: properties.contains(NOTIFY),
                    indicate: properties.contains(INDICATE),
                    authenticated_signed_writes: properties.contains(AUTHENTICATED_SIGNED_WRITES),
                    reliable_write: properties.contains(RELIABLE_WRITE),
                    writable_auxiliaries: properties.contains(WRITABLE_AUXILIARIES),
                };
                return sender.send(message).unwrap();
            }
        }
        send_error!(sender, CHARACTERISTIC_ERROR);
    }

    fn get_characteristics(&mut self,
                           service_id: String,
                           uuid: Option<String>,
                           sender: IpcSender<BluetoothObjectMsg>) {
        let mut adapter = match self.get_or_create_adapter() {
            Some(a) => a,
            None => send_error!(sender, ADAPTER_ERROR),
        };
        let characteristics = match uuid {
            Some(id) => self.get_gatt_characteristics_by_uuid(&mut adapter, &service_id, &id),
            None => self.get_and_cache_gatt_characteristics(&mut adapter, &service_id),
        };
        if characteristics.is_empty() {
            send_error!(sender, CHARACTERISTIC_ERROR);
        }
        let mut characteristics_vec = vec!();
        for characteristic in characteristics {
            if let Ok(uuid) = characteristic.get_uuid() {
                let properties = self.get_characteristic_properties(&characteristic);
                characteristics_vec.push(BluetoothObjectMsg::BluetoothCharacteristic {
                    uuid: uuid,
                    instance_id: characteristic.get_object_path(),
                    broadcast: properties.contains(BROADCAST),
                    read: properties.contains(READ),
                    write_without_response: properties.contains(WRITE_WITHOUT_RESPONSE),
                    write: properties.contains(WRITE),
                    notify: properties.contains(NOTIFY),
                    indicate: properties.contains(INDICATE),
                    authenticated_signed_writes: properties.contains(AUTHENTICATED_SIGNED_WRITES),
                    reliable_write: properties.contains(RELIABLE_WRITE),
                    writable_auxiliaries: properties.contains(WRITABLE_AUXILIARIES),
                });
            }
        }
        if characteristics_vec.is_empty() {
            send_error!(sender, CHARACTERISTIC_ERROR);
        }
        let message = BluetoothObjectMsg::BluetoothCharacteristics { characteristics_vec: characteristics_vec };
        sender.send(message).unwrap();
    }

    fn get_descriptor(&mut self, characteristic_id: String, uuid: String, sender: IpcSender<BluetoothObjectMsg>) {
        let mut adapter = match self.get_or_create_adapter() {
            Some(a) => a,
            None => send_error!(sender, ADAPTER_ERROR),
        };
        let descriptors = self.get_gatt_descriptors_by_uuid(&mut adapter, &characteristic_id, &uuid);
        if descriptors.is_empty() {
            send_error!(sender, DESCRIPTOR_ERROR);
        }
        for descriptor in descriptors {
            if let Ok(uuid) = descriptor.get_uuid() {
                let message = BluetoothObjectMsg::BluetoothDescriptor {
                    uuid: uuid,
                    instance_id: descriptor.get_object_path(),
                };
                return sender.send(message).unwrap();
            }
        }
        send_error!(sender, DESCRIPTOR_ERROR);
    }

    fn get_descriptors(&mut self,
                       characteristic_id: String,
                       uuid: Option<String>,
                       sender: IpcSender<BluetoothObjectMsg>) {
        let mut adapter = match self.get_or_create_adapter() {
            Some(a) => a,
            None => send_error!(sender, ADAPTER_ERROR),
        };
        let descriptors = match uuid {
            Some(id) => self.get_gatt_descriptors_by_uuid(&mut adapter, &characteristic_id, &id),
            None => self.get_and_cache_gatt_descriptors(&mut adapter, &characteristic_id),
        };
        if descriptors.is_empty() {
            send_error!(sender, DESCRIPTOR_ERROR);
        }
        let mut descriptors_vec = vec!();
        for descriptor in descriptors {
            if let Ok(uuid) = descriptor.get_uuid() {
                descriptors_vec.push(BluetoothObjectMsg::BluetoothDescriptor {
                    uuid: uuid,
                    instance_id: descriptor.get_object_path(),
                });
            }
        }
        if descriptors_vec.is_empty() {
            send_error!(sender, DESCRIPTOR_ERROR);
        }
        let message = BluetoothObjectMsg::BluetoothDescriptors { descriptors_vec: descriptors_vec };
        sender.send(message).unwrap();
    }

    fn read_value(&mut self, id: String, sender: IpcSender<BluetoothObjectMsg>) {
        let mut adapter = match self.get_or_create_adapter() {
            Some(a) => a,
            None => send_error!(sender, ADAPTER_ERROR),
        };
        let mut value = self.get_gatt_characteristic(&mut adapter, &id)
                            .map(|c| c.read_value().unwrap_or(vec![]));
        if value.is_none() {
            value = self.get_gatt_descriptor(&mut adapter, &id)
                        .map(|d| d.read_value().unwrap_or(vec![]));
        }

        let message = match value {
            Some(v) => BluetoothObjectMsg::BluetoothReadValue { value: v },
            None => send_error!(sender, VALUE_ERROR),
        };

        sender.send(message).unwrap();
    }

    fn write_value(&mut self, id: String, value: Vec<u8>, sender: IpcSender<BluetoothObjectMsg>) {
        let mut adapter = match self.get_or_create_adapter() {
            Some(a) => a,
            None => send_error!(sender, ADAPTER_ERROR),
        };
        let mut result = self.get_gatt_characteristic(&mut adapter, &id)
                             .map(|c| c.write_value(value.clone()));
        if result.is_none() {
            result = self.get_gatt_descriptor(&mut adapter, &id)
                         .map(|d| d.write_value(value.clone()));
        }

        let message = match result {
            Some(v) => match v {
                Ok(_) => BluetoothObjectMsg::BluetoothWriteValue,
                Err(e) => send_error!(sender, e.to_string()),
            },
            None => send_error!(sender, VALUE_ERROR),
        };

        sender.send(message).unwrap();
    }
}
