// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.since
description: >
  Returns a blank duration when epoch nanoseconds are equal.
info: |
  Temporal.ZonedDateTime.prototype.since ( other [ , options ] )

  3. Return ? DifferenceTemporalZonedDateTime(since, zonedDateTime, other, options).

  DifferenceTemporalZonedDateTime ( operation, zonedDateTime, other, options )

  ...
  8. If zonedDateTime.[[EpochNanoseconds]] = other.[[EpochNanoseconds]], then
    a. Return ! CreateTemporalDuration(0, 0, 0, 0, 0, 0, 0, 0, 0, 0).
  ...
includes: [temporalHelpers.js]
features: [Temporal]
---*/

var epochNanoseconds = [
  0n,
  1n,
  -1n,
];

var timeZones = [
  "UTC",
  "+00",
  "+01",
  "-01",
];

var units = [
  "years",
  "months",
  "weeks",
  "days",
  "hours",
  "minutes",
  "seconds",
  "milliseconds",
  "microseconds",
  "nanoseconds",
];

for (var timeZone of timeZones) {
  for (var epochNs of epochNanoseconds) {
    var zdt = new Temporal.ZonedDateTime(epochNs, timeZone);
    var other = new Temporal.ZonedDateTime(epochNs, timeZone);

    for (var i = 0; i < units.length; ++i) {
      for (var j = i; j < units.length; ++j) {
        var options = {
          largestUnit: units[i],
          smallestUnit: units[j],
        };

        TemporalHelpers.assertDuration(
          zdt.since(other, options),
          0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
          `epochNs = ${epochNs}, timeZone = ${timeZone}, options = ${JSON.stringify(options)})`
        );
      }
    }
  }
}
