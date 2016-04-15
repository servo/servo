/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://webbluetoothcg.github.io/web-bluetooth/#bluetooth

dictionary BluetoothScanFilter {
  sequence<BluetoothServiceUUID> services;
  DOMString name;
  DOMString namePrefix;
};

dictionary RequestDeviceOptions {
  required sequence<BluetoothScanFilter> filters;
  sequence<BluetoothServiceUUID> optionalServices /*= []*/;
};

interface Bluetooth {
    // Promise<BluetoothDevice> requestDevice(RequestDeviceOptions options);
    [Throws]
    BluetoothDevice requestDevice(RequestDeviceOptions options);
};

// Bluetooth implements EventTarget;
// Bluetooth implements CharacteristicEventHandlers;
// Bluetooth implements ServiceEventHandlers;
