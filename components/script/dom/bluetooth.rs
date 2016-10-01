/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use bluetooth_blacklist::{Blacklist, uuid_is_blacklisted};
use core::clone::Clone;
use dom::bindings::codegen::Bindings::BluetoothBinding::{self, BluetoothMethods, BluetoothRequestDeviceFilter};
use dom::bindings::codegen::Bindings::BluetoothBinding::RequestDeviceOptions;
use dom::bindings::error::Error::{self, Security, Type};
use dom::bindings::error::Fallible;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::Root;
use dom::bindings::reflector::{Reflectable, Reflector, reflect_dom_object};
use dom::bindings::str::DOMString;
use dom::bluetoothadvertisingdata::BluetoothAdvertisingData;
use dom::bluetoothdevice::BluetoothDevice;
use dom::bluetoothuuid::{BluetoothServiceUUID, BluetoothUUID};
use dom::promise::Promise;
use ipc_channel::ipc::{self, IpcSender};
use js::conversions::ToJSValConvertible;
use net_traits::bluetooth_scanfilter::{BluetoothScanfilter, BluetoothScanfilterSequence};
use net_traits::bluetooth_scanfilter::{RequestDeviceoptions, ServiceUUIDSequence};
use net_traits::bluetooth_thread::{BluetoothError, BluetoothMethodMsg};
use std::rc::Rc;

const FILTER_EMPTY_ERROR: &'static str = "'filters' member, if present, must be nonempty to find any devices.";
const FILTER_ERROR: &'static str = "A filter must restrict the devices in some way.";
const FILTER_NAME_TOO_LONG_ERROR: &'static str = "A 'name' or 'namePrefix' can't be longer then 29 bytes.";
// 248 is the maximum number of UTF-8 code units in a Bluetooth Device Name.
const MAX_DEVICE_NAME_LENGTH: usize = 248;
// A device name can never be longer than 29 bytes.
// An advertising packet is at most 31 bytes long.
// The length and identifier of the length field take 2 bytes.
// That leaves 29 bytes for the name.
const MAX_FILTER_NAME_LENGTH: usize = 29;
const NAME_PREFIX_ERROR: &'static str = "'namePrefix', if present, must be nonempty.";
const NAME_TOO_LONG_ERROR: &'static str = "A device name can't be longer than 248 bytes.";
const SERVICE_ERROR: &'static str = "'services', if present, must contain at least one service.";
const OPTIONS_ERROR: &'static str = "Fields of 'options' conflict with each other.
 Either 'acceptAllDevices' member must be true, or 'filters' member must be set to a value.";

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

    fn request_device(&self, option: &RequestDeviceOptions) -> Fallible<Root<BluetoothDevice>> {
        // Step 1.
        // TODO(#4282): Reject promise.
        if (option.filters.is_some() && option.acceptAllDevices) ||
           (option.filters.is_none() && !option.acceptAllDevices) {
            return Err(Type(OPTIONS_ERROR.to_owned()));
        }
        // Step 2.
        if !option.acceptAllDevices {
            return self.request_bluetooth_devices(&option.filters, &option.optionalServices);
        }

        self.request_bluetooth_devices(&None, &option.optionalServices)
        // TODO(#4282): Step 3-5: Reject and resolve promise.
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#request-bluetooth-devices
    fn request_bluetooth_devices(&self,
                                 filters: &Option<Vec<BluetoothRequestDeviceFilter>>,
                                 optional_services: &Option<Vec<BluetoothServiceUUID>>)
                                 -> Fallible<Root<BluetoothDevice>> {
        // TODO: Step 1: Triggered by user activation.

        // Step 2.
        let option = try!(convert_request_device_options(filters, optional_services));

        // TODO: Step 3-5: Implement the permission API.

        // Note: Steps 6-8 are implemented in
        // components/net/bluetooth_thread.rs in request_device function.
        let (sender, receiver) = ipc::channel().unwrap();
        self.get_bluetooth_thread().send(BluetoothMethodMsg::RequestDevice(option, sender)).unwrap();
        let device = receiver.recv().unwrap();

        // TODO: Step 9-10: Implement the permission API.

        // Step 11: This step is optional.

        // Step 12-13.
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
                Err(Error::from(error))
            },
        }

    }
}

// https://webbluetoothcg.github.io/web-bluetooth/#request-bluetooth-devices
fn convert_request_device_options(filters: &Option<Vec<BluetoothRequestDeviceFilter>>,
                                  optional_services: &Option<Vec<BluetoothServiceUUID>>)
                                  -> Fallible<RequestDeviceoptions> {
    // Step 2.2: There is no requiredServiceUUIDS, we scan for all devices.
    let mut uuid_filters = vec!();

    if let &Some(ref filters) = filters {
        // Step 2.1.
        if filters.is_empty()  {
            return Err(Type(FILTER_EMPTY_ERROR.to_owned()));
        }

        // Step 2.3: There is no requiredServiceUUIDS, we scan for all devices.

        // Step 2.4.
        for filter in filters {
            // Step 2.4.8.
            uuid_filters.push(try!(canonicalize_filter(&filter)));
        }
    }

    let mut optional_services_uuids = vec!();
    if let &Some(ref opt_services) = optional_services {
        for opt_service in opt_services {
            // Step 2.5 - 2.6.
            let uuid = try!(BluetoothUUID::service(opt_service.clone())).to_string();

            // Step 2.7.
            // Note: What we are doing here is adding the not blacklisted UUIDs to the result vector,
            // insted of removing them from an already filled vector.
            if !uuid_is_blacklisted(uuid.as_ref(), Blacklist::All) {
                optional_services_uuids.push(uuid);
            }
        }
    }

    Ok(RequestDeviceoptions::new(BluetoothScanfilterSequence::new(uuid_filters),
                                 ServiceUUIDSequence::new(optional_services_uuids)))
}

// https://webbluetoothcg.github.io/web-bluetooth/#request-bluetooth-devices
fn canonicalize_filter(filter: &BluetoothRequestDeviceFilter) -> Fallible<BluetoothScanfilter> {
    // Step 2.4.1.
    if filter.services.is_none() &&
       filter.name.is_none() &&
       filter.namePrefix.is_none() &&
       filter.manufacturerId.is_none() &&
       filter.serviceDataUUID.is_none() {
           return Err(Type(FILTER_ERROR.to_owned()));
    }

    // Step 2.4.2: There is no empty canonicalizedFilter member,
    // we create a BluetoothScanfilter instance at the end of the function.

    // Step 2.4.3.
    let services_vec = match filter.services {
        Some(ref services) => {
            // Step 2.4.3.1.
            if services.is_empty() {
                return Err(Type(SERVICE_ERROR.to_owned()));
            }

            let mut services_vec = vec!();

            for service in services {
                // Step 2.4.3.2 - 2.4.3.3.
                let uuid = try!(BluetoothUUID::service(service.clone())).to_string();

                // Step 2.4.3.4.
                if uuid_is_blacklisted(uuid.as_ref(), Blacklist::All) {
                    return Err(Security)
                }

                services_vec.push(uuid);
            }
            // Step 2.4.3.5.
            services_vec
            // Step 2.4.3.6: There is no requiredServiceUUIDS, we scan for all devices.
        },
        None => vec!(),
    };

    // Step 2.4.4.
    let name = match filter.name {
        Some(ref name) => {
            // Step 2.4.4.1.
            // Note: DOMString::len() gives back the size in bytes.
            if name.len() > MAX_DEVICE_NAME_LENGTH {
                return Err(Type(NAME_TOO_LONG_ERROR.to_owned()));
            }
            if name.len() > MAX_FILTER_NAME_LENGTH {
                return Err(Type(FILTER_NAME_TOO_LONG_ERROR.to_owned()));
            }

            // Step 2.4.4.2.
            name.to_string()
        },
        None => String::new(),
    };

    // Step 2.4.5.
    let name_prefix = match filter.namePrefix {
        Some(ref name_prefix) => {
            // Step 2.4.5.1.
            if name_prefix.is_empty() {
                return Err(Type(NAME_PREFIX_ERROR.to_owned()));
            }
            if name_prefix.len() > MAX_DEVICE_NAME_LENGTH {
                return Err(Type(NAME_TOO_LONG_ERROR.to_owned()));
            }
            if name_prefix.len() > MAX_FILTER_NAME_LENGTH {
                return Err(Type(FILTER_NAME_TOO_LONG_ERROR.to_owned()));
            }

            // Step 2.4.5.2.
            name_prefix.to_string()
        },
        None => String::new(),
    };

    // Step 2.4.6.
    let manufacturer_id = filter.manufacturerId;

    // Step 2.4.7.
    let service_data_uuid = match filter.serviceDataUUID {
        Some(ref service_data_uuid) => {
            // Step 2.4.7.1 - 2.4.7.2.
            let uuid = try!(BluetoothUUID::service(service_data_uuid.clone())).to_string();

            // Step 2.4.7.3.
            if uuid_is_blacklisted(uuid.as_ref(), Blacklist::All) {
                return Err(Security)
            }

            // Step 2.4.7.4.
            uuid
        },
        None => String::new(),
    };

    Ok(BluetoothScanfilter::new(name,
                                name_prefix,
                                services_vec,
                                manufacturer_id,
                                service_data_uuid))
}

#[allow(unrooted_must_root)]
pub fn result_to_promise<T: ToJSValConvertible>(global_ref: GlobalRef,
                                                bluetooth_result: Fallible<T>)
                                                -> Rc<Promise> {
    let p = Promise::new(global_ref);
    match bluetooth_result {
        Ok(v) => p.resolve_native(p.global().r().get_cx(), &v),
        Err(e) => p.reject_error(p.global().r().get_cx(), e),
    }
    p
}

impl From<BluetoothError> for Error {
    fn from(error: BluetoothError) -> Self {
        match error {
            BluetoothError::Type(message) => Error::Type(message),
            BluetoothError::Network => Error::Network,
            BluetoothError::NotFound => Error::NotFound,
            BluetoothError::NotSupported => Error::NotSupported,
            BluetoothError::Security => Error::Security,
        }
    }
}

impl BluetoothMethods for Bluetooth {
    #[allow(unrooted_must_root)]
    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetooth-requestdevice
    fn RequestDevice(&self, option: &RequestDeviceOptions) -> Rc<Promise> {
        result_to_promise(self.global().r(), self.request_device(option))
    }
}
