// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.inleapyear
description: Leap years in the Chinese calendar
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "chinese";
const options = { overflow: "reject" };

const leapYears = [
  1971,
  1974,
  1976,
  1979,
  1982,
  1984,
  1987,
  1990,
  1993,
  1995,
  1998,
  2001,
  2004,
  2006,
  2009,
  2012,
  2014,
  2017,
  2020,
  2023,
  2025,
  2028,
  2031,
  2033,
  2036,
  2039,
  2042,
  2044,
  2047,
];

for (var year = 1970; year < 2050; year++) {
    const date = Temporal.PlainDateTime.from({
        year,
        month: 1,
        calendar, day: 1, hour: 12, minute: 34
    });
    assert.sameValue(date.inLeapYear, leapYears.includes(year));
}
