/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://webbluetoothcg.github.io/web-bluetooth/#bluetooth

dictionary BluetoothDataFilterInit {
  // BufferSource dataPrefix;
  sequence<octet> dataPrefix;
  // BufferSource mask;
  sequence<octet> mask;
};

dictionary BluetoothLEScanFilterInit {
  sequence<BluetoothServiceUUID> services;
  DOMString name;
  DOMString namePrefix;
  // Maps unsigned shorts to BluetoothDataFilters.
  record<DOMString, BluetoothDataFilterInit> manufacturerData;
  // Maps BluetoothServiceUUIDs to BluetoothDataFilters.
  record<DOMString, BluetoothDataFilterInit> serviceData;
};

dictionary RequestDeviceOptions {
  sequence<BluetoothLEScanFilterInit> filters;
  sequence<BluetoothServiceUUID> optionalServices /*= []*/;
  boolean acceptAllDevices = false;
};

[Pref="dom.bluetooth.enabled"]
interface Bluetooth : EventTarget {
  [SecureContext]
  Promise<boolean> getAvailability();
  [SecureContext]
  attribute EventHandler onavailabilitychanged;
  // [SecureContext, SameObject]
  // readonly attribute BluetoothDevice? referringDevice;
  [SecureContext]
  Promise<BluetoothDevice> requestDevice(optional RequestDeviceOptions options);
};

// Bluetooth implements BluetoothDeviceEventHandlers;
// Bluetooth implements CharacteristicEventHandlers;
// Bluetooth implements ServiceEventHandlers;
