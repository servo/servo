// Copyright (C) 2021 Igalia, S.L., and the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.until
description: Check various basic calculations involving leap years
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const date19600216 = Temporal.PlainDate.from("1960-02-16");
const date20190101 = Temporal.PlainDate.from("2019-01-01");
const date20190201 = Temporal.PlainDate.from("2019-02-01");
const date20190301 = Temporal.PlainDate.from("2019-03-01");
const date20190601 = Temporal.PlainDate.from("2019-06-01");
const date20200101 = Temporal.PlainDate.from("2020-01-01");
const date20200201 = Temporal.PlainDate.from("2020-02-01");
const date20200301 = Temporal.PlainDate.from("2020-03-01");
const date20200315 = Temporal.PlainDate.from("2020-03-15");
const date20200601 = Temporal.PlainDate.from("2020-06-01");
const date20210101 = Temporal.PlainDate.from("2021-01-01");
const date20210107 = Temporal.PlainDate.from("2021-01-07");
const date20210201 = Temporal.PlainDate.from("2021-02-01");
const date20210228 = new Temporal.PlainDate(2021, 2, 28);
const date20210307 = Temporal.PlainDate.from("2021-03-07");
const date20210315 = Temporal.PlainDate.from("2021-03-15");
const date20210601 = Temporal.PlainDate.from("2021-06-01");
const date20220228 = new Temporal.PlainDate(2022, 2, 28);

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
