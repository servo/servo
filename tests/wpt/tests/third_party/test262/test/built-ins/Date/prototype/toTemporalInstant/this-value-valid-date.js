// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-date.prototype.totemporalinstant
description: >
  Return value for valid dates.
info: |
  Date.prototype.toTemporalInstant ( )

  ...
  3. Let t be dateObject.[[DateValue]].
  4. Let ns be ? NumberToBigInt(t) × ℤ(10**6).
  5. Return ! CreateTemporalInstant(ns).
features: [Temporal, BigInt]
---*/

assert.sameValue(
  new Date(0).toTemporalInstant().epochNanoseconds,
  0n,
  "the (Unix) epoch"
);

assert.sameValue(
  new Date(123_456_789).toTemporalInstant().epochNanoseconds,
  123_456_789_000_000n,
  "date after the (Unix) epoch"
);

assert.sameValue(
  new Date(-123_456_789).toTemporalInstant().epochNanoseconds,
  -123_456_789_000_000n,
  "date before the (Unix) epoch"
);

assert.sameValue(
  new Date(-8.64e15).toTemporalInstant().epochNanoseconds,
  -8640_000_000_000_000_000_000n,
  "start of time"
);

assert.sameValue(
  new Date(8.64e15).toTemporalInstant().epochNanoseconds,
  8640_000_000_000_000_000_000n,
  "end of time"
);
