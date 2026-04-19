// Copyright (C) 2020 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.prototype.with
description: Properties on an object passed to with() are accessed in the correct order
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const instance = new Temporal.PlainTime(12, 34, 56, 987, 654, 321);
const expectedOpsForPrimitiveOptions = [
  // RejectObjectWithCalendarOrTimeZone
  "get fields.calendar",
  "get fields.timeZone",
  // ToTemporalTimeRecord
  "get fields.hour",
  "get fields.hour.valueOf",
  "call fields.hour.valueOf",
  "get fields.microsecond",
  "get fields.microsecond.valueOf",
  "call fields.microsecond.valueOf",
  "get fields.millisecond",
  "get fields.millisecond.valueOf",
  "call fields.millisecond.valueOf",
  "get fields.minute",
  "get fields.minute.valueOf",
  "call fields.minute.valueOf",
  "get fields.nanosecond",
  "get fields.nanosecond.valueOf",
  "call fields.nanosecond.valueOf",
  "get fields.second",
  "get fields.second.valueOf",
  "call fields.second.valueOf",
];
const expected = expectedOpsForPrimitiveOptions.concat([
  // GetTemporalOverflowOption
  "get options.overflow",
  "get options.overflow.toString",
  "call options.overflow.toString",
]);
const actual = [];

const fields = TemporalHelpers.propertyBagObserver(actual, {
  hour: 1.7,
  minute: 1.7,
  second: 1.7,
  millisecond: 1.7,
  microsecond: 1.7,
  nanosecond: 1.7,
}, "fields");

const options = TemporalHelpers.propertyBagObserver(actual, {
  overflow: "constrain",
}, "options");

const result = instance.with(fields, options);
assert.compareArray(actual, expected, "order of operations");

actual.splice(0); // clear

assert.throws(TypeError, () => instance.with(fields, null));
assert.compareArray(actual, expectedOpsForPrimitiveOptions,
  "argument fields are read before TypeError is thrown for primitive options");

actual.splice(0); // clear
