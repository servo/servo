/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://webbluetoothcg.github.io/web-bluetooth/#bluetooth

dictionary BluetoothRequestDeviceFilter {
  sequence<BluetoothServiceUUID> services;
  DOMString name;
  DOMString namePrefix;
  unsigned short manufacturerId;
  BluetoothServiceUUID serviceDataUUID;
};

dictionary RequestDeviceOptions {
  sequence<BluetoothRequestDeviceFilter> filters;
  sequence<BluetoothServiceUUID> optionalServices /*= []*/;
  boolean acceptAllDevices = false;
};

[Pref="dom.bluetooth.enabled"]
interface Bluetooth {
//  [SecureContext]
//  readonly attribute BluetoothDevice? referringDevice;
//  [SecureContext]
  Promise<BluetoothDevice> requestDevice(optional RequestDeviceOptions options);
};

// Bluetooth implements EventTarget;
// Bluetooth implements BluetoothDeviceEventHandlers;
// Bluetooth implements CharacteristicEventHandlers;
// Bluetooth implements ServiceEventHandlers;
