// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.compare
description: Properties on objects passed to compare() are accessed in the correct order
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const expected = [
  "get one.calendar",
  // PrepareTemporalFields
  "get one.day",
  "get one.day.valueOf",
  "call one.day.valueOf",
  "get one.hour",
  "get one.hour.valueOf",
  "call one.hour.valueOf",
  "get one.microsecond",
  "get one.microsecond.valueOf",
  "call one.microsecond.valueOf",
  "get one.millisecond",
  "get one.millisecond.valueOf",
  "call one.millisecond.valueOf",
  "get one.minute",
  "get one.minute.valueOf",
  "call one.minute.valueOf",
  "get one.month",
  "get one.month.valueOf",
  "call one.month.valueOf",
  "get one.monthCode",
  "get one.monthCode.toString",
  "call one.monthCode.toString",
  "get one.nanosecond",
  "get one.nanosecond.valueOf",
  "call one.nanosecond.valueOf",
  "get one.offset",
  "get one.offset.toString",
  "call one.offset.toString",
  "get one.second",
  "get one.second.valueOf",
  "call one.second.valueOf",
  "get one.timeZone",
  "get one.year",
  "get one.year.valueOf",
  "call one.year.valueOf",
  // Same set of operations, for the other argument:
  "get two.calendar",
  // PrepareTemporalFields
  "get two.day",
  "get two.day.valueOf",
  "call two.day.valueOf",
  "get two.hour",
  "get two.hour.valueOf",
  "call two.hour.valueOf",
  "get two.microsecond",
  "get two.microsecond.valueOf",
  "call two.microsecond.valueOf",
  "get two.millisecond",
  "get two.millisecond.valueOf",
  "call two.millisecond.valueOf",
  "get two.minute",
  "get two.minute.valueOf",
  "call two.minute.valueOf",
  "get two.month",
  "get two.month.valueOf",
  "call two.month.valueOf",
  "get two.monthCode",
  "get two.monthCode.toString",
  "call two.monthCode.toString",
  "get two.nanosecond",
  "get two.nanosecond.valueOf",
  "call two.nanosecond.valueOf",
  "get two.offset",
  "get two.offset.toString",
  "call two.offset.toString",
  "get two.second",
  "get two.second.valueOf",
  "call two.second.valueOf",
  "get two.timeZone",
  "get two.year",
  "get two.year.valueOf",
  "call two.year.valueOf",
];
const actual = [];

const one = TemporalHelpers.propertyBagObserver(actual, {
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
}, "one", ["calendar", "timeZone"]);

const two = TemporalHelpers.propertyBagObserver(actual, {
  year: 2014,
  month: 7,
  monthCode: "M07",
  day: 19,
  hour: 12,
  minute: 34,
  second: 56,
  millisecond: 123,
  microsecond: 456,
  nanosecond: 789,
  offset: "+00:00",
  calendar: "iso8601",
  timeZone: "UTC",
}, "two", ["calendar", "timeZone"]);

Temporal.ZonedDateTime.compare(one, two);
assert.compareArray(actual, expected, "order of operations");
