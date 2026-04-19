// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.since
description: >
  Throws if rounded date outside valid ISO date range.
info: |
  Temporal.PlainDate.prototype.since ( other [ , options ] )

  ...
  3. Return ? DifferenceTemporalPlainDate(since, temporalDate, other, options).

  DifferenceTemporalPlainDate ( operation, temporalDate, other, options )

  ...
  8. If settings.[[SmallestUnit]] is not day or settings.[[RoundingIncrement]] ≠ 1, then
    ...
    d. Set duration to ? RoundRelativeDuration(duration, destEpochNs, isoDateTime,
       unset, temporalDate.[[Calendar]], settings.[[LargestUnit]],
       settings.[[RoundingIncrement]], settings.[[SmallestUnit]], settings.[[RoundingMode]]).
  ...

  RoundRelativeDuration ( duration, destEpochNs, isoDateTime, timeZone, calendar,
                          largestUnit, increment, smallestUnit, roundingMode )

  ...
  5. If irregularLengthUnit is true, then
    a. Let record be ? NudgeToCalendarUnit(sign, duration, destEpochNs, isoDateTime,
       timeZone, calendar, increment, smallestUnit, roundingMode).
    ...

  NudgeToCalendarUnit ( sign, duration, destEpochNs, isoDateTime, timeZone, calendar,
                        increment, unit, roundingMode )

  ...
  8. Let end be ? CalendarDateAdd(calendar, isoDateTime.[[ISODate]], endDuration, constrain).
  ...

features: [Temporal]
---*/

var from = new Temporal.PlainDate(1970, 1, 1);
var to = new Temporal.PlainDate(1971, 1, 1);
var options = {roundingIncrement: 100_000_000, smallestUnit: "months"};

assert.throws(RangeError, () => from.since(to, options));
