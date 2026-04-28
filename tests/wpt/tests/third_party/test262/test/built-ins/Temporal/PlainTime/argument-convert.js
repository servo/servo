// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime
description: PlainTime constructor with non-integer arguments.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

TemporalHelpers.assertPlainTime(new Temporal.PlainTime(11.9, 12.8, 13.7, 14.6, 15.5, 1.999999),
  11, 12, 13, 14, 15, 1, "positive fractional");

TemporalHelpers.assertPlainTime(new Temporal.PlainTime(null, 1, 2, 3, 4, 5),
  0, 1, 2, 3, 4, 5, "null defaults to zero");

TemporalHelpers.assertPlainTime(new Temporal.PlainTime(false, true),
  0, 1, 0, 0, 0, 0, "boolean defaults");

TemporalHelpers.assertPlainTime(new Temporal.PlainTime(11, 24, undefined),
  11, 24, 0, 0, 0, 0, "undefined defaults to 0");

TemporalHelpers.assertPlainTime(new Temporal.PlainTime("11.9", "12.8", "13.7", "14.6", "15.5", "1.999999"),
  11, 12, 13, 14, 15, 1, "fractional strings");

for (const invalid of [Symbol(), 1n]) {
  assert.throws(TypeError, () => new Temporal.PlainTime(invalid, 2, 3, 4, 5, 6), `hour ${typeof invalid}`);
  assert.throws(TypeError, () => new Temporal.PlainTime(1, invalid, 3, 4, 5, 6), `minute ${typeof invalid}`);
  assert.throws(TypeError, () => new Temporal.PlainTime(1, 2, invalid, 4, 5, 6), `second ${typeof invalid}`);
  assert.throws(TypeError, () => new Temporal.PlainTime(1, 2, 3, invalid, 5, 6), `millisecond ${typeof invalid}`);
  assert.throws(TypeError, () => new Temporal.PlainTime(1, 2, 3, 4, invalid, 6), `microsecond ${typeof invalid}`);
  assert.throws(TypeError, () => new Temporal.PlainTime(1, 2, 3, 4, 5, invalid), `nanosecond ${typeof invalid}`);
}

for (const invalid of ["invalid"]) {
  assert.throws(RangeError, () => new Temporal.PlainTime(invalid, 2, 3, 4, 5, 6), `hour ${typeof invalid}`);
  assert.throws(RangeError, () => new Temporal.PlainTime(1, invalid, 3, 4, 5, 6), `minute ${typeof invalid}`);
  assert.throws(RangeError, () => new Temporal.PlainTime(1, 2, invalid, 4, 5, 6), `second ${typeof invalid}`);
  assert.throws(RangeError, () => new Temporal.PlainTime(1, 2, 3, invalid, 5, 6), `millisecond ${typeof invalid}`);
  assert.throws(RangeError, () => new Temporal.PlainTime(1, 2, 3, 4, invalid, 6), `microsecond ${typeof invalid}`);
  assert.throws(RangeError, () => new Temporal.PlainTime(1, 2, 3, 4, 5, invalid), `nanosecond ${typeof invalid}`);
}

