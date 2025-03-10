/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://w3c.github.io/sensors/#the-sensor-interface
[Unimplemented, SecureContext, Exposed=(DedicatedWorker, Window)]
interface Sensor : EventTarget {
  [Unimplemented] readonly attribute boolean activated;
  [Unimplemented] readonly attribute boolean hasReading;
  [Unimplemented] readonly attribute DOMHighResTimeStamp? timestamp;
  [Unimplemented] undefined start();
  [Unimplemented] undefined stop();
  [Unimplemented] attribute EventHandler onreading;
  [Unimplemented] attribute EventHandler onactivate;
  [Unimplemented] attribute EventHandler onerror;
};

dictionary SensorOptions {
  double frequency;
};
