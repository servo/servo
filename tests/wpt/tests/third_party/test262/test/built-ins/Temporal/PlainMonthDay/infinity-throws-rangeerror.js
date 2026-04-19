// Copyright (C) 2020 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Temporal.PlainMonthDay throws a RangeError if any numerical value is Infinity
esid: sec-temporal.plainmonthday
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

assert.throws(RangeError, () => new Temporal.PlainMonthDay(Infinity, 1));
assert.throws(RangeError, () => new Temporal.PlainMonthDay(1, Infinity));
assert.throws(RangeError, () => new Temporal.PlainMonthDay(1, 1, "iso8601", Infinity));

const O = (primitiveValue, propertyName) => (calls) => TemporalHelpers.toPrimitiveObserver(calls, primitiveValue, propertyName);
const tests = [
  [
    "infinite month",
    [O(Infinity, "month"), O(1, "day"), () => "iso8601", O(1, "year")],
    ["get month.valueOf", "call month.valueOf"]
  ],
  [
    "infinite day",
    [O(2, "month"), O(Infinity, "day"), () => "iso8601", O(1, "year")],
    ["get month.valueOf", "call month.valueOf", "get day.valueOf", "call day.valueOf"]
  ],
  [
    "infinite year",
    [O(2, "month"), O(1, "day"), () => "iso8601", O(Infinity, "year")],
    ["get month.valueOf", "call month.valueOf", "get day.valueOf", "call day.valueOf", "get year.valueOf", "call year.valueOf"]
  ],
];

for (const [description, args, expected] of tests) {
  const actual = [];
  const args_ = args.map((o) => o(actual));
  assert.throws(RangeError, () => new Temporal.PlainMonthDay(...args_), description);
  assert.compareArray(actual, expected, `${description} order of operations`);
}
