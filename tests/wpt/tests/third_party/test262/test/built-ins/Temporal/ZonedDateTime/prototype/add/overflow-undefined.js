// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.add
description: Fallback value for overflow option
info: |
    sec-getoption step 3:
      3. If _value_ is *undefined*, return _fallback_.
    sec-temporal-totemporaloverflow step 1:
      1. Return ? GetOption(_normalizedOptions_, *"overflow"*, « String », « *"constrain"*, *"reject"* », *"constrain"*).
    sec-temporal.calendar.prototype.dateadd step 7:
      7. Let _overflow_ be ? ToTemporalOverflow(_options_).
    sec-temporal-addzoneddatetime step 6:
      6. Let _addedDate_ be ? CalendarDateAdd(_calendar_, _datePart_, _dateDuration_, _options_).
    sec-temporal.zoneddatetime.prototype.add step 7:
      7. Let _epochNanoseconds_ be ? AddZonedDateTime(_zonedDateTime_.[[Nanoseconds]], _timeZone_, _calendar_, _duration_.[[Years]], _duration_.[[Months]], _duration_.[[Weeks]], _duration_.[[Days]], _duration_.[[Hours]], _duration_.[[Minutes]], _duration_.[[Seconds]], _duration_.[[Milliseconds]], _duration_.[[Microseconds]], _duration_.[[Nanoseconds]], _options_).
features: [Temporal]
---*/

const datetime = new Temporal.ZonedDateTime(-1n, "UTC");
const duration = new Temporal.Duration(0, 2);

const explicit = datetime.add(duration, { overflow: undefined });
assert.sameValue(explicit.epochNanoseconds, 5097599_999_999_999n, "default overflow is constrain");
const implicit = datetime.add(duration, {});
assert.sameValue(implicit.epochNanoseconds, 5097599_999_999_999n, "default overflow is constrain");
const lambda = datetime.add(duration, () => {});
assert.sameValue(lambda.epochNanoseconds, 5097599_999_999_999n, "default overflow is constrain");
