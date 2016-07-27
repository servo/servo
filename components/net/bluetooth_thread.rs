/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use device::bluetooth::BluetoothAdapter;
use device::bluetooth::BluetoothDevice;
use device::bluetooth::BluetoothDiscoverySession;
use device::bluetooth::BluetoothGATTCharacteristic;
use device::bluetooth::BluetoothGATTDescriptor;
use device::bluetooth::BluetoothGATTService;
use ipc_channel::ipc::{self, IpcReceiver, IpcSender};
use net_traits::bluetooth_scanfilter::{BluetoothScanfilter, BluetoothScanfilterSequence, RequestDeviceoptions};
use net_traits::bluetooth_thread::{BluetoothCharacteristicMsg, BluetoothCharacteristicsMsg};
use net_traits::bluetooth_thread::{BluetoothDescriptorMsg, BluetoothDescriptorsMsg};
use net_traits::bluetooth_thread::{BluetoothDeviceMsg, BluetoothError, BluetoothMethodMsg};
use net_traits::bluetooth_thread::{BluetoothResult, BluetoothServiceMsg, BluetoothServicesMsg};
use rand::{self, Rng};
use std::borrow::ToOwned;
use std::collections::{HashMap, HashSet};
use std::string::String;
use std::thread;
use std::time::Duration;
#[cfg(target_os = "linux")]
use tinyfiledialogs;
use util::thread::spawn_named;

const ADAPTER_ERROR: &'static str = "No adapter found";

// A transaction not completed within 30 seconds shall time out. Such a transaction shall be considered to have failed.
// https://www.bluetooth.org/DocMan/handlers/DownloadDoc.ashx?doc_id=286439 (Vol. 3, page 480)
const MAXIMUM_TRANSACTION_TIME: u8 = 30;
const CONNECTION_TIMEOUT_MS: u64 = 1000;
// The discovery session needs some time to find any nearby devices
const DISCOVERY_TIMEOUT_MS: u64 = 1500;
#[cfg(target_os = "linux")]
const DIALOG_TITLE: &'static str = "Choose a device";
#[cfg(target_os = "linux")]
const DIALOG_COLUMN_ID: &'static str = "Id";
#[cfg(target_os = "linux")]
const DIALOG_COLUMN_NAME: &'static str = "Name";

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

macro_rules! return_if_cached(
    ($cache:expr, $key:expr) => (
        if $cache.contains_key($key) {
            return $cache.get($key);
        }
    );
);

macro_rules! get_adapter_or_return_error(
    ($bl_manager:expr, $sender:expr) => (
        match $bl_manager.get_or_create_adapter() {
            Some(adapter) => adapter,
            None => return drop($sender.send(Err(BluetoothError::Type(ADAPTER_ERROR.to_string())))),
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
    address_to_id: HashMap<String, String>,
    service_to_device: HashMap<String, String>,
    characteristic_to_service: HashMap<String, String>,
    descriptor_to_characteristic: HashMap<String, String>,
    cached_devices: HashMap<String, BluetoothDevice>,
    cached_services: HashMap<String, BluetoothGATTService>,
    cached_characteristics: HashMap<String, BluetoothGATTCharacteristic>,
    cached_descriptors: HashMap<String, BluetoothGATTDescriptor>,
    allowed_services: HashMap<String, HashSet<String>>,
}

impl BluetoothManager {
    pub fn new (receiver: IpcReceiver<BluetoothMethodMsg>, adapter: Option<BluetoothAdapter>) -> BluetoothManager {
        BluetoothManager {
            receiver: receiver,
            adapter: adapter,
            address_to_id: HashMap::new(),
            service_to_device: HashMap::new(),
            characteristic_to_service: HashMap::new(),
            descriptor_to_characteristic: HashMap::new(),
            cached_devices: HashMap::new(),
            cached_services: HashMap::new(),
            cached_characteristics: HashMap::new(),
            cached_descriptors: HashMap::new(),
            allowed_services: HashMap::new(),
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
                BluetoothMethodMsg::GetIncludedService(service_id, uuid, sender) => {
                    self.get_included_service(service_id, uuid, sender)
                },
                BluetoothMethodMsg::GetIncludedServices(service_id, uuid, sender) => {
                    self.get_included_services(service_id, uuid, sender)
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
                if !self.address_to_id.contains_key(&address) {
                    let generated_id = self.generate_device_id();
                    self.address_to_id.insert(address, generated_id.clone());
                    self.cached_devices.insert(generated_id.clone(), device.clone());
                    self.allowed_services.insert(generated_id, HashSet::new());
                }
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

    #[cfg(target_os = "linux")]
    fn select_device(&mut self, devices: Vec<BluetoothDevice>) -> Option<String> {
        let mut dialog_rows: Vec<String> = vec!();
        for device in devices {
            dialog_rows.extend_from_slice(&[device.get_address().unwrap_or("".to_string()),
                                            device.get_name().unwrap_or("".to_string())]);
        }
        let dialog_rows: Vec<&str> = dialog_rows.iter()
                                                .map(|s| s.as_ref())
                                                .collect();
        let dialog_rows: &[&str] = dialog_rows.as_slice();

        if let Some(device) = tinyfiledialogs::list_dialog(DIALOG_TITLE,
                                                           &[DIALOG_COLUMN_ID, DIALOG_COLUMN_NAME],
                                                           Some(dialog_rows)) {
            // The device string format will be "Address|Name". We need the first part of it.
            return device.split("|").next().map(|s| s.to_string());
        }
        None
    }

    #[cfg(not(target_os = "linux"))]
    fn select_device(&mut self, devices: Vec<BluetoothDevice>) -> Option<String> {
        for device in devices {
            if let Ok(address) = device.get_address() {
                return Some(address);
            }
        }
        None
    }

    fn generate_device_id(&mut self) -> String {
        let mut device_id;
        let mut rng = rand::thread_rng();
        loop {
            device_id = rng.gen::<u32>().to_string();
            if !self.cached_devices.contains_key(&device_id) {
                break;
            }
        }
        device_id
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

    fn request_device(&mut self,
                      options: RequestDeviceoptions,
                      sender: IpcSender<BluetoothResult<BluetoothDeviceMsg>>) {
        let mut adapter = get_adapter_or_return_error!(self, sender);
        if let Some(ref session) = BluetoothDiscoverySession::create_session(adapter.get_object_path()).ok() {
            if session.start_discovery().is_ok() {
                thread::sleep(Duration::from_millis(DISCOVERY_TIMEOUT_MS));
            }
            let _ = session.stop_discovery();
        }
        let devices = self.get_and_cache_devices(&mut adapter);
        let matched_devices: Vec<BluetoothDevice> = devices.into_iter()
                                                           .filter(|d| matches_filters(d, options.get_filters()))
                                                           .collect();
        if let Some(address) = self.select_device(matched_devices) {
            let device_id = match self.address_to_id.get(&address) {
                Some(id) => id.clone(),
                None => return drop(sender.send(Err(BluetoothError::NotFound))),
            };
            let mut services = options.get_services_set();
            if let Some(services_set) = self.allowed_services.get(&device_id) {
                services = services_set | &services;
            }
            self.allowed_services.insert(device_id.clone(), services);
            if let Some(device) = self.get_device(&mut adapter, &device_id) {
                let message = Ok(BluetoothDeviceMsg {
                                     id: device_id,
                                     name: device.get_name().ok(),
                                     appearance: device.get_appearance().ok(),
                                     tx_power: device.get_tx_power().ok().map(|p| p as i8),
                                     rssi: device.get_rssi().ok().map(|p| p as i8),
                                 });
                return drop(sender.send(message));
            }
        }
        return drop(sender.send(Err(BluetoothError::NotFound)));
    }

    fn gatt_server_connect(&mut self, device_id: String, sender: IpcSender<BluetoothResult<bool>>) {
        let mut adapter = get_adapter_or_return_error!(self, sender);

        match self.get_device(&mut adapter, &device_id) {
            Some(d) => {
                if d.is_connected().unwrap_or(false) {
                    return drop(sender.send(Ok(true)));
                }
                let _ = d.connect();
                for _ in 0..MAXIMUM_TRANSACTION_TIME {
                    match d.is_connected().unwrap_or(false) {
                        true => return drop(sender.send(Ok(true))),
                        false => thread::sleep(Duration::from_millis(CONNECTION_TIMEOUT_MS)),
                    }
                }
                return drop(sender.send(Err(BluetoothError::Network)));
            },
            None => return drop(sender.send(Err(BluetoothError::NotFound))),
        }
    }

    fn gatt_server_disconnect(&mut self, device_id: String, sender: IpcSender<BluetoothResult<bool>>) {
        let mut adapter = get_adapter_or_return_error!(self, sender);

        match self.get_device(&mut adapter, &device_id) {
            Some(d) => {
                if !d.is_connected().unwrap_or(true) {
                    return drop(sender.send(Ok(false)));
                }
                let _ = d.disconnect();
                for _ in 0..MAXIMUM_TRANSACTION_TIME {
                    match d.is_connected().unwrap_or(true) {
                        true => thread::sleep(Duration::from_millis(CONNECTION_TIMEOUT_MS)),
                        false => return drop(sender.send(Ok(false))),
                    }
                }
                return drop(sender.send(Err(BluetoothError::Network)));
            },
            None => return drop(sender.send(Err(BluetoothError::NotFound))),
        }
    }

    fn get_primary_service(&mut self,
                           device_id: String,
                           uuid: String,
                           sender: IpcSender<BluetoothResult<BluetoothServiceMsg>>) {
        let mut adapter = get_adapter_or_return_error!(self, sender);
        if !self.allowed_services.get(&device_id).map_or(false, |s| s.contains(&uuid)) {
            return drop(sender.send(Err(BluetoothError::Security)));
        }
        let services = self.get_gatt_services_by_uuid(&mut adapter, &device_id, &uuid);
        if services.is_empty() {
            return drop(sender.send(Err(BluetoothError::NotFound)));
        }
        for service in services {
            if service.is_primary().unwrap_or(false) {
                if let Ok(uuid) = service.get_uuid() {
                    return drop(sender.send(Ok(BluetoothServiceMsg {
                                                   uuid: uuid,
                                                   is_primary: true,
                                                   instance_id: service.get_object_path(),
                                               })));
                }
            }
        }
        return drop(sender.send(Err(BluetoothError::NotFound)));
    }

    fn get_primary_services(&mut self,
                            device_id: String,
                            uuid: Option<String>,
                            sender: IpcSender<BluetoothResult<BluetoothServicesMsg>>) {
        let mut adapter = get_adapter_or_return_error!(self, sender);
        let services = match uuid {
            Some(ref id) => {
                if !self.allowed_services.get(&device_id).map_or(false, |s| s.contains(id)) {
                    return drop(sender.send(Err(BluetoothError::Security)))
                }
                self.get_gatt_services_by_uuid(&mut adapter, &device_id, id)
            },
            None => self.get_and_cache_gatt_services(&mut adapter, &device_id),
        };
        if services.is_empty() {
            return drop(sender.send(Err(BluetoothError::NotFound)));
        }
        let mut services_vec = vec!();
        for service in services {
            if service.is_primary().unwrap_or(false) {
                if let Ok(uuid) = service.get_uuid() {
                    services_vec.push(BluetoothServiceMsg {
                                          uuid: uuid,
                                          is_primary: true,
                                          instance_id: service.get_object_path(),
                                      });
                }
            }
        }
        if services_vec.is_empty() {
            return drop(sender.send(Err(BluetoothError::NotFound)));
        }

        let _ = sender.send(Ok(services_vec));
    }

    fn get_included_service(&mut self,
                            service_id: String,
                            uuid: String,
                            sender: IpcSender<BluetoothResult<BluetoothServiceMsg>>) {
        let mut adapter = match self.get_or_create_adapter() {
            Some(a) => a,
            None => return drop(sender.send(Err(BluetoothError::Type(ADAPTER_ERROR.to_string())))),
        };
        let primary_service = match self.get_gatt_service(&mut adapter, &service_id) {
            Some(s) => s,
            None => return drop(sender.send(Err(BluetoothError::NotFound))),
        };
        let services = primary_service.get_includes().unwrap_or(vec!());
        for service in services {
            if let Ok(service_uuid) = service.get_uuid() {
                if uuid == service_uuid {
                    return drop(sender.send(Ok(BluetoothServiceMsg {
                                                   uuid: uuid,
                                                   is_primary: service.is_primary().unwrap_or(false),
                                                   instance_id: service.get_object_path(),
                                               })));
                }
            }
        }
        return drop(sender.send(Err(BluetoothError::NotFound)));
    }

    fn get_included_services(&mut self,
                             service_id: String,
                             uuid: Option<String>,
                             sender: IpcSender<BluetoothResult<BluetoothServicesMsg>>) {
        let mut adapter = match self.get_or_create_adapter() {
            Some(a) => a,
            None => return drop(sender.send(Err(BluetoothError::Type(ADAPTER_ERROR.to_string())))),
        };
        let primary_service = match self.get_gatt_service(&mut adapter, &service_id) {
            Some(s) => s,
            None => return drop(sender.send(Err(BluetoothError::NotFound))),
        };
        let services = primary_service.get_includes().unwrap_or(vec!());
        let mut services_vec = vec!();
        for service in services {
            if let Ok(service_uuid) = service.get_uuid() {
                services_vec.push(BluetoothServiceMsg {
                                      uuid: service_uuid,
                                      is_primary: service.is_primary().unwrap_or(false),
                                      instance_id: service.get_object_path(),
                                  });
            }
        }
        if let Some(uuid) = uuid {
            services_vec.retain(|ref s| s.uuid == uuid);
        }
        if services_vec.is_empty() {
            return drop(sender.send(Err(BluetoothError::NotFound)));
        }

        let _ = sender.send(Ok(services_vec));
    }

    fn get_characteristic(&mut self,
                          service_id: String,
                          uuid: String,
                          sender: IpcSender<BluetoothResult<BluetoothCharacteristicMsg>>) {
        let mut adapter = get_adapter_or_return_error!(self, sender);
        let characteristics = self.get_gatt_characteristics_by_uuid(&mut adapter, &service_id, &uuid);
        if characteristics.is_empty() {
            return drop(sender.send(Err(BluetoothError::NotFound)));
        }
        for characteristic in characteristics {
            if let Ok(uuid) = characteristic.get_uuid() {
                let properties = self.get_characteristic_properties(&characteristic);
                let message = Ok(BluetoothCharacteristicMsg {
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
                return drop(sender.send(message));
            }
        }
        return drop(sender.send(Err(BluetoothError::NotFound)));
    }

    fn get_characteristics(&mut self,
                           service_id: String,
                           uuid: Option<String>,
                           sender: IpcSender<BluetoothResult<BluetoothCharacteristicsMsg>>) {
        let mut adapter = get_adapter_or_return_error!(self, sender);
        let characteristics = match uuid {
            Some(id) => self.get_gatt_characteristics_by_uuid(&mut adapter, &service_id, &id),
            None => self.get_and_cache_gatt_characteristics(&mut adapter, &service_id),
        };
        if characteristics.is_empty() {
            return drop(sender.send(Err(BluetoothError::NotFound)));
        }
        let mut characteristics_vec = vec!();
        for characteristic in characteristics {
            if let Ok(uuid) = characteristic.get_uuid() {
                let properties = self.get_characteristic_properties(&characteristic);
                characteristics_vec.push(
                                BluetoothCharacteristicMsg {
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
            return drop(sender.send(Err(BluetoothError::NotFound)));
        }

        let _ = sender.send(Ok(characteristics_vec));
    }

    fn get_descriptor(&mut self,
                      characteristic_id: String,
                      uuid: String,
                      sender: IpcSender<BluetoothResult<BluetoothDescriptorMsg>>) {
        let mut adapter = get_adapter_or_return_error!(self, sender);
        let descriptors = self.get_gatt_descriptors_by_uuid(&mut adapter, &characteristic_id, &uuid);
        if descriptors.is_empty() {
            return drop(sender.send(Err(BluetoothError::NotFound)));
        }
        for descriptor in descriptors {
            if let Ok(uuid) = descriptor.get_uuid() {
                return drop(sender.send(Ok(BluetoothDescriptorMsg {
                                               uuid: uuid,
                                               instance_id: descriptor.get_object_path(),
                                           })));
            }
        }
        return drop(sender.send(Err(BluetoothError::NotFound)));
    }

    fn get_descriptors(&mut self,
                       characteristic_id: String,
                       uuid: Option<String>,
                       sender: IpcSender<BluetoothResult<BluetoothDescriptorsMsg>>) {
        let mut adapter = get_adapter_or_return_error!(self, sender);
        let descriptors = match uuid {
            Some(id) => self.get_gatt_descriptors_by_uuid(&mut adapter, &characteristic_id, &id),
            None => self.get_and_cache_gatt_descriptors(&mut adapter, &characteristic_id),
        };
        if descriptors.is_empty() {
            return drop(sender.send(Err(BluetoothError::NotFound)));
        }
        let mut descriptors_vec = vec!();
        for descriptor in descriptors {
            if let Ok(uuid) = descriptor.get_uuid() {
                descriptors_vec.push(BluetoothDescriptorMsg {
                                         uuid: uuid,
                                         instance_id: descriptor.get_object_path(),
                                     });
            }
        }
        if descriptors_vec.is_empty() {
            return drop(sender.send(Err(BluetoothError::NotFound)));
        }
        let _ = sender.send(Ok(descriptors_vec));
    }

    fn read_value(&mut self, id: String, sender: IpcSender<BluetoothResult<Vec<u8>>>) {
        let mut adapter = get_adapter_or_return_error!(self, sender);
        let mut value = self.get_gatt_characteristic(&mut adapter, &id)
                            .map(|c| c.read_value().unwrap_or(vec![]));
        if value.is_none() {
            value = self.get_gatt_descriptor(&mut adapter, &id)
                        .map(|d| d.read_value().unwrap_or(vec![]));
        }
        let _ = sender.send(value.ok_or(BluetoothError::NotSupported));
    }

    fn write_value(&mut self, id: String, value: Vec<u8>, sender: IpcSender<BluetoothResult<bool>>) {
        let mut adapter = get_adapter_or_return_error!(self, sender);
        let mut result = self.get_gatt_characteristic(&mut adapter, &id)
                             .map(|c| c.write_value(value.clone()));
        if result.is_none() {
            result = self.get_gatt_descriptor(&mut adapter, &id)
                         .map(|d| d.write_value(value.clone()));
        }
        let message = match result {
            Some(v) => match v {
                Ok(_) => Ok(true),
                Err(_) => return drop(sender.send(Err(BluetoothError::NotSupported))),
            },
            None => return drop(sender.send(Err(BluetoothError::NotSupported))),
        };
        let _ = sender.send(message);
    }
}
