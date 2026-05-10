// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.from
description: PlainMonthDay can be created for all month codes (M01-M12) in ROC calendar
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

// Test that all month codes M01-M12 are valid for the ROC calendar

const calendar = "roc";

for (const { month, monthCode, daysInMonth } of TemporalHelpers.ISOMonths) {
  // Test creation with monthCode and day 1
  const pmd = Temporal.PlainMonthDay.from({ calendar, monthCode, day: 1 });
  TemporalHelpers.assertPlainMonthDay(pmd, monthCode, 1, `monthCode ${monthCode} should be preserved`);

  // Test creation with month and day 1
  const pmdMonth = Temporal.PlainMonthDay.from({ calendar, year: 61, month, day: 1 });
  TemporalHelpers.assertPlainMonthDay(pmdMonth, monthCode, 1, `Equivalent monthCode ${monthCode} and month ${month} are resolved to the same PlainMonthDay`);

  // Test with maximum day value for this month (minimum for PlainMonthDay)
  const pmdMax = Temporal.PlainMonthDay.from({ calendar, monthCode, day: daysInMonth });
  TemporalHelpers.assertPlainMonthDay(pmdMax, monthCode, daysInMonth, `${monthCode} with day ${daysInMonth} should be valid`);

  // Test constrain overflow
  const constrained = Temporal.PlainMonthDay.from(
    { calendar, monthCode, day: daysInMonth + 1 },
    { overflow: "constrain" }
  );
  TemporalHelpers.assertPlainMonthDay(constrained, monthCode, daysInMonth, `day ${daysInMonth + 1} should be constrained to ${daysInMonth} for ${monthCode}`);

  // Test reject overflow
  assert.throws(RangeError, () => {
    Temporal.PlainMonthDay.from({ calendar, monthCode, day: daysInMonth + 1 }, { overflow: "reject" });
  }, `${monthCode} with day ${daysInMonth + 1} should throw with reject overflow`);
}
