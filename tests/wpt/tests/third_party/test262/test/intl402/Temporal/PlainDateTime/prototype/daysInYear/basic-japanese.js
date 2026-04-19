// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.daysinyear
description: Leap years in the Japanese calendar
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "japanese";
const options = { overflow: "reject" };

const leapYears = [
  1972,
  1976,
  1980,
  1984,
  1988,
  1992,
  1996,
  2000,
  2004,
  2008,
  2012,
  2016,
  2020,
  2024,
  2028,
  2032,
  2036,
  2040,
  2044,
  2048,
];

for (var year = 1970; year < 2050; year++) {
    const date = Temporal.PlainDateTime.from({
        year,
        month: 1,
        calendar, day: 1, hour: 12, minute: 34
    });
    assert.sameValue(date.daysInYear, leapYears.includes(year) ? 366 : 365);
}
