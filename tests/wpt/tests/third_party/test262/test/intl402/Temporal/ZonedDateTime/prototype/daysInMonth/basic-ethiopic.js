// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.daysinmonth
description: Days in each month in the ethiopic calendar
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "ethiopic";
const options = { overflow: "reject" };

// 1963 = ISO year 1971
// 1963 is a leap year; 1964 is a common year

const leapYear = 1963;
const commonYear = 1964;

for (var year of [leapYear, commonYear]) {
  for (var month = 1; month < 14; month++) {
    const date = Temporal.ZonedDateTime.from({
      year,
      month,
      day: 1,
      calendar, hour: 12, minute: 34, timeZone: "UTC"
    });
    if (month !== 13)
      assert.sameValue(date.daysInMonth, 30, `${date}`);
    else if (year == leapYear)
      assert.sameValue(date.daysInMonth, 6, `${date}`);
    else
      assert.sameValue(date.daysInMonth, 5, `${date}`);
  }
}

