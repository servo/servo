/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use device::bluetooth::BluetoothDevice;

// A device name can never be longer than 29 bytes. An adv packet is at most
// 31 bytes long. The length and identifier of the length field take 2 bytes.
// That least 29 bytes for the name.
const MAX_NAME_LENGTH: usize = 29;

#[derive(Deserialize, Serialize)]
pub struct ServiceUUIDSequence(Vec<String>);

impl ServiceUUIDSequence {
    pub fn new(vec: Vec<String>) -> ServiceUUIDSequence {
        ServiceUUIDSequence(vec)
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[derive(Deserialize, Serialize)]
pub struct BluetoothScanfilter {
    name: String,
    name_prefix: String,
    services: ServiceUUIDSequence,
}

impl BluetoothScanfilter {
    fn is_empty_or_invalid_filter(&self) -> bool {
        self.name.is_empty() &&
        self.name_prefix.is_empty() &&
        self.services.is_empty() &&
        self.name.len() > MAX_NAME_LENGTH &&
        self.name_prefix.len() > MAX_NAME_LENGTH
    }

    pub fn new(name: String, name_prefix: String, services: Vec<String>) -> BluetoothScanfilter {
        BluetoothScanfilter {
            name: name,
            name_prefix: name_prefix,
            services: ServiceUUIDSequence::new(services),
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct BluetoothScanfilterSequence(Vec<BluetoothScanfilter>);

impl BluetoothScanfilterSequence {
    pub fn new(vec: Vec<BluetoothScanfilter>) -> BluetoothScanfilterSequence {
        BluetoothScanfilterSequence(vec)
    }
}

impl BluetoothScanfilterSequence {
    fn has_empty_or_invalid_filter(&self) -> bool {
        self.0.is_empty() &&
        self.0.iter().all(|x| !(x.is_empty_or_invalid_filter()))
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
}

pub fn matches_filter(device: &BluetoothDevice, filter: &BluetoothScanfilter) -> bool {
    if filter.is_empty_or_invalid_filter() {
        return false;
    }

     if !filter.name.is_empty() {
        if let Ok(device_name) = device.get_name() {
            if !device_name.eq(&filter.name) {
                return false;
            }
        } else {
            return false;
        }
    }

    if !filter.name_prefix.is_empty() {
        if let Ok(device_name) = device.get_name() {
            if !device_name.starts_with(&*filter.name_prefix) {
                return false;
            }
        } else {
            return false;
        }
    }

    if !filter.services.is_empty() {
        if let Ok(stringvec) = device.get_uuids() {
            for service in &filter.services.0 {
                if !stringvec.iter().any(|x| x == service) {
                    return false;
                }
            }
        }
    }
    return true;
}

pub fn matches_filters(device: &BluetoothDevice, filters: &BluetoothScanfilterSequence) -> bool {
    if filters.has_empty_or_invalid_filter() {
        return false;
    }

    for filter in &filters.0 {
        if matches_filter(device, filter) {
            return true;
        }
    }
    return false;
}
