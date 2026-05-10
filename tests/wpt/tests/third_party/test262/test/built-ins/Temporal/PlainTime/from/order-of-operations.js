// Copyright (C) 2020 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.from
description: Properties on an object passed to from() are accessed in the correct order
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const expectedOptionsReading = [
  // GetTemporalOverflowOption
  "get options.overflow",
  "get options.overflow.toString",
  "call options.overflow.toString",
];

const expectedOpsForPrimitiveOptions = [
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
const expected = expectedOpsForPrimitiveOptions.concat(expectedOptionsReading);
const actual = [];

const fields = TemporalHelpers.propertyBagObserver(actual, {
  hour: 1.7,
  minute: 1.7,
  second: 1.7,
  millisecond: 1.7,
  microsecond: 1.7,
  nanosecond: 1.7,
  calendar: "iso8601",
}, "fields");

const options = TemporalHelpers.propertyBagObserver(actual, {
  overflow: "constrain",
}, "options");

const result = Temporal.PlainTime.from(fields, options);
assert.compareArray(actual, expected, "order of operations");

actual.splice(0);  // clear for next test

Temporal.PlainTime.from(new Temporal.PlainTime(12, 34), options);
assert.compareArray(actual, expectedOptionsReading, "order of operations when cloning a PlainTime instance");

actual.splice(0);

Temporal.PlainTime.from(new Temporal.PlainDateTime(2000, 5, 2), options);
assert.compareArray(actual, expectedOptionsReading, "order of operations when converting a PlainDateTime instance");

actual.splice(0);

Temporal.PlainTime.from(new Temporal.ZonedDateTime(0n, "UTC"), options);
assert.compareArray(actual, expectedOptionsReading, "order of operations when converting a ZonedDateTime instance");

actual.splice(0);

Temporal.PlainTime.from("12:34", options);
assert.compareArray(actual, expectedOptionsReading, "order of operations when parsing a string");

actual.splice(0);

assert.throws(TypeError, () => Temporal.PlainTime.from(fields, null));
assert.compareArray(actual, expectedOpsForPrimitiveOptions,
  "item fields are read before TypeError is thrown for primitive options");

actual.splice(0); // clear
