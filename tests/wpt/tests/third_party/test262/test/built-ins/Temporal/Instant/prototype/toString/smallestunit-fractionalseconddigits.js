// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.tostring
description: fractionalSecondDigits option is not used with smallestUnit present
features: [Temporal]
---*/

const instant = new Temporal.Instant(56_789_999_999n);
const tests = [
  ["minute", "1970-01-01T00:00Z"],
  ["second", "1970-01-01T00:00:56Z"],
  ["millisecond", "1970-01-01T00:00:56.789Z"],
  ["microsecond", "1970-01-01T00:00:56.789999Z"],
  ["nanosecond", "1970-01-01T00:00:56.789999999Z"],
];

for (const [smallestUnit, expected] of tests) {
  const string = instant.toString({
    smallestUnit,
    fractionalSecondDigits: 5,
  });
  assert.sameValue(string, expected, `smallestUnit: "${smallestUnit}" overrides fractionalSecondDigits`);
}

assert.throws(RangeError, () => instant.toString({
  smallestUnit: "hour",
  fractionalSecondDigits: 5,
}), "hour is an invalid smallestUnit but still overrides fractionalSecondDigits");
