// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.add
description: |
  Temporal.Instant.prototype.add() throws RangeError when the duration has
  non-zero years, months, weeks, or days.
info: |
  1. Let instant be the this value.
  3. Let duration be ? ToLimitedTemporalDuration(temporalDurationLike, « "years", "months", "weeks", "days" »).
features: [Temporal]
---*/

const inst = new Temporal.Instant(500000n);
assert.throws(RangeError, () => inst.add(new Temporal.Duration(1)),
    "should throw RangeError when the duration has non-zero years (positive)");
assert.throws(RangeError, () => inst.add(new Temporal.Duration(0, 2)),
    "should throw RangeError when the duration has non-zero months (positive)");
assert.throws(RangeError, () => inst.add(new Temporal.Duration(0, 0, 3)),
    "should throw RangeError when the duration has non-zero weeks (positive)");
assert.throws(RangeError, () => inst.add(new Temporal.Duration(0, 0, 0, 4)),
    "should throw RangeError when the duration has non-zero days (positive)");
assert.throws(RangeError, () => inst.add(new Temporal.Duration(-1)),
    "should throw RangeError when the duration has non-zero years (negative)");
assert.throws(RangeError, () => inst.add(new Temporal.Duration(0, -2)),
    "should throw RangeError when the duration has non-zero months (negative)");
assert.throws(RangeError, () => inst.add(new Temporal.Duration(0, 0, -3)),
    "should throw RangeError when the duration has non-zero weeks (negative)");
assert.throws(RangeError, () => inst.add(new Temporal.Duration(0, 0, 0, -4)),
    "should throw RangeError when the duration has non-zero days (negative)");
