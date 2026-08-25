// Copyright 2025 Google Inc, Igalia S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.with
description: with should work for Japanese calendar dates
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

const calendar = "japanese";

const cases = {
  year2000: {
    era: "heisei",
    year: 2000,
    eraYear: 12,
    month: 1,
    monthCode: "M01",
    day: 1, hour: 12, minute: 34,
    calendar
  }
};
for (var [name, result] of Object.entries(cases)) {
  const inCal = Temporal.PlainDateTime.from(result);

  var afterWithDay = inCal.with({ day: 1 });
  TemporalHelpers.assertPlainDateTime(afterWithDay,
    inCal.year, inCal.month, inCal.monthCode, 1,  12, 34, 0, 0, 0, 0, `${name} after setting day`, inCal.era, inCal.eraYear);

  var afterWithMonth = afterWithDay.with({ month: 1 });
  TemporalHelpers.assertPlainDateTime(afterWithMonth,
    inCal.year, 1, "M01", 1,  12, 34, 0, 0, 0, 0, `${name} after setting month`, inCal.era, inCal.eraYear);

  var afterWithYear = afterWithMonth.with({ year: 1970 });
  TemporalHelpers.assertPlainDateTime(afterWithYear,
    1970, 1, "M01", 1,  12, 34, 0, 0, 0, 0, `${name} after setting year`, "showa", 45);
}
