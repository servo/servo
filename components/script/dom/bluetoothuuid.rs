/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::UnionTypes::StringOrUnsignedLong;
use dom::bindings::error::Error::Syntax;
use dom::bindings::error::Fallible;
use dom::bindings::global::GlobalRef;
use dom::bindings::reflector::Reflector;
use regex::Regex;
use util::str::DOMString;

pub type UUID = DOMString;

// https://webbluetoothcg.github.io/web-bluetooth/#bluetoothuuid
 #[dom_struct]
pub struct BluetoothUUID {
    reflector_: Reflector,
}

const BLUETOOTH_ASSIGNED_SERVICES: &'static [(&'static str, u32)] = &[
//TODO(zakorgy) create all the services
//https://developer.bluetooth.org/gatt/services/Pages/ServicesHome.aspx
    ("org.bluetooth.service.alert_notification", 0x1811_u32),
    ("org.bluetooth.service.automation_io", 0x1815_u32),
    ("org.bluetooth.service.battery_service", 0x180f_u32)
];

const BLUETOOTH_ASSIGNED_CHARCTERISTICS: &'static [(&'static str, u32)] = &[
//TODO(zakorgy) create all the characteristics
//https://developer.bluetooth.org/gatt/services/Pages/ServicesHome.aspx
    ("org.bluetooth.characteristic.aerobic_heart_rate_lower_limit", 0x2a7e_u32),
    ("org.bluetooth.characteristic.aerobic_heart_rate_upper_limit", 0x2a84_u32),
    ("org.bluetooth.characteristic.battery_level", 0x2a19_u32)
];

const BLUETOOTH_ASSIGNED_DESCRIPTORS: &'static [(&'static str, u32)] = &[
//TODO(zakorgy) create all the descriptors
//https://developer.bluetooth.org/gatt/services/Pages/ServicesHome.aspx
    ("org.bluetooth.descriptor.gatt.characteristic_extended_properties", 0x2900_u32),
    ("org.bluetooth.descriptor.gatt.characteristic_user_description", 0x2901_u32)
];

const BASE_UUID: &'static str = "-0000-1000-8000-00805f9b34fb";
const SERVICE_PREFIX: &'static str = "org.bluetooth.service";
const CHARACTERISTIC_PREFIX: &'static str = "org.bluetooth.characteristic";
const DESCRIPTOR_PREFIX: &'static str = "org.bluetooth.descriptor";
const VALID_UUID_REGEX: &'static str = "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}";

impl BluetoothUUID {

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothuuid-canonicaluuid
    pub fn CanonicalUUID(_: GlobalRef, alias: u32) -> UUID {
        DOMString::from(format!("{:08x}", &alias) + BASE_UUID)
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothuuid-getservice
    pub fn GetService(globalref: GlobalRef,
                      name: StringOrUnsignedLong)
                      -> Fallible<UUID> {
      BluetoothUUID::resolve_uuid_name(globalref,
                                       name,
                                       BLUETOOTH_ASSIGNED_SERVICES,
                                       DOMString::from(SERVICE_PREFIX))
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothuuid-getcharacteristic
    pub fn GetCharacteristic(globalref: GlobalRef,
                             name: StringOrUnsignedLong)
                             -> Fallible<UUID> {
        BluetoothUUID::resolve_uuid_name(globalref,
                                         name,
                                         BLUETOOTH_ASSIGNED_CHARCTERISTICS,
                                         DOMString::from(CHARACTERISTIC_PREFIX))
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothuuid-getdescriptor
    pub fn GetDescriptor(globalref: GlobalRef,
                         name: StringOrUnsignedLong)
                         -> Fallible<UUID> {
        BluetoothUUID::resolve_uuid_name(globalref,
                                         name,
                                         BLUETOOTH_ASSIGNED_DESCRIPTORS,
                                         DOMString::from(DESCRIPTOR_PREFIX))
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#resolveuuidname
    pub fn resolve_uuid_name(globalref: GlobalRef,
                             name: StringOrUnsignedLong,
                             assigned_numbers_table: &'static [(&'static str, u32)],
                             prefix: DOMString)
                             -> Fallible<DOMString> {
        match name {
            // Step 1
            StringOrUnsignedLong::UnsignedLong(unsigned32) =>{
                Ok(BluetoothUUID::CanonicalUUID(globalref, unsigned32))
            },
            StringOrUnsignedLong::String(dstring) => {
            // Step 2
                let regex = Regex::new(VALID_UUID_REGEX).unwrap();
                if regex.is_match(&*dstring) {
                    Ok(dstring)
                } else {
                // Step 3
                    let concatenated = format!("{}.{}", prefix, dstring);
                    let is_in_table = assigned_numbers_table.iter()
                                                            .find(|p| p.0 == concatenated);
                    match is_in_table {
                        Some(&(_, alias)) => Ok(BluetoothUUID::CanonicalUUID(globalref, alias)),
                        None => Err(Syntax),
                    }
                }
            },
        }
    }
}
