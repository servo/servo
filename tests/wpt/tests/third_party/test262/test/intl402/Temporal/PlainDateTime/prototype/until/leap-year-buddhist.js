// Copyright (C) 2025 Igalia, S.L., and the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.until
description: Check various basic calculations involving leap years (buddhist calendar)
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "buddhist";
const options = { overflow: "reject" };

const date25030216 = Temporal.PlainDateTime.from({ year: 2503, monthCode: "M02", day: 16, hour: 12, minute: 34, calendar }, options);
const date25620101 = Temporal.PlainDateTime.from({ year: 2562, monthCode: "M01", day: 1, hour: 12, minute: 34, calendar }, options);
const date25620201 = Temporal.PlainDateTime.from({ year: 2562, monthCode: "M02", day: 1, hour: 12, minute: 34, calendar }, options);
const date25620301 = Temporal.PlainDateTime.from({ year: 2562, monthCode: "M03", day: 1, hour: 12, minute: 34, calendar }, options);
const date25620601 = Temporal.PlainDateTime.from({ year: 2562, monthCode: "M06", day: 1, hour: 12, minute: 34, calendar }, options);
const date25630101 = Temporal.PlainDateTime.from({ year: 2563, monthCode: "M01", day: 1, hour: 12, minute: 34, calendar }, options);
const date25630201 = Temporal.PlainDateTime.from({ year: 2563, monthCode: "M02", day: 1, hour: 12, minute: 34, calendar }, options);
const date25630301 = Temporal.PlainDateTime.from({ year: 2563, monthCode: "M03", day: 1, hour: 12, minute: 34, calendar }, options);
const date25630315 = Temporal.PlainDateTime.from({ year: 2563, monthCode: "M03", day: 15, hour: 12, minute: 34, calendar }, options);
const date25630601 = Temporal.PlainDateTime.from({ year: 2563, monthCode: "M06", day: 1, hour: 12, minute: 34, calendar }, options);
const date25640101 = Temporal.PlainDateTime.from({ year: 2564, monthCode: "M01", day: 1, hour: 12, minute: 34, calendar }, options);
const date25640107 = Temporal.PlainDateTime.from({ year: 2564, monthCode: "M01", day: 7, hour: 12, minute: 34, calendar }, options);
const date25640201 = Temporal.PlainDateTime.from({ year: 2564, monthCode: "M02", day: 1, hour: 12, minute: 34, calendar }, options);
const date25640228 = Temporal.PlainDateTime.from({ year: 2564, monthCode: "M02", day: 28, hour: 12, minute: 34, calendar }, options);
const date25640307 = Temporal.PlainDateTime.from({ year: 2564, monthCode: "M03", day: 7, hour: 12, minute: 34, calendar }, options);
const date25640315 = Temporal.PlainDateTime.from({ year: 2564, monthCode: "M03", day: 15, hour: 12, minute: 34, calendar }, options);
const date25640601 = Temporal.PlainDateTime.from({ year: 2564, monthCode: "M06", day: 1, hour: 12, minute: 34, calendar }, options);
const date25650228 = Temporal.PlainDateTime.from({ year: 2565, monthCode: "M02", day: 28, hour: 12, minute: 34, calendar }, options);

const tests = [
  [
    date25640107, date25640307, "2 months in same year across Feb 28",
    ["years", 0, 2, 0, 0],
    ["months", 0, 2, 0, 0],
  ],
  [
    date25030216, date25640315, "61 years, 27 days in common year",
    ["years", 61, 0, 0, 27],
  ],
  [
    date25030216, date25630315, "60 years, 28 days in leap year",
    ["years", 60, 0, 0, 28],
  ],
  [
    date25640315, date25030216, "negative 61 years, 28 days in common year",
    ["years", -61, 0, 0, -28],
  ],
  [
    date25630315, date25030216, "negative 60 years, 28 days in leap year",
    ["years", -60, 0, 0, -28],
  ],
  [
    date25640307, date25640107, "negative 2 month in same year across Feb 28",
    ["years", 0, -2, 0, 0],
    ["months", 0, -2, 0, 0],
  ],
  [
    date25630201, date25640201, "year including leap day",
    ["weeks", 0, 0, 52, 2],
  ],
  [
    date25640228, date25650228, "year not including leap day",
    ["weeks", 0, 0, 52, 1],
  ],
  [
    date25620101, date25630101, "length of year from January 2562",
    ["days", 0, 0, 0, 365],
  ],
  [
    date25630101, date25640101, "length of year from January 2563",
    ["days", 0, 0, 0, 366],
  ],
  [
    date25620601, date25630601, "length of year from June 2562",
    ["days", 0, 0, 0, 366],
  ],
  [
    date25630601, date25640601, "length of year from June 2563",
    ["days", 0, 0, 0, 365],
  ],
  [
    date25620201, date25620301, "length of Feb 2562",
    ["days", 0, 0, 0, 28],
  ],
  [
    date25630201, date25630301, "length of Feb 2563",
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
