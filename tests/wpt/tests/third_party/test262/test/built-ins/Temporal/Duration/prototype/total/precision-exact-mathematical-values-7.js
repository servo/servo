// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.total
description: >
  DivideNormalizedTimeDuration computes on exact mathematical values.
info: |
  Temporal.Duration.prototype.total ( totalOf )

  ...
  20. Let roundRecord be ? RoundDuration(unbalanceResult.[[Years]],
      unbalanceResult.[[Months]], unbalanceResult.[[Weeks]], days, norm, 1,
      unit, "trunc", plainRelativeTo, calendarRec, zonedRelativeTo, timeZoneRec,
      precalculatedPlainDateTime).
  21. Return 𝔽(roundRecord.[[Total]]).

  RoundDuration ( ... )

  ...
  16. Else if unit is "second", then
    a. Let divisor be 10^9.
    b. Set total to DivideNormalizedTimeDuration(norm, divisor).
    ...
  17. Else if unit is "millisecond", then
    a. Let divisor be 10^6.
    b. Set total to DivideNormalizedTimeDuration(norm, divisor).
    ...
  18. Else if unit is "microsecond", then
    a. Let divisor be 10^3.
    b. Set total to DivideNormalizedTimeDuration(norm, divisor).
    ...

  DivideNormalizedTimeDuration ( d, divisor )

  1. Assert: divisor ≠ 0.
  2. Return d.[[TotalNanoseconds]] / divisor.
features: [Temporal]
---*/

// Test duration units where the fractional part is a power of ten.
const units = [
  "seconds", "milliseconds", "microseconds", "nanoseconds",
];

// Conversion factors to nanoseconds precision.
const toNanos = {
  "seconds": 1_000_000_000n,
  "milliseconds": 1_000_000n,
  "microseconds": 1_000n,
  "nanoseconds": 1n,
};

const integers = [
  // Small integers.
  0,
  1,
  2,

  // Large integers around Number.MAX_SAFE_INTEGER.
  2**51,
  2**52,
  2**53,
  2**54,
];

const fractions = [
  // True fractions.
  0, 1, 10, 100, 125, 200, 250, 500, 750, 800, 900, 950, 999,

  // Fractions with overflow.
  1_000,
  1_999,
  2_000,
  2_999,
  3_000,
  3_999,
  4_000,
  4_999,

  999_999,
  1_000_000,
  1_000_001,

  999_999_999,
  1_000_000_000,
  1_000_000_001,
];

const maxTimeDuration = (2n ** 53n) * (10n ** 9n) - 1n;

// Iterate over all units except the last one.
for (let unit of units.slice(0, -1)) {
  let smallerUnit = units[units.indexOf(unit) + 1];

  for (let integer of integers) {
    for (let fraction of fractions) {
      // Total nanoseconds must not exceed |maxTimeDuration|.
      let totalNanoseconds = BigInt(integer) * toNanos[unit] + BigInt(fraction) * toNanos[smallerUnit];
      if (totalNanoseconds > maxTimeDuration) {
        continue;
      }

      // Get the Number approximation from the string representation.
      let i = BigInt(integer) + BigInt(fraction) / 1000n;
      let f = String(fraction % 1000).padStart(3, "0");
      let expected = Number(`${i}.${f}`);

      let d = Temporal.Duration.from({[unit]: integer, [smallerUnit]: fraction});
      let actual = d.total(unit);

      assert.sameValue(
        actual,
        expected,
        `${unit}=${integer}, ${smallerUnit}=${fraction}`,
      );
    }
  }
}
