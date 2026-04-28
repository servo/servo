// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.year
description: >
  Determination of the epoch year for arithmetic years for all non-ISO8601
  calendars
info: |
  4.1.12 CalendarDateArithmeticYear ( _calendar_, _date_ )

  1. Let _r_ be the row in Table 4 which the value of the Calendar column is
     _calendar_.
  2. Let _epochYear_ be the value given in the "Epoch ISO Year" column of _r_.
  3. Let _epochDate_ be the first day of the calendar year starting in ISO year
     epochYear in the calendar represented by _calendar_, according to
     implementation-defined processing.
  4. Let _newYear_ be the first day of the calendar year of _date_ in the
     calendar represented by _calendar_, according to implementation-defined
     processing.
  5. Let _arithmeticYear_ be the number of whole years between _epochDate_ and
     _newYear_ in the calendar represented by _calendar_, according to
     implementation-defined processing.
  6. Return _arithmeticYear_.
features: [Temporal, Intl.Era-monthcode]
---*/

const epochYears = {
  buddhist: -543,
  // Chinese calendar omitted, in order to avoid creating an instance outside
  // the well-defined range
  coptic: 283,
  // Dangi calendar omitted, see above
  ethioaa: -5493,
  ethiopic: 7,
  gregory: 0,
  hebrew: -3761,
  indian: 78,
  "islamic-civil": 621,
  "islamic-tbla": 621,
  "islamic-umalqura": 621,
  japanese: 0,
  persian: 621,
  roc: 1911,
};

for (const [calendar, epochYear] of Object.entries(epochYears)) {
  const epochDate = new Temporal.PlainDate(epochYear, 12, 31)
    .withCalendar(calendar)
    .with({ monthCode: "M01", day: 1 });

  assert.sameValue(epochDate.year, 0, `${calendar} arithmetic year 0 should start in ISO year ${epochYear}`);
}
