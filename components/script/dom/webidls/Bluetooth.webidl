/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://webbluetoothcg.github.io/web-bluetooth/#bluetooth

[Pref="dom.bluetooth.enabled"]
interface Bluetooth {
    // Promise<BluetoothDevice> requestDevice(RequestDeviceOptions options);
    BluetoothDevice? requestDevice(/*RequestDeviceOptions options*/);
};

// Bluetooth implements EventTarget;
// Bluetooth implements CharacteristicEventHandlers;
// Bluetooth implements ServiceEventHandlers;
