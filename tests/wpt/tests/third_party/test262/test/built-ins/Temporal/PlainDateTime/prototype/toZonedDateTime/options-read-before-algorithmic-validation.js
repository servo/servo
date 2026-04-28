// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.tozoneddatetime
description: >
  All options properties are read and cast before any algorithmic validation
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const expected = [
  "get options.disambiguation",
  "get options.disambiguation.toString",
  "call options.disambiguation.toString",
];
const actual = [];

const options = TemporalHelpers.propertyBagObserver(actual, {
  disambiguation: "reject",
}, "options");

const instance = new Temporal.PlainDateTime(-271821, 4, 20, 0);

assert.throws(RangeError, function () {
  instance.toZonedDateTime("+23:59", options);
}, "exception thrown when wall time out of range for exact time");
assert.compareArray(actual, expected, "all options should be read first");
