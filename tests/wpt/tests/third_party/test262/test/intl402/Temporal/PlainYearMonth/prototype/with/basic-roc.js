// Copyright 2025 Google Inc, Igalia S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.with
description: with should work for roc calendar dates
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

const calendar = "roc";

const cases = {
  year2000: {
    era: "roc",
    year: 89,
    eraYear: 89,
    month: 1,
    monthCode: "M01",
    calendar
  },
};
for (var [name, result] of Object.entries(cases)) {
  const inCal = Temporal.PlainYearMonth.from(result);

  var afterWithMonth = inCal.with({ month: 1 });
  TemporalHelpers.assertPlainYearMonth(afterWithMonth,
    inCal.year, 1, "M01", `${name} after setting month`, inCal.era, inCal.eraYear);

  var afterWithYear = afterWithMonth.with({ year: 130 });
  TemporalHelpers.assertPlainYearMonth(afterWithYear,
    130, 1, "M01", `${name} after setting year`, inCal.era, 130);
}
