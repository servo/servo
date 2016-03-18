/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//https://webbluetoothcg.github.io/web-bluetooth/#bluetoothadvertisingdata

/*interface BluetoothManufacturerDataMap {
  readonly maplike<unsigned short, DataView>;
};

interface BluetoothServiceDataMap {
  readonly maplike<UUID, DataView>;
};*/

[Pref="dom.bluetooth.enabled"]
interface BluetoothAdvertisingData {
  readonly attribute unsigned short? appearance;
  readonly attribute byte? txPower;
  readonly attribute byte? rssi;
  // readonly attribute BluetoothManufacturerDataMap manufacturerData;
  // readonly attribute BluetoothServiceDataMap serviceData;
};
