// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.from
description: PlainMonthDay can be created for all regular month codes (M01-M12) in Chinese calendar
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

// Test that all regular month codes M01-M12 are valid for the Chinese calendar
// The Chinese calendar is a lunisolar calendar, so months vary in length
// Leap months (M01L-M12L) are tested elsewhere

const calendar = "chinese";
const monthCodesWithYears = [
  { monthCode: "M01", referenceYear30: 1970 },
  { monthCode: "M02", referenceYear30: 1972 },
  { monthCode: "M03", referenceYear30: 1966 },
  { monthCode: "M04", referenceYear30: 1970 },
  { monthCode: "M05", referenceYear30: 1972 },
  { monthCode: "M06", referenceYear30: 1971 },
  { monthCode: "M07", referenceYear30: 1972 },
  { monthCode: "M08", referenceYear30: 1971 },
  { monthCode: "M09", referenceYear30: 1972 },
  { monthCode: "M10", referenceYear30: 1972 },
  { monthCode: "M11", referenceYear30: 1970 },
  { monthCode: "M12", referenceYear30: 1972 }
];

for (const { monthCode, referenceYear30 } of monthCodesWithYears) {
  // Test creation with monthCode
  // The reference year is 1972 for days that occur every year
  const pmd = Temporal.PlainMonthDay.from({ calendar, monthCode, day: 1 });
  TemporalHelpers.assertPlainMonthDay(pmd, monthCode, 1, `${monthCode}-01`, 1972);

  // Test with day 29 (every month has at least 29 days every year)
  const pmd29 = Temporal.PlainMonthDay.from({ calendar, monthCode, day: 29 });
  TemporalHelpers.assertPlainMonthDay(pmd29, monthCode, 29, `${monthCode}-29`, 1972);

  // Test with day 30 (months can have 29 or 30 days)
  const pmd30 = Temporal.PlainMonthDay.from({ calendar, monthCode, day: 30 });
  TemporalHelpers.assertPlainMonthDay(pmd30, monthCode, 30, `${monthCode}-30`, referenceYear30);

  // Test constrain overflow - Chinese months vary from 29-30 days
  const constrained = Temporal.PlainMonthDay.from({ calendar, monthCode, day: 31 });
  TemporalHelpers.assertPlainMonthDay(constrained, monthCode, 30, `${monthCode}-31 constrained`, referenceYear30);

  // Test reject overflow for day 31
  assert.throws(RangeError, () => {
    Temporal.PlainMonthDay.from({ calendar, monthCode, day: 31 }, { overflow: "reject" });
  }, `${monthCode} with day 31 should throw with reject overflow`);
}
