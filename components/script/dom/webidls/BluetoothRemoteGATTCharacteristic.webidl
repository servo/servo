/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://webbluetoothcg.github.io/web-bluetooth/#bluetoothremotegattcharacteristic

[Pref="dom.bluetooth.enabled", Exposed=(Window,Worker)]
interface BluetoothRemoteGATTCharacteristic {
  readonly attribute BluetoothRemoteGATTService service;
  readonly attribute DOMString uuid;
  readonly attribute BluetoothCharacteristicProperties properties;
  readonly attribute ByteString? value;
  [Throws]
  Promise<BluetoothRemoteGATTDescriptor> getDescriptor(BluetoothDescriptorUUID descriptor);
  [Throws]
  Promise<sequence<BluetoothRemoteGATTDescriptor>>
  getDescriptors(optional BluetoothDescriptorUUID descriptor);
  [Throws]
  Promise<ByteString> readValue();
  //Promise<DataView> readValue();
  [Throws]
  Promise<void> writeValue(sequence<octet> value);
  //Promise<void> writeValue(BufferSource value);
  //Promise<void> startNotifications();
  //Promise<void> stopNotifications();
};

//BluetootRemoteGATTCharacteristic implements EventTarget;
//BluetootRemoteGATTCharacteristic implements CharacteristicEventHandlers;
