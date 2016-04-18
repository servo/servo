/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://webbluetoothcg.github.io/web-bluetooth/#bluetoothremotegattservice

interface BluetoothRemoteGATTService {
    readonly attribute BluetoothDevice device;
    readonly attribute DOMString uuid;
    readonly attribute boolean isPrimary;
    [Throws]
    BluetoothRemoteGATTCharacteristic getCharacteristic((DOMString or unsigned long) characteristic);
    [Throws]
    sequence<BluetoothRemoteGATTCharacteristic> getCharacteristics
        (optional (DOMString or unsigned long) characteristic);
    //Promise<BluetoothRemoteGATTCharacteristic>getCharacteristic(BluetoothCharacteristicUUID characteristic);
    //Promise<sequence<BluetoothRemoteGATTCharacteristic>>
    //getCharacteristics(optional BluetoothCharacteristicUUID characteristic);
    //Promise<BluetoothRemoteGATTService>getIncludedService(BluetoothServiceUUID service);
    //Promise<sequence<BluetoothRemoteGATTService>>getIncludedServices(optional BluetoothServiceUUID service);
};
