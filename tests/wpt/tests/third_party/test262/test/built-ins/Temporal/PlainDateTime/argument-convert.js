// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime
description: PlainDateTime constructor with non-integer arguments.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

TemporalHelpers.assertPlainDateTime(new Temporal.PlainDateTime(2020.6, 11.7, 24.1, 11.9, 12.8, 13.7, 14.6, 15.5, 1.999999),
  2020, 11, "M11", 24, 11, 12, 13, 14, 15, 1, "positive fractional");

TemporalHelpers.assertPlainDateTime(new Temporal.PlainDateTime(-2020.6, 11.7, 24.1, 11.9, 12.8, 13.7, 14.6, 15.5, 1.999999),
  -2020, 11, "M11", 24, 11, 12, 13, 14, 15, 1, "negative fractional");

TemporalHelpers.assertPlainDateTime(new Temporal.PlainDateTime(null, 11, 24, 1, 2, 3, 4, 5, 6),
  0, 11, "M11", 24, 1, 2, 3, 4, 5, 6, "null defaults to zero");

TemporalHelpers.assertPlainDateTime(new Temporal.PlainDateTime(false, 11, 24, true),
  0, 11, "M11", 24, 1, 0, 0, 0, 0, 0, "boolean defaults");

TemporalHelpers.assertPlainDateTime(new Temporal.PlainDateTime(2020, 11, 24, undefined),
  2020, 11, "M11", 24, 0, 0, 0, 0, 0, 0, "undefined hour defaults to 0");

TemporalHelpers.assertPlainDateTime(new Temporal.PlainDateTime("2020.6", "11.7", "24.1", "11.9", "12.8", "13.7", "14.6", "15.5", "1.999999"),
  2020, 11, "M11", 24, 11, 12, 13, 14, 15, 1, "fractional strings");

for (const invalid of [Symbol(), 1n]) {
  assert.throws(TypeError, () => new Temporal.PlainDateTime(invalid, 11, 24), `year ${typeof invalid}`);
  assert.throws(TypeError, () => new Temporal.PlainDateTime(2020, invalid, 24), `month ${typeof invalid}`);
  assert.throws(TypeError, () => new Temporal.PlainDateTime(2020, 11, invalid), `day ${typeof invalid}`);
}

for (const invalid of [undefined, "invalid"]) {
  assert.throws(RangeError, () => new Temporal.PlainDateTime(invalid, 11, 24), `year ${typeof invalid}`);
  assert.throws(RangeError, () => new Temporal.PlainDateTime(2020, invalid, 24), `month ${typeof invalid}`);
  assert.throws(RangeError, () => new Temporal.PlainDateTime(2020, 11, invalid), `day ${typeof invalid}`);
}
