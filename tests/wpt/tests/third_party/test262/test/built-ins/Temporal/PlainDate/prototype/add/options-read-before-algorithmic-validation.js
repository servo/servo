// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.add
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
  overflow: "reject",
}, "options");

const instance = new Temporal.PlainDate(2025, 8, 31);

assert.throws(RangeError, function () {
  instance.add(new Temporal.Duration(0, 1), options);
}, "overflow reject exception thrown");
assert.compareArray(actual, expected, "all options should be read first");
