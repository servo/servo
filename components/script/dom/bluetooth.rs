/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use core::clone::Clone;
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::BluetoothBinding::{self, BluetoothMethods, BluetoothRequestDeviceFilter};
use dom::bindings::codegen::Bindings::BluetoothBinding::RequestDeviceOptions;
use dom::bindings::error::Error::{self, NotFound, Security, Type};
use dom::bindings::error::Fallible;
use dom::bindings::js::{JS, MutHeap, Root};
use dom::bindings::reflector::{Reflectable, Reflector, reflect_dom_object};
use dom::bindings::str::DOMString;
use dom::bluetoothadvertisingdata::BluetoothAdvertisingData;
use dom::bluetoothdevice::BluetoothDevice;
use dom::bluetoothremotegattcharacteristic::BluetoothRemoteGATTCharacteristic;
use dom::bluetoothremotegattdescriptor::BluetoothRemoteGATTDescriptor;
use dom::bluetoothremotegattservice::BluetoothRemoteGATTService;
use dom::bluetoothuuid::{BluetoothServiceUUID, BluetoothUUID};
use dom::globalscope::GlobalScope;
use dom::promise::Promise;
use ipc_channel::ipc::{self, IpcSender};
use js::conversions::ToJSValConvertible;
use net_traits::bluetooth_blacklist::{Blacklist, uuid_is_blacklisted};
use net_traits::bluetooth_scanfilter::{BluetoothScanfilter, BluetoothScanfilterSequence};
use net_traits::bluetooth_scanfilter::{RequestDeviceoptions, ServiceUUIDSequence};
use net_traits::bluetooth_thread::{BluetoothError, BluetoothMethodMsg};
use std::collections::HashMap;
use std::rc::Rc;

const FILTER_EMPTY_ERROR: &'static str = "'filters' member, if present, must be nonempty to find any devices.";
const FILTER_ERROR: &'static str = "A filter must restrict the devices in some way.";
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
    device_instance_map: DOMRefCell<HashMap<String, MutHeap<JS<BluetoothDevice>>>>,
    service_instance_map: DOMRefCell<HashMap<String, MutHeap<JS<BluetoothRemoteGATTService>>>>,
    characteristic_instance_map: DOMRefCell<HashMap<String, MutHeap<JS<BluetoothRemoteGATTCharacteristic>>>>,
    descriptor_instance_map: DOMRefCell<HashMap<String, MutHeap<JS<BluetoothRemoteGATTDescriptor>>>>,
}

impl Bluetooth {
    pub fn new_inherited() -> Bluetooth {
        Bluetooth {
            reflector_: Reflector::new(),
            device_instance_map: DOMRefCell::new(HashMap::new()),
            service_instance_map: DOMRefCell::new(HashMap::new()),
            characteristic_instance_map: DOMRefCell::new(HashMap::new()),
            descriptor_instance_map: DOMRefCell::new(HashMap::new()),
        }
    }

    pub fn new(global: &GlobalScope) -> Root<Bluetooth> {
        reflect_dom_object(box Bluetooth::new_inherited(),
                           global,
                           BluetoothBinding::Wrap)
    }

    pub fn get_service_map(&self) -> &DOMRefCell<HashMap<String, MutHeap<JS<BluetoothRemoteGATTService>>>> {
        &self.service_instance_map
    }

    pub fn get_characteristic_map(&self)
            -> &DOMRefCell<HashMap<String, MutHeap<JS<BluetoothRemoteGATTCharacteristic>>>> {
        &self.characteristic_instance_map
    }

    pub fn get_descriptor_map(&self) -> &DOMRefCell<HashMap<String, MutHeap<JS<BluetoothRemoteGATTDescriptor>>>> {
        &self.descriptor_instance_map
    }

    fn get_bluetooth_thread(&self) -> IpcSender<BluetoothMethodMsg> {
        self.global().as_window().bluetooth_thread()
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
                let mut device_instance_map = self.device_instance_map.borrow_mut();
                if let Some(existing_device) = device_instance_map.get(&device.id.clone()) {
                    return Ok(existing_device.get());
                }
                let ad_data = BluetoothAdvertisingData::new(&self.global(),
                                                            device.appearance,
                                                            device.tx_power,
                                                            device.rssi);
                let bt_device = BluetoothDevice::new(&self.global(),
                                                     DOMString::from(device.id.clone()),
                                                     device.name.map(DOMString::from),
                                                     &ad_data,
                                                     &self);
                device_instance_map.insert(device.id, MutHeap::new(&bt_device));
                Ok(bt_device)
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
                return Err(NotFound);
            }

            // Step 2.4.4.2.
            Some(name.to_string())
        },
        None => None,
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
                return Err(NotFound);
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
pub fn result_to_promise<T: ToJSValConvertible>(global: &GlobalScope,
                                                bluetooth_result: Fallible<T>)
                                                -> Rc<Promise> {
    let p = Promise::new(global);
    match bluetooth_result {
        Ok(v) => p.resolve_native(p.global().get_cx(), &v),
        Err(e) => p.reject_error(p.global().get_cx(), e),
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
            BluetoothError::InvalidState => Error::InvalidState,
        }
    }
}

impl BluetoothMethods for Bluetooth {
    #[allow(unrooted_must_root)]
    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetooth-requestdevice
    fn RequestDevice(&self, option: &RequestDeviceOptions) -> Rc<Promise> {
        result_to_promise(&self.global(), self.request_device(option))
    }
}
