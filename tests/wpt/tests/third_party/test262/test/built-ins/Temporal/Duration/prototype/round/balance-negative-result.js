// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.round
description: A negative duration result is balanced correctly by the modulo operation in NanosecondsToDays
info: |
    sec-temporal-nanosecondstodays step 6:
      6. If Type(_relativeTo_) is not Object or _relativeTo_ does not have an [[InitializedTemporalZonedDateTime]] internal slot, then
        a. Return the new Record { ..., [[Nanoseconds]]: abs(_nanoseconds_) modulo _dayLengthNs_ × _sign_, ... }.
    sec-temporal-balanceduration step 4:
      4. If _largestUnit_ is one of *"year"*, *"month"*, *"week"*, or *"day"*, then
        a. Let _result_ be ? NanosecondsToDays(_nanoseconds_, _relativeTo_).
    sec-temporal.duration.prototype.round step 25:
      25. Let _result_ be ? BalanceDuration(_balanceResult_.[[Days]], _adjustResult_.[[Hours]], _adjustResult_.[[Minutes]], _adjustResult_.[[Seconds]], _adjustResult_.[[Milliseconds]], _adjustResult_.[[Microseconds]], _adjustResult_.[[Nanoseconds]], _largestUnit_, _relativeTo_).
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const duration = new Temporal.Duration(0, 0, 0, 0, -60);
const result = duration.round({ largestUnit: "days" });
TemporalHelpers.assertDuration(result, 0, 0, 0, -2, -12, 0, 0, 0, 0, 0);
