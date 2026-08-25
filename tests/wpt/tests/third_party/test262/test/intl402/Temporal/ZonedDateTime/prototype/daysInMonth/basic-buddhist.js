// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.daysinmonth
description: Days in each month in the Buddhist calendar
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "buddhist";
const options = { overflow: "reject" };

// 2515 = ISO year 1972
// 2515 is a leap year; 2516 is a common year

const sampleYears = {
  2515: [
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
  2516: [
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
  for (var month = 1; month < 13; month++) {
    const date = Temporal.ZonedDateTime.from({
      year,
      month,
      day: 1,
      calendar, hour: 12, minute: 34, timeZone: "UTC"
    });
    assert.sameValue(date.daysInMonth, daysInMonth[month - 1], `${date}`);
  }
}

