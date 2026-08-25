// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.add
description: Basic functionality of Temporal.Instant.prototype.add()
info: |
  1. Let instant be the this value.
  2. Perform ? RequireInternalSlot(instant, [[InitializedTemporalInstant]]).
  3. Let duration be ? ToLimitedTemporalDuration(temporalDurationLike, « "years", "months", "weeks", "days" »).
  4. Let ns be ? AddInstant(instant.[[EpochNanoseconds]], duration.[[Hours]], duration.[[Minutes]], duration.[[Seconds]], duration.[[Milliseconds]], duration.[[Microseconds]], duration.[[Nanoseconds]]).
  5. Return ! CreateTemporalInstant(ns).
features: [Temporal]
---*/

const inst = new Temporal.Instant(50000n);

let result = inst.add(new Temporal.Duration(0, 0, 0, 0, 0, 0, 0, 3, 2, 1));
assert.sameValue(
  3052001n,
  result.epochNanoseconds,
  "add positive sub-seconds"
);

result = inst.add(new Temporal.Duration(0, 0, 0, 0, 0, 0, 4, 3, 2, 1));
assert.sameValue(
  BigInt(4 * 1e9) + 3052001n,
  result.epochNanoseconds,
  "add positive seconds"
);

result = inst.add(new Temporal.Duration(0, 0, 0, 0, 0, 5, 4, 3, 2, 1));
assert.sameValue(
  BigInt(5 * 60 + 4) * 1000000000n + 3052001n,
  result.epochNanoseconds,
  "add positive minutes"
);

result = inst.add(new Temporal.Duration(0, 0, 0, 0, 6, 5, 4, 3, 2, 1));
assert.sameValue(
  BigInt(6 * 3600 + 5 * 60 + 4) * 1000000000n + 3052001n,
  result.epochNanoseconds,
  "add positive hours"
);

result = inst.add(new Temporal.Duration(0, 0, 0, 0, 0, 0, 0, -3, -2, -1));
assert.sameValue(
  -2952001n,
  result.epochNanoseconds,
  "add negative sub-seconds"
);

result = inst.add(new Temporal.Duration(0, 0, 0, 0, 0, 0, -4, -3, -2, -1));
assert.sameValue(
  BigInt(-4 * 1e9) - 2952001n,
  result.epochNanoseconds,
  "add negative seconds"
);

result = inst.add(new Temporal.Duration(0, 0, 0, 0, 0, -5, -4, -3, -2, -1));
assert.sameValue(
  BigInt(5 * 60 + 4) * -1000000000n - 2952001n,
  result.epochNanoseconds,
  "add negative minutes"
);

result = inst.add(new Temporal.Duration(0, 0, 0, 0, -6, -5, -4, -3, -2, -1));
assert.sameValue(
  BigInt(6 * 3600 + 5 * 60 + 4) * -1000000000n - 2952001n,
  result.epochNanoseconds,
  "add negative hours"
);
