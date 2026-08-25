/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// skip-unless CARGO_FEATURE_BLUETOOTH

//https://webbluetoothcg.github.io/web-bluetooth/#bluetoothremotegattserver

[Exposed=Window, Pref="dom_bluetooth_enabled"]
interface BluetoothRemoteGATTServer {
  [SameObject]
  readonly attribute BluetoothDevice device;
  readonly attribute boolean connected;
  Promise<BluetoothRemoteGATTServer> connect();
  [Throws]
  undefined disconnect();
  Promise<BluetoothRemoteGATTService> getPrimaryService(BluetoothServiceUUID service);
  Promise<sequence<BluetoothRemoteGATTService>> getPrimaryServices(optional BluetoothServiceUUID service);
};
