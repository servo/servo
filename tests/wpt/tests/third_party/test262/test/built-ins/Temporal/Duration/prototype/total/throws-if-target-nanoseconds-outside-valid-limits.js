// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.total
description: >
  Throws RangeError when targetEpochNs is not a valid epoch nanoseconds value.
info: |
  Temporal.Duration.prototype.total ( totalOf )

  ...
  11. If zonedRelativeTo is not undefined, then
    ...
    e. Let targetEpochNs be ? AddZonedDateTime(relativeEpochNs, timeZone, calendar,
       internalDuration, constrain).
    ...

  AddZonedDateTime ( epochNanoseconds, timeZone, calendar, duration, overflow )

  1. If DateDurationSign(duration.[[Date]]) = 0, then
    a. Return ? AddInstant(epochNanoseconds, duration.[[Time]]).
  ...
features: [Temporal]
---*/

var duration = new Temporal.Duration(0, 0, 0, 0, 0, 0, 0, 0, 0, 1);

var relativeTo = new Temporal.ZonedDateTime(864n * 10n**19n, "UTC");

var totalOf = {
  unit: "nanoseconds",
  relativeTo,
};

assert.throws(RangeError, () => duration.total(totalOf));
