// Copyright (C) 2020 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime
description: Arguments are converted to primitives in the correct order
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const actual = [];
const expected = [
  "get (argument 0).valueOf",
  "call (argument 0).valueOf",
  "get (argument 1).valueOf",
  "call (argument 1).valueOf",
  "get (argument 2).valueOf",
  "call (argument 2).valueOf",
  "get (argument 3).valueOf",
  "call (argument 3).valueOf",
  "get (argument 4).valueOf",
  "call (argument 4).valueOf",
  "get (argument 5).valueOf",
  "call (argument 5).valueOf",
  "get (argument 6).valueOf",
  "call (argument 6).valueOf",
  "get (argument 7).valueOf",
  "call (argument 7).valueOf",
  "get (argument 8).valueOf",
  "call (argument 8).valueOf",
];

const dateTimeArgs = [2020, 12, 24, 12, 34, 56, 123, 456, 789].map((value, idx) =>
  TemporalHelpers.toPrimitiveObserver(actual, value, `(argument ${idx})`));

const dateTime = new Temporal.PlainDateTime(...dateTimeArgs, "iso8601");
assert.compareArray(actual, expected);

TemporalHelpers.assertPlainDateTime(dateTime, 2020, 12, "M12", 24, 12, 34, 56, 123, 456, 789);
