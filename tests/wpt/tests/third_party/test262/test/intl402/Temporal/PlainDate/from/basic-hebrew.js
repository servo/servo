// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.from
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

  const startOfMonth = Temporal.PlainDate.from({ year: commonYear, month, day: 1, calendar }, options);
  TemporalHelpers.assertPlainDate(
    startOfMonth,
    commonYear, month, monthCode, 1,
    `Start of month ${monthCode} in common year`,
    "am", commonYear
  );

  const day = monthLengths5783[month];
  const endOfMonth = Temporal.PlainDate.from({ year: commonYear, month, day, calendar }, options);
  TemporalHelpers.assertPlainDate(
    endOfMonth,
    commonYear, month, monthCode, day,
    `End of month ${monthCode} in common year`,
    "am", commonYear
  );
}

