// Copyright (C) 2022 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.tozoneddatetime
description: Values near the date/time limit and a fixed offset.
features: [Temporal, exponentiation]
---*/

const oneHour = 1n * 60n * 60n * 1000n**3n;

const minDt = new Temporal.PlainDateTime(-271821, 4, 19, 1, 0, 0, 0, 0, 0);
const minValidDt = new Temporal.PlainDateTime(-271821, 4, 20, 0, 0, 0, 0, 0, 0);
const maxDt = new Temporal.PlainDateTime(275760, 9, 13, 0, 0, 0, 0, 0, 0);

// Try the minimum date-time.
assert.throws(RangeError, () => minDt.toZonedDateTime("+00"));
assert.throws(RangeError, () => minDt.toZonedDateTime("+01"));
assert.throws(RangeError, () => minDt.toZonedDateTime("-01"));

// Try the minimum valid date-time.
["earlier", "later"].forEach((disambiguation) => {
  const zdt = minValidDt.toZonedDateTime("+00", { disambiguation });
  assert.sameValue(zdt.epochNanoseconds, -86_40000_00000_00000_00000n);
});

["earlier", "later"].forEach((disambiguation) => {
  const zdt = minValidDt.toZonedDateTime("-01", { disambiguation });
  assert.sameValue(zdt.epochNanoseconds, -86_40000_00000_00000_00000n + oneHour);
});

assert.throws(RangeError, () => minValidDt.toZonedDateTime("+01"));

// Try the maximum valid date-time.
["earlier", "later"].forEach((disambiguation) => {
  const zdt = maxDt.toZonedDateTime("+00");
  assert.sameValue(zdt.epochNanoseconds, 86_40000_00000_00000_00000n);
});

["earlier", "later"].forEach((disambiguation) => {
  const zdt = maxDt.toZonedDateTime("+01");
  assert.sameValue(zdt.epochNanoseconds, 86_40000_00000_00000_00000n - oneHour);
});

assert.throws(RangeError, () => maxDt.toZonedDateTime("-01"));
