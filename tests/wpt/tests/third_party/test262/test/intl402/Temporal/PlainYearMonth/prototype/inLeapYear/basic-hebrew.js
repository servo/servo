// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.inleapyear
description: Leap years in the Hebrew calendar
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "hebrew";
const options = { overflow: "reject" };

const leapYears = [
  5730,
  5733,
  5736,
  5738,
  5741,
  5744,
  5746,
  5749,
  5752,
  5755,
  5757,
  5760,
  5763,
  5765,
  5768,
  5771,
  5774,
  5776,
  5779,
  5782,
  5784,
  5787,
  5790,
  5793,
  5795,
  5798,
  5801,
  5803,
  5806,
  5809,
];

for (var year = 5730; year < 5810; year++) {
    const date = Temporal.PlainYearMonth.from({
        year,
        month: 1,
        calendar
    });
    assert.sameValue(date.inLeapYear, leapYears.includes(year));
}
