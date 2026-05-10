// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.prototype.with
description: Throws TypeError on an argument that is not a PlainTime-like property bag
features: [Temporal]
---*/

const plainTime = new Temporal.PlainTime(12, 34, 56, 987, 654, 321);

const tests = [
  // Step 3.
  [undefined],
  [null],
  [true],
  ["2019-05-17"],
  ["2019-05-17T12:34"],
  ["2019-05-17T12:34Z"],
  ["18:05:42.577"],
  ["42"],
  [Symbol(), "symbol"],
  [42, "number"],
  [42n, "bigint"],

  // Step 4.
  [Temporal.PlainDate.from("2019-05-17"), "PlainDate"],
  [Temporal.PlainDateTime.from("2019-05-17T12:34"), "PlainDateTime"],
  [Temporal.PlainMonthDay.from("2019-05-17"), "PlainMonthDay"],
  [Temporal.PlainTime.from("12:34"), "PlainTime"],
  [Temporal.PlainYearMonth.from("2019-05-17"), "PlainYearMonth"],
  [Temporal.ZonedDateTime.from("2019-05-17T12:34Z[UTC]"), "ZonedDateTime"],

  // Step 5-6.
  [{ hour: 14, calendar: "iso8601" }, "calendar"],

  // Step 7-8.
  [{ hour: 14, timeZone: "UTC" }, "timeZone"],

  // Step 9.
  [{}, "empty object"],
  [{ hours: 14 }, "only plural property"],
];

for (const [value, message = String(value)] of tests) {
  assert.throws(TypeError, () => plainTime.with(value), message);
}
