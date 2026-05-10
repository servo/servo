// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.since
description: >
  Check various basic calculations not involving leap years or constraining
  (Islamic tbla calendar)
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "islamic-tbla";

// Years

const date13760216 = Temporal.ZonedDateTime.from({ year: 1376, monthCode: "M02", day: 16, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date13770216 = Temporal.ZonedDateTime.from({ year: 1377, monthCode: "M02", day: 16, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date13780216 = Temporal.ZonedDateTime.from({ year: 1378, monthCode: "M02", day: 16, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date13780330 = Temporal.ZonedDateTime.from({ year: 1378, monthCode: "M03", day: 30, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date13870724 = Temporal.ZonedDateTime.from({ year: 1387, monthCode: "M07", day: 24, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date14131202 = Temporal.ZonedDateTime.from({ year: 1413, monthCode: "M12", day: 2, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date14150504 = Temporal.ZonedDateTime.from({ year: 1415, monthCode: "M05", day: 4, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date14150614 = Temporal.ZonedDateTime.from({ year: 1415, monthCode: "M06", day: 14, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date14150615 = Temporal.ZonedDateTime.from({ year: 1415, monthCode: "M06", day: 15, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date14150616 = Temporal.ZonedDateTime.from({ year: 1415, monthCode: "M06", day: 16, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date14150617 = Temporal.ZonedDateTime.from({ year: 1415, monthCode: "M06", day: 17, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date14150618 = Temporal.ZonedDateTime.from({ year: 1415, monthCode: "M06", day: 18, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date14150716 = Temporal.ZonedDateTime.from({ year: 1415, monthCode: "M07", day: 16, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date14151202 = Temporal.ZonedDateTime.from({ year: 1415, monthCode: "M12", day: 2, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date14151216 = Temporal.ZonedDateTime.from({ year: 1415, monthCode: "M12", day: 16, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date14151230 = Temporal.ZonedDateTime.from({ year: 1415, monthCode: "M12", day: 30, hour: 12, minute: 34, timeZone: "UTC", calendar })
const date14160104 = Temporal.ZonedDateTime.from({ year: 1416, monthCode: "M01", day: 4, hour: 12, minute: 34, timeZone: "UTC", calendar });;
const date14160504 = Temporal.ZonedDateTime.from({ year: 1416, monthCode: "M05", day: 4, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date14171201 = Temporal.ZonedDateTime.from({ year: 1417, monthCode: "M12", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date14200101 = Temporal.ZonedDateTime.from({ year: 1420, monthCode: "M01", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date14201015 = Temporal.ZonedDateTime.from({ year: 1420, monthCode: "M10", day: 15, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date14201201 = Temporal.ZonedDateTime.from({ year: 1420, monthCode: "M12", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date14210601 = Temporal.ZonedDateTime.from({ year: 1421, monthCode: "M06", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date14210618 = Temporal.ZonedDateTime.from({ year: 1421, monthCode: "M06", day: 18, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date14370101 = Temporal.ZonedDateTime.from({ year: 1437, monthCode: "M01", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date14370201 = Temporal.ZonedDateTime.from({ year: 1437, monthCode: "M02", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date14370316 = Temporal.ZonedDateTime.from({ year: 1437, monthCode: "M03", day: 16, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date14370724 = Temporal.ZonedDateTime.from({ year: 1437, monthCode: "M07", day: 24, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date14371230 = Temporal.ZonedDateTime.from({ year: 1437, monthCode: "M12", day: 30, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date14380201 = Temporal.ZonedDateTime.from({ year: 1438, monthCode: "M02", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date14380316 = Temporal.ZonedDateTime.from({ year: 1438, monthCode: "M03", day: 16, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date14380330 = Temporal.ZonedDateTime.from({ year: 1438, monthCode: "M03", day: 30, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date14381216 = Temporal.ZonedDateTime.from({ year: 1438, monthCode: "M12", day: 16, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date14381230 = Temporal.ZonedDateTime.from({ year: 1438, monthCode: "M12", day: 30, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date14390105 = Temporal.ZonedDateTime.from({ year: 1439, monthCode: "M01", day: 5, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date14390107 = Temporal.ZonedDateTime.from({ year: 1439, monthCode: "M01", day: 7, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date14390116 = Temporal.ZonedDateTime.from({ year: 1439, monthCode: "M01", day: 16, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date14390201 = Temporal.ZonedDateTime.from({ year: 1439, monthCode: "M02", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date14390205 = Temporal.ZonedDateTime.from({ year: 1439, monthCode: "M02", day: 5, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date14390216 = Temporal.ZonedDateTime.from({ year: 1439, monthCode: "M02", day: 16, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date14390228 = Temporal.ZonedDateTime.from({ year: 1439, monthCode: "M02", day: 28, hour: 12, minute: 34, timeZone: "UTC", calendar })
const date14390304 = Temporal.ZonedDateTime.from({ year: 1439, monthCode: "M03", day: 4, hour: 12, minute: 34, timeZone: "UTC", calendar });;
const date14390305 = Temporal.ZonedDateTime.from({ year: 1439, monthCode: "M03", day: 5, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date14390307 = Temporal.ZonedDateTime.from({ year: 1439, monthCode: "M03", day: 7, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date14390316 = Temporal.ZonedDateTime.from({ year: 1439, monthCode: "M03", day: 16, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date14390330 = Temporal.ZonedDateTime.from({ year: 1439, monthCode: "M03", day: 30, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date14390416 = Temporal.ZonedDateTime.from({ year: 1439, monthCode: "M04", day: 16, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date14390513 = Temporal.ZonedDateTime.from({ year: 1439, monthCode: "M05", day: 13, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date14390515 = Temporal.ZonedDateTime.from({ year: 1439, monthCode: "M05", day: 15, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date14390516 = Temporal.ZonedDateTime.from({ year: 1439, monthCode: "M05", day: 16, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date14390517 = Temporal.ZonedDateTime.from({ year: 1439, monthCode: "M05", day: 17, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date14390613 = Temporal.ZonedDateTime.from({ year: 1439, monthCode: "M06", day: 13, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date14390615 = Temporal.ZonedDateTime.from({ year: 1439, monthCode: "M06", day: 15, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date14390616 = Temporal.ZonedDateTime.from({ year: 1439, monthCode: "M06", day: 16, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date14390617 = Temporal.ZonedDateTime.from({ year: 1439, monthCode: "M06", day: 17, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date14390713 = Temporal.ZonedDateTime.from({ year: 1439, monthCode: "M07", day: 13, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date14390714 = Temporal.ZonedDateTime.from({ year: 1439, monthCode: "M07", day: 14, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date14390715 = Temporal.ZonedDateTime.from({ year: 1439, monthCode: "M07", day: 15, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date14390716 = Temporal.ZonedDateTime.from({ year: 1439, monthCode: "M07", day: 16, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date14390717 = Temporal.ZonedDateTime.from({ year: 1439, monthCode: "M07", day: 17, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date14390723 = Temporal.ZonedDateTime.from({ year: 1439, monthCode: "M07", day: 23, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date14390813 = Temporal.ZonedDateTime.from({ year: 1439, monthCode: "M08", day: 13, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date14390814 = Temporal.ZonedDateTime.from({ year: 1439, monthCode: "M08", day: 14, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date14390816 = Temporal.ZonedDateTime.from({ year: 1439, monthCode: "M08", day: 16, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date14390817 = Temporal.ZonedDateTime.from({ year: 1439, monthCode: "M08", day: 17, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date14390916 = Temporal.ZonedDateTime.from({ year: 1439, monthCode: "M09", day: 16, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date14400228 = Temporal.ZonedDateTime.from({ year: 1440, monthCode: "M02", day: 28, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date14400716 = Temporal.ZonedDateTime.from({ year: 1440, monthCode: "M07", day: 16, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date14400719 = Temporal.ZonedDateTime.from({ year: 1440, monthCode: "M07", day: 19, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date14400919 = Temporal.ZonedDateTime.from({ year: 1440, monthCode: "M09", day: 19, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date14490716 = Temporal.ZonedDateTime.from({ year: 1449, monthCode: "M07", day: 16, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date14491216 = Temporal.ZonedDateTime.from({ year: 1449, monthCode: "M12", day: 16, hour: 12, minute: 34, timeZone: "UTC", calendar });

const tests = [
  [
    date14390716, date14390716, "same day",
    ["years", 0, 0, 0, 0],
    ["months", 0, 0, 0, 0],
    ["weeks", 0, 0, 0, 0],
    ["days", 0, 0, 0, 0],
  ],
  [
    date14390716, date14390717, "one day",
    ["years", 0, 0, 0, -1],
    ["months", 0, 0, 0, -1],
    ["weeks", 0, 0, 0, -1],
    ["days", 0, 0, 0, -1],
  ],
  [
    date14390716, date14390723, "7 days",
    ["years", 0, 0, 0, -7],
    ["months", 0, 0, 0, -7],
    ["weeks", 0, 0, -1, 0],
  ],
  [
    date14390716, date14390816, "1 month in same year",
    ["years", 0, -1, 0, 0],
    ["months", 0, -1, 0, 0],
    ["weeks", 0, 0, -4, -2],
  ],
  [
    date14381216, date14390116, "1 month in different year",
    ["years", 0, -1, 0, 0],
    ["months", 0, -1, 0, 0],
  ],
  [
    date14390105, date14390205, "1 month in same year",
    ["years", 0, -1, 0, 0],
    ["months", 0, -1, 0, 0],
  ],
  [
    date14390516, date14390617, "1 month and 1 day in a month with 30 days",
    ["years", 0, -1, 0, -1],
    ["months", 0, -1, 0, -1],
    ["days", 0, 0, 0, -31],
  ],
  [
    date14390716, date14390814, "28 days across a month which has 30 days",
    ["years", 0, 0, 0, -28],
    ["months", 0, 0, 0, -28],
    ["weeks", 0, 0, -4, 0],
  ],
  [
    date14390316, date14390516, "2 months which both have 30 days",
    ["years", 0, -2, 0, 0],
    ["months", 0, -2, 0, 0],
    ["weeks", 0, 0, -8, -3],
    ["days", 0, 0, 0, -59],
  ],
  [
    date14390716, date14400716, "1 year",
    ["years", -1, 0, 0, 0],
    ["months", 0, -12, 0, 0],
    ["weeks", 0, 0, -50, -5],
    ["days", 0, 0, 0, -355],
  ],
  [
    date14380201, date14390201, "start of Safar",
    ["years", -1, 0, 0, 0],
    ["months", 0, -12, 0, 0],
  ],
  [
    date14390228, date14400228, "end of Safar",
    ["years", -1, 0, 0, 0],
    ["months", 0, -12, 0, 0],
  ],
  [
    date14370101, date14370201, "length of Muharram 1437",
    ["days", 0, 0, 0, -30],
  ],
  [
    date14390716, date14490716, "10 years",
    ["years", -10, 0, 0, 0],
    ["months", 0, -120, 0, 0],
    ["weeks", 0, 0, -506, -2],
    ["days", 0, 0, 0, -3544],
  ],
  [
    date14390716, date14400719, "1 year and 3 days",
    ["years", -1, 0, 0, -3],
  ],
  [
    date14390716, date14400919, "1 year 2 months and 3 days",
    ["years", -1, -2, 0, -3],
  ],
  [
    date14390716, date14491216, "10 years and 5 months",
    ["years", -10, -5, 0, 0],
  ],
  [
    date14151216, date14390716, "23 years and 7 months",
    ["years", -23, -7, 0, 0],
  ],
  [
    date14150716, date14390716, "24 years",
    ["years", -24, 0, 0, 0],
  ],
  [
    date14150716, date14390715, "23 years, 11 months and 28 days",
    ["years", -23, -11, 0, -28],
  ],
  [
    date14150616, date14390615, "23 years, 11 months and 29 days",
    ["years", -23, -11, 0, -29],
  ],
  [
    date13780216, date14380316, "60 years, 1 month",
    ["years", -60, -1, 0, 0],
  ],
  [
    date14390330, date14390716, "3 months and 16 days",
    ["years", 0, -3, 0, -16],
  ],
  [
    date14380330, date14390716, "1 year, 3 months and 16 days",
    ["years", -1, -3, 0, -16],
  ],
  [
    date14171201, date14210618, "3 years, 6 months and 17 days",
    ["years", -3, -6, 0, -17],
  ],
  [
    date13780330, date14390716, "61 years, 3 months and 16 days",
    ["years", -61, -3, 0, -16],
  ],
  [
    date14371230, date14390716, "1 year, 6 months and 16 days",
    ["years", -1, -6, 0, -16],
  ],
  [
    date14381230, date14390716, "6 months and 16 days",
    ["years", 0, -6, 0, -16],
  ],
  [
    date14201201, date14210601, "6 months",
    ["months", 0, -6, 0, 0],
  ],
  [
    date14200101, date14201015, "40 weeks",
    ["weeks", 0, 0, -40, 0],
    ["days", 0, 0, 0, -280],
  ],
  [
    date14151230, date14390716, "23 years, 6 months and 16 days",
    ["years", -23, -6, 0, -16],
  ],
  [
    date14371230, date14390305, "1 year, 2 months and 5 days",
    ["years", -1, -2, 0, -5],
  ],
  [
    date14390717, date14390716, "negative one day",
    ["years", 0, 0, 0, 1],
    ["months", 0, 0, 0, 1],
    ["weeks", 0, 0, 0, 1],
    ["days", 0, 0, 0, 1],
  ],
  [
    date14390723, date14390716, "negative 7 days",
    ["years", 0, 0, 0, 7],
    ["months", 0, 0, 0, 7],
    ["weeks", 0, 0, 1, 0],
  ],
  [
    date14390816, date14390716, "negative 1 month in same year (1)",
    ["years", 0, 1, 0, 0],
    ["months", 0, 1, 0, 0],
    ["weeks", 0, 0, 4, 2],
  ],
  [
    date14390116, date14381216, "negative 1 month in different year",
    ["years", 0, 1, 0, 0],
    ["months", 0, 1, 0, 0],
  ],
  [
    date14390205, date14390105, "negative 1 month in same year (2)",
    ["years", 0, 1, 0, 0],
    ["months", 0, 1, 0, 0],
  ],
  [
    date14390617, date14390516, "negative 1 month and 1 day in a month with 30 days",
    ["years", 0, 1, 0, 1],
    ["months", 0, 1, 0, 1],
    ["days", 0, 0, 0, 31],
  ],
  [
    date14390515, date14390416, "negative 28 days across a month which has 30 days",
    ["years", 0, 0, 0, 28],
    ["months", 0, 0, 0, 28],
    ["weeks", 0, 0, 4, 0],
  ],
  [
    date14390716, date14390516, "negative 2 months which both have 30 days",
    ["years", 0, 2, 0, 0],
    ["months", 0, 2, 0, 0],
    ["weeks", 0, 0, 8, 3],
    ["days", 0, 0, 0, 59],
  ],
  [
    date14400716, date14390716, "negative 1 year",
    ["years", 1, 0, 0, 0],
    ["months", 0, 12, 0, 0],
    ["weeks", 0, 0, 50, 5],
    ["days", 0, 0, 0, 355],
  ],
  [
    date14490716, date14390716, "negative 10 years",
    ["years", 10, 0, 0, 0],
    ["months", 0, 120, 0, 0],
    ["weeks", 0, 0, 506, 2],
    ["days", 0, 0, 0, 3544],
  ],
  [
    date14400719, date14390716, "negative 1 year and 3 days",
    ["years", 1, 0, 0, 3],
  ],
  [
    date14400919, date14390716, "negative 1 year 2 months and 3 days",
    ["years", 1, 2, 0, 3],
  ],
  [
    date14491216, date14390716, "negative 10 years and 5 months",
    ["years", 10, 5, 0, 0],
  ],
  [
    date14390716, date14151216, "negative 23 years and 7 months",
    ["years", 23, 7, 0, 0],
  ],
  [
    date14390716, date14150716, "negative 24 years",
    ["years", 24, 0, 0, 0],
  ],
  [
    date14390615, date14150616, "negative 23 years, 11 months and 28 days",
    ["years", 23, 11, 0, 28],
  ],
  [
    date14370316, date13770216, "negative 60 years, 1 month",
    ["years", 60, 1, 0, 0],
  ],
  [
    date14390716, date14390330, "negative 3 months and 16 days",
    ["years", 0, 3, 0, 16],
  ],
  [
    date14390716, date14380330, "negative 1 year, 3 months and 16 days",
    ["years", 1, 3, 0, 16],
  ],
  [
    date14390716, date13780330, "negative 61 years, 3 months and 16 days",
    ["years", 61, 3, 0, 16],
  ],
  [
    date14390716, date14371230, "negative 1 year, 6 months and 16 days",
    ["years", 1, 6, 0, 16],
  ],
  [
    date14390716, date14381230, "negative 6 months and 16 days",
    ["years", 0, 6, 0, 16],
  ],
  [
    date14390716, date14151230, "negative 23 years, 6 months and 16 days",
    ["years", 23, 6, 0, 16],
  ],
  [
    date14390305, date14371230, "negative 1 year, 2 months and 5 days",
    ["years", 1, 2, 0, 5],
  ]
];

for (const [one, two, descr, ...units] of tests) {
  for (const [largestUnit, years, months, weeks, days] of units) {
    TemporalHelpers.assertDuration(
      one.since(two, { largestUnit }),
      years, months, weeks, days, 0, 0, 0, 0, 0, 0,
      descr
    );
  }
}
