// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.daysinyear
description: Leap years in the ethiopic calendar
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "ethiopic";
const options = { overflow: "reject" };

// 7462 = ISO year 1970
const leapYears = [
  7463,
  7467,
  7471,
  7475,
  7479,
  7483,
  7487,
  7491,
  7495,
  7499,
  7503,
  7507,
  7511,
  7515,
  7519,
  7523,
  7527,
  7531,
  7535,
  7539,
];

for (var year = 7462; year < 7542; year++) {
    const date = Temporal.PlainDate.from({
        year,
        month: 1,
        calendar, day: 1
    });
    assert.sameValue(date.daysInYear, leapYears.includes(year) ? 366 : 365);
}
