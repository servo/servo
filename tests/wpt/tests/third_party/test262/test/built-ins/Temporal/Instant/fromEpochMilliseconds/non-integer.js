// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.fromepochmilliseconds
description: >
  RangeError thrown if input doesn't convert.
info: |
  Temporal.Instant.fromEpochMilliseconds ( epochMilliseconds )

  ...
  2. Set epochMilliseconds to ? NumberToBigInt(epochMilliseconds).
  ...

  NumberToBigInt ( number )

  1. If number is not an integral Number, throw a RangeError exception.
  ...
features: [Temporal]
---*/

assert.throws(RangeError, () => Temporal.Instant.fromEpochMilliseconds(), "undefined");
assert.throws(RangeError, () => Temporal.Instant.fromEpochMilliseconds(undefined), "undefined");
assert.throws(RangeError, () => Temporal.Instant.fromEpochMilliseconds(Infinity), "Infinity");
assert.throws(RangeError, () => Temporal.Instant.fromEpochMilliseconds(-Infinity), "-Infinity");
assert.throws(RangeError, () => Temporal.Instant.fromEpochMilliseconds(NaN), "NaN");
assert.throws(RangeError, () => Temporal.Instant.fromEpochMilliseconds(1.3), "1.3");
assert.throws(RangeError, () => Temporal.Instant.fromEpochMilliseconds(-0.5), "-0.5");
