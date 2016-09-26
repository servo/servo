/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// http://webbluetoothcg.github.io/web-bluetooth/#bluetoothremotegattdescriptor

[Pref="dom.bluetooth.enabled", Exposed=(Window,Worker)]
interface BluetoothRemoteGATTDescriptor {
  readonly attribute BluetoothRemoteGATTCharacteristic characteristic;
  readonly attribute DOMString uuid;
  readonly attribute ByteString? value;
  [Throws]
  Promise<ByteString> readValue();
  //Promise<DataView> readValue();
  [Throws]
  Promise<void> writeValue(sequence<octet> value);
  //Promise<void> writeValue(BufferSource value);
};
