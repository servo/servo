/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://webbluetoothcg.github.io/web-bluetooth/#bluetoothremotegattcharacteristic

interface BluetoothRemoteGATTCharacteristic {
  readonly attribute BluetoothRemoteGATTService service;
  readonly attribute DOMString uuid;
  readonly attribute BluetoothCharacteristicProperties properties;
  readonly attribute ByteString? value;
  [Throws]
  BluetoothRemoteGATTDescriptor getDescriptor((DOMString or unsigned long) descriptor);
  [Throws]
  sequence<BluetoothRemoteGATTDescriptor> getDescriptors(optional (DOMString or unsigned long) descriptor);
  //Promise<BluetoothRemoteGATTDescriptor> getDescriptor(BluetoothDescriptorUUID descriptor);
  //Promise<sequence<BluetoothRemoteGATTDescriptor>>
  //getDescriptors(optional BluetoothDescriptorUUID descriptor);
  [Throws]
  ByteString readValue();
  //Promise<DataView> readValue();
  [Throws]
  void writeValue(sequence<octet> value);
  //Promise<void> writeValue(BufferSource value);
  //Promise<void> startNotifications();
  //Promise<void> stopNotifications();
};

//BluetootRemoteGATTCharacteristic implements EventTarget;
//BluetootRemoteGATTCharacteristic implements CharacteristicEventHandlers;
