// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.daysinmonth
description: Days in each month in the Coptic calendar
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "coptic";
const options = { overflow: "reject" };

// 1687 = ISO year 1971
// 1687 is a leap year; 1688 is a common year

const leapYear = 1687;
const commonYear = 1688;

for (var year of [leapYear, commonYear]) {
  for (var month = 1; month < 14; month++) {
    const date = Temporal.PlainDate.from({
      year,
      month,
      day: 1,
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

