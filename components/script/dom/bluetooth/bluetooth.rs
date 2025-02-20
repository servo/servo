/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use bluetooth_traits::{BluetoothError, BluetoothRequest, GATTType};
use bluetooth_traits::{BluetoothResponse, BluetoothResponseResult};
use bluetooth_traits::blocklist::{Blocklist, uuid_is_blocklisted};
use bluetooth_traits::scanfilter::{BluetoothScanfilter, BluetoothScanfilterSequence};
use bluetooth_traits::scanfilter::{RequestDeviceoptions, ServiceUUIDSequence};
use crate::realms::{AlreadyInRealm, InRealm};
use crate::conversions::Convert;
use crate::dom::bindings::cell::{DomRefCell, Ref};
use crate::dom::bindings::codegen::Bindings::BluetoothBinding::BluetoothDataFilterInit;
use crate::dom::bindings::codegen::Bindings::BluetoothBinding::{BluetoothMethods, RequestDeviceOptions};
use crate::dom::bindings::codegen::Bindings::BluetoothBinding::BluetoothLEScanFilterInit;
use crate::dom::bindings::codegen::Bindings::BluetoothPermissionResultBinding::BluetoothPermissionDescriptor;
use crate::dom::bindings::codegen::Bindings::BluetoothRemoteGATTServerBinding::BluetoothRemoteGATTServer_Binding::
BluetoothRemoteGATTServerMethods;
use crate::dom::bindings::codegen::Bindings::PermissionStatusBinding::{PermissionName, PermissionState};
use crate::dom::bindings::codegen::UnionTypes::{ArrayBufferViewOrArrayBuffer, StringOrUnsignedLong};
use crate::dom::bindings::error::Error::{self, Network, Security, Type};
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::refcounted::{Trusted, TrustedPromise};
use crate::dom::bindings::reflector::{DomGlobal, DomObject, reflect_dom_object};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::bluetoothdevice::BluetoothDevice;
use crate::dom::bluetoothpermissionresult::BluetoothPermissionResult;
use crate::dom::bluetoothuuid::{BluetoothServiceUUID, BluetoothUUID, UUID};
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::permissions::{descriptor_permission_state, PermissionAlgorithm};
use crate::dom::promise::Promise;
use crate::script_runtime::{CanGc, JSContext};
use crate::task::TaskOnce;
use dom_struct::dom_struct;
use ipc_channel::ipc::{self, IpcSender};
use ipc_channel::router::ROUTER;
use js::conversions::ConversionResult;
use js::jsapi::JSObject;
use js::jsval::{ObjectValue, UndefinedValue};
use profile_traits::ipc as ProfiledIpc;
use std::collections::HashMap;
use std::rc::Rc;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

const KEY_CONVERSION_ERROR: &str =
    "This `manufacturerData` key can not be parsed as unsigned short:";
const FILTER_EMPTY_ERROR: &str =
    "'filters' member, if present, must be nonempty to find any devices.";
const FILTER_ERROR: &str = "A filter must restrict the devices in some way.";
const MANUFACTURER_DATA_ERROR: &str =
    "'manufacturerData', if present, must be non-empty to filter devices.";
const MASK_LENGTH_ERROR: &str = "`mask`, if present, must have the same length as `dataPrefix`.";
// 248 is the maximum number of UTF-8 code units in a Bluetooth Device Name.
const MAX_DEVICE_NAME_LENGTH: usize = 248;
const NAME_PREFIX_ERROR: &str = "'namePrefix', if present, must be nonempty.";
const NAME_TOO_LONG_ERROR: &str = "A device name can't be longer than 248 bytes.";
const SERVICE_DATA_ERROR: &str = "'serviceData', if present, must be non-empty to filter devices.";
const SERVICE_ERROR: &str = "'services', if present, must contain at least one service.";
const OPTIONS_ERROR: &str = "Fields of 'options' conflict with each other.
 Either 'acceptAllDevices' member must be true, or 'filters' member must be set to a value.";
const BT_DESC_CONVERSION_ERROR: &str =
    "Can't convert to an IDL value of type BluetoothPermissionDescriptor";

#[derive(JSTraceable, MallocSizeOf)]
#[allow(non_snake_case)]
pub(crate) struct AllowedBluetoothDevice {
    pub(crate) deviceId: DOMString,
    pub(crate) mayUseGATT: bool,
}

#[derive(JSTraceable, MallocSizeOf)]
pub(crate) struct BluetoothExtraPermissionData {
    allowed_devices: DomRefCell<Vec<AllowedBluetoothDevice>>,
}

impl BluetoothExtraPermissionData {
    pub(crate) fn new() -> BluetoothExtraPermissionData {
        BluetoothExtraPermissionData {
            allowed_devices: DomRefCell::new(Vec::new()),
        }
    }

    pub(crate) fn add_new_allowed_device(&self, allowed_device: AllowedBluetoothDevice) {
        self.allowed_devices.borrow_mut().push(allowed_device);
    }

    fn get_allowed_devices(&self) -> Ref<Vec<AllowedBluetoothDevice>> {
        self.allowed_devices.borrow()
    }

    pub(crate) fn allowed_devices_contains_id(&self, id: DOMString) -> bool {
        self.allowed_devices
            .borrow()
            .iter()
            .any(|d| d.deviceId == id)
    }
}

impl Default for BluetoothExtraPermissionData {
    fn default() -> Self {
        Self::new()
    }
}

struct BluetoothContext<T: AsyncBluetoothListener + DomObject> {
    promise: Option<TrustedPromise>,
    receiver: Trusted<T>,
}

pub(crate) trait AsyncBluetoothListener {
    fn handle_response(&self, result: BluetoothResponse, promise: &Rc<Promise>, can_gc: CanGc);
}

impl<T> BluetoothContext<T>
where
    T: AsyncBluetoothListener + DomObject,
{
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    fn response(&mut self, response: BluetoothResponseResult, can_gc: CanGc) {
        let promise = self.promise.take().expect("bt promise is missing").root();

        // JSAutoRealm needs to be manually made.
        // Otherwise, Servo will crash.
        match response {
            Ok(response) => self
                .receiver
                .root()
                .handle_response(response, &promise, can_gc),
            // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetooth-requestdevice
            // Step 3 - 4.
            Err(error) => promise.reject_error(error.convert()),
        }
    }
}

// https://webbluetoothcg.github.io/web-bluetooth/#bluetooth
#[dom_struct]
pub(crate) struct Bluetooth {
    eventtarget: EventTarget,
    device_instance_map: DomRefCell<HashMap<String, Dom<BluetoothDevice>>>,
}

impl Bluetooth {
    pub(crate) fn new_inherited() -> Bluetooth {
        Bluetooth {
            eventtarget: EventTarget::new_inherited(),
            device_instance_map: DomRefCell::new(HashMap::new()),
        }
    }

    pub(crate) fn new(global: &GlobalScope, can_gc: CanGc) -> DomRoot<Bluetooth> {
        reflect_dom_object(Box::new(Bluetooth::new_inherited()), global, can_gc)
    }

    fn get_bluetooth_thread(&self) -> IpcSender<BluetoothRequest> {
        self.global().as_window().bluetooth_thread()
    }

    pub(crate) fn get_device_map(&self) -> &DomRefCell<HashMap<String, Dom<BluetoothDevice>>> {
        &self.device_instance_map
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#request-bluetooth-devices
    fn request_bluetooth_devices(
        &self,
        p: &Rc<Promise>,
        filters: &Option<Vec<BluetoothLEScanFilterInit>>,
        optional_services: &[BluetoothServiceUUID],
        sender: IpcSender<BluetoothResponseResult>,
    ) {
        // TODO: Step 1: Triggered by user activation.

        // Step 2.2: There are no requiredServiceUUIDS, we scan for all devices.
        let mut uuid_filters = vec![];

        if let Some(filters) = filters {
            // Step 2.1.
            if filters.is_empty() {
                p.reject_error(Type(FILTER_EMPTY_ERROR.to_owned()));
                return;
            }

            // Step 2.3: There are no requiredServiceUUIDS, we scan for all devices.

            // Step 2.4.
            for filter in filters {
                // Step 2.4.1.
                match canonicalize_filter(filter) {
                    // Step 2.4.2.
                    Ok(f) => uuid_filters.push(f),
                    Err(e) => {
                        p.reject_error(e);
                        return;
                    },
                }
                // Step 2.4.3: There are no requiredServiceUUIDS, we scan for all devices.
            }
        }

        let mut optional_services_uuids = vec![];
        for opt_service in optional_services {
            // Step 2.5 - 2.6.
            let uuid = match BluetoothUUID::service(opt_service.clone()) {
                Ok(u) => u.to_string(),
                Err(e) => {
                    p.reject_error(e);
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

        let option = RequestDeviceoptions::new(
            self.global().as_window().webview_id(),
            BluetoothScanfilterSequence::new(uuid_filters),
            ServiceUUIDSequence::new(optional_services_uuids),
        );

        // Step 4 - 5.
        if let PermissionState::Denied =
            descriptor_permission_state(PermissionName::Bluetooth, None)
        {
            return p.reject_error(Error::NotFound);
        }

        // Note: Step 3, 6 - 8 are implemented in
        // components/net/bluetooth_thread.rs in request_device function.
        self.get_bluetooth_thread()
            .send(BluetoothRequest::RequestDevice(option, sender))
            .unwrap();
    }
}

pub(crate) fn response_async<T: AsyncBluetoothListener + DomObject + 'static>(
    promise: &Rc<Promise>,
    receiver: &T,
) -> IpcSender<BluetoothResponseResult> {
    let (action_sender, action_receiver) = ipc::channel().unwrap();
    let task_source = receiver
        .global()
        .task_manager()
        .networking_task_source()
        .to_sendable();
    let context = Arc::new(Mutex::new(BluetoothContext {
        promise: Some(TrustedPromise::new(promise.clone())),
        receiver: Trusted::new(receiver),
    }));
    ROUTER.add_typed_route(
        action_receiver,
        Box::new(move |message| {
            struct ListenerTask<T: AsyncBluetoothListener + DomObject> {
                context: Arc<Mutex<BluetoothContext<T>>>,
                action: BluetoothResponseResult,
            }

            impl<T> TaskOnce for ListenerTask<T>
            where
                T: AsyncBluetoothListener + DomObject,
            {
                fn run_once(self) {
                    let mut context = self.context.lock().unwrap();
                    context.response(self.action, CanGc::note());
                }
            }

            let task = ListenerTask {
                context: context.clone(),
                action: message.unwrap(),
            };

            task_source.queue_unconditionally(task);
        }),
    );
    action_sender
}

// https://webbluetoothcg.github.io/web-bluetooth/#getgattchildren
#[allow(clippy::too_many_arguments)]
pub(crate) fn get_gatt_children<T, F>(
    attribute: &T,
    single: bool,
    uuid_canonicalizer: F,
    uuid: Option<StringOrUnsignedLong>,
    instance_id: String,
    connected: bool,
    child_type: GATTType,
    can_gc: CanGc,
) -> Rc<Promise>
where
    T: AsyncBluetoothListener + DomObject + 'static,
    F: FnOnce(StringOrUnsignedLong) -> Fallible<UUID>,
{
    let in_realm_proof = AlreadyInRealm::assert();
    let p = Promise::new_in_current_realm(InRealm::Already(&in_realm_proof), can_gc);

    let result_uuid = if let Some(u) = uuid {
        // Step 1.
        let canonicalized = match uuid_canonicalizer(u) {
            Ok(canonicalized_uuid) => canonicalized_uuid.to_string(),
            Err(e) => {
                p.reject_error(e);
                return p;
            },
        };
        // Step 2.
        if uuid_is_blocklisted(canonicalized.as_ref(), Blocklist::All) {
            p.reject_error(Security);
            return p;
        }
        Some(canonicalized)
    } else {
        None
    };

    // Step 3 - 4.
    if !connected {
        p.reject_error(Network);
        return p;
    }

    // TODO: Step 5: Implement representedDevice internal slot for BluetoothDevice.

    // Note: Steps 6 - 7 are implemented in components/bluetooth/lib.rs in get_descriptor function
    // and in handle_response function.
    let sender = response_async(&p, attribute);
    attribute
        .global()
        .as_window()
        .bluetooth_thread()
        .send(BluetoothRequest::GetGATTChildren(
            instance_id,
            result_uuid,
            single,
            child_type,
            sender,
        ))
        .unwrap();
    p
}

// https://webbluetoothcg.github.io/web-bluetooth/#bluetoothlescanfilterinit-canonicalizing
fn canonicalize_filter(filter: &BluetoothLEScanFilterInit) -> Fallible<BluetoothScanfilter> {
    // Step 1.
    if filter.services.is_none() &&
        filter.name.is_none() &&
        filter.namePrefix.is_none() &&
        filter.manufacturerData.is_none() &&
        filter.serviceData.is_none()
    {
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

            let mut services_vec = vec![];

            for service in services {
                // Step 3.2 - 3.3.
                let uuid = BluetoothUUID::service(service.clone())?.to_string();

                // Step 3.4.
                if uuid_is_blocklisted(uuid.as_ref(), Blocklist::All) {
                    return Err(Security);
                }

                services_vec.push(uuid);
            }
            // Step 3.5.
            services_vec
        },
        None => vec![],
    };

    // Step 4.
    let name = match filter.name {
        Some(ref name) => {
            // Step 4.1.
            // Note: DOMString::len() gives back the size in bytes.
            if name.len() > MAX_DEVICE_NAME_LENGTH {
                return Err(Type(NAME_TOO_LONG_ERROR.to_owned()));
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
                    Err(err) => {
                        return Err(Type(format!("{} {} {}", KEY_CONVERSION_ERROR, key, err)));
                    },
                };

                // Step 7.3: No need to convert to IDL values since this is only used by native code.

                // Step 7.4 - 7.5.
                map.insert(
                    manufacturer_id,
                    canonicalize_bluetooth_data_filter_init(bdfi)?,
                );
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
                    _ => StringOrUnsignedLong::String(key.clone()),
                };

                // Step 9.3 - 9.4.
                let service = BluetoothUUID::service(service_name)?.to_string();

                // Step 9.5.
                if uuid_is_blocklisted(service.as_ref(), Blocklist::All) {
                    return Err(Security);
                }

                // Step 9.6: No need to convert to IDL values since this is only used by native code.

                // Step 9.7 - 9.8.
                map.insert(service, canonicalize_bluetooth_data_filter_init(bdfi)?);
            }
            Some(map)
        },
        None => None,
    };

    // Step 10.
    Ok(BluetoothScanfilter::new(
        name,
        name_prefix,
        services_vec,
        manufacturer_data,
        service_data,
    ))
}

// https://webbluetoothcg.github.io/web-bluetooth/#bluetoothdatafilterinit-canonicalizing
fn canonicalize_bluetooth_data_filter_init(
    bdfi: &BluetoothDataFilterInit,
) -> Fallible<(Vec<u8>, Vec<u8>)> {
    // Step 1.
    let data_prefix = match bdfi.dataPrefix {
        Some(ArrayBufferViewOrArrayBuffer::ArrayBufferView(ref avb)) => avb.to_vec(),
        Some(ArrayBufferViewOrArrayBuffer::ArrayBuffer(ref ab)) => ab.to_vec(),
        None => vec![],
    };

    // Step 2.
    // If no mask present, mask will be a sequence of 0xFF bytes the same length as dataPrefix.
    // Masking dataPrefix with this, leaves dataPrefix untouched.
    let mask = match bdfi.mask {
        Some(ArrayBufferViewOrArrayBuffer::ArrayBufferView(ref avb)) => avb.to_vec(),
        Some(ArrayBufferViewOrArrayBuffer::ArrayBuffer(ref ab)) => ab.to_vec(),
        None => vec![0xFF; data_prefix.len()],
    };

    // Step 3.
    if mask.len() != data_prefix.len() {
        return Err(Type(MASK_LENGTH_ERROR.to_owned()));
    }

    // Step 4.
    Ok((data_prefix, mask))
}

impl Convert<Error> for BluetoothError {
    fn convert(self) -> Error {
        match self {
            BluetoothError::Type(message) => Error::Type(message),
            BluetoothError::Network => Error::Network,
            BluetoothError::NotFound => Error::NotFound,
            BluetoothError::NotSupported => Error::NotSupported,
            BluetoothError::Security => Error::Security,
            BluetoothError::InvalidState => Error::InvalidState,
        }
    }
}

impl BluetoothMethods<crate::DomTypeHolder> for Bluetooth {
    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetooth-requestdevice
    fn RequestDevice(
        &self,
        option: &RequestDeviceOptions,
        comp: InRealm,
        can_gc: CanGc,
    ) -> Rc<Promise> {
        let p = Promise::new_in_current_realm(comp, can_gc);
        // Step 1.
        if (option.filters.is_some() && option.acceptAllDevices) ||
            (option.filters.is_none() && !option.acceptAllDevices)
        {
            p.reject_error(Error::Type(OPTIONS_ERROR.to_owned()));
            return p;
        }

        // Step 2.
        let sender = response_async(&p, self);
        self.request_bluetooth_devices(&p, &option.filters, &option.optionalServices, sender);
        //Note: Step 3 - 4. in response function, Step 5. in handle_response function.
        p
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetooth-getavailability
    fn GetAvailability(&self, comp: InRealm, can_gc: CanGc) -> Rc<Promise> {
        let p = Promise::new_in_current_realm(comp, can_gc);
        // Step 1. We did not override the method
        // Step 2 - 3. in handle_response
        let sender = response_async(&p, self);
        self.get_bluetooth_thread()
            .send(BluetoothRequest::GetAvailability(sender))
            .unwrap();
        p
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetooth-onavailabilitychanged
    event_handler!(
        availabilitychanged,
        GetOnavailabilitychanged,
        SetOnavailabilitychanged
    );
}

impl AsyncBluetoothListener for Bluetooth {
    fn handle_response(&self, response: BluetoothResponse, promise: &Rc<Promise>, can_gc: CanGc) {
        match response {
            // https://webbluetoothcg.github.io/web-bluetooth/#request-bluetooth-devices
            // Step 11, 13 - 14.
            BluetoothResponse::RequestDevice(device) => {
                let mut device_instance_map = self.device_instance_map.borrow_mut();
                if let Some(existing_device) = device_instance_map.get(&device.id.clone()) {
                    return promise.resolve_native(&**existing_device);
                }
                let bt_device = BluetoothDevice::new(
                    &self.global(),
                    DOMString::from(device.id.clone()),
                    device.name.map(DOMString::from),
                    self,
                    can_gc,
                );
                device_instance_map.insert(device.id.clone(), Dom::from_ref(&bt_device));

                self.global()
                    .as_window()
                    .bluetooth_extra_permission_data()
                    .add_new_allowed_device(AllowedBluetoothDevice {
                        deviceId: DOMString::from(device.id),
                        mayUseGATT: true,
                    });
                // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetooth-requestdevice
                // Step 5.
                promise.resolve_native(&bt_device);
            },
            // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetooth-getavailability
            // Step 2 - 3.
            BluetoothResponse::GetAvailability(is_available) => {
                promise.resolve_native(&is_available);
            },
            _ => promise.reject_error(Error::Type("Something went wrong...".to_owned())),
        }
    }
}

impl PermissionAlgorithm for Bluetooth {
    type Descriptor = BluetoothPermissionDescriptor;
    type Status = BluetoothPermissionResult;

    fn create_descriptor(
        cx: JSContext,
        permission_descriptor_obj: *mut JSObject,
    ) -> Result<BluetoothPermissionDescriptor, Error> {
        rooted!(in(*cx) let mut property = UndefinedValue());
        property
            .handle_mut()
            .set(ObjectValue(permission_descriptor_obj));
        match BluetoothPermissionDescriptor::new(cx, property.handle()) {
            Ok(ConversionResult::Success(descriptor)) => Ok(descriptor),
            Ok(ConversionResult::Failure(error)) => Err(Error::Type(error.into_owned())),
            Err(_) => Err(Error::Type(String::from(BT_DESC_CONVERSION_ERROR))),
        }
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#query-the-bluetooth-permission
    fn permission_query(
        _cx: JSContext,
        promise: &Rc<Promise>,
        descriptor: &BluetoothPermissionDescriptor,
        status: &BluetoothPermissionResult,
    ) {
        // Step 1: We are not using the `global` variable.

        // Step 2.
        status.set_state(descriptor_permission_state(status.get_query(), None));

        // Step 3.
        if let PermissionState::Denied = status.get_state() {
            status.set_devices(Vec::new());
            return promise.resolve_native(status);
        }

        // Step 4.
        rooted_vec!(let mut matching_devices);

        // Step 5.
        let global = status.global();
        let allowed_devices = global
            .as_window()
            .bluetooth_extra_permission_data()
            .get_allowed_devices();

        let bluetooth = status.get_bluetooth();
        let device_map = bluetooth.get_device_map().borrow();

        // Step 6.
        for allowed_device in allowed_devices.iter() {
            // Step 6.1.
            if let Some(ref id) = descriptor.deviceId {
                if &allowed_device.deviceId != id {
                    continue;
                }
            }
            let device_id = String::from(allowed_device.deviceId.as_ref());

            // Step 6.2.
            if let Some(ref filters) = descriptor.filters {
                let mut scan_filters: Vec<BluetoothScanfilter> = Vec::new();

                // Step 6.2.1.
                for filter in filters {
                    match canonicalize_filter(filter) {
                        Ok(f) => scan_filters.push(f),
                        Err(error) => return promise.reject_error(error),
                    }
                }

                // Step 6.2.2.
                // Instead of creating an internal slot we send an ipc message to the Bluetooth thread
                // to check if one of the filters matches.
                let (sender, receiver) =
                    ProfiledIpc::channel(global.time_profiler_chan().clone()).unwrap();
                status
                    .get_bluetooth_thread()
                    .send(BluetoothRequest::MatchesFilter(
                        device_id.clone(),
                        BluetoothScanfilterSequence::new(scan_filters),
                        sender,
                    ))
                    .unwrap();

                match receiver.recv().unwrap() {
                    Ok(true) => (),
                    Ok(false) => continue,
                    Err(error) => return promise.reject_error(error.convert()),
                };
            }

            // Step 6.3.
            // TODO: Implement this correctly, not just using device ids here.
            // https://webbluetoothcg.github.io/web-bluetooth/#get-the-bluetoothdevice-representing
            if let Some(device) = device_map.get(&device_id) {
                matching_devices.push(Dom::from_ref(&**device));
            }
        }

        // Step 7.
        status.set_devices(matching_devices.drain(..).collect());

        // https://w3c.github.io/permissions/#dom-permissions-query
        // Step 7.
        promise.resolve_native(status);
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#request-the-bluetooth-permission
    fn permission_request(
        _cx: JSContext,
        promise: &Rc<Promise>,
        descriptor: &BluetoothPermissionDescriptor,
        status: &BluetoothPermissionResult,
    ) {
        // Step 1.
        if descriptor.filters.is_some() == descriptor.acceptAllDevices {
            return promise.reject_error(Error::Type(OPTIONS_ERROR.to_owned()));
        }

        // Step 2.
        let sender = response_async(promise, status);
        let bluetooth = status.get_bluetooth();
        bluetooth.request_bluetooth_devices(
            promise,
            &descriptor.filters,
            &descriptor.optionalServices,
            sender,
        );

        // NOTE: Step 3. is in BluetoothPermissionResult's `handle_response` function.
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    // https://webbluetoothcg.github.io/web-bluetooth/#revoke-bluetooth-access
    fn permission_revoke(
        _descriptor: &BluetoothPermissionDescriptor,
        status: &BluetoothPermissionResult,
        can_gc: CanGc,
    ) {
        // Step 1.
        let global = status.global();
        let allowed_devices = global
            .as_window()
            .bluetooth_extra_permission_data()
            .get_allowed_devices();
        // Step 2.
        let bluetooth = status.get_bluetooth();
        let device_map = bluetooth.get_device_map().borrow();
        for (id, device) in device_map.iter() {
            let id = DOMString::from(id.clone());
            // Step 2.1.
            if allowed_devices.iter().any(|d| d.deviceId == id) &&
                !device.is_represented_device_null()
            {
                // Note: We don't need to update the allowed_services,
                // because we store it in the lower level
                // where it is already up-to-date
                continue;
            }
            // Step 2.2 - 2.4
            let _ = device.get_gatt().Disconnect(can_gc);
        }
    }
}
