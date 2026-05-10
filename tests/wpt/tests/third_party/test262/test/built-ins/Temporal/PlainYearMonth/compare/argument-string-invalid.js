// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.compare
description: An invalid ISO string is never supported
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const arg2 = new Temporal.PlainYearMonth(1976, 11);
for (const arg of TemporalHelpers.ISO.plainYearMonthStringsInvalid()) {
  assert.throws(RangeError, () => Temporal.PlainYearMonth.compare(arg, arg2), `"${arg}" is invalid (first argument)`);
  assert.throws(RangeError, () => Temporal.PlainYearMonth.compare(arg2, arg), `"${arg}" is invalid (second argument)`);
}
