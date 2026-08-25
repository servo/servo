// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.tostring
description: >
  All options properties are read and cast before any algorithmic validation
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const expected = [
  "get options.fractionalSecondDigits",
  "get options.fractionalSecondDigits.toString",
  "call options.fractionalSecondDigits.toString",
  "get options.roundingMode",
  "get options.roundingMode.toString",
  "call options.roundingMode.toString",
  "get options.smallestUnit",
  "get options.smallestUnit.toString",
  "call options.smallestUnit.toString",
];
const actual = [];

const options = TemporalHelpers.propertyBagObserver(actual, {
  smallestUnit: "seconds",
  fractionalSecondDigits: "auto",
  roundingMode: "expand",
}, "options");

const instance = new Temporal.Duration(0, 0, 0, 0, 0, 0, /* seconds = */ Number.MAX_SAFE_INTEGER, 1);

assert.throws(RangeError, function () {
  instance.toString(options);
}, "exception thrown when result is out of range");
assert.compareArray(actual, expected, "all options should be read first");
