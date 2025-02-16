/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// skip-unless CARGO_FEATURE_BLUETOOTH

// https://webbluetoothcg.github.io/web-bluetooth/#bluetoothdevice

[Exposed=Window, Pref="dom_bluetooth_enabled"]
interface BluetoothDevice : EventTarget {
  readonly attribute DOMString id;
  readonly attribute DOMString? name;
  readonly attribute BluetoothRemoteGATTServer? gatt;

  Promise<undefined> watchAdvertisements();
  undefined unwatchAdvertisements();
  readonly attribute boolean watchingAdvertisements;
};

interface mixin BluetoothDeviceEventHandlers {
  attribute EventHandler ongattserverdisconnected;
};

// BluetoothDevice includes EventTarget;
BluetoothDevice includes BluetoothDeviceEventHandlers;
// BluetoothDevice includes CharacteristicEventHandlers;
// BluetoothDevice includes ServiceEventHandlers;
