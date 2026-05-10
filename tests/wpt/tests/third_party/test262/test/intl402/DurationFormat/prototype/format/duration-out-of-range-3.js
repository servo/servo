// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.DurationFormat.prototype.format
description: >
  IsValidDurationRecord rejects too large time duration units.
info: |
  Intl.DurationFormat.prototype.format ( duration )
  ...
  3. Let record be ? ToDurationRecord(duration).
  ...

  ToDurationRecord ( input )
  ...
  24. If IsValidDurationRecord(result) is false, throw a RangeError exception.
  ...

  IsValidDurationRecord ( record )
  ...
  16. Let normalizedSeconds be days × 86,400 + hours × 3600 + minutes × 60 + seconds +
      milliseconds × 10^-3 + microseconds × 10^-6 + nanoseconds × 10^-9.
  17. If abs(normalizedSeconds) ≥ 2^53, return false.
  ...

features: [Intl.DurationFormat]
---*/

// Return the next Number value in direction to -Infinity.
function nextDown(num) {
  if (!Number.isFinite(num)) {
    return num;
  }
  if (num === 0) {
    return -Number.MIN_VALUE;
  }

  var f64 = new Float64Array([num]);
  var u64 = new BigUint64Array(f64.buffer);
  u64[0] += (num < 0 ? 1n : -1n);
  return f64[0];
}

// Negate |duration| similar to Temporal.Duration.prototype.negated.
function negatedDuration(duration) {
  let result = {...duration};
  for (let key of Object.keys(result)) {
    // Add +0 to normalize -0 to +0.
    result[key] = -result[key] + 0;
  }
  return result;
}

function fromNanoseconds(unit, value) {
  switch (unit) {
    case "days":
      return value / (86400n * 1_000_000_000n);
    case "hours":
      return value / (3600n * 1_000_000_000n);
    case "minutes":
      return value / (60n * 1_000_000_000n);
    case "seconds":
      return value / 1_000_000_000n;
    case "milliseconds":
      return value / 1_000_000n;
    case "microseconds":
      return value / 1_000n;
    case "nanoseconds":
      return value;
  }
  throw new Error("invalid unit:" + unit);
}

function toNanoseconds(unit, value) {
  switch (unit) {
    case "days":
      return value * 86400n * 1_000_000_000n;
    case "hours":
      return value * 3600n * 1_000_000_000n;
    case "minutes":
      return value * 60n * 1_000_000_000n;
    case "seconds":
      return value * 1_000_000_000n;
    case "milliseconds":
      return value * 1_000_000n;
    case "microseconds":
      return value * 1_000n;
    case "nanoseconds":
      return value;
  }
  throw new Error("invalid unit:" + unit);
}

const df = new Intl.DurationFormat();

const units = [
  "days",
  "hours",
  "minutes",
  "seconds",
  "milliseconds",
  "microseconds",
  "nanoseconds",
];

const zeroDuration = {
  days: 0,
  hours: 0,
  minutes: 0,
  seconds: 0,
  milliseconds: 0,
  microseconds: 0,
  nanoseconds: 0,
};

const maxTimeDuration = BigInt(Number.MAX_SAFE_INTEGER) * 1_000_000_000n + 999_999_999n;

// Iterate over all time duration units and create the largest possible duration.
for (let i = 0; i < units.length; ++i) {
  let unit = units[i];

  // Test not only the next smallest unit, but all smaller units.
  for (let j = i + 1; j < units.length; ++j) {
    // Maximum duration value for |unit|.
    let maxUnit = fromNanoseconds(unit, maxTimeDuration);

    // Adjust |maxUnit| when the value is too large for Number.
    let adjusted = BigInt(Number(maxUnit));
    if (adjusted <= maxUnit) {
      maxUnit = adjusted;
    } else {
      maxUnit = BigInt(nextDown(Number(maxUnit)));
    }

    // Remaining number of nanoseconds.
    let remaining = maxTimeDuration - toNanoseconds(unit, maxUnit);

    // Create the maximum valid duration.
    let maxDuration = {
      ...zeroDuration,
      [unit]: Number(maxUnit),
    };
    for (let k = j; k < units.length; ++k) {
      let smallerUnit = units[k];

      // Remaining number of nanoseconds in |smallerUnit|.
      let remainingSmallerUnit = fromNanoseconds(smallerUnit, remaining);
      maxDuration[smallerUnit] = Number(remainingSmallerUnit);

      remaining -= toNanoseconds(smallerUnit, remainingSmallerUnit);
    }
    assert.sameValue(remaining, 0n, "zero remaining nanoseconds");

    // We don't care about the exact contents of the returned string, the call
    // just shouldn't throw an exception.
    assert.sameValue(
      typeof df.format(maxDuration),
      "string",
      `Duration "${JSON.stringify(maxDuration)}" doesn't throw`
    );

    // Also test with flipped sign.
    let minDuration = negatedDuration(maxDuration);

    // We don't care about the exact contents of the returned string, the call
    // just shouldn't throw an exception.
    assert.sameValue(
      typeof df.format(minDuration),
      "string",
      `Duration "${JSON.stringify(minDuration)}" doesn't throw`
    );

    // Adding a single nanoseconds creates a too large duration.
    let tooLargeDuration = {
      ...maxDuration,
      nanoseconds: maxDuration.nanoseconds + 1,
    };

    assert.throws(
      RangeError,
      () => df.format(tooLargeDuration),
      `Duration "${JSON.stringify(tooLargeDuration)}" throws`
    );

    // Also test with flipped sign.
    let tooSmallDuration = negatedDuration(tooLargeDuration);

    assert.throws(
      RangeError,
      () => df.format(tooSmallDuration),
      `Duration "${JSON.stringify(tooSmallDuration)}" throws`
    );
  }
}
