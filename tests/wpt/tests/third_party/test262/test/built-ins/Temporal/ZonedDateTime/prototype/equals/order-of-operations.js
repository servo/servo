// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.equals
description: Properties on objects passed to equals() are accessed in the correct order
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const expected = [
  "get other.calendar",
  // PrepareTemporalFields
  "get other.day",
  "get other.day.valueOf",
  "call other.day.valueOf",
  "get other.hour",
  "get other.hour.valueOf",
  "call other.hour.valueOf",
  "get other.microsecond",
  "get other.microsecond.valueOf",
  "call other.microsecond.valueOf",
  "get other.millisecond",
  "get other.millisecond.valueOf",
  "call other.millisecond.valueOf",
  "get other.minute",
  "get other.minute.valueOf",
  "call other.minute.valueOf",
  "get other.month",
  "get other.month.valueOf",
  "call other.month.valueOf",
  "get other.monthCode",
  "get other.monthCode.toString",
  "call other.monthCode.toString",
  "get other.nanosecond",
  "get other.nanosecond.valueOf",
  "call other.nanosecond.valueOf",
  "get other.offset",
  "get other.offset.toString",
  "call other.offset.toString",
  "get other.second",
  "get other.second.valueOf",
  "call other.second.valueOf",
  "get other.timeZone",
  "get other.year",
  "get other.year.valueOf",
  "call other.year.valueOf",
];
const actual = [];

const other = TemporalHelpers.propertyBagObserver(actual, {
  year: 2001,
  month: 5,
  monthCode: "M05",
  day: 2,
  hour: 6,
  minute: 54,
  second: 32,
  millisecond: 987,
  microsecond: 654,
  nanosecond: 321,
  offset: "+00:00",
  calendar: "iso8601",
  timeZone: "UTC",
}, "other", ["calendar", "timeZone"]);

const instance = new Temporal.ZonedDateTime(988786472_987_654_321n,  /* 2001-05-02T06:54:32.987654321Z */ "UTC");

instance.equals(other);
assert.compareArray(actual, expected, "order of operations");
