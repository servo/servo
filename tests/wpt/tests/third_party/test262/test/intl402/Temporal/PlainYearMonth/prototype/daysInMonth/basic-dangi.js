// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.daysinmonth
description: Days in each month in the Dangi calendar
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "dangi";
const options = { overflow: "reject" };

// 1971 is a common year; 1972 is a leap year

const sampleYears = {
  1971: [
    29,
    30,
    29,
    29,
    30,
    29,
    30,
    29,
    30,
    30,
    30,
    29,
  ],
  1972: [
    29,
    30,
    29,
    29,
    30,
    29,
    30,
    29,
    30,
    30,
    30,
    29,
  ]
};

for (var [year, daysInMonth] of Object.entries(sampleYears)) {
  for (var month = 1; month < daysInMonth.length; month++) {
    const date = Temporal.PlainYearMonth.from({
      year,
      month,
      calendar
    });
    assert.sameValue(date.daysInMonth, daysInMonth[month - 1], `${date}`);
  }
}
