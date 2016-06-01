/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use bluetooth_blacklist::{Blacklist, uuid_is_blacklisted};
use core::clone::Clone;
use dom::bindings::codegen::Bindings::BluetoothBinding;
use dom::bindings::codegen::Bindings::BluetoothBinding::RequestDeviceOptions;
use dom::bindings::codegen::Bindings::BluetoothBinding::{BluetoothScanFilter, BluetoothMethods};
use dom::bindings::error::Error::{Security, Type};
use dom::bindings::error::Fallible;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::Root;
use dom::bindings::reflector::{Reflectable, Reflector, reflect_dom_object};
use dom::bindings::str::DOMString;
use dom::bluetoothadvertisingdata::BluetoothAdvertisingData;
use dom::bluetoothdevice::BluetoothDevice;
use dom::bluetoothuuid::BluetoothUUID;
use ipc_channel::ipc::{self, IpcSender};
use net_traits::bluetooth_scanfilter::{BluetoothScanfilter, BluetoothScanfilterSequence};
use net_traits::bluetooth_scanfilter::{RequestDeviceoptions, ServiceUUIDSequence};
use net_traits::bluetooth_thread::BluetoothMethodMsg;

const FILTER_EMPTY_ERROR: &'static str = "'filters' member must be non - empty to find any devices.";
const FILTER_ERROR: &'static str = "A filter must restrict the devices in some way.";
const FILTER_NAME_TOO_LONG_ERROR: &'static str = "A 'name' or 'namePrefix' can't be longer then 29 bytes.";
// 248 is the maximum number of UTF-8 code units in a Bluetooth Device Name.
const MAX_DEVICE_NAME_LENGTH: usize = 248;
// A device name can never be longer than 29 bytes.
// An advertising packet is at most 31 bytes long.
// The length and identifier of the length field take 2 bytes.
// That leaves 29 bytes for the name.
const MAX_FILTER_NAME_LENGTH: usize = 29;
const NAME_PREFIX_ERROR: &'static str = "'namePrefix', if present, must be non - empty.";
const NAME_TOO_LONG_ERROR: &'static str = "A device name can't be longer than 248 bytes.";
const SERVICE_ERROR: &'static str = "'services', if present, must contain at least one service.";

// https://webbluetoothcg.github.io/web-bluetooth/#bluetooth
#[dom_struct]
pub struct Bluetooth {
    reflector_: Reflector,
}

impl Bluetooth {
    pub fn new_inherited() -> Bluetooth {
        Bluetooth {
            reflector_: Reflector::new(),
        }
    }

    pub fn new(global: GlobalRef) -> Root<Bluetooth> {
        reflect_dom_object(box Bluetooth::new_inherited(),
                           global,
                           BluetoothBinding::Wrap)
    }

    fn get_bluetooth_thread(&self) -> IpcSender<BluetoothMethodMsg> {
        let global_root = self.global();
        let global_ref = global_root.r();
        global_ref.as_window().bluetooth_thread()
    }
}

fn canonicalize_filter(filter: &BluetoothScanFilter, global: GlobalRef) -> Fallible<BluetoothScanfilter> {
    if filter.services.is_none() && filter.name.is_none() && filter.namePrefix.is_none() {
        return Err(Type(FILTER_ERROR.to_owned()));
    }

    let mut services_vec = vec!();
    if let Some(ref services) = filter.services {
        if services.is_empty() {
            return Err(Type(SERVICE_ERROR.to_owned()));
        }
        for service in services {
            let uuid = try!(BluetoothUUID::GetService(global, service.clone())).to_string();
            if uuid_is_blacklisted(uuid.as_ref(), Blacklist::All) {
                return Err(Security)
            }
            services_vec.push(uuid);
        }
    }

    let mut name = String::new();
    if let Some(ref filter_name) = filter.name {
        //NOTE: DOMString::len() gives back the size in bytes
        if filter_name.len() > MAX_DEVICE_NAME_LENGTH {
            return Err(Type(NAME_TOO_LONG_ERROR.to_owned()));
        }
        if filter_name.len() > MAX_FILTER_NAME_LENGTH {
            return Err(Type(FILTER_NAME_TOO_LONG_ERROR.to_owned()));
        }
        name = filter_name.to_string();
    }

    let mut name_prefix = String::new();
    if let Some(ref filter_name_prefix) = filter.namePrefix {
        if filter_name_prefix.is_empty() {
            return Err(Type(NAME_PREFIX_ERROR.to_owned()));
        }
        if filter_name_prefix.len() > MAX_DEVICE_NAME_LENGTH {
            return Err(Type(NAME_TOO_LONG_ERROR.to_owned()));
        }
        if filter_name_prefix.len() > MAX_FILTER_NAME_LENGTH {
            return Err(Type(FILTER_NAME_TOO_LONG_ERROR.to_owned()));
        }
        name_prefix = filter_name_prefix.to_string();
    }

    Ok(BluetoothScanfilter::new(name, name_prefix, services_vec))
}

fn convert_request_device_options(options: &RequestDeviceOptions,
                                  global: GlobalRef)
                                  -> Fallible<RequestDeviceoptions> {
    if options.filters.is_empty() {
        return Err(Type(FILTER_EMPTY_ERROR.to_owned()));
    }

    let mut filters = vec!();
    for filter in &options.filters {
        filters.push(try!(canonicalize_filter(&filter, global)));
    }

    let mut optional_services = vec!();
    if let Some(ref opt_services) = options.optionalServices {
        for opt_service in opt_services {
            let uuid = try!(BluetoothUUID::GetService(global, opt_service.clone())).to_string();
            if uuid_is_blacklisted(uuid.as_ref(), Blacklist::All) {
                return Err(Security)
            }
            optional_services.push(uuid);
        }
    }

    Ok(RequestDeviceoptions::new(BluetoothScanfilterSequence::new(filters),
                                 ServiceUUIDSequence::new(optional_services)))
}

impl BluetoothMethods for Bluetooth {
    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetooth-requestdevice
    fn RequestDevice(&self, option: &RequestDeviceOptions) -> Fallible<Root<BluetoothDevice>> {
        let (sender, receiver) = ipc::channel().unwrap();
        let option = try!(convert_request_device_options(option, self.global().r()));
        self.get_bluetooth_thread().send(BluetoothMethodMsg::RequestDevice(option, sender)).unwrap();
        let device = receiver.recv().unwrap();
        match device {
            Ok(device) => {
                let ad_data = BluetoothAdvertisingData::new(self.global().r(),
                                                            device.appearance,
                                                            device.tx_power,
                                                            device.rssi);
                Ok(BluetoothDevice::new(self.global().r(),
                                        DOMString::from(device.id),
                                        device.name.map(DOMString::from),
                                        &ad_data))
            },
            Err(error) => {
                Err(Type(error))
            },
        }
    }
}
