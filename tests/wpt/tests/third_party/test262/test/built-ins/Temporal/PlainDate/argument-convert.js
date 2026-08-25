// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate
description: PlainDate constructor with non-integer arguments.
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

TemporalHelpers.assertPlainDate(new Temporal.PlainDate(2020.6, 11.7, 24.1),
  2020, 11, "M11", 24, "positive fractional");

TemporalHelpers.assertPlainDate(new Temporal.PlainDate(-2020.6, 11.7, 24.1),
  -2020, 11, "M11", 24, "negative fractional");

TemporalHelpers.assertPlainDate(new Temporal.PlainDate(null, 11, 24),
  0, 11, "M11", 24, "null");

TemporalHelpers.assertPlainDate(new Temporal.PlainDate(true, 11, 24),
  1, 11, "M11", 24, "boolean");

TemporalHelpers.assertPlainDate(new Temporal.PlainDate("2020.6", "11.7", "24.1"),
  2020, 11, "M11", 24, "fractional strings");

for (const invalid of [Symbol(), 1n]) {
  assert.throws(TypeError, () => new Temporal.PlainDate(invalid, 11, 24), `year ${typeof invalid}`);
  assert.throws(TypeError, () => new Temporal.PlainDate(2020, invalid, 24), `month ${typeof invalid}`);
  assert.throws(TypeError, () => new Temporal.PlainDate(2020, 11, invalid), `day ${typeof invalid}`);
}

for (const invalid of [undefined, "invalid"]) {
  assert.throws(RangeError, () => new Temporal.PlainDate(invalid, 11, 24), `year ${typeof invalid}`);
  assert.throws(RangeError, () => new Temporal.PlainDate(2020, invalid, 24), `month ${typeof invalid}`);
  assert.throws(RangeError, () => new Temporal.PlainDate(2020, 11, invalid), `day ${typeof invalid}`);
} 
const actual = [];
const args = [
  TemporalHelpers.toPrimitiveObserver(actual, 2020, "year"),
  TemporalHelpers.toPrimitiveObserver(actual, 11, "month"),
  TemporalHelpers.toPrimitiveObserver(actual, 24, "day"),
];
TemporalHelpers.assertPlainDate(new Temporal.PlainDate(...args),
  2020, 11, "M11", 24, "invalid string");
assert.compareArray(actual, [
  "get year.valueOf",
  "call year.valueOf",
  "get month.valueOf",
  "call month.valueOf",
  "get day.valueOf",
  "call day.valueOf",
]);
