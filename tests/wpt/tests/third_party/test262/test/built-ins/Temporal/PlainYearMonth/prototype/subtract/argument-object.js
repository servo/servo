// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.subtract
description: Passing an object to subtract() works
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const ym = Temporal.PlainYearMonth.from("2019-11");

const tests = [
  [{ months: 2 }, 2019, 9, "M09"],
  [{ years: 1 }, 2018, 11, "M11"],
  [{ months: -2 }, 2020, 1, "M01"],
  [{ years: -1 }, 2020, 11, "M11"],
];

for (const [argument, ...expected] of tests) {
  TemporalHelpers.assertPlainYearMonth(ym.subtract(argument), ...expected, "no options");
  TemporalHelpers.assertPlainYearMonth(ym.subtract(argument, { overflow: "constrain" }), ...expected, "constrain");
  TemporalHelpers.assertPlainYearMonth(ym.subtract(argument, { overflow: "reject" }), ...expected, "reject");
}
