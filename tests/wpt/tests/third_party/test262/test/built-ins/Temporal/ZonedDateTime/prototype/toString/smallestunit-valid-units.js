// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.tostring
description: Valid units for the smallestUnit option
features: [Temporal]
---*/

const datetime = new Temporal.ZonedDateTime(1_000_000_000_123_456_789n, "UTC");

function test(instance, expectations, description) {
  for (const [smallestUnit, expectedResult] of expectations) {
    assert.sameValue(instance.toString({ smallestUnit }), expectedResult,
      `${description} with smallestUnit "${smallestUnit}"`);
  }
}

test(
  datetime,
  [
    ["minute", "2001-09-09T01:46+00:00[UTC]"],
    ["second", "2001-09-09T01:46:40+00:00[UTC]"],
    ["millisecond", "2001-09-09T01:46:40.123+00:00[UTC]"],
    ["microsecond", "2001-09-09T01:46:40.123456+00:00[UTC]"],
    ["nanosecond", "2001-09-09T01:46:40.123456789+00:00[UTC]"],
  ],
  "subseconds toString"
);

test(
  new Temporal.ZonedDateTime(999_999_960_000_000_000n, "UTC"),
  [
    ["minute", "2001-09-09T01:46+00:00[UTC]"],
    ["second", "2001-09-09T01:46:00+00:00[UTC]"],
    ["millisecond", "2001-09-09T01:46:00.000+00:00[UTC]"],
    ["microsecond", "2001-09-09T01:46:00.000000+00:00[UTC]"],
    ["nanosecond", "2001-09-09T01:46:00.000000000+00:00[UTC]"],
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
