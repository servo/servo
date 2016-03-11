/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// http://webbluetoothcg.github.io/web-bluetooth/#bluetoothremotegattdescriptor

interface BluetoothRemoteGATTDescriptor {
  readonly attribute BluetoothRemoteGATTCharacteristic characteristic;
  readonly attribute DOMString uuid;
  readonly attribute ByteString? value;

  ByteString readValue();
  //Promise<DataView> readValue();
  //Promise<void> writeValue(BufferSource value);
};
