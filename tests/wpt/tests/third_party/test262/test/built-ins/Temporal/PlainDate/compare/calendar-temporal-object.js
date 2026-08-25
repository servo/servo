// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.compare
description: Fast path for converting other Temporal objects to Temporal.Calendar by reading internal slots
info: |
    sec-temporal.plaindate.compare steps 1–2:
      1. Set _one_ to ? ToTemporalDate(_one_).
      2. Set _two_ to ? ToTemporalDate(_two_).
    sec-temporal-totemporaldate step 2.c:
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
  Temporal.PlainDate.compare(
    { year: 2000, month: 5, day: 2, calendar: temporalObject },
    { year: 2001, month: 6, day: 3, calendar: temporalObject },
  );
});
