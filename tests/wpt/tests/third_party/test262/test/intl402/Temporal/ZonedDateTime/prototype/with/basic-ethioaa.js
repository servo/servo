// Copyright 2025 Google Inc, Igalia S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.with
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
    day: 22, hour: 12, minute: 34, timeZone: "UTC",
    calendar
  },
  year1: {
    era: "aa",
    eraYear: 5493,
    year: 5493,
    month: 5,
    monthCode: "M05",
    day: 8, hour: 12, minute: 34, timeZone: "UTC",
    calendar
  }
};
for (var [name, result] of Object.entries(cases)) {
  const inCal = Temporal.ZonedDateTime.from(result);

  var afterWithDay = inCal.with({ day: 1 });
  TemporalHelpers.assertPlainDateTime(afterWithDay.toPlainDateTime(),
    inCal.year, inCal.month, inCal.monthCode, 1,  12, 34, 0, 0, 0, 0, `${name} after setting day`, inCal.era, inCal.eraYear);

  var afterWithMonth = afterWithDay.with({ month: 1 });
  TemporalHelpers.assertPlainDateTime(afterWithMonth.toPlainDateTime(),
    inCal.year, 1, "M01", 1,  12, 34, 0, 0, 0, 0, `${name} after setting month`, inCal.era, inCal.eraYear);

  var afterWithYear = afterWithMonth.with({ year: 7593 });
  TemporalHelpers.assertPlainDateTime(afterWithYear.toPlainDateTime(),
    7593, 1, "M01", 1,  12, 34, 0, 0, 0, 0, `${name} after setting year`, inCal.era, 7593);
}
