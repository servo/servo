// Copyright (C) 2020 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.with
description: Properties on an object passed to with() are accessed in the correct order
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const expectedOpsForPrimitiveOptions = [
  // RejectObjectWithCalendarOrTimeZone
  "get fields.calendar",
  "get fields.timeZone",
  // PrepareTemporalFields on argument
  "get fields.day",
  "get fields.day.valueOf",
  "call fields.day.valueOf",
  "get fields.month",
  "get fields.month.valueOf",
  "call fields.month.valueOf",
  "get fields.monthCode",
  "get fields.monthCode.toString",
  "call fields.monthCode.toString",
  "get fields.year",
  "get fields.year.valueOf",
  "call fields.year.valueOf",
];
const expected = expectedOpsForPrimitiveOptions.concat([
  // GetTemporalOverflowOption
  "get options.overflow",
  "get options.overflow.toString",
  "call options.overflow.toString",
]);
const actual = [];

const instance = new Temporal.PlainDate(2000, 5, 2, "iso8601");
// clear observable operations that occurred during the constructor call
actual.splice(0);

const fields = TemporalHelpers.propertyBagObserver(actual, {
  year: 1.7,
  month: 1.7,
  monthCode: "M01",
  day: 1.7,
}, "fields");

const options = TemporalHelpers.propertyBagObserver(actual, {
  overflow: "constrain",
  extra: "property",
}, "options");

instance.with(fields, options);
assert.compareArray(actual, expected, "order of operations");

actual.splice(0); // clear

assert.throws(TypeError, () => instance.with(fields, null));
assert.compareArray(actual, expectedOpsForPrimitiveOptions,
  "argument fields are read before TypeError is thrown for primitive options");

actual.splice(0); // clear
