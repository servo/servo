// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.subtract
description: Maximum allowed duration
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const instance = new Temporal.PlainYearMonth(1970, 1);

const maxCases = [
  ["P273790Y8M", "string with max years"],
  [{ years: 273790, months: 8 }, "property bag with max years"],
  ["P3285488M", "string with max months"],
  [{ months: 3285488 }, "property bag with max months"],
];

for (const [arg, descr] of maxCases) {
  const result = instance.subtract(arg);
  TemporalHelpers.assertPlainYearMonth(result, -271821, 5, "M05", `operation succeeds with ${descr}`);
}

const minCases = [
  ["-P273790Y8M", "string with min years"],
  [{ years: -273790, months: -8 }, "property bag with min years"],
  ["-P3285488M", "string with min months"],
  [{ months: -3285488 }, "property bag with min months"],
];

for (const [arg, descr] of minCases) {
  const result = instance.subtract(arg);
  TemporalHelpers.assertPlainYearMonth(result, 275760, 9, "M09", `operation succeeds with ${descr}`);
}
