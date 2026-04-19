// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.from
description: PlainMonthDay can be created for all regular month codes and leap month in Hebrew calendar
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

// Test that all regular month codes M01-M12 and leap month M05L are valid for the Hebrew calendar

const calendar = "hebrew";
const commonMonthCodes = [
  { monthCode: "M01", has30: true },
  { monthCode: "M02", has30: true, referenceYear30: 1971 },
  { monthCode: "M03", has30: true, referenceYear30: 1971 },
  { monthCode: "M04" },
  { monthCode: "M05", has30: true },
  { monthCode: "M06" },
  { monthCode: "M07", has30: true },
  { monthCode: "M08" },
  { monthCode: "M09", has30: true },
  { monthCode: "M10" },
  { monthCode: "M11", has30: true },
  { monthCode: "M12" },
];

for (const { monthCode, has30 = false, referenceYear = 1972, referenceYear30 = referenceYear } of commonMonthCodes) {
  const pmd = Temporal.PlainMonthDay.from({ calendar, monthCode, day: 1 });
  TemporalHelpers.assertPlainMonthDay(pmd, monthCode, 1, `${monthCode}-01`, referenceYear);

  const pmd29 = Temporal.PlainMonthDay.from({ calendar, monthCode, day: 29 });
  TemporalHelpers.assertPlainMonthDay(pmd29, monthCode, 29, `${monthCode}-29`, referenceYear);

  const pmd30 = Temporal.PlainMonthDay.from(
    { calendar, monthCode, day: 30 },
    { overflow: "constrain" }
  );
  const constrained = Temporal.PlainMonthDay.from(
    { calendar, monthCode, day: 31 },
    { overflow: "constrain" }
  );
  if (has30) {
    TemporalHelpers.assertPlainMonthDay(pmd30, monthCode, 30, `${monthCode}-30`, referenceYear30);
    TemporalHelpers.assertPlainMonthDay(constrained, monthCode, 30, `day 31 should be constrained to 30 for ${monthCode}`, referenceYear30);
  } else {
    TemporalHelpers.assertPlainMonthDay(pmd30, monthCode, 29, `day 30 should be constrained to 29 for ${monthCode}`, referenceYear);
    TemporalHelpers.assertPlainMonthDay(constrained, monthCode, 29, `day 31 should be constrained to 29 for ${monthCode}`, referenceYear);

    assert.throws(RangeError, () => {
      Temporal.PlainMonthDay.from({ calendar, monthCode, day: 30 }, { overflow: "reject" });
    }, `${monthCode} with day 30 should throw with reject overflow`);
  }

  assert.throws(RangeError, () => {
    Temporal.PlainMonthDay.from({ calendar, monthCode, day: 31 }, { overflow: "reject" });
  }, `${monthCode} with day 31 should throw with reject overflow`);
}

// Leap month
{
  const monthCode = "M05L";
  const referenceYear = 1970;

  const pmd = Temporal.PlainMonthDay.from({ calendar, monthCode, day: 1 });
  TemporalHelpers.assertPlainMonthDay(pmd, monthCode, 1, `${monthCode}-01`, referenceYear);

  const pmd29 = Temporal.PlainMonthDay.from({ calendar, monthCode, day: 29 });
  TemporalHelpers.assertPlainMonthDay(pmd29, monthCode, 29, `${monthCode}-29`, referenceYear);

  const pmd30 = Temporal.PlainMonthDay.from({ calendar, monthCode, day: 30 });
  TemporalHelpers.assertPlainMonthDay(pmd30, monthCode, 30, `${monthCode}-30`, referenceYear);

  const constrained = Temporal.PlainMonthDay.from(
    { calendar, monthCode, day: 31 },
    { overflow: "constrain" }
  );
  // This follows the same rules as Chinese leap months that are known to have
  // 30 days, and therefore constrains to M05L-30, rather than M05-30 as with
  // Chinese leap months that are not certain to have ever had 30 days.
  TemporalHelpers.assertPlainMonthDay(constrained, "M05L", 30, "day 31 of leap month should be constrained to day 30", referenceYear);

  assert.throws(RangeError, () => {
    Temporal.PlainMonthDay.from({ calendar, monthCode, day: 31 }, { overflow: "reject" });
  }, `${monthCode} with day 31 should throw with reject overflow`);
}
