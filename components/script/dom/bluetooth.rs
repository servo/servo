/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use core::clone::Clone;
use dom::bindings::codegen::Bindings::BluetoothBinding;
use dom::bindings::codegen::Bindings::BluetoothBinding::RequestDeviceOptions;
use dom::bindings::codegen::Bindings::BluetoothBinding::{BluetoothScanFilter, BluetoothMethods};
use dom::bindings::codegen::Bindings::BluetoothDeviceBinding::VendorIDSource;
use dom::bindings::error::Error::Type;
use dom::bindings::error::Fallible;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::Root;
use dom::bindings::reflector::{Reflectable, Reflector, reflect_dom_object};
use dom::bluetoothadvertisingdata::BluetoothAdvertisingData;
use dom::bluetoothdevice::BluetoothDevice;
use dom::bluetoothuuid::BluetoothUUID;
use ipc_channel::ipc::{self, IpcSender};
use net_traits::bluetooth_scanfilter::{BluetoothScanfilter, BluetoothScanfilterSequence};
use net_traits::bluetooth_scanfilter::{RequestDeviceoptions, ServiceUUIDSequence};
use net_traits::bluetooth_thread::{BluetoothMethodMsg, BluetoothObjectMsg};
use util::str::DOMString;

// A device name can never be longer than 29 bytes. An adv packet is at most
// 31 bytes long. The length and identifier of the length field take 2 bytes.
const FILTER_EMPTY_ERROR: &'static str = "'filters' member must be non - empty to find any devices.";
const FILTER_ERROR: &'static str = "A filter must restrict the devices in some way.";
const FILTER_NAME_TOO_LONG_ERROR: &'static str = "A 'name' or 'namePrefix' can't be longer then 29 bytes.";
// 248 is the maximum number of UTF-8 code units in a Bluetooth Device Name.
const MAX_DEVICE_NAME_LENGTH: usize = 248;
// That least 29 bytes for the name.
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

impl Clone for BluetoothScanFilter {
    fn clone(&self) -> BluetoothScanFilter {
        BluetoothScanFilter {
            name: self.name.clone(),
            namePrefix: self.namePrefix.clone(),
            services: self.services.clone(),
        }
    }
}

impl Clone for RequestDeviceOptions {
    fn clone(&self) -> RequestDeviceOptions {
        RequestDeviceOptions {
            filters: self.filters.clone(),
            optionalServices: self.optionalServices.clone(),
        }
    }
}

fn canonicalize_filter(filter: &BluetoothScanFilter, global: GlobalRef) -> Fallible<BluetoothScanfilter> {
    if !(filter.services.is_some() || filter.name.is_some() || filter.namePrefix.is_some()) {
        return Err(Type(FILTER_ERROR.to_owned()));
    }

    let mut services_vec: Vec<String> = vec!();
    if let Some(services) = filter.services.clone() {
        if services.is_empty() {
            return Err(Type(SERVICE_ERROR.to_owned()));
        }
        for service in services {
            match BluetoothUUID::GetService(global, service) {
                Ok(valid) => services_vec.push(valid.to_string()),
                Err(err) => return Err(err),
            }
        }
    }

    let mut name = String::new();
    if let Some(filter_name) = filter.name.clone() {
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
    if let Some(filter_name_prefix) = filter.namePrefix.clone() {
        if filter_name_prefix.len() == 0 {
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
        match canonicalize_filter(&filter, global) {
            Ok(canonicalized_filter) => filters.push(canonicalized_filter),
            Err(err) => return Err(err),
        }
    }

    let mut optional_services = vec!();
    if let Some(opt_services) = options.optionalServices.clone() {
        for opt_service in opt_services {
            match BluetoothUUID::GetService(global, opt_service) {
                Ok(valid_service) => optional_services.push(valid_service.to_string()),
                Err(err) => return Err(err),
            }
        }
    }

    Ok(RequestDeviceoptions::new(BluetoothScanfilterSequence::new(filters),
                                 ServiceUUIDSequence::new(optional_services)))
}

impl BluetoothMethods for Bluetooth {

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetooth-requestdevice
    fn RequestDevice(&self, option: &RequestDeviceOptions) -> Fallible<Root<BluetoothDevice>> {
        let (sender, receiver) = ipc::channel().unwrap();
        match convert_request_device_options(option, self.global().r()) {
            Ok(option) => {
                self.get_bluetooth_thread().send(
                    BluetoothMethodMsg::RequestDevice(option, sender)).unwrap();
                let device = receiver.recv().unwrap();
                match device {
                    BluetoothObjectMsg::BluetoothDevice {
                        id,
                        name,
                        device_class,
                        vendor_id_source,
                        vendor_id,
                        product_id,
                        product_version,
                        appearance,
                        tx_power,
                        rssi,
                    } => {
                        let ad_data = &BluetoothAdvertisingData::new(self.global().r(),
                                                                     appearance,
                                                                     tx_power,
                                                                     rssi);
                        let vendor_id_source = match vendor_id_source {
                            Some(vid) => match vid.as_ref() {
                                "bluetooth" => Some(VendorIDSource::Bluetooth),
                                "usb" => Some(VendorIDSource::Usb),
                                _ => Some(VendorIDSource::Unknown),
                            },
                            None => None,
                        };
                        let name = match name {
                            Some(n) => Some(DOMString::from(n)),
                            None => None,
                        };
                        Ok(BluetoothDevice::new(self.global().r(),
                                                DOMString::from(id),
                                                name,
                                                ad_data,
                                                device_class,
                                                vendor_id_source,
                                                vendor_id,
                                                product_id,
                                                product_version))
                    },
                    BluetoothObjectMsg::Error {
                        error
                    } => {
                        Err(Type(error))
                    },
                    _ => unreachable!()
                }
            },
            Err(err) => Err(err),
        }
    }
}
