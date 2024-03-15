// Copyright (c) 2017 Akos Kiss.
//
// Licensed under the BSD 3-Clause License
// <LICENSE.md or https://opensource.org/licenses/BSD-3-Clause>.
// This file may not be copied, modified, or distributed except
// according to those terms.

use std::error::Error;
use std::os::raw::c_uint;
use std::slice;
use std::sync::Arc;

use delegate::bmx;
use framework::{cb, nil, ns};
use gatt_service::BluetoothGATTService;
use objc::runtime::{Object, NO, YES};
use utils::{cbx, wait, NOT_SUPPORTED_ERROR, NO_CHARACTERISTIC_FOUND};

#[derive(Clone, Debug)]
pub struct BluetoothGATTCharacteristic {
    pub(crate) service: Arc<BluetoothGATTService>,
    pub(crate) characteristic: *mut Object,
}
// TODO: implement std::fmt::Debug and/or std::fmt::Display instead of derive?

impl BluetoothGATTCharacteristic {
    pub fn new(service: Arc<BluetoothGATTService>, uuid: String) -> BluetoothGATTCharacteristic {
        // NOTE: It can happen that there is no characteristic for the given UUID, in that case
        // self.characteristic will be nil and all methods that return a Result will return
        // Err(Box::from(NO_CHARACTERISTIC_FOUND)), while others will return some meaningless value.
        let characteristic = Self::characteristic_by_uuid(service.service, &uuid);

        if characteristic == nil {
            warn!(
                "BluetoothGATTCharacteristic::new found no characteristic for UUID {}",
                uuid
            );
        }

        BluetoothGATTCharacteristic {
            service: service.clone(),
            characteristic,
        }
    }

    fn characteristic_by_uuid(service: *mut Object, uuid: &String) -> *mut Object {
        if service != nil {
            let chars = cb::service_characteristics(service);
            for i in 0..ns::array_count(chars) {
                let c = ns::array_objectatindex(chars, i);
                if cbx::uuid_to_canonical_uuid_string(cb::attribute_uuid(c)) == *uuid {
                    return c;
                }
            }
        }
        nil
    }

    pub fn get_id(&self) -> String {
        trace!("BluetoothGATTCharacteristic::get_id");
        self.get_uuid().unwrap_or_default()
    }

    pub fn get_uuid(&self) -> Result<String, Box<dyn Error>> {
        trace!("BluetoothGATTCharacteristic::get_uuid");
        if self.characteristic == nil {
            return Err(Box::from(NO_CHARACTERISTIC_FOUND));
        }

        let uuid_string =
            cbx::uuid_to_canonical_uuid_string(cb::attribute_uuid(self.characteristic));
        debug!("BluetoothGATTCharacteristic::get_uuid -> {}", uuid_string);
        Ok(uuid_string)
    }

    pub fn get_value(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        trace!("BluetoothGATTCharacteristic::get_value");
        if self.characteristic == nil {
            return Err(Box::from(NO_CHARACTERISTIC_FOUND));
        }

        let value = cb::characteristic_value(self.characteristic);
        let length = ns::data_length(value);
        if length == 0 {
            return Ok(vec![]);
        }

        let bytes = ns::data_bytes(value);
        let v = unsafe { slice::from_raw_parts(bytes, length as usize).to_vec() };
        debug!("BluetoothGATTCharacteristic::get_value -> {:?}", v);
        Ok(v)
    }

    pub fn read_value(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        trace!("BluetoothGATTCharacteristic::read_value");
        if self.characteristic == nil {
            return Err(Box::from(NO_CHARACTERISTIC_FOUND));
        }

        let events = bmx::peripheralevents(
            self.service.device.adapter.delegate,
            self.service.device.peripheral,
        )?;
        let key = bmx::valueupdatedkey(self.characteristic);
        let t = wait::get_timestamp();

        cb::peripheral_readvalueforcharacteristic(
            self.service.device.peripheral,
            self.characteristic,
        );

        wait::wait_or_timeout(|| {
            let nsnumber = ns::dictionary_objectforkey(events, key);
            (nsnumber != nil) && (ns::number_unsignedlonglongvalue(nsnumber) >= t)
        })?;

        self.get_value()
    }

    pub fn write_value(&self, values: Vec<u8>) -> Result<(), Box<dyn Error>> {
        trace!("BluetoothGATTCharacteristic::write_value");
        if self.characteristic == nil {
            return Err(Box::from(NO_CHARACTERISTIC_FOUND));
        }

        let events = bmx::peripheralevents(
            self.service.device.adapter.delegate,
            self.service.device.peripheral,
        )?;
        let key = bmx::valuewrittenkey(self.characteristic);
        let t = wait::get_timestamp();

        cb::peripheral_writevalue_forcharacteristic(
            self.service.device.peripheral,
            ns::data(values.as_ptr(), values.len() as c_uint),
            self.characteristic,
        );

        wait::wait_or_timeout(|| {
            let nsnumber = ns::dictionary_objectforkey(events, key);
            (nsnumber != nil) && (ns::number_unsignedlonglongvalue(nsnumber) >= t)
        })?;

        Ok(())
    }

    pub fn is_notifying(&self) -> Result<bool, Box<dyn Error>> {
        trace!("BluetoothGATTCharacteristic::is_notifying");
        if self.characteristic == nil {
            return Err(Box::from(NO_CHARACTERISTIC_FOUND));
        }

        let notifying = cb::characteristic_isnotifying(self.characteristic);
        debug!("BluetoothGATTCharacteristic::is_notifying -> {}", notifying);
        Ok(notifying != NO)
    }

    pub fn start_notify(&self) -> Result<(), Box<dyn Error>> {
        trace!("BluetoothGATTCharacteristic::start_notify");
        if self.characteristic == nil {
            return Err(Box::from(NO_CHARACTERISTIC_FOUND));
        }

        cb::peripheral_setnotifyvalue_forcharacteristic(
            self.service.device.peripheral,
            YES,
            self.characteristic,
        );
        Ok(())
    }

    pub fn stop_notify(&self) -> Result<(), Box<dyn Error>> {
        trace!("BluetoothGATTCharacteristic::stop_notify");
        if self.characteristic == nil {
            return Err(Box::from(NO_CHARACTERISTIC_FOUND));
        }

        cb::peripheral_setnotifyvalue_forcharacteristic(
            self.service.device.peripheral,
            NO,
            self.characteristic,
        );
        Ok(())
    }

    pub fn get_gatt_descriptors(&self) -> Result<Vec<String>, Box<dyn Error>> {
        warn!("BluetoothGATTCharacteristic::get_gatt_descriptors");
        Err(Box::from(NOT_SUPPORTED_ERROR))
    }

    pub fn get_flags(&self) -> Result<Vec<String>, Box<dyn Error>> {
        trace!("BluetoothGATTCharacteristic::get_flags");
        if self.characteristic == nil {
            return Err(Box::from(NO_CHARACTERISTIC_FOUND));
        }

        let flags = cb::characteristic_properties(self.characteristic);
        // NOTE: It is not documented anywhere what strings to return. Strings below were
        // reverse-engineered from the sources of blurdroid.
        let mut v = vec![];
        if (flags & cb::CHARACTERISTICPROPERTY_BROADCAST) != 0 {
            v.push(String::from("broadcast"));
        }
        if (flags & cb::CHARACTERISTICPROPERTY_READ) != 0 {
            v.push(String::from("read"));
        }
        if (flags & cb::CHARACTERISTICPROPERTY_WRITEWITHOUTRESPONSE) != 0 {
            v.push(String::from("write-without-response"));
        }
        if (flags & cb::CHARACTERISTICPROPERTY_WRITE) != 0 {
            v.push(String::from("write"));
        }
        if (flags & cb::CHARACTERISTICPROPERTY_NOTIFY) != 0 {
            v.push(String::from("notify"));
        }
        if (flags & cb::CHARACTERISTICPROPERTY_INDICATE) != 0 {
            v.push(String::from("indicate"));
        }
        if (flags & cb::CHARACTERISTICPROPERTY_AUTHENTICATEDSIGNEDWRITES) != 0 {
            v.push(String::from("authenticated-signed-writes"));
        }
        debug!("BluetoothGATTCharacteristic::get_flags -> {:?}", v);
        Ok(v)
    }
}
