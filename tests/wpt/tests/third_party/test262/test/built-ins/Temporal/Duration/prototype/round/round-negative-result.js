// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.round
description: A negative duration result is balanced correctly by the modulo operation in NanosecondsToDays
info: |
    sec-temporal-nanosecondstodays step 6:
      6. If Type(_relativeTo_) is not Object or _relativeTo_ does not have an [[InitializedTemporalZonedDateTime]] internal slot, then
        a. Return the new Record { ..., [[Nanoseconds]]\: abs(_nanoseconds_) modulo _dayLengthNs_ × _sign_, ... }.
    sec-temporal-roundduration step 6:
      6. If _unit_ is one of *"year"*, *"month"*, *"week"*, or *"day"*, then
        ...
        d. Let _result_ be ? NanosecondsToDays(_nanoseconds_, _intermediate_).
    sec-temporal.duration.prototype.round step 21:
      21. Let _roundResult_ be ? RoundDuration(_unbalanceResult_.[[Years]], _unbalanceResult_.[[Months]], _unbalanceResult_.[[Weeks]], _unbalanceResult_.[[Days]], _duration_.[[Hours]], _duration_.[[Minutes]], _duration_[[Seconds]], _duration_[[Milliseconds]], _duration_.[[Microseconds]], _duration_.[[Nanoseconds]], _roundingIncrement_, _smallestUnit_, _roundingMode_, _relativeTo_).
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const duration = new Temporal.Duration(0, 0, 0, 0, -60);
const result = duration.round({ smallestUnit: "days" });
TemporalHelpers.assertDuration(result, 0, 0, 0, -3, 0, 0, 0, 0, 0, 0);
