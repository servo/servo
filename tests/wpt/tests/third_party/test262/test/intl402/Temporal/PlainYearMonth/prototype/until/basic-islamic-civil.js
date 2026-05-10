// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.until
description: >
  Check various basic calculations not involving leap years or constraining
  (Islamic civil calendar)
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "islamic-civil";

// Years

const date137702 = Temporal.PlainYearMonth.from({ year: 1377, monthCode: "M02", calendar });
const date137802 = Temporal.PlainYearMonth.from({ year: 1378, monthCode: "M02", calendar });
const date137803 = Temporal.PlainYearMonth.from({ year: 1378, monthCode: "M03", calendar });
const date141507 = Temporal.PlainYearMonth.from({ year: 1415, monthCode: "M07", calendar });
const date141512 = Temporal.PlainYearMonth.from({ year: 1415, monthCode: "M12", calendar });
const date141712 = Temporal.PlainYearMonth.from({ year: 1417, monthCode: "M12", calendar });
const date142106 = Temporal.PlainYearMonth.from({ year: 1421, monthCode: "M06", calendar });
const date143703 = Temporal.PlainYearMonth.from({ year: 1437, monthCode: "M03", calendar });
const date143712 = Temporal.PlainYearMonth.from({ year: 1437, monthCode: "M12", calendar });
const date143803 = Temporal.PlainYearMonth.from({ year: 1438, monthCode: "M03", calendar });
const date143812 = Temporal.PlainYearMonth.from({ year: 1438, monthCode: "M12", calendar });
const date143901 = Temporal.PlainYearMonth.from({ year: 1439, monthCode: "M01", calendar });
const date143903 = Temporal.PlainYearMonth.from({ year: 1439, monthCode: "M03", calendar });
const date143905 = Temporal.PlainYearMonth.from({ year: 1439, monthCode: "M05", calendar });
const date143907 = Temporal.PlainYearMonth.from({ year: 1439, monthCode: "M07", calendar });
const date143908 = Temporal.PlainYearMonth.from({ year: 1439, monthCode: "M08", calendar });
const date144007 = Temporal.PlainYearMonth.from({ year: 1440, monthCode: "M07", calendar });
const date144009 = Temporal.PlainYearMonth.from({ year: 1440, monthCode: "M09", calendar });
const date144907 = Temporal.PlainYearMonth.from({ year: 1449, monthCode: "M07", calendar });
const date144912 = Temporal.PlainYearMonth.from({ year: 1449, monthCode: "M12", calendar });

const tests = [
  [
    date143907, date143907, "same day",
    ["years", 0, 0],
    ["months", 0, 0],
  ],
  [
    date143907, date143908, "1 month in same year",
    ["years", 0, 1],
    ["months", 0, 1],
  ],
  [
    date143812, date143901, "1 month in different year",
    ["years", 0, 1],
    ["months", 0, 1],
  ],
  [
    date143903, date143905, "2 months which both have 30 days",
    ["years", 0, 2],
    ["months", 0, 2],
  ],
  [
    date143907, date144007, "1 year",
    ["years", 1, 0],
    ["months", 0, 12],
  ],
  [
    date143907, date144907, "10 years",
    ["years", 10, 0],
    ["months", 0, 120],
  ],
  [
    date143907, date144009, "1 year 2 months",
    ["years", 1, 2],
  ],
  [
    date143907, date144912, "10 years and 5 months",
    ["years", 10, 5],
  ],
  [
    date141512, date143907, "23 years and 7 months",
    ["years", 23, 7],
  ],
  [
    date141507, date143907, "24 years",
    ["years", 24, 0],
  ],
  [
    date137802, date143803, "60 years, 1 month",
    ["years", 60, 1],
  ],
  [
    date143903, date143907, "4 months",
    ["years", 0, 4],
  ],
  [
    date143803, date143907, "1 year, 4 months",
    ["years", 1, 4],
  ],
  [
    date141712, date142106, "3 years, 6 months",
    ["years", 3, 6],
  ],
  [
    date137803, date143907, "61 years, 4 months",
    ["years", 61, 4],
  ],
  [
    date143712, date143907, "1 year, 7 months",
    ["years", 1, 7],
  ],
  [
    date143812, date143907, "7 months",
    ["years", 0, 7],
  ],
  [
    date141512, date143907, "23 years, 7 months",
    ["years", 23, 7],
  ],
  [
    date143712, date143903, "1 year, 3 months",
    ["years", 1, 3],
  ],
  [
    date143908, date143907, "negative 1 month in same year",
    ["years", 0, -1],
    ["months", 0, -1],
  ],
  [
    date143901, date143812, "negative 1 month in different year",
    ["years", 0, -1],
    ["months", 0, -1],
  ],
  [
    date143907, date143905, "negative 2 months which both have 30 days",
    ["years", 0, -2],
    ["months", 0, -2],
  ],
  [
    date144007, date143907, "negative 1 year",
    ["years", -1, 0],
    ["months", 0, -12],
  ],
  [
    date144907, date143907, "negative 10 years",
    ["years", -10, 0],
    ["months", 0, -120],
  ],
  [
    date144009, date143907, "negative 1 year 2 months",
    ["years", -1, -2],
  ],
  [
    date144912, date143907, "negative 10 years and 5 months",
    ["years", -10, -5],
  ],
  [
    date143907, date141512, "negative 23 years and 7 months",
    ["years", -23, -7],
  ],
  [
    date143907, date141507, "negative 24 years",
    ["years", -24, 0],
  ],
  [
    date143703, date137702, "negative 60 years, 1 month",
    ["years", -60, -1],
  ],
  [
    date143907, date143903, "negative 4 months",
    ["years", 0, -4],
  ],
  [
    date143907, date143803, "negative 1 year, 4 months",
    ["years", -1, -4],
  ],
  [
    date143907, date137803, "negative 61 years, 4 months",
    ["years", -61, -4],
  ],
  [
    date143907, date143712, "negative 1 year, 7 months",
    ["years", -1, -7],
  ],
  [
    date143907, date143812, "negative 7 months",
    ["years", 0, -7],
  ],
  [
    date143907, date141512, "negative 23 years, 7 months",
    ["years", -23, -7],
  ],
  [
    date143903, date143712, "negative 1 year, 3 months",
    ["years", -1, -3],
  ]
];

for (const [one, two, descr, ...units] of tests) {
  for (const [largestUnit, years, months] of units) {
    TemporalHelpers.assertDuration(
      one.until(two, { largestUnit }),
      years, months, 0, 0, 0, 0, 0, 0, 0, 0,
      descr
    );
  }
}
