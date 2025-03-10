/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://w3c.github.io/accelerometer/#accelerometer-interface
[Unimplemented, SecureContext, Exposed=Window]
interface Accelerometer : Sensor {
  [Unimplemented] constructor(optional AccelerometerSensorOptions options = {});
  [Unimplemented] readonly attribute double? x;
  [Unimplemented] readonly attribute double? y;
  [Unimplemented] readonly attribute double? z;
};

enum AccelerometerLocalCoordinateSystem { "device", "screen" };

dictionary AccelerometerSensorOptions : SensorOptions {
  AccelerometerLocalCoordinateSystem referenceFrame = "device";
};
