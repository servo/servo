/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use script::dom::bindings::codegen::UnionTypes::StringOrUnsignedLong;
use script::dom::bindings::str::DOMString;
use script::dom::bluetoothuuid::BluetoothUUID;

const ADEADBEEF_ALIAS: &'static StringOrUnsignedLong = &StringOrUnsignedLong::UnsignedLong(0xADEADBEEF);
const AEROBIC_HEART_RATE: &'static str = "aerobic_heart_rate_lower_limit";
const AEROBIC_HEART_RATE_UUID: &'static str = "00002a7e-0000-1000-8000-00805f9b34fb";
const ALERT_NOTIFICATION: &'static str = "alert_notification";
const ALERT_NOTIFICATION_UUID: &'static str = "00001811-0000-1000-8000-00805f9b34fb";
const ALL_CAPS_UUID: &'static str = "1A2B3C4D-5E6F-7A8B-9C0D-1E2F3A4B5C6D";
const BASE_ALIAS: &'static StringOrUnsignedLong = &StringOrUnsignedLong::UnsignedLong(0x0);
const BASE_UUID: &'static str = "00000000-0000-1000-8000-00805f9b34fb";
const BASIC_UUID: &'static str = "1a2b3c4d-5e6f-7a8b-9c0d-1e2f3a4b5c6d";
const CHARACTERISTIC_EXTENDED: &'static str = "gatt.characteristic_extended_properties";
const CHARACTERISTIC_EXTENDED_UUID: &'static str = "00002900-0000-1000-8000-00805f9b34fb";
const DEADBEEF_ALIAS: &'static StringOrUnsignedLong = &StringOrUnsignedLong::UnsignedLong(0xDEADBEEF);
const DEADBEEF_UUID: &'static str = "deadbeef-0000-1000-8000-00805f9b34fb";
const DEADBEEF_STRING: &'static str = "deadbeef";
const FOURTEEN_DIGITS: &'static StringOrUnsignedLong = &StringOrUnsignedLong::UnsignedLong(0xffffffffffffff);
const INVALID_CHARACTER_UUID: &'static str = "0000000g-0000-1000-8000-00805f9b34fb";
const MAX_UUID: &'static str = "ffffffff-0000-1000-8000-00805f9b34fb";
const NINE_DIGITS: &'static StringOrUnsignedLong = &StringOrUnsignedLong::UnsignedLong(0xfffffffff);
const SYNTAX_ERROR: &'static str = "Syntax";
const THIRTEEN_DIGITS: &'static StringOrUnsignedLong = &StringOrUnsignedLong::UnsignedLong(0xfffffffffffff);
const WRONG_NAME: &'static str = "wrong_name";

#[test]
fn get_correct_base_uuid() {
    assert_eq!(BluetoothUUID::service(BASE_ALIAS.clone()).unwrap(), DOMString::from(BASE_UUID));
    assert_eq!(BluetoothUUID::characteristic(BASE_ALIAS.clone()).unwrap(), DOMString::from(BASE_UUID));
    assert_eq!(BluetoothUUID::descriptor(BASE_ALIAS.clone()).unwrap(), DOMString::from(BASE_UUID));
}

#[test]
fn get_max_uuid_with_bigger_aliases() {
    assert_eq!(BluetoothUUID::service(NINE_DIGITS.clone()).unwrap(), DOMString::from(MAX_UUID));
    assert_eq!(BluetoothUUID::characteristic(NINE_DIGITS.clone()).unwrap(), DOMString::from(MAX_UUID));
    assert_eq!(BluetoothUUID::descriptor(NINE_DIGITS.clone()).unwrap(), DOMString::from(MAX_UUID));
    assert_eq!(BluetoothUUID::service(THIRTEEN_DIGITS.clone()).unwrap(), DOMString::from(MAX_UUID));
    assert_eq!(BluetoothUUID::characteristic(THIRTEEN_DIGITS.clone()).unwrap(), DOMString::from(MAX_UUID));
    assert_eq!(BluetoothUUID::descriptor(THIRTEEN_DIGITS.clone()).unwrap(), DOMString::from(MAX_UUID));
    assert_eq!(BluetoothUUID::service(FOURTEEN_DIGITS.clone()).unwrap(), DOMString::from(MAX_UUID));
    assert_eq!(BluetoothUUID::characteristic(FOURTEEN_DIGITS.clone()).unwrap(), DOMString::from(MAX_UUID));
    assert_eq!(BluetoothUUID::descriptor(FOURTEEN_DIGITS.clone()).unwrap(), DOMString::from(MAX_UUID));
}

#[test]
fn get_valid_deadbeef_uuid() {
    assert_eq!(BluetoothUUID::service(DEADBEEF_ALIAS.clone()).unwrap(), DOMString::from(DEADBEEF_UUID));
    assert_eq!(BluetoothUUID::characteristic(DEADBEEF_ALIAS.clone()).unwrap(), DOMString::from(DEADBEEF_UUID));
    assert_eq!(BluetoothUUID::descriptor(DEADBEEF_ALIAS.clone()).unwrap(), DOMString::from(DEADBEEF_UUID));
}

#[test]
fn first_32_bits_used() {
    assert_eq!(BluetoothUUID::service(ADEADBEEF_ALIAS.clone()).unwrap(), DOMString::from(DEADBEEF_UUID));
    assert_eq!(BluetoothUUID::characteristic(ADEADBEEF_ALIAS.clone()).unwrap(), DOMString::from(DEADBEEF_UUID));
    assert_eq!(BluetoothUUID::descriptor(ADEADBEEF_ALIAS.clone()).unwrap(), DOMString::from(DEADBEEF_UUID));
}

#[test]
fn valid_uuid_string() {
    assert_eq!(BluetoothUUID::service(StringOrUnsignedLong::String(DOMString::from(BASIC_UUID))).unwrap(),
               DOMString::from(BASIC_UUID));
    assert_eq!(BluetoothUUID::characteristic(StringOrUnsignedLong::String(DOMString::from(BASIC_UUID))).unwrap(),
               DOMString::from(BASIC_UUID));
    assert_eq!(BluetoothUUID::descriptor(StringOrUnsignedLong::String(DOMString::from(BASIC_UUID))).unwrap(),
               DOMString::from(BASIC_UUID));
}

#[test]
fn uppercase_invalid_uuid() {
    let all_caps_uuid = StringOrUnsignedLong::String(DOMString::from(ALL_CAPS_UUID));
    assert_eq!(format!("{:?}", BluetoothUUID::service(all_caps_uuid.clone()).unwrap_err()), SYNTAX_ERROR);
    assert_eq!(format!("{:?}", BluetoothUUID::characteristic(all_caps_uuid.clone()).unwrap_err()), SYNTAX_ERROR);
    assert_eq!(format!("{:?}", BluetoothUUID::descriptor(all_caps_uuid).unwrap_err()), SYNTAX_ERROR);
}

#[test]
fn invalid_string_alias() {
    let deadbeef_string = StringOrUnsignedLong::String(DOMString::from(DEADBEEF_STRING));
    assert_eq!(format!("{:?}", BluetoothUUID::service(deadbeef_string.clone()).unwrap_err()), SYNTAX_ERROR);
    assert_eq!(format!("{:?}", BluetoothUUID::characteristic(deadbeef_string.clone()).unwrap_err()), SYNTAX_ERROR);
    assert_eq!(format!("{:?}", BluetoothUUID::descriptor(deadbeef_string).unwrap_err()), SYNTAX_ERROR);
}

#[test]
fn invalid_uuid_with_invalid_characters() {
    let invalid_character = StringOrUnsignedLong::String(DOMString::from(INVALID_CHARACTER_UUID));
    assert_eq!(format!("{:?}", BluetoothUUID::service(invalid_character.clone()).unwrap_err()), SYNTAX_ERROR);
    assert_eq!(format!("{:?}", BluetoothUUID::characteristic(invalid_character.clone()).unwrap_err()), SYNTAX_ERROR);
    assert_eq!(format!("{:?}", BluetoothUUID::descriptor(invalid_character).unwrap_err()), SYNTAX_ERROR);
}

#[test]
fn valid_uuid_from_name() {
    let alert_notification = StringOrUnsignedLong::String(DOMString::from(ALERT_NOTIFICATION));
    let alert_notification_uuid = DOMString::from(ALERT_NOTIFICATION_UUID);
    let aerobic_h_r_lower_limit = StringOrUnsignedLong::String(DOMString::from(AEROBIC_HEART_RATE));
    let aerobic_h_r_lower_limit_uuid = DOMString::from(AEROBIC_HEART_RATE_UUID);
    let char_extended_prop = StringOrUnsignedLong::String(DOMString::from(CHARACTERISTIC_EXTENDED));
    let char_extended_prop_uuid = DOMString::from(CHARACTERISTIC_EXTENDED_UUID);
    assert_eq!(BluetoothUUID::service(alert_notification).unwrap(), alert_notification_uuid);
    assert_eq!(BluetoothUUID::characteristic(aerobic_h_r_lower_limit).unwrap(), aerobic_h_r_lower_limit_uuid);
    assert_eq!(BluetoothUUID::descriptor(char_extended_prop).unwrap(), char_extended_prop_uuid);
}

#[test]
fn wrong_function_call() {
    let alert_notification = StringOrUnsignedLong::String(DOMString::from(ALERT_NOTIFICATION));
    let aerobic_h_r_lower_limit = StringOrUnsignedLong::String(DOMString::from(AEROBIC_HEART_RATE));
    let char_extended_prop = StringOrUnsignedLong::String(DOMString::from(CHARACTERISTIC_EXTENDED));
    assert_eq!(format!("{:?}", BluetoothUUID::service(aerobic_h_r_lower_limit.clone()).unwrap_err()), SYNTAX_ERROR);
    assert_eq!(format!("{:?}", BluetoothUUID::service(char_extended_prop.clone()).unwrap_err()), SYNTAX_ERROR);
    assert_eq!(format!("{:?}", BluetoothUUID::characteristic(alert_notification.clone()).unwrap_err()), SYNTAX_ERROR);
    assert_eq!(format!("{:?}", BluetoothUUID::characteristic(char_extended_prop).unwrap_err()), SYNTAX_ERROR);
    assert_eq!(format!("{:?}", BluetoothUUID::descriptor(alert_notification).unwrap_err()), SYNTAX_ERROR);
    assert_eq!(format!("{:?}", BluetoothUUID::descriptor(aerobic_h_r_lower_limit).unwrap_err()), SYNTAX_ERROR);
}

#[test]
fn invalid_name() {
    let wrong_name = StringOrUnsignedLong::String(DOMString::from(WRONG_NAME));
    assert_eq!(format!("{:?}", BluetoothUUID::service(wrong_name.clone()).unwrap_err()), SYNTAX_ERROR);
    assert_eq!(format!("{:?}", BluetoothUUID::characteristic(wrong_name.clone()).unwrap_err()), SYNTAX_ERROR);
    assert_eq!(format!("{:?}", BluetoothUUID::descriptor(wrong_name).unwrap_err()), SYNTAX_ERROR);
}
