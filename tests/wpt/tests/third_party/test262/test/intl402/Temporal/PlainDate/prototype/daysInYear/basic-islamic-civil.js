// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.daysinyear
description: Days in years in the islamic-civil calendar
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "islamic-civil";
const options = { overflow: "reject" };

// 1390 = ISO year 1970
const leapYears = [
  1390,
  1393,
  1396,
  1398,
  1401,
  1404,
  1406,
  1409,
  1412,
  1415,
  1417,
  1420,
  1423,
  1426,
  1428,
  1431,
  1434,
  1436,
  1439,
  1442,
  1445,
  1447,
  1450,
  1453,
  1456,
  1458,
  1461,
  1464,
  1466,
  1469,
]

for (var year = 1390; year < 1470; year++) {
    const date = Temporal.PlainDate.from({
        year,
        month: 1,
        calendar, day: 1
    });
    assert.sameValue(date.daysInYear, leapYears.includes(year) ? 355 : 354);
}
