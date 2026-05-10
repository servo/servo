// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.tostring
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
    sec-temporal-balanceisodatetime step 1:
      1. Let _balancedTime_ be ? BalanceTime(_hour_, _minute_, _second_, _millisecond_, _microsecond_, _nanosecond_).
    sec-temporal-builtintimezonegetplaindatetimefor step 3:
      3. Set _result_ to ? BalanceISODateTime(_result_.[[Year]], _result_.[[Month]], _result_.[[Day]], _result_.[[Hour]], _result_.[[Minute]], _result_.[[Second]], _result_.[[Millisecond]], _result_.[[Microsecond]], _result_.[[Nanosecond]] + _offsetNanoseconds_).
    sec-temporal-temporalzoneddatetimetostring step 9:
      9. Let _dateTime_ be ? BuiltinTimeZoneGetPlainDateTimeFor(_timeZone_, _instant_, _isoCalendar_).
    sec-get-temporal.zoneddatetime.prototype.tostring step 9:
      9. Return ? TemporalZonedDateTimeToString(_zonedDateTime_, _precision_.[[Precision]], _showCalendar_, _showTimeZone_, _showOffset_, _precision_.[[Increment]], _precision_.[[Unit]], _roundingMode_).
features: [Temporal]
---*/

// This code path is encountered if the time zone offset is negative and its
// absolute value in nanoseconds is greater than the nanosecond field of the
// ZonedDateTime
const datetime = new Temporal.ZonedDateTime(1001n, "-00:02");

const isoString = datetime.toString();

assert.sameValue(isoString, "1969-12-31T23:58:00.000001001-00:02[-00:02]");
