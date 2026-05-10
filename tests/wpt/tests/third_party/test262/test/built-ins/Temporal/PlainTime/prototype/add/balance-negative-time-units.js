// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.prototype.add
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
    sec-temporal.plaintime.prototype.add step 4:
      4. Let _result_ be ? AddTime(_temporalTime_.[[ISOHour]], _temporalTime_.[[ISOMinute]], _temporalTime_.[[ISOSecond]], _temporalTime_.[[ISOMillisecond]], _temporalTime_.[[ISOMicrosecond]], _temporalTime_.[[ISONanosecond]], _duration_.[[Hours]], _duration_.[[Minutes]], _duration_.[[Seconds]], _duration_.[[Milliseconds]], _duration_.[[Microseconds]], _duration_.[[Nanoseconds]]).
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const time = new Temporal.PlainTime(1, 1, 1, 1, 1, 1);

const result1 = time.add(new Temporal.Duration(0, 0, 0, 0, 0, 0, 0, 0, 0, -2));
TemporalHelpers.assertPlainTime(result1, 1, 1, 1, 1, 0, 999, "nanoseconds balance");

const result2 = time.add(new Temporal.Duration(0, 0, 0, 0, 0, 0, 0, 0, -2));
TemporalHelpers.assertPlainTime(result2, 1, 1, 1, 0, 999, 1, "microseconds balance");

const result3 = time.add(new Temporal.Duration(0, 0, 0, 0, 0, 0, 0, -2));
TemporalHelpers.assertPlainTime(result3, 1, 1, 0, 999, 1, 1, "milliseconds balance");

const result4 = time.add(new Temporal.Duration(0, 0, 0, 0, 0, 0, -2));
TemporalHelpers.assertPlainTime(result4, 1, 0, 59, 1, 1, 1, "seconds balance");

const result5 = time.add(new Temporal.Duration(0, 0, 0, 0, 0, -2));
TemporalHelpers.assertPlainTime(result5, 0, 59, 1, 1, 1, 1, "minutes balance");

const result6 = time.add(new Temporal.Duration(0, 0, 0, 0, -2));
TemporalHelpers.assertPlainTime(result6, 23, 1, 1, 1, 1, 1, "hours mod 24");
