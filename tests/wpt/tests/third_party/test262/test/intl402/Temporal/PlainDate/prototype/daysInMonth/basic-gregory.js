// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.daysinmonth
description: Days in each month in the gregory calendar
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "gregory";
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
    31,
  ]
};

for (var [year, daysInMonth] of Object.entries(sampleYears)) {
  for (var month = 1; month < 13; month++) {
    const date = Temporal.PlainDate.from({
      year,
      month,
      day: 1,
      calendar
    });
    assert.sameValue(date.daysInMonth, daysInMonth[month - 1], `${date}`);
  }
}

