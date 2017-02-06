'use strict';

// Bluetooth UUID constants:
// Services:
var blocklist_test_service_uuid = "611c954a-263b-4f4a-aab6-01ddb953f985";
var request_disconnection_service_uuid = "01d7d889-7451-419f-aeb8-d65e7b9277af";
// Characteristics:
var blocklist_exclude_reads_characteristic_uuid = "bad1c9a2-9a5b-4015-8b60-1579bbbf2135";
var request_disconnection_characteristic_uuid = "01d7d88a-7451-419f-aeb8-d65e7b9277af";
// Descriptors:
var blocklist_exclude_reads_descriptor_uuid = "aaaaaaaa-aaaa-1181-0510-810819516110";
var blocklist_descriptor_uuid = "07711111-6104-0970-7011-1107105110aaa";
var characteristic_user_description_uuid = "00002901-0000-1000-8000-00805f9b34fb";

// Bluetooth Adapter types:
var adapter_type = {
    not_present: 'NotPresentAdapter',
    not_powered: 'NotPoweredAdapter',
    empty: 'EmptyAdapter',
    heart_rate: 'HeartRateAdapter',
    two_heart_rate: 'TwoHeartRateServicesAdapter',
    empty_name_heart_rate: 'EmptyNameHeartRateAdapter',
    no_name_heart_rate: 'NoNameHeartRateAdapter',
    glucose_heart_rate: 'GlucoseHeartRateAdapter',
    unicode_device: 'UnicodeDeviceAdapter',
    blocklist: 'BlocklistTestAdapter',
    missing_characteristic_heart_rate: 'MissingCharacteristicHeartRateAdapter',
    missing_service_heart_rate: 'MissingServiceHeartRateAdapter',
    missing_descriptor_heart_rate: 'MissingDescriptorHeartRateAdapter'
};

var mock_device_name = {
    heart_rate: 'Heart Rate Device',
    glucose: 'Glucose Device'
};

var wrong = {
    name: 'wrong_name',
    service: 'wrong_service'
};

// Sometimes we need to test that using either the name, alias, or UUID
// produces the same result. The following objects help us do that.
var generic_access = {
    alias: 0x1800,
    name: 'generic_access',
    uuid: '00001800-0000-1000-8000-00805f9b34fb'
};

var device_name = {
    alias: 0x2a00,
    name: 'gap.device_name',
    uuid: '00002a00-0000-1000-8000-00805f9b34fb'
};

var reconnection_address = {
    alias: 0x2a03,
    name: 'gap.reconnection_address',
    uuid: '00002a03-0000-1000-8000-00805f9b34fb'
};

var heart_rate = {
    alias: 0x180d,
    name: 'heart_rate',
    uuid: '0000180d-0000-1000-8000-00805f9b34fb'
};

var heart_rate_measurement = {
    alias: 0x2a37,
    name: 'heart_rate_measurement',
    uuid: '00002a37-0000-1000-8000-00805f9b34fb'
};

var body_sensor_location = {
    alias: 0x2a38,
    name: 'body_sensor_location',
    uuid: '00002a38-0000-1000-8000-00805f9b34fb'
};

var glucose = {
    alias: 0x1808,
    name: 'glucose',
    uuid: '00001808-0000-1000-8000-00805f9b34fb'
};

var battery_service = {
    alias: 0x180f,
    name: 'battery_service',
    uuid: '0000180f-0000-1000-8000-00805f9b34fb'
};

var battery_level = {
    alias: 0x2a19,
    name: 'battery_level',
    uuid: '00002a19-0000-1000-8000-00805f9b34fb'
};

var tx_power = {
    alias: 0x1804,
    name: 'tx_power',
    uuid: '00001804-0000-1000-8000-00805f9b34fb'
};

var human_interface_device = {
    alias: 0x1812,
    name: 'human_interface_device',
    uuid: '00001812-0000-1000-8000-00805f9b34fb'
};

var device_information = {
    alias: 0x180a,
    name: 'device_information',
    uuid: '0000180a-0000-1000-8000-00805f9b34fb'
};

var peripherial_privacy_flag = {
    alias: 0x2a02,
    name: 'gap.peripheral_privacy_flag',
    uuid: '00002a02-0000-1000-8000-00805f9b34fb'
};

var serial_number_string = {
    alias: 0x2a25,
    name: 'serial_number_string',
    uuid: '00002a25-0000-1000-8000-00805f9b34fb'
};

var client_characteristic_configuration = {
    alias: 0x2902,
    name: 'gatt.client_characteristic_configuration',
    uuid: '00002902-0000-1000-8000-00805f9b34fb'
};

var number_of_digitals = {
    alias: 0x2909,
    name: 'number_of_digitals',
    uuid: '00002909-0000-1000-8000-00805f9b34fb'
};

// Helper function for converting strings to an array of bytes.
function asciiToDecimal(bytestr) {
    var result = [];
    for(var i = 0; i < bytestr.length; i++) {
        result[i] = bytestr.charCodeAt(i) ;
    }
    return result;
}
