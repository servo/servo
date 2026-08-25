// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.until
description: Properties on objects passed to until() are accessed in the correct order
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const expectedOpsForPrimitiveOptions = [
  // ToTemporalDate
  "get other.calendar",
  "get other.day",
  "get other.day.valueOf",
  "call other.day.valueOf",
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

const instance = new Temporal.PlainDate(2000, 5, 2, "iso8601");

const otherDatePropertyBag = TemporalHelpers.propertyBagObserver(actual, {
  year: 2001,
  month: 6,
  monthCode: "M06",
  day: 2,
  calendar: "iso8601",
}, "other", ["calendar"]);

function createOptionsObserver({ smallestUnit = "days", largestUnit = "auto", roundingMode = "halfExpand", roundingIncrement = 1 } = {}) {
  return TemporalHelpers.propertyBagObserver(actual, {
    // order is significant, due to iterating through properties in order to
    // copy them to an internal null-prototype object:
    roundingIncrement,
    roundingMode,
    largestUnit,
    smallestUnit,
    additional: "property",
  }, "options");
}

// basic order of observable operations with calendar call, without rounding:
instance.until(otherDatePropertyBag, createOptionsObserver({ largestUnit: "years" }));
assert.compareArray(actual, expected, "order of operations");
actual.splice(0); // clear

assert.throws(TypeError, () => instance.until(otherDatePropertyBag, null));
assert.compareArray(actual, expectedOpsForPrimitiveOptions,
  "other date fields are read before TypeError is thrown for primitive options");
actual.splice(0); // clear
