// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.microsecond
description: Negative time fields are balanced upwards
info: |
    sec-temporal-balancetime steps 3–4:
      3. Set _microsecond_ to _microsecond_ + floor(_nanosecond_ / 1000).
      4. Set _nanosecond_ to _nanosecond_ modulo 1000.
    sec-temporal-balanceisodatetime step 1:
      1. Let _balancedTime_ be ? BalanceTime(_hour_, _minute_, _second_, _millisecond_, _microsecond_, _nanosecond_).
    sec-temporal-builtintimezonegetplaindatetimefor step 3:
      3. Set _result_ to ? BalanceISODateTime(_result_.[[Year]], _result_.[[Month]], _result_.[[Day]], _result_.[[Hour]], _result_.[[Minute]], _result_.[[Second]], _result_.[[Millisecond]], _result_.[[Microsecond]], _result_.[[Nanosecond]] + _offsetNanoseconds_).
    sec-get-temporal.zoneddatetime.prototype.microsecond step 6:
      6. Let _temporalDateTime_ be ? BuiltinTimeZoneGetPlainDateTimeFor(_timeZone_, _instant_, _calendar_).
features: [Temporal]
---*/

// This code path is encountered if the time zone offset is negative and its
// absolute value in nanoseconds is greater than the nanosecond field of the
// exact time's epoch parts
const datetime = new Temporal.ZonedDateTime(1001n, "-00:02");

assert.sameValue(datetime.minute, 58);
assert.sameValue(datetime.microsecond, 1);
assert.sameValue(datetime.nanosecond, 1);
