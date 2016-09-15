/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://webbluetoothcg.github.io/web-bluetooth/#bluetoothremotegattservice

[Pref="dom.bluetooth.enabled", Exposed=(Window,Worker)]
interface BluetoothRemoteGATTService {
    readonly attribute BluetoothDevice device;
    readonly attribute DOMString uuid;
    readonly attribute boolean isPrimary;
    [Throws]
    Promise<BluetoothRemoteGATTCharacteristic> getCharacteristic(BluetoothCharacteristicUUID characteristic);
    [Throws]
    Promise<sequence<BluetoothRemoteGATTCharacteristic>>
    getCharacteristics(optional BluetoothCharacteristicUUID characteristic);
    [Throws]
    Promise<BluetoothRemoteGATTService> getIncludedService(BluetoothServiceUUID service);
    [Throws]
    Promise<sequence<BluetoothRemoteGATTService>> getIncludedServices(optional BluetoothServiceUUID service);
};
