/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://webbluetoothcg.github.io/web-bluetooth/#bluetoothremotegattcharacteristic

[Pref="dom.bluetooth.enabled"]
interface BluetoothRemoteGATTCharacteristic : EventTarget {
  [SameObject]
  readonly attribute BluetoothRemoteGATTService service;
  readonly attribute DOMString uuid;
  readonly attribute BluetoothCharacteristicProperties properties;
  readonly attribute ByteString? value;
  Promise<BluetoothRemoteGATTDescriptor> getDescriptor(BluetoothDescriptorUUID descriptor);
  Promise<sequence<BluetoothRemoteGATTDescriptor>>
  getDescriptors(optional BluetoothDescriptorUUID descriptor);
  Promise<ByteString> readValue();
  //Promise<DataView> readValue();
  Promise<void> writeValue(sequence<octet> value);
  //Promise<void> writeValue(BufferSource value);
  Promise<BluetoothRemoteGATTCharacteristic> startNotifications();
  Promise<BluetoothRemoteGATTCharacteristic> stopNotifications();
};

[NoInterfaceObject]
interface CharacteristicEventHandlers {
  attribute EventHandler oncharacteristicvaluechanged;
};

// BluetoothRemoteGATTCharacteristic implements EventTarget;
BluetoothRemoteGATTCharacteristic implements CharacteristicEventHandlers;
