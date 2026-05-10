// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.until
description: >
  Throws if rounded date outside valid ISO date range.
info: |
  Temporal.PlainYearMonth.prototype.until ( other [ , options ] )

  ...
  3. Return ? DifferenceTemporalPlainYearMonth(until, yearMonth, other, options).

  DifferenceTemporalPlainYearMonth ( operation, yearMonth, other, options )

  ...
  16. If settings.[[SmallestUnit]] is not month or settings.[[RoundingIncrement]] ≠ 1, then
    ...
    d. Set duration to ? RoundRelativeDuration(duration, destEpochNs, isoDateTime,
       unset, calendar, settings.[[LargestUnit]], settings.[[RoundingIncrement]],
       settings.[[SmallestUnit]], settings.[[RoundingMode]]).[[Duration]].
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

var from = new Temporal.PlainYearMonth(1970, 1);
var to = new Temporal.PlainYearMonth(1971, 1);
var options = {roundingIncrement: 100_000_000};

assert.throws(RangeError, () => from.until(to, options));
