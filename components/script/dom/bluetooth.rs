/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use bluetooth_traits::{BluetoothError, BluetoothRequest, GATTType};
use bluetooth_traits::{BluetoothResponse, BluetoothResponseResult};
use bluetooth_traits::blocklist::{Blocklist, uuid_is_blocklisted};
use bluetooth_traits::scanfilter::{BluetoothScanfilter, BluetoothScanfilterSequence};
use bluetooth_traits::scanfilter::{RequestDeviceoptions, ServiceUUIDSequence};
use core::clone::Clone;
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::BluetoothBinding::{self, BluetoothDataFilterInit, BluetoothLEScanFilterInit};
use dom::bindings::codegen::Bindings::BluetoothBinding::{BluetoothMethods, RequestDeviceOptions};
use dom::bindings::codegen::Bindings::EventHandlerBinding::EventHandlerNonNull;
use dom::bindings::codegen::UnionTypes::StringOrUnsignedLong;
use dom::bindings::error::Error::{self, Network, NotFound, Security, Type};
use dom::bindings::error::Fallible;
use dom::bindings::js::{MutJS, Root};
use dom::bindings::refcounted::{Trusted, TrustedPromise};
use dom::bindings::reflector::{DomObject, reflect_dom_object};
use dom::bindings::str::DOMString;
use dom::bluetoothdevice::BluetoothDevice;
use dom::bluetoothuuid::{BluetoothServiceUUID, BluetoothUUID, UUID};
use dom::eventtarget::EventTarget;
use dom::globalscope::GlobalScope;
use dom::promise::Promise;
use ipc_channel::ipc::{self, IpcSender};
use ipc_channel::router::ROUTER;
use js::jsapi::{JSAutoCompartment, JSContext};
use script_thread::Runnable;
use std::collections::HashMap;
use std::rc::Rc;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

const KEY_CONVERSION_ERROR: &'static str = "This `manufacturerData` key can not be parsed as unsigned short:";
const FILTER_EMPTY_ERROR: &'static str = "'filters' member, if present, must be nonempty to find any devices.";
const FILTER_ERROR: &'static str = "A filter must restrict the devices in some way.";
const MANUFACTURER_DATA_ERROR: &'static str = "'manufacturerData', if present, must be non-empty to filter devices.";
const MASK_LENGTH_ERROR: &'static str = "`mask`, if present, must have the same length as `dataPrefix`.";
// 248 is the maximum number of UTF-8 code units in a Bluetooth Device Name.
const MAX_DEVICE_NAME_LENGTH: usize = 248;
// A device name can never be longer than 29 bytes.
// An advertising packet is at most 31 bytes long.
// The length and identifier of the length field take 2 bytes.
// That leaves 29 bytes for the name.
const MAX_FILTER_NAME_LENGTH: usize = 29;
const NAME_PREFIX_ERROR: &'static str = "'namePrefix', if present, must be nonempty.";
const NAME_TOO_LONG_ERROR: &'static str = "A device name can't be longer than 248 bytes.";
const SERVICE_DATA_ERROR: &'static str = "'serviceData', if present, must be non-empty to filter devices.";
const SERVICE_ERROR: &'static str = "'services', if present, must contain at least one service.";
const OPTIONS_ERROR: &'static str = "Fields of 'options' conflict with each other.
 Either 'acceptAllDevices' member must be true, or 'filters' member must be set to a value.";

struct BluetoothContext<T: AsyncBluetoothListener + DomObject> {
    promise: Option<TrustedPromise>,
    receiver: Trusted<T>,
}

pub trait AsyncBluetoothListener {
    fn handle_response(&self, result: BluetoothResponse, cx: *mut JSContext, promise: &Rc<Promise>);
}

impl<T: AsyncBluetoothListener + DomObject> BluetoothContext<T> {
    #[allow(unrooted_must_root)]
    fn response(&mut self, response: BluetoothResponseResult) {
        let promise = self.promise.take().expect("bt promise is missing").root();
        let promise_cx = promise.global().get_cx();

        // JSAutoCompartment needs to be manually made.
        // Otherwise, Servo will crash.
        let _ac = JSAutoCompartment::new(promise_cx, promise.reflector().get_jsobject().get());
        match response {
            Ok(response) => self.receiver.root().handle_response(response, promise_cx, &promise),
            // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetooth-requestdevice
            // Step 3 - 4.
            Err(error) => promise.reject_error(promise_cx, Error::from(error)),
        }
    }
}

// https://webbluetoothcg.github.io/web-bluetooth/#bluetooth
#[dom_struct]
pub struct Bluetooth {
    eventtarget: EventTarget,
    device_instance_map: DOMRefCell<HashMap<String, MutJS<BluetoothDevice>>>,
}

impl Bluetooth {
    pub fn new_inherited() -> Bluetooth {
        Bluetooth {
            eventtarget: EventTarget::new_inherited(),
            device_instance_map: DOMRefCell::new(HashMap::new()),
        }
    }

    pub fn new(global: &GlobalScope) -> Root<Bluetooth> {
        reflect_dom_object(box Bluetooth::new_inherited(),
                           global,
                           BluetoothBinding::Wrap)
    }

    fn get_bluetooth_thread(&self) -> IpcSender<BluetoothRequest> {
        self.global().as_window().bluetooth_thread()
    }

    pub fn get_device_map(&self) -> &DOMRefCell<HashMap<String, MutJS<BluetoothDevice>>> {
        &self.device_instance_map
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#request-bluetooth-devices
    fn request_bluetooth_devices(&self,
                                 p: &Rc<Promise>,
                                 filters: &Option<Vec<BluetoothLEScanFilterInit>>,
                                 optional_services: &Option<Vec<BluetoothServiceUUID>>) {
        // TODO: Step 1: Triggered by user activation.

        // Step 2.2: There are no requiredServiceUUIDS, we scan for all devices.
        let mut uuid_filters = vec!();

        if let &Some(ref filters) = filters {
            // Step 2.1.
            if filters.is_empty()  {
                p.reject_error(p.global().get_cx(), Type(FILTER_EMPTY_ERROR.to_owned()));
                return;
            }

            // Step 2.3: There are no requiredServiceUUIDS, we scan for all devices.

            // Step 2.4.
            for filter in filters {
                // Step 2.4.1.
                match canonicalize_filter(&filter) {
                    // Step 2.4.2.
                    Ok(f) => uuid_filters.push(f),
                    Err(e) => {
                        p.reject_error(p.global().get_cx(), e);
                        return;
                    },
                }
                // Step 2.4.3: There are no requiredServiceUUIDS, we scan for all devices.
            }
        }

        let mut optional_services_uuids = vec!();
        if let &Some(ref opt_services) = optional_services {
            for opt_service in opt_services {
                // Step 2.5 - 2.6.
                let uuid = match BluetoothUUID::service(opt_service.clone()) {
                    Ok(u) => u.to_string(),
                    Err(e) => {
                        p.reject_error(p.global().get_cx(), e);
                        return;
                    },
                };

                // Step 2.7.
                // Note: What we are doing here, is adding the not blocklisted UUIDs to the result vector,
                // instead of removing them from an already filled vector.
                if !uuid_is_blocklisted(uuid.as_ref(), Blocklist::All) {
                    optional_services_uuids.push(uuid);
                }
            }
        }

        let option = RequestDeviceoptions::new(BluetoothScanfilterSequence::new(uuid_filters),
                                               ServiceUUIDSequence::new(optional_services_uuids));

        // TODO: Step 3 - 5: Implement the permission API.

        // Note: Steps 6 - 8 are implemented in
        // components/net/bluetooth_thread.rs in request_device function.
        let sender = response_async(p, self);
        self.get_bluetooth_thread().send(BluetoothRequest::RequestDevice(option, sender)).unwrap();
    }
}

pub fn response_async<T: AsyncBluetoothListener + DomObject + 'static>(
        promise: &Rc<Promise>,
        receiver: &T) -> IpcSender<BluetoothResponseResult> {
    let (action_sender, action_receiver) = ipc::channel().unwrap();
    let task_source = receiver.global().networking_task_source();
    let context = Arc::new(Mutex::new(BluetoothContext {
        promise: Some(TrustedPromise::new(promise.clone())),
        receiver: Trusted::new(receiver),
    }));
    ROUTER.add_route(action_receiver.to_opaque(), box move |message| {
        struct ListenerRunnable<T: AsyncBluetoothListener + DomObject> {
            context: Arc<Mutex<BluetoothContext<T>>>,
            action: BluetoothResponseResult,
        }

        impl<T: AsyncBluetoothListener + DomObject> Runnable for ListenerRunnable<T> {
            fn handler(self: Box<Self>) {
                let this = *self;
                let mut context = this.context.lock().unwrap();
                context.response(this.action);
            }
        }

        let runnable = box ListenerRunnable {
            context: context.clone(),
            action: message.to().unwrap(),
        };

        let result = task_source.queue_wrapperless(runnable);
        if let Err(err) = result {
            warn!("failed to deliver network data: {:?}", err);
        }
    });
    action_sender
}

#[allow(unrooted_must_root)]
// https://webbluetoothcg.github.io/web-bluetooth/#getgattchildren
pub fn get_gatt_children<T, F> (
        attribute: &T,
        single: bool,
        uuid_canonicalizer: F,
        uuid: Option<StringOrUnsignedLong>,
        instance_id: String,
        connected: bool,
        child_type: GATTType)
        -> Rc<Promise>
        where T: AsyncBluetoothListener + DomObject + 'static,
              F: FnOnce(StringOrUnsignedLong) -> Fallible<UUID> {
    let p = Promise::new(&attribute.global());
    let p_cx = p.global().get_cx();

    let result_uuid = if let Some(u) = uuid {
        // Step 1.
        let canonicalized = match uuid_canonicalizer(u) {
            Ok(canonicalized_uuid) => canonicalized_uuid.to_string(),
            Err(e) => {
                p.reject_error(p_cx, e);
                return p;
            }
        };
        // Step 2.
        if uuid_is_blocklisted(canonicalized.as_ref(), Blocklist::All) {
            p.reject_error(p_cx, Security);
            return p;
        }
        Some(canonicalized)
    } else {
        None
    };

    // Step 3 - 4.
    if !connected {
        p.reject_error(p_cx, Network);
        return p;
    }

    // TODO: Step 5: Implement representedDevice internal slot for BluetoothDevice.

    // Note: Steps 6 - 7 are implemented in components/bluetooth/lib.rs in get_descriptor function
    // and in handle_response function.
    let sender = response_async(&p, attribute);
    attribute.global().as_window().bluetooth_thread().send(
        BluetoothRequest::GetGATTChildren(instance_id, result_uuid, single, child_type, sender)).unwrap();
    return p;
}

// https://webbluetoothcg.github.io/web-bluetooth/#bluetoothlescanfilterinit-canonicalizing
fn canonicalize_filter(filter: &BluetoothLEScanFilterInit) -> Fallible<BluetoothScanfilter> {
    // Step 1.
    if filter.services.is_none() &&
       filter.name.is_none() &&
       filter.namePrefix.is_none() &&
       filter.manufacturerData.is_none() &&
       filter.serviceData.is_none() {
           return Err(Type(FILTER_ERROR.to_owned()));
    }

    // Step 2: There is no empty canonicalizedFilter member,
    // we create a BluetoothScanfilter instance at the end of the function.

    // Step 3.
    let services_vec = match filter.services {
        Some(ref services) => {
            // Step 3.1.
            if services.is_empty() {
                return Err(Type(SERVICE_ERROR.to_owned()));
            }

            let mut services_vec = vec!();

            for service in services {
                // Step 3.2 - 3.3.
                let uuid = try!(BluetoothUUID::service(service.clone())).to_string();

                // Step 3.4.
                if uuid_is_blocklisted(uuid.as_ref(), Blocklist::All) {
                    return Err(Security)
                }

                services_vec.push(uuid);
            }
            // Step 3.5.
            services_vec
        },
        None => vec!(),
    };

    // Step 4.
    let name = match filter.name {
        Some(ref name) => {
            // Step 4.1.
            // Note: DOMString::len() gives back the size in bytes.
            if name.len() > MAX_DEVICE_NAME_LENGTH {
                return Err(Type(NAME_TOO_LONG_ERROR.to_owned()));
            }
            if name.len() > MAX_FILTER_NAME_LENGTH {
                return Err(NotFound);
            }

            // Step 4.2.
            Some(name.to_string())
        },
        None => None,
    };

    // Step 5.
    let name_prefix = match filter.namePrefix {
        Some(ref name_prefix) => {
            // Step 5.1.
            if name_prefix.is_empty() {
                return Err(Type(NAME_PREFIX_ERROR.to_owned()));
            }
            if name_prefix.len() > MAX_DEVICE_NAME_LENGTH {
                return Err(Type(NAME_TOO_LONG_ERROR.to_owned()));
            }
            if name_prefix.len() > MAX_FILTER_NAME_LENGTH {
                return Err(NotFound);
            }

            // Step 5.2.
            name_prefix.to_string()
        },
        None => String::new(),
    };

    // Step 6 - 7.
    let manufacturer_data = match filter.manufacturerData {
        Some(ref manufacturer_data_map) => {
            // Note: If manufacturer_data_map is empty, that means there are no key values in it.
            if manufacturer_data_map.is_empty() {
                return Err(Type(MANUFACTURER_DATA_ERROR.to_owned()));
            }
            let mut map = HashMap::new();
            for (key, bdfi) in manufacturer_data_map.iter() {
                // Step 7.1 - 7.2.
                let manufacturer_id = match u16::from_str(key.as_ref()) {
                    Ok(id) => id,
                    Err(err) => return Err(Type(format!("{} {} {}", KEY_CONVERSION_ERROR, key, err))),
                };

                // Step 7.3: No need to convert to IDL values since this is only used by native code.

                // Step 7.4 - 7.5.
                map.insert(manufacturer_id, try!(canonicalize_bluetooth_data_filter_init(bdfi)));
            }
            Some(map)
        },
        None => None,
    };

    // Step 8 - 9.
    let service_data = match filter.serviceData {
        Some(ref service_data_map) => {
            // Note: If service_data_map is empty, that means there are no key values in it.
            if service_data_map.is_empty() {
                return Err(Type(SERVICE_DATA_ERROR.to_owned()));
            }
            let mut map = HashMap::new();
            for (key, bdfi) in service_data_map.iter() {
                let service_name = match u32::from_str(key.as_ref()) {
                    // Step 9.1.
                    Ok(number) => StringOrUnsignedLong::UnsignedLong(number),
                    // Step 9.2.
                    _ => StringOrUnsignedLong::String(key.clone())
                };

                // Step 9.3 - 9.4.
                let service = try!(BluetoothUUID::service(service_name)).to_string();

                // Step 9.5.
                if uuid_is_blocklisted(service.as_ref(), Blocklist::All) {
                    return Err(Security);
                }

                // Step 9.6: No need to convert to IDL values since this is only used by native code.

                // Step 9.7 - 9.8.
                map.insert(service, try!(canonicalize_bluetooth_data_filter_init(bdfi)));
            }
            Some(map)
        },
        None => None,
    };

    // Step 10.
    Ok(BluetoothScanfilter::new(name, name_prefix, services_vec, manufacturer_data, service_data))
}

// https://webbluetoothcg.github.io/web-bluetooth/#bluetoothdatafilterinit-canonicalizing
fn canonicalize_bluetooth_data_filter_init(bdfi: &BluetoothDataFilterInit) -> Fallible<(Vec<u8>, Vec<u8>)> {
    // Step 1.
    let data_prefix = bdfi.dataPrefix.clone().unwrap_or(vec![]);

    // Step 2.
    // If no mask present, mask will be a sequence of 0xFF bytes the same length as dataPrefix.
    // Masking dataPrefix with this, leaves dataPrefix untouched.
    let mask = bdfi.mask.clone().unwrap_or(vec![0xFF; data_prefix.len()]);

    // Step 3.
    if mask.len() != data_prefix.len() {
        return Err(Type(MASK_LENGTH_ERROR.to_owned()));
    }

    // Step 4.
    Ok((data_prefix, mask))
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
        let p = Promise::new(&self.global());
        // Step 1.
        if (option.filters.is_some() && option.acceptAllDevices) ||
           (option.filters.is_none() && !option.acceptAllDevices) {
            p.reject_error(p.global().get_cx(), Error::Type(OPTIONS_ERROR.to_owned()));
            return p;
        }

        // Step 2.
        self.request_bluetooth_devices(&p, &option.filters, &option.optionalServices);
        //Note: Step 3 - 4. in response function, Step 5. in handle_response function.
        return p;
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetooth-onavailabilitychanged
    event_handler!(availabilitychanged, GetOnavailabilitychanged, SetOnavailabilitychanged);
}

impl AsyncBluetoothListener for Bluetooth {
    fn handle_response(&self, response: BluetoothResponse, promise_cx: *mut JSContext, promise: &Rc<Promise>) {
        match response {
            // https://webbluetoothcg.github.io/web-bluetooth/#request-bluetooth-devices
            // Step 13 - 14.
            BluetoothResponse::RequestDevice(device) => {
                let mut device_instance_map = self.device_instance_map.borrow_mut();
                if let Some(existing_device) = device_instance_map.get(&device.id.clone()) {
                    return promise.resolve_native(promise_cx, &existing_device.get());
                }
                let bt_device = BluetoothDevice::new(&self.global(),
                                                     DOMString::from(device.id.clone()),
                                                     device.name.map(DOMString::from),
                                                     &self);
                device_instance_map.insert(device.id, MutJS::new(&bt_device));
                // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetooth-requestdevice
                // Step 5.
                promise.resolve_native(promise_cx, &bt_device);
            },
            _ => promise.reject_error(promise_cx, Error::Type("Something went wrong...".to_owned())),
        }
    }
}
