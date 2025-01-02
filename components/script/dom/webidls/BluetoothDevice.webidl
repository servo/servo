/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://webbluetoothcg.github.io/web-bluetooth/#bluetoothdevice

[Exposed=Window, Pref="dom.bluetooth.enabled"]
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
