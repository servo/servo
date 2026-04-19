// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday
description: PlainMonthDay constructor with non-integer arguments.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

TemporalHelpers.assertPlainMonthDay(new Temporal.PlainMonthDay(11.7, 24.1),
  "M11", 24, "positive fractional");

TemporalHelpers.assertPlainMonthDay(new Temporal.PlainMonthDay("11.7", "24.1"),
  "M11", 24, "fractional strings");

for (const invalid of [Symbol(), 1n]) {
  assert.throws(TypeError, () => new Temporal.PlainMonthDay(invalid, 24), `month ${typeof invalid}`);
  assert.throws(TypeError, () => new Temporal.PlainMonthDay(11, invalid), `day ${typeof invalid}`);
}

for (const invalid of [undefined, "invalid"]) {
  assert.throws(RangeError, () => new Temporal.PlainMonthDay(invalid, 24), `month ${typeof invalid}`);
  assert.throws(RangeError, () => new Temporal.PlainMonthDay(11, invalid), `day ${typeof invalid}`);
}
