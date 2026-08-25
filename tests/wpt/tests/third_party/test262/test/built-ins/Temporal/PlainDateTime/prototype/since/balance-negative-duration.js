// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.since
description: Negative durations are balanced correctly by the modulo operation in NanosecondsToDays
info: |
    sec-temporal-nanosecondstodays step 6:
      6. If Type(_relativeTo_) is not Object or _relativeTo_ does not have an [[InitializedTemporalZonedDateTime]] internal slot, then
        a. Return the new Record { ..., [[Nanoseconds]]: abs(_nanoseconds_) modulo _dayLengthNs_ × _sign_, ... }.
    sec-temporal-balanceduration step 4:
      4. If _largestUnit_ is one of *"year"*, *"month"*, *"week"*, or *"day"*, then
        a. Let _result_ be ? NanosecondsToDays(_nanoseconds_, _relativeTo_).
    sec-temporal-differenceisodatetime steps 7 and 13:
      7. If _timeSign_ is -_dateSign_, then
        ...
        b. Set _timeDifference_ to ? BalanceDuration(-_timeSign_, _timeDifference_.[[Hours]], _timeDifference_.[[Minutes]], _timeDifference_.[[Seconds]], _timeDifference_.[[Milliseconds]], _timeDifference_.[[Microseconds]], _timeDifference_.[[Nanoseconds]], _largestUnit_).
      ...
      16. Return ? BalanceDuration(_dateDifference_.[[Years]], _dateDifference_.[[Months]], _dateDifference_.[[Weeks]], _dateDifference_.[[Days]], _timeDifference_.[[Hours]], _timeDifference_.[[Minutes]], _timeDifference_.[[Seconds]], _timeDifference_.[[Milliseconds]], _timeDifference_.[[Microseconds]], _timeDifference_.[[Nanoseconds]], _largestUnit_).
    sec-temporal.plaindatetime.prototype.since step 14:
      14. Let _diff_ be ? DifferenceISODateTime(_other_.[[ISOYear]], _other_.[[ISOMonth]], _other_.[[ISODay]], _other_.[[ISOHour]], _other_.[[ISOMinute]], _other_.[[ISOSecond]], _other_.[[ISOMillisecond]], _other_.[[ISOMicrosecond]], _other_.[[ISONanosecond]], _dateTime_.[[ISOYear]], _dateTime_.[[ISOMonth]], _dateTime_.[[ISODay]], _dateTime_.[[ISOHour]], _dateTime_.[[ISOMinute]], _dateTime_.[[ISOSecond]], _dateTime_.[[ISOMillisecond]], _dateTime_.[[ISOMicrosecond]], _dateTime_.[[ISONanosecond]], _dateTime_.[[Calendar]], _largestUnit_, _options_).
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const earlier1 = new Temporal.PlainDateTime(2000, 5, 2, 9);
const later1 = new Temporal.PlainDateTime(2000, 5, 5, 10);
const result1 = later1.since(earlier1, { largestUnit: 'day' });
TemporalHelpers.assertDuration(result1, 0, 0, 0, 3, 1, 0, 0, 0, 0, 0, "date sign == time sign");

const earlier2 = new Temporal.PlainDateTime(2000, 5, 2, 10);
const later2 = new Temporal.PlainDateTime(2000, 5, 5, 9);
const result2 = later2.since(earlier2, { largestUnit: 'day' });
TemporalHelpers.assertDuration(result2, 0, 0, 0, 2, 23, 0, 0, 0, 0, 0, "date sign != time sign");
