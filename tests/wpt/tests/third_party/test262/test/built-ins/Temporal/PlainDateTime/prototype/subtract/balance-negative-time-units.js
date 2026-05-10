// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.subtract
description: Negative time fields are balanced upwards
info: |
    sec-temporal-balancetime steps 3–14:
      3. Set _microsecond_ to _microsecond_ + floor(_nanosecond_ / 1000).
      4. Set _nanosecond_ to _nanosecond_ modulo 1000.
      5. Set _millisecond_ to _millisecond_ + floor(_microsecond_ / 1000).
      6. Set _microsecond_ to _microsecond_ modulo 1000.
      7. Set _second_ to _second_ + floor(_millisecond_ / 1000).
      8. Set _millisecond_ to _millisecond_ modulo 1000.
      9. Set _minute_ to _minute_ + floor(_second_ / 60).
      10. Set _second_ to _second_ modulo 60.
      11. Set _hour_ to _hour_ + floor(_minute_ / 60).
      12. Set _minute_ to _minute_ modulo 60.
      13. Let _days_ be floor(_hour_ / 24).
      14. Set _hour_ to _hour_ modulo 24.
    sec-temporal-addtime step 8:
      8. Return ? BalanceTime(_hour_, _minute_, _second_, _millisecond_, _microsecond_, _nanosecond_).
    sec-temporal-adddatetime step 1:
      1. Let _timeResult_ be ? AddTime(_hour_, _minute_, _second_, _millisecond_, _microsecond_, _nanosecond_, _hours_, _minutes_, _seconds_, _milliseconds_, _microseconds_, _nanoseconds_).
    sec-temporal.plaindatetime.prototype.subtract step 5:
      5. Let _result_ be ? AddDateTime(_dateTime_.[[ISOYear]], _dateTime_.[[ISOMonth]], _dateTime_.[[ISODay]], _dateTime_.[[ISOHour]], _dateTime_.[[ISOMinute]], _dateTime_.[[ISOSecond]], _dateTime_.[[ISOMillisecond]], _dateTime_.[[ISOMicrosecond]], _dateTime_.[[ISONanosecond]], _dateTime_.[[Calendar]], −_duration_.[[Years]], −_duration_.[[Months]], −_duration_.[[Weeks]], −_duration_.[[Days]], −_duration_.[[Hours]], −_duration_.[[Minutes]], −_duration_.[[Seconds]], −_duration_.[[Milliseconds]], −_duration_.[[Microseconds]], −_duration_.[[Nanoseconds]], _options_).
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const datetime = new Temporal.PlainDateTime(1996, 5, 2, 1, 1, 1, 1, 1, 1);

const result1 = datetime.subtract(new Temporal.Duration(0, 0, 0, 0, 0, 0, 0, 0, 0, 2));
TemporalHelpers.assertPlainDateTime(result1, 1996, 5, "M05", 2, 1, 1, 1, 1, 0, 999, "nanoseconds balance");

const result2 = datetime.subtract(new Temporal.Duration(0, 0, 0, 0, 0, 0, 0, 0, 2));
TemporalHelpers.assertPlainDateTime(result2, 1996, 5, "M05", 2, 1, 1, 1, 0, 999, 1, "microseconds balance");

const result3 = datetime.subtract(new Temporal.Duration(0, 0, 0, 0, 0, 0, 0, 2));
TemporalHelpers.assertPlainDateTime(result3, 1996, 5, "M05", 2, 1, 1, 0, 999, 1, 1, "milliseconds balance");

const result4 = datetime.subtract(new Temporal.Duration(0, 0, 0, 0, 0, 0, 2));
TemporalHelpers.assertPlainDateTime(result4, 1996, 5, "M05", 2, 1, 0, 59, 1, 1, 1, "seconds balance");

const result5 = datetime.subtract(new Temporal.Duration(0, 0, 0, 0, 0, 2));
TemporalHelpers.assertPlainDateTime(result5, 1996, 5, "M05", 2, 0, 59, 1, 1, 1, 1, "minutes balance");

const result6 = datetime.subtract(new Temporal.Duration(0, 0, 0, 0, 2));
TemporalHelpers.assertPlainDateTime(result6, 1996, 5, "M05", 1, 23, 1, 1, 1, 1, 1, "hours balance");
