// Copyright (C) 2020 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.add
description: Properties on an object passed to add() are accessed in the correct order
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const expectedOpsForPrimitiveOptions = [
  // ToTemporalDurationRecord
  "get fields.days",
  "get fields.days.valueOf",
  "call fields.days.valueOf",
  "get fields.hours",
  "get fields.hours.valueOf",
  "call fields.hours.valueOf",
  "get fields.microseconds",
  "get fields.microseconds.valueOf",
  "call fields.microseconds.valueOf",
  "get fields.milliseconds",
  "get fields.milliseconds.valueOf",
  "call fields.milliseconds.valueOf",
  "get fields.minutes",
  "get fields.minutes.valueOf",
  "call fields.minutes.valueOf",
  "get fields.months",
  "get fields.months.valueOf",
  "call fields.months.valueOf",
  "get fields.nanoseconds",
  "get fields.nanoseconds.valueOf",
  "call fields.nanoseconds.valueOf",
  "get fields.seconds",
  "get fields.seconds.valueOf",
  "call fields.seconds.valueOf",
  "get fields.weeks",
  "get fields.weeks.valueOf",
  "call fields.weeks.valueOf",
  "get fields.years",
  "get fields.years.valueOf",
  "call fields.years.valueOf",
];
const expected = expectedOpsForPrimitiveOptions.concat([
  "get options.overflow",
  "get options.overflow.toString",
  "call options.overflow.toString",
]);
const actual = [];

const instance = new Temporal.PlainDate(2000, 5, 2, "iso8601");

const fields = TemporalHelpers.propertyBagObserver(actual, {
  years: 1,
  months: 1,
  weeks: 1,
  days: 1,
  hours: 1,
  minutes: 1,
  seconds: 1,
  milliseconds: 1,
  microseconds: 1,
  nanoseconds: 1,
}, "fields");

const options = TemporalHelpers.propertyBagObserver(actual, {
  overflow: "constrain",
}, "options");

instance.add(fields, options);
assert.compareArray(actual, expected, "order of operations");

actual.splice(0); // clear

assert.throws(TypeError, () => instance.add(fields, null));
assert.compareArray(actual, expectedOpsForPrimitiveOptions,
  "duration fields are read before TypeError is thrown for primitive options");

actual.splice(0); // clear
