// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.equals
description: Fast path for converting other Temporal objects to Temporal.Calendar by reading internal slots
info: |
    sec-temporal.plaindatetime.prototype.equals step 3:
      3. Set _other_ to ? ToTemporalDateTime(_other_).
    sec-temporal-totemporaldatetime step 2.c:
      c. Let _calendar_ be ? GetTemporalCalendarWithISODefault(_item_).
    sec-temporal-gettemporalcalendarwithisodefault step 2:
      2. Return ? ToTemporalCalendarWithISODefault(_calendar_).
    sec-temporal-totemporalcalendarwithisodefault step 2:
      3. Return ? ToTemporalCalendar(_temporalCalendarLike_).
    sec-temporal-totemporalcalendar step 1.a:
      a. If _temporalCalendarLike_ has an [[InitializedTemporalDate]], [[InitializedTemporalDateTime]], [[InitializedTemporalMonthDay]], [[InitializedTemporalYearMonth]], or [[InitializedTemporalZonedDateTime]] internal slot, then
        i. Return _temporalCalendarLike_.[[Calendar]].
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

TemporalHelpers.checkToTemporalCalendarFastPath((temporalObject) => {
  const datetime = new Temporal.PlainDateTime(2000, 5, 2, 12, 34, 56, 987, 654, 321);
  datetime.equals({ year: 2005, month: 6, day: 2, calendar: temporalObject });
});
