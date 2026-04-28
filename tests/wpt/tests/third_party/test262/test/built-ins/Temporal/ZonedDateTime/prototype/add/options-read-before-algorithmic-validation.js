// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.add
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

// 1970-01-31T00:00[UTC]
const instance = new Temporal.ZonedDateTime(2592000_000_000_000n, "UTC");

assert.throws(RangeError, function () {
  instance.add(new Temporal.Duration(0, 1), options);
}, "overflow reject exception thrown");
assert.compareArray(actual, expected, "all options should be read first");
actual.splice(0);  // clear

// Try again, but this time test that the options are read before the first
// possible throw completion in the time-units-only path

assert.throws(RangeError, function () {
  instance.add(new Temporal.Duration(0, 0, 0, 0, 0, 0, Number.MAX_SAFE_INTEGER), options);
}, "exception thrown when resulting exact time out of range");
assert.compareArray(actual, expected, "all options should be read first");
