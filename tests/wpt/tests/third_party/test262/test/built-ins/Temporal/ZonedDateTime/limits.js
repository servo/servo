// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime
description: >
  RangeError thrown when epoch nanoseconds not valid.
info: |
  Temporal.ZonedDateTime ( epochNanoseconds, timeZone [ , calendar ] )

  2. Set epochNanoseconds to ? ToBigInt(epochNanoseconds).
  3. If IsValidEpochNanoseconds(epochNanoseconds) is false, throw a RangeError exception.
  ...
features: [Temporal]
---*/

var nsMaxInstant = 864n * 10n ** 19n;
var nsMinInstant = -nsMaxInstant;

var invalidEpochNanoseconds = [
  nsMaxInstant + 1n,
  nsMinInstant - 1n,
  2n ** 128n,
  -(2n ** 128n),
];

var timeZones = [
  "UTC",
  "+00",
  "+01",
  "-01",
];

for (var timeZone of timeZones) {
  for (var epochNs of invalidEpochNanoseconds) {
    assert.throws(
      RangeError,
      () => new Temporal.ZonedDateTime(epochNs, timeZone),
      `epochNs = ${epochNs}, timeZone = ${timeZone}`
    );
  }
}
