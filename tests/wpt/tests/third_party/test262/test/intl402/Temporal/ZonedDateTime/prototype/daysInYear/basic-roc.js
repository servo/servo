// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.daysinyear
description: Leap years in the roc calendar
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "roc";
const options = { overflow: "reject" };

const leapYears = [
  61,
  65,
  69,
  73,
  77,
  81,
  85,
  89,
  93,
  97,
  101,
  105,
  109,
  113,
  117,
  121,
  125,
  129,
  133,
  137,
];

for (var year = 59; year < 139; year++) {
    const date = Temporal.ZonedDateTime.from({
        year,
        month: 1,
        calendar, day: 1, hour: 12, minute: 34, timeZone: "UTC"
    });
    assert.sameValue(date.daysInYear, leapYears.includes(year) ? 366 : 365);
}
