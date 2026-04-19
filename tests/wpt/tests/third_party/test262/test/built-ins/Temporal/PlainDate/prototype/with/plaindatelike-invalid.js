// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.with
description: Throws TypeError on an argument that is not a PlainDate-like property bag
features: [Temporal]
---*/

const plainDate = new Temporal.PlainDate(1976, 11, 18);

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
  [NaN, "NaN"],
  [Infinity, "Infinity"],

  // Step 4.
  //   RejectObjectWithCalendarOrTimeZone step 2.
  [Temporal.PlainDate.from("2019-05-17"), "PlainDate"],
  [Temporal.PlainDateTime.from("2019-05-17T12:34"), "PlainDateTime"],
  [Temporal.PlainMonthDay.from("2019-05-17"), "PlainMonthDay"],
  [Temporal.PlainTime.from("12:34"), "PlainTime"],
  [Temporal.PlainYearMonth.from("2019-05-17"), "PlainYearMonth"],
  [Temporal.ZonedDateTime.from("2019-05-17T12:34Z[UTC]"), "ZonedDateTime"],
  //   RejectObjectWithCalendarOrTimeZone step 3-4.
  [{ year: 2021, calendar: "iso8601" }, "calendar"],
  //   RejectObjectWithCalendarOrTimeZone step 5-6.
  [{ year: 2021, timeZone: "UTC" }, "timeZone"],

  // Step 6.
  [{}, "empty object"],
  [[], "array"],
  [{ months: 12 }, "only plural property"],
];

for (const [value, message = String(value)] of tests) {
  assert.throws(TypeError, () => plainDate.with(value), message);
}
