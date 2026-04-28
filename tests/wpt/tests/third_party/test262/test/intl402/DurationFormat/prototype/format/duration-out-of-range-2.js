// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.DurationFormat.prototype.format
description: >
  IsValidDurationRecord rejects too large "days", "hours", ... values.
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

  let f64 = new Float64Array([num]);
  let u64 = new BigUint64Array(f64.buffer);
  u64[0] += (num < 0 ? 1n : -1n);
  return f64[0];
}

const df = new Intl.DurationFormat();

const invalidValues = {
  days: [
    Math.ceil((Number.MAX_SAFE_INTEGER + 1) / 86400),
  ],
  hours: [
    Math.ceil((Number.MAX_SAFE_INTEGER + 1) / 3600),
  ],
  minutes: [
    Math.ceil((Number.MAX_SAFE_INTEGER + 1) / 60),
  ],
  seconds: [
    Number.MAX_SAFE_INTEGER + 1,
  ],
  milliseconds: [
    (Number.MAX_SAFE_INTEGER + 1) * 1e3,
    9007199254740992_000,
  ],
  microseconds: [
    (Number.MAX_SAFE_INTEGER + 1) * 1e6,
    9007199254740992_000_000,
  ],
  nanoseconds: [
    (Number.MAX_SAFE_INTEGER + 1) * 1e9,
    9007199254740992_000_000_000,
  ],
};

const validValues = {
  days: [
    Math.floor(Number.MAX_SAFE_INTEGER / 86400),
  ],
  hours: [
    Math.floor(Number.MAX_SAFE_INTEGER / 3600),
  ],
  minutes: [
    Math.floor(Number.MAX_SAFE_INTEGER / 60),
  ],
  seconds: [
    Number.MAX_SAFE_INTEGER,
  ],
  milliseconds: [
    Number.MAX_SAFE_INTEGER * 1e3,
    nextDown(9007199254740992_000),
  ],
  microseconds: [
    Number.MAX_SAFE_INTEGER * 1e6,
    nextDown(9007199254740992_000_000),
  ],
  nanoseconds: [
    Number.MAX_SAFE_INTEGER * 1e9,
    nextDown(9007199254740992_000_000_000),
  ],
};

for (let [unit, values] of Object.entries(invalidValues)) {
  for (let value of values) {
    let positive = {[unit]: value};
    assert.throws(
      RangeError,
      () => df.format(positive),
      `Duration "${unit}" throws when value is ${value}`
    );

    // Also test with flipped sign.
    let negative = {[unit]: -value};
    assert.throws(
      RangeError,
      () => df.format(negative),
      `Duration "${unit}" throws when value is ${-value}`
    );
  }
}

for (let [unit, values] of Object.entries(validValues)) {
  for (let value of values) {
    // We don't care about the exact contents of the returned string, the call
    // just shouldn't throw an exception.
    let positive = {[unit]: value};
    assert.sameValue(
      typeof df.format(positive),
      "string",
      `Duration "${unit}" doesn't throw when value is ${value}`
    );

    // Also test with flipped sign.
    let negative = {[unit]: -value};
    assert.sameValue(
      typeof df.format(negative),
      "string",
      `Duration "${unit}" doesn't throw when value is ${-value}`
    );
  }
}
