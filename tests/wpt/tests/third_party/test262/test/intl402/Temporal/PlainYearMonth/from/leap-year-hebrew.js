// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.from
description: Basic functionality of resolving fields in hebrew calendar leap year
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

const calendar = "hebrew";
const options = { overflow: "reject" };

const leapYear = 5784;
const monthCodes5784 = [
  undefined,
  "M01",
  "M02",
  "M03",
  "M04",
  "M05",
  "M05L",
  "M06",
  "M07",
  "M08",
  "M09",
  "M10",
  "M11",
  "M12"
];

for (let month = 1; month < 14; month++) {
  const monthCode = monthCodes5784[month];

  const instance = Temporal.PlainYearMonth.from({ year: leapYear, month, calendar }, options);
  TemporalHelpers.assertPlainYearMonth(
    instance,
    leapYear, month, monthCode,
    `month ${monthCode} in leap year`,
    "am", leapYear, null
  );
}

