// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.from
description: Basic tests for PlainMonthDay.from(PlainMonthDay).
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const expected = [
  "get overflow",
  "get overflow.toString",
  "call overflow.toString",
];
const actual = [];
const options = {
  get overflow() {
    actual.push("get overflow");
    return TemporalHelpers.toPrimitiveObserver(actual, "reject", "overflow");
  }
};

const fields = new Temporal.PlainMonthDay(11, 16, undefined, 1960);
const result = Temporal.PlainMonthDay.from(fields, options);
TemporalHelpers.assertPlainMonthDay(result, "M11", 16, "should copy reference year", 1960);
assert.compareArray(actual, expected, "Should get overflow");
