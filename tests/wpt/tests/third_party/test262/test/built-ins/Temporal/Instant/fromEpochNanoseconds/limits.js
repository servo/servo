// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.fromepochnanoseconds
description: >
  Throws a RangeError if the input is not a valid epoch nanoseconds value.
info: |
  Temporal.Instant.fromEpochNanoseconds ( epochNanoseconds )

  ...
  2. If IsValidEpochNanoseconds(epochNanoseconds) is false, throw a RangeError exception.
  ...
includes: [temporalHelpers.js]
features: [Temporal]
---*/

var limit = 8640000000000000000000n;

assert.throws(RangeError, () => Temporal.Instant.fromEpochNanoseconds(-limit - 1n));
assert.throws(RangeError, () => Temporal.Instant.fromEpochNanoseconds(limit + 1n));
TemporalHelpers.assertInstantsEqual(Temporal.Instant.fromEpochNanoseconds(-limit),
                                    Temporal.Instant.from("-271821-04-20T00:00:00Z"));
TemporalHelpers.assertInstantsEqual(Temporal.Instant.fromEpochNanoseconds(limit),
                                    Temporal.Instant.from("+275760-09-13T00:00:00Z"));
