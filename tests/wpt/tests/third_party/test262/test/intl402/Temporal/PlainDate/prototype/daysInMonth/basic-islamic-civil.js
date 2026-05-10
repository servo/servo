// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.daysinmonth
description: Days in each month in the islamic-civil calendar
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "islamic-civil";
const options = { overflow: "reject" };

// 1390 = ISO year 1970
const leapYear = 1390;
const commonYear = 1391;

const leapYearDaysInMonth = [
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
  30
];

const commonYearDaysInMonth = [
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
  29
];

const daysInMonth = {};
daysInMonth[leapYear] = leapYearDaysInMonth;
daysInMonth[commonYear] = commonYearDaysInMonth;

for (let year of [leapYear, commonYear]) {
  for (var month = 1; month < 13; month++) {
    const date = Temporal.PlainDate.from({
      year,
      month,
      day: 1,
      calendar
    });
    assert.sameValue(date.daysInMonth, daysInMonth[year][month - 1], `${date}`);
  }
}

