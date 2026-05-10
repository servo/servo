// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.since
description: A string argument is parsed into a PlainYearMonth
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const instance = new Temporal.PlainYearMonth(1976, 11);
for (const arg of TemporalHelpers.ISO.plainYearMonthStringsValid()) {
  const result = instance.since(arg);
  TemporalHelpers.assertDuration(result, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, `"${arg}" is a valid PlainYearMonth string`);
}

const instanceNegativeYear = new Temporal.PlainYearMonth(-9999, 11);
for (const arg of TemporalHelpers.ISO.plainYearMonthStringsValidNegativeYear()) {
  const result = instanceNegativeYear.since(arg);
  TemporalHelpers.assertDuration(result, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, `"${arg}" is a valid PlainYearMonth string`);
}
