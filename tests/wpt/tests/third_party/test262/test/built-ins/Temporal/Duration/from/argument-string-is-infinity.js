// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.from
description: >
  Throws RangeError when when any duration component is Infinity.
info: |
  Temporal.Duration.from ( item )

  1. Return ? ToTemporalDuration(item).

  ToTemporalDuration ( item )

  ...
  2. If item is not an Object, then
    ...
    b. Return ? ParseTemporalDurationString(item).
  ...

  ParseTemporalDurationString ( isoString )

  ...
  44. Return ? CreateTemporalDuration(yearsMV, monthsMV, weeksMV, daysMV, hoursMV,
      minutesMV, secondsMV, millisecondsMV, microsecondsMV, nanosecondsMV).


  CreateTemporalDuration ( years, months, weeks, days, hours, minutes, seconds,
                           milliseconds, microseconds, nanoseconds [ , newTarget ] )

  1. If IsValidDuration(years, months, weeks, days, hours, minutes, seconds, milliseconds,
     microseconds, nanoseconds) is false, throw a RangeError exception.
  ...

  IsValidDuration ( years, months, weeks, days, hours, minutes, seconds, milliseconds,
                    microseconds, nanoseconds )

  ...
  2. For each value v of « years, months, weeks, days, hours, minutes, seconds,
     milliseconds, microseconds, nanoseconds », do
    a. If 𝔽(v) is not finite, return false.
    ...
features: [Temporal]
---*/

var inf = "9".repeat(1000);

assert.throws(RangeError, () => Temporal.Duration.from(`P${inf}Y`));
assert.throws(RangeError, () => Temporal.Duration.from(`P${inf}M`));
assert.throws(RangeError, () => Temporal.Duration.from(`P${inf}W`));
assert.throws(RangeError, () => Temporal.Duration.from(`P${inf}D`));
assert.throws(RangeError, () => Temporal.Duration.from(`PT${inf}H`));
assert.throws(RangeError, () => Temporal.Duration.from(`PT${inf}M`));
assert.throws(RangeError, () => Temporal.Duration.from(`PT${inf}S`));
