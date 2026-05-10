// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.daysinmonth
description: Days in each month in the Japanese calendar
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "japanese";
const options = { overflow: "reject" };

// 1972 is a leap year; 1973 is a common year

const sampleYears = {
  1972: [
    31,
    29,
    31,
    30,
    31,
    30,
    31,
    31,
    30,
    31,
    30,
    31,
  ],
  1973: [
    31,
    28,
    31,
    30,
    31,
    30,
    31,
    31,
    30,
    31,
    30,
    31,
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
