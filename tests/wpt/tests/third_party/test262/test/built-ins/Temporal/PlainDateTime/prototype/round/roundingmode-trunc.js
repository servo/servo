// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.round
description: Tests calculations with roundingMode "trunc".
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const instance = new Temporal.PlainDateTime(1976, 11, 18, 14, 23, 30, 123, 987, 500);

const expected = [
  ["day", [1976, 11, 'M11', 18]],
  ["hour", [1976, 11, 'M11', 18, 14]],
  ["minute", [1976, 11, 'M11', 18, 14, 23]],
  ["second", [1976, 11, 'M11', 18, 14, 23, 30]],
  ["millisecond", [1976, 11, 'M11', 18, 14, 23, 30, 123]],
  ["microsecond", [1976, 11, 'M11', 18, 14, 23, 30, 123, 987]],
  ["nanosecond", [1976, 11, 'M11', 18, 14, 23, 30, 123, 987, 500]],
];

const roundingMode = "trunc";

expected.forEach(([smallestUnit, expected]) => {
  const [y, mon, mc, d, h = 0, min = 0, s = 0, ms = 0, µs = 0, ns = 0] = expected;
  TemporalHelpers.assertPlainDateTime(
    instance.round({ smallestUnit, roundingMode }),
    y, mon, mc, d, h, min, s, ms, µs, ns,
    `rounds to ${smallestUnit} (roundingMode = ${roundingMode})`
  );
});
