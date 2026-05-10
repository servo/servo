// Copyright (C) 2022 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.tostring
description: >
  RoundDuration throws when the rounded duration can't be represented using
  float64-representable integers.
info: |
  Temporal.Duration.prototype.toString ( [ options ] )

  ...
  7. Let result be (? RoundDuration(...)).[[DurationRecord]].
  ...

  RoundDuration ( years, months, weeks, days, hours, minutes, seconds, milliseconds, microseconds,
                  nanoseconds, increment, unit, roundingMode [ , relativeTo ] )

  ...
  15. Else if unit is "second", then
    a. Set seconds to RoundNumberToIncrement(fractionalSeconds, increment, roundingMode).
    b. Set remainder to fractionalSeconds - seconds.
    c. Set milliseconds, microseconds, and nanoseconds to 0.
  ...
  19. Let duration be ? CreateDurationRecord(years, months, weeks, days, hours, minutes, seconds,
                                             milliseconds, microseconds, nanoseconds).
  ...

  CreateDurationRecord ( years, months, weeks, days, hours, minutes, seconds, milliseconds,
                         microseconds, nanoseconds )

  1. If ! IsValidDuration(years, months, weeks, days, hours, minutes, seconds, milliseconds,
                          microseconds, nanoseconds) is false, throw a RangeError exception.
  ...
features: [Temporal]
---*/

var duration = Temporal.Duration.from({
  seconds: Number.MAX_SAFE_INTEGER,
  milliseconds: 999,
});

var options = {smallestUnit: "seconds", roundingMode: "ceil"};

assert.throws(RangeError, () => duration.toString(options));
