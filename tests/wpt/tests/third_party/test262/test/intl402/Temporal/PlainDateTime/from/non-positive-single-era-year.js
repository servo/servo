// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.from
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
    const date = Temporal.PlainDateTime.from({ era, eraYear, monthCode: "M01", day: 1, hour: 12, minute: 34, calendar }, options);
    TemporalHelpers.assertPlainDateTime(
      date,
      eraYear, 1, "M01", 1, 12, 34, 0, 0, 0, 0, `era year ${eraYear} is not remapped`,
      era, eraYear);
  }
}
