// Copyright (c) 2017 Akos Kiss.
//
// Licensed under the BSD 3-Clause License
// <LICENSE.md or https://opensource.org/licenses/BSD-3-Clause>.
// This file may not be copied, modified, or distributed except
// according to those terms.

use std::os::raw::{c_char, c_int, c_uint};

use objc::runtime::{Class, Object, BOOL};

#[allow(non_upper_case_globals)]
pub const nil: *mut Object = 0 as *mut Object;

pub mod ns {
    use super::*;

    // NSObject

    pub fn object_copy(nsobject: *mut Object) -> *mut Object {
        unsafe {
            let copy: *mut Object = msg_send![nsobject, copy];
            copy
        }
    }

    // NSNumber

    pub fn number_withbool(value: BOOL) -> *mut Object {
        unsafe {
            let nsnumber: *mut Object =
                msg_send![Class::get("NSNumber").unwrap(), numberWithBool: value];
            nsnumber
        }
    }

    pub fn number_withunsignedlonglong(value: u64) -> *mut Object {
        unsafe {
            let nsnumber: *mut Object = msg_send![
                Class::get("NSNumber").unwrap(),
                numberWithUnsignedLongLong: value
            ];
            nsnumber
        }
    }

    pub fn number_unsignedlonglongvalue(nsnumber: *mut Object) -> u64 {
        unsafe {
            let value: u64 = msg_send![nsnumber, unsignedLongLongValue];
            value
        }
    }

    // NSString

    pub fn string(cstring: *const c_char) -> *mut Object /* NSString* */ {
        unsafe {
            let nsstring: *mut Object = msg_send![
                Class::get("NSString").unwrap(),
                stringWithUTF8String: cstring
            ];
            nsstring
        }
    }

    pub fn string_utf8string(nsstring: *mut Object) -> *const c_char {
        unsafe {
            let utf8string: *const c_char = msg_send![nsstring, UTF8String];
            utf8string
        }
    }

    // NSArray

    pub fn array_count(nsarray: *mut Object) -> c_uint {
        unsafe {
            let count: c_uint = msg_send![nsarray, count];
            count
        }
    }

    pub fn array_objectatindex(nsarray: *mut Object, index: c_uint) -> *mut Object {
        unsafe {
            let object: *mut Object = msg_send![nsarray, objectAtIndex: index];
            object
        }
    }

    // NSDictionary

    pub fn dictionary_allkeys(nsdict: *mut Object) -> *mut Object /* NSArray* */ {
        unsafe {
            let keys: *mut Object = msg_send![nsdict, allKeys];
            keys
        }
    }

    pub fn dictionary_objectforkey(nsdict: *mut Object, key: *mut Object) -> *mut Object {
        unsafe {
            let object: *mut Object = msg_send![nsdict, objectForKey: key];
            object
        }
    }

    // NSMutableDictionary : NSDictionary

    pub fn mutabledictionary() -> *mut Object {
        unsafe {
            let nsmutdict: *mut Object =
                msg_send![Class::get("NSMutableDictionary").unwrap(), dictionaryWithCapacity:0];
            nsmutdict
        }
    }

    pub fn mutabledictionary_removeobjectforkey(nsmutdict: *mut Object, key: *mut Object) {
        unsafe {
            let () = msg_send![nsmutdict, removeObjectForKey: key];
        }
    }

    pub fn mutabledictionary_setobject_forkey(
        nsmutdict: *mut Object,
        object: *mut Object,
        key: *mut Object,
    ) {
        unsafe {
            let () = msg_send![nsmutdict, setObject:object forKey:key];
        }
    }

    // NSData

    pub fn data(bytes: *const u8, length: c_uint) -> *mut Object /* NSData* */ {
        unsafe {
            let data: *mut Object =
                msg_send![Class::get("NSData").unwrap(), dataWithBytes:bytes length:length];
            data
        }
    }

    pub fn data_length(nsdata: *mut Object) -> c_uint {
        unsafe {
            let length: c_uint = msg_send![nsdata, length];
            length
        }
    }

    pub fn data_bytes(nsdata: *mut Object) -> *const u8 {
        unsafe {
            let bytes: *const u8 = msg_send![nsdata, bytes];
            bytes
        }
    }

    // NSUUID

    pub fn uuid_uuidstring(nsuuid: *mut Object) -> *mut Object /* NSString* */ {
        unsafe {
            let uuidstring: *mut Object = msg_send![nsuuid, UUIDString];
            uuidstring
        }
    }
}

pub mod io {
    use super::*;

    #[link(name = "IOBluetooth", kind = "framework")]
    extern "C" {
        pub fn IOBluetoothPreferenceGetControllerPowerState() -> c_int;
        pub fn IOBluetoothPreferenceSetControllerPowerState(state: c_int);

        pub fn IOBluetoothPreferenceGetDiscoverableState() -> c_int;
        pub fn IOBluetoothPreferenceSetDiscoverableState(state: c_int);
    }

    // IOBluetoothHostController

    pub fn bluetoothhostcontroller_defaultcontroller() -> *mut Object /* IOBluetoothHostController* */
    {
        unsafe {
            let defaultcontroller: *mut Object = msg_send![
                Class::get("IOBluetoothHostController").unwrap(),
                defaultController
            ];
            defaultcontroller
        }
    }

    pub fn bluetoothhostcontroller_nameasstring(iobthc: *mut Object) -> *mut Object /* NSString* */
    {
        unsafe {
            let name: *mut Object = msg_send![iobthc, nameAsString];
            name
        }
    }

    pub fn bluetoothhostcontroller_addressasstring(iobthc: *mut Object) -> *mut Object /* NSString* */
    {
        unsafe {
            let address: *mut Object = msg_send![iobthc, addressAsString];
            address
        }
    }

    pub fn bluetoothhostcontroller_classofdevice(iobthc: *mut Object) -> u32 {
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
        extern "C" {
            pub static CBAdvertisementDataServiceUUIDsKey: *mut Object;

            pub static CBCentralManagerScanOptionAllowDuplicatesKey: *mut Object;
        }
    }

    // CBCentralManager

    pub fn centralmanager(delegate: *mut Object, /*CBCentralManagerDelegate* */) -> *mut Object /*CBCentralManager* */
    {
        unsafe {
            let cbcentralmanager: *mut Object =
                msg_send![Class::get("CBCentralManager").unwrap(), alloc];
            let () = msg_send![cbcentralmanager, initWithDelegate:delegate queue:nil];
            cbcentralmanager
        }
    }

    pub fn centralmanager_scanforperipherals_options(
        cbcentralmanager: *mut Object,
        options: *mut Object, /* NSDictionary<NSString*,id> */
    ) {
        unsafe {
            let () =
                msg_send![cbcentralmanager, scanForPeripheralsWithServices:nil options:options];
        }
    }

    pub fn centralmanager_stopscan(cbcentralmanager: *mut Object) {
        unsafe {
            let () = msg_send![cbcentralmanager, stopScan];
        }
    }

    pub fn centralmanager_connectperipheral(
        cbcentralmanager: *mut Object,
        peripheral: *mut Object, /* CBPeripheral* */
    ) {
        unsafe {
            let () = msg_send![cbcentralmanager, connectPeripheral:peripheral options:nil];
        }
    }

    pub fn centralmanager_cancelperipheralconnection(
        cbcentralmanager: *mut Object,
        peripheral: *mut Object, /* CBPeripheral* */
    ) {
        unsafe {
            let () = msg_send![cbcentralmanager, cancelPeripheralConnection: peripheral];
        }
    }

    // CBPeer

    pub fn peer_identifier(cbpeer: *mut Object) -> *mut Object /* NSUUID* */ {
        unsafe {
            let identifier: *mut Object = msg_send![cbpeer, identifier];
            identifier
        }
    }

    // CBPeripheral : CBPeer

    pub fn peripheral_name(cbperipheral: *mut Object) -> *mut Object /* NSString* */ {
        unsafe {
            let name: *mut Object = msg_send![cbperipheral, name];
            name
        }
    }

    pub fn peripheral_state(cbperipheral: *mut Object) -> c_int {
        unsafe {
            let state: c_int = msg_send![cbperipheral, state];
            state
        }
    }

    pub fn peripheral_setdelegate(
        cbperipheral: *mut Object,
        delegate: *mut Object, /* CBPeripheralDelegate* */
    ) {
        unsafe {
            let () = msg_send![cbperipheral, setDelegate: delegate];
        }
    }

    pub fn peripheral_discoverservices(cbperipheral: *mut Object) {
        unsafe {
            let () = msg_send![cbperipheral, discoverServices: nil];
        }
    }

    pub fn peripheral_discoverincludedservicesforservice(
        cbperipheral: *mut Object,
        service: *mut Object, /* CBService* */
    ) {
        unsafe {
            let () = msg_send![cbperipheral, discoverIncludedServices:nil forService:service];
        }
    }

    pub fn peripheral_services(cbperipheral: *mut Object) -> *mut Object /* NSArray<CBService*>* */
    {
        unsafe {
            let services: *mut Object = msg_send![cbperipheral, services];
            services
        }
    }

    pub fn peripheral_discovercharacteristicsforservice(
        cbperipheral: *mut Object,
        service: *mut Object, /* CBService* */
    ) {
        unsafe {
            let () = msg_send![cbperipheral, discoverCharacteristics:nil forService:service];
        }
    }

    pub fn peripheral_readvalueforcharacteristic(
        cbperipheral: *mut Object,
        characteristic: *mut Object, /* CBCharacteristic* */
    ) {
        unsafe {
            let () = msg_send![cbperipheral, readValueForCharacteristic: characteristic];
        }
    }

    pub fn peripheral_writevalue_forcharacteristic(
        cbperipheral: *mut Object,
        value: *mut Object,          /* NSData* */
        characteristic: *mut Object, /* CBCharacteristic* */
    ) {
        unsafe {
            let () =
                msg_send![cbperipheral, writeValue:value forCharacteristic:characteristic type:0];
            // CBCharacteristicWriteWithResponse from CBPeripheral.h
        }
    }

    pub fn peripheral_setnotifyvalue_forcharacteristic(
        cbperipheral: *mut Object,
        value: BOOL,
        characteristic: *mut Object, /* CBCharacteristic* */
    ) {
        unsafe {
            let () = msg_send![cbperipheral, setNotifyValue:value forCharacteristic:characteristic];
        }
    }

    pub fn peripheral_discoverdescriptorsforcharacteristic(
        cbperipheral: *mut Object,
        characteristic: *mut Object, /* CBCharacteristic* */
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

    pub fn attribute_uuid(cbattribute: *mut Object) -> *mut Object /* CBUUID* */ {
        unsafe {
            let uuid: *mut Object = msg_send![cbattribute, UUID];
            uuid
        }
    }

    // CBService : CBAttribute

    // pub fn service_isprimary(cbservice: *mut Object) -> BOOL {
    //     unsafe {
    //         let isprimary: BOOL = msg_send![cbservice, isPrimary];
    //         isprimary
    //     }
    // }

    pub fn service_includedservices(cbservice: *mut Object) -> *mut Object /* NSArray<CBService*>* */
    {
        unsafe {
            let includedservices: *mut Object = msg_send![cbservice, includedServices];
            includedservices
        }
    }

    pub fn service_characteristics(cbservice: *mut Object) -> *mut Object /* NSArray<CBCharacteristic*>* */
    {
        unsafe {
            let characteristics: *mut Object = msg_send![cbservice, characteristics];
            characteristics
        }
    }

    // CBCharacteristic : CBAttribute

    pub fn characteristic_isnotifying(cbcharacteristic: *mut Object) -> BOOL {
        unsafe {
            let isnotifying: BOOL = msg_send![cbcharacteristic, isNotifying];
            isnotifying
        }
    }

    pub fn characteristic_value(cbcharacteristic: *mut Object) -> *mut Object /* NSData* */ {
        unsafe {
            let value: *mut Object = msg_send![cbcharacteristic, value];
            value
        }
    }

    pub fn characteristic_properties(cbcharacteristic: *mut Object) -> c_uint {
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

    pub fn uuid_uuidstring(cbuuid: *mut Object) -> *mut Object /* NSString* */ {
        unsafe {
            let uuidstring: *mut Object = msg_send![cbuuid, UUIDString];
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
