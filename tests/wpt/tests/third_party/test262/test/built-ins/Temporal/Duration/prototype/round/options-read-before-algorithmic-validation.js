// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.round
description: >
  All options properties are read and cast before any algorithmic validation
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const expected = [
  "get options.largestUnit",
  "get options.largestUnit.toString",
  "call options.largestUnit.toString",
  "get options.relativeTo",
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
  smallestUnit: "years",
  largestUnit: "nanoseconds",
  roundingIncrement: 1,
  roundingMode: "halfCeil",
  relativeTo: undefined,
}, "options");

const instance = new Temporal.Duration(1);

assert.throws(RangeError, function () {
  instance.round(options);
}, "exception thrown when smallestUnit > largestUnit");
assert.compareArray(actual, expected, "all options should be read first");
actual.splice(0);  // clear

// Test again, with largestUnit and smallestUnit undefined. The error that's
// thrown in that case is the earliest algorithmic validation that takes place,
// but we can't test it simultaneously with testing that largestUnit and
// smallestUnit are cast at the right time.

const expectedWithoutUnits = [
  "get options.largestUnit",
  "get options.relativeTo",
  "get options.roundingIncrement",
  "get options.roundingIncrement.valueOf",
  "call options.roundingIncrement.valueOf",
  "get options.roundingMode",
  "get options.roundingMode.toString",
  "call options.roundingMode.toString",
  "get options.smallestUnit",
];

const optionsWithoutUnits = TemporalHelpers.propertyBagObserver(actual, {
  smallestUnit: undefined,
  largestUnit: undefined,
  roundingIncrement: 1,
  roundingMode: "halfFloor",
  relativeTo: undefined,
}, "options");

assert.throws(RangeError, function () {
  instance.round(optionsWithoutUnits);
}, "exception thrown when neither smallestUnit nor largestUnit present");
assert.compareArray(actual, expectedWithoutUnits, "all options should be read first");
actual.splice(0);  // clear
