// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.from
description: >
  Start-of-day is outside the valid epoch nanoseconds limits.
info: |
  Temporal.ZonedDateTime.from ( item [ , options ] )

  1. Return ? ToTemporalZonedDateTime(item, options).

  ToTemporalZonedDateTime ( item [ , options ] )

  ...
  8. Let epochNanoseconds be ? InterpretISODateTimeOffset(isoDate, time,
     offsetBehaviour, offsetNanoseconds, timeZone, disambiguation, offsetOption,
     matchBehaviour).
  ...

  InterpretISODateTimeOffset ( isoDate, time, offsetBehaviour, offsetNanoseconds,
                               timeZone, disambiguation, offsetOption, matchBehaviour )

  1. If time is start-of-day, then
    a. Assert: offsetBehaviour is wall.
    b. Assert: offsetNanoseconds is 0.
    c. Return ? GetStartOfDay(timeZone, isoDate).
  ...
features: [Temporal]
---*/

assert.throws(
  RangeError,
  () => Temporal.ZonedDateTime.from("-271821-04-20[+01]"),
  "From '-271821-04-20[+01]'"
);

assert.throws(
  RangeError,
  () => Temporal.ZonedDateTime.from("+275760-09-13[-01]"),
  "From '+275760-09-13[-01]'"
);
