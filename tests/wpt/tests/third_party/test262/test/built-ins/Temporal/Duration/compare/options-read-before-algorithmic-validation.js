// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.compare
description: All options properties are read before any algorithmic validation
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const expected = ["get options.relativeTo"];
const actual = [];

const options = TemporalHelpers.propertyBagObserver(actual, { relativeTo: undefined }, "options");

const d1 = new Temporal.Duration(0, 0, 0, /* days = */ 1);
const d2 = new Temporal.Duration(1);

assert.throws(RangeError, function () {
  Temporal.Duration.compare(d1, d2, options);
}, "exception thrown when calendar units provided without relativeTo");
assert.compareArray(actual, expected, "all options should be read first");
