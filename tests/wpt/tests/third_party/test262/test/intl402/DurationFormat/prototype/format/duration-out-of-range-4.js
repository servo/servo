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

const df = new Intl.DurationFormat();

const duration = {
  // Actual value is: 4503599627370497024
  milliseconds: 4503599627370497_000,

  // Actual value is: 4503599627370494951424
  microseconds: 4503599627370495_000000,
};

// The naive approach to compute the duration seconds leads to an incorrect result.
let durationSecondsNaive = Math.trunc(duration.milliseconds / 1e3 + duration.microseconds / 1e6);
assert.sameValue(
  Number.isSafeInteger(durationSecondsNaive),
  false,
  "Naive approach incorrectly computes duration seconds as out-of-range"
);

// The exact approach to compute the duration seconds leads to the correct result.
let durationSecondsExact = Number(BigInt(duration.milliseconds) / 1_000n) +
                           Number(BigInt(duration.microseconds) / 1_000_000n) +
                           Math.trunc(((duration.milliseconds % 1e3) * 1e3 + (duration.microseconds % 1e6)) / 1e6);
assert.sameValue(
  Number.isSafeInteger(Number(durationSecondsExact)),
  true,
  "Exact approach correctly computes duration seconds as in-range"
);

// We don't care about the exact contents of the returned string, the call
// just shouldn't throw an exception.
assert.sameValue(
  typeof df.format(duration),
  "string",
  `Duration "${JSON.stringify(duration)}" doesn't throw`
);
