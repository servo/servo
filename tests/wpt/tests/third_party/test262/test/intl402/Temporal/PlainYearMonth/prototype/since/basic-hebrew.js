// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.since
description: >
  Check various basic calculations not involving leap years or constraining
  (hebrew calendar)
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "hebrew";

// Years

const date572202 = Temporal.PlainYearMonth.from({ year: 5722, monthCode: "M02", calendar });
const date572203 = Temporal.PlainYearMonth.from({ year: 5722, monthCode: "M03", calendar });
const date572302 = Temporal.PlainYearMonth.from({ year: 5723, monthCode: "M02", calendar });
const date572303 = Temporal.PlainYearMonth.from({ year: 5723, monthCode: "M03", calendar });
const date575901 = Temporal.PlainYearMonth.from({ year: 5759, monthCode: "M01", calendar });
const date575910 = Temporal.PlainYearMonth.from({ year: 5759, monthCode: "M10", calendar });
const date575912 = Temporal.PlainYearMonth.from({ year: 5759, monthCode: "M12", calendar });
const date576001 = Temporal.PlainYearMonth.from({ year: 5760, monthCode: "M01", calendar });
const date576007 = Temporal.PlainYearMonth.from({ year: 5760, monthCode: "M07", calendar });
const date576012 = Temporal.PlainYearMonth.from({ year: 5760, monthCode: "M12", calendar });
const date576206 = Temporal.PlainYearMonth.from({ year: 5762, monthCode: "M06", calendar });
const date578112 = Temporal.PlainYearMonth.from({ year: 5781, monthCode: "M12", calendar });
const date578203 = Temporal.PlainYearMonth.from({ year: 5782, monthCode: "M03", calendar });
const date578207 = Temporal.PlainYearMonth.from({ year: 5782, monthCode: "M07", calendar });
const date578212 = Temporal.PlainYearMonth.from({ year: 5782, monthCode: "M12", calendar });
const date578301 = Temporal.PlainYearMonth.from({ year: 5783, monthCode: "M01", calendar });
const date578303 = Temporal.PlainYearMonth.from({ year: 5783, monthCode: "M03", calendar });
const date578307 = Temporal.PlainYearMonth.from({ year: 5783, monthCode: "M07", calendar });
const date578308 = Temporal.PlainYearMonth.from({ year: 5783, monthCode: "M08", calendar });
const date578309 = Temporal.PlainYearMonth.from({ year: 5783, monthCode: "M09", calendar });
const date578407 = Temporal.PlainYearMonth.from({ year: 5784, monthCode: "M07", calendar });
const date578409 = Temporal.PlainYearMonth.from({ year: 5784, monthCode: "M09", calendar });
const date579307 = Temporal.PlainYearMonth.from({ year: 5793, monthCode: "M07", calendar });
const date579312 = Temporal.PlainYearMonth.from({ year: 5793, monthCode: "M12", calendar });

const tests = [
  [
    date578307, date578307, "same day",
    ["years", 0, 0],
    ["months", 0, 0],
  ],
  [
    date578307, date578308, "1 month in same year",
    ["years", 0, -1],
    ["months", 0, -1],
  ],
  [
    date578212, date578301, "1 month in different year",
    ["years", 0, -1],
    ["months", 0, -1],
  ],
  [
    date578307, date578309, "2 months which both have 30 days",
    ["years", 0, -2],
    ["months", 0, -2],
  ],
  [
    date578207, date578307, "1 year",
    ["years", -1, 0],
    ["months", 0, -12],
  ],
  [
    date578307, date579307, "10 years",
    ["years", -10, 0],
    ["months", 0, -124],
  ],
  [
    date578307, date578409, "1 year 2 months",
    ["years", -1, -2],
  ],
  [
    date575912, date576206, "2 years 6 months",
    ["years", -2, -6],
  ],
  [
    date578307, date579312, "10 years and 5 months",
    ["years", -10, -5],
  ],
  [
    date576012, date578407, "23 years and 8 months",
    ["years", -23, -8],
  ],
  [
    date576007, date578407, "24 years",
    ["years", -24, 0],
  ],
  [
    date572302, date578303, "60 years, 1 month",
    ["years", -60, -1],
  ],
  [
    date578303, date578307, "4 months",
    ["years", 0, -4],
  ],
  [
    date578203, date578307, "1 year, 4 months",
    ["years", -1, -4],
  ],
  [
    date572303, date578407, "61 years, 5 months",
    ["years", -61, -5],
  ],
  [
    date578112, date578307, "1 year, 7 months",
    ["years", -1, -7],
  ],
  [
    date578212, date578307, "7 months",
    ["years", 0, -7],
  ],
  [
    date575901, date575910, "40 weeks",
  ],
  [
    date576001, date578307, "23 years, 6 months",
    ["years", -23, -6],
  ],
  [
    date578112, date578303, "1 year, 3 months",
    ["years", -1, -3],
  ],
  [
    date578308, date578307, "negative 1 month in same year",
    ["years", 0, 1],
    ["months", 0, 1],
  ],
  [
    date578301, date578212, "negative 1 month in different year",
    ["years", 0, 1],
    ["months", 0, 1],
  ],
  [
    date578309, date578307, "negative 2 months which both have 30 days",
    ["years", 0, 2],
    ["months", 0, 2],
  ],
  [
    date578307, date578207, "negative 1 year",
    ["years", 1, 0],
    ["months", 0, 12],
  ],
  [
    date579307, date578307, "negative 10 years",
    ["years", 10, 0],
    ["months", 0, 124],
  ],
  [
    date578409, date578307, "negative 1 year 2 months",
    ["years", 1, 2],
  ],
  [
    date579312, date578307, "negative 10 years and 5 months",
    ["years", 10, 5],
  ],
  [
    date578407, date576012, "negative 23 years and 7 months",
    ["years", 23, 7],
  ],
  [
    date578407, date576007, "negative 24 years",
    ["years", 24, 0],
  ],
  [
    date578203, date572202, "negative 60 years, 1 month",
    ["years", 60, 1],
  ],
  [
    date578307, date578303, "negative 4 months",
    ["years", 0, 4],
  ],
  [
    date578307, date578203, "negative 1 year, 5 months",
    ["years", 1, 5],
  ],
  [
    date578307, date572203, "negative 61 years, 5 months",
    ["years", 61, 5],
  ],
  [
    date578307, date578112, "negative 1 year, 8 months",
    ["years", 1, 8],
  ],
  [
    date578307, date578212, "negative 7 months",
    ["years", 0, 7],
  ],
  [
    date578307, date575912, "negative 23 years, 8 months",
    ["years", 23, 8],
  ],
  [
    date578303, date578112, "negative 1 year, 3 months",
    ["years", 1, 3],
  ]
];

for (const [one, two, descr, ...units] of tests) {
  for (const [largestUnit, years, months] of units) {
    TemporalHelpers.assertDuration(
      one.since(two, { largestUnit }),
      years, months, 0, 0, 0, 0, 0, 0, 0, 0,
      descr
    );
  }
}
