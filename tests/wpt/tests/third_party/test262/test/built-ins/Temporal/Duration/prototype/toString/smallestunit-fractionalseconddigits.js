// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.tostring
description: fractionalSecondDigits option is not used with smallestUnit present
features: [Temporal]
---*/

const duration = new Temporal.Duration(1, 2, 3, 4, 5, 6, 7, 789, 999, 999);
const tests = [
  ["second", "P1Y2M3W4DT5H6M7S"],
  ["millisecond", "P1Y2M3W4DT5H6M7.789S"],
  ["microsecond", "P1Y2M3W4DT5H6M7.789999S"],
  ["nanosecond", "P1Y2M3W4DT5H6M7.789999999S"],
];

for (const [smallestUnit, expected] of tests) {
  const string = duration.toString({
    smallestUnit,
    fractionalSecondDigits: 5,
  });
  assert.sameValue(string, expected, `smallestUnit: "${smallestUnit}" overrides fractionalSecondDigits`);
}

assert.throws(RangeError, () => duration.toString({
  smallestUnit: "hour",
  fractionalSecondDigits: 5,
}), "hour is an invalid smallestUnit but still overrides fractionalSecondDigits");
