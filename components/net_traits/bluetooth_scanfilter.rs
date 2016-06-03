/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::collections::HashSet;
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
        let mut set = HashSet::new();
        self.0.iter().map(|m| set.insert(m.clone())).collect::<Vec<_>>();
        set
    }
}

#[derive(Deserialize, Serialize)]
pub struct BluetoothScanfilter {
    name: String,
    name_prefix: String,
    services: ServiceUUIDSequence,
}

impl BluetoothScanfilter {
    pub fn new(name: String, name_prefix: String, services: Vec<String>) -> BluetoothScanfilter {
        BluetoothScanfilter {
            name: name,
            name_prefix: name_prefix,
            services: ServiceUUIDSequence::new(services),
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_name_prefix(&self) -> &str {
        &self.name_prefix
    }

    pub fn get_services(&self) -> &[String] {
        &self.services.0
    }

    pub fn is_empty_or_invalid(&self) -> bool {
        (self.name.is_empty() && self.name_prefix.is_empty() && self.get_services().is_empty()) ||
        self.name.len() > MAX_NAME_LENGTH ||
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
        self.0.is_empty() ||
        self.0.iter().any(BluetoothScanfilter::is_empty_or_invalid)
    }

    pub fn iter(&self) -> Iter<BluetoothScanfilter> {
        self.0.iter()
    }

    fn get_services_set(&self) -> HashSet<String> {
        let mut set = HashSet::new();
        for filter in self.iter() {
            set = &set | &filter.services.get_services_set();
        }
        set
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
}
