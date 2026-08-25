// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.until
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
    sec-temporal-differencetime step 8:
      8. Let _bt_ be ? BalanceTime(_hours_, _minutes_, _seconds_, _milliseconds_, _microseconds_, _nanoseconds_).
    sec-temporal-differenceisodatetime step 2:
      2. Let _timeDifference_ be ? DifferenceTime(_h1_, _min1_, _s1_, _ms1_, _mus1_, _ns1_, _h2_, _min2_, _s2_, _ms2_, _mus2_, _ns2_).
    sec-temporal.plaindatetime.prototype.until step 13:
      13. Let _diff_ be ? DifferenceISODateTime(_dateTime_.[[ISOYear]], _dateTime_.[[ISOMonth]], _dateTime_.[[ISODay]], _dateTime_.[[ISOHour]], _dateTime_.[[ISOMinute]], _dateTime_.[[ISOSecond]], _dateTime_.[[ISOMillisecond]], _dateTime_.[[ISOMicrosecond]], _dateTime_.[[ISONanosecond]], _other_.[[ISOYear]], _other_.[[ISOMonth]], _other_.[[ISODay]], _other_.[[ISOHour]], _other_.[[ISOMinute]], _other_.[[ISOSecond]], _other_.[[ISOMillisecond]], _other_.[[ISOMicrosecond]], _other_.[[ISONanosecond]], _dateTime_.[[Calendar]], _largestUnit_, _options_).
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const datetime = new Temporal.PlainDateTime(1996, 5, 2, 1, 1, 1, 1, 1, 1);

const result1 = new Temporal.PlainDateTime(1996, 5, 2, 0, 0, 0, 0, 0, 2).until(datetime);
TemporalHelpers.assertDuration(result1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 999, "nanoseconds balance");

const result2 = new Temporal.PlainDateTime(1996, 5, 2, 0, 0, 0, 0, 2).until(datetime);
TemporalHelpers.assertDuration(result2, 0, 0, 0, 0, 1, 1, 1, 0, 999, 1, "microseconds balance");

const result3 = new Temporal.PlainDateTime(1996, 5, 2, 0, 0, 0, 2).until(datetime);
TemporalHelpers.assertDuration(result3, 0, 0, 0, 0, 1, 1, 0, 999, 1, 1, "milliseconds balance");

const result4 = new Temporal.PlainDateTime(1996, 5, 2, 0, 0, 2).until(datetime);
TemporalHelpers.assertDuration(result4, 0, 0, 0, 0, 1, 0, 59, 1, 1, 1, "seconds balance");

const result5 = new Temporal.PlainDateTime(1996, 5, 2, 0, 2).until(datetime);
TemporalHelpers.assertDuration(result5, 0, 0, 0, 0, 0, 59, 1, 1, 1, 1, "minutes balance");

// This one is different because hours are later balanced again in BalanceDuration
const result6 = new Temporal.PlainDateTime(1996, 5, 2, 2).until(datetime);
TemporalHelpers.assertDuration(result6, 0, 0, 0, 0, 0, -58, -58, -998, -998, -999, "hours balance");
