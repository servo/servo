// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.tostring
description: Valid units for the smallestUnit option
features: [Temporal]
---*/

const duration = new Temporal.Duration(1, 2, 3, 4, 5, 6, 7, 987, 654, 321);

function test(instance, expectations, description) {
  for (const [smallestUnit, expectedResult] of expectations) {
    assert.sameValue(instance.toString({ smallestUnit }), expectedResult,
      `${description} with smallestUnit "${smallestUnit}"`);
  }
}

test(
  duration,
  [
    ["seconds", "P1Y2M3W4DT5H6M7S"],
    ["milliseconds", "P1Y2M3W4DT5H6M7.987S"],
    ["microseconds", "P1Y2M3W4DT5H6M7.987654S"],
    ["nanoseconds", "P1Y2M3W4DT5H6M7.987654321S"],
  ],
  "subseconds toString"
);

test(
  new Temporal.Duration(1, 2, 3, 4, 5, 6, 7),
  [
    ["seconds", "P1Y2M3W4DT5H6M7S"],
    ["milliseconds", "P1Y2M3W4DT5H6M7.000S"],
    ["microseconds", "P1Y2M3W4DT5H6M7.000000S"],
    ["nanoseconds", "P1Y2M3W4DT5H6M7.000000000S"],
  ],
  "whole seconds toString"
);

const notValid = [
  "era",
  "year",
  "month",
  "week",
  "day",
  "hour",
  "minute",
];

notValid.forEach((smallestUnit) => {
  assert.throws(RangeError, () => duration.toString({ smallestUnit }),
    `"${smallestUnit}" is not a valid unit for the smallestUnit option`);
});
