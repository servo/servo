// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.prototype.tostring
description: Valid units for the smallestUnit option
features: [Temporal]
---*/

const time = new Temporal.PlainTime(12, 34, 56, 123, 456, 789);

function test(instance, expectations, description) {
  for (const [smallestUnit, expectedResult] of expectations) {
    assert.sameValue(instance.toString({ smallestUnit }), expectedResult,
      `${description} with smallestUnit "${smallestUnit}"`);
  }
}

test(
  time,
  [
    ["minute", "12:34"],
    ["second", "12:34:56"],
    ["millisecond", "12:34:56.123"],
    ["microsecond", "12:34:56.123456"],
    ["nanosecond", "12:34:56.123456789"],
  ],
  "subseconds toString"
);

test(
  new Temporal.PlainTime(12, 34),
  [
    ["minute", "12:34"],
    ["second", "12:34:00"],
    ["millisecond", "12:34:00.000"],
    ["microsecond", "12:34:00.000000"],
    ["nanosecond", "12:34:00.000000000"],
  ],
  "whole minutes toString"
);

const notValid = [
  "era",
  "year",
  "month",
  "week",
  "day",
  "hour",
];

notValid.forEach((smallestUnit) => {
  assert.throws(RangeError, () => time.toString({ smallestUnit }),
    `"${smallestUnit}" is not a valid unit for the smallestUnit option`);
});
