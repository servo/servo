// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Temporal.PlainMonthDay throws a RangeError if the calendar argument is invalid
esid: sec-temporal.plainmonthday
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const expected = ["get month.valueOf", "call month.valueOf", "get day.valueOf", "call day.valueOf"];
const actual = [];
const args = [
  TemporalHelpers.toPrimitiveObserver(actual, 2, "month"),
  TemporalHelpers.toPrimitiveObserver(actual, 1, "day"),
  "local",
  TemporalHelpers.toPrimitiveObserver(actual, 1, "year")
];
assert.throws(RangeError, () => new Temporal.PlainMonthDay(...args));
assert.compareArray(actual, expected, "order of operations");
