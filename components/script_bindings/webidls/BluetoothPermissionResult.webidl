/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// skip-unless CARGO_FEATURE_BLUETOOTH

// https://webbluetoothcg.github.io/web-bluetooth/#bluetoothpermissionresult

dictionary BluetoothPermissionDescriptor : PermissionDescriptor {
  DOMString deviceId;
  // These match RequestDeviceOptions.
  sequence<BluetoothLEScanFilterInit> filters;
  sequence<BluetoothServiceUUID> optionalServices = [];
  boolean acceptAllDevices = false;
};

[Exposed=Window, Pref="dom_bluetooth_enabled"]
interface BluetoothPermissionResult : PermissionStatus {
  // attribute FrozenArray<BluetoothDevice> devices;
  // Workaround until FrozenArray get implemented.
  sequence<BluetoothDevice> devices();
};
