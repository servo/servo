// Copyright (C) 2020 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Temporal.PlainDate throws a RangeError if any value is -Infinity
esid: sec-temporal.plaintime
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

assert.throws(RangeError, () => new Temporal.PlainTime(-Infinity));
assert.throws(RangeError, () => new Temporal.PlainTime(0, -Infinity));
assert.throws(RangeError, () => new Temporal.PlainTime(0, 0, -Infinity));
assert.throws(RangeError, () => new Temporal.PlainTime(0, 0, 0, -Infinity));
assert.throws(RangeError, () => new Temporal.PlainTime(0, 0, 0, 0, -Infinity));
assert.throws(RangeError, () => new Temporal.PlainTime(0, 0, 0, 0, 0, -Infinity));

const O = (primitiveValue, propertyName) => (calls) => TemporalHelpers.toPrimitiveObserver(calls, primitiveValue, propertyName);
const tests = [
  [
    "infinite hour",
    [O(-Infinity, "hour"), O(1, "minute"), O(1, "second"), O(1, "millisecond"), O(1, "microsecond"), O(1, "nanosecond")],
    ["get hour.valueOf", "call hour.valueOf"]
  ],
  [
    "infinite minute",
    [O(1, "hour"), O(-Infinity, "minute"), O(1, "second"), O(1, "millisecond"), O(1, "microsecond"), O(1, "nanosecond")],
    ["get hour.valueOf", "call hour.valueOf", "get minute.valueOf", "call minute.valueOf"]
  ],
  [
    "infinite second",
    [O(1, "hour"), O(1, "minute"), O(-Infinity, "second"), O(1, "millisecond"), O(1, "microsecond"), O(1, "nanosecond")],
    ["get hour.valueOf", "call hour.valueOf", "get minute.valueOf", "call minute.valueOf", "get second.valueOf", "call second.valueOf"]
  ],
  [
    "infinite millisecond",
    [O(1, "hour"), O(1, "minute"), O(1, "second"), O(-Infinity, "millisecond"), O(1, "microsecond"), O(1, "nanosecond")],
    ["get hour.valueOf", "call hour.valueOf", "get minute.valueOf", "call minute.valueOf", "get second.valueOf", "call second.valueOf", "get millisecond.valueOf", "call millisecond.valueOf"]
  ],
  [
    "infinite microsecond",
    [O(1, "hour"), O(1, "minute"), O(1, "second"), O(1, "millisecond"), O(-Infinity, "microsecond"), O(1, "nanosecond")],
    ["get hour.valueOf", "call hour.valueOf", "get minute.valueOf", "call minute.valueOf", "get second.valueOf", "call second.valueOf", "get millisecond.valueOf", "call millisecond.valueOf", "get microsecond.valueOf", "call microsecond.valueOf"]
  ],
  [
    "infinite nanosecond",
    [O(1, "hour"), O(1, "minute"), O(1, "second"), O(1, "millisecond"), O(1, "microsecond"), O(-Infinity, "nanosecond")],
    ["get hour.valueOf", "call hour.valueOf", "get minute.valueOf", "call minute.valueOf", "get second.valueOf", "call second.valueOf", "get millisecond.valueOf", "call millisecond.valueOf", "get microsecond.valueOf", "call microsecond.valueOf", "get nanosecond.valueOf", "call nanosecond.valueOf"]
  ],
];

for (const [description, args, expected] of tests) {
  const actual = [];
  const args_ = args.map((o) => o(actual));
  assert.throws(RangeError, () => new Temporal.PlainTime(...args_), description);
  assert.compareArray(actual, expected, `${description} order of operations`);
}
