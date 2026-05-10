// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.total
description: >
  All options properties are read and cast before any algorithmic validation
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

const options = TemporalHelpers.propertyBagObserver(actual, {
  unit: "weeks",
  relativeTo: undefined,
}, "options");

const instance = new Temporal.Duration(1);

assert.throws(RangeError, function () {
  instance.total(options);
}, "exception thrown when total of calendar unit requested without relativeTo");
assert.compareArray(actual, expected, "all options should be read first");
