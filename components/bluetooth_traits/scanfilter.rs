/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::collections::{HashMap, HashSet};
use std::slice::Iter;

// A device name can never be longer than 29 bytes. An adv packet is at most
// 31 bytes long. The length and identifier of the length field take 2 bytes.
// That leaves 29 bytes for the name.
const MAX_NAME_LENGTH: usize = 29;

#[derive(Deserialize, Serialize)]
pub struct ServiceUUIDSequence(Vec<String>);

impl ServiceUUIDSequence {
    pub fn new(vec: Vec<String>) -> ServiceUUIDSequence {
        ServiceUUIDSequence(vec)
    }

    fn get_services_set(&self) -> HashSet<String> {
        self.0.iter().map(String::clone).collect()
    }
}

type ManufacturerData = HashMap<u16, (Vec<u8>, Vec<u8>)>;
type ServiceData = HashMap<String, (Vec<u8>, Vec<u8>)>;

#[derive(Deserialize, Serialize)]
pub struct BluetoothScanfilter {
    name: Option<String>,
    name_prefix: String,
    services: ServiceUUIDSequence,
    manufacturer_data: Option<ManufacturerData>,
    service_data: Option<ServiceData>,
}

impl BluetoothScanfilter {
    pub fn new(name: Option<String>,
               name_prefix: String,
               services: Vec<String>,
               manufacturer_data: Option<ManufacturerData>,
               service_data: Option<ServiceData>)
               -> BluetoothScanfilter {
        BluetoothScanfilter {
            name: name,
            name_prefix: name_prefix,
            services: ServiceUUIDSequence::new(services),
            manufacturer_data: manufacturer_data,
            service_data: service_data,
        }
    }

    pub fn get_name(&self) -> Option<&str> {
        self.name.as_ref().map(|s| s.as_str())
    }

    pub fn get_name_prefix(&self) -> &str {
        &self.name_prefix
    }

    pub fn get_services(&self) -> &[String] {
        &self.services.0
    }

    pub fn get_manufacturer_data(&self) -> &Option<ManufacturerData> {
        &self.manufacturer_data
    }

    pub fn get_service_data(&self) -> &Option<ServiceData> {
        &self.service_data
    }

    pub fn is_empty_or_invalid(&self) -> bool {
        (self.name.is_none() &&
         self.name_prefix.is_empty() &&
         self.get_services().is_empty() &&
         self.manufacturer_data.is_none() &&
         self.service_data.is_none()) ||
        self.get_name().unwrap_or("").len() > MAX_NAME_LENGTH ||
        self.name_prefix.len() > MAX_NAME_LENGTH
    }
}

#[derive(Deserialize, Serialize)]
pub struct BluetoothScanfilterSequence(Vec<BluetoothScanfilter>);

impl BluetoothScanfilterSequence {
    pub fn new(vec: Vec<BluetoothScanfilter>) -> BluetoothScanfilterSequence {
        BluetoothScanfilterSequence(vec)
    }

    pub fn has_empty_or_invalid_filter(&self) -> bool {
        self.0.iter().any(BluetoothScanfilter::is_empty_or_invalid)
    }

    pub fn iter(&self) -> Iter<BluetoothScanfilter> {
        self.0.iter()
    }

    fn get_services_set(&self) -> HashSet<String> {
        self.iter().flat_map(|filter| filter.services.get_services_set()).collect()
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[derive(Deserialize, Serialize)]
pub struct RequestDeviceoptions {
    filters: BluetoothScanfilterSequence,
    optional_services: ServiceUUIDSequence,
}

impl RequestDeviceoptions {
    pub fn new(filters: BluetoothScanfilterSequence,
               services: ServiceUUIDSequence)
               -> RequestDeviceoptions {
        RequestDeviceoptions {
            filters: filters,
            optional_services: services,
        }
    }

    pub fn get_filters(&self) -> &BluetoothScanfilterSequence {
        &self.filters
    }

    pub fn get_services_set(&self) -> HashSet<String> {
        &self.filters.get_services_set() | &self.optional_services.get_services_set()
    }

    pub fn is_accepting_all_devices(&self) -> bool {
        self.filters.is_empty()
    }
}
