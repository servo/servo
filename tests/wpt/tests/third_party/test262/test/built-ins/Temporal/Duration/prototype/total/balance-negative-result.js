// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.total
description: A negative duration result is balanced correctly by the modulo operation in NanosecondsToDays
info: |
    sec-temporal-nanosecondstodays step 6:
      6. If Type(_relativeTo_) is not Object or _relativeTo_ does not have an [[InitializedTemporalZonedDateTime]] internal slot, then
        a. Return the new Record { ..., [[Nanoseconds]]: abs(_nanoseconds_) modulo _dayLengthNs_ × _sign_, ... }.
    sec-temporal-balanceduration step 4:
      4. If _largestUnit_ is one of *"year"*, *"month"*, *"week"*, or *"day"*, then
        a. Let _result_ be ? NanosecondsToDays(_nanoseconds_, _relativeTo_).
    sec-temporal.duration.prototype.round step 9:
      9. Let _balanceResult_ be ? BalanceDuration(_unbalanceResult_.[[Days]], _unbalanceResult_.[[Hours]], _unbalanceResult_.[[Minutes]], _unbalanceResult_.[[Seconds]], _unbalanceResult_.[[Milliseconds]], _unbalanceResult_.[[Microseconds]], _unbalanceResult_.[[Nanoseconds]], _unit_, _intermediate_).
features: [Temporal]
---*/

const duration = new Temporal.Duration(0, 0, 0, 0, -60);
const result = duration.total({ unit: "days" });
assert.sameValue(result, -2.5);
