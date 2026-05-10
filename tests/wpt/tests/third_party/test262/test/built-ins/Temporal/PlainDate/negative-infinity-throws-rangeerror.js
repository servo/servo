// Copyright (C) 2020 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Temporal.PlainDate throws a RangeError if any value is -Infinity
esid: sec-temporal.plaindate
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

assert.throws(RangeError, () => new Temporal.PlainDate(-Infinity, 1, 1));
assert.throws(RangeError, () => new Temporal.PlainDate(1970, -Infinity, 1));
assert.throws(RangeError, () => new Temporal.PlainDate(1970, 1, -Infinity));

const O = (primitiveValue, propertyName) => (calls) => TemporalHelpers.toPrimitiveObserver(calls, primitiveValue, propertyName);
const tests = [
  [
    "infinite year",
    [O(-Infinity, "year"), O(1, "month"), O(1, "day")],
    ["get year.valueOf", "call year.valueOf"]
  ],
  [
    "infinite month",
    [O(2, "year"), O(-Infinity, "month"), O(1, "day")],
    ["get year.valueOf", "call year.valueOf", "get month.valueOf", "call month.valueOf"]
  ],
  [
    "infinite day",
    [O(2, "year"), O(1, "month"), O(-Infinity, "day")],
    ["get year.valueOf", "call year.valueOf", "get month.valueOf", "call month.valueOf", "get day.valueOf", "call day.valueOf"]
  ],
];

for (const [description, args, expected] of tests) {
  const actual = [];
  const args_ = args.map((o) => o(actual));
  assert.throws(RangeError, () => new Temporal.PlainDate(...args_), description);
  assert.compareArray(actual, expected, `${description} order of operations`);
}
