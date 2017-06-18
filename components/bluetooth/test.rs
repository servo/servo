/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use BluetoothManager;
use device::bluetooth::{BluetoothAdapter, BluetoothDevice};
use device::bluetooth::{BluetoothGATTCharacteristic, BluetoothGATTDescriptor, BluetoothGATTService};
use std::borrow::ToOwned;
use std::cell::RefCell;
use std::collections::{HashSet, HashMap};
use std::error::Error;
use std::string::String;
use uuid::Uuid;

thread_local!(pub static CACHED_IDS: RefCell<HashSet<Uuid>> = RefCell::new(HashSet::new()));

const ADAPTER_ERROR: &'static str = "No adapter found";
const WRONG_DATA_SET_ERROR: &'static str = "Wrong data set name was provided";
const READ_FLAG: &'static str = "read";
const WRITE_FLAG: &'static str = "write";
const NOTIFY_FLAG: &'static str = "notify";

// Adapter names
// https://cs.chromium.org/chromium/src/content/shell/browser/layout_test/layout_test_bluetooth_adapter_provider.h?l=65
const NOT_PRESENT_ADAPTER: &'static str = "NotPresentAdapter";
// https://cs.chromium.org/chromium/src/content/shell/browser/layout_test/layout_test_bluetooth_adapter_provider.h?l=83
const NOT_POWERED_ADAPTER: &'static str = "NotPoweredAdapter";
// https://cs.chromium.org/chromium/src/content/shell/browser/layout_test/layout_test_bluetooth_adapter_provider.h?l=118
const EMPTY_ADAPTER: &'static str = "EmptyAdapter";
// https://cs.chromium.org/chromium/src/content/shell/browser/layout_test/layout_test_bluetooth_adapter_provider.h?l=126
const GLUCOSE_HEART_RATE_ADAPTER: &'static str = "GlucoseHeartRateAdapter";
// https://cs.chromium.org/chromium/src/content/shell/browser/layout_test/layout_test_bluetooth_adapter_provider.h?l=135
const UNICODE_DEVICE_ADAPTER: &'static str = "UnicodeDeviceAdapter";
// https://cs.chromium.org/chromium/src/content/shell/browser/layout_test/layout_test_bluetooth_adapter_provider.h?l=205
const MISSING_SERVICE_HEART_RATE_ADAPTER: &'static str = "MissingServiceHeartRateAdapter";
// https://cs.chromium.org/chromium/src/content/shell/browser/layout_test/layout_test_bluetooth_adapter_provider.h?l=219
const MISSING_CHARACTERISTIC_HEART_RATE_ADAPTER: &'static str = "MissingCharacteristicHeartRateAdapter";
const MISSING_DESCRIPTOR_HEART_RATE_ADAPTER: &'static str = "MissingDescriptorHeartRateAdapter";
// https://cs.chromium.org/chromium/src/content/shell/browser/layout_test/layout_test_bluetooth_adapter_provider.h?l=234
const HEART_RATE_ADAPTER: &'static str = "HeartRateAdapter";
// https://cs.chromium.org/chromium/src/content/shell/browser/layout_test/layout_test_bluetooth_adapter_provider.h?l=250
const EMPTY_NAME_HEART_RATE_ADAPTER: &'static str = "EmptyNameHeartRateAdapter";
// https://cs.chromium.org/chromium/src/content/shell/browser/layout_test/layout_test_bluetooth_adapter_provider.h?l=267
const NO_NAME_HEART_RATE_ADAPTER: &'static str = "NoNameHeartRateAdapter";
// https://cs.chromium.org/chromium/src/content/shell/browser/layout_test/layout_test_bluetooth_adapter_provider.h?l=284
const TWO_HEART_RATE_SERVICES_ADAPTER: &'static str = "TwoHeartRateServicesAdapter";
const BLOCKLIST_TEST_ADAPTER: &'static str = "BlocklistTestAdapter";

// Device names
const CONNECTABLE_DEVICE_NAME: &'static str = "Connectable Device";
const EMPTY_DEVICE_NAME: &'static str = "";
// https://webbluetoothcg.github.io/web-bluetooth/tests.html#glucosedevice
const GLUCOSE_DEVICE_NAME: &'static str = "Glucose Device";
// https://webbluetoothcg.github.io/web-bluetooth/tests.html#heartratedevice
const HEART_RATE_DEVICE_NAME: &'static str = "Heart Rate Device";
const UNICODE_DEVICE_NAME: &'static str = "❤❤❤❤❤❤❤❤❤";

// Device addresses
const CONNECTABLE_DEVICE_ADDRESS: &'static str = "00:00:00:00:00:04";
// https://webbluetoothcg.github.io/web-bluetooth/tests.html#glucosedevice
const GLUCOSE_DEVICE_ADDRESS: &'static str = "00:00:00:00:00:02";
// https://webbluetoothcg.github.io/web-bluetooth/tests.html#heartratedevice
const HEART_RATE_DEVICE_ADDRESS: &'static str = "00:00:00:00:00:03";
const UNICODE_DEVICE_ADDRESS: &'static str = "00:00:00:00:00:01";

// Service UUIDs
const BLOCKLIST_TEST_SERVICE_UUID: &'static str = "611c954a-263b-4f4a-aab6-01ddb953f985";
// https://www.bluetooth.com/specifications/gatt/viewer?attributeXmlFile=org.bluetooth.service.device_information.xml
const DEVICE_INFORMATION_UUID: &'static str = "0000180a-0000-1000-8000-00805f9b34fb";
// https://www.bluetooth.com/specifications/gatt/viewer?attributeXmlFile=org.bluetooth.service.generic_access.xml
const GENERIC_ACCESS_SERVICE_UUID: &'static str = "00001800-0000-1000-8000-00805f9b34fb";
// https://www.bluetooth.com/specifications/gatt/viewer?attributeXmlFile=org.bluetooth.service.glucose.xml
const GLUCOSE_SERVICE_UUID: &'static str = "00001808-0000-1000-8000-00805f9b34fb";
// https://www.bluetooth.com/specifications/gatt/viewer?attributeXmlFile=org.bluetooth.service.heart_rate.xml
const HEART_RATE_SERVICE_UUID: &'static str = "0000180d-0000-1000-8000-00805f9b34fb";
// https://www.bluetooth.com/specifications/gatt/
// viewer?attributeXmlFile=org.bluetooth.service.human_interface_device.xml
const HUMAN_INTERFACE_DEVICE_SERVICE_UUID: &'static str = "00001812-0000-1000-8000-00805f9b34fb";
// https://www.bluetooth.com/specifications/gatt/viewer?attributeXmlFile=org.bluetooth.service.tx_power.xml
const TX_POWER_SERVICE_UUID: &'static str = "00001804-0000-1000-8000-00805f9b34fb";

// Characteristic UUIDs
const BLOCKLIST_EXCLUDE_READS_CHARACTERISTIC_UUID: &'static str = "bad1c9a2-9a5b-4015-8b60-1579bbbf2135";
// https://www.bluetooth.com/specifications/gatt/
// viewer?attributeXmlFile=org.bluetooth.characteristic.body_sensor_location.xml
const BODY_SENSOR_LOCATION_CHARACTERISTIC_UUID: &'static str = "00002a38-0000-1000-8000-00805f9b34fb";
// https://www.bluetooth.com/specifications/gatt/
// viewer?attributeXmlFile=org.bluetooth.characteristic.gap.device_name.xml
const DEVICE_NAME_CHARACTERISTIC_UUID: &'static str = "00002a00-0000-1000-8000-00805f9b34fb";
// https://www.bluetooth.com/specifications/gatt/
// viewer?attributeXmlFile=org.bluetooth.characteristic.heart_rate_measurement.xml
const HEART_RATE_MEASUREMENT_CHARACTERISTIC_UUID: &'static str = "00002a37-0000-1000-8000-00805f9b34fb";
// https://www.bluetooth.com/specifications/gatt/
// viewer?attributeXmlFile=org.bluetooth.characteristic.gap.peripheral_privacy_flag.xml
const PERIPHERAL_PRIVACY_FLAG_CHARACTERISTIC_UUID: &'static str = "00002a02-0000-1000-8000-00805f9b34fb";
// https://www.bluetooth.com/specifications/gatt/
// viewer?attributeXmlFile=org.bluetooth.characteristic.serial_number_string.xml
const SERIAL_NUMBER_STRING_UUID: &'static str = "00002a25-0000-1000-8000-00805f9b34fb";

// Descriptor UUIDs
const BLOCKLIST_EXCLUDE_READS_DESCRIPTOR_UUID: &'static str = "aaaaaaaa-aaaa-1181-0510-810819516110";
const BLOCKLIST_DESCRIPTOR_UUID: &'static str = "07711111-6104-0970-7011-1107105110aa";
// https://www.bluetooth.com/specifications/gatt/
// viewer?attributeXmlFile=org.bluetooth.descriptor.gatt.characteristic_user_description.xml
const CHARACTERISTIC_USER_DESCRIPTION_UUID: &'static str = "00002901-0000-1000-8000-00805f9b34fb";
// https://www.bluetooth.com/specifications/gatt/
// viewer?attributeXmlFile=org.bluetooth.descriptor.gatt.client_characteristic_configuration.xml
const CLIENT_CHARACTERISTIC_CONFIGURATION_UUID: &'static str = "00002902-0000-1000-8000-00805f9b34fb";
// https://www.bluetooth.com/specifications/gatt/
// viewer?attributeXmlFile=org.bluetooth.descriptor.number_of_digitals.xml
const NUMBER_OF_DIGITALS_UUID: &'static str = "00002909-0000-1000-8000-00805f9b34fb";

const HEART_RATE_DEVICE_NAME_DESCRIPTION: &'static str = "The name of this device.";

fn generate_id() -> Uuid {
    let mut id = Uuid::nil();
    let mut generated = false;
    while !generated {
        id = Uuid::new_v4();
        CACHED_IDS.with(|cache|
            if !cache.borrow().contains(&id) {
                cache.borrow_mut().insert(id.clone());
                generated = true;
            }
        );
    }
    id
}

// Set the adapter's name, is_powered and is_discoverable attributes
fn set_adapter(adapter: &BluetoothAdapter, adapter_name: String) -> Result<(), Box<Error>> {
    adapter.set_name(adapter_name)?;
    adapter.set_powered(true)?;
    adapter.set_discoverable(true)?;
    Ok(())
}

// Create Device
fn create_device(adapter: &BluetoothAdapter,
                 name: String,
                 address: String)
                 -> Result<BluetoothDevice, Box<Error>> {
    let device = BluetoothDevice::create_mock_device(adapter.clone(), generate_id().to_string())?;
    device.set_name(Some(name))?;
    device.set_address(address)?;
    device.set_connectable(true)?;
    Ok(device)
}

// Create Device with UUIDs
fn create_device_with_uuids(adapter: &BluetoothAdapter,
                            name: String,
                            address: String,
                            uuids: Vec<String>)
                            -> Result<BluetoothDevice, Box<Error>> {
    let device = create_device(adapter, name, address)?;
    device.set_uuids(uuids)?;
    Ok(device)
}

// Create Service
fn create_service(device: &BluetoothDevice,
                  uuid: String)
                  -> Result<BluetoothGATTService, Box<Error>> {
    let service = BluetoothGATTService::create_mock_service(device.clone(), generate_id().to_string())?;
    service.set_uuid(uuid)?;
    Ok(service)
}

// Create Characteristic
fn create_characteristic(service: &BluetoothGATTService,
                         uuid: String)
                         -> Result<BluetoothGATTCharacteristic, Box<Error>> {
    let characteristic =
        BluetoothGATTCharacteristic::create_mock_characteristic(service.clone(), generate_id().to_string())?;
    characteristic.set_uuid(uuid)?;
    Ok(characteristic)
}

// Create Characteristic with value
fn create_characteristic_with_value(service: &BluetoothGATTService,
                                    uuid: String,
                                    value: Vec<u8>)
                                    -> Result<BluetoothGATTCharacteristic, Box<Error>> {
    let characteristic = create_characteristic(service, uuid)?;
    characteristic.set_value(value)?;
    Ok(characteristic)
}

// Create Descriptor
fn create_descriptor(characteristic: &BluetoothGATTCharacteristic,
                                     uuid: String)
                                     -> Result<BluetoothGATTDescriptor, Box<Error>> {
    let descriptor =
        BluetoothGATTDescriptor::create_mock_descriptor(characteristic.clone(), generate_id().to_string())?;
    descriptor.set_uuid(uuid)?;
    Ok(descriptor)
}

// Create Descriptor with value
fn create_descriptor_with_value(characteristic: &BluetoothGATTCharacteristic,
                                uuid: String,
                                value: Vec<u8>)
                                -> Result<BluetoothGATTDescriptor, Box<Error>> {
    let descriptor = create_descriptor(characteristic, uuid)?;
    descriptor.set_value(value)?;
    Ok(descriptor)
}

fn create_heart_rate_service(device: &BluetoothDevice,
                             empty: bool)
                             -> Result<BluetoothGATTService, Box<Error>> {
    // Heart Rate Service
    let heart_rate_service = create_service(device, HEART_RATE_SERVICE_UUID.to_owned())?;

    if empty {
        return Ok(heart_rate_service)
    }

    // Heart Rate Measurement Characteristic
    let heart_rate_measurement_characteristic =
        create_characteristic_with_value(&heart_rate_service,
                                              HEART_RATE_MEASUREMENT_CHARACTERISTIC_UUID.to_owned(),
                                              vec![0])?;
    heart_rate_measurement_characteristic.set_flags(vec![NOTIFY_FLAG.to_string(),
                                                              READ_FLAG.to_string(),
                                                              WRITE_FLAG.to_string()])?;

    // Body Sensor Location Characteristic 1
    let body_sensor_location_characteristic_1 =
        create_characteristic_with_value(&heart_rate_service,
                                              BODY_SENSOR_LOCATION_CHARACTERISTIC_UUID.to_owned(),
                                              vec![49])?;
    body_sensor_location_characteristic_1.set_flags(vec![READ_FLAG.to_string(), WRITE_FLAG.to_string()])?;

    // Body Sensor Location Characteristic 2
    let body_sensor_location_characteristic_2 =
        create_characteristic_with_value(&heart_rate_service,
                                              BODY_SENSOR_LOCATION_CHARACTERISTIC_UUID.to_owned(),
                                              vec![50])?;
    body_sensor_location_characteristic_2.set_flags(vec![READ_FLAG.to_string(), WRITE_FLAG.to_string()])?;
    Ok(heart_rate_service)
}

fn create_generic_access_service(device: &BluetoothDevice,
                                 empty: bool)
                                 -> Result<BluetoothGATTService, Box<Error>> {
    // Generic Access Service
    let generic_access_service =
        create_service(device, GENERIC_ACCESS_SERVICE_UUID.to_owned())?;

    if empty {
        return Ok(generic_access_service)
    }

    // Device Name Characteristic
    let device_name_characteristic =
        create_characteristic_with_value(&generic_access_service,
                                              DEVICE_NAME_CHARACTERISTIC_UUID.to_owned(),
                                              HEART_RATE_DEVICE_NAME.as_bytes().to_vec())?;
    device_name_characteristic.set_flags(vec![READ_FLAG.to_string(), WRITE_FLAG.to_string()])?;

    // Number of Digitals descriptor
    let number_of_digitals_descriptor_1 =
        create_descriptor_with_value(&device_name_characteristic,
                                          NUMBER_OF_DIGITALS_UUID.to_owned(),
                                          vec![49])?;
    number_of_digitals_descriptor_1.set_flags(vec![READ_FLAG.to_string(), WRITE_FLAG.to_string()])?;

    let number_of_digitals_descriptor_2 =
        create_descriptor_with_value(&device_name_characteristic,
                                          NUMBER_OF_DIGITALS_UUID.to_owned(),
                                          vec![50])?;
    number_of_digitals_descriptor_2.set_flags(vec![READ_FLAG.to_string(), WRITE_FLAG.to_string()])?;

    // Characteristic User Description Descriptor
    let _characteristic_user_description =
        create_descriptor_with_value(&device_name_characteristic,
                                          CHARACTERISTIC_USER_DESCRIPTION_UUID.to_owned(),
                                          HEART_RATE_DEVICE_NAME_DESCRIPTION.as_bytes().to_vec())?;

    // Client Characteristic Configuration descriptor
    let _client_characteristic_configuration =
        create_descriptor_with_value(&device_name_characteristic,
                                          CLIENT_CHARACTERISTIC_CONFIGURATION_UUID.to_owned(),
                                          vec![0])?;

    // Peripheral Privacy Flag Characteristic
    let peripheral_privacy_flag_characteristic =
        create_characteristic(&generic_access_service, PERIPHERAL_PRIVACY_FLAG_CHARACTERISTIC_UUID.to_owned())?;
    peripheral_privacy_flag_characteristic
         .set_flags(vec![READ_FLAG.to_string(), WRITE_FLAG.to_string()])?;
    Ok(generic_access_service)
}

// Create Heart Rate Device
fn create_heart_rate_device(adapter: &BluetoothAdapter,
                            empty: bool)
                            -> Result<BluetoothDevice, Box<Error>> {
    // Heart Rate Device
    let heart_rate_device =
        create_device_with_uuids(adapter,
                                      HEART_RATE_DEVICE_NAME.to_owned(),
                                      HEART_RATE_DEVICE_ADDRESS.to_owned(),
                                      vec![GENERIC_ACCESS_SERVICE_UUID.to_owned(),
                                           HEART_RATE_SERVICE_UUID.to_owned()])?;

    if empty {
        return Ok(heart_rate_device);
    }

    // Generic Access Service
    let _generic_access_service = create_generic_access_service(&heart_rate_device, false)?;

    // Heart Rate Service
    let _heart_rate_service = create_heart_rate_service(&heart_rate_device, false)?;

    Ok(heart_rate_device)
}

fn create_missing_characterisitc_heart_rate_device(adapter: &BluetoothAdapter) -> Result<(), Box<Error>> {
    let heart_rate_device_empty = create_heart_rate_device(adapter, true)?;

    let _generic_access_service_empty = create_generic_access_service(&heart_rate_device_empty, true)?;

    let _heart_rate_service_empty = create_heart_rate_service(&heart_rate_device_empty, true)?;

    Ok(())
}

fn create_missing_descriptor_heart_rate_device(adapter: &BluetoothAdapter) -> Result<(), Box<Error>> {
    let heart_rate_device_empty = create_heart_rate_device(adapter, true)?;

    let generic_access_service_empty = create_generic_access_service(&heart_rate_device_empty, true)?;

    let _device_name_characteristic =
        create_characteristic_with_value(&generic_access_service_empty,
                                              DEVICE_NAME_CHARACTERISTIC_UUID.to_owned(),
                                              HEART_RATE_DEVICE_NAME.as_bytes().to_vec())?;

    let peripheral_privacy_flag_characteristic =
        create_characteristic(&generic_access_service_empty,
                                   PERIPHERAL_PRIVACY_FLAG_CHARACTERISTIC_UUID.to_owned())?;
    peripheral_privacy_flag_characteristic.set_flags(vec![READ_FLAG.to_string(), WRITE_FLAG.to_string()])?;

    let _heart_rate_service = create_heart_rate_service(&heart_rate_device_empty, false)?;

    Ok(())
}

fn create_two_heart_rate_services_device(adapter: &BluetoothAdapter) -> Result<(), Box<Error>> {
    let heart_rate_device_empty = create_heart_rate_device(adapter, true)?;

    heart_rate_device_empty.set_uuids(vec![GENERIC_ACCESS_SERVICE_UUID.to_owned(),
                                                HEART_RATE_SERVICE_UUID.to_owned(),
                                                HEART_RATE_SERVICE_UUID.to_owned()])?;

    let _generic_access_service = create_generic_access_service(&heart_rate_device_empty, false)?;

    let heart_rate_service_empty_1 = create_heart_rate_service(&heart_rate_device_empty, true)?;

    let heart_rate_service_empty_2 = create_heart_rate_service(&heart_rate_device_empty, true)?;

    let heart_rate_measurement_characteristic =
        create_characteristic_with_value(&heart_rate_service_empty_1,
                                              HEART_RATE_MEASUREMENT_CHARACTERISTIC_UUID.to_owned(),
                                              vec![0])?;
    heart_rate_measurement_characteristic.set_flags(vec![NOTIFY_FLAG.to_string()])?;

    let _body_sensor_location_characteristic_1 =
        create_characteristic_with_value(&heart_rate_service_empty_1,
                                              BODY_SENSOR_LOCATION_CHARACTERISTIC_UUID.to_owned(),
                                              vec![49])?;

    let _body_sensor_location_characteristic_2 =
        create_characteristic_with_value(&heart_rate_service_empty_2,
                                              BODY_SENSOR_LOCATION_CHARACTERISTIC_UUID.to_owned(),
                                              vec![50])?;
    Ok(())
}

fn create_blocklisted_device(adapter: &BluetoothAdapter) -> Result<(), Box<Error>> {
    let connectable_device =
    create_device_with_uuids(adapter,
                                 CONNECTABLE_DEVICE_NAME.to_owned(),
                                 CONNECTABLE_DEVICE_ADDRESS.to_owned(),
                                 vec![BLOCKLIST_TEST_SERVICE_UUID.to_owned(),
                                      DEVICE_INFORMATION_UUID.to_owned(),
                                      GENERIC_ACCESS_SERVICE_UUID.to_owned(),
                                      HEART_RATE_SERVICE_UUID.to_owned(),
                                      HUMAN_INTERFACE_DEVICE_SERVICE_UUID.to_owned()])?;

    let blocklist_test_service = create_service(&connectable_device, BLOCKLIST_TEST_SERVICE_UUID.to_owned())?;

    let blocklist_exclude_reads_characteristic =
        create_characteristic(&blocklist_test_service,
                                   BLOCKLIST_EXCLUDE_READS_CHARACTERISTIC_UUID.to_owned())?;
    blocklist_exclude_reads_characteristic
         .set_flags(vec![READ_FLAG.to_string(), WRITE_FLAG.to_string()])?;

    let _blocklist_exclude_reads_descriptor =
        create_descriptor_with_value(&blocklist_exclude_reads_characteristic,
                                          BLOCKLIST_EXCLUDE_READS_DESCRIPTOR_UUID.to_owned(),
                                          vec![54; 3])?;

    let _blocklist_descriptor =
        create_descriptor_with_value(&blocklist_exclude_reads_characteristic,
                                          BLOCKLIST_DESCRIPTOR_UUID.to_owned(),
                                          vec![54; 3])?;

    let device_information_service = create_service(&connectable_device, DEVICE_INFORMATION_UUID.to_owned())?;

    let _serial_number_string_characteristic =
        create_characteristic(&device_information_service, SERIAL_NUMBER_STRING_UUID.to_owned())?;

    let _generic_access_service = create_generic_access_service(&connectable_device, false)?;

    let _heart_rate_service = create_heart_rate_service(&connectable_device, false)?;

    let _human_interface_device_service =
        create_service(&connectable_device, HUMAN_INTERFACE_DEVICE_SERVICE_UUID.to_owned())?;
    Ok(())
}

fn create_glucose_heart_rate_devices(adapter: &BluetoothAdapter) -> Result<(), Box<Error>> {
    let glucose_devie = create_device_with_uuids(adapter,
                                                      GLUCOSE_DEVICE_NAME.to_owned(),
                                                      GLUCOSE_DEVICE_ADDRESS.to_owned(),
                                                      vec![GLUCOSE_SERVICE_UUID.to_owned(),
                                                           TX_POWER_SERVICE_UUID.to_owned()])?;

    let heart_rate_device_empty = create_heart_rate_device(adapter, true)?;

    let mut manufacturer_dta = HashMap::new();
    manufacturer_dta.insert(17, vec![1, 2, 3]);
    glucose_devie.set_manufacturer_data(manufacturer_dta)?;

    let mut service_data = HashMap::new();
    service_data.insert(GLUCOSE_SERVICE_UUID.to_owned(), vec![1, 2, 3]);
    glucose_devie.set_service_data(service_data)?;

    service_data = HashMap::new();
    service_data.insert(HEART_RATE_SERVICE_UUID.to_owned(), vec![1, 2, 3]);
    heart_rate_device_empty.set_service_data(service_data)?;
    Ok(())
}

pub fn test(manager: &mut BluetoothManager, data_set_name: String) -> Result<(), Box<Error>> {
    let may_existing_adapter = manager.get_or_create_adapter();
    let adapter = match may_existing_adapter.as_ref() {
        Some(adapter) => adapter,
        None => return Err(Box::from(ADAPTER_ERROR.to_string())),
    };
    match data_set_name.as_str() {
        NOT_PRESENT_ADAPTER => {
            set_adapter(adapter, NOT_PRESENT_ADAPTER.to_owned())?;
            adapter.set_present(false)?;
        },
        NOT_POWERED_ADAPTER => {
            set_adapter(adapter, NOT_POWERED_ADAPTER.to_owned())?;
            adapter.set_powered(false)?;
        },
        EMPTY_ADAPTER => {
            set_adapter(adapter, EMPTY_ADAPTER.to_owned())?;
        },
        GLUCOSE_HEART_RATE_ADAPTER => {
            set_adapter(adapter, GLUCOSE_HEART_RATE_ADAPTER.to_owned())?;
            let _ = create_glucose_heart_rate_devices(adapter)?;
        },
        UNICODE_DEVICE_ADAPTER => {
            set_adapter(adapter, UNICODE_DEVICE_ADAPTER.to_owned())?;

            let _unicode_device = create_device(adapter,
                                                     UNICODE_DEVICE_NAME.to_owned(),
                                                     UNICODE_DEVICE_ADDRESS.to_owned())?;
        },
        MISSING_SERVICE_HEART_RATE_ADAPTER => {
            set_adapter(adapter, MISSING_SERVICE_HEART_RATE_ADAPTER.to_owned())?;

            let _heart_rate_device_empty = create_heart_rate_device(adapter, true)?;
        },
        MISSING_CHARACTERISTIC_HEART_RATE_ADAPTER => {
            set_adapter(adapter, MISSING_CHARACTERISTIC_HEART_RATE_ADAPTER.to_owned())?;

            let _ = create_missing_characterisitc_heart_rate_device(adapter)?;
        },
        MISSING_DESCRIPTOR_HEART_RATE_ADAPTER => {
            set_adapter(adapter, MISSING_DESCRIPTOR_HEART_RATE_ADAPTER.to_owned())?;

            let _ = create_missing_descriptor_heart_rate_device(adapter)?;
        },
        HEART_RATE_ADAPTER => {
            set_adapter(adapter, HEART_RATE_ADAPTER.to_owned())?;

            let _heart_rate_device = create_heart_rate_device(adapter, false)?;
        },
        EMPTY_NAME_HEART_RATE_ADAPTER => {
            set_adapter(adapter, EMPTY_NAME_HEART_RATE_ADAPTER.to_owned())?;

            let heart_rate_device = create_heart_rate_device(adapter, false)?;
            heart_rate_device.set_name(Some(EMPTY_DEVICE_NAME.to_owned()))?;
        },
        NO_NAME_HEART_RATE_ADAPTER => {
            set_adapter(adapter, NO_NAME_HEART_RATE_ADAPTER.to_owned())?;

            let heart_rate_device = create_heart_rate_device(adapter, false)?;
            heart_rate_device.set_name(None)?;
        },
        TWO_HEART_RATE_SERVICES_ADAPTER => {
            set_adapter(adapter, TWO_HEART_RATE_SERVICES_ADAPTER.to_owned())?;

            let _ = create_two_heart_rate_services_device(adapter)?;
        },
        BLOCKLIST_TEST_ADAPTER => {
            set_adapter(adapter, BLOCKLIST_TEST_ADAPTER.to_owned())?;

            let _ = create_blocklisted_device(adapter)?;
        },
        _ => return Err(Box::from(WRONG_DATA_SET_ERROR.to_string())),
    }
    return Ok(());
}
