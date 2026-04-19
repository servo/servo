// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.daysinmonth
description: Days in each month in the islamic-umalqura calendar
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "islamic-umalqura";
const options = { overflow: "reject" };

// 1390 = ISO year 1970
const sampleYears = {
  1390: [
    29,
    30,
    29,
    30,
    30,
    30,
    29,
    30,
    29,
    30,
    29,
    30,
  ],
  1391: [
    29,
    29,
    30,
    29,
    30,
    30,
    29,
    30,
    30,
    29,
    30,
    29
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

