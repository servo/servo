// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.daysinmonth
description: Days in each month in the ethioaa calendar
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "ethioaa";
const options = { overflow: "reject" };

// 7463 = ISO year 1971
// 7463 is a leap year; 7464 is a common year

const leapYear = 7463;
const commonYear = 7464;

for (var year of [leapYear, commonYear]) {
  for (var month = 1; month < 14; month++) {
    const date = Temporal.PlainYearMonth.from({
      year,
      month,
      calendar
    });
    if (month !== 13)
      assert.sameValue(date.daysInMonth, 30, `${date}`);
    else if (year == leapYear)
      assert.sameValue(date.daysInMonth, 6, `${date}`);
    else
      assert.sameValue(date.daysInMonth, 5, `${date}`);
  }
}
