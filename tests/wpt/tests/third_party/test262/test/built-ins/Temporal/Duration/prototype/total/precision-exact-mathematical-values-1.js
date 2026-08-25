// Copyright (C) 2022 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.total
description: >
  RoundDuration computes on exact mathematical values.
features: [Temporal]
---*/

// Return the next Number value in direction to +Infinity.
function nextUp(num) {
  if (!Number.isFinite(num)) {
    return num;
  }
  if (num === 0) {
    return Number.MIN_VALUE;
  }

  var f64 = new Float64Array([num]);
  var u64 = new BigUint64Array(f64.buffer);
  u64[0] += (num < 0 ? -1n : 1n);
  return f64[0];
}

// Return the next Number value in direction to -Infinity.
function nextDown(num) {
  if (!Number.isFinite(num)) {
    return num;
  }
  if (num === 0) {
    return -Number.MIN_VALUE;
  }

  var f64 = new Float64Array([num]);
  var u64 = new BigUint64Array(f64.buffer);
  u64[0] += (num < 0 ? 1n : -1n);
  return f64[0];
}

let duration = Temporal.Duration.from({
  hours: 4000,
  nanoseconds: 1,
});

let total = duration.total({unit: "hours"});

// From RoundDuration():
//
// 7. Let fractionalSeconds be nanoseconds × 10^-9 + microseconds × 10^-6 + milliseconds × 10^-3 + seconds.
// = nanoseconds × 10^-9
// = 1 × 10^-9
// = 10^-9
// = 0.000000001
//
// 13.a. Let fractionalHours be (fractionalSeconds / 60 + minutes) / 60 + hours.
// = (fractionalSeconds / 60) / 60 + 4000
// = 0.000000001 / 3600 + 4000
//
// 13.b. Set hours to RoundNumberToIncrement(fractionalHours, increment, roundingMode).
// = trunc(fractionalHours)
// = trunc(0.000000001 / 3600 + 4000)
// = 4000
//
// 13.c. Set remainder to fractionalHours - hours.
// = fractionalHours - hours
// = 0.000000001 / 3600 + 4000 - 4000
// = 0.000000001 / 3600
//
// From Temporal.Duration.prototype.total ( options ):
//
// 18. If unit is "hours", then let whole be roundResult.[[Hours]].
// ...
// 24. Return whole + roundResult.[[Remainder]].
//
// |whole| is 4000 and the remainder is (0.000000001 / 3600).
//
//   0.000000001 / 3600
// = (1 / 10^9) / 3600
// = (1 / 36) / 10^11
// = 0.02777.... / 10^11
// = 0.0000000000002777...
//
// 4000.0000000000002777... can't be represented exactly, the next best approximation
// is 4000.0000000000005.

const expected = 4000.0000000000005;
assert.sameValue(expected, 4000.0000000000002777, "the float representation of the result is 4000.0000000000005");

// The next Number in direction -Infinity is less precise.
assert.sameValue(nextDown(expected), 4000, "the next Number in direction -Infinity is less precise");

// The next Number in direction +Infinity is less precise.
assert.sameValue(nextUp(expected), 4000.000000000001, "the next Number in direction +Infinity is less precise");

assert.sameValue(total, expected, "return value of total()");
