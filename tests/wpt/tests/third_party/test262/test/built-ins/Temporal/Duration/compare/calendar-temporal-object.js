// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.compare
description: Fast path for converting other Temporal objects to Temporal.Calendar by reading internal slots
info: |
    sec-temporal.duration.compare step 4:
      4. Let _relativeTo_ be ? ToRelativeTemporalObject(_options_).
    sec-temporal-torelativetemporalobject step 4.b:
      b. Let _calendar_ be ? GetTemporalCalendarWithISODefault(_item_).
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
  const duration1 = new Temporal.Duration(1);
  const duration2 = new Temporal.Duration(0, 12);
  Temporal.Duration.compare(duration1, duration2, { relativeTo: { year: 2000, month: 1, day: 1, calendar: temporalObject } });
});
