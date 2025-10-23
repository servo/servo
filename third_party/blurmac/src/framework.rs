// Copyright (c) 2017 Akos Kiss.
//
// Licensed under the BSD 3-Clause License
// <LICENSE.md or https://opensource.org/licenses/BSD-3-Clause>.
// This file may not be copied, modified, or distributed except
// according to those terms.

use std::os::raw::{c_char, c_int, c_uint};

use objc2::ffi::BOOL;
use objc2::runtime::{AnyClass, AnyObject};

#[allow(non_upper_case_globals)]
pub const nil: *mut AnyObject = 0 as *mut AnyObject;

pub mod ns {
    use super::*;

    // NSAnyObject

    pub fn object_copy(nsobject: *mut AnyObject) -> *mut AnyObject {
        unsafe {
            let copy: *mut AnyObject = msg_send![nsobject, copy];
            copy
        }
    }

    // NSNumber

    pub fn number_withbool(value: BOOL) -> *mut AnyObject {
        unsafe {
            let nsnumber: *mut AnyObject =
                msg_send![AnyClass::get("NSNumber").unwrap(), numberWithBool: value];
            nsnumber
        }
    }

    pub fn number_withunsignedlonglong(value: u64) -> *mut AnyObject {
        unsafe {
            let nsnumber: *mut AnyObject = msg_send![
                AnyClass::get("NSNumber").unwrap(),
                numberWithUnsignedLongLong: value
            ];
            nsnumber
        }
    }

    pub fn number_unsignedlonglongvalue(nsnumber: *mut AnyObject) -> u64 {
        unsafe {
            let value: u64 = msg_send![nsnumber, unsignedLongLongValue];
            value
        }
    }

    // NSString

    pub fn string(cstring: *const c_char) -> *mut AnyObject /* NSString* */ {
        unsafe {
            let nsstring: *mut AnyObject = msg_send![
                AnyClass::get("NSString").unwrap(),
                stringWithUTF8String: cstring
            ];
            nsstring
        }
    }

    pub fn string_utf8string(nsstring: *mut AnyObject) -> *const c_char {
        unsafe {
            let utf8string: *const c_char = msg_send![nsstring, UTF8String];
            utf8string
        }
    }

    // NSArray

    pub fn array_count(nsarray: *mut AnyObject) -> c_uint {
        unsafe {
            let count: c_uint = msg_send![nsarray, count];
            count
        }
    }

    pub fn array_objectatindex(nsarray: *mut AnyObject, index: c_uint) -> *mut AnyObject {
        unsafe {
            let object: *mut AnyObject = msg_send![nsarray, objectAtIndex: index];
            object
        }
    }

    // NSDictionary

    pub fn dictionary_allkeys(nsdict: *mut AnyObject) -> *mut AnyObject /* NSArray* */ {
        unsafe {
            let keys: *mut AnyObject = msg_send![nsdict, allKeys];
            keys
        }
    }

    pub fn dictionary_objectforkey(nsdict: *mut AnyObject, key: *mut AnyObject) -> *mut AnyObject {
        unsafe {
            let object: *mut AnyObject = msg_send![nsdict, objectForKey: key];
            object
        }
    }

    // NSMutableDictionary : NSDictionary

    pub fn mutabledictionary() -> *mut AnyObject {
        unsafe {
            let nsmutdict: *mut AnyObject =
                msg_send![AnyClass::get("NSMutableDictionary").unwrap(), dictionaryWithCapacity:0];
            nsmutdict
        }
    }

    pub fn mutabledictionary_removeobjectforkey(nsmutdict: *mut AnyObject, key: *mut AnyObject) {
        unsafe {
            let () = msg_send![nsmutdict, removeAnyObjectForKey: key];
        }
    }

    pub fn mutabledictionary_setobject_forkey(
        nsmutdict: *mut AnyObject,
        object: *mut AnyObject,
        key: *mut AnyObject,
    ) {
        unsafe {
            let () = msg_send![nsmutdict, setAnyObject:object forKey:key];
        }
    }

    // NSData

    pub fn data(bytes: *const u8, length: c_uint) -> *mut AnyObject /* NSData* */ {
        unsafe {
            let data: *mut AnyObject =
                msg_send![AnyClass::get("NSData").unwrap(), dataWithBytes:bytes length:length];
            data
        }
    }

    pub fn data_length(nsdata: *mut AnyObject) -> c_uint {
        unsafe {
            let length: c_uint = msg_send![nsdata, length];
            length
        }
    }

    pub fn data_bytes(nsdata: *mut AnyObject) -> *const u8 {
        unsafe {
            let bytes: *const u8 = msg_send![nsdata, bytes];
            bytes
        }
    }

    // NSUUID

    pub fn uuid_uuidstring(nsuuid: *mut AnyObject) -> *mut AnyObject /* NSString* */ {
        unsafe {
            let uuidstring: *mut AnyObject = msg_send![nsuuid, UUIDString];
            uuidstring
        }
    }
}

pub mod io {
    use super::*;

    #[link(name = "IOBluetooth", kind = "framework")]
    unsafe extern "C" {
        pub fn IOBluetoothPreferenceGetControllerPowerState() -> c_int;
        pub fn IOBluetoothPreferenceSetControllerPowerState(state: c_int);

        pub fn IOBluetoothPreferenceGetDiscoverableState() -> c_int;
        pub fn IOBluetoothPreferenceSetDiscoverableState(state: c_int);
    }

    // IOBluetoothHostController

    pub fn bluetoothhostcontroller_defaultcontroller() -> *mut AnyObject /* IOBluetoothHostController* */
    {
        unsafe {
            let defaultcontroller: *mut AnyObject = msg_send![
                AnyClass::get("IOBluetoothHostController").unwrap(),
                defaultController
            ];
            defaultcontroller
        }
    }

    pub fn bluetoothhostcontroller_nameasstring(iobthc: *mut AnyObject) -> *mut AnyObject /* NSString* */
    {
        unsafe {
            let name: *mut AnyObject = msg_send![iobthc, nameAsString];
            name
        }
    }

    pub fn bluetoothhostcontroller_addressasstring(iobthc: *mut AnyObject) -> *mut AnyObject /* NSString* */
    {
        unsafe {
            let address: *mut AnyObject = msg_send![iobthc, addressAsString];
            address
        }
    }

    pub fn bluetoothhostcontroller_classofdevice(iobthc: *mut AnyObject) -> u32 {
        unsafe {
            let classofdevice: u32 = msg_send![iobthc, classOfDevice];
            classofdevice
        }
    }

    // IOBluetoothPreference...

    pub fn bluetoothpreferencegetcontrollerpowerstate() -> c_int {
        unsafe { IOBluetoothPreferenceGetControllerPowerState() }
    }

    pub fn bluetoothpreferencesetcontrollerpowerstate(state: c_int) {
        unsafe {
            IOBluetoothPreferenceSetControllerPowerState(state);
        }
    }

    pub fn bluetoothpreferencegetdiscoverablestate() -> c_int {
        unsafe { IOBluetoothPreferenceGetDiscoverableState() }
    }

    pub fn bluetoothpreferencesetdiscoverablestate(state: c_int) {
        unsafe {
            IOBluetoothPreferenceSetDiscoverableState(state);
        }
    }
}

pub mod cb {
    use super::*;

    mod link {
        use super::*;

        #[link(name = "CoreBluetooth", kind = "framework")]
        unsafe extern "C" {
            pub static CBAdvertisementDataServiceUUIDsKey: *mut AnyObject;

            pub static CBCentralManagerScanOptionAllowDuplicatesKey: *mut AnyObject;
        }
    }

    // CBCentralManager

    pub fn centralmanager(
        delegate: *mut AnyObject, /*CBCentralManagerDelegate* */
    ) -> *mut AnyObject /*CBCentralManager* */ {
        unsafe {
            let cbcentralmanager: *mut AnyObject =
                msg_send![AnyClass::get("CBCentralManager").unwrap(), alloc];
            let () = msg_send![cbcentralmanager, initWithDelegate:delegate queue:nil];
            cbcentralmanager
        }
    }

    pub fn centralmanager_scanforperipherals_options(
        cbcentralmanager: *mut AnyObject,
        options: *mut AnyObject, /* NSDictionary<NSString*,id> */
    ) {
        unsafe {
            let () =
                msg_send![cbcentralmanager, scanForPeripheralsWithServices:nil options:options];
        }
    }

    pub fn centralmanager_stopscan(cbcentralmanager: *mut AnyObject) {
        unsafe {
            let () = msg_send![cbcentralmanager, stopScan];
        }
    }

    pub fn centralmanager_connectperipheral(
        cbcentralmanager: *mut AnyObject,
        peripheral: *mut AnyObject, /* CBPeripheral* */
    ) {
        unsafe {
            let () = msg_send![cbcentralmanager, connectPeripheral:peripheral options:nil];
        }
    }

    pub fn centralmanager_cancelperipheralconnection(
        cbcentralmanager: *mut AnyObject,
        peripheral: *mut AnyObject, /* CBPeripheral* */
    ) {
        unsafe {
            let () = msg_send![cbcentralmanager, cancelPeripheralConnection: peripheral];
        }
    }

    // CBPeer

    pub fn peer_identifier(cbpeer: *mut AnyObject) -> *mut AnyObject /* NSUUID* */ {
        unsafe {
            let identifier: *mut AnyObject = msg_send![cbpeer, identifier];
            identifier
        }
    }

    // CBPeripheral : CBPeer

    pub fn peripheral_name(cbperipheral: *mut AnyObject) -> *mut AnyObject /* NSString* */ {
        unsafe {
            let name: *mut AnyObject = msg_send![cbperipheral, name];
            name
        }
    }

    pub fn peripheral_state(cbperipheral: *mut AnyObject) -> c_int {
        unsafe {
            let state: c_int = msg_send![cbperipheral, state];
            state
        }
    }

    pub fn peripheral_setdelegate(
        cbperipheral: *mut AnyObject,
        delegate: *mut AnyObject, /* CBPeripheralDelegate* */
    ) {
        unsafe {
            let () = msg_send![cbperipheral, setDelegate: delegate];
        }
    }

    pub fn peripheral_discoverservices(cbperipheral: *mut AnyObject) {
        unsafe {
            let () = msg_send![cbperipheral, discoverServices: nil];
        }
    }

    pub fn peripheral_discoverincludedservicesforservice(
        cbperipheral: *mut AnyObject,
        service: *mut AnyObject, /* CBService* */
    ) {
        unsafe {
            let () = msg_send![cbperipheral, discoverIncludedServices:nil forService:service];
        }
    }

    pub fn peripheral_services(cbperipheral: *mut AnyObject) -> *mut AnyObject /* NSArray<CBService*>* */
    {
        unsafe {
            let services: *mut AnyObject = msg_send![cbperipheral, services];
            services
        }
    }

    pub fn peripheral_discovercharacteristicsforservice(
        cbperipheral: *mut AnyObject,
        service: *mut AnyObject, /* CBService* */
    ) {
        unsafe {
            let () = msg_send![cbperipheral, discoverCharacteristics:nil forService:service];
        }
    }

    pub fn peripheral_readvalueforcharacteristic(
        cbperipheral: *mut AnyObject,
        characteristic: *mut AnyObject, /* CBCharacteristic* */
    ) {
        unsafe {
            let () = msg_send![cbperipheral, readValueForCharacteristic: characteristic];
        }
    }

    pub fn peripheral_writevalue_forcharacteristic(
        cbperipheral: *mut AnyObject,
        value: *mut AnyObject,          /* NSData* */
        characteristic: *mut AnyObject, /* CBCharacteristic* */
    ) {
        unsafe {
            let () =
                msg_send![cbperipheral, writeValue:value forCharacteristic:characteristic type:0];
            // CBCharacteristicWriteWithResponse from CBPeripheral.h
        }
    }

    pub fn peripheral_setnotifyvalue_forcharacteristic(
        cbperipheral: *mut AnyObject,
        value: BOOL,
        characteristic: *mut AnyObject, /* CBCharacteristic* */
    ) {
        unsafe {
            let () = msg_send![cbperipheral, setNotifyValue:value forCharacteristic:characteristic];
        }
    }

    pub fn peripheral_discoverdescriptorsforcharacteristic(
        cbperipheral: *mut AnyObject,
        characteristic: *mut AnyObject, /* CBCharacteristic* */
    ) {
        unsafe {
            let () = msg_send![
                cbperipheral,
                discoverDescriptorsForCharacteristic: characteristic
            ];
        }
    }

    // CBPeripheralState = NSInteger from CBPeripheral.h

    pub const PERIPHERALSTATE_CONNECTED: c_int = 2; // CBPeripheralStateConnected

    // CBAttribute

    pub fn attribute_uuid(cbattribute: *mut AnyObject) -> *mut AnyObject /* CBUUID* */ {
        unsafe {
            let uuid: *mut AnyObject = msg_send![cbattribute, UUID];
            uuid
        }
    }

    // CBService : CBAttribute

    // pub fn service_isprimary(cbservice: *mut AnyObject) -> BOOL {
    //     unsafe {
    //         let isprimary: BOOL = msg_send![cbservice, isPrimary];
    //         isprimary
    //     }
    // }

    pub fn service_includedservices(cbservice: *mut AnyObject) -> *mut AnyObject /* NSArray<CBService*>* */
    {
        unsafe {
            let includedservices: *mut AnyObject = msg_send![cbservice, includedServices];
            includedservices
        }
    }

    pub fn service_characteristics(cbservice: *mut AnyObject) -> *mut AnyObject /* NSArray<CBCharacteristic*>* */
    {
        unsafe {
            let characteristics: *mut AnyObject = msg_send![cbservice, characteristics];
            characteristics
        }
    }

    // CBCharacteristic : CBAttribute

    pub fn characteristic_isnotifying(cbcharacteristic: *mut AnyObject) -> BOOL {
        unsafe {
            let isnotifying: BOOL = msg_send![cbcharacteristic, isNotifying];
            isnotifying
        }
    }

    pub fn characteristic_value(cbcharacteristic: *mut AnyObject) -> *mut AnyObject /* NSData* */ {
        unsafe {
            let value: *mut AnyObject = msg_send![cbcharacteristic, value];
            value
        }
    }

    pub fn characteristic_properties(cbcharacteristic: *mut AnyObject) -> c_uint {
        unsafe {
            let properties: c_uint = msg_send![cbcharacteristic, properties];
            properties
        }
    }

    // CBCharacteristicProperties = NSUInteger from CBCharacteristic.h

    pub const CHARACTERISTICPROPERTY_BROADCAST: c_uint = 0x01; // CBCharacteristicPropertyBroadcast
    pub const CHARACTERISTICPROPERTY_READ: c_uint = 0x02; // CBCharacteristicPropertyRead
    pub const CHARACTERISTICPROPERTY_WRITEWITHOUTRESPONSE: c_uint = 0x04; // CBCharacteristicPropertyWriteWithoutResponse
    pub const CHARACTERISTICPROPERTY_WRITE: c_uint = 0x08; // CBCharacteristicPropertyWrite
    pub const CHARACTERISTICPROPERTY_NOTIFY: c_uint = 0x10; // CBCharacteristicPropertyNotify
    pub const CHARACTERISTICPROPERTY_INDICATE: c_uint = 0x20; // CBCharacteristicPropertyIndicate
    pub const CHARACTERISTICPROPERTY_AUTHENTICATEDSIGNEDWRITES: c_uint = 0x40; // CBCharacteristicPropertyAuthenticatedSignedWrites

    // CBUUID

    pub fn uuid_uuidstring(cbuuid: *mut AnyObject) -> *mut AnyObject /* NSString* */ {
        unsafe {
            let uuidstring: *mut AnyObject = msg_send![cbuuid, UUIDString];
            uuidstring
        }
    }

    // CBCentralManagerScanOption...Key

    // CBAdvertisementData...Key
    pub use self::link::{
        CBAdvertisementDataServiceUUIDsKey as ADVERTISEMENTDATASERVICEUUIDSKEY,
        CBCentralManagerScanOptionAllowDuplicatesKey as CENTRALMANAGERSCANOPTIONALLOWDUPLICATESKEY,
    };
}
