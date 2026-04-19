// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.from
description: Basic functionality of resolving fields in hebrew calendar
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

const calendar = "hebrew";
const options = { overflow: "reject" };

const commonYear = 5783;
const monthLengths5783 = [undefined, 30, 30, 30, 29, 30, 29, 30, 29, 30, 29, 30, 29];

for (let month = 1; month < 13; month++) {
  const monthCode = `M${String(month).padStart(2, '0')}`;

  const startOfMonth = Temporal.PlainDateTime.from({ year: commonYear, month, day: 1, hour: 12, minute: 34, calendar }, options);
  TemporalHelpers.assertPlainDateTime(
    startOfMonth,
    commonYear, month, monthCode, 1, 12, 34, 0, 0, 0, 0,
    `Start of month ${monthCode} in common year`,
    "am", commonYear
  );

  const day = monthLengths5783[month];
  const endOfMonth = Temporal.PlainDateTime.from({ year: commonYear, month, day, hour: 12, minute: 34, calendar }, options);
  TemporalHelpers.assertPlainDateTime(
    endOfMonth,
    commonYear, month, monthCode, day, 12, 34, 0, 0, 0, 0,
    `End of month ${monthCode} in common year`,
    "am", commonYear
  );
}

