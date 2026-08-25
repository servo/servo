// Copyright 2025 Google Inc, Igalia S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.with
description: with should work for ethioaa calendar dates
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

const calendar = "ethioaa";

const cases = {
  year2000: {
    era: "aa",
    eraYear: 7492,
    year: 7492,
    month: 4,
    monthCode: "M04",
    calendar
  },
  year1: {
    era: "aa",
    eraYear: 5493,
    year: 5493,
    month: 5,
    monthCode: "M05",
    calendar
  }
};
for (var [name, result] of Object.entries(cases)) {
  const inCal = Temporal.PlainYearMonth.from(result);

  var afterWithMonth = inCal.with({ month: 1 });
  TemporalHelpers.assertPlainYearMonth(afterWithMonth,
    inCal.year, 1, "M01", `${name} after setting month`, inCal.era, inCal.eraYear, null);

  var afterWithYear = afterWithMonth.with({ year: 7593 });
  TemporalHelpers.assertPlainYearMonth(afterWithYear,
    7593, 1, "M01", `${name} after setting year`, inCal.era, 7593, null);
}
