/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//https://webbluetoothcg.github.io/web-bluetooth/#bluetoothremotegattserver

[Pref="dom.bluetooth.enabled", Exposed=(Window,Worker)]
interface BluetoothRemoteGATTServer {
  readonly attribute BluetoothDevice device;
  readonly attribute boolean connected;
  [Throws]
  Promise<BluetoothRemoteGATTServer> connect();
  [Throws]
  void disconnect();
  [Throws]
  Promise<BluetoothRemoteGATTService> getPrimaryService(BluetoothServiceUUID service);
  [Throws]
  Promise<sequence<BluetoothRemoteGATTService>> getPrimaryServices(optional BluetoothServiceUUID service);
};
