// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.tostring
description: Valid units for the smallestUnit option
features: [Temporal]
---*/

const datetime = new Temporal.PlainDateTime(2000, 5, 2, 12, 34, 56, 123, 456, 789);

function test(instance, expectations, description) {
  for (const [smallestUnit, expectedResult] of expectations) {
    assert.sameValue(instance.toString({ smallestUnit }), expectedResult,
      `${description} with smallestUnit "${smallestUnit}"`);
  }
}

test(
  datetime,
  [
    ["minute", "2000-05-02T12:34"],
    ["second", "2000-05-02T12:34:56"],
    ["millisecond", "2000-05-02T12:34:56.123"],
    ["microsecond", "2000-05-02T12:34:56.123456"],
    ["nanosecond", "2000-05-02T12:34:56.123456789"],
  ],
  "subseconds toString"
);

test(
  new Temporal.PlainDateTime(2000, 5, 2, 12, 34),
  [
    ["minute", "2000-05-02T12:34"],
    ["second", "2000-05-02T12:34:00"],
    ["millisecond", "2000-05-02T12:34:00.000"],
    ["microsecond", "2000-05-02T12:34:00.000000"],
    ["nanosecond", "2000-05-02T12:34:00.000000000"],
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
  assert.throws(RangeError, () => datetime.toString({ smallestUnit }),
    `"${smallestUnit}" is not a valid unit for the smallestUnit option`);
});
