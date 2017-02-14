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

[Pref="dom.bluetooth.enabled"]
interface BluetoothPermissionResult : PermissionStatus {
  // attribute FrozenArray<BluetoothDevice> devices;
  // Workaround until FrozenArray get implemented.
  sequence<BluetoothDevice> devices();
};
