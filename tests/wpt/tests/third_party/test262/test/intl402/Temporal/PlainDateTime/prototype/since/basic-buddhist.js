// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.since
description: >
  Check various basic calculations not involving leap years or constraining
  (buddhist calendar)
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "buddhist";

// Years

const date25030216 = Temporal.PlainDateTime.from({ year: 2503, monthCode: "M02", day: 16, hour: 12, minute: 34, calendar });
const date25030330 = Temporal.PlainDateTime.from({ year: 2503, monthCode: "M03", day: 30, hour: 12, minute: 34, calendar });
const date25120724 = Temporal.PlainDateTime.from({ year: 2512, monthCode: "M07", day: 24, hour: 12, minute: 34, calendar });
const date25400616 = Temporal.PlainDateTime.from({ year: 2540, monthCode: "M06", day: 16, hour: 12, minute: 34, calendar });
const date25400716 = Temporal.PlainDateTime.from({ year: 2540, monthCode: "M07", day: 16, hour: 12, minute: 34, calendar });
const date25401216 = Temporal.PlainDateTime.from({ year: 2540, monthCode: "M12", day: 16, hour: 12, minute: 34, calendar });
const date25401230 = Temporal.PlainDateTime.from({ year: 2540, monthCode: "M12", day: 30, hour: 12, minute: 34, calendar });
const date25550101 = Temporal.PlainDateTime.from({ year: 2555, monthCode: "M01", day: 1, hour: 12, minute: 34, calendar });
const date25551007 = Temporal.PlainDateTime.from({ year: 2555, monthCode: "M10", day: 7, hour: 12, minute: 34, calendar });
const date25551201 = Temporal.PlainDateTime.from({ year: 2555, monthCode: "M12", day: 1, hour: 12, minute: 34, calendar });
const date25560601 = Temporal.PlainDateTime.from({ year: 2556, monthCode: "M06", day: 1, hour: 12, minute: 34, calendar });
const date25590618 = Temporal.PlainDateTime.from({ year: 2559, monthCode: "M06", day: 18, hour: 12, minute: 34, calendar });
const date25620101 = Temporal.PlainDateTime.from({ year: 2562, monthCode: "M01", day: 1, hour: 12, minute: 34, calendar });
const date25620201 = Temporal.PlainDateTime.from({ year: 2562, monthCode: "M02", day: 1, hour: 12, minute: 34, calendar });
const date25620724 = Temporal.PlainDateTime.from({ year: 2562, monthCode: "M07", day: 24, hour: 12, minute: 34, calendar });
const date25621230 = Temporal.PlainDateTime.from({ year: 2562, monthCode: "M12", day: 30, hour: 12, minute: 34, calendar });
const date25630201 = Temporal.PlainDateTime.from({ year: 2563, monthCode: "M02", day: 1, hour: 12, minute: 34, calendar });
const date25630316 = Temporal.PlainDateTime.from({ year: 2563, monthCode: "M03", day: 16, hour: 12, minute: 34, calendar });
const date25630330 = Temporal.PlainDateTime.from({ year: 2563, monthCode: "M03", day: 30, hour: 12, minute: 34, calendar });
const date25631216 = Temporal.PlainDateTime.from({ year: 2563, monthCode: "M12", day: 16, hour: 12, minute: 34, calendar });
const date25631230 = Temporal.PlainDateTime.from({ year: 2563, monthCode: "M12", day: 30, hour: 12, minute: 34, calendar });
const date25640105 = Temporal.PlainDateTime.from({ year: 2564, monthCode: "M01", day: 5, hour: 12, minute: 34, calendar });
const date25640107 = Temporal.PlainDateTime.from({ year: 2564, monthCode: "M01", day: 7, hour: 12, minute: 34, calendar });
const date25640116 = Temporal.PlainDateTime.from({ year: 2564, monthCode: "M01", day: 16, hour: 12, minute: 34, calendar });
const date25640201 = Temporal.PlainDateTime.from({ year: 2564, monthCode: "M02", day: 1, hour: 12, minute: 34, calendar });
const date25640205 = Temporal.PlainDateTime.from({ year: 2564, monthCode: "M02", day: 5, hour: 12, minute: 34, calendar });
const date25640228 = Temporal.PlainDateTime.from({ year: 2564, monthCode: "M02", day: 28, hour: 12, minute: 34, calendar });
const date25640305 = Temporal.PlainDateTime.from({ year: 2564, monthCode: "M03", day: 5, hour: 12, minute: 34, calendar });
const date25640307 = Temporal.PlainDateTime.from({ year: 2564, monthCode: "M03", day: 7, hour: 12, minute: 34, calendar });
const date25640330 = Temporal.PlainDateTime.from({ year: 2564, monthCode: "M03", day: 30, hour: 12, minute: 34, calendar });
const date25640615 = Temporal.PlainDateTime.from({ year: 2564, monthCode: "M06", day: 15, hour: 12, minute: 34, calendar });
const date25640715 = Temporal.PlainDateTime.from({ year: 2564, monthCode: "M07", day: 15, hour: 12, minute: 34, calendar });
const date25640716 = Temporal.PlainDateTime.from({ year: 2564, monthCode: "M07", day: 16, hour: 12, minute: 34, calendar });
const date25640717 = Temporal.PlainDateTime.from({ year: 2564, monthCode: "M07", day: 17, hour: 12, minute: 34, calendar });
const date25640723 = Temporal.PlainDateTime.from({ year: 2564, monthCode: "M07", day: 23, hour: 12, minute: 34, calendar });
const date25640813 = Temporal.PlainDateTime.from({ year: 2564, monthCode: "M08", day: 13, hour: 12, minute: 34, calendar });
const date25640816 = Temporal.PlainDateTime.from({ year: 2564, monthCode: "M08", day: 16, hour: 12, minute: 34, calendar });
const date25640817 = Temporal.PlainDateTime.from({ year: 2564, monthCode: "M08", day: 17, hour: 12, minute: 34, calendar });
const date25640916 = Temporal.PlainDateTime.from({ year: 2564, monthCode: "M09", day: 16, hour: 12, minute: 34, calendar });
const date25650228 = Temporal.PlainDateTime.from({ year: 2565, monthCode: "M02", day: 28, hour: 12, minute: 34, calendar });
const date25650716 = Temporal.PlainDateTime.from({ year: 2565, monthCode: "M07", day: 16, hour: 12, minute: 34, calendar });
const date25650719 = Temporal.PlainDateTime.from({ year: 2565, monthCode: "M07", day: 19, hour: 12, minute: 34, calendar });
const date25650919 = Temporal.PlainDateTime.from({ year: 2565, monthCode: "M09", day: 19, hour: 12, minute: 34, calendar });
const date25740716 = Temporal.PlainDateTime.from({ year: 2574, monthCode: "M07", day: 16, hour: 12, minute: 34, calendar });
const date25741216 = Temporal.PlainDateTime.from({ year: 2574, monthCode: "M12", day: 16, hour: 12, minute: 34, calendar });

const tests = [
  [
    date25640716, date25640716, "same day",
    ["years", 0, 0, 0, 0],
    ["months", 0, 0, 0, 0],
    ["weeks", 0, 0, 0, 0],
    ["days", 0, 0, 0, 0],
  ],
  [
    date25640716, date25640717, "one day",
    ["years", 0, 0, 0, -1],
    ["months", 0, 0, 0, -1],
    ["weeks", 0, 0, 0, -1],
    ["days", 0, 0, 0, -1],
  ],
  [
    date25640716, date25640723, "7 days",
    ["years", 0, 0, 0, -7],
    ["months", 0, 0, 0, -7],
    ["weeks", 0, 0, -1, 0],
  ],
  [
    date25640716, date25640816, "1 month in same year",
    ["years", 0, -1, 0, 0],
    ["months", 0, -1, 0, 0],
    ["weeks", 0, 0, -4, -3],
  ],
  [
    date25631216, date25640116, "1 month in different year",
    ["years", 0, -1, 0, 0],
    ["months", 0, -1, 0, 0],
  ],
  [
    date25640105, date25640205, "1 month in same year",
    ["years", 0, -1, 0, 0],
    ["months", 0, -1, 0, 0],
  ],
  [
    date25640716, date25640817, "1 month and 1 day in a month with 31 days",
    ["years", 0, -1, 0, -1],
    ["months", 0, -1, 0, -1],
    ["days", 0, 0, 0, -32],
  ],
  [
    date25640716, date25640813, "28 days across a month which has 31 days",
    ["years", 0, 0, 0, -28],
    ["months", 0, 0, 0, -28],
    ["weeks", 0, 0, -4, 0],
  ],
  [
    date25640716, date25640916, "2 months which both have 31 days",
    ["years", 0, -2, 0, 0],
    ["months", 0, -2, 0, 0],
    ["weeks", 0, 0, -8, -6],
    ["days", 0, 0, 0, -62],
  ],
  [
    date25640716, date25650716, "1 year",
    ["years", -1, 0, 0, 0],
    ["months", 0, -12, 0, 0],
    ["weeks", 0, 0, -52, -1],
    ["days", 0, 0, 0, -365],
  ],
  [
    date25630201, date25640201, "start of February",
    ["years", -1, 0, 0, 0],
    ["months", 0, -12, 0, 0],
  ],
  [
    date25640228, date25650228, "end of February",
    ["years", -1, 0, 0, 0],
    ["months", 0, -12, 0, 0],
  ],
  [
    date25620101, date25620201, "length of January 2562",
    ["days", 0, 0, 0, -31],
  ],
  [
    date25640716, date25740716, "10 years",
    ["years", -10, 0, 0, 0],
    ["months", 0, -120, 0, 0],
    ["weeks", 0, 0, -521, -5],
    ["days", 0, 0, 0, -3652],
  ],
  [
    date25640716, date25650719, "1 year and 3 days",
    ["years", -1, 0, 0, -3],
  ],
  [
    date25640716, date25650919, "1 year 2 months and 3 days",
    ["years", -1, -2, 0, -3],
  ],
  [
    date25640716, date25741216, "10 years and 5 months",
    ["years", -10, -5, 0, 0],
  ],
  [
    date25401216, date25640716, "23 years and 7 months",
    ["years", -23, -7, 0, 0],
  ],
  [
    date25400716, date25640716, "24 years",
    ["years", -24, 0, 0, 0],
  ],
  [
    date25400716, date25640715, "23 years, 11 months and 29 days",
    ["years", -23, -11, 0, -29],
  ],
  [
    date25400616, date25640615, "23 years, 11 months and 30 days",
    ["years", -23, -11, 0, -30],
  ],
  [
    date25030216, date25630316, "60 years, 1 month",
    ["years", -60, -1, 0, 0],
  ],
  [
    date25640330, date25640716, "3 months and 16 days",
    ["years", 0, -3, 0, -16],
  ],
  [
    date25630330, date25640716, "1 year, 3 months and 16 days",
    ["years", -1, -3, 0, -16],
  ],
  [
    date25030330, date25640716, "61 years, 3 months and 16 days",
    ["years", -61, -3, 0, -16],
  ],
  [
    date25621230, date25640716, "1 year, 6 months and 16 days",
    ["years", -1, -6, 0, -16],
  ],
  [
    date25551201, date25560601, "6 months",
    ["months", 0, -6, 0, 0],
  ],
  [
    date25631230, date25640716, "6 months and 16 days",
    ["years", 0, -6, 0, -16],
  ],
  [
    date25550101, date25551007, "40 weeks",
    ["weeks", 0, 0, -40, 0],
    ["days", 0, 0, 0, -280],
  ],
  [
    date25401230, date25640716, "23 years, 6 months and 16 days",
    ["years", -23, -6, 0, -16],
  ],
  [
    date25621230, date25640305, "1 year, 2 months and 5 days",
    ["years", -1, -2, 0, -5],
  ],
  [
    date25551201, date25590618, "3 years, 6 months, and 17 days",
    ["years", -3, -6, 0, -17],
  ],
  [
    date25120724, date25620724, "crossing epoch",
    ["years", -50, 0, 0, 0],
  ],
  [
    date25640717, date25640716, "negative one day",
    ["years", 0, 0, 0, 1],
    ["months", 0, 0, 0, 1],
    ["weeks", 0, 0, 0, 1],
    ["days", 0, 0, 0, 1],
  ],
  [
    date25640723, date25640716, "negative 7 days",
    ["years", 0, 0, 0, 7],
    ["months", 0, 0, 0, 7],
    ["weeks", 0, 0, 1, 0],
  ],
  [
    date25640816, date25640716, "negative 1 month in same year",
    ["years", 0, 1, 0, 0],
    ["months", 0, 1, 0, 0],
    ["weeks", 0, 0, 4, 3],
  ],
  [
    date25640116, date25631216, "negative 1 month in different year",
    ["years", 0, 1, 0, 0],
    ["months", 0, 1, 0, 0],
  ],
  [
    date25640205, date25640105, "negative 1 month in same year",
    ["years", 0, 1, 0, 0],
    ["months", 0, 1, 0, 0],
  ],
  [
    date25640817, date25640716, "negative 1 month and 1 day in a month with 31 days",
    ["years", 0, 1, 0, 1],
    ["months", 0, 1, 0, 1],
    ["days", 0, 0, 0, 32],
  ],
  [
    date25640813, date25640716, "negative 28 days across a month which has 31 days",
    ["years", 0, 0, 0, 28],
    ["months", 0, 0, 0, 28],
    ["weeks", 0, 0, 4, 0],
  ],
  [
    date25640916, date25640716, "negative 2 months which both have 31 days",
    ["years", 0, 2, 0, 0],
    ["months", 0, 2, 0, 0],
    ["weeks", 0, 0, 8, 6],
    ["days", 0, 0, 0, 62],
  ],
  [
    date25650716, date25640716, "negative 1 year",
    ["years", 1, 0, 0, 0],
    ["months", 0, 12, 0, 0],
    ["weeks", 0, 0, 52, 1],
    ["days", 0, 0, 0, 365],
  ],
  [
    date25740716, date25640716, "negative 10 years",
    ["years", 10, 0, 0, 0],
    ["months", 0, 120, 0, 0],
    ["weeks", 0, 0, 521, 5],
    ["days", 0, 0, 0, 3652],
  ],
  [
    date25650719, date25640716, "negative 1 year and 3 days",
    ["years", 1, 0, 0, 3],
  ],
  [
    date25650919, date25640716, "negative 1 year 2 months and 3 days",
    ["years", 1, 2, 0, 3],
  ],
  [
    date25741216, date25640716, "negative 10 years and 5 months",
    ["years", 10, 5, 0, 0],
  ],
  [
    date25640716, date25401216, "negative 23 years and 7 months",
    ["years", 23, 7, 0, 0],
  ],
  [
    date25640716, date25400716, "negative 24 years",
    ["years", 24, 0, 0, 0],
  ],
  [
    date25640715, date25400716, "negative 23 years, 11 months and 30 days",
    ["years", 23, 11, 0, 30],
  ],
  [
    date25640615, date25400616, "negative 23 years, 11 months and 29 days",
    ["years", 23, 11, 0, 29],
  ],
  [
    date25630316, date25030216, "negative 60 years, 1 month",
    ["years", 60, 1, 0, 0],
  ],
  [
    date25640716, date25640330, "negative 3 months and 17 days",
    ["years", 0, 3, 0, 17],
  ],
  [
    date25640716, date25630330, "negative 1 year, 3 months and 17 days",
    ["years", 1, 3, 0, 17],
  ],
  [
    date25640716, date25030330, "negative 61 years, 3 months and 17 days",
    ["years", 61, 3, 0, 17],
  ],
  [
    date25640716, date25621230, "negative 1 year, 6 months and 17 days",
    ["years", 1, 6, 0, 17],
  ],
  [
    date25640716, date25631230, "negative 6 months and 17 days",
    ["years", 0, 6, 0, 17],
  ],
  [
    date25640716, date25401230, "negative 23 years, 6 months and 17 days",
    ["years", 23, 6, 0, 17],
  ],
  [
    date25640305, date25621230, "negative 1 year, 2 months and 6 days",
    ["years", 1, 2, 0, 6],
  ],
  [
    date25620724, date25120724, "crossing epoch",
    ["years", 50, 0, 0, 0],
  ],
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
