// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.total
description: BalanceTimeDuration computes on exact mathematical values.
features: [BigInt, Temporal]
---*/

const seconds = 8692288669465520;

{
  const milliseconds = 513;
  const d = new Temporal.Duration(0, 0, 0, 0, 0, 0, seconds, milliseconds);

  const result = d.total({ unit: "milliseconds" });

  // The result should be the nearest Number value to 8692288669465520512
  const expectedMilliseconds = Number(BigInt(seconds) * 1000n + BigInt(milliseconds));
  assert.sameValue(expectedMilliseconds, 8692288669465520_513, "check expected value (ms)");

  assert.sameValue(
    result, expectedMilliseconds,
    "BalanceTimeDuration should implement floating-point calculation correctly for largestUnit milliseconds"
  );
}

{
  const microseconds = 373761;
  const d = new Temporal.Duration(0, 0, 0, 0, 0, 0, seconds, 0, microseconds);

  const result = d.total({ unit: "microseconds" });

  // The result should be the nearest Number value to 8692288669465520373761
  const expectedMicroseconds = Number(BigInt(seconds) * 1_000_000n + BigInt(microseconds));
  assert.sameValue(expectedMicroseconds, 8692288669465520_373_761, "check expected value (µs)");

  assert.sameValue(
    result, expectedMicroseconds,
    "BalanceTimeDuration should implement floating-point calculation correctly for largestUnit microseconds"
  );
}

{
  const nanoseconds = 321_414_345;
  const d = new Temporal.Duration(0, 0, 0, 0, 0, 0, seconds, 0, 0, nanoseconds);

  const result = d.total({ unit: "nanoseconds" });

  // The result should be the nearest Number value to 8692288669465520321414345
  const expectedNanoseconds = Number(BigInt(seconds) * 1_000_000_000n + BigInt(nanoseconds));
  assert.sameValue(expectedNanoseconds, 8692288669465520_321_414_345, "check expected value (ns)");

  assert.sameValue(
    result, expectedNanoseconds,
    "BalanceTimeDuration should implement floating-point calculation correctly for largestUnit nanoseconds"
  );
}

{
  const d = new Temporal.Duration(0, 0, 5, 5);

  const result = d.total({ unit: "months", relativeTo: "1972-01-31" })

/*
Expected months checked using Decimals in Python:

>>> from decimal import *
>>> getcontext().prec = 18
>>> dest_epoch_ns = Decimal(69120000000000000)
>>> start_epoch_ns = Decimal(68169600000000000)
>>> end_epoch_ns = 70848000000000000
>>> progress = ((dest_epoch_ns - start_epoch_ns) / (end_epoch_ns - start_epoch_ns))
>>> progress
Decimal('0.354838709677419355')
>>> Decimal(1) + progress
Decimal('1.35483870967741936')

The result should be truncated.
*/
  const expectedMonths = 1.3548387096774193;

  assert.sameValue(result, expectedMonths,
    "NudgeToCalendarUnit should implement floating-point calculation correctly for largestUnit months");
}
