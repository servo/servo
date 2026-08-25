// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.since
description: >
  Check various basic calculations not involving leap years or constraining
  (coptic calendar)
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "coptic";

// Years

const date171002 = Temporal.PlainYearMonth.from({ year: 1710, monthCode: "M02", calendar });
const date171003 = Temporal.PlainYearMonth.from({ year: 1710, monthCode: "M03", calendar });
const date171103 = Temporal.PlainYearMonth.from({ year: 1711, monthCode: "M03", calendar });
const date171312 = Temporal.PlainYearMonth.from({ year: 1713, monthCode: "M12", calendar });
const date171612 = Temporal.PlainYearMonth.from({ year: 1716, monthCode: "M12", calendar });
const date171705 = Temporal.PlainYearMonth.from({ year: 1717, monthCode: "M05", calendar });
const date174707 = Temporal.PlainYearMonth.from({ year: 1747, monthCode: "M07", calendar });
const date174712 = Temporal.PlainYearMonth.from({ year: 1747, monthCode: "M12", calendar });
const date174713 = Temporal.PlainYearMonth.from({ year: 1747, monthCode: "M13", calendar });
const date176912 = Temporal.PlainYearMonth.from({ year: 1769, monthCode: "M12", calendar });
const date176913 = Temporal.PlainYearMonth.from({ year: 1769, monthCode: "M13", calendar });
const date177003 = Temporal.PlainYearMonth.from({ year: 1770, monthCode: "M03", calendar });
const date177012 = Temporal.PlainYearMonth.from({ year: 1770, monthCode: "M12", calendar });
const date177013 = Temporal.PlainYearMonth.from({ year: 1770, monthCode: "M13", calendar });
const date177101 = Temporal.PlainYearMonth.from({ year: 1771, monthCode: "M01", calendar });
const date177102 = Temporal.PlainYearMonth.from({ year: 1771, monthCode: "M02", calendar });
const date177103 = Temporal.PlainYearMonth.from({ year: 1771, monthCode: "M03", calendar });
const date177104 = Temporal.PlainYearMonth.from({ year: 1771, monthCode: "M04", calendar });
const date177106 = Temporal.PlainYearMonth.from({ year: 1771, monthCode: "M06", calendar });
const date177107 = Temporal.PlainYearMonth.from({ year: 1771, monthCode: "M07", calendar });
const date177108 = Temporal.PlainYearMonth.from({ year: 1771, monthCode: "M08", calendar });
const date177112 = Temporal.PlainYearMonth.from({ year: 1771, monthCode: "M12", calendar });
const date177207 = Temporal.PlainYearMonth.from({ year: 1772, monthCode: "M07", calendar });
const date177209 = Temporal.PlainYearMonth.from({ year: 1772, monthCode: "M09", calendar });
const date177307 = Temporal.PlainYearMonth.from({ year: 1773, monthCode: "M07", calendar });
const date178107 = Temporal.PlainYearMonth.from({ year: 1781, monthCode: "M07", calendar });
const date178112 = Temporal.PlainYearMonth.from({ year: 1781, monthCode: "M12", calendar });

const tests = [
  [
    date177107, date177107, "same day",
    ["years", 0, 0],
    ["months", 0, 0],
  ],
  [
    date177107, date177108, "1 month in same year (30-day month to 29-day month)",
    ["years", 0, -1],
    ["months", 0, -1],
  ],
  [
    date177013, date177101, "1 month in different year",
    ["years", 0, -1],
    ["months", 0, -1],
  ],
  [
    date177101, date177102, "1 month in same year (29-day month to 30-day month)",
    ["years", 0, -1],
    ["months", 0, -1],
  ],
  [
    date171612, date171705, "6 months in different year",
    ["months", 0, -6],
  ],
  [
    date177207, date177307, "1 year",
    ["years", -1, 0],
    ["months", 0, -13],
  ],
  [
    date177107, date178107, "10 years",
    ["years", -10, 0],
    ["months", 0, -130],
  ],
  [
    date177107, date177209, "1 year 2 months",
    ["years", -1, -2],
  ],
  [
    date171312, date171705, "3 years, 6 months",
    ["years", -3, -6],
  ],
  [
    date177107, date178112, "10 years and 5 months",
    ["years", -10, -5],
  ],
  [
    date174712, date177106, "23 years and 7 months",
    ["years", -23, -7],
  ],
  [
    date174707, date177107, "24 years",
    ["years", -24, 0],
  ],
  [
    date174707, date177106, "23 years, 12 months",
    ["years", -23, -12],
  ],
  [
    date171002, date177003, "60 years, 1 month",
    ["years", -60, -1],
  ],
  [
    date177103, date177107, "4 months",
    ["years", 0, -4],
  ],
  [
    date177003, date177107, "1 year, 4 months",
    ["years", -1, -4],
  ],
  [
    date171003, date177107, "61 years, 4 months",
    ["years", -61, -4],
  ],
  [
    date176912, date177107, "1 year, 8 months",
    ["years", -1, -8],
  ],
  [
    date177013, date177106, "6 months",
    ["years", 0, -6],
  ],
  [
    date174713, date177106, "23 years, 6 months",
    ["years", -23, -6],
  ],
  [
    date176913, date177102, "1 year, 2 months",
    ["years", -1, -2],
  ],
  [
    date177108, date177107, "negative 1 month in same year",
    ["years", 0, 1],
    ["months", 0, 1],
  ],
  [
    date177101, date177013, "negative 1 month in different year",
    ["years", 0, 1],
    ["months", 0, 1],
  ],
  [
    date177104, date177102, "negative 2 months which both have 30 days",
    ["years", 0, 2],
    ["months", 0, 2],
  ],
  [
    date177307, date177207, "negative 1 year",
    ["years", 1, 0],
    ["months", 0, 13],
  ],
  [
    date178107, date177107, "negative 10 years",
    ["years", 10, 0],
    ["months", 0, 130],
  ],
  [
    date177209, date177107, "negative 1 year 2 months",
    ["years", 1, 2],
  ],
  [
    date178112, date177107, "negative 10 years and 5 months",
    ["years", 10, 5],
  ],
  [
    date177107, date174712, "negative 23 years and 8 months",
    ["years", 23, 8],
  ],
  [
    date177107, date174707, "negative 24 years",
    ["years", 24, 0],
  ],
  [
    date177003, date171002, "negative 60 years, 1 month",
    ["years", 60, 1],
  ],
  [
    date177107, date177103, "negative 4 months",
    ["years", 0, 4],
  ],
  [
    date177207, date177103, "negative 1 year, 4 months",
    ["years", 1, 4],
  ],
  [
    date177207, date171103, "negative 61 years, 4 months",
    ["years", 61, 4],
  ],
  [
    date177307, date177112, "negative 1 year, 8 months",
    ["years", 1, 8],
  ],
  [
    date177107, date177012, "negative 8 months",
    ["years", 0, 8],
  ],
  [
    date177107, date174712, "negative 23 years, 8 months",
    ["years", 23, 8],
  ],
  [
    date177103, date176912, "negative 1 year, 4 months",
    ["years", 1, 4],
  ]
];

for (const [one, two, descr, ...units] of tests) {
  for (const [largestUnit, years, months] of units) {
    TemporalHelpers.assertDuration(
      one.since(two, { largestUnit }),
      years, months, 0, 0, 0, 0, 0, 0, 0, 0,
      `${descr} (largestUnit = ${largestUnit})`
    );
  }
}
