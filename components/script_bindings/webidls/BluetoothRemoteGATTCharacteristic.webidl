/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// skip-unless CARGO_FEATURE_BLUETOOTH

// https://webbluetoothcg.github.io/web-bluetooth/#bluetoothremotegattcharacteristic

[Exposed=Window, Pref="dom_bluetooth_enabled"]
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
  Promise<undefined> writeValue(BufferSource value);
  Promise<BluetoothRemoteGATTCharacteristic> startNotifications();
  Promise<BluetoothRemoteGATTCharacteristic> stopNotifications();
};

interface mixin CharacteristicEventHandlers {
  attribute EventHandler oncharacteristicvaluechanged;
};

// BluetoothRemoteGATTCharacteristic includes EventTarget;
BluetoothRemoteGATTCharacteristic includes CharacteristicEventHandlers;
