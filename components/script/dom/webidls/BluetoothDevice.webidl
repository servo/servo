/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://webbluetoothcg.github.io/web-bluetooth/#bluetoothdevice

[Pref="dom.bluetooth.enabled"]
interface BluetoothDevice {
    readonly attribute DOMString id;
    readonly attribute DOMString? name;
    readonly attribute BluetoothAdvertisingData adData;
    readonly attribute BluetoothRemoteGATTServer gatt;
    // readonly attribute FrozenArray[] uuids;
};

// BluetoothDevice implements EventTarget;
// BluetoothDevice implements BluetoothDeviceEventHandlers;
// BluetoothDevice implements CharacteristicEventHandlers;
// BluetoothDevice implements ServiceEventHandlers;
