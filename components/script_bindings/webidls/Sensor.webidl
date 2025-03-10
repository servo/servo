/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://w3c.github.io/sensors/#the-sensor-interface
[Unimplemented, SecureContext, Exposed=(DedicatedWorker, Window)]
interface Sensor : EventTarget {
  readonly attribute boolean activated;
  readonly attribute boolean hasReading;
  readonly attribute DOMHighResTimeStamp? timestamp;
  undefined start();
  undefined stop();
  attribute EventHandler onreading;
  attribute EventHandler onactivate;
  attribute EventHandler onerror;
};

dictionary SensorOptions {
  double frequency;
};
