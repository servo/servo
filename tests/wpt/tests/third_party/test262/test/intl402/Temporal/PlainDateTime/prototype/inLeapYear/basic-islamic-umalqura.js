// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.inleapyear
description: Leap years in the islamic-umalqura calendar
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "islamic-umalqura";
const options = { overflow: "reject" };

// 1390 = ISO year 1970
const leapYears = [
  1390,
  1392,
  1397,
  1399,
  1403,
  1405,
  1406,
  1411,
  1412,
  1414,
  1418,
  1420,
  1425,
  1426,
  1428,
  1433,
  1435,
  1439,
  1441,
  1443,
  1447,
  1448,
  1451,
  1454,
  1455,
  1457,
  1462,
  1463,
  1467,
  1469
]

for (var year = 1390; year < 1470; year++) {
    const date = Temporal.PlainDateTime.from({
        year,
        month: 1,
        calendar, day: 1, hour: 12, minute: 34
    });
    assert.sameValue(date.inLeapYear, leapYears.includes(year));
}
