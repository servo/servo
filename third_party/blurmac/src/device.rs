// Copyright (c) 2017 Akos Kiss.
//
// Licensed under the BSD 3-Clause License
// <LICENSE.md or https://opensource.org/licenses/BSD-3-Clause>.
// This file may not be copied, modified, or distributed except
// according to those terms.

use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;

use adapter::BluetoothAdapter;
use delegate::{bm, bmx};
use framework::{cb, nil, ns};
use objc::runtime::Object;
use utils::{cbx, nsx, wait, NOT_SUPPORTED_ERROR, NO_PERIPHERAL_FOUND};

#[derive(Clone, Debug)]
pub struct BluetoothDevice {
    pub(crate) adapter: Arc<BluetoothAdapter>,
    pub(crate) peripheral: *mut Object,
}
// TODO: implement std::fmt::Debug and/or std::fmt::Display instead of derive?

impl BluetoothDevice {
    pub fn new(adapter: Arc<BluetoothAdapter>, uuid: String) -> BluetoothDevice {
        trace!("BluetoothDevice::new");
        // NOTE: It can happen that there is no peripheral for the given UUID, in that case
        // self.peripheral will be nil and all methods that return a Result will return
        // Err(Box::from(NO_PERIPHERAL_FOUND)), while others will return some meaningless value.
        let peripheral = Self::peripheral_by_uuid(adapter.delegate, &uuid);

        if peripheral == nil {
            warn!("BluetoothDevice::new found no peripheral for UUID {}", uuid);
        }

        BluetoothDevice {
            adapter: adapter.clone(),
            peripheral,
        }
    }

    fn peripheral_by_uuid(delegate: *mut Object, uuid: &String) -> *mut Object {
        let peripherals = bm::delegate_peripherals(delegate);
        let keys = ns::dictionary_allkeys(peripherals);
        for i in 0..ns::array_count(keys) {
            let uuid_nsstring = ns::array_objectatindex(keys, i);
            if nsx::string_to_string(uuid_nsstring) == *uuid {
                let data = ns::dictionary_objectforkey(peripherals, uuid_nsstring);
                return ns::dictionary_objectforkey(
                    data,
                    nsx::string_from_str(bm::PERIPHERALDATA_PERIPHERALKEY),
                );
            }
        }
        nil
    }

    pub fn get_id(&self) -> String {
        trace!("BluetoothDevice::get_id -> get_address");
        self.get_address().unwrap_or_default()
    }

    pub fn get_address(&self) -> Result<String, Box<dyn Error>> {
        trace!("BluetoothDevice::get_address");
        if self.peripheral == nil {
            return Err(Box::from(NO_PERIPHERAL_FOUND));
        }

        // NOTE: There is no better substitute for address than identifier.
        let uuid_string =
            nsx::string_to_string(ns::uuid_uuidstring(cb::peer_identifier(self.peripheral)));
        debug!("BluetoothDevice::get_address -> {}", uuid_string);
        Ok(uuid_string)
    }

    pub fn get_name(&self) -> Result<String, Box<dyn Error>> {
        trace!("BluetoothDevice::get_name");
        if self.peripheral == nil {
            return Err(Box::from(NO_PERIPHERAL_FOUND));
        }

        let name_nsstring = cb::peripheral_name(self.peripheral);
        let name = if name_nsstring != nil {
            nsx::string_to_string(name_nsstring)
        } else {
            String::from("")
        };
        debug!("BluetoothDevice::get_name -> {}", name);
        Ok(name)
    }

    pub fn get_uuids(&self) -> Result<Vec<String>, Box<dyn Error>> {
        trace!("BluetoothDevice::get_uuids");
        if self.peripheral == nil {
            return Err(Box::from(NO_PERIPHERAL_FOUND));
        }

        let data = bmx::peripheraldata(self.adapter.delegate, self.peripheral)?;
        let mut v = vec![];
        let cbuuids_nsarray =
            ns::dictionary_objectforkey(data, nsx::string_from_str(bm::PERIPHERALDATA_UUIDSKEY));
        if cbuuids_nsarray != nil {
            for i in 0..ns::array_count(cbuuids_nsarray) {
                v.push(cbx::uuid_to_canonical_uuid_string(ns::array_objectatindex(
                    cbuuids_nsarray,
                    i,
                )));
            }
        }
        debug!("BluetoothDevice::get_uuids -> {:?}", v);
        Ok(v)
    }

    pub fn connect(&self) -> Result<(), Box<dyn Error>> {
        trace!("BluetoothDevice::connect");
        if self.peripheral == nil {
            return Err(Box::from(NO_PERIPHERAL_FOUND));
        }

        cb::centralmanager_connectperipheral(self.adapter.manager, self.peripheral);
        Ok(())
    }

    pub fn disconnect(&self) -> Result<(), Box<dyn Error>> {
        trace!("BluetoothDevice::disconnect");
        if self.peripheral == nil {
            return Err(Box::from(NO_PERIPHERAL_FOUND));
        }

        cb::centralmanager_cancelperipheralconnection(self.adapter.manager, self.peripheral);
        Ok(())
    }

    pub fn is_connected(&self) -> Result<bool, Box<dyn Error>> {
        trace!("BluetoothDevice::is_connected");
        if self.peripheral == nil {
            return Err(Box::from(NO_PERIPHERAL_FOUND));
        }

        let state = cb::peripheral_state(self.peripheral);
        debug!("BluetoothDevice::is_connected -> {}", state);
        Ok(state == cb::PERIPHERALSTATE_CONNECTED)
    }

    pub fn get_gatt_services(&self) -> Result<Vec<String>, Box<dyn Error>> {
        trace!("BluetoothDevice::get_gatt_services");
        if self.peripheral == nil {
            return Err(Box::from(NO_PERIPHERAL_FOUND));
        }

        let events = bmx::peripheralevents(self.adapter.delegate, self.peripheral)?;
        let key = nsx::string_from_str(bm::PERIPHERALEVENT_SERVICESDISCOVEREDKEY);
        wait::wait_or_timeout(|| ns::dictionary_objectforkey(events, key) != nil)?;

        let mut v = vec![];
        let services = cb::peripheral_services(self.peripheral);
        for i in 0..ns::array_count(services) {
            let uuid_string = cbx::uuid_to_canonical_uuid_string(cb::attribute_uuid(
                ns::array_objectatindex(services, i),
            ));
            v.push(uuid_string);
        }
        debug!("BluetoothDevice::get_gatt_services -> {:?}", v);
        Ok(v)
    }

    // Not supported

    pub fn get_rssi(&self) -> Result<i16, Box<dyn Error>> {
        warn!("BluetoothDevice::get_rssi not supported by BlurMac");
        // TODO: Now available from peripheral data in BluetoothAdapter.
        Err(Box::from(NOT_SUPPORTED_ERROR))
    }

    pub fn get_tx_power(&self) -> Result<i16, Box<dyn Error>> {
        warn!("BluetoothDevice::get_tx_power not supported by BlurMac");
        Err(Box::from(NOT_SUPPORTED_ERROR))
    }

    pub fn get_manufacturer_data(&self) -> Result<HashMap<u16, Vec<u8>>, Box<dyn Error>> {
        warn!("BluetoothDevice::get_manufacturer_data not supported by BlurMac");
        Err(Box::from(NOT_SUPPORTED_ERROR))
    }

    pub fn get_service_data(&self) -> Result<HashMap<String, Vec<u8>>, Box<dyn Error>> {
        warn!("BluetoothDevice::get_service_data not supported by BlurMac");
        Err(Box::from(NOT_SUPPORTED_ERROR))
    }

    pub fn get_icon(&self) -> Result<String, Box<dyn Error>> {
        warn!("BluetoothDevice::get_icon not supported by BlurMac");
        Err(Box::from(NOT_SUPPORTED_ERROR))
    }

    pub fn get_class(&self) -> Result<u32, Box<dyn Error>> {
        warn!("BluetoothDevice::get_class not supported by BlurMac");
        Err(Box::from(NOT_SUPPORTED_ERROR))
    }

    pub fn get_appearance(&self) -> Result<u16, Box<dyn Error>> {
        warn!("BluetoothDevice::get_appearance not supported by BlurMac");
        Err(Box::from(NOT_SUPPORTED_ERROR))
    }

    pub fn is_paired(&self) -> Result<bool, Box<dyn Error>> {
        warn!("BluetoothDevice::is_paired not supported by BlurMac");
        Err(Box::from(NOT_SUPPORTED_ERROR))
    }

    pub fn is_trusted(&self) -> Result<bool, Box<dyn Error>> {
        warn!("BluetoothDevice::is_trusted not supported by BlurMac");
        Err(Box::from(NOT_SUPPORTED_ERROR))
    }

    pub fn is_blocked(&self) -> Result<bool, Box<dyn Error>> {
        warn!("BluetoothDevice::is_blocked not supported by BlurMac");
        Err(Box::from(NOT_SUPPORTED_ERROR))
    }

    pub fn get_alias(&self) -> Result<String, Box<dyn Error>> {
        warn!("BluetoothDevice::get_alias not supported by BlurMac");
        Err(Box::from(NOT_SUPPORTED_ERROR))
    }

    pub fn set_alias(&self, _value: String) -> Result<(), Box<dyn Error>> {
        warn!("BluetoothDevice::set_alias not supported by BlurMac");
        Err(Box::from(NOT_SUPPORTED_ERROR))
    }

    pub fn is_legacy_pairing(&self) -> Result<bool, Box<dyn Error>> {
        warn!("BluetoothDevice::is_legacy_pairing not supported by BlurMac");
        Err(Box::from(NOT_SUPPORTED_ERROR))
    }

    pub fn get_vendor_id_source(&self) -> Result<String, Box<dyn Error>> {
        warn!("BluetoothDevice::get_vendor_id_source not supported by BlurMac");
        Err(Box::from(NOT_SUPPORTED_ERROR))
    }

    pub fn get_vendor_id(&self) -> Result<u32, Box<dyn Error>> {
        warn!("BluetoothDevice::get_vendor_id not supported by BlurMac");
        Err(Box::from(NOT_SUPPORTED_ERROR))
    }

    pub fn get_product_id(&self) -> Result<u32, Box<dyn Error>> {
        warn!("BluetoothDevice::get_product_id not supported by BlurMac");
        Err(Box::from(NOT_SUPPORTED_ERROR))
    }

    pub fn get_device_id(&self) -> Result<u32, Box<dyn Error>> {
        warn!("BluetoothDevice::get_device_id not supported by BlurMac");
        Err(Box::from(NOT_SUPPORTED_ERROR))
    }

    pub fn get_modalias(&self) -> Result<(String, u32, u32, u32), Box<dyn Error>> {
        warn!("BluetoothDevice::get_modalias not supported by BlurMac");
        Err(Box::from(NOT_SUPPORTED_ERROR))
    }

    pub fn connect_profile(&self, _uuid: String) -> Result<(), Box<dyn Error>> {
        warn!("BluetoothDevice::connect_profile not supported by BlurMac");
        Err(Box::from(NOT_SUPPORTED_ERROR))
    }

    pub fn disconnect_profile(&self, _uuid: String) -> Result<(), Box<dyn Error>> {
        warn!("BluetoothDevice::disconnect_profile not supported by BlurMac");
        Err(Box::from(NOT_SUPPORTED_ERROR))
    }

    pub fn pair(&self) -> Result<(), Box<dyn Error>> {
        warn!("BluetoothDevice::pair not supported by BlurMac");
        Err(Box::from(NOT_SUPPORTED_ERROR))
    }

    pub fn cancel_pairing(&self) -> Result<(), Box<dyn Error>> {
        warn!("BluetoothDevice::cancel_pairing not supported by BlurMac");
        Err(Box::from(NOT_SUPPORTED_ERROR))
    }
}
