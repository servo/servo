/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// http://webbluetoothcg.github.io/web-bluetooth/#bluetoothremotegattdescriptor

[Exposed=Window, Pref="dom.bluetooth.enabled"]
interface BluetoothRemoteGATTDescriptor {
  [SameObject]
  readonly attribute BluetoothRemoteGATTCharacteristic characteristic;
  readonly attribute DOMString uuid;
  readonly attribute ByteString? value;
  Promise<ByteString> readValue();
  //Promise<DataView> readValue();
  Promise<undefined> writeValue(BufferSource value);
};
