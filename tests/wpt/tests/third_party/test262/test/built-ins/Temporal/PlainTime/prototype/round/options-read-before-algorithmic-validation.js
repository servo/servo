// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.prototype.round
description: >
  All options properties are read and cast before any algorithmic validation
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const expected = [
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
  smallestUnit: "hour",
  roundingIncrement: 25,
  roundingMode: "expand",
}, "options");

const instance = new Temporal.PlainTime(12);

assert.throws(RangeError, function () {
  instance.round(options);
}, "exception thrown when roundingIncrement invalid for smallestUnit");
assert.compareArray(actual, expected, "all options should be read first");
