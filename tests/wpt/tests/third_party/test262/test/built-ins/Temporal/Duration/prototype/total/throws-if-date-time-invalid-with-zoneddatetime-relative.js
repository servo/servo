// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.total
description: >
  Throws RangeError when date-time is outside the valid limits.
info: |
  Temporal.Duration.prototype.total ( totalOf )

  ...
  11. If zonedRelativeTo is not undefined, then
    ...
    f. Let total be ? DifferenceZonedDateTimeWithTotal(relativeEpochNs,
       targetEpochNs, timeZone, calendar, unit).
  ...

  DifferenceZonedDateTimeWithTotal ( ns1, ns2, timeZone, calendar, unit )

  ...
  4. Return ? TotalRelativeDuration(difference, ns2, dateTime, timeZone, calendar, unit).

  TotalRelativeDuration ( duration, destEpochNs, isoDateTime, timeZone, calendar, unit )

  1. If IsCalendarUnit(unit) is true, or timeZone is not unset and unit is day, then
    a. Let sign be InternalDurationSign(duration).
    b. Let record be ? NudgeToCalendarUnit(sign, duration, destEpochNs, isoDateTime,
       timeZone, calendar, 1, unit, trunc).
    ...

  NudgeToCalendarUnit ( sign, duration, destEpochNs, isoDateTime, timeZone, calendar,
                        increment, unit, roundingMode )

  ...
  8. Let end be ? CalendarDateAdd(calendar, isoDateTime.[[ISODate]], endDuration, constrain).
  ...

features: [Temporal]
---*/

var duration = new Temporal.Duration(0, 0, 0, 0, 0, 0, 0, 0, 0, 1);

var relativeTo = new Temporal.ZonedDateTime(864n * 10n**19n - 1n, "UTC");

var totalOf = {
  unit: "years",
  relativeTo,
};

assert.throws(RangeError, () => duration.total(totalOf));
