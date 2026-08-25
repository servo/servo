// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.total
description: Properties on objects passed to total() are accessed in the correct order
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const expected = [
  "get options.relativeTo",
  "get options.unit",
  "get options.unit.toString",
  "call options.unit.toString",
];
const actual = [];

function createOptionsObserver({ unit = "nanoseconds", roundingMode = "halfExpand", roundingIncrement = 1, relativeTo = undefined } = {}) {
  return TemporalHelpers.propertyBagObserver(actual, {
    unit,
    roundingMode,
    roundingIncrement,
    relativeTo,
  }, "options");
}

const instance = new Temporal.Duration(0, 0, 0, 0, 2400);

// basic order of observable operations, with no relativeTo
instance.total(createOptionsObserver({ unit: "nanoseconds" }));
assert.compareArray(actual, expected, "order of operations");
actual.splice(0); // clear

// Check fast path for temporal objects.
function checkTemporalObject(object) {
  ["year", "month", "monthCode", "day", "hour", "minute", "second", "millisecond", "microsecond", "nanosecond"].forEach((property) => {
    Object.defineProperty(object, property, {
      get() {
        throw new Test262Error(`should not get ${property}`);
      }});
  });
}

// basic order of operations, with relativeTo a Temporal object
const pd = new Temporal.PlainDate(2026, 3, 6);
checkTemporalObject(pd);
instance.total(createOptionsObserver({ unit: "nanoseconds", relativeTo: pd }));
assert.compareArray(actual, expected,
  "relativeTo PlainDate should not read property bag fields");
actual.splice(0); // clear

const zdt = new Temporal.ZonedDateTime(1772751600000000000n, "UTC");
checkTemporalObject(zdt);
instance.total(createOptionsObserver({ unit: "nanoseconds", relativeTo: zdt }));
assert.compareArray(actual, expected,
  "relativeTo ZonedDateTime should not read property bag fields");
actual.splice(0); // clear


const expectedOpsForPlainRelativeTo = [
  // ToRelativeTemporalObject
  "get options.relativeTo",
  "get options.relativeTo.calendar",
  "get options.relativeTo.day",
  "get options.relativeTo.day.valueOf",
  "call options.relativeTo.day.valueOf",
  "get options.relativeTo.hour",
  "get options.relativeTo.microsecond",
  "get options.relativeTo.millisecond",
  "get options.relativeTo.minute",
  "get options.relativeTo.month",
  "get options.relativeTo.month.valueOf",
  "call options.relativeTo.month.valueOf",
  "get options.relativeTo.monthCode",
  "get options.relativeTo.monthCode.toString",
  "call options.relativeTo.monthCode.toString",
  "get options.relativeTo.nanosecond",
  "get options.relativeTo.offset",
  "get options.relativeTo.second",
  "get options.relativeTo.timeZone",
  "get options.relativeTo.year",
  "get options.relativeTo.year.valueOf",
  "call options.relativeTo.year.valueOf",
  // GetTemporalUnit
  "get options.unit",
  "get options.unit.toString",
  "call options.unit.toString",
];

const plainRelativeTo = TemporalHelpers.propertyBagObserver(actual, {
  year: 2001,
  month: 5,
  monthCode: "M05",
  day: 2,
  calendar: "iso8601",
}, "options.relativeTo", ["calendar"]);

// basic order of observable operations, without rounding:
instance.total(createOptionsObserver({ unit: "nanoseconds", relativeTo: plainRelativeTo }));
assert.compareArray(actual, expectedOpsForPlainRelativeTo, "order of operations for PlainDate relativeTo");
actual.splice(0); // clear

const expectedOpsForZonedRelativeTo = [
  // ToRelativeTemporalObject
  "get options.relativeTo",
  "get options.relativeTo.calendar",
  "get options.relativeTo.day",
  "get options.relativeTo.day.valueOf",
  "call options.relativeTo.day.valueOf",
  "get options.relativeTo.hour",
  "get options.relativeTo.hour.valueOf",
  "call options.relativeTo.hour.valueOf",
  "get options.relativeTo.microsecond",
  "get options.relativeTo.microsecond.valueOf",
  "call options.relativeTo.microsecond.valueOf",
  "get options.relativeTo.millisecond",
  "get options.relativeTo.millisecond.valueOf",
  "call options.relativeTo.millisecond.valueOf",
  "get options.relativeTo.minute",
  "get options.relativeTo.minute.valueOf",
  "call options.relativeTo.minute.valueOf",
  "get options.relativeTo.month",
  "get options.relativeTo.month.valueOf",
  "call options.relativeTo.month.valueOf",
  "get options.relativeTo.monthCode",
  "get options.relativeTo.monthCode.toString",
  "call options.relativeTo.monthCode.toString",
  "get options.relativeTo.nanosecond",
  "get options.relativeTo.nanosecond.valueOf",
  "call options.relativeTo.nanosecond.valueOf",
  "get options.relativeTo.offset",
  "get options.relativeTo.offset.toString",
  "call options.relativeTo.offset.toString",
  "get options.relativeTo.second",
  "get options.relativeTo.second.valueOf",
  "call options.relativeTo.second.valueOf",
  "get options.relativeTo.timeZone",
  "get options.relativeTo.year",
  "get options.relativeTo.year.valueOf",
  "call options.relativeTo.year.valueOf",
  // GetTemporalUnit
  "get options.unit",
  "get options.unit.toString",
  "call options.unit.toString",
];

const zonedRelativeTo = TemporalHelpers.propertyBagObserver(actual, {
  year: 2001,
  month: 5,
  monthCode: "M05",
  day: 2,
  hour: 6,
  minute: 54,
  second: 32,
  millisecond: 987,
  microsecond: 654,
  nanosecond: 321,
  offset: "+00:00",
  calendar: "iso8601",
  timeZone: "UTC",
}, "options.relativeTo", ["calendar", "timeZone"]);

// basic order of observable operations, without rounding:
instance.total(createOptionsObserver({ unit: "nanoseconds", relativeTo: zonedRelativeTo }));
assert.compareArray(actual, expectedOpsForZonedRelativeTo, "order of operations for ZonedDateTime relativeTo");
actual.splice(0); // clear
