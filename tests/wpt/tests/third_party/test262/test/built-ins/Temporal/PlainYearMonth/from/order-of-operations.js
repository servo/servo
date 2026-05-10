// Copyright (C) 2020 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.from
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
  // GetTemporalCalendarSlotValueWithISODefault
  "get fields.calendar",
  // PrepareTemporalFields
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
  calendar: "iso8601",
}, "fields", ["calendar"]);

const options = TemporalHelpers.propertyBagObserver(actual, {
  overflow: "constrain",
  extra: "property",
}, "options");

Temporal.PlainYearMonth.from(fields, options);
assert.compareArray(actual, expected, "order of operations");

actual.splice(0);  // clear for next test

Temporal.PlainYearMonth.from(new Temporal.PlainYearMonth(2000, 5), options);
assert.compareArray(actual, expectedOptionsReading, "order of operations when cloning a PlainYearMonth instance");

actual.splice(0);

Temporal.PlainYearMonth.from("2000-05", options);
assert.compareArray(actual, expectedOptionsReading, "order of operations when parsing a string");

actual.splice(0);

assert.throws(TypeError, () => Temporal.PlainYearMonth.from(fields, null));
assert.compareArray(actual, expectedOpsForPrimitiveOptions,
  "item fields are read before TypeError is thrown for primitive options");

actual.splice(0); // clear
