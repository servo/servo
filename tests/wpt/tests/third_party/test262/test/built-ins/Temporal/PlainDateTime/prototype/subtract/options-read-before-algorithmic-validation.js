// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.subtract
description: >
  All options properties are read and cast before any algorithmic validation
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const expected = [
  "get options.overflow",
  "get options.overflow.toString",
  "call options.overflow.toString",
];
const actual = [];

const options = TemporalHelpers.propertyBagObserver(actual, {
  overflow: "constrain",
}, "options");

const instance = new Temporal.PlainDateTime(2025, 8, 31, 1);

assert.throws(RangeError, function () {
  instance.subtract(new Temporal.Duration(0, 0, 0, /* days = */ 104249991374, 2), options);
}, "RangeError thrown when time addition overflows days component");
assert.compareArray(actual, expected, "all options should be read first");
