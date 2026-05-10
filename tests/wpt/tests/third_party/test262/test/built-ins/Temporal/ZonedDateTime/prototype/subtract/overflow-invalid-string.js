// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.subtract
description: RangeError thrown when overflow option not one of the allowed string values
info: |
    sec-getoption step 10:
      10. If _values_ is not *undefined* and _values_ does not contain an element equal to _value_, throw a *RangeError* exception.
    sec-temporal-totemporaloverflow step 1:
      1. Return ? GetOption(_normalizedOptions_, *"overflow"*, « String », « *"constrain"*, *"reject"* », *"constrain"*).
    sec-temporal.calendar.prototype.dateadd step 7:
      7. Let _overflow_ be ? ToTemporalOverflow(_options_).
    sec-temporal-addzoneddatetime step 6:
      6. Let _addedDate_ be ? CalendarDateAdd(_calendar_, _datePart_, _dateDuration_, _options_).
    sec-temporal.zoneddatetime.prototype.subtract step 7:
      7. Let _epochNanoseconds_ be ? AddZonedDateTime(_zonedDateTime_.[[Nanoseconds]], _timeZone_, _calendar_, −_duration_.[[Years]], −_duration_.[[Months]], −_duration_.[[Weeks]], −_duration_.[[Days]], −_duration_.[[Hours]], −_duration_.[[Minutes]], −_duration_.[[Seconds]], −_duration_.[[Milliseconds]], −_duration_.[[Microseconds]], −_duration_.[[Nanoseconds]], _options_).
features: [Temporal]
---*/

const datetime = new Temporal.ZonedDateTime(1_000_000_000_987_654_321n, "UTC");
const duration = new Temporal.Duration(0, 0, 0, 1);
const badOverflows = ["", "CONSTRAIN", "balance", "other string", "constra\u0131n", "reject\0"];
for (const overflow of badOverflows) {
  assert.throws(
    RangeError,
    () => datetime.subtract(duration, { overflow }),
    `invalid overflow ("${overflow}")`
  );
}
