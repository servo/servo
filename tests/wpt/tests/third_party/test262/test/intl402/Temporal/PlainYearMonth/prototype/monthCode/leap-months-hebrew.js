// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.monthcode
description: Hebrew calendar has leap months
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "hebrew";

const commonYearMonthCodes = ["M01", "M02", "M03", "M04", "M05", "M06", "M07", "M08", "M09", "M10", "M11", "M12"];
const leapYearMonthCodes = ["M01", "M02", "M03", "M04", "M05", "M05L", "M06", "M07", "M08", "M09", "M10", "M11", "M12"];

for (var year = 5730; year < 5735; year++) {
  const monthsInYear = Temporal.PlainYearMonth.from({
    year,
    month: 1,
    calendar
  }).monthsInYear;
  for (var month = 1; month < monthsInYear; month++) {
    const date = Temporal.PlainYearMonth.from({
        year,
        month,
        calendar
    });
    if (date.inLeapYear)
      assert.sameValue(date.monthCode, leapYearMonthCodes[month - 1]);
    else
      assert.sameValue(date.monthCode, commonYearMonthCodes[month - 1]);
  }
}
