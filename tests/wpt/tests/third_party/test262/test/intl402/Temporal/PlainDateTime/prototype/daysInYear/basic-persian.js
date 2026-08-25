// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.daysinyear
description: Leap years in the Persian calendar
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "persian";
const options = { overflow: "reject" };

const leapYears = [
  1350,
  1354,
  1358,
  1362,
  1366,
  1370,
  1375,
  1379,
  1383,
  1387,
  1391,
  1395,
  1399,
  1403,
  1408,
  1412,
  1416,
  1420,
  1424,
];

for (var year = 1348; year < 1428; year++) {
    const date = Temporal.PlainDateTime.from({
        year,
        month: 1,
        calendar, day: 1, hour: 12, minute: 34
    });
    assert.sameValue(date.daysInYear, leapYears.includes(year) ? 366 : 365);
}
