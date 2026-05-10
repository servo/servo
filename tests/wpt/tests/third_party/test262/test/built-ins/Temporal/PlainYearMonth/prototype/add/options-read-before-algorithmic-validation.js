// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.add
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

const instance = new Temporal.PlainYearMonth(-271821, 4);

assert.throws(RangeError, function () {
  instance.add(new Temporal.Duration(0, 1), options);
}, "exception thrown when converting -271821-04 to date");
assert.compareArray(actual, expected, "all options should be read first");

actual.splice(0);  // clear

const instance2 = new Temporal.PlainYearMonth(1999, 12);

assert.throws(RangeError, function () {
  instance2.add(new Temporal.Duration(0, 0, 1), options);
}, "exception thrown when attempting to add too-low unit");
assert.compareArray(actual, expected, "all options should be read first");
