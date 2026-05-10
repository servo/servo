// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.until
description: Negative durations are rounded correctly by the modulo operation in NanosecondsToDays
info: |
    sec-temporal-nanosecondstodays step 6:
      6. If Type(_relativeTo_) is not Object or _relativeTo_ does not have an [[InitializedTemporalZonedDateTime]] internal slot, then
        a. Return the new Record { ..., [[Nanoseconds]]: abs(_nanoseconds_) modulo _dayLengthNs_ × _sign_, ... }.
    sec-temporal-roundduration step 6:
      6. If _unit_ is one of *"year"*, *"month"*, *"week"*, or *"day"*, then
        ...
        d. Let _result_ be ? NanosecondsToDays(_nanoseconds_, _intermediate_).
    sec-temporal.plaindatetime.prototype.until step 14:
      14. Let _roundResult_ be ? RoundDuration(_diff_.[[Years]], _diff_.[[Months]], _diff_.[[Weeks]], _diff_.[[Days]], _diff_.[[Hours]], _diff_.[[Minutes]], _diff_.[[Seconds]], _diff_.[[Milliseconds]], _diff_.[[Microseconds]], _diff_.[[Nanoseconds]], _roundingIncrement_, _smallestUnit_, _roundingMode_, _dateTime_).
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const earlier = new Temporal.PlainDateTime(2000, 5, 2, 12);
const later = new Temporal.PlainDateTime(2000, 5, 5);
const result = later.until(earlier, { smallestUnit: "day", roundingIncrement: 2 });
TemporalHelpers.assertDuration(result, 0, 0, 0, -2, 0, 0, 0, 0, 0, 0);
