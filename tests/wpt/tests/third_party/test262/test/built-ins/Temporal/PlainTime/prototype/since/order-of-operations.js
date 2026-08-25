// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.prototype.since
description: Properties on an object passed to since() are accessed in the correct order
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const expectedOpsForPrimitiveOptions = [
  // ToTemporalTime
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
  "get other.nanosecond",
  "get other.nanosecond.valueOf",
  "call other.nanosecond.valueOf",
  "get other.second",
  "get other.second.valueOf",
  "call other.second.valueOf",
];
const expected = expectedOpsForPrimitiveOptions.concat([
  // GetDifferenceSettings
  "get options.largestUnit",
  "get options.largestUnit.toString",
  "call options.largestUnit.toString",
  "get options.roundingIncrement",
  "get options.roundingIncrement.valueOf",
  "call options.roundingIncrement.valueOf",
  "get options.roundingMode",
  "get options.roundingMode.toString",
  "call options.roundingMode.toString",
  "get options.smallestUnit",
  "get options.smallestUnit.toString",
  "call options.smallestUnit.toString",
]);
const actual = [];

const instance = new Temporal.PlainTime(12, 34, 56, 987, 654, 321);

const other = TemporalHelpers.propertyBagObserver(actual, {
  hour: 1.7,
  minute: 1.7,
  second: 1.7,
  millisecond: 1.7,
  microsecond: 1.7,
  nanosecond: 1.7,
  calendar: "iso8601",
}, "other", ["calendar"]);

const options = TemporalHelpers.propertyBagObserver(actual, {
  roundingIncrement: 1,
  roundingMode: "trunc",
  largestUnit: "hours",
  smallestUnit: "nanoseconds",
  additional: true,
}, "options");

const result = instance.since(other, options);
assert.compareArray(actual, expected, "order of operations");

actual.splice(0); // clear

assert.throws(TypeError, () => instance.since(other, null));
assert.compareArray(actual, expectedOpsForPrimitiveOptions,
  "other time fields are read before TypeError is thrown for primitive options");

actual.splice(0); // clear
