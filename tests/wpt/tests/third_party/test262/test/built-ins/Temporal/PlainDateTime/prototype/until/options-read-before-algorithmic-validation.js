// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.until
description: >
  All options properties are read and cast before any algorithmic validation
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const expected = [
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
];
const actual = [];

const options = TemporalHelpers.propertyBagObserver(actual, {
  smallestUnit: "year",
  largestUnit: "nanosecond",
  roundingIncrement: 1,
  roundingMode: "halfFloor",
}, "options");

const instance = new Temporal.PlainDateTime(2025, 8, 14, 12);
const other = new Temporal.PlainDateTime(2025, 3, 14, 17);

assert.throws(RangeError, function () {
  instance.until(other, options);
}, "exception thrown when smallestUnit > largestUnit");
assert.compareArray(actual, expected, "all options should be read first");
