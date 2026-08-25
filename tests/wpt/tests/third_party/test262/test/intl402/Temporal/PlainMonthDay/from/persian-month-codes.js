// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.from
description: PlainMonthDay can be created for all month codes (M01-M12) in Persian calendar
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

// Test that all month codes M01-M12 are valid for the Persian calendar
// Persian calendar month lengths:
// - M01-M06: 31 days
// - M07-M12: 30 days
// All days have reference year 1972, including the leap day M12-30

const calendar = "persian";

// Months with 31 days
const monthsWith31Days = ["M01", "M02", "M03", "M04", "M05", "M06"];

for (const monthCode of monthsWith31Days) {
  const pmd = Temporal.PlainMonthDay.from({ calendar, monthCode, day: 1 });
  TemporalHelpers.assertPlainMonthDay(pmd, monthCode, 1, `${monthCode}-01`);

  const pmd31 = Temporal.PlainMonthDay.from({ calendar, monthCode, day: 31 });
  TemporalHelpers.assertPlainMonthDay(pmd31, monthCode, 31, `${monthCode}-31`);

  const constrained = Temporal.PlainMonthDay.from(
    { calendar, monthCode, day: 32 },
    { overflow: "constrain" }
  );
  TemporalHelpers.assertPlainMonthDay(constrained, monthCode, 31, `day 32 should be constrained to 31 for ${monthCode}`);

  assert.throws(RangeError, () => {
    Temporal.PlainMonthDay.from({ calendar, monthCode, day: 32 }, { overflow: "reject" });
  }, `${monthCode} with day 32 should throw with reject overflow`);
}

// Months with 30 days
const monthsWith30Days = ["M07", "M08", "M09", "M10", "M11", "M12"];

for (const monthCode of monthsWith30Days) {
  const pmd = Temporal.PlainMonthDay.from({ calendar, monthCode, day: 1 });
  TemporalHelpers.assertPlainMonthDay(pmd, monthCode, 1, `${monthCode}-01`);

  const pmd30 = Temporal.PlainMonthDay.from({ calendar, monthCode, day: 30 });
  TemporalHelpers.assertPlainMonthDay(pmd30, monthCode, 30, `${monthCode}-31`);

  const constrained = Temporal.PlainMonthDay.from(
    { calendar, monthCode, day: 31 },
    { overflow: "constrain" }
  );
  TemporalHelpers.assertPlainMonthDay(constrained, monthCode, 30, `day 31 should be constrained to 30 for ${monthCode}`);

  assert.throws(RangeError, () => {
    Temporal.PlainMonthDay.from({ calendar, monthCode, day: 31 }, { overflow: "reject" });
  }, `${monthCode} with day 31 should throw with reject overflow`);
}
