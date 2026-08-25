// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.inleapyear
description: Leap years in the Coptic calendar
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "coptic";
const options = { overflow: "reject" };

// 1686 = ISO year 1970
const leapYears = [
  1687,
  1691,
  1695,
  1699,
  1703,
  1707,
  1711,
  1715,
  1719,
  1723,
  1727,
  1731,
  1735,
  1739,
  1743,
  1747,
  1751,
  1755,
  1759,
  1763,
];

for (var year = 1686; year < 1766; year++) {
    const date = Temporal.PlainYearMonth.from({
        year,
        month: 1,
        calendar
    });
    assert.sameValue(date.inLeapYear, leapYears.includes(year));
}
