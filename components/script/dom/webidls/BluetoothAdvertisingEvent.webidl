/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://webbluetoothcg.github.io/web-bluetooth/#advertising-events

/*interface BluetoothManufacturerDataMap {
  readonly maplike<unsigned short, DataView>;
};
interface BluetoothServiceDataMap {
  readonly maplike<UUID, DataView>;
};*/
[Pref="dom.bluetooth.enabled", Constructor(DOMString type, BluetoothAdvertisingEventInit init)]
interface BluetoothAdvertisingEvent : Event {
  [SameObject]
  readonly attribute BluetoothDevice device;
  // readonly attribute FrozenArray<UUID> uuids;
  readonly attribute DOMString? name;
  readonly attribute unsigned short? appearance;
  readonly attribute byte? txPower;
  readonly attribute byte? rssi;
  // [SameObject]
  // readonly attribute BluetoothManufacturerDataMap manufacturerData;
  // [SameObject]
  // readonly attribute BluetoothServiceDataMap serviceData;
};
dictionary BluetoothAdvertisingEventInit : EventInit {
  required BluetoothDevice device;
  // sequence<(DOMString or unsigned long)> uuids;
  DOMString name;
  unsigned short appearance;
  byte txPower;
  byte rssi;
  // Map manufacturerData;
  // Map serviceData;
};
