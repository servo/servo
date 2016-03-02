/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//https://webbluetoothcg.github.io/web-bluetooth/#bluetoothremotegattserver

interface BluetoothRemoteGATTServer {
  readonly attribute BluetoothDevice device;
  readonly attribute boolean connected;
  BluetoothRemoteGATTServer connect();
  void disconnect();
  BluetoothRemoteGATTService? getPrimaryService();
  //Promise<BluetoothRemoteGATTService> getPrimaryService(BluetoothServiceUUID service);
  //Promise<sequence<BluetoothRemoteGATTService>>getPrimaryServices(optional BluetoothServiceUUID service);
  //Promise<BluetoothRemoteGATTServer> connect();
};
