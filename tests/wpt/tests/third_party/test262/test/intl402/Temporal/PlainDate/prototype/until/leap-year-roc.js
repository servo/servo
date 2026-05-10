// Copyright (C) 2025 Igalia, S.L., and the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.until
description: Check various basic calculations involving leap years (roc calendar)
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "roc";
const options = { overflow: "reject" };

const date0490216 = Temporal.PlainDate.from({ year: 49, monthCode: "M02", day: 16, calendar }, options);
const date1080101 = Temporal.PlainDate.from({ year: 108, monthCode: "M01", day: 1, calendar }, options);
const date1080201 = Temporal.PlainDate.from({ year: 108, monthCode: "M02", day: 1, calendar }, options);
const date1080301 = Temporal.PlainDate.from({ year: 108, monthCode: "M03", day: 1, calendar }, options);
const date1080601 = Temporal.PlainDate.from({ year: 108, monthCode: "M06", day: 1, calendar }, options);
const date1090101 = Temporal.PlainDate.from({ year: 109, monthCode: "M01", day: 1, calendar }, options);
const date1090201 = Temporal.PlainDate.from({ year: 109, monthCode: "M02", day: 1, calendar }, options);
const date1090301 = Temporal.PlainDate.from({ year: 109, monthCode: "M03", day: 1, calendar }, options);
const date1090315 = Temporal.PlainDate.from({ year: 109, monthCode: "M03", day: 15, calendar }, options);
const date1090601 = Temporal.PlainDate.from({ year: 109, monthCode: "M06", day: 1, calendar }, options);
const date1100101 = Temporal.PlainDate.from({ year: 110, monthCode: "M01", day: 1, calendar }, options);
const date1100107 = Temporal.PlainDate.from({ year: 110, monthCode: "M01", day: 7, calendar }, options);
const date1100201 = Temporal.PlainDate.from({ year: 110, monthCode: "M02", day: 1, calendar }, options);
const date1100228 = Temporal.PlainDate.from({ year: 110, monthCode: "M02", day: 28, calendar }, options);
const date1100307 = Temporal.PlainDate.from({ year: 110, monthCode: "M03", day: 7, calendar }, options);
const date1100315 = Temporal.PlainDate.from({ year: 110, monthCode: "M03", day: 15, calendar }, options);
const date1100601 = Temporal.PlainDate.from({ year: 110, monthCode: "M06", day: 1, calendar }, options);
const date1110228 = Temporal.PlainDate.from({ year: 111, monthCode: "M02", day: 28, calendar }, options);

const tests = [
  [
    date1100107, date1100307, "2 months in same year across Feb 28",
    ["years", 0, 2, 0, 0],
    ["months", 0, 2, 0, 0],
  ],
  [
    date0490216, date1100315, "61 years, 27 days in common year",
    ["years", 61, 0, 0, 27],
  ],
  [
    date0490216, date1090315, "60 years, 28 days in leap year",
    ["years", 60, 0, 0, 28],
  ],
  [
    date1100315, date0490216, "negative 61 years, 28 days in common year",
    ["years", -61, 0, 0, -28],
  ],
  [
    date1090315, date0490216, "negative 60 years, 28 days in leap year",
    ["years", -60, 0, 0, -28],
  ],
  [
    date1100307, date1100107, "negative 2 month in same year across Feb 28",
    ["years", 0, -2, 0, 0],
    ["months", 0, -2, 0, 0],
  ],
  [
    date1090201, date1100201, "year including leap day",
    ["weeks", 0, 0, 52, 2],
  ],
  [
    date1100228, date1110228, "year not including leap day",
    ["weeks", 0, 0, 52, 1],
  ],
  [
    date1080101, date1090101, "length of year from January 108",
    ["days", 0, 0, 0, 365],
  ],
  [
    date1090101, date1100101, "length of year from January 109",
    ["days", 0, 0, 0, 366],
  ],
  [
    date1080601, date1090601, "length of year from June 108",
    ["days", 0, 0, 0, 366],
  ],
  [
    date1090601, date1100601, "length of year from June 109",
    ["days", 0, 0, 0, 365],
  ],
  [
    date1080201, date1080301, "length of Feb 108",
    ["days", 0, 0, 0, 28],
  ],
  [
    date1090201, date1090301, "length of Feb 109",
    ["days", 0, 0, 0, 29],
  ],
];

for (const [one, two, descr, ...units] of tests) {
  for (const [largestUnit, years, months, weeks, days] of units) {
    TemporalHelpers.assertDuration(
      one.until(two, { largestUnit }),
      years, months, weeks, days, 0, 0, 0, 0, 0, 0,
      descr
    );
  }
}
