// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.until
description: >
  Throws if rounded date outside valid ISO date range.
info: |
  Temporal.PlainDateTime.prototype.until ( other [ , options ] )

  ...
  3. Return ? DifferenceTemporalPlainDateTime(until, dateTime, other, options).

  DifferenceTemporalPlainDateTime ( operation, dateTime, other, options )

  ...
  6. Let internalDuration be ? DifferencePlainDateTimeWithRounding(dateTime.[[ISODateTime]],
     other.[[ISODateTime]], dateTime.[[Calendar]], settings.[[LargestUnit]],
     settings.[[RoundingIncrement]], settings.[[SmallestUnit]], settings.[[RoundingMode]]).
  ...

  DifferencePlainDateTimeWithRounding ( isoDateTime1, isoDateTime2, calendar, largestUnit,
                                        roundingIncrement, smallestUnit, roundingMode )

  ...
  5. Return ? RoundRelativeDuration(diff, destEpochNs, isoDateTime1, unset, calendar,
     largestUnit, roundingIncrement, smallestUnit, roundingMode).

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

var from = new Temporal.PlainDateTime(1970, 1, 1);
var to = new Temporal.PlainDateTime(1971, 1, 1);
var options = {roundingIncrement: 100_000_000, smallestUnit: "months"};

assert.throws(RangeError, () => from.until(to, options));
