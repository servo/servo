/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://webbluetoothcg.github.io/web-bluetooth/#bluetoothremotegattservice

[Exposed=Window, Pref="dom.bluetooth.enabled"]
interface BluetoothRemoteGATTService : EventTarget {
  [SameObject]
  readonly attribute BluetoothDevice device;
  readonly attribute DOMString uuid;
  readonly attribute boolean isPrimary;
  Promise<BluetoothRemoteGATTCharacteristic> getCharacteristic(BluetoothCharacteristicUUID characteristic);
  Promise<sequence<BluetoothRemoteGATTCharacteristic>>
  getCharacteristics(optional BluetoothCharacteristicUUID characteristic);
  Promise<BluetoothRemoteGATTService> getIncludedService(BluetoothServiceUUID service);
  Promise<sequence<BluetoothRemoteGATTService>> getIncludedServices(optional BluetoothServiceUUID service);
};

interface mixin ServiceEventHandlers {
  attribute EventHandler onserviceadded;
  attribute EventHandler onservicechanged;
  attribute EventHandler onserviceremoved;
};

// BluetoothRemoteGATTService includes EventTarget;
// BluetoothRemoteGATTService includes CharacteristicEventHandlers;
BluetoothRemoteGATTService includes ServiceEventHandlers;
