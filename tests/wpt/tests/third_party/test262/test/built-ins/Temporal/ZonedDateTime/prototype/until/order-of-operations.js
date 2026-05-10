// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.until
description: Properties on objects passed to until() are accessed in the correct order
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const expectedOpsForPrimitiveOptions = [
  // ToTemporalZonedDateTime
  "get other.calendar",
  "get other.day",
  "get other.day.valueOf",
  "call other.day.valueOf",
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
  "get other.month",
  "get other.month.valueOf",
  "call other.month.valueOf",
  "get other.monthCode",
  "get other.monthCode.toString",
  "call other.monthCode.toString",
  "get other.nanosecond",
  "get other.nanosecond.valueOf",
  "call other.nanosecond.valueOf",
  "get other.offset",
  "get other.offset.toString",
  "call other.offset.toString",
  "get other.second",
  "get other.second.valueOf",
  "call other.second.valueOf",
  "get other.timeZone",
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

const instance = new Temporal.ZonedDateTime(1_000_000_000_000_000_000n, "UTC");

const otherDateTimePropertyBag = TemporalHelpers.propertyBagObserver(actual, {
  year: 2004,
  month: 5,
  monthCode: "M05",
  day: 12,
  hour: 1,
  minute: 46,
  second: 40,
  millisecond: 250,
  microsecond: 500,
  nanosecond: 750,
  offset: "+00:00",
  calendar: "iso8601",
  timeZone: "UTC",
}, "other", ["calendar", "timeZone"]);

function createOptionsObserver({ smallestUnit = "nanoseconds", largestUnit = "auto", roundingMode = "halfExpand", roundingIncrement = 1 } = {}) {
  return TemporalHelpers.propertyBagObserver(actual, {
    roundingIncrement,
    roundingMode,
    largestUnit,
    smallestUnit,
    additional: "property",
  }, "options");
}

// basic order of observable operations, without rounding:
instance.until(otherDateTimePropertyBag, createOptionsObserver());
assert.compareArray(actual, expected, "order of operations");
actual.splice(0); // clear

assert.throws(TypeError, () => instance.until(otherDateTimePropertyBag, null));
assert.compareArray(actual, expectedOpsForPrimitiveOptions,
  "other zoned datetime fields are read before TypeError is thrown for primitive options");
actual.splice(0); // clear
