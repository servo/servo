// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.until
description: Properties on objects passed to until() are accessed in the correct order
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const expectedOpsForPrimitiveOptions = [
  // ToTemporalYearMonth
  "get other.calendar",
  "get other.month",
  "get other.month.valueOf",
  "call other.month.valueOf",
  "get other.monthCode",
  "get other.monthCode.toString",
  "call other.monthCode.toString",
  "get other.year",
  "get other.year.valueOf",
  "call other.year.valueOf",
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

const instance = new Temporal.PlainYearMonth(2000, 5, "iso8601", 1);

const otherYearMonthPropertyBag = TemporalHelpers.propertyBagObserver(actual, {
  year: 2001,
  month: 6,
  monthCode: "M06",
  calendar: "iso8601"
}, "other", ["calendar"]);

function createOptionsObserver({ smallestUnit = "months", largestUnit = "auto", roundingMode = "halfExpand", roundingIncrement = 1 } = {}) {
  return TemporalHelpers.propertyBagObserver(actual, {
    roundingIncrement,
    roundingMode,
    largestUnit,
    smallestUnit,
    additional: "property",
  }, "options");
}

instance.until(otherYearMonthPropertyBag, createOptionsObserver({ smallestUnit: "months", roundingIncrement: 1 }));
assert.compareArray(actual, expected, "order of operations");
actual.splice(0); // clear

assert.throws(TypeError, () => instance.until(otherYearMonthPropertyBag, null));
assert.compareArray(actual, expectedOpsForPrimitiveOptions,
  "other year-month fields are read before TypeError is thrown for primitive options");
actual.splice(0); // clear
