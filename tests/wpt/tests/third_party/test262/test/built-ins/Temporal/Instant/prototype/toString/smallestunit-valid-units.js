// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.tostring
description: Valid units for the smallestUnit option
features: [Temporal]
---*/

const instant = new Temporal.Instant(1_000_000_000_123_456_789n);

function test(instance, expectations, description) {
  for (const [smallestUnit, expectedResult] of expectations) {
    assert.sameValue(instance.toString({ smallestUnit }), expectedResult,
      `${description} with smallestUnit "${smallestUnit}"`);
  }
}

test(
  instant,
  [
    ["minute", "2001-09-09T01:46Z"],
    ["second", "2001-09-09T01:46:40Z"],
    ["millisecond", "2001-09-09T01:46:40.123Z"],
    ["microsecond", "2001-09-09T01:46:40.123456Z"],
    ["nanosecond", "2001-09-09T01:46:40.123456789Z"],
  ],
  "subseconds toString"
);

test(
  new Temporal.Instant(999_999_960_000_000_000n),
  [
    ["minute", "2001-09-09T01:46Z"],
    ["second", "2001-09-09T01:46:00Z"],
    ["millisecond", "2001-09-09T01:46:00.000Z"],
    ["microsecond", "2001-09-09T01:46:00.000000Z"],
    ["nanosecond", "2001-09-09T01:46:00.000000000Z"],
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
  assert.throws(RangeError, () => instant.toString({ smallestUnit }),
    `"${smallestUnit}" is not a valid unit for the smallestUnit option`);
});
