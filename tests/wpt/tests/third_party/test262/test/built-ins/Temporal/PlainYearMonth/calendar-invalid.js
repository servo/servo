// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Temporal.PlainYearMonth throws a RangeError if the calendar argument is invalid
esid: sec-temporal.plainyearmonth
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const expected = ["get year.valueOf", "call year.valueOf", "get month.valueOf", "call month.valueOf"];
const actual = [];
const args = [
  TemporalHelpers.toPrimitiveObserver(actual, 1970, "year"),
  TemporalHelpers.toPrimitiveObserver(actual, 1, "month"),
  "local",
  TemporalHelpers.toPrimitiveObserver(actual, 1, "day")
];
assert.throws(RangeError, () => new Temporal.PlainYearMonth(...args));
assert.compareArray(actual, expected, "order of operations");
