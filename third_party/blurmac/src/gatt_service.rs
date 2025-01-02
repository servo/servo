// Copyright (c) 2017 Akos Kiss.
//
// Licensed under the BSD 3-Clause License
// <LICENSE.md or https://opensource.org/licenses/BSD-3-Clause>.
// This file may not be copied, modified, or distributed except
// according to those terms.

use std::error::Error;
use std::sync::Arc;

use delegate::bmx;
use device::BluetoothDevice;
use framework::{cb, nil, ns};
use objc::runtime::Object;
use utils::{cbx, wait, NO_SERVICE_FOUND};

#[derive(Clone, Debug)]
pub struct BluetoothGATTService {
    pub(crate) device: Arc<BluetoothDevice>,
    pub(crate) service: *mut Object,
}
// TODO: implement std::fmt::Debug and/or std::fmt::Display instead of derive?

impl BluetoothGATTService {
    pub fn new(device: Arc<BluetoothDevice>, uuid: String) -> BluetoothGATTService {
        trace!("BluetoothGATTService::new");
        // NOTE: It can happen that there is no service for the given UUID, in that case
        // self.service will be nil and all methods that return a Result will return
        // Err(Box::from(NO_SERVICE_FOUND)), while others will return some meaningless value.
        let service = Self::service_by_uuid(device.peripheral, &uuid);

        if service == nil {
            warn!(
                "BluetoothGATTService::new found no service for UUID {}",
                uuid
            );
        }

        BluetoothGATTService {
            device: device.clone(),
            service,
        }
    }

    fn service_by_uuid(peripheral: *mut Object, uuid: &String) -> *mut Object {
        if peripheral != nil {
            // TODO: This function will most probably not find included services. Make it recursively
            // descend into included services if first loop did not find what it was looking for.
            let services = cb::peripheral_services(peripheral);
            for i in 0..ns::array_count(services) {
                let s = ns::array_objectatindex(services, i);
                if cbx::uuid_to_canonical_uuid_string(cb::attribute_uuid(s)) == *uuid {
                    return s;
                }
            }
        }
        nil
    }

    pub fn get_id(&self) -> String {
        trace!("BluetoothGATTService::get_id");
        self.get_uuid().unwrap_or_default()
    }

    pub fn get_uuid(&self) -> Result<String, Box<dyn Error>> {
        trace!("BluetoothGATTService::get_uuid");
        if self.service == nil {
            return Err(Box::from(NO_SERVICE_FOUND));
        }

        let uuid_string = cbx::uuid_to_canonical_uuid_string(cb::attribute_uuid(self.service));
        debug!("BluetoothGATTService::get_uuid -> {}", uuid_string);
        Ok(uuid_string)
    }

    pub fn is_primary(&self) -> Result<bool, Box<dyn Error>> {
        trace!("BluetoothGATTService::is_primary");
        if self.service == nil {
            return Err(Box::from(NO_SERVICE_FOUND));
        }

        // let primary = cb::service_isprimary(self.service);
        // debug!("BluetoothGATTService::is_primary -> {}", primary);
        // Ok(primary != NO)
        // FIXME: dirty hack. no idea why [CBService isPrimary] returns NO for a primary service.
        Ok(true)
    }

    pub fn get_includes(&self) -> Result<Vec<String>, Box<dyn Error>> {
        trace!("BluetoothGATTService::get_includes");
        if self.service == nil {
            return Err(Box::from(NO_SERVICE_FOUND));
        }

        let events = bmx::peripheralevents(self.device.adapter.delegate, self.device.peripheral)?;
        let key = bmx::includedservicesdiscoveredkey(self.service);
        wait::wait_or_timeout(|| ns::dictionary_objectforkey(events, key) != nil)?;

        let mut v = vec![];
        let includes = cb::service_includedservices(self.service);
        for i in 0..ns::array_count(includes) {
            v.push(cbx::uuid_to_canonical_uuid_string(cb::attribute_uuid(
                ns::array_objectatindex(includes, i),
            )));
        }
        Ok(v)
    }

    pub fn get_gatt_characteristics(&self) -> Result<Vec<String>, Box<dyn Error>> {
        trace!("BluetoothGATTService::get_gatt_characteristics");
        if self.service == nil {
            return Err(Box::from(NO_SERVICE_FOUND));
        }

        let events = bmx::peripheralevents(self.device.adapter.delegate, self.device.peripheral)?;
        let key = bmx::characteristicsdiscoveredkey(self.service);
        wait::wait_or_timeout(|| ns::dictionary_objectforkey(events, key) != nil)?;

        let mut v = vec![];
        let chars = cb::service_characteristics(self.service);
        for i in 0..ns::array_count(chars) {
            v.push(cbx::uuid_to_canonical_uuid_string(cb::attribute_uuid(
                ns::array_objectatindex(chars, i),
            )));
        }
        Ok(v)
    }
}
