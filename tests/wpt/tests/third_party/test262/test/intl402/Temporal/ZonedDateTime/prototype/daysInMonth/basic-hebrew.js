// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.daysinmonth
description: Days in each month in the Hebrew calendar
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "hebrew";
const options = { overflow: "reject" };

// 5732 = ISO year 1972

const sampleYears = {
  // Deficient leap year
  5730: [
    30,
    29,
    29,
    29,
    30,
    30,
    29,
    30,
    29,
    30,
    29,
    30,
    29,
  ],
  // Complete common year
  5732: [
    30,
    30,
    30,
    29,
    30,
    29,
    30,
    29,
    30,
    29,
    30,
    29,
  ],
  // Regular common year
  5778: [
    30,
    29,
    30,
    29,
    30,
    29,
    30,
    29,
    30,
    29,
    30,
    29,
  ],
  // Complete leap year
  5779: [
    30,
    30,
    30,
    29,
    30,
    30,
    29,
    30,
    29,
    30,
    29,
    30,
    29,
  ],
  // Deficient common year
  5781: [
    30,
    29,
    29,
    29,
    30,
    29,
    30,
    29,
    30,
    29,
    30,
    29,
  ],
  // Regular leap year
  5782: [
    30,
    29,
    30,
    29,
    30,
    30,
    29,
    30,
    29,
    30,
    29,
    30,
    29,
  ],
};

for (var [year, daysInMonth] of Object.entries(sampleYears)) {
  for (var month = 1; month < daysInMonth.length; month++) {
    const date = Temporal.ZonedDateTime.from({
      year,
      month,
      day: 1,
      calendar, hour: 12, minute: 34, timeZone: "UTC"
    });
    assert.sameValue(date.daysInMonth, daysInMonth[month - 1], `${date}`);
  }
}

