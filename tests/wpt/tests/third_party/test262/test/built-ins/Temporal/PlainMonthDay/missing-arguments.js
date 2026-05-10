// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday
description: RangeError thrown after processing given args when invoked without all required args
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const expected = [
  "get month.valueOf",
  "call month.valueOf",
];
const actual = [];
const args = [
  TemporalHelpers.toPrimitiveObserver(actual, 1, "month"),
];

assert.throws(RangeError, () => new Temporal.PlainMonthDay(...args));
assert.compareArray(actual, expected, "order of operations");

assert.throws(RangeError, () => new Temporal.PlainMonthDay(), "no arguments");
