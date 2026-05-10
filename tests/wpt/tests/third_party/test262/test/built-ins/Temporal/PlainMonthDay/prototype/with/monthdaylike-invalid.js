// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.prototype.with
description: >
  Throws TypeError on an argument that is not a PlainMonthDay-like property bag
info: |
  Temporal.PlainMonthDay.prototype.with ( temporalMonthDayLike [ , options ] )

  ...
  3. If ? IsPartialTemporalObject(temporalMonthDayLike) is false, throw a TypeError exception.
  ...
  6. Let partialMonthDay be ? PrepareCalendarFields(calendar, temporalMonthDayLike, « year, month, month-code, day », « », partial).
  ...
features: [Temporal]
---*/

const plainMonthDay = new Temporal.PlainMonthDay(1, 1);

const tests = [
  // Step 3.
  // IsPartialTemporalObject, step 1.
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

  // IsPartialTemporalObject, step 2.
  [Temporal.PlainDate.from("2019-05-17"), "PlainDate"],
  [Temporal.PlainDateTime.from("2019-05-17T12:34"), "PlainDateTime"],
  [Temporal.PlainMonthDay.from("2019-05-17"), "PlainMonthDay"],
  [Temporal.PlainTime.from("12:34"), "PlainTime"],
  [Temporal.PlainYearMonth.from("2019-05-17"), "PlainYearMonth"],
  [Temporal.ZonedDateTime.from("2019-05-17T12:34Z[UTC]"), "ZonedDateTime"],

  // IsPartialTemporalObject, step 3.
  [{ year: 2021, calendar: "iso8601" }, "calendar"],

  // IsPartialTemporalObject, step 4.
  [{ year: 2021, timeZone: "UTC" }, "timeZone"],

  // Step 6.
  // PrepareCalendarFields, step 10.
  [{}, "empty object"],
  [{ months: 12 }, "only plural property"],
];

for (const [value, message = String(value)] of tests) {
  assert.throws(TypeError, () => plainMonthDay.with(value), message);
}
