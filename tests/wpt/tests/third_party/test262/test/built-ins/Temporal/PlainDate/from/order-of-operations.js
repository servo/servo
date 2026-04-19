// Copyright (C) 2020 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.from
description: Properties on an object passed to from() are accessed in the correct order
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const expectedOptionsReading = [
  "get options.overflow",
  "get options.overflow.toString",
  "call options.overflow.toString",
];

const expectedOpsForPrimitiveOptions = [
  "get fields.calendar",
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
const expected = expectedOpsForPrimitiveOptions.concat(expectedOptionsReading);
const actual = [];

const fields = TemporalHelpers.propertyBagObserver(actual, {
  year: 1.7,
  month: 1.7,
  monthCode: "M01",
  day: 1.7,
  calendar: "iso8601",
}, "fields", ["calendar"]);

const options = TemporalHelpers.propertyBagObserver(actual, {
  overflow: "constrain",
  extra: "property",
}, "options");

const result = Temporal.PlainDate.from(fields, options);
assert.compareArray(actual, expected, "order of operations");

actual.splice(0);  // clear for next test

Temporal.PlainDate.from(new Temporal.PlainDate(2000, 5, 2), options);
assert.compareArray(actual, expectedOptionsReading, "order of operations when cloning a PlainDate instance");

actual.splice(0);

Temporal.PlainDate.from(new Temporal.PlainDateTime(2000, 5, 2), options);
assert.compareArray(actual, expectedOptionsReading, "order of operations when converting a PlainDateTime instance");

actual.splice(0);

Temporal.PlainDate.from(new Temporal.ZonedDateTime(0n, "UTC"), options);
assert.compareArray(actual, expectedOptionsReading, "order of operations when converting a ZonedDateTime instance");

actual.splice(0);

Temporal.PlainDate.from("2001-05-02", options);
assert.compareArray(actual, expectedOptionsReading, "order of operations when parsing a string");

actual.splice(0);

assert.throws(TypeError, () => Temporal.PlainDate.from(fields, null));
assert.compareArray(actual, expectedOpsForPrimitiveOptions,
  "item fields are read before TypeError is thrown for primitive options");

actual.splice(0); // clear
