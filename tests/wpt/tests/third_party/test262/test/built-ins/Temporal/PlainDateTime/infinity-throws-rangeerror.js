// Copyright (C) 2020 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Temporal.PlainDateTime throws a RangeError if any value is Infinity
esid: sec-temporal.plaindatetime
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

assert.throws(RangeError, () => new Temporal.PlainDateTime(Infinity, 1, 1));
assert.throws(RangeError, () => new Temporal.PlainDateTime(1970, Infinity, 1));
assert.throws(RangeError, () => new Temporal.PlainDateTime(1970, 1, Infinity));
assert.throws(RangeError, () => new Temporal.PlainDateTime(1970, 1, 1, Infinity));
assert.throws(RangeError, () => new Temporal.PlainDateTime(1970, 1, 1, 0, Infinity));
assert.throws(RangeError, () => new Temporal.PlainDateTime(1970, 1, 1, 0, 0, Infinity));
assert.throws(RangeError, () => new Temporal.PlainDateTime(1970, 1, 1, 0, 0, 0, Infinity));
assert.throws(RangeError, () => new Temporal.PlainDateTime(1970, 1, 1, 0, 0, 0, 0, Infinity));
assert.throws(RangeError, () => new Temporal.PlainDateTime(1970, 1, 1, 0, 0, 0, 0, 0, Infinity));

const O = (primitiveValue, propertyName) => (calls) => TemporalHelpers.toPrimitiveObserver(calls, primitiveValue, propertyName);
const tests = [
  [
    "infinite year",
    [O(Infinity, "year"), O(1, "month"), O(1, "day")],
    ["get year.valueOf", "call year.valueOf"
    ]
  ],
  [
    "infinite month",
    [O(2, "year"), O(Infinity, "month"), O(1, "day")],
    ["get year.valueOf", "call year.valueOf", "get month.valueOf", "call month.valueOf"
    ]
  ],
  [
    "infinite day",
    [O(2, "year"), O(1, "month"), O(Infinity, "day")],
    ["get year.valueOf", "call year.valueOf", "get month.valueOf", "call month.valueOf", "get day.valueOf", "call day.valueOf"
    ]
  ],
  [
    "infinite hour",
    [O(2, "year"), O(1, "month"), O(1, "day"), O(Infinity, "hour"), O(1, "minute"), O(1, "second"), O(1, "millisecond"), O(1, "microsecond"), O(1, "nanosecond")],
    [
      "get year.valueOf",
      "call year.valueOf",
      "get month.valueOf",
      "call month.valueOf",
      "get day.valueOf",
      "call day.valueOf",
      "get hour.valueOf",
      "call hour.valueOf"
    ]
  ],
  [
    "infinite minute",
    [O(2, "year"), O(1, "month"), O(1, "day"), O(1, "hour"), O(Infinity, "minute"), O(1, "second"), O(1, "millisecond"), O(1, "microsecond"), O(1, "nanosecond")],
    [
      "get year.valueOf",
      "call year.valueOf",
      "get month.valueOf",
      "call month.valueOf",
      "get day.valueOf",
      "call day.valueOf",
      "get hour.valueOf",
      "call hour.valueOf",
      "get minute.valueOf",
      "call minute.valueOf"
    ]
  ],
  [
    "infinite second",
    [O(2, "year"), O(1, "month"), O(1, "day"), O(1, "hour"), O(1, "minute"), O(Infinity, "second"), O(1, "millisecond"), O(1, "microsecond"), O(1, "nanosecond")],
    [
      "get year.valueOf",
      "call year.valueOf",
      "get month.valueOf",
      "call month.valueOf",
      "get day.valueOf",
      "call day.valueOf",
      "get hour.valueOf",
      "call hour.valueOf",
      "get minute.valueOf",
      "call minute.valueOf",
      "get second.valueOf",
      "call second.valueOf"
    ]
  ],
  [
    "infinite millisecond",
    [O(2, "year"), O(1, "month"), O(1, "day"), O(1, "hour"), O(1, "minute"), O(1, "second"), O(Infinity, "millisecond"), O(1, "microsecond"), O(1, "nanosecond")],
    [
      "get year.valueOf",
      "call year.valueOf",
      "get month.valueOf",
      "call month.valueOf",
      "get day.valueOf",
      "call day.valueOf",
      "get hour.valueOf",
      "call hour.valueOf",
      "get minute.valueOf",
      "call minute.valueOf",
      "get second.valueOf",
      "call second.valueOf",
      "get millisecond.valueOf",
      "call millisecond.valueOf"
    ]
  ],
  [
    "infinite microsecond",
    [O(2, "year"), O(1, "month"), O(1, "day"), O(1, "hour"), O(1, "minute"), O(1, "second"), O(1, "millisecond"), O(Infinity, "microsecond"), O(1, "nanosecond")],
    [
      "get year.valueOf",
      "call year.valueOf",
      "get month.valueOf",
      "call month.valueOf",
      "get day.valueOf",
      "call day.valueOf",
      "get hour.valueOf",
      "call hour.valueOf",
      "get minute.valueOf",
      "call minute.valueOf",
      "get second.valueOf",
      "call second.valueOf",
      "get millisecond.valueOf",
      "call millisecond.valueOf",
      "get microsecond.valueOf",
      "call microsecond.valueOf"
    ]
  ],
  [
    "infinite nanosecond",
    [O(2, "year"), O(1, "month"), O(1, "day"), O(1, "hour"), O(1, "minute"), O(1, "second"), O(1, "millisecond"), O(1, "microsecond"), O(Infinity, "nanosecond")],
    [
      "get year.valueOf",
      "call year.valueOf",
      "get month.valueOf",
      "call month.valueOf",
      "get day.valueOf",
      "call day.valueOf",
      "get hour.valueOf",
      "call hour.valueOf",
      "get minute.valueOf",
      "call minute.valueOf",
      "get second.valueOf",
      "call second.valueOf",
      "get millisecond.valueOf",
      "call millisecond.valueOf",
      "get microsecond.valueOf",
      "call microsecond.valueOf",
      "get nanosecond.valueOf",
      "call nanosecond.valueOf"
    ]
  ],
];

for (const [description, args, expected] of tests) {
  const actual = [];
  const args_ = args.map((o) => o(actual));
  assert.throws(RangeError, () => new Temporal.PlainDateTime(...args_), description);
  assert.compareArray(actual, expected, `${description} order of operations`);
}
