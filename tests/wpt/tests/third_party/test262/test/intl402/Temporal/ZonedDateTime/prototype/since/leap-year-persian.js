// Copyright (C) 2025 Igalia, S.L., and the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.since
description: Check various basic calculations involving leap years (Persian calendar)
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "persian";
const options = { overflow: "reject" };

const date13011130 = Temporal.ZonedDateTime.from({ year: 1301, monthCode: "M11", day: 30, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const date13021130 = Temporal.ZonedDateTime.from({ year: 1302, monthCode: "M11", day: 30, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const date13030218 = Temporal.ZonedDateTime.from({ year: 1303, monthCode: "M02", day: 18, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const date13610101 = Temporal.ZonedDateTime.from({ year: 1361, monthCode: "M01", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const date13610201 = Temporal.ZonedDateTime.from({ year: 1361, monthCode: "M02", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const date13610301 = Temporal.ZonedDateTime.from({ year: 1361, monthCode: "M03", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const date13610601 = Temporal.ZonedDateTime.from({ year: 1361, monthCode: "M06", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const date13611201 = Temporal.ZonedDateTime.from({ year: 1361, monthCode: "M12", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const date13620101 = Temporal.ZonedDateTime.from({ year: 1362, monthCode: "M01", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const date13620201 = Temporal.ZonedDateTime.from({ year: 1362, monthCode: "M02", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const date13620301 = Temporal.ZonedDateTime.from({ year: 1362, monthCode: "M03", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const date13620315 = Temporal.ZonedDateTime.from({ year: 1362, monthCode: "M03", day: 15, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const date13620601 = Temporal.ZonedDateTime.from({ year: 1362, monthCode: "M06", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const date13621201 = Temporal.ZonedDateTime.from({ year: 1362, monthCode: "M12", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const date13621230 = Temporal.ZonedDateTime.from({ year: 1362, monthCode: "M12", day: 30, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const date13630101 = Temporal.ZonedDateTime.from({ year: 1363, monthCode: "M01", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const date13630107 = Temporal.ZonedDateTime.from({ year: 1363, monthCode: "M01", day: 7, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const date13630201 = Temporal.ZonedDateTime.from({ year: 1363, monthCode: "M02", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const date13630228 = Temporal.ZonedDateTime.from({ year: 1363, monthCode: "M02", day: 28, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const date13630307 = Temporal.ZonedDateTime.from({ year: 1363, monthCode: "M03", day: 7, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const date13630315 = Temporal.ZonedDateTime.from({ year: 1363, monthCode: "M03", day: 15, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const date13630601 = Temporal.ZonedDateTime.from({ year: 1363, monthCode: "M06", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const date13631107 = Temporal.ZonedDateTime.from({ year: 1363, monthCode: "M11", day: 7, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const date13631229 = Temporal.ZonedDateTime.from({ year: 1363, monthCode: "M12", day: 29, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const date13640107 = Temporal.ZonedDateTime.from({ year: 1364, monthCode: "M01", day: 7, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const date13640228 = Temporal.ZonedDateTime.from({ year: 1364, monthCode: "M02", day: 28, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const date13640315 = Temporal.ZonedDateTime.from({ year: 1364, monthCode: "M03", day: 15, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);

const tests = [
  [
    date13630107, date13630307, "2 months in same year across Feb 28",
    ["years", 0, -2, 0, 0],
    ["months", 0, -2, 0, 0],
  ],
  [
    date13021130, date13631229, "61 years, 29 days in common year",
    ["years", -61, 0, 0, -29],
  ],
  [
    date13021130, date13621230, "60 years, 1 month in leap year",
    ["years", -60, -1, 0, 0],
  ],
  [
    date13640315, date13030218, "negative 61 years, 28 days in common year",
    ["years", 61, 0, 0, 28],
  ],
  [
    date13621230, date13021130, "negative 60 years, 1 month in leap year",
    ["years", 60, 1, 0, 0],
  ],
  [
    date13640107, date13631107, "negative 2 months in different years across Esfand 29",
    ["years", 0, 2, 0, 0],
    ["months", 0, 2, 0, 0],
  ],
  [
    date13620201, date13630201, "year including leap day",
    ["weeks", 0, 0, -52, -2],
  ],
  [
    date13630228, date13640228, "year not including leap day",
    ["weeks", 0, 0, -52, -1],
  ],
  [
    date13610101, date13620101, "length of year from Farvardin 1361",
    ["days", 0, 0, 0, -365],
  ],
  [
    date13620101, date13630101, "length of year from Farvardin 1362",
    ["days", 0, 0, 0, -366],
  ],
  [
    date13610601, date13620601, "length of year from Shahrivar 1361",
    ["days", 0, 0, 0, -365],
  ],
  [
    date13620601, date13630601, "length of year from Shahrivar 1362",
    ["days", 0, 0, 0, -366],
  ],
  [
    date13611201, date13620101, "length of Esfand 1361",
    ["days", 0, 0, 0, -29],
  ],
  [
    date13621201, date13630101, "length of Esfand 1362",
    ["days", 0, 0, 0, -30],
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
