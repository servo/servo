// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.tostring
description: fractionalSecondDigits option is not used with smallestUnit present
features: [Temporal]
---*/

const datetime = new Temporal.ZonedDateTime(56_789_999_999n, "UTC");
const tests = [
  ["minute", "1970-01-01T00:00+00:00[UTC]"],
  ["second", "1970-01-01T00:00:56+00:00[UTC]"],
  ["millisecond", "1970-01-01T00:00:56.789+00:00[UTC]"],
  ["microsecond", "1970-01-01T00:00:56.789999+00:00[UTC]"],
  ["nanosecond", "1970-01-01T00:00:56.789999999+00:00[UTC]"],
];

for (const [smallestUnit, expected] of tests) {
  const string = datetime.toString({
    smallestUnit,
    fractionalSecondDigits: 5,
  });
  assert.sameValue(string, expected, `smallestUnit: "${smallestUnit}" overrides fractionalSecondDigits`);
}

assert.throws(RangeError, () => datetime.toString({
  smallestUnit: "hour",
  fractionalSecondDigits: 5,
}), "hour is an invalid smallestUnit but still overrides fractionalSecondDigits");
