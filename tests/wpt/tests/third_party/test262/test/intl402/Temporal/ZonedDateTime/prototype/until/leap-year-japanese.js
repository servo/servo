// Copyright (C) 2025 Igalia, S.L., and the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.until
description: Check various basic calculations involving leap years (japanese calendar)
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "japanese";
const options = { overflow: "reject" };

const date19600216 = Temporal.ZonedDateTime.from({ year: 1960, monthCode: "M02", day: 16, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const date20190101 = Temporal.ZonedDateTime.from({ year: 2019, monthCode: "M01", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const date20190201 = Temporal.ZonedDateTime.from({ year: 2019, monthCode: "M02", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const date20190301 = Temporal.ZonedDateTime.from({ year: 2019, monthCode: "M03", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const date20190601 = Temporal.ZonedDateTime.from({ year: 2019, monthCode: "M06", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const date20200101 = Temporal.ZonedDateTime.from({ year: 2020, monthCode: "M01", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const date20200201 = Temporal.ZonedDateTime.from({ year: 2020, monthCode: "M02", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const date20200301 = Temporal.ZonedDateTime.from({ year: 2020, monthCode: "M03", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const date20200315 = Temporal.ZonedDateTime.from({ year: 2020, monthCode: "M03", day: 15, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const date20200601 = Temporal.ZonedDateTime.from({ year: 2020, monthCode: "M06", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const date20210101 = Temporal.ZonedDateTime.from({ year: 2021, monthCode: "M01", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const date20210107 = Temporal.ZonedDateTime.from({ year: 2021, monthCode: "M01", day: 7, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const date20210201 = Temporal.ZonedDateTime.from({ year: 2021, monthCode: "M02", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const date20210228 = Temporal.ZonedDateTime.from({ year: 2021, monthCode: "M02", day: 28, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const date20210307 = Temporal.ZonedDateTime.from({ year: 2021, monthCode: "M03", day: 7, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const date20210315 = Temporal.ZonedDateTime.from({ year: 2021, monthCode: "M03", day: 15, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const date20210601 = Temporal.ZonedDateTime.from({ year: 2021, monthCode: "M06", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const date20220228 = Temporal.ZonedDateTime.from({ year: 2022, monthCode: "M02", day: 28, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);

const tests = [
  [
    date20210107, date20210307, "2 months in same year across Feb 28",
    ["years", 0, 2, 0, 0],
    ["months", 0, 2, 0, 0],
  ],
  [
    date19600216, date20210315, "61 years, 27 days in common year",
    ["years", 61, 0, 0, 27],
  ],
  [
    date19600216, date20200315, "60 years, 28 days in leap year",
    ["years", 60, 0, 0, 28],
  ],
  [
    date20210315, date19600216, "negative 61 years, 28 days in common year",
    ["years", -61, 0, 0, -28],
  ],
  [
    date20200315, date19600216, "negative 60 years, 28 days in leap year",
    ["years", -60, 0, 0, -28],
  ],
  [
    date20210307, date20210107, "negative 2 month in same year across Feb 28",
    ["years", 0, -2, 0, 0],
    ["months", 0, -2, 0, 0],
  ],
  [
    date20200201, date20210201, "year including leap day",
    ["weeks", 0, 0, 52, 2],
  ],
  [
    date20210228, date20220228, "year not including leap day",
    ["weeks", 0, 0, 52, 1],
  ],
  [
    date20190101, date20200101, "length of year from January 2019",
    ["days", 0, 0, 0, 365],
  ],
  [
    date20200101, date20210101, "length of year from January 2020",
    ["days", 0, 0, 0, 366],
  ],
  [
    date20190601, date20200601, "length of year from June 2019",
    ["days", 0, 0, 0, 366],
  ],
  [
    date20200601, date20210601, "length of year from June 2020",
    ["days", 0, 0, 0, 365],
  ],
  [
    date20190201, date20190301, "length of Feb 2019",
    ["days", 0, 0, 0, 28],
  ],
  [
    date20200201, date20200301, "length of Feb 2020",
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
