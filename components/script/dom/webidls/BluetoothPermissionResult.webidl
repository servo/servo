/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://webbluetoothcg.github.io/web-bluetooth/#bluetoothpermissionresult

dictionary BluetoothPermissionDescriptor : PermissionDescriptor {
  DOMString deviceId;
  // These match RequestDeviceOptions.
  sequence<BluetoothLEScanFilterInit> filters;
  sequence<BluetoothServiceUUID> optionalServices/* = []*/;
  boolean acceptAllDevices = false;
};

dictionary AllowedBluetoothDevice {
  required DOMString deviceId;
  required boolean mayUseGATT;
  // An allowedServices of "all" means all services are allowed.
  required (DOMString or sequence<UUID>) allowedServices;
};

dictionary BluetoothPermissionData {
  required sequence<AllowedBluetoothDevice> allowedDevices/* = []*/;
};

// [Pref="dom.bluetooth.enabled"]
interface BluetoothPermissionResult : PermissionStatus {
  // attribute FrozenArray<BluetoothDevice> devices;
  // Workaround until FrozenArray get implemented.
  sequence<BluetoothDevice> devices();
};
