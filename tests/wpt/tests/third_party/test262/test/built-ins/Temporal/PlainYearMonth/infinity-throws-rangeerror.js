// Copyright (C) 2020 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Temporal.PlainYearMonth throws a RangeError if any numerical value is Infinity
esid: sec-temporal.plainyearmonth
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

assert.throws(RangeError, () => new Temporal.PlainYearMonth(Infinity, 1));
assert.throws(RangeError, () => new Temporal.PlainYearMonth(1970, Infinity));
assert.throws(RangeError, () => new Temporal.PlainYearMonth(1970, 1, "iso8601", Infinity));

const O = (primitiveValue, propertyName) => (calls) => TemporalHelpers.toPrimitiveObserver(calls, primitiveValue, propertyName);
const tests = [
  [
    "infinite year",
    [O(Infinity, "year"), O(1, "month"), () => "iso8601", O(1, "day")],
    ["get year.valueOf", "call year.valueOf"]
  ],
  [
    "infinite month",
    [O(1970, "year"), O(Infinity, "month"), () => "iso8601", O(1, "day")],
    ["get year.valueOf", "call year.valueOf", "get month.valueOf", "call month.valueOf"]
  ],
  [
    "infinite day",
    [O(1970, "year"), O(1, "month"), () => "iso8601", O(Infinity, "day")],
    ["get year.valueOf", "call year.valueOf", "get month.valueOf", "call month.valueOf", "get day.valueOf", "call day.valueOf"]
  ],
];

for (const [description, args, expected] of tests) {
  const actual = [];
  const args_ = args.map((o) => o(actual));
  assert.throws(RangeError, () => new Temporal.PlainYearMonth(...args_), description);
  assert.compareArray(actual, expected, `${description} order of operations`);
}
