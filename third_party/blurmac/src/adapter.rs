// Copyright (c) 2017 Akos Kiss.
//
// Licensed under the BSD 3-Clause License
// <LICENSE.md or https://opensource.org/licenses/BSD-3-Clause>.
// This file may not be copied, modified, or distributed except
// according to those terms.

use std::error::Error;
use std::os::raw::c_int;

use delegate::bm;
use framework::{cb, io, ns};
use objc::runtime::{Object, YES};
use utils::{nsx, NOT_SUPPORTED_ERROR};

#[derive(Clone, Debug)]
pub struct BluetoothAdapter {
    pub(crate) manager: *mut Object,
    pub(crate) delegate: *mut Object,
}
// TODO: implement std::fmt::Debug and/or std::fmt::Display instead of derive?

unsafe impl Send for BluetoothAdapter {}
unsafe impl Sync for BluetoothAdapter {}

impl BluetoothAdapter {
    pub fn init() -> Result<BluetoothAdapter, Box<dyn Error>> {
        trace!("BluetoothAdapter::init");
        let delegate = bm::delegate();
        let manager = cb::centralmanager(delegate);
        let adapter = BluetoothAdapter { manager, delegate };

        // NOTE: start discovery at once, servo leaves close to no time to do a proper discovery
        // in a BluetoothDiscoverySession
        adapter.start_discovery().unwrap();

        Ok(adapter)
    }

    pub fn get_id(&self) -> String {
        trace!("BluetoothAdapter::get_id");
        // NOTE: not aware of any better native ID than the address string
        self.get_address().unwrap()
    }

    pub fn get_name(&self) -> Result<String, Box<dyn Error>> {
        trace!("BluetoothAdapter::get_name");
        let controller = io::bluetoothhostcontroller_defaultcontroller();
        let name = io::bluetoothhostcontroller_nameasstring(controller);
        Ok(nsx::string_to_string(name))
    }

    pub fn get_address(&self) -> Result<String, Box<dyn Error>> {
        trace!("BluetoothAdapter::get_address");
        let controller = io::bluetoothhostcontroller_defaultcontroller();
        let address = io::bluetoothhostcontroller_addressasstring(controller);
        Ok(nsx::string_to_string(address))
    }

    pub fn get_class(&self) -> Result<u32, Box<dyn Error>> {
        trace!("BluetoothAdapter::get_class");
        let controller = io::bluetoothhostcontroller_defaultcontroller();
        let device_class = io::bluetoothhostcontroller_classofdevice(controller);
        Ok(device_class)
    }

    pub fn is_powered(&self) -> Result<bool, Box<dyn Error>> {
        trace!("BluetoothAdapter::is_powered");
        // NOTE: might be also available through
        // [[IOBluetoothHostController defaultController] powerState], but that's readonly, so keep
        // it in sync with set_powered
        Ok(io::bluetoothpreferencegetcontrollerpowerstate() == 1)
    }

    pub fn set_powered(&self, value: bool) -> Result<(), Box<dyn Error>> {
        trace!("BluetoothAdapter::set_powered");
        io::bluetoothpreferencesetcontrollerpowerstate(value as c_int);
        // TODO: wait for change to happen? whether it really happened?
        Ok(())
    }

    pub fn is_discoverable(&self) -> Result<bool, Box<dyn Error>> {
        trace!("BluetoothAdapter::is_discoverable");
        Ok(io::bluetoothpreferencegetdiscoverablestate() == 1)
    }

    pub fn set_discoverable(&self, value: bool) -> Result<(), Box<dyn Error>> {
        trace!("BluetoothAdapter::set_discoverable");
        io::bluetoothpreferencesetdiscoverablestate(value as c_int);
        // TODO: wait for change to happen? whether it really happened?
        Ok(())
    }

    pub fn get_device_list(&self) -> Result<Vec<String>, Box<dyn Error>> {
        trace!("BluetoothAdapter::get_device_list");
        let mut v = vec![];
        let peripherals = bm::delegate_peripherals(self.delegate);
        let keys = ns::dictionary_allkeys(peripherals);
        for i in 0..ns::array_count(keys) {
            v.push(nsx::string_to_string(ns::array_objectatindex(keys, i)));
        }
        Ok(v)
    }

    // Was in BluetoothDiscoverySession

    fn start_discovery(&self) -> Result<(), Box<dyn Error>> {
        trace!("BluetoothAdapter::start_discovery");
        let options = ns::mutabledictionary();
        // NOTE: If duplicates are not allowed then a peripheral will not show up again once
        // connected and then disconnected.
        ns::mutabledictionary_setobject_forkey(options, ns::number_withbool(YES), unsafe {
            cb::CENTRALMANAGERSCANOPTIONALLOWDUPLICATESKEY
        });
        cb::centralmanager_scanforperipherals_options(self.manager, options);
        Ok(())
    }

    fn stop_discovery(&self) -> Result<(), Box<dyn Error>> {
        trace!("BluetoothAdapter::stop_discovery");
        cb::centralmanager_stopscan(self.manager);
        Ok(())
    }

    // Not supported

    pub fn get_alias(&self) -> Result<String, Box<dyn Error>> {
        warn!("BluetoothAdapter::get_alias not supported by BlurMac");
        Err(Box::from(NOT_SUPPORTED_ERROR))
    }

    pub fn set_alias(&self, _value: String) -> Result<(), Box<dyn Error>> {
        warn!("BluetoothAdapter::set_alias not supported by BlurMac");
        Err(Box::from(NOT_SUPPORTED_ERROR))
    }

    pub fn is_pairable(&self) -> Result<bool, Box<dyn Error>> {
        warn!("BluetoothAdapter::is_pairable not supported by BlurMac");
        Err(Box::from(NOT_SUPPORTED_ERROR))
    }

    pub fn set_pairable(&self, _value: bool) -> Result<(), Box<dyn Error>> {
        warn!("BluetoothAdapter::set_pairable not supported by BlurMac");
        Err(Box::from(NOT_SUPPORTED_ERROR))
    }

    pub fn get_pairable_timeout(&self) -> Result<u32, Box<dyn Error>> {
        warn!("BluetoothAdapter::get_pairable_timeout not supported by BlurMac");
        Err(Box::from(NOT_SUPPORTED_ERROR))
    }

    pub fn set_pairable_timeout(&self, _value: u32) -> Result<(), Box<dyn Error>> {
        warn!("BluetoothAdapter::set_pairable_timeout not supported by BlurMac");
        Err(Box::from(NOT_SUPPORTED_ERROR))
    }

    pub fn get_discoverable_timeout(&self) -> Result<u32, Box<dyn Error>> {
        warn!("BluetoothAdapter::get_discoverable_timeout not supported by BlurMac");
        Err(Box::from(NOT_SUPPORTED_ERROR))
    }

    pub fn set_discoverable_timeout(&self, _value: u32) -> Result<(), Box<dyn Error>> {
        warn!("BluetoothAdapter::set_discoverable_timeout not supported by BlurMac");
        Err(Box::from(NOT_SUPPORTED_ERROR))
    }

    pub fn is_discovering(&self) -> Result<bool, Box<dyn Error>> {
        warn!("BluetoothAdapter::is_discovering not supported by BlurMac");
        Err(Box::from(NOT_SUPPORTED_ERROR))
    }

    pub fn get_uuids(&self) -> Result<Vec<String>, Box<dyn Error>> {
        warn!("BluetoothAdapter::get_uuids not supported by BlurMac");
        Err(Box::from(NOT_SUPPORTED_ERROR))
    }

    pub fn get_vendor_id_source(&self) -> Result<String, Box<dyn Error>> {
        warn!("BluetoothAdapter::get_vendor_id_source not supported by BlurMac");
        Err(Box::from(NOT_SUPPORTED_ERROR))
    }

    pub fn get_vendor_id(&self) -> Result<u32, Box<dyn Error>> {
        warn!("BluetoothAdapter::get_vendor_id not supported by BlurMac");
        Err(Box::from(NOT_SUPPORTED_ERROR))
    }

    pub fn get_product_id(&self) -> Result<u32, Box<dyn Error>> {
        warn!("BluetoothAdapter::get_product_id not supported by BlurMac");
        Err(Box::from(NOT_SUPPORTED_ERROR))
    }

    pub fn get_device_id(&self) -> Result<u32, Box<dyn Error>> {
        warn!("BluetoothAdapter::get_device_id not supported by BlurMac");
        Err(Box::from(NOT_SUPPORTED_ERROR))
    }

    pub fn get_modalias(&self) -> Result<(String, u32, u32, u32), Box<dyn Error>> {
        warn!("BluetoothAdapter::get_modalias not supported by BlurMac");
        Err(Box::from(NOT_SUPPORTED_ERROR))
    }
}

impl Drop for BluetoothAdapter {
    fn drop(&mut self) {
        trace!("BluetoothAdapter::drop");
        // NOTE: stop discovery only here instead of in BluetoothDiscoverySession
        self.stop_discovery().unwrap();
    }
}
