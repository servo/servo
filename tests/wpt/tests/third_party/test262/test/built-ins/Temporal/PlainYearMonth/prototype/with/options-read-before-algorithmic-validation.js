// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.with
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

const instance = new Temporal.PlainYearMonth(2025, 7);

assert.throws(RangeError, function () {
  instance.with({ monthCode: "M08L" }, options);
}, "exception thrown when month code incorrect for calendar");
assert.compareArray(actual, expected, "all options should be read first");
