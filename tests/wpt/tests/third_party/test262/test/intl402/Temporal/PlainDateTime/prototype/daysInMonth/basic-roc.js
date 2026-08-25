// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.daysinmonth
description: Days in each month in the roc calendar
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "roc";
const options = { overflow: "reject" };

// 61 is a leap year; 62 is a common year

const sampleYears = {
  61: [
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
  62: [
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
    const date = Temporal.PlainDateTime.from({
      year,
      month,
      day: 1,
      calendar, hour: 12, minute: 34
    });
    assert.sameValue(date.daysInMonth, daysInMonth[month - 1], `${date}`);
  }
}

