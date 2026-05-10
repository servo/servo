// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.prototype.tostring
description: fractionalSecondDigits option is not used with smallestUnit present
features: [Temporal]
---*/

const time = new Temporal.PlainTime(12, 34, 56, 789, 999, 999);
const tests = [
  ["minute", "12:34"],
  ["second", "12:34:56"],
  ["millisecond", "12:34:56.789"],
  ["microsecond", "12:34:56.789999"],
  ["nanosecond", "12:34:56.789999999"],
];

for (const [smallestUnit, expected] of tests) {
  const string = time.toString({
    smallestUnit,
    fractionalSecondDigits: 5,
  });
  assert.sameValue(string, expected, `smallestUnit: "${smallestUnit}" overrides fractionalSecondDigits`);
}

assert.throws(RangeError, () => time.toString({
  smallestUnit: "hour",
  fractionalSecondDigits: 5,
}), "hour is an invalid smallestUnit but still overrides fractionalSecondDigits");
