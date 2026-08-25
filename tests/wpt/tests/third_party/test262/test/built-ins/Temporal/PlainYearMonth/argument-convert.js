// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth
description: PlainYearMonth constructor with non-integer arguments.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

TemporalHelpers.assertPlainYearMonth(new Temporal.PlainYearMonth(2020.6, 11.7),
  2020, 11, "M11", "positive fractional");

TemporalHelpers.assertPlainYearMonth(new Temporal.PlainYearMonth(-2020.6, 11.7),
  -2020, 11, "M11", "negative fractional");

TemporalHelpers.assertPlainYearMonth(new Temporal.PlainYearMonth(null, 11),
  0, 11, "M11", "null defaults to zero");

TemporalHelpers.assertPlainYearMonth(new Temporal.PlainYearMonth(false, true),
  0, 1, "M01", "boolean defaults");

TemporalHelpers.assertPlainYearMonth(new Temporal.PlainYearMonth("2020.6", "11.7"),
  2020, 11, "M11", "fractional strings");

for (const invalid of [Symbol(), 1n]) {
  assert.throws(TypeError, () => new Temporal.PlainYearMonth(invalid, 11), `year ${typeof invalid}`);
  assert.throws(TypeError, () => new Temporal.PlainYearMonth(2020, invalid), `month ${typeof invalid}`);
}

for (const invalid of [undefined, "invalid"]) {
  assert.throws(RangeError, () => new Temporal.PlainYearMonth(invalid, 11), `year ${typeof invalid}`);
  assert.throws(RangeError, () => new Temporal.PlainYearMonth(2020, invalid), `month ${typeof invalid}`);
}
