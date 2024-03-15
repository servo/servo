// Copyright (c) 2017 Akos Kiss.
//
// Licensed under the BSD 3-Clause License
// <LICENSE.md or https://opensource.org/licenses/BSD-3-Clause>.
// This file may not be copied, modified, or distributed except
// according to those terms.

use std::error::Error;
use std::ffi::{CStr, CString};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::{thread, time};

use framework::{cb, nil, ns};
use objc::runtime::Object;

pub const NOT_SUPPORTED_ERROR: &str = "Error! Not supported by blurmac!";
pub const NO_PERIPHERAL_FOUND: &str = "Error! No peripheral found!";
pub const NO_SERVICE_FOUND: &str = "Error! No service found!";
pub const NO_CHARACTERISTIC_FOUND: &str = "Error! No characteristic found!";

pub mod nsx {
    use super::*;

    pub fn string_to_string(nsstring: *mut Object) -> String {
        if nsstring == nil {
            return String::from("nil");
        }
        unsafe {
            String::from(
                CStr::from_ptr(ns::string_utf8string(nsstring))
                    .to_str()
                    .unwrap(),
            )
        }
    }

    pub fn string_from_str(string: &str) -> *mut Object {
        let cstring = CString::new(string).unwrap();
        ns::string(cstring.as_ptr())
    }
}

pub mod cbx {
    use super::*;

    pub fn uuid_to_canonical_uuid_string(cbuuid: *mut Object) -> String {
        // NOTE: CoreBluetooth tends to return uppercase UUID strings, and only 4 character long if the
        // UUID is short (16 bits). However, WebBluetooth mandates lowercase UUID strings. And Servo
        // seems to compare strings, not the binary representation.
        let uuid = nsx::string_to_string(cb::uuid_uuidstring(cbuuid));
        let long = if uuid.len() == 4 {
            format!("0000{}-0000-1000-8000-00805f9b34fb", uuid)
        } else {
            uuid
        };
        long.to_lowercase()
    }

    pub fn peripheral_debug(peripheral: *mut Object) -> String {
        if peripheral == nil {
            return String::from("nil");
        }
        let name = cb::peripheral_name(peripheral);
        let uuid = ns::uuid_uuidstring(cb::peer_identifier(peripheral));
        if name != nil {
            format!(
                "CBPeripheral({}, {})",
                nsx::string_to_string(name),
                nsx::string_to_string(uuid)
            )
        } else {
            format!("CBPeripheral({})", nsx::string_to_string(uuid))
        }
    }

    pub fn service_debug(service: *mut Object) -> String {
        if service == nil {
            return String::from("nil");
        }
        let uuid = cb::uuid_uuidstring(cb::attribute_uuid(service));
        format!("CBService({})", nsx::string_to_string(uuid))
    }

    pub fn characteristic_debug(characteristic: *mut Object) -> String {
        if characteristic == nil {
            return String::from("nil");
        }
        let uuid = cb::uuid_uuidstring(cb::attribute_uuid(characteristic));
        format!("CBCharacteristic({})", nsx::string_to_string(uuid))
    }
}

pub mod wait {
    use super::*;

    pub type Timestamp = u64;

    static TIMESTAMP: AtomicUsize = AtomicUsize::new(0);

    pub fn get_timestamp() -> Timestamp {
        TIMESTAMP.fetch_add(1, Ordering::SeqCst) as u64
    }

    pub fn now() -> *mut Object {
        ns::number_withunsignedlonglong(get_timestamp())
    }

    pub fn wait_or_timeout<F>(mut f: F) -> Result<(), Box<dyn Error>>
    where
        F: FnMut() -> bool,
    {
        let now = time::Instant::now();

        while !f() {
            thread::sleep(time::Duration::from_secs(1));
            if now.elapsed().as_secs() > 30 {
                return Err(Box::from("timeout"));
            }
        }
        Ok(())
    }
}
