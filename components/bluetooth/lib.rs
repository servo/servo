/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[macro_use]
extern crate bitflags;
extern crate bluetooth_traits;
extern crate device;
extern crate ipc_channel;
extern crate rand;
#[cfg(target_os = "linux")]
extern crate tinyfiledialogs;
extern crate uuid;

pub mod test;

use bluetooth_traits::{BluetoothCharacteristicMsg, BluetoothDescriptorMsg, BluetoothServiceMsg};
use bluetooth_traits::{BluetoothDeviceMsg, BluetoothRequest, BluetoothResponse, GATTType};
use bluetooth_traits::{BluetoothError, BluetoothResponseResult, BluetoothResult};
use bluetooth_traits::blocklist::{uuid_is_blocklisted, Blocklist};
use bluetooth_traits::scanfilter::{BluetoothScanfilter, BluetoothScanfilterSequence, RequestDeviceoptions};
use device::bluetooth::{BluetoothAdapter, BluetoothDevice, BluetoothGATTCharacteristic};
use device::bluetooth::{BluetoothGATTDescriptor, BluetoothGATTService};
use ipc_channel::ipc::{self, IpcReceiver, IpcSender};
use rand::Rng;
use std::borrow::ToOwned;
use std::collections::{HashMap, HashSet};
use std::string::String;
use std::thread;
use std::time::Duration;

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

pub trait BluetoothThreadFactory {
    fn new() -> Self;
}

impl BluetoothThreadFactory for IpcSender<BluetoothRequest> {
    fn new() -> IpcSender<BluetoothRequest> {
        let (sender, receiver) = ipc::channel().unwrap();
        let adapter = BluetoothAdapter::init().ok();
        thread::Builder::new().name("BluetoothThread".to_owned()).spawn(move || {
            BluetoothManager::new(receiver, adapter).start();
        }).expect("Thread spawning failed");
        sender
    }
}

// https://webbluetoothcg.github.io/web-bluetooth/#matches-a-filter
fn matches_filter(device: &BluetoothDevice, filter: &BluetoothScanfilter) -> bool {
    if filter.is_empty_or_invalid() {
        return false;
    }

    // Step 1.
    if let Some(name) = filter.get_name() {
        if device.get_name().ok() != Some(name.to_string()) {
            return false;
        }
    }

    // Step 2.
    if !filter.get_name_prefix().is_empty() {
        if let Ok(device_name) = device.get_name() {
            if !device_name.starts_with(filter.get_name_prefix()) {
                return false;
            }
        } else {
            return false;
        }
    }

    // Step 3.
    if !filter.get_services().is_empty() {
        if let Ok(device_uuids) = device.get_uuids() {
            for service in filter.get_services() {
                if device_uuids.iter().find(|x| x == &service).is_none() {
                    return false;
                }
            }
        }
    }

    // Step 4.
    if let Some(ref manufacturer_data) = filter.get_manufacturer_data() {
        let advertised_manufacturer_data = match device.get_manufacturer_data() {
            Ok(data) => data,
            Err(_) => return false,
        };
        for (ref id, &(ref prefix, ref mask)) in manufacturer_data.iter() {
            if let Some(advertised_data) = advertised_manufacturer_data.get(id) {
                if !data_filter_matches(advertised_data, prefix, mask) {
                    return false;
                }
            } else {
                return false;
            }
        }
    }

    // Step 5.
    if let Some(ref service_data) = filter.get_service_data() {
        let advertised_service_data = match device.get_service_data() {
            Ok(data) => data,
            Err(_) => return false,
        };
        for (uuid, &(ref prefix, ref mask)) in service_data.iter() {
            if let Some(advertised_data) = advertised_service_data.get(uuid.as_str()) {
                if !data_filter_matches(advertised_data, prefix, mask) {
                    return false;
                }
            } else {
                return false;
            }
        }
    }

    // Step 6.
    true
}

// https://webbluetoothcg.github.io/web-bluetooth/#bluetoothdatafilterinit-matches
fn data_filter_matches(data: &[u8], prefix: &[u8], mask: &[u8]) -> bool {
    // Step 1-2: No need to copy the bytes here.
    // Step 3.
    if data.len() < prefix.len() {
        return false;
    }

    // Step 4.
    for ((data, mask), prefix) in data.iter().zip(mask.iter()).zip(prefix.iter()) {
        if data & mask != prefix & mask {
            return false;
        }
    }

    // Step 5.
    true
}

fn matches_filters(device: &BluetoothDevice, filters: &BluetoothScanfilterSequence) -> bool {
    if filters.has_empty_or_invalid_filter() {
        return false;
    }

    return filters.iter().any(|f| matches_filter(device, f))
}

fn is_mock_adapter(adapter: &BluetoothAdapter) -> bool {
    match adapter {
        &BluetoothAdapter::Mock(_) => true,
        _ => false,
    }
}

pub struct BluetoothManager {
    receiver: IpcReceiver<BluetoothRequest>,
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
    pub fn new(receiver: IpcReceiver<BluetoothRequest>, adapter: Option<BluetoothAdapter>) -> BluetoothManager {
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
                BluetoothRequest::RequestDevice(options, sender) => {
                    let _ = sender.send(self.request_device(options));
                },
                BluetoothRequest::GATTServerConnect(device_id, sender) => {
                    let _ = sender.send(self.gatt_server_connect(device_id));
                },
                BluetoothRequest::GATTServerDisconnect(device_id, sender) => {
                    let _ = sender.send(self.gatt_server_disconnect(device_id));
                },
                BluetoothRequest::GetGATTChildren(id, uuid, single, child_type, sender) => {
                    let _ = sender.send(self.get_gatt_children(id, uuid, single, child_type));
                },
                BluetoothRequest::ReadValue(id, sender) => {
                    let _ = sender.send(self.read_value(id));
                },
                BluetoothRequest::WriteValue(id, value, sender) => {
                    let _ = sender.send(self.write_value(id, value));
                },
                BluetoothRequest::EnableNotification(id, enable, sender) => {
                    let _ = sender.send(self.enable_notification(id, enable));
                },
                BluetoothRequest::WatchAdvertisements(id, sender) => {
                    let _ = sender.send(self.watch_advertisements(id));
                },
                BluetoothRequest::Test(data_set_name, sender) => {
                    let _ = sender.send(self.test(data_set_name));
                }
                BluetoothRequest::SetRepresentedToNull(service_ids, characteristic_ids, descriptor_ids) => {
                    self.remove_ids_from_caches(service_ids, characteristic_ids, descriptor_ids)
                }
                BluetoothRequest::IsRepresentedDeviceNull(id, sender) => {
                    let _ = sender.send(!self.device_is_cached(&id));
                }
                BluetoothRequest::Exit => {
                    break
                },
            }
        }
    }

    // Test

    fn test(&mut self, data_set_name: String) -> BluetoothResult<()> {
        self.address_to_id.clear();
        self.service_to_device.clear();
        self.characteristic_to_service.clear();
        self.descriptor_to_characteristic.clear();
        self.cached_devices.clear();
        self.cached_services.clear();
        self.cached_characteristics.clear();
        self.cached_descriptors.clear();
        self.allowed_services.clear();
        self.adapter = BluetoothAdapter::init_mock().ok();
        match test::test(self, data_set_name) {
            Ok(_) => return Ok(()),
            Err(error) => Err(BluetoothError::Type(error.description().to_owned())),
        }
    }

    fn remove_ids_from_caches(&mut self,
                              service_ids: Vec<String>,
                              characteristic_ids: Vec<String>,
                              descriptor_ids: Vec<String>) {
        for id in service_ids {
            self.cached_services.remove(&id);
            self.service_to_device.remove(&id);
        }

        for id in characteristic_ids {
            self.cached_characteristics.remove(&id);
            self.characteristic_to_service.remove(&id);
        }

        for id in descriptor_ids {
            self.cached_descriptors.remove(&id);
            self.descriptor_to_characteristic.remove(&id);
        }
    }

    // Adapter

    pub fn get_or_create_adapter(&mut self) -> Option<BluetoothAdapter> {
        let adapter_valid = self.adapter.as_ref().map_or(false, |a| a.get_address().is_ok());
        if !adapter_valid {
            self.adapter = BluetoothAdapter::init().ok();
        }

        let adapter = match self.adapter.as_ref() {
            Some(adapter) => adapter,
            None => return None,
        };

        if is_mock_adapter(adapter) && !adapter.is_present().unwrap_or(false) {
            return None;
        }

        self.adapter.clone()
    }

    fn get_adapter(&mut self) -> BluetoothResult<BluetoothAdapter> {
        match self.get_or_create_adapter() {
            Some(adapter) => {
                if !adapter.is_powered().unwrap_or(false) {
                    return Err(BluetoothError::NotFound);
                }
                return Ok(adapter);
            },
            None => return Err(BluetoothError::NotFound),
        }
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
    fn select_device(&mut self, devices: Vec<BluetoothDevice>, adapter: &BluetoothAdapter) -> Option<String> {
        if is_mock_adapter(adapter) {
            for device in devices {
                if let Ok(address) = device.get_address() {
                    return Some(address);
                }
            }
            return None;
        }

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
    fn select_device(&mut self, devices: Vec<BluetoothDevice>, _adapter: &BluetoothAdapter) -> Option<String> {
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

    fn device_from_service_id(&self, service_id: &str) -> Option<BluetoothDevice> {
        let device_id = match self.service_to_device.get(service_id) {
            Some(id) => id,
            None => return None,
        };
        match self.cached_devices.get(device_id) {
            Some(d) => Some(d.clone()),
            None => None,
        }
    }

    fn device_is_cached(&self, device_id: &str) -> bool {
        self.cached_devices.contains_key(device_id) && self.address_to_id.values().any(|v| v == device_id)
    }

    // Service

    fn get_and_cache_gatt_services(&mut self,
                                   adapter: &mut BluetoothAdapter,
                                   device_id: &str)
                                   -> Vec<BluetoothGATTService> {
        let mut services = match self.get_device(adapter, device_id) {
            Some(d) => d.get_gatt_services().unwrap_or(vec!()),
            None => vec!(),
        };

        services.retain(|s| !uuid_is_blocklisted(&s.get_uuid().unwrap_or(String::new()), Blocklist::All) &&
                            self.allowed_services
                                .get(device_id)
                                .map_or(false, |uuids| uuids.contains(&s.get_uuid().unwrap_or(String::new()))));
        for service in &services {
            self.cached_services.insert(service.get_id(), service.clone());
            self.service_to_device.insert(service.get_id(), device_id.to_owned());
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

    fn service_is_cached(&self, service_id: &str) -> bool {
        self.cached_services.contains_key(service_id) && self.service_to_device.contains_key(service_id)
    }

    // Characteristic

    fn get_and_cache_gatt_characteristics(&mut self,
                                          adapter: &mut BluetoothAdapter,
                                          service_id: &str)
                                          -> Vec<BluetoothGATTCharacteristic> {
        let mut characteristics = match self.get_gatt_service(adapter, service_id) {
            Some(s) => s.get_gatt_characteristics().unwrap_or(vec!()),
            None => vec!(),
        };

        characteristics.retain(|c| !uuid_is_blocklisted(&c.get_uuid().unwrap_or(String::new()), Blocklist::All));
        for characteristic in &characteristics {
            self.cached_characteristics.insert(characteristic.get_id(), characteristic.clone());
            self.characteristic_to_service.insert(characteristic.get_id(), service_id.to_owned());
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

    fn get_characteristic_properties(&self, characteristic: &BluetoothGATTCharacteristic) -> Flags {
        let mut props: Flags = Flags::empty();
        let flags = characteristic.get_flags().unwrap_or(vec!());
        for flag in flags {
            match flag.as_ref() {
                "broadcast" => props.insert(BROADCAST),
                "read" => props.insert(READ),
                "write-without-response" => props.insert(WRITE_WITHOUT_RESPONSE),
                "write" => props.insert(WRITE),
                "notify" => props.insert(NOTIFY),
                "indicate" => props.insert(INDICATE),
                "authenticated-signed-writes" => props.insert(AUTHENTICATED_SIGNED_WRITES),
                "reliable-write" => props.insert(RELIABLE_WRITE),
                "writable-auxiliaries" => props.insert(WRITABLE_AUXILIARIES),
                _ => (),
            }
        }
        props
    }

    fn characteristic_is_cached(&self, characteristic_id: &str) -> bool {
        self.cached_characteristics.contains_key(characteristic_id) &&
        self.characteristic_to_service.contains_key(characteristic_id)
    }

    // Descriptor

    fn get_and_cache_gatt_descriptors(&mut self,
                                      adapter: &mut BluetoothAdapter,
                                      characteristic_id: &str)
                                      -> Vec<BluetoothGATTDescriptor> {
        let mut descriptors = match self.get_gatt_characteristic(adapter, characteristic_id) {
            Some(c) => c.get_gatt_descriptors().unwrap_or(vec!()),
            None => vec!(),
        };

        descriptors.retain(|d| !uuid_is_blocklisted(&d.get_uuid().unwrap_or(String::new()), Blocklist::All));
        for descriptor in &descriptors {
            self.cached_descriptors.insert(descriptor.get_id(), descriptor.clone());
            self.descriptor_to_characteristic.insert(descriptor.get_id(), characteristic_id.to_owned());
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

    // Methods

    // https://webbluetoothcg.github.io/web-bluetooth/#request-bluetooth-devices
    fn request_device(&mut self,
                      options: RequestDeviceoptions)
                      -> BluetoothResponseResult {
        // Step 6.
        let mut adapter = try!(self.get_adapter());
        if let Ok(ref session) = adapter.create_discovery_session() {
            if session.start_discovery().is_ok() {
                if !is_mock_adapter(&adapter) {
                    thread::sleep(Duration::from_millis(DISCOVERY_TIMEOUT_MS));
                }
            }
            let _ = session.stop_discovery();
        }

        // Step 7.
        // Note: There are no requiredServiceUUIDS, we scan for all devices.
        let mut matched_devices = self.get_and_cache_devices(&mut adapter);

        // Step 8.
        if !options.is_accepting_all_devices() {
            matched_devices = matched_devices.into_iter()
                                             .filter(|d| matches_filters(d, options.get_filters()))
                                             .collect();
        }

        // Step 9.
        // TODO: After the permission API implementation
        //       https://w3c.github.io/permissions/#prompt-the-user-to-choose
        if let Some(address) = self.select_device(matched_devices, &adapter) {
            let device_id = match self.address_to_id.get(&address) {
                Some(id) => id.clone(),
                None => return Err(BluetoothError::NotFound),
            };
            let mut services = options.get_services_set();
            if let Some(services_set) = self.allowed_services.get(&device_id) {
                services = services_set | &services;
            }
            self.allowed_services.insert(device_id.clone(), services);
            if let Some(device) = self.get_device(&mut adapter, &device_id) {
                let message = BluetoothDeviceMsg {
                    id: device_id,
                    name: device.get_name().ok(),
                };
                return Ok(BluetoothResponse::RequestDevice(message));
            }
        }
        // TODO: Step 10 - 11: Implement the permission API.
        return Err(BluetoothError::NotFound);
        // Step 12: Missing, because it is optional.
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattserver-connect
    fn gatt_server_connect(&mut self, device_id: String) -> BluetoothResponseResult {
        // Step 2.
        if !self.device_is_cached(&device_id) {
            return Err(BluetoothError::Network);
        }
        let mut adapter = try!(self.get_adapter());

        // Step 5.1.1.
        match self.get_device(&mut adapter, &device_id) {
            Some(d) => {
                if d.is_connected().unwrap_or(false) {
                    return Ok(BluetoothResponse::GATTServerConnect(true));
                }
                let _ = d.connect();
                for _ in 0..MAXIMUM_TRANSACTION_TIME {
                    match d.is_connected().unwrap_or(false) {
                        true => return Ok(BluetoothResponse::GATTServerConnect(true)),
                        false => {
                            if is_mock_adapter(&adapter) {
                                break;
                            }
                            thread::sleep(Duration::from_millis(CONNECTION_TIMEOUT_MS));
                        },
                    }
                // TODO: Step 5.1.4: Use the exchange MTU procedure.
                }
                // Step 5.1.3.
                return Err(BluetoothError::Network);
            },
            None => return Err(BluetoothError::NotFound),
        }
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattserver-disconnect
    fn gatt_server_disconnect(&mut self, device_id: String) -> BluetoothResult<()> {
        let mut adapter = try!(self.get_adapter());
        match self.get_device(&mut adapter, &device_id) {
            Some(d) => {
                // Step 2.
                if !d.is_connected().unwrap_or(true) {
                    return Ok(());
                }
                let _ = d.disconnect();
                for _ in 0..MAXIMUM_TRANSACTION_TIME {
                    match d.is_connected().unwrap_or(true) {
                        true => thread::sleep(Duration::from_millis(CONNECTION_TIMEOUT_MS)),
                        false => return Ok(()),
                    }
                }
                return Err(BluetoothError::Network);
            },
            None => return Err(BluetoothError::NotFound),
        }
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#getgattchildren
    fn get_gatt_children(&mut self,
                         id: String,
                         uuid: Option<String>,
                         single: bool,
                         child_type: GATTType)
                         -> BluetoothResponseResult {
        let mut adapter = try!(self.get_adapter());
        match child_type {
            GATTType::PrimaryService => {
                // Step 5.
                if !self.device_is_cached(&id) {
                    return Err(BluetoothError::InvalidState);
                }
                // Step 6.
                if let Some(ref uuid) = uuid {
                    if !self.allowed_services.get(&id).map_or(false, |s| s.contains(uuid)) {
                        return Err(BluetoothError::Security);
                    }
                }
                let mut services = self.get_and_cache_gatt_services(&mut adapter, &id);
                if let Some(uuid) = uuid {
                    services.retain(|ref e| e.get_uuid().unwrap_or(String::new()) == uuid);
                }
                let mut services_vec = vec!();
                for service in services {
                    if service.is_primary().unwrap_or(false) {
                        if let Ok(uuid) = service.get_uuid() {
                            services_vec.push(
                                BluetoothServiceMsg {
                                    uuid: uuid,
                                    is_primary: true,
                                    instance_id: service.get_id(),
                                }
                            );
                        }
                    }
                }
                // Step 7.
                if services_vec.is_empty() {
                    return Err(BluetoothError::NotFound);
                }

                return Ok(BluetoothResponse::GetPrimaryServices(services_vec, single));
            },
            GATTType::Characteristic => {
                // Step 5.
                if !self.service_is_cached(&id) {
                    return Err(BluetoothError::InvalidState);
                }
                // Step 6.
                let mut characteristics = self.get_and_cache_gatt_characteristics(&mut adapter, &id);
                if let Some(uuid) = uuid {
                    characteristics.retain(|ref e| e.get_uuid().unwrap_or(String::new()) == uuid);
                }
                let mut characteristics_vec = vec!();
                for characteristic in characteristics {
                    if let Ok(uuid) = characteristic.get_uuid() {
                        let properties = self.get_characteristic_properties(&characteristic);
                        characteristics_vec.push(
                            BluetoothCharacteristicMsg {
                                uuid: uuid,
                                instance_id: characteristic.get_id(),
                                broadcast: properties.contains(BROADCAST),
                                read: properties.contains(READ),
                                write_without_response: properties.contains(WRITE_WITHOUT_RESPONSE),
                                write: properties.contains(WRITE),
                                notify: properties.contains(NOTIFY),
                                indicate: properties.contains(INDICATE),
                                authenticated_signed_writes: properties.contains(AUTHENTICATED_SIGNED_WRITES),
                                reliable_write: properties.contains(RELIABLE_WRITE),
                                writable_auxiliaries: properties.contains(WRITABLE_AUXILIARIES),
                            }
                        );
                    }
                }

                // Step 7.
                if characteristics_vec.is_empty() {
                    return Err(BluetoothError::NotFound);
                }

                return Ok(BluetoothResponse::GetCharacteristics(characteristics_vec, single));
            },
            GATTType::IncludedService => {
                // Step 5.
                if !self.service_is_cached(&id) {
                    return Err(BluetoothError::InvalidState);
                }
                // Step 6.
                let device = match self.device_from_service_id(&id) {
                    Some(device) => device,
                    None => return Err(BluetoothError::NotFound),
                };
                let primary_service = match self.get_gatt_service(&mut adapter, &id) {
                    Some(s) => s,
                    None => return Err(BluetoothError::NotFound),
                };
                let services = primary_service.get_includes(device).unwrap_or(vec!());
                let mut services_vec = vec!();
                for service in services {
                    if let Ok(service_uuid) = service.get_uuid() {
                        services_vec.push(
                            BluetoothServiceMsg {
                                uuid: service_uuid,
                                is_primary: service.is_primary().unwrap_or(false),
                                instance_id: service.get_id(),
                            }
                        );
                    }
                }
                if let Some(uuid) = uuid {
                    services_vec.retain(|ref s| s.uuid == uuid);
                }
                services_vec.retain(|s| !uuid_is_blocklisted(&s.uuid, Blocklist::All));

                // Step 7.
                if services_vec.is_empty() {
                    return Err(BluetoothError::NotFound);
                }

                return Ok(BluetoothResponse::GetIncludedServices(services_vec, single));
            },
            GATTType::Descriptor => {
                // Step 5.
                if !self.characteristic_is_cached(&id) {
                    return Err(BluetoothError::InvalidState);
                }
                // Step 6.
                let mut descriptors = self.get_and_cache_gatt_descriptors(&mut adapter, &id);
                if let Some(uuid) = uuid {
                    descriptors.retain(|ref e| e.get_uuid().unwrap_or(String::new()) == uuid);
                }
                let mut descriptors_vec = vec!();
                for descriptor in descriptors {
                    if let Ok(uuid) = descriptor.get_uuid() {
                        descriptors_vec.push(
                            BluetoothDescriptorMsg {
                                uuid: uuid,
                                instance_id: descriptor.get_id(),
                            }
                        );
                    }
                }

                // Step 7.
                if descriptors_vec.is_empty() {
                    return Err(BluetoothError::NotFound);
                }
                return Ok(BluetoothResponse::GetDescriptors(descriptors_vec, single));
            },
        }
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattcharacteristic-readvalue
    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattdescriptor-readvalue
    fn read_value(&mut self, id: String) -> BluetoothResponseResult {
        // (Characteristic) Step 5.2: Missing because it is optional.
        // (Descriptor)     Step 5.1: Missing because it is optional.
        let mut adapter = try!(self.get_adapter());

        // (Characteristic) Step 5.3.
        let mut value = self.get_gatt_characteristic(&mut adapter, &id)
                            .map(|c| c.read_value().unwrap_or(vec![]));

        // (Characteristic) TODO: Step 5.4: Handle all the errors returned from the read_value call.

        // (Descriptor) Step 5.2.
        if value.is_none() {
            value = self.get_gatt_descriptor(&mut adapter, &id)
                        .map(|d| d.read_value().unwrap_or(vec![]));
        }

        // (Descriptor) TODO: Step 5.3: Handle all the errors returned from the read_value call.

        match value {
            // (Characteristic) Step 5.5.4.
            // (Descriptor)     Step 5.4.3.
            Some(v) => return Ok(BluetoothResponse::ReadValue(v)),

            // (Characteristic) Step 4.
            // (Descriptor)     Step 4.
            None => return Err(BluetoothError::InvalidState),
        }
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattcharacteristic-writevalue
    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattdescriptor-writevalue
    fn write_value(&mut self, id: String, value: Vec<u8>) -> BluetoothResponseResult {
        // (Characteristic) Step 7.2: Missing because it is optional.
        // (Descriptor)     Step 7.1: Missing because it is optional.
        let mut adapter = try!(self.get_adapter());

        // (Characteristic) Step 7.3.
        let mut result = self.get_gatt_characteristic(&mut adapter, &id)
                             .map(|c| c.write_value(value.clone()));

        // (Characteristic) TODO: Step 7.4: Handle all the errors returned from the write_value call.

        // (Descriptor) Step 7.2.
        if result.is_none() {
            result = self.get_gatt_descriptor(&mut adapter, &id)
                         .map(|d| d.write_value(value.clone()));
        }

        // (Descriptor) TODO: Step 7.3: Handle all the errors returned from the write_value call.

        match result {
            Some(v) => match v {
                // (Characteristic) Step 7.5.3.
                // (Descriptor) Step 7.4.3.
                Ok(_) => return Ok(BluetoothResponse::WriteValue(value)),

                // (Characteristic) Step 7.1.
                Err(_) => return Err(BluetoothError::NotSupported),
            },

            // (Characteristic) Step 6.
            // (Descriptor)     Step 6.
            None => return Err(BluetoothError::InvalidState),
        }
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattcharacteristic-startnotifications
    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattcharacteristic-stopnotifications
    fn enable_notification(&mut self, id: String, enable: bool) -> BluetoothResponseResult {
        // (StartNotifications) Step 2 - 3.
        // (StopNotifications) Step 1 - 2.
        if !self.characteristic_is_cached(&id) {
            return Err(BluetoothError::InvalidState);
        }

        // (StartNotification) TODO: Step 7: Missing because it is optional.
        let mut adapter = try!(self.get_adapter());
        match self.get_gatt_characteristic(&mut adapter, &id) {
            Some(c) => {
                let result = match enable {
                    // (StartNotification) Step 8.
                    // TODO: Handle all the errors returned from the start_notify call.
                    true => c.start_notify(),

                    // (StopNotification) Step 4.
                    false => c.stop_notify(),
                };
                match result {
                    // (StartNotification) Step 11.
                    // (StopNotification)  Step 5.
                    Ok(_) => return Ok(BluetoothResponse::EnableNotification(())),

                    // (StartNotification) Step 4.
                    Err(_) => return Err(BluetoothError::NotSupported),
                }
            },
            // (StartNotification) Step 3.
            None => return Err(BluetoothError::InvalidState),
        }
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothdevice-watchadvertisements
    fn watch_advertisements(&mut self, _device_id: String) -> BluetoothResponseResult {
        // Step 2.
        // TODO: Implement this when supported in lower level
        return Err(BluetoothError::NotSupported);
    }
}
