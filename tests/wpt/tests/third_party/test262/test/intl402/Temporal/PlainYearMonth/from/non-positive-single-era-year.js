// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.from
description: Non-positive era years in calendars with a single era
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const calendarEras = {
  buddhist: "be",
  coptic: "am",
  ethioaa: "aa",
  hebrew: "am",
  indian: "shaka",
  persian: "ap",
};
const options = { overflow: "reject" };

for (const [calendar, era] of Object.entries(calendarEras)) {
  for (const eraYear of [-1, 0, 1]) {
    const date = Temporal.PlainYearMonth.from({ era, eraYear, monthCode: "M01", calendar }, options);
    TemporalHelpers.assertPlainYearMonth(
      date,
      eraYear, 1, "M01", `era year ${eraYear} is not remapped`,
      era, eraYear, null);
  }
}
