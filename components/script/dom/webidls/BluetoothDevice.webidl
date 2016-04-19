/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://webbluetoothcg.github.io/web-bluetooth/#bluetoothdevice

// Allocation authorities for Vendor IDs:
enum VendorIDSource {
    "bluetooth",
    "usb",
    "unknown"
};

interface BluetoothDevice {
    readonly attribute DOMString id;
    readonly attribute DOMString? name;
    readonly attribute BluetoothAdvertisingData adData;
    readonly attribute unsigned long? deviceClass;
    readonly attribute VendorIDSource? vendorIDSource;
    readonly attribute unsigned long? vendorID;
    readonly attribute unsigned long? productID;
    readonly attribute unsigned long? productVersion;
    readonly attribute BluetoothRemoteGATTServer gatt;
    // readonly attribute FrozenArray[] uuids;
};

// BluetoothDevice implements EventTarget;
// BluetoothDevice implements BluetoothDeviceEventHandlers;
// BluetoothDevice implements CharacteristicEventHandlers;
// BluetoothDevice implements ServiceEventHandlers;
