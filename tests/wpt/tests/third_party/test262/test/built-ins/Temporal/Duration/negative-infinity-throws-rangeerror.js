// Copyright (C) 2020 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Temporal.Duration throws a RangeError if any value is -Infinity
esid: sec-temporal.duration
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

assert.throws(RangeError, () => new Temporal.Duration(-Infinity));
assert.throws(RangeError, () => new Temporal.Duration(0, -Infinity));
assert.throws(RangeError, () => new Temporal.Duration(0, 0, -Infinity));
assert.throws(RangeError, () => new Temporal.Duration(0, 0, 0, -Infinity));
assert.throws(RangeError, () => new Temporal.Duration(0, 0, 0, 0, -Infinity));
assert.throws(RangeError, () => new Temporal.Duration(0, 0, 0, 0, 0, -Infinity));
assert.throws(RangeError, () => new Temporal.Duration(0, 0, 0, 0, 0, 0, -Infinity));
assert.throws(RangeError, () => new Temporal.Duration(0, 0, 0, 0, 0, 0, 0, -Infinity));
assert.throws(RangeError, () => new Temporal.Duration(0, 0, 0, 0, 0, 0, 0, 0, -Infinity));
assert.throws(RangeError, () => new Temporal.Duration(0, 0, 0, 0, 0, 0, 0, 0, 0, -Infinity));

const O = (primitiveValue, propertyName) => (calls) => TemporalHelpers.toPrimitiveObserver(calls, primitiveValue, propertyName);
const tests = [
  [
    "infinite years",
    [O(-Infinity, "years"), O(0, "months"), O(0, "weeks"), O(0, "days"), O(0, "hours"), O(0, "minutes"), O(0, "seconds"), O(0, "milliseconds"), O(0, "microseconds"), O(0, "nanoseconds")],
    ["get years.valueOf", "call years.valueOf"]
  ],
  [
    "infinite months",
    [O(0, "years"), O(-Infinity, "months"), O(0, "weeks"), O(0, "days"), O(0, "hours"), O(0, "minutes"), O(0, "seconds"), O(0, "milliseconds"), O(0, "microseconds"), O(0, "nanoseconds")],
    ["get years.valueOf", "call years.valueOf", "get months.valueOf", "call months.valueOf"]
  ],
  [
    "infinite weeks",
    [O(0, "years"), O(0, "months"), O(-Infinity, "weeks"), O(0, "days"), O(0, "hours"), O(0, "minutes"), O(0, "seconds"), O(0, "milliseconds"), O(0, "microseconds"), O(0, "nanoseconds")],
    ["get years.valueOf", "call years.valueOf", "get months.valueOf", "call months.valueOf", "get weeks.valueOf", "call weeks.valueOf"]
  ],
  [
    "infinite days",
    [O(0, "years"), O(0, "months"), O(0, "weeks"), O(-Infinity, "days"), O(0, "hours"), O(0, "minutes"), O(0, "seconds"), O(0, "milliseconds"), O(0, "microseconds"), O(0, "nanoseconds")],
    ["get years.valueOf", "call years.valueOf", "get months.valueOf", "call months.valueOf", "get weeks.valueOf", "call weeks.valueOf", "get days.valueOf", "call days.valueOf"]
  ],
  [
    "infinite hours",
    [O(0, "years"), O(0, "months"), O(0, "weeks"), O(0, "days"), O(-Infinity, "hours"), O(0, "minutes"), O(0, "seconds"), O(0, "milliseconds"), O(0, "microseconds"), O(0, "nanoseconds")],
    ["get years.valueOf", "call years.valueOf", "get months.valueOf", "call months.valueOf", "get weeks.valueOf", "call weeks.valueOf", "get days.valueOf", "call days.valueOf", "get hours.valueOf", "call hours.valueOf"]
  ],
  [
    "infinite minutes",
    [O(0, "years"), O(0, "months"), O(0, "weeks"), O(0, "days"), O(0, "hours"), O(-Infinity, "minutes"), O(0, "seconds"), O(0, "milliseconds"), O(0, "microseconds"), O(0, "nanoseconds")],
    ["get years.valueOf", "call years.valueOf", "get months.valueOf", "call months.valueOf", "get weeks.valueOf", "call weeks.valueOf", "get days.valueOf", "call days.valueOf", "get hours.valueOf", "call hours.valueOf", "get minutes.valueOf", "call minutes.valueOf"]
  ],
  [
    "infinite seconds",
    [O(0, "years"), O(0, "months"), O(0, "weeks"), O(0, "days"), O(0, "hours"), O(0, "minutes"), O(-Infinity, "seconds"), O(0, "milliseconds"), O(0, "microseconds"), O(0, "nanoseconds")],
    ["get years.valueOf", "call years.valueOf", "get months.valueOf", "call months.valueOf", "get weeks.valueOf", "call weeks.valueOf", "get days.valueOf", "call days.valueOf", "get hours.valueOf", "call hours.valueOf", "get minutes.valueOf", "call minutes.valueOf", "get seconds.valueOf", "call seconds.valueOf"]
  ],
  [
    "infinite milliseconds",
    [O(0, "years"), O(0, "months"), O(0, "weeks"), O(0, "days"), O(0, "hours"), O(0, "minutes"), O(0, "seconds"), O(-Infinity, "milliseconds"), O(0, "microseconds"), O(0, "nanoseconds")],
    ["get years.valueOf", "call years.valueOf", "get months.valueOf", "call months.valueOf", "get weeks.valueOf", "call weeks.valueOf", "get days.valueOf", "call days.valueOf", "get hours.valueOf", "call hours.valueOf", "get minutes.valueOf", "call minutes.valueOf", "get seconds.valueOf", "call seconds.valueOf", "get milliseconds.valueOf", "call milliseconds.valueOf"]
  ],
  [
    "infinite microseconds",
    [O(0, "years"), O(0, "months"), O(0, "weeks"), O(0, "days"), O(0, "hours"), O(0, "minutes"), O(0, "seconds"), O(0, "milliseconds"), O(-Infinity, "microseconds"), O(0, "nanoseconds")],
    ["get years.valueOf", "call years.valueOf", "get months.valueOf", "call months.valueOf", "get weeks.valueOf", "call weeks.valueOf", "get days.valueOf", "call days.valueOf", "get hours.valueOf", "call hours.valueOf", "get minutes.valueOf", "call minutes.valueOf", "get seconds.valueOf", "call seconds.valueOf", "get milliseconds.valueOf", "call milliseconds.valueOf", "get microseconds.valueOf", "call microseconds.valueOf"]
  ],
  [
    "infinite nanoseconds",
    [O(0, "years"), O(0, "months"), O(0, "weeks"), O(0, "days"), O(0, "hours"), O(0, "minutes"), O(0, "seconds"), O(0, "milliseconds"), O(0, "microseconds"), O(-Infinity, "nanoseconds")],
    ["get years.valueOf", "call years.valueOf", "get months.valueOf", "call months.valueOf", "get weeks.valueOf", "call weeks.valueOf", "get days.valueOf", "call days.valueOf", "get hours.valueOf", "call hours.valueOf", "get minutes.valueOf", "call minutes.valueOf", "get seconds.valueOf", "call seconds.valueOf", "get milliseconds.valueOf", "call milliseconds.valueOf", "get microseconds.valueOf", "call microseconds.valueOf", "get nanoseconds.valueOf", "call nanoseconds.valueOf"]
  ],
];

for (const [description, args, expected] of tests) {
  const actual = [];
  const args_ = args.map((o) => o(actual));
  assert.throws(RangeError, () => new Temporal.Duration(...args_), description);
  assert.compareArray(actual, expected, `${description} order of operations`);
}
